use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};

use iced::{
    Element, Length, Subscription, Task,
    keyboard::{self, Key, key::Named},
    time,
    widget::{Id, button, column, container, operation, row, text, text_input},
    window,
};
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    agent::{self, AuthInfo, UiRequest, UiResponse},
    config::{self, Config},
};

fn password_id() -> Id {
    Id::new("iguard-password")
}

pub struct App {
    window: Option<window::Id>,
    dialog: Option<Dialog>,
    responses: agent::ResponderSlot,
    config: Config,
    shared_config: Arc<RwLock<Config>>,
    config_path: PathBuf,
    config_modified: Option<SystemTime>,
    theme: iced::Theme,
}

impl App {
    pub fn window_title(&self) -> String {
        self.config.text.window_title.clone()
    }

    pub fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }
}

struct Dialog {
    info: AuthInfo,
    prompt: Option<String>,
    echo_on: bool,
    password: String,
    notice: Option<String>,
    notice_is_error: bool,
    busy: bool,
    responder: Option<oneshot::Sender<UiResponse>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Agent(UiRequest),
    PasswordChanged(String),
    Submit,
    Cancel,
    Escape,
    Tick,
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    CloseRequested(window::Id),
}

pub fn boot() -> (App, Task<Message>) {
    let (ui_tx, ui_rx) = mpsc::unbounded_channel();
    let responses = agent::ResponderSlot::default();
    let (config, config_modified) = config::load();
    let config_path = config::path();
    let shared_config = Arc::new(RwLock::new(config.clone()));
    let theme = config.appearance.theme();
    agent::spawn(ui_tx, responses.clone(), shared_config.clone());

    let requests = UnboundedReceiverStream::new(ui_rx);

    (
        App {
            window: None,
            dialog: None,
            responses,
            config,
            shared_config,
            config_path,
            config_modified,
            theme,
        },
        Task::stream(requests).map(Message::Agent),
    )
}

pub fn subscription(_state: &App) -> Subscription<Message> {
    Subscription::batch([
        window::close_requests().map(Message::CloseRequested),
        window::close_events().map(Message::WindowClosed),
        time::every(Duration::from_secs(1)).map(|_| Message::Tick),
        keyboard::listen().filter_map(|event| match event {
            keyboard::Event::KeyPressed {
                key: Key::Named(Named::Escape),
                ..
            } => Some(Message::Escape),
            _ => None,
        }),
    ])
}

pub fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::Agent(request) => handle_agent(app, request),
        Message::PasswordChanged(password) => {
            if let Some(dialog) = app.dialog.as_mut()
                && !dialog.busy
            {
                dialog.password = password;
            }

            Task::none()
        }
        Message::Submit => submit(app),
        Message::Cancel => cancel(app),
        Message::Escape => {
            if app.config.behavior.close_on_escape && app.dialog.is_some() {
                cancel(app)
            } else {
                Task::none()
            }
        }
        Message::Tick => reload_config(app),
        Message::WindowOpened(id) => {
            if app.window == Some(id) && app.dialog.is_some() {
                focus_window(app, id)
            } else {
                Task::none()
            }
        }
        Message::WindowClosed(id) => {
            if app.window != Some(id) {
                return Task::none();
            }

            app.window = None;
            cancel_dialog(app);

            Task::none()
        }
        Message::CloseRequested(id) => {
            if app.window == Some(id) {
                cancel(app)
            } else {
                Task::none()
            }
        }
    }
}

