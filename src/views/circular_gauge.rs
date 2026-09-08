use std::f32::consts::{FRAC_PI_2, TAU};

use cosmic::iced::mouse::Cursor;
use cosmic::iced::widget::canvas::{self, path::Arc, Canvas, Frame, Geometry, LineCap, Path, Stroke, Text};
use cosmic::iced::{alignment, Color, Length, Pixels, Point, Radians, Rectangle};
use cosmic::{Element, Renderer, Theme};

pub struct CircularGauge {
    pub percent: f32,
    pub value_text: String,
    pub sub_text: Option<String>,
    pub label: &'static str,
    pub color: Color,
    pub is_dark: bool,
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

    pub fn with_sub_text(mut self, sub_text: Option<String>) -> Self {
        self.sub_text = sub_text;
        self
    }

    pub fn with_theme(mut self, is_dark: bool) -> Self {
        self.is_dark = is_dark;
        self
    }

    pub fn view<Message: 'static>(self, diameter: f32) -> Element<'static, Message> {
        let label = self.label;
        let canvas_widget = Canvas::new(self)
            .width(Length::Fixed(diameter))
            .height(Length::Fixed(diameter));

        let col = cosmic::widget::column![
            canvas_widget,
            cosmic::widget::text(label).size(12)
        ]
        .align_x(cosmic::iced::Alignment::Center)
        .spacing(6);

        col.into()
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
        let w = bounds.width;
        let h = bounds.height;
        let center = Point::new(w / 2.0, h / 2.0);
        let stroke_w = 4.0;
        let radius = (w.min(h) / 2.0) - stroke_w - 2.0;

        // 1. Draw background circular track
        let track_color = if self.is_dark {
            Color::from_rgba(1.0, 1.0, 1.0, 0.10)
        } else {
            Color::from_rgba(0.0, 0.0, 0.0, 0.08)
        };
        let track_stroke = Stroke::default()
            .with_color(track_color)
            .with_width(stroke_w);

        let track_path = Path::circle(center, radius);
        frame.stroke(&track_path, track_stroke);

        // 2. Draw progress arc
        let fraction = self.percent / 100.0;
        if fraction > 0.005 {
            let start_angle = -FRAC_PI_2;
            let end_angle = start_angle + (fraction * TAU);

            let arc_path = Path::new(|b| {
                b.arc(Arc {
                    center,
                    radius,
                    start_angle: Radians(start_angle),
                    end_angle: Radians(end_angle),
                });
            });

            let arc_stroke = Stroke::default()
                .with_color(self.color)
                .with_width(stroke_w)
                .with_line_cap(LineCap::Round);

            frame.stroke(&arc_path, arc_stroke);
        }

        // 3. Draw text in the center
        let text_color = if self.is_dark {
            Color::from_rgb(0.95, 0.95, 0.95)
        } else {
            Color::from_rgb(0.12, 0.14, 0.17)
        };

        let sub_color = if self.is_dark {
            Color::from_rgba(0.85, 0.85, 0.85, 0.75)
        } else {
            Color::from_rgba(0.25, 0.28, 0.33, 0.85)
        };

        if let Some(sub) = &self.sub_text {
            // Main percent slightly above center
            frame.fill_text(Text {
                content: self.value_text.clone(),
                position: Point::new(center.x, center.y - 7.0),
                color: text_color,
                size: Pixels(13.0),
                line_height: cosmic::iced::widget::text::LineHeight::default(),
                font: cosmic::iced::Font {
                    weight: cosmic::iced::font::Weight::Bold,
                    ..cosmic::iced::Font::DEFAULT
                },
                align_x: alignment::Horizontal::Center.into(),
                align_y: alignment::Vertical::Center.into(),
                shaping: cosmic::iced::widget::text::Shaping::Basic,
                max_width: f32::INFINITY,
            });

            // Sub text (temperature) slightly below center
            frame.fill_text(Text {
                content: sub.clone(),
                position: Point::new(center.x, center.y + 8.0),
                color: sub_color,
                size: Pixels(11.0),
                line_height: cosmic::iced::widget::text::LineHeight::default(),
                font: cosmic::iced::Font::DEFAULT,
                align_x: alignment::Horizontal::Center.into(),
                align_y: alignment::Vertical::Center.into(),
                shaping: cosmic::iced::widget::text::Shaping::Basic,
                max_width: f32::INFINITY,
            });
        } else {
            // Main percent directly in the middle
            frame.fill_text(Text {
                content: self.value_text.clone(),
                position: center,
                color: text_color,
                size: Pixels(14.0),
                line_height: cosmic::iced::widget::text::LineHeight::default(),
                font: cosmic::iced::Font {
                    weight: cosmic::iced::font::Weight::Bold,
                    ..cosmic::iced::Font::DEFAULT
                },
                align_x: alignment::Horizontal::Center.into(),
                align_y: alignment::Vertical::Center.into(),
                shaping: cosmic::iced::widget::text::Shaping::Basic,
                max_width: f32::INFINITY,
            });
        }

        vec![frame.into_geometry()]
    }
}
