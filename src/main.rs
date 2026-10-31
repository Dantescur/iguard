mod agent;
mod app;
mod config;

fn main() -> iced::Result {
    iced::daemon(app::boot, app::update, app::view)
        .title(|app: &app::App, _| app.window_title())
        .theme(|app: &app::App, _| app.theme())
        .subscription(app::subscription)
        .run()
}