pub fn view(app: &App, id: window::Id) -> Element<'_, Message> {
    if app.window != Some(id) {
        return container("").into();
    }

    let Some(dialog) = app.dialog.as_ref() else {
        return container("").into();
    };

    let config = &app.config;
    let typography = &config.typography;
    let font = typography.font();
    let danger = app.theme.palette().danger;

    let status: Element<'_, Message> = match (&dialog.notice, dialog.busy) {
        (Some(notice), _) if dialog.notice_is_error => text(notice)
            .size(typography.small_size)
            .font(font)
            .color(danger)
            .into(),
        (Some(notice), _) => text(notice).size(typography.small_size).font(font).into(),
        (None, true) => text(&config.text.authenticating)
            .size(typography.small_size)
            .font(font)
            .into(),
        (None, false) => text("").size(typography.small_size).font(font).into(),
    };

    let can_submit = !dialog.busy && dialog.prompt.is_some() && !dialog.password.is_empty();

    let authenticate = button(
        text(&config.text.submit)
            .size(typography.body_size)
            .font(font),
    )
    .style(button::primary)
    .padding([typography.button_padding_y, typography.button_padding_x])
    .width(Length::Fill)
    .on_press_maybe(can_submit.then_some(Message::Submit));

    let cancel = button(
        text(&config.text.cancel)
            .size(typography.body_size)
            .font(font),
    )
    .padding([typography.button_padding_y, typography.button_padding_x])
    .width(Length::Fill)
    .on_press(Message::Cancel);

    let mut content = column![
        text(&config.text.title)
            .size(typography.title_size)
            .font(font),
        text(&dialog.info.message)
            .size(typography.body_size)
            .font(font),
        text(format!(
            "{}: {}",
            config.text.identity, dialog.info.user_name
        ))
        .size(typography.small_size)
        .font(font),
    ]
    .spacing(typography.spacing);

    if config.behavior.show_action_id && !dialog.info.action_id.is_empty() {
        content = content.push(
            text(format!("{}: {}", config.text.action, dialog.info.action_id))
                .size(typography.small_size)
                .font(font),
        );
    }

    if config.behavior.show_details && !dialog.info.details.is_empty() {
        content = content.push(
            text(&config.text.details)
                .size(typography.small_size)
                .font(font),
        );

        for (key, value) in &dialog.info.details {
            content = content.push(
                text(format!("{key}: {value}"))
                    .size(typography.small_size)
                    .font(font),
            );
        }
    }

    if let Some(prompt) = &dialog.prompt {
        content = content.push(text(prompt).size(typography.body_size).font(font));

        let mut password = text_input(&config.text.password, &dialog.password)
            .id(password_id())
            .padding(typography.input_padding)
            .size(typography.input_size)
            .font(font)
            .secure(!dialog.echo_on)
            .width(Length::Fill);

        if !dialog.busy {
            password = password
                .on_input(Message::PasswordChanged)
                .on_submit_maybe(config.behavior.submit_on_enter.then_some(Message::Submit));
        }

        content = content.push(password);
    } else {
        content = content.push(
            text(&config.text.waiting)
                .size(typography.body_size)
                .font(font),
        );
    }

    content = content
        .push(status)
        .push(row![cancel, authenticate].spacing(typography.spacing));

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(typography.padding)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn handle_agent(app: &mut App, request: UiRequest) -> Task<Message> {
    match request {
        UiRequest::Begin { info } => {
            cancel_dialog(app);
            let close_previous = close_window(app);

            app.dialog = Some(Dialog {
                info,
                prompt: None,
                echo_on: false,
                password: String::new(),
                notice: None,
                notice_is_error: false,
                busy: false,
                responder: None,
            });

            Task::batch([close_previous, open_window(app)])
        }
        UiRequest::Prompt {
            info,
            prompt,
            echo_on,
            notice,
            notice_is_error,
        } => {
            let responder = app
                .responses
                .lock()
                .expect("responder slot poisoned")
                .take();

            if let Some(dialog) = app
                .dialog
                .as_mut()
                .filter(|dialog| dialog.info.cookie == info.cookie)
            {
                dialog.prompt = Some(prompt);
                dialog.echo_on = echo_on;
                dialog.password.clear();
                dialog.notice = notice;
                dialog.notice_is_error = notice_is_error;
                dialog.busy = false;
                dialog.responder = responder;
            } else {
                cancel_dialog(app);

                app.dialog = Some(Dialog {
                    info,
                    prompt: Some(prompt),
                    echo_on,
                    password: String::new(),
                    notice,
                    notice_is_error,
                    busy: false,
                    responder,
                });
            }

            focus_or_open_window(app)
        }
        UiRequest::Notice {
            cookie,
            notice,
            is_error,
        } => {
            if let Some(dialog) = app.dialog.as_mut()
                && dialog.info.cookie == cookie
            {
                dialog.notice = Some(notice);
                dialog.notice_is_error = is_error;
            }

            Task::none()
        }
        UiRequest::Close {
            cookie,
            success,
            error,
        } => {
            let is_current = app
                .dialog
                .as_ref()
                .is_some_and(|dialog| dialog.info.cookie == cookie);

            if !is_current {
                return Task::none();
            }

            app.dialog = None;

            if !success && let Some(error) = error {
                eprintln!("iguard: authentication failed: {error}");
            }

            close_window(app)
        }
    }
}

