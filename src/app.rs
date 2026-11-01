use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};

use iced::{
    Background, Border, Color, Element, Length, Shadow, Subscription, Task,
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
    responses: agent::ResponderMap,
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
    attempt: usize,
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
    let responses = agent::ResponderMap::default();
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
    let palette = app.theme.palette();
    let ink = Color::from_rgb(0.90, 0.93, 0.97);
    let muted = Color::from_rgb(0.58, 0.65, 0.74);
    let surface = Color::from_rgb(0.075, 0.095, 0.13);
    let inset = Color::from_rgb(0.045, 0.06, 0.085);

    let status: Element<'_, Message> = match (&dialog.notice, dialog.busy) {
        (Some(notice), _) if dialog.notice_is_error => text(notice)
            .size(typography.small_size)
            .font(font)
            .color(palette.danger)
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
    .style(move |_, status| button::Style {
        background: Some(Background::Color(if can_submit {
            palette.primary
        } else {
            Color::from_rgb(0.22, 0.28, 0.36)
        })),
        text_color: if can_submit { Color::WHITE } else { muted },
        border: Border {
            radius: 9.0.into(),
            ..Border::default()
        },
        ..match status {
            button::Status::Hovered => button::Style {
                shadow: Shadow {
                    color: palette.primary.scale_alpha(0.22),
                    blur_radius: 12.0,
                    ..Shadow::default()
                },
                ..button::Style::default()
            },
            _ => button::Style::default(),
        }
    })
    .padding([
        typography.button_padding_y + 2.0,
        typography.button_padding_x,
    ])
    .width(Length::Fill)
    .on_press_maybe(can_submit.then_some(Message::Submit));

    let cancel = button(
        text(&config.text.cancel)
            .size(typography.body_size)
            .font(font),
    )
    .style(move |_, status| button::Style {
        background: Some(Background::Color(
            if matches!(status, button::Status::Hovered) {
                Color::from_rgb(0.16, 0.19, 0.25)
            } else {
                surface
            },
        )),
        text_color: ink,
        border: Border {
            radius: 9.0.into(),
            color: Color::from_rgb(0.20, 0.25, 0.32),
            width: 1.0,
        },
        ..button::Style::default()
    })
    .padding([
        typography.button_padding_y + 2.0,
        typography.button_padding_x,
    ])
    .width(Length::Fill)
    .on_press(Message::Cancel);

    let header = container(
        row![
            text("▣").size(14).color(palette.primary),
            text("org.freedesktop.PolicyKit1")
                .size(10)
                .font(typography.font())
                .color(muted),
            iced::widget::Space::new().width(Length::Fill),
            text("●").size(8).color(palette.success),
        ]
        .spacing(9)
        .align_y(iced::Alignment::Center),
    )
    .padding([13, 18])
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(inset)),
        ..container::Style::default()
    });

    let mut content = column![
        text("AUTHORIZATION REQUEST")
            .size(10)
            .font(typography.font())
            .color(palette.primary),
        text(&config.text.title)
            .size(typography.title_size + 4.0)
            .font(font)
            .color(ink),
        text(&dialog.info.message)
            .size(typography.body_size)
            .font(font)
            .color(muted),
    ]
    .spacing(8);

    if config.behavior.show_action_id && !dialog.info.action_id.is_empty() {
        content = content.push(
            container(
                text(&dialog.info.action_id)
                    .size(typography.small_size)
                    .font(typography.font())
                    .color(muted),
            )
            .padding([7, 9])
            .width(Length::Fill)
            .style(move |_| container::Style {
                background: Some(Background::Color(inset)),
                border: Border {
                    radius: 5.0.into(),
                    color: Color::from_rgb(0.16, 0.21, 0.28),
                    width: 1.0,
                },
                ..container::Style::default()
            }),
        );
    }

    if config.behavior.show_details && !dialog.info.details.is_empty() {
        content = content.push(
            text(&config.text.details)
                .size(typography.small_size)
                .font(font)
                .color(muted),
        );

        for (key, value) in &dialog.info.details {
            content = content.push(
                text(format!("{key}: {value}"))
                    .size(typography.small_size)
                    .font(font)
                    .color(muted),
            );
        }
    }

    if let Some(prompt) = &dialog.prompt {
        content = content.push(
            text(prompt)
                .size(typography.small_size)
                .font(font)
                .color(muted),
        );

        let mut password = text_input(&config.text.password, &dialog.password)
            .id(password_id())
            .padding(typography.input_padding)
            .size(typography.input_size)
            .font(font)
            .secure(!dialog.echo_on)
            .width(Length::Fill);

        password = password.style(move |_, status| text_input::Style {
            background: Background::Color(inset),
            border: Border {
                radius: 9.0.into(),
                color: if matches!(status, text_input::Status::Focused { .. }) {
                    palette.primary
                } else {
                    Color::from_rgb(0.20, 0.25, 0.32)
                },
                width: 1.0,
            },
            icon: muted,
            placeholder: muted,
            value: ink,
            selection: palette.primary,
        });

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

    let actions = row![
        iced::widget::Space::new().width(Length::Fill),
        cancel.width(Length::Shrink),
        authenticate.width(Length::Shrink)
    ]
    .spacing(8);
    content = content.push(status).push(actions);

    let footer = container(
        row![
            text("●").size(8).color(palette.success),
            text(format!("identity: {}", dialog.info.user_name))
                .size(9)
                .font(typography.font())
                .color(muted),
            iced::widget::Space::new().width(Length::Fill),
            text(format!(
                "attempt {} of {}",
                dialog.attempt,
                config.behavior.max_attempts()
            ))
            .size(9)
            .font(typography.font())
            .color(muted),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    )
    .padding([10, 16])
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(inset)),
        ..container::Style::default()
    });

    let card = container(column![
        header,
        container(content).padding([22, 22]),
        footer
    ])
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(surface)),
        border: Border {
            radius: 14.0.into(),
            color: Color::from_rgb(0.17, 0.23, 0.31),
            width: 1.0,
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.38),
            blur_radius: 28.0,
            ..Shadow::default()
        },
        ..container::Style::default()
    });

    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(typography.padding)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.025, 0.035, 0.052))),
            ..container::Style::default()
        })
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
                attempt: 1,
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
                .ok()
                .and_then(|mut responders| responders.remove(&info.cookie));
            let Some(responder) = responder else {
                return Task::none();
            };

            if let Some(dialog) = app
                .dialog
                .as_mut()
                .filter(|dialog| dialog.info.cookie == info.cookie)
            {
                dialog.prompt = Some(prompt);
                dialog.echo_on = echo_on;
                dialog.password.clear();
                dialog.notice = notice;
                if notice_is_error {
                    dialog.attempt = dialog.attempt.saturating_add(1);
                }
                dialog.notice_is_error = notice_is_error;
                dialog.busy = false;
                dialog.responder = Some(responder);
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
                    attempt: 1,
                    responder: Some(responder),
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

    let previous_window = app.config.window.clone();
    let window_changed = previous_window != config.window;
    app.config = config.clone();

    if let Ok(mut shared) = app.shared_config.write() {
        *shared = config.clone();
    }

    app.theme = config.appearance.theme();

    if window_changed {
        apply_window_config(app, &previous_window)
    } else {
        Task::none()
    }
}

