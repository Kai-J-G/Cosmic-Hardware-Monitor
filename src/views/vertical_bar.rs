//! A vertical thermometer bar, filled from the bottom up.

use cosmic::iced::mouse::Cursor;
use cosmic::iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke};
use cosmic::iced::{Color, Length, Point, Rectangle, Size};
use cosmic::{Element, Renderer, Theme};

pub struct VerticalBar {
    fraction: f32,
    fill_color: Color,
    is_dark: bool,
}

impl VerticalBar {
    pub fn new(fraction: f32, fill_color: Color, is_dark: bool) -> Self {
        Self { fraction: fraction.clamp(0.0, 1.0), fill_color, is_dark }
    }

    pub fn view<Message: 'static>(self, width: f32, height: f32) -> Element<'static, Message> {
        Canvas::new(self).width(Length::Fixed(width)).height(Length::Fixed(height)).into()
    }
}

impl<Message> canvas::Program<Message, Theme, Renderer> for VerticalBar {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let (width, height) = (bounds.width, bounds.height);
        // Fully rounded ends, so the bar reads as a capsule.
        let radius = width / 2.0;

        let (track, outline) = if self.is_dark {
            (Color::from_rgba(1.0, 1.0, 1.0, 0.08), Color::from_rgba(1.0, 1.0, 1.0, 0.04))
        } else {
            (Color::from_rgba(0.0, 0.0, 0.0, 0.08), Color::from_rgba(0.0, 0.0, 0.0, 0.06))
        };

        let track_path =
            Path::rounded_rectangle(Point::new(0.0, 0.0), Size::new(width, height), radius.into());
        frame.fill(&track_path, track);
        frame.stroke(&track_path, Stroke::default().with_color(outline).with_width(1.0));

        // Never shorter than one full cap, or the rounding would distort it.
        let fill_height = (height * self.fraction).max(radius * 2.0);
        let fill_top = height - fill_height;
        frame.fill(
            &Path::rounded_rectangle(
                Point::new(0.0, fill_top),
                Size::new(width, fill_height),
                radius.into(),
            ),
            self.fill_color,
        );

        // A highlight on the fill's tip, suggesting the current level.
        frame.fill(
            &Path::circle(Point::new(radius, fill_top + radius), radius * 0.7),
            Color::from_rgba(1.0, 1.0, 1.0, 0.35),
        );

        vec![frame.into_geometry()]
    }
}
