use std::env;
use std::fs::File;
use std::io::Write;

use eigencurve::sample_curve;
use eigencurve::ProcessedData;
use eigencurve::{load_font_curves, perform_svd_on_curves};

const K: usize = 10;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <font_path> <output_path>", args[0]);
        std::process::exit(1);
    }

    let font_path = &args[1];
    let output_path = &args[2];

    let characters = vec!['A', 'B', 'C', 'D', 'E', 'F', 'G'];
    let curves = load_font_curves(font_path, &characters)
        .map_err(|e| format!("Failed to load font curves: {}", e))?;

    // Perform SVD on all curves
    let (basis, coefficients) = perform_svd_on_curves(&curves, K);

    println!("Number of curves: {}", curves.len());
    println!("Basis shape: {} x {}", basis.nrows(), basis.ncols());
    println!("Number of coefficients: {}", coefficients.len());

    evaluate_approximation(&curves, &basis);

    // Prepare data for storage
    let processed_data = ProcessedData {
        curves: curves.iter().map(sample_curve).collect(),
        coefficients: coefficients.iter().map(|c| c.as_slice().to_vec()).collect(),
        basis: basis
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

use eigencurve::{Curve, Point2};
use nalgebra as na;

#[allow(non_snake_case)]
pub fn evaluate_approximation(curves: &[Curve], basis: &na::DMatrix<f32>) {
    let num_curves = curves.len();
    let sampled_curves: Vec<Vec<Point2>> = curves.iter().map(sample_curve).collect();
    let points_per_curve = sampled_curves[0].len();

    let mut A = na::DMatrix::zeros(points_per_curve * 2, num_curves);
    for (i, curve) in sampled_curves.iter().enumerate() {
        for (j, point) in curve.iter().enumerate() {
            A[(j * 2, i)] = point.x;
            A[(j * 2 + 1, i)] = point.y;
        }
    }

    // Evaluate for different K values
    for k in 1..=basis.ncols() {
        // Low-rank approximation
        let U_trunc = basis.columns(0, k);
        let C = U_trunc.transpose() * &A;
        let A_recon = U_trunc * C;

        // Calculate error
        let error = (&A_recon - &A).norm();
        let avg_error = error / (num_curves as f32);

        println!("k: {}\tnum params: {}\terror: {:.4}", k, k, avg_error);
    }
}
