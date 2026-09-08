//! The panel button and the popup surface it opens.
//!
//! This is the plumbing between the applet and the COSMIC shell: the icon that
//! lives in the panel, and the positioning of the popup relative to it.

use std::sync::LazyLock;

use cosmic::app::Task;
use cosmic::applet::cosmic_panel_config::PanelAnchor;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::window::Id;
use cosmic::iced::{Alignment, Border, Color, Length, Limits, Rectangle, Shadow};
use cosmic::surface::action::{app_popup, destroy_popup, LiveSettings};
use cosmic::widget::{autosize, button, column, container, icon, row, text};
use cosmic::Element;

use crate::app::{AppModel, Message};
use crate::views;

static PANEL_ID: LazyLock<cosmic::widget::Id> =
    LazyLock::new(|| cosmic::widget::Id::new("autosize-main"));
static POPUP_ID: LazyLock<cosmic::widget::Id> =
    LazyLock::new(|| cosmic::widget::Id::new("cosmic-applet-autosize"));

/// Bounds the shell allows the popup to occupy.
const MIN_WIDTH: f32 = 400.0;
const MAX_WIDTH: f32 = 780.0;
const MAX_HEIGHT: f32 = 1400.0;

/// A thermometer glyph, inlined so it can be recoloured per thermal state.
fn thermometer_svg(color: &str) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="34" height="34" viewBox="0 0 16 16" fill="{color}">
  <path d="M10 5a2 2 0 1 0-4 0v4.2a3 3 0 1 0 4 0V5zm-2-1a1 1 0 0 1 1 1v4.5a.5.5 0 0 0 .2.4 2 2 0 1 1-2.4 0 .5.5 0 0 0 .2-.4V5a1 1 0 0 1 1-1zm0 7a1 1 0 1 0 0 2 1 1 0 0 0 0-2z"/>
</svg>"##
    )
}

/// The applet's panel entry: thermometer icon plus the current temperature.
pub fn view(app: &AppModel) -> Element<'_, Message> {
    let sp = cosmic::theme::spacing();
    let [r, g, b] = app.snapshot.status().rgb();
    let hex = format!("#{:02x}{:02x}{:02x}", (r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);

    let size = app.core.applet.suggested_size(true);
    let thermometer =
        icon::icon(icon::from_svg_bytes(thermometer_svg(&hex).into_bytes())).size(size.0);
    let label = text::body(app.unit.format_short(app.snapshot.cpu.package_temp)).size(13);

    // Stack the readout under the icon when the panel runs vertically.
    let horizontal = app.core.applet.is_horizontal();
    let content: Element<'_, Message> = if horizontal {
        row![thermometer, label].spacing(sp.space_xxxs).align_y(Alignment::Center).into()
    } else {
        column![thermometer, label].spacing(2).align_x(Alignment::Center).into()
    };

    let padding = app.core.applet.suggested_padding(true).0;
    let is_open = app.popup.is_some();

    let button = button::custom(content)
        .padding(if horizontal { [0, padding] } else { [padding, 0] })
        .class(cosmic::theme::Button::AppletIcon)
        .on_press_with_rectangle(move |offset, bounds| {
            if is_open {
                Message::ClosePopup
            } else {
                Message::Surface(open_popup(offset, bounds))
            }
        });

    autosize::autosize(button, PANEL_ID.clone()).into()
}

/// Builds the request that opens the popup anchored to the panel button.
fn open_popup(offset: cosmic::iced::Vector, bounds: Rectangle) -> cosmic::surface::Action {
    app_popup::<AppModel>(
        |_| LiveSettings { blur: Some(true), ..Default::default() },
        move |app: &mut AppModel| {
            let id = Id::unique();
            app.popup = Some(id);

            let width = views::popup_width(app.active_tab);
            let mut settings = app.core.applet.get_popup_settings(
                app.core.main_window_id().expect("applet always has a main window"),
                id,
                Some((width as u32, 600)),
                None,
                None,
            );

            settings.positioner.size_limits = Limits::NONE
                .min_height(1.0)
                .min_width(MIN_WIDTH)
                .max_width(MAX_WIDTH)
                .max_height(MAX_HEIGHT);
            settings.positioner.anchor_rect = Rectangle {
                x: (bounds.x - offset.x) as i32,
                y: (bounds.y - offset.y) as i32,
                width: bounds.width as i32,
                height: bounds.height as i32,
            };
            settings.positioner.offset = centering_offset(app, settings.positioner.offset, bounds);
            settings
        },
        None,
    )
}

/// Nudges the popup so it clears the whole panel slot, not just the button.
///
/// The popup is anchored to the button's rectangle, but a button can be
/// shorter than the slot the panel reserves for it. When that happens the
/// popup would overlap the panel, so it is pushed out by half the difference.
///
/// The panel supplies an offset already pointing away from its edge — negative
/// above or left of it, positive below or right — so multiplying by the sign
/// moves the popup further out whichever edge the panel is docked to.
fn centering_offset(app: &AppModel, offset: (i32, i32), bounds: Rectangle) -> (i32, i32) {
    let (icon_width, icon_height) = app.core.applet.suggested_size(true);
    let (_, padding_across) = app.core.applet.suggested_padding(true);

    // Measure across the panel: its height when horizontal, width when not.
    let (slot, button) = if app.core.applet.is_horizontal() {
        (f32::from(icon_height + 2 * padding_across), bounds.height)
    } else {
        (f32::from(icon_width + 2 * padding_across), bounds.width)
    };

    let shortfall = ((slot - button) / 2.0).max(0.0).round() as i32;
    let (x, y) = offset;

    (x + x.signum() * shortfall, y + y.signum() * shortfall)
}

/// Wraps the popup content in the shell's rounded, themed surface.
pub fn popup_container<'a>(
    app: &AppModel,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    // Grow away from the panel edge the applet is docked to.
    let (align_y, align_x) = match app.core.applet.anchor {
        PanelAnchor::Left => (Vertical::Center, Horizontal::Left),
        PanelAnchor::Right => (Vertical::Center, Horizontal::Right),
        PanelAnchor::Top => (Vertical::Top, Horizontal::Center),
        PanelAnchor::Bottom => (Vertical::Bottom, Horizontal::Center),
    };

    let surface = container(content).style(|theme| {
        let cosmic = theme.cosmic();
        let background = cosmic.background(true);
        cosmic::iced::widget::container::Style {
            text_color: Some(background.on.into()),
            background: Some(Color::from(background.base).into()),
            border: Border {
                radius: cosmic.corner_radii.radius_m.into(),
                width: 1.0,
                color: background.divider.into(),
            },
            shadow: Shadow::default(),
            icon_color: Some(background.on.into()),
            snap: true,
        }
    });

    let width = views::popup_width(app.active_tab);
    autosize::autosize(
        container(surface).height(Length::Shrink).align_x(align_x).align_y(align_y),
        POPUP_ID.clone(),
    )
    .limits(
        Limits::NONE.min_height(1.0).min_width(width).max_width(width).max_height(MAX_HEIGHT),
    )
    .into()
}

/// Asks the shell to tear down the popup surface.
pub fn destroy(id: Id) -> Task<Message> {
    cosmic::task::message(cosmic::Action::Cosmic(cosmic::app::Action::Surface(destroy_popup(id))))
}
