mod app;
mod config;
mod hardware;
mod sparkline;
mod views;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<app::AppModel>(())
}
