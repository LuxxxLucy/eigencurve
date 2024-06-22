// serialize.rs
use super::ProcessedData;
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub fn save_processed_data(
    data: &ProcessedData,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, data)?;
    Ok(())
}

pub fn load_processed_data(path: &str) -> Result<ProcessedData, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
}
