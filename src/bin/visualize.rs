use eigencurve::Point2;
use nalgebra as na;
use raylib::prelude::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize)]
struct ProcessedData {
    curves: Vec<Vec<Point2>>,
    coefficients: Vec<Vec<f32>>,
    basis: Vec<Vec<f32>>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <data_path>", args[0]);
        std::process::exit(1);
    }

    let data_path = &args[1];

    // Load data
    let mut file = File::open(data_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let data: ProcessedData = serde_json::from_str(&contents)?;

    // Perform dimensionality reduction (PCA)
    let (reduced_coeffs, _) = perform_pca(&data.coefficients, 2);

    // Initialize Raylib
    let (mut rl, thread) = raylib::init()
        .size(800, 600)
        .title("Curve Visualization")
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);

        // Draw reduced coefficient space
        draw_coefficient_space(&mut d, &reduced_coeffs);

        // Draw curves at their coefficient positions
        for (curve, coeff) in data.curves.iter().zip(reduced_coeffs.iter()) {
            draw_curve_at_position(&mut d, curve, coeff[0], coeff[1]);
        }
    }

    Ok(())
}

fn perform_pca(data: &[Vec<f32>], components: usize) -> (Vec<Vec<f32>>, na::DMatrix<f32>) {
    let rows: Vec<na::DVector<f32>> = data
        .iter()
        .map(|row| na::DVector::from_vec(row.clone()))
        .collect();
    let matrix = na::DMatrix::from_columns(&rows);

    println!("Matrix dimensions: {}x{}", matrix.nrows(), matrix.ncols());

    let mean = matrix.column_mean();

    println!("Mean dimensions: {}x{}", mean.nrows(), mean.ncols());

    // Ensure mean is a column vector
    let mean_column = na::DVector::from_iterator(mean.len(), mean.iter().cloned());

    println!(
        "Mean column dimensions: {}x{}",
        mean_column.nrows(),
        mean_column.ncols()
    );

    // Broadcast subtraction
    let centered = matrix
        .column_iter()
        .map(|col| col - &mean_column)
        .collect::<Vec<_>>();
    let centered = na::DMatrix::from_columns(&centered);

    println!(
        "Centered matrix dimensions: {}x{}",
        centered.nrows(),
        centered.ncols()
    );

    let cov = (centered.transpose() * &centered) / (data.len() as f32 - 1.0);

    println!(
        "Covariance matrix dimensions: {}x{}",
        cov.nrows(),
        cov.ncols()
    );

    let eigen = na::SymmetricEigen::new(cov);

    // Extract the first 'components' eigenvectors and convert to DMatrix
    let proj_matrix = na::DMatrix::from_iterator(
        eigen.eigenvectors.nrows(),
        components,
        eigen.eigenvectors.columns(0, components).iter().cloned(),
    );

    println!(
        "Projection matrix dimensions: {}x{}",
        proj_matrix.nrows(),
        proj_matrix.ncols()
    );

    let projected = &centered * &proj_matrix;

    println!(
        "Projected data dimensions: {}x{}",
        projected.nrows(),
        projected.ncols()
    );

    (
        projected
            .row_iter()
            .map(|row| row.iter().cloned().collect())
            .collect(),
        proj_matrix,
    )
}

// fn perform_pca(data: &[Vec<f32>], components: usize) -> (Vec<Vec<f32>>, na::DMatrix<f32>) {
//     let rows: Vec<na::OVector<f32, na::Dyn>> = data.iter()
//         .map(|row| na::OVector<f32, na::Dyn>::from_vec(row.clone()))
//         .collect();
//
//     let matrix = na::DMatrix::from_rows(&rows);
//     let mean = matrix.column_mean();
//     let centered = &matrix - mean.transpose();
//     let cov = (centered.transpose() * &centered) / (data.len() as f32 - 1.0);
//
//     let eigen = na::SymmetricEigen::new(cov);
//     let proj_matrix = eigen.eigenvectors.columns(0, components).into_owned();
//
//     let projected = &centered * &proj_matrix;
//
//     (projected.row_iter().map(|row| row.iter().cloned().collect()).collect(), proj_matrix)
// }

fn draw_coefficient_space(d: &mut RaylibDrawHandle, coeffs: &[Vec<f32>]) {
    let width = d.get_screen_width() as f32;
    let height = d.get_screen_height() as f32;

    let x_min = coeffs.iter().map(|c| c[0]).fold(f32::INFINITY, f32::min);
    let x_max = coeffs
        .iter()
        .map(|c| c[0])
        .fold(f32::NEG_INFINITY, f32::max);
    let y_min = coeffs.iter().map(|c| c[1]).fold(f32::INFINITY, f32::min);
    let y_max = coeffs
        .iter()
        .map(|c| c[1])
        .fold(f32::NEG_INFINITY, f32::max);

    let scale_x = (width - 100.0) / (x_max - x_min);
    let scale_y = (height - 100.0) / (y_max - y_min);

    for coeff in coeffs {
        let x = (coeff[0] - x_min) * scale_x + 50.0;
        let y = (coeff[1] - y_min) * scale_y + 50.0;
        d.draw_circle(x as i32, y as i32, 3.0, Color::RED);
    }
}

fn draw_curve_at_position(d: &mut RaylibDrawHandle, curve: &[Point2], x: f32, y: f32) {
    let width = d.get_screen_width() as f32;
    let height = d.get_screen_height() as f32;

    let rect_size = 30.0;
    let scale = rect_size
        / curve
            .iter()
            .map(|p| p.x.max(p.y))
            .fold(f32::NEG_INFINITY, f32::max);

    let x_min = curve.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
    let y_min = curve.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);

    for window in curve.windows(2) {
        let start = &window[0];
        let end = &window[1];
        d.draw_line(
            (x + (start.x - x_min) * scale) as i32,
            (y + (start.y - y_min) * scale) as i32,
            (x + (end.x - x_min) * scale) as i32,
            (y + (end.y - y_min) * scale) as i32,
            Color::BLUE,
        );
    }
}
