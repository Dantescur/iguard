use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use tokio::sync::{mpsc, oneshot};
use zbus_polkit_agent::{
    Identity, UnixUser,
    agent_session::{Message as SessionMessage, PolkitAgentSession, Response},
    polkit_agent_instance,
    server::Error as PolkitError,
};

use crate::config::{Config, Text};

const OBJECT_PATH: &str = "/org/iguard/PolicyKit1/AuthenticationAgent";

#[derive(Debug, Clone)]
pub struct AuthInfo {
    pub cookie: String,
    pub message: String,
    pub user_name: String,
    pub action_id: String,
    pub details: Vec<(String, String)>,
}

#[derive(Debug)]
pub enum UiResponse {
    Password(String),
    Cancel,
}

pub type ResponderSlot = Arc<Mutex<Option<oneshot::Sender<UiResponse>>>>;

#[derive(Debug, Clone)]
pub enum UiRequest {
    Begin {
        info: AuthInfo,
    },
    Prompt {
        info: AuthInfo,
        prompt: String,
        echo_on: bool,
        notice: Option<String>,
        notice_is_error: bool,
    },
    Notice {
        cookie: String,
        notice: String,
        is_error: bool,
    },
    Close {
        cookie: String,
        success: bool,
        error: Option<String>,
    },
}

#[derive(Clone)]
struct Shared {
    ui: mpsc::UnboundedSender<UiRequest>,
    responses: ResponderSlot,
    cancellations: Arc<Mutex<HashMap<String, oneshot::Sender<()>>>>,
    config: Arc<RwLock<Config>>,
}

pub struct Agent {
    shared: Shared,
}

impl Agent {
    fn new(
        ui: mpsc::UnboundedSender<UiRequest>,
        responses: ResponderSlot,
        config: Arc<RwLock<Config>>,
    ) -> Self {
        Self {
            shared: Shared {
                ui,
                responses,
                cancellations: Arc::new(Mutex::new(HashMap::new())),
                config,
            },
        }
    }
}

pub fn spawn(
    ui: mpsc::UnboundedSender<UiRequest>,
    responses: ResponderSlot,
    config: Arc<RwLock<Config>>,
) {
    let result = std::thread::Builder::new()
        .name("iguard-polkit".to_owned())
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    eprintln!("iguard: could not start the polkit runtime: {error}");
                    return;
                }
            };

            runtime.block_on(async move {
                let builder = polkit_agent_instance(
                    move || Agent::new(ui, responses, config),
                    authenticate,
                    cancel_authentication,
                );

                match builder.connect(OBJECT_PATH).await {
                    Ok(_connection) => std::future::pending::<()>().await,
                    Err(error) => {
                        eprintln!("iguard: could not register the polkit agent: {error}");
                    }
                }
            });
        });

    if let Err(error) = result {
        eprintln!("iguard: could not spawn the polkit thread: {error}");
    }
}

async fn authenticate(
    agent: &mut Agent,
    action_id: &str,
    message: &str,
    _icon_name: &str,
    details: HashMap<&str, &str>,
    cookie: &str,
    identities: Vec<Identity<'_>>,
) -> Result<(), PolkitError> {
    let identity = identities.into_iter().next().ok_or(PolkitError::Failed)?;
    let user: UnixUser = identity.try_into()?;
    let mut session = match PolkitAgentSession::new(user, cookie) {
        Ok(session) => session,
        Err(error) => {
            eprintln!("iguard: could not start an authentication session: {error}");
            return Err(error.into());
        }
    };

    let (max_attempts, text) = agent
        .shared
        .config
        .read()
        .map(|config| (config.behavior.max_attempts(), config.text.clone()))
        .unwrap_or_else(|_| (3, Text::default()));

    let mut details: Vec<_> = details
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect();
    details.sort_by(|left, right| left.0.cmp(&right.0));

    let info = AuthInfo {
        cookie: cookie.to_owned(),
        message: message.to_owned(),
        user_name: session.user_name().to_owned(),
        action_id: action_id.to_owned(),
        details,
    };

    if agent
        .shared
        .ui
        .send(UiRequest::Begin { info: info.clone() })
        .is_err()
    {
        return Err(PolkitError::Cancelled);
    }

    let (cancel_tx, mut cancel_rx) = oneshot::channel();
    agent
        .shared
        .cancellations
        .lock()
        .expect("cancellation map poisoned")
        .insert(cookie.to_owned(), cancel_tx);

    let ui = agent.shared.ui.clone();
    let responses = agent.shared.responses.clone();
    let result = run_session(
        &mut session,
        info,
        ui,
        responses,
        max_attempts,
        text,
        &mut cancel_rx,
    )
    .await;

    agent
        .shared
        .cancellations
        .lock()
        .expect("cancellation map poisoned")
        .remove(cookie);

    result
}

