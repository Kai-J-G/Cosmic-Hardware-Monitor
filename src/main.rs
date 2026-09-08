//! TempTyle — a hardware and thermal monitor applet for the COSMIC desktop.
//!
//! # How it fits together
//!
//! The applet is an [Elm-architecture] program, which libcosmic supplies: state
//! lives in one struct, messages describe what happened, and the UI is a
//! function of the state. Nothing here runs on a background thread.
//!
//! ```text
//!            every N seconds
//!                  │
//!                  ▼
//!   ┌───────────────────────────┐    reads /sys and /proc
//!   │  hardware::               │◄──────────────────────────  the machine
//!   │  HardwareCollector        │
//!   └───────────────────────────┘
//!                  │  HardwareSnapshot
//!                  ▼
//!   ┌───────────────────────────┐
//!   │  app::AppModel            │  the snapshot, chart history,
//!   │                           │  which tab is open, settings
//!   └───────────────────────────┘
//!                  │  &AppModel
//!                  ▼
//!   ┌───────────────────────────┐
//!   │  views::                  │  panel button, popup, tabs
//!   └───────────────────────────┘
//!                  │  Message
//!                  └──────────────►  back to app::AppModel::update
//! ```
//!
//! # Where to make a change
//!
//! - **Read something new from the hardware** — add a field to the relevant
//!   struct in [`hardware::types`], fill it in that subsystem's collector, then
//!   show it in the matching view.
//! - **Change how the popup looks** — the tab views are in [`views`], and the
//!   card and badge styling they share is in [`views::style`].
//! - **Add a setting** — extend [`config::TempTyleConfig`], add a
//!   [`app::Message`] variant to change it, and add a control to
//!   [`views::settings`].
//!
//! [Elm-architecture]: https://guide.elm-lang.org/architecture/

mod app;
mod config;
mod hardware;
mod views;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<app::AppModel>(())
}
