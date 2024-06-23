use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use ttf_parser;

#[derive(Clone, Serialize, Deserialize)]
pub enum Curve {
    Line {
        start: Point2,
        end: Point2,
    },
    Quadratic {
        start: Point2,
        control: Point2,
        end: Point2,
    },
    Cubic {
        start: Point2,
        control1: Point2,
        control2: Point2,
        end: Point2,
    },
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

pub fn sample_curve(curve: &Curve, num_points: usize) -> Vec<Point2> {
    let t_values: Vec<f32> = (0..num_points)
        .map(|i| i as f32 / (num_points - 1) as f32)
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

pub fn load_font_curves(
    font_path: &str,
    characters: &[char],
) -> Result<Vec<Curve>, Box<dyn std::error::Error>> {
    let mut file = File::open(font_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let face = ttf_parser::Face::parse(&buffer, 0)?;
    let mut curves = Vec::new();

    for &c in characters {
        if let Some(glyph_id) = face.glyph_index(c) {
            face.outline_glyph(
                glyph_id,
                &mut CurveExtractor {
                    curves: &mut curves,
                    current_point: None,
                },
            );
        }
    }

    Ok(curves)
}

struct CurveExtractor<'a> {
    curves: &'a mut Vec<Curve>,
    current_point: Option<Point2>,
}

impl<'a> ttf_parser::OutlineBuilder for CurveExtractor<'a> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.current_point = Some(Point2 { x, y });
    }

    fn line_to(&mut self, x: f32, y: f32) {
        if let Some(start) = &self.current_point {
            let end = Point2 { x, y };
            self.curves.push(Curve::Line { start: *start, end });
            self.current_point = Some(end);
        }
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        if let Some(start) = &self.current_point {
            let control = Point2 { x: x1, y: y1 };
            let end = Point2 { x, y };
            self.curves.push(Curve::Quadratic {
                start: *start,
                control,
                end,
            });
            self.current_point = Some(end);
        }
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        if let Some(start) = &self.current_point {
            let control1 = Point2 { x: x1, y: y1 };
            let control2 = Point2 { x: x2, y: y2 };
            let end = Point2 { x, y };
            self.curves.push(Curve::Cubic {
                start: *start,
                control1,
                control2,
                end,
            });
            self.current_point = Some(end);
        }
    }

    fn close(&mut self) {
        // We don't need to do anything here as we're storing individual curves
    }
}