async fn cancel_authentication(agent: &mut Agent, cookie: &str) -> Result<(), PolkitError> {
    let cancellation = agent
        .shared
        .cancellations
        .lock()
        .expect("cancellation map poisoned")
        .remove(cookie);

    if let Some(cancellation) = cancellation {
        let _ = cancellation.send(());
    }

    let cancelled = agent
        .shared
        .config
        .read()
        .map(|config| config.text.cancelled.clone())
        .unwrap_or_else(|_| Text::default().cancelled);

    send_close(&agent.shared.ui, cookie, false, Some(cancelled));

    Ok(())
}

async fn run_session(
    session: &mut PolkitAgentSession,
    info: AuthInfo,
    ui: mpsc::UnboundedSender<UiRequest>,
    responses: ResponderSlot,
    max_attempts: usize,
    text: Text,
    cancel: &mut oneshot::Receiver<()>,
) -> Result<(), PolkitError> {
    let mut notice = None;
    let mut attempts = 0;

    loop {
        let mut prompt = None;

        loop {
            let message = tokio::select! {
                biased;

                _ = &mut *cancel => {
                    send_close(
                        &ui,
                        &info.cookie,
                        false,
                        Some(text.cancelled.clone()),
                    );

                    return Err(PolkitError::Cancelled);
                }
                message = session.async_dispatch() => message.map_err(PolkitError::from)?,
            };

            match message {
                SessionMessage::Request {
                    echo_on,
                    prompt: prompt_text,
                } => {
                    prompt = Some((echo_on, prompt_text));
                    break;
                }
                SessionMessage::Info(text) => {
                    let _ = ui.send(UiRequest::Notice {
                        cookie: info.cookie.clone(),
                        notice: text.clone(),
                        is_error: false,
                    });
                    notice = Some((text, false));
                }
                SessionMessage::Error(text) => {
                    let _ = ui.send(UiRequest::Notice {
                        cookie: info.cookie.clone(),
                        notice: text.clone(),
                        is_error: true,
                    });
                    notice = Some((text, true));
                }
                SessionMessage::Complete(_) => break,
            }
        }

        if session.is_complete() {
            if session.succeeded() {
                send_close(&ui, &info.cookie, true, None);
                return Ok(());
            }

            attempts += 1;

            if attempts >= max_attempts {
                let error = notice
                    .take()
                    .map(|(text, _)| text)
                    .or_else(|| Some(text.failed.clone()));

                send_close(&ui, &info.cookie, false, error);
                return Err(PolkitError::Failed);
            }

            if let Err(error) = session.restart() {
                send_close(&ui, &info.cookie, false, Some(error.to_string()));
                return Err(PolkitError::from(error));
            }

            notice = Some((text.retry.clone(), true));
            continue;
        }

        let Some((echo_on, prompt_text)) = prompt else {
            continue;
        };

        let (responder, response) = oneshot::channel();
        *responses.lock().expect("responder slot poisoned") = Some(responder);

        let (notice_text, notice_is_error) = match notice.take() {
            Some((text, is_error)) => (Some(text), is_error),
            None => (None, false),
        };

        if ui
            .send(UiRequest::Prompt {
                info: info.clone(),
                prompt: prompt_text,
                echo_on,
                notice: notice_text,
                notice_is_error,
            })
            .is_err()
        {
            return Err(PolkitError::Cancelled);
        }

        let response = tokio::select! {
            biased;

            _ = &mut *cancel => {
                send_close(
                    &ui,
                    &info.cookie,
                    false,
                    Some(text.cancelled.clone()),
                );

                return Err(PolkitError::Cancelled);
            }
            response = response => response,
        };

        match response {
            Ok(UiResponse::Password(password)) => {
                session
                    .response(Response {
                        password: &password,
                    })
                    .map_err(PolkitError::from)?;
            }
            _ => {
                send_close(&ui, &info.cookie, false, Some(text.cancelled.clone()));

                return Err(PolkitError::Cancelled);
            }
        }
    }
}

fn send_close(
    ui: &mpsc::UnboundedSender<UiRequest>,
    cookie: &str,
    success: bool,
    error: Option<String>,
) {
    let _ = ui.send(UiRequest::Close {
        cookie: cookie.to_owned(),
        success,
        error,
    });
}
