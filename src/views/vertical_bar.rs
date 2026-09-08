use cosmic::iced::mouse::Cursor;
use cosmic::iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke};
use cosmic::iced::{Color, Length, Point, Rectangle, Size};
use cosmic::{Element, Renderer, Theme};

pub struct VerticalBar {
    fraction: f32,
    fill_color: Color,
    track_color: Color,
}

impl VerticalBar {
    pub fn new(fraction: f32, fill_color: Color) -> Self {
        Self {
            fraction: fraction.clamp(0.0, 1.0),
            fill_color,
            track_color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
        }
    }

    pub fn view<Message: 'static>(self, width: f32, height: f32) -> Element<'static, Message> {
        Canvas::new(self)
            .width(Length::Fixed(width))
            .height(Length::Fixed(height))
            .into()
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
        let width = bounds.width;
        let height = bounds.height;
        let radius = width / 2.0;

        // 1. Draw track (dark rounded background)
        let track_rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width,
            height,
        };
        let track_path =
            Path::rounded_rectangle(Point::new(0.0, 0.0), track_rect.size(), radius.into());
        frame.fill(&track_path, self.track_color);

        // Subtle track outline
        let outline_stroke = Stroke::default()
            .with_color(Color::from_rgba(1.0, 1.0, 1.0, 0.04))
            .with_width(1.0);
        frame.stroke(&track_path, outline_stroke);

        // 2. Draw fill (from bottom upwards)
        let fill_height = (height * self.fraction).max(radius * 2.0);
        let fill_y = height - fill_height;
        let fill_path = Path::rounded_rectangle(
            Point::new(0.0, fill_y),
            Size::new(width, fill_height),
            radius.into(),
        );
        frame.fill(&fill_path, self.fill_color);

        // Glow tip on top of the fill
        let tip_center = Point::new(radius, fill_y + radius);
        let tip_circle = Path::circle(tip_center, radius * 0.7);
        frame.fill(&tip_circle, Color::from_rgba(1.0, 1.0, 1.0, 0.35));

        vec![frame.into_geometry()]
    }
}
