use crate::{Curve, Point2};

// number of points to sample from a curve
const SAMPLE_POINTS: usize = 30;

pub fn sample_curve(curve: &Curve) -> Vec<Point2> {
    let t_values: Vec<f32> = (0..SAMPLE_POINTS)
        .map(|i| i as f32 / (SAMPLE_POINTS - 1) as f32)
        .collect();

    t_values
        .into_iter()
        .map(|t| match curve {
            Curve::Line { start, end } => Point2 {
                x: start.x + t * (end.x - start.x),
                y: start.y + t * (end.y - start.y),
            },
            Curve::Quadratic {
                start,
                control,
                end,
            } => {
                let x = (1.0 - t).powi(2) * start.x
                    + 2.0 * (1.0 - t) * t * control.x
                    + t.powi(2) * end.x;
                let y = (1.0 - t).powi(2) * start.y
                    + 2.0 * (1.0 - t) * t * control.y
                    + t.powi(2) * end.y;
                Point2 { x, y }
            }
            Curve::Cubic {
                start,
                control1,
                control2,
                end,
            } => {
                let x = (1.0 - t).powi(3) * start.x
                    + 3.0 * (1.0 - t).powi(2) * t * control1.x
                    + 3.0 * (1.0 - t) * t.powi(2) * control2.x
                    + t.powi(3) * end.x;
                let y = (1.0 - t).powi(3) * start.y
                    + 3.0 * (1.0 - t).powi(2) * t * control1.y
                    + 3.0 * (1.0 - t) * t.powi(2) * control2.y
                    + t.powi(3) * end.y;
                Point2 { x, y }
            }
        })
        .collect()
}