fn cancel_dialog(app: &mut App) {
    if let Some(mut dialog) = app.dialog.take()
        && let Some(responder) = dialog.responder.take()
    {
        let _ = responder.send(UiResponse::Cancel);
    }
}

fn submit(app: &mut App) -> Task<Message> {
    let Some(dialog) = app.dialog.as_mut() else {
        return Task::none();
    };

    if dialog.busy || dialog.prompt.is_none() || dialog.password.is_empty() {
        return Task::none();
    }

    let Some(responder) = dialog.responder.take() else {
        return Task::none();
    };

    let password = std::mem::take(&mut dialog.password);
    dialog.busy = true;
    dialog.notice = None;
    dialog.notice_is_error = false;

    if responder.send(UiResponse::Password(password)).is_err() {
        app.dialog = None;
        return close_window(app);
    }

    Task::none()
}

fn cancel(app: &mut App) -> Task<Message> {
    cancel_dialog(app);
    close_window(app)
}

fn open_window(app: &mut App) -> Task<Message> {
    let (id, open) = window::open(app.config.window.settings());
    app.window = Some(id);

    Task::batch([open.map(Message::WindowOpened), focus_window(app, id)])
}

fn focus_or_open_window(app: &mut App) -> Task<Message> {
    if let Some(id) = app.window {
        focus_window(app, id)
    } else {
        open_window(app)
    }
}

fn focus_window(app: &App, id: window::Id) -> Task<Message> {
    let mut tasks = vec![window::gain_focus(id)];

    if app.config.behavior.autofocus {
        tasks.push(operation::focus(password_id()));
    }

    Task::batch(tasks)
}

fn close_window(app: &mut App) -> Task<Message> {
    if let Some(id) = app.window.take() {
        window::close(id)
    } else {
        Task::none()
    }
}

fn reload_config(app: &mut App) -> Task<Message> {
    let modified = config::modified(&app.config_path);

    if modified == app.config_modified {
        return Task::none();
    }

    app.config_modified = modified;

    if modified.is_none() {
        return Task::none();
    }

    let Some(config) = config::load_from(&app.config_path) else {
        return Task::none();
    };

    let window_changed = app.config.window != config.window;
    app.config = config.clone();

    if let Ok(mut shared) = app.shared_config.write() {
        *shared = config.clone();
    }

    app.theme = config.appearance.theme();

    if window_changed {
        apply_window_config(app)
    } else {
        Task::none()
    }
}

fn apply_window_config(app: &App) -> Task<Message> {
    let Some(id) = app.window else {
        return Task::none();
    };

    let mut tasks = vec![window::resize(id, app.config.window.size())];

    if let window::Position::Specific(position) = app.config.window.position() {
        tasks.push(window::move_to(id, position));
    }

    Task::batch(tasks)
}
