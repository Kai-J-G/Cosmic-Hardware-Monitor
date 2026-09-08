use cosmic::iced::mouse::Cursor;
use cosmic::iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke};
use cosmic::iced::{Color, Length, Point, Rectangle};
use cosmic::{Element, Renderer, Theme};

pub struct Sparkline<'a> {
    history: &'a [f32],
    line_color: Color,
}

impl<'a> Sparkline<'a> {
    pub fn new(history: &'a [f32], line_color: Color) -> Self {
        Self {
            history,
            line_color,
        }
    }

    pub fn view<Message: 'static>(self, width: Length, height: Length) -> Element<'a, Message> {
        Canvas::new(self).width(width).height(height).into()
    }
}

impl<'a, Message> canvas::Program<Message, Theme, Renderer> for Sparkline<'a> {
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

        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            return vec![frame.into_geometry()];
        }

        let width = bounds.width;
        let height = bounds.height;

        // Draw subtle background grid
        let grid_color = Color::from_rgba(1.0, 1.0, 1.0, 0.05);
        let grid_stroke = Stroke::default().with_color(grid_color).with_width(1.0);

        // Horizontal grid lines (25%, 50%, 75%)
        for frac in [0.25, 0.50, 0.75] {
            let y = height * frac;
            frame.stroke(
                &Path::line(Point::new(0.0, y), Point::new(width, y)),
                grid_stroke,
            );
        }

        // Vertical grid lines (20%, 40%, 60%, 80%)
        for frac in [0.20, 0.40, 0.60, 0.80] {
            let x = width * frac;
            frame.stroke(
                &Path::line(Point::new(x, 0.0), Point::new(x, height)),
                grid_stroke,
            );
        }

        if self.history.len() < 2 {
            let mid_y = height * 0.5;
            let baseline_stroke = Stroke::default()
                .with_color(Color::from_rgba(
                    self.line_color.r,
                    self.line_color.g,
                    self.line_color.b,
                    0.3,
                ))
                .with_width(1.5);
            frame.stroke(
                &Path::line(Point::new(0.0, mid_y), Point::new(width, mid_y)),
                baseline_stroke,
            );
            return vec![frame.into_geometry()];
        }

        // Determine min and max with headroom
        let mut min_val = f32::MAX;
        let mut max_val = f32::MIN;
        for &v in self.history {
            if v < min_val {
                min_val = v;
            }
            if v > max_val {
                max_val = v;
            }
        }

        let span = (max_val - min_val).max(12.0);
        let pad = span * 0.12;
        let y_min = (min_val - pad).max(0.0);
        let y_max = max_val + pad;
        let y_range = (y_max - y_min).max(1.0);

        let n = self.history.len();
        let step_x = width / (n - 1) as f32;

        let points: Vec<Point> = self
            .history
            .iter()
            .enumerate()
            .map(|(i, &temp)| {
                let x = i as f32 * step_x;
                let norm = (temp - y_min) / y_range;
                let y = height - (norm * height).clamp(3.0, height - 3.0);
                Point::new(x, y)
            })
            .collect();

        // 1. Translucent gradient-like fill underneath line
        let fill_path = Path::new(|builder| {
            if let Some(first) = points.first() {
                builder.move_to(Point::new(first.x, height));
                builder.line_to(*first);
                for pt in &points[1..] {
                    builder.line_to(*pt);
                }
                if let Some(last) = points.last() {
                    builder.line_to(Point::new(last.x, height));
                    builder.close();
                }
            }
        });

        let fill_color = Color::from_rgba(
            self.line_color.r,
            self.line_color.g,
            self.line_color.b,
            0.14,
        );
        frame.fill(&fill_path, fill_color);

        // 2. Main smooth stroke
        let stroke_path = Path::new(|builder| {
            if let Some(first) = points.first() {
                builder.move_to(*first);
                for pt in &points[1..] {
                    builder.line_to(*pt);
                }
            }
        });

        let line_stroke = Stroke::default()
            .with_color(self.line_color)
            .with_width(2.2);
        frame.stroke(&stroke_path, line_stroke);

        // 3. Glowing current-value indicator at the newest data point
        if let Some(last_pt) = points.last() {
            // Soft outer halo
            let halo = Path::circle(*last_pt, 7.0);
            frame.fill(
                &halo,
                Color::from_rgba(
                    self.line_color.r,
                    self.line_color.g,
                    self.line_color.b,
                    0.25,
                ),
            );

            // Vibrant mid circle
            let mid_dot = Path::circle(*last_pt, 4.0);
            frame.fill(&mid_dot, self.line_color);

            // Crisp center point
            let center_dot = Path::circle(*last_pt, 1.8);
            frame.fill(&center_dot, Color::WHITE);
        }

        vec![frame.into_geometry()]
    }
}
