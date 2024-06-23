use std::env;
use std::fs::File;
use std::io::Write;

use eigencurve::{
    load_font_curves,
    sample_curve,
    Curve,
    CurveEncoder,
    Point2,
    ProcessedData,
    SVDEncoder, // Import directly from the crate root
};
use nalgebra as na;

const SAMPLE_POINTS: usize = 30;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <font_path> <output_path>", args[0]);
        std::process::exit(1);
    }

    let font_path = &args[1];
    let output_path = &args[2];

    // let characters = vec!['A', 'B', 'C', 'D', 'E', 'F', 'G'];
    let characters = vec!['A', 'B', 'C'];
    let curves = load_font_curves(font_path, &characters)
        .map_err(|e| format!("Failed to load font curves: {}", e))?;

    // Train the SVD encoder
    let encoder = SVDEncoder::train(&curves, SAMPLE_POINTS);

    // Report training results
    println!("Training Results:");
    println!("----------------");
    println!("Number of input curves: {}", curves.len());
    println!(
        "Number of sample points per curve: {}",
        encoder.num_sample_points()
    );
    println!(
        "Basis shape: {} x {}",
        encoder.get_basis().nrows(),
        encoder.get_basis().ncols()
    );
    println!(
        "Number of basis vectors (K): {}",
        encoder.num_basis_vectors()
    );
    println!("----------------\n");

    evaluate_approximation(&curves, &encoder);

    // Use all basis vectors for the final processing
    let embeddings = encoder.encode_batch(&curves);

    // Prepare data for storage
    let processed_data = ProcessedData {
        curves: curves
            .iter()
            .map(|c| sample_curve(c, encoder.num_sample_points()))
            .collect(),
        coefficients: embeddings.iter().map(|c| c.as_slice().to_vec()).collect(),
        basis: encoder
            .get_basis()
            .column_iter()
            .map(|c| c.iter().cloned().collect())
            .collect(),
    };

    // Store data
    let json = serde_json::to_string(&processed_data)?;
    let mut file = File::create(output_path)?;
    file.write_all(json.as_bytes())?;

    println!("Processing complete. Data saved to {}", output_path);
    Ok(())
}

#[allow(non_snake_case)]
fn evaluate_approximation(curves: &[Curve], encoder: &SVDEncoder) {
    let num_curves = curves.len();
    let num_points = encoder.num_sample_points();
    let sampled_curves: Vec<Vec<Point2>> =
        curves.iter().map(|c| sample_curve(c, num_points)).collect();

    // Create the full matrix of sampled curves
    let mut A = na::DMatrix::zeros(num_points * 2, num_curves);
    for (i, curve) in sampled_curves.iter().enumerate() {
        for (j, point) in curve.iter().enumerate() {
            A[(j * 2, i)] = point.x;
            A[(j * 2 + 1, i)] = point.y;
        }
    }

    println!("Approximation Evaluation:");
    println!("-------------------------");
    println!("k\tnum params\terror");
    // Evaluate for different K values
    for k in 1..=encoder.num_basis_vectors() {
        // Use subset of basis vectors
        let U_trunc = encoder.get_basis().columns(0, k);
        let C = U_trunc.transpose() * &A;
        let A_recon = U_trunc * C;

        // Calculate error
        let error = (&A_recon - &A).norm();
        let avg_error = error / (num_curves as f32);

        println!("{}\t{}\t{:.6}", k, k * 2, avg_error);
    }
    println!("-------------------------\n");
}
