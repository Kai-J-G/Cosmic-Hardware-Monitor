//! A history chart: a filled trend line with a marker on the newest sample.

use cosmic::iced::mouse::Cursor;
use cosmic::iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke};
use cosmic::iced::{Color, Length, Point, Rectangle};
use cosmic::{Element, Renderer, Theme};

/// Smallest vertical range the chart will scale to, in the samples' own units.
/// Without a floor, a flat line would be amplified into meaningless noise.
const MIN_SPAN: f32 = 12.0;

/// Headroom above and below the data, as a fraction of its range.
const PADDING: f32 = 0.12;

/// Keeps the line clear of the chart's own edges.
const EDGE_INSET: f32 = 3.0;

const GRID_COLOR: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.05);
const HORIZONTAL_GRID: [f32; 3] = [0.25, 0.50, 0.75];
const VERTICAL_GRID: [f32; 4] = [0.20, 0.40, 0.60, 0.80];

pub struct Sparkline<'a> {
    history: &'a [f32],
    line_color: Color,
}

impl<'a> Sparkline<'a> {
    pub fn new(history: &'a [f32], line_color: Color) -> Self {
        Self { history, line_color }
    }

    pub fn view<Message: 'static>(self, width: Length, height: Length) -> Element<'a, Message> {
        Canvas::new(self).width(width).height(height).into()
    }
}

impl<Message> canvas::Program<Message, Theme, Renderer> for Sparkline<'_> {
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
        if width <= 0.0 || height <= 0.0 {
            return vec![frame.into_geometry()];
        }

        self.draw_grid(&mut frame, width, height);

        // A single sample cannot describe a trend; show a baseline instead.
        let Some(points) = self.plot(width, height) else {
            let y = height * 0.5;
            frame.stroke(
                &Path::line(Point::new(0.0, y), Point::new(width, y)),
                Stroke::default().with_color(self.tint(0.3)).with_width(1.5),
            );
            return vec![frame.into_geometry()];
        };

        // Fill first, so the line and marker sit on top of it.
        frame.fill(&area_under(&points, height), self.tint(0.14));
        frame.stroke(
            &line_through(&points),
            Stroke::default().with_color(self.line_color).with_width(2.2),
        );

        if let Some(&latest) = points.last() {
            frame.fill(&Path::circle(latest, 7.0), self.tint(0.25));
            frame.fill(&Path::circle(latest, 4.0), self.line_color);
            frame.fill(&Path::circle(latest, 1.8), Color::WHITE);
        }

        vec![frame.into_geometry()]
    }
}

impl Sparkline<'_> {
    /// The line colour at a given opacity.
    fn tint(&self, alpha: f32) -> Color {
        Color { a: alpha, ..self.line_color }
    }

    fn draw_grid(&self, frame: &mut Frame, width: f32, height: f32) {
        let stroke = Stroke::default().with_color(GRID_COLOR).with_width(1.0);

        for fraction in HORIZONTAL_GRID {
            let y = height * fraction;
            frame.stroke(&Path::line(Point::new(0.0, y), Point::new(width, y)), stroke);
        }
        for fraction in VERTICAL_GRID {
            let x = width * fraction;
            frame.stroke(&Path::line(Point::new(x, 0.0), Point::new(x, height)), stroke);
        }
    }

    /// Maps the samples onto the canvas, scaled to their own range plus
    /// headroom. Returns `None` when there is nothing to draw a line through.
    fn plot(&self, width: f32, height: f32) -> Option<Vec<Point>> {
        if self.history.len() < 2 {
            return None;
        }

        // Floats have no total ordering, so `iter().min()` isn't available;
        // folding with `f32::min` is the standard way round it.
        let min = self.history.iter().copied().fold(f32::MAX, f32::min);
        let max = self.history.iter().copied().fold(f32::MIN, f32::max);
        let padding = (max - min).max(MIN_SPAN) * PADDING;
        let floor = (min - padding).max(0.0);
        let range = (max + padding - floor).max(1.0);

        let step = width / (self.history.len() - 1) as f32;
        Some(
            self.history
                .iter()
                .enumerate()
                .map(|(i, &value)| {
                    let offset = ((value - floor) / range * height).clamp(EDGE_INSET, height - EDGE_INSET);
                    Point::new(i as f32 * step, height - offset)
                })
                .collect(),
        )
    }
}

/// The trend line itself.
fn line_through(points: &[Point]) -> Path {
    Path::new(|builder| {
        let Some((first, rest)) = points.split_first() else { return };
        builder.move_to(*first);
        for point in rest {
            builder.line_to(*point);
        }
    })
}

/// The trend line closed down to the baseline, for the translucent fill.
fn area_under(points: &[Point], height: f32) -> Path {
    Path::new(|builder| {
        let (Some(first), Some(last)) = (points.first(), points.last()) else { return };
        builder.move_to(Point::new(first.x, height));
        for point in points {
            builder.line_to(*point);
        }
        builder.line_to(Point::new(last.x, height));
        builder.close();
    })
}
