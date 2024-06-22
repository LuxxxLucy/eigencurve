use super::{Curve, Point2};
use std::fs::File;
use std::io::Read;
use ttf_parser;

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
