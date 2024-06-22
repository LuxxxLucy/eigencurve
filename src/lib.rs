use nalgebra as na;
use serde::{Deserialize, Serialize};

// number of points to sample from a curve. see sample.rs
pub const SAMPLE_POINTS: usize = 30;

#[derive(Serialize, Deserialize)]
pub struct ProcessedData {
    pub curves: Vec<Vec<Point2>>,
    pub coefficients: Vec<Vec<f32>>,
    pub basis: Vec<Vec<f32>>,
}

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

impl From<na::Point2<f32>> for Point2 {
    fn from(p: na::Point2<f32>) -> Self {
        Point2 { x: p.x, y: p.y }
    }
}

impl From<Point2> for na::Point2<f32> {
    fn from(val: Point2) -> Self {
        na::Point2::new(val.x, val.y)
    }
}

pub mod algorithm;
pub mod load_font;
pub mod sample;
pub mod serialize;

// Public interface
pub use algorithm::{encode_curve, perform_svd_on_curves, reconstruct_curve};
pub use load_font::load_font_curves;
pub use sample::sample_curve;
pub use serialize::{load_processed_data, save_processed_data};
