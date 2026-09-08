//! TempTyle — a hardware and thermal monitor applet for the COSMIC desktop.

mod app;
mod config;
mod hardware;
mod views;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<app::AppModel>(())
}
