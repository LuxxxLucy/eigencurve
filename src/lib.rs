mod curve;
pub mod encoding;

pub use curve::{Curve, Point2, sample_curve, load_font_curves};
pub use encoding::{CurveEncoder, CurveDecoder, SVDEncoder};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ProcessedData {
    pub curves: Vec<Vec<Point2>>,
    pub coefficients: Vec<Vec<f32>>,
    pub basis: Vec<Vec<f32>>,
}

// If needed, you can add serialization functions here
pub fn save_processed_data(data: &ProcessedData, path: &str) -> std::io::Result<()> {
    let json = serde_json::to_string(data)?;
    std::fs::write(path, json)
}

pub fn load_processed_data(path: &str) -> std::io::Result<ProcessedData> {
    let json = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&json)?)
}