fn apply_window_config(app: &mut App, previous: &config::WindowConfig) -> Task<Message> {
    let Some(id) = app.window else {
        return Task::none();
    };

    let current = &app.config.window;
    let requires_reopen = previous.placement != current.placement
        || previous.closeable != current.closeable
        || previous.minimizable != current.minimizable;

    if requires_reopen {
        let (new_id, open) = window::open(current.settings());
        app.window = Some(new_id);

        return window::close::<Message>(id)
            .chain(open.map(Message::WindowOpened))
            .chain(focus_window(app, new_id));
    }

    let mut tasks = Vec::new();

    if previous.size() != current.size() {
        tasks.push(window::resize(id, current.size()));
    }

    if previous.resizable != current.resizable {
        tasks.push(window::set_resizable(id, current.resizable));
    }

    if previous.decorations != current.decorations {
        tasks.push(window::toggle_decorations(id));
    }

    if previous.always_on_top != current.always_on_top {
        tasks.push(window::set_level(
            id,
            if current.always_on_top {
                window::Level::AlwaysOnTop
            } else {
                window::Level::Normal
            },
        ));
    }

    if (previous.x != current.x || previous.y != current.y)
        && current.placement == config::Placement::Fixed
    {
        if let window::Position::Specific(position) = current.position() {
            tasks.push(window::move_to(id, position));
        }
    }

    Task::batch(tasks)
}
