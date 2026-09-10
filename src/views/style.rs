//! The popup's visual language: one definition per surface, tinted for the
//! active light or dark theme.
//!
//! Every card, badge and divider draws its colours from here so the whole
//! popup stays consistent, and a change of palette is a change in one place.

use cosmic::iced::{Border, Color, Length};
use cosmic::theme;
use cosmic::widget::{container, Space};
use cosmic::Element;

/// Corner radius shared by every card-sized surface.
const CARD_RADIUS: f32 = 12.0;

/// The desktop's accent colour.
///
/// Used for readings that carry no meaning of their own — the dials, the
/// throughput cards, the network dot — so the applet picks up whatever accent
/// the user has chosen in COSMIC™ Settings rather than imposing its own brand
/// colour. Readings that *do* carry meaning, namely temperatures, keep the
/// thermal palette in [`crate::hardware::types::ThermalStatus`]; recolouring
/// those would throw away what the colour is telling you.
pub fn accent() -> Color {
    let accent = cosmic::theme::active().cosmic().accent_color();

    Color::from_rgb(accent.red, accent.green, accent.blue)
}

/// Colour that reads as a faint lift off the background in either theme:
/// white over dark, black over light.
fn overlay(is_dark: bool, alpha: f32) -> Color {
    if is_dark {
        Color::from_rgba(1.0, 1.0, 1.0, alpha)
    } else {
        Color::from_rgba(0.0, 0.0, 0.0, alpha)
    }
}

/// Builds a container style. A fully transparent border draws no outline.
fn surface(background: Color, border: Color, radius: f32) -> theme::Container<'static> {
    let width = if border.a > 0.0 { 1.0 } else { 0.0 };
    theme::Container::Custom(Box::new(move |_theme| cosmic::iced::widget::container::Style {
        background: Some(background.into()),
        border: Border { color: border, width, radius: radius.into() },
        ..Default::default()
    }))
}

/// The default panel behind a group of readings.
pub fn card(is_dark: bool) -> theme::Container<'static> {
    surface(overlay(is_dark, 0.035), overlay(is_dark, 0.07), CARD_RADIUS)
}

/// A card tinted with an accent, used to highlight the reading that matters
/// most in a row — typically the temperature.
pub fn accent_card(accent: Color, is_dark: bool) -> theme::Container<'static> {
    let tint = |alpha| Color::from_rgba(accent.r, accent.g, accent.b, alpha);
    let (fill, edge) = if is_dark { (0.12, 0.35) } else { (0.08, 0.25) };
    surface(tint(fill), tint(edge), CARD_RADIUS)
}

/// A small pill behind a short label, such as a filesystem type.
pub fn chip(is_dark: bool) -> theme::Container<'static> {
    surface(overlay(is_dark, 0.06), Color::TRANSPARENT, 6.0)
}

/// A pill tinted by thermal state, used for inline temperature readings.
pub fn accent_chip(accent: Color) -> theme::Container<'static> {
    let tint = |alpha| Color::from_rgba(accent.r, accent.g, accent.b, alpha);
    surface(tint(0.12), tint(0.35), 6.0)
}

/// A hairline rule separating sections of the overview.
pub fn divider<'a, M: 'static>(is_dark: bool) -> Element<'a, M> {
    let color = overlay(is_dark, if is_dark { 0.07 } else { 0.08 });
    container(Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .class(surface(color, Color::TRANSPARENT, 0.0))
        .into()
}

/// A small filled square or circle used as a legend marker.
pub fn swatch<'a, M: 'static>(color: Color, size: f32, radius: f32) -> Element<'a, M> {
    container(Space::new())
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .class(surface(color, Color::TRANSPARENT, radius))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guards the wiring: the accent must come from the active theme, so that
    /// pinning light or dark in settings, or the desktop changing its accent,
    /// is reflected. A reintroduced constant would fail this.
    #[test]
    fn accent_tracks_the_active_theme() {
        let theme = cosmic::theme::active();
        let expected = theme.cosmic().accent_color();
        let actual = accent();

        assert!((actual.r - expected.red).abs() < f32::EPSILON);
        assert!((actual.g - expected.green).abs() < f32::EPSILON);
        assert!((actual.b - expected.blue).abs() < f32::EPSILON);

        for channel in [actual.r, actual.g, actual.b] {
            assert!((0.0..=1.0).contains(&channel), "channel out of range: {channel}");
        }
    }
}
