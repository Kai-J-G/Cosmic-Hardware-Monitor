//! A ring gauge: a percentage drawn as an arc, with the value in the middle.

use std::f32::consts::{FRAC_PI_2, TAU};

use cosmic::iced::mouse::Cursor;
use cosmic::iced::widget::canvas::{self, path::Arc, Canvas, Frame, Geometry, LineCap, Path, Stroke, Text};
use cosmic::iced::widget::text::{LineHeight, Shaping};
use cosmic::iced::{alignment, Color, Font, Length, Pixels, Point, Radians, Rectangle};
use cosmic::{Element, Renderer, Theme};

/// Thickness of both the track and the progress arc.
const STROKE_WIDTH: f32 = 4.0;

/// Below this fraction the arc would be shorter than its own rounded cap.
const MIN_VISIBLE_FRACTION: f32 = 0.005;

pub struct CircularGauge {
    percent: f32,
    value_text: String,
    sub_text: Option<String>,
    label: &'static str,
    color: Color,
    is_dark: bool,
}

impl CircularGauge {
    pub fn new(percent: f32, value_text: String, label: &'static str, color: Color) -> Self {
        Self {
            percent: percent.clamp(0.0, 100.0),
            value_text,
            sub_text: None,
            label,
            color,
            is_dark: true,
        }
    }

    /// A secondary reading shown under the value, typically a temperature.
    pub fn with_sub_text(mut self, sub_text: Option<String>) -> Self {
        self.sub_text = sub_text;
        self
    }

    pub fn with_theme(mut self, is_dark: bool) -> Self {
        self.is_dark = is_dark;
        self
    }

    /// Renders the gauge with its caption beneath.
    pub fn view<Message: 'static>(self, diameter: f32) -> Element<'static, Message> {
        let label = self.label;
        let canvas = Canvas::new(self).width(Length::Fixed(diameter)).height(Length::Fixed(diameter));

        cosmic::widget::column![canvas, cosmic::widget::text(label).size(12)]
            .align_x(cosmic::iced::Alignment::Center)
            .spacing(6)
            .into()
    }
}

impl<Message> canvas::Program<Message, Theme, Renderer> for CircularGauge {
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
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        // Inset by the stroke so neither end of the ring is clipped.
        let radius = (bounds.width.min(bounds.height) / 2.0) - STROKE_WIDTH - 2.0;

        self.draw_ring(&mut frame, center, radius);
        self.draw_readout(&mut frame, center);

        vec![frame.into_geometry()]
    }
}

impl CircularGauge {
    /// The background track and the progress arc over it.
    fn draw_ring(&self, frame: &mut Frame, center: Point, radius: f32) {
        let track = if self.is_dark {
            Color::from_rgba(1.0, 1.0, 1.0, 0.10)
        } else {
            Color::from_rgba(0.0, 0.0, 0.0, 0.08)
        };
        frame.stroke(
            &Path::circle(center, radius),
            Stroke::default().with_color(track).with_width(STROKE_WIDTH),
        );

        // The arc sweeps clockwise from twelve o'clock.
        let fraction = self.percent / 100.0;
        if fraction > MIN_VISIBLE_FRACTION {
            let start_angle = -FRAC_PI_2;
            let arc = Path::new(|builder| {
                builder.arc(Arc {
                    center,
                    radius,
                    start_angle: Radians(start_angle),
                    end_angle: Radians(start_angle + fraction * TAU),
                });
            });
            frame.stroke(
                &arc,
                Stroke::default()
                    .with_color(self.color)
                    .with_width(STROKE_WIDTH)
                    .with_line_cap(LineCap::Round),
            );
        }
    }

    /// The value in the middle of the ring, with its optional second line.
    fn draw_readout(&self, frame: &mut Frame, center: Point) {
        let (value_color, sub_color) = if self.is_dark {
            (Color::from_rgb(0.95, 0.95, 0.95), Color::from_rgba(0.85, 0.85, 0.85, 0.75))
        } else {
            (Color::from_rgb(0.12, 0.14, 0.17), Color::from_rgba(0.25, 0.28, 0.33, 0.85))
        };

        match &self.sub_text {
            // With two lines, straddle the centre; alone, sit on it.
            Some(sub) => {
                frame.fill_text(centered_text(
                    self.value_text.clone(),
                    Point::new(center.x, center.y - 7.0),
                    value_color,
                    13.0,
                    true,
                ));
                frame.fill_text(centered_text(
                    sub.clone(),
                    Point::new(center.x, center.y + 8.0),
                    sub_color,
                    11.0,
                    false,
                ));
            }
            None => frame.fill_text(centered_text(
                self.value_text.clone(),
                center,
                value_color,
                14.0,
                true,
            )),
        }
    }
}

/// Canvas text anchored on its own centre in both axes.
fn centered_text(content: String, position: Point, color: Color, size: f32, bold: bool) -> Text {
    Text {
        content,
        position,
        color,
        size: Pixels(size),
        line_height: LineHeight::default(),
        font: Font {
            weight: if bold {
                cosmic::iced::font::Weight::Bold
            } else {
                cosmic::iced::font::Weight::Normal
            },
            ..Font::DEFAULT
        },
        align_x: alignment::Horizontal::Center.into(),
        align_y: alignment::Vertical::Center.into(),
        shaping: Shaping::Basic,
        max_width: f32::INFINITY,
    }
}
