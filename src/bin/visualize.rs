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
    coefficients: Vec<Vec<f64>>,
    basis: Vec<Vec<f64>>,
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

    println!("Loaded {} curves", data.curves.len());
    println!("Coefficient dimensions: {}", data.coefficients[0].len());

    // Convert coefficients to 2d.
    let n_components = 2;
    let coeffs = vec_to_matrix_rows(data.coefficients.clone());
    let reduced_coeffs = perform_pca(&coeffs, n_components);

    println!("Reduced coefficients shape: {:?}", reduced_coeffs.shape());

    let (mut rl, thread) = raylib::init()
        .size(800, 600)
        .title("Curve Visualization")
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);

        let viewport = calculate_viewport(
            &reduced_coeffs,
            d.get_screen_width() as f32,
            d.get_screen_height() as f32,
        );

        // Draw curves and points
        for (curve, coeff) in data.curves.iter().zip(reduced_coeffs.row_iter()) {
            draw_curve(&mut d, &viewport, curve, coeff[0], coeff[1]);
            draw_point(&mut d, &viewport, coeff[0], coeff[1], Color::RED);
        }
    }

    Ok(())
}

fn vec_to_matrix_rows(data: Vec<Vec<f64>>) -> na::DMatrix<f64> {
    let rows = data.len();
    let cols = data[0].len(); // Assuming all inner vectors have the same length
    let flat_data: Vec<f64> = data.into_iter().flatten().collect();
    na::DMatrix::from_row_slice(rows, cols, &flat_data)
}

fn perform_pca(data: &na::DMatrix<f64>, n_components: usize) -> na::DMatrix<f64> {
    let (nrows, ncols) = data.shape();
    println!("Input data shape: {}x{}", nrows, ncols);

    // Compute the mean of each column (feature)
    let mean = data.row_mean();
    println!("Mean shape: {}x{}", mean.nrows(), mean.ncols());

    // Center the data
    let mut centered = data.clone();
    for i in 0..nrows {
        centered.set_row(i, &(centered.row(i) - &mean));
    }
    println!(
        "Centered data shape: {}x{}",
        centered.nrows(),
        centered.ncols()
    );

    // Perform SVD on the centered data
    let svd = na::SVD::new(centered.clone(), true, true);

    // Get the right singular vectors (V)
    let v = svd.v_t.unwrap().transpose();
    println!("V shape: {}x{}", v.nrows(), v.ncols());

    // Select the top n_components
    let pc = v.columns(0, n_components.min(ncols));
    println!("PC shape: {}x{}", pc.nrows(), pc.ncols());

    // Project the centered data onto the principal components
    println!(
        "Projection shapes: ({} x {}) * ({} x {})",
        centered.nrows(),
        centered.ncols(),
        pc.nrows(),
        pc.ncols()
    );

    let result = centered * pc;
    println!("Result shape: {}x{}", result.nrows(), result.ncols());

    result
}

const CURVE_SIZE: f32 = 80.0; // Size of each curve
const POINT_RADIUS: f32 = 5.0; // Radius of the red circles representing points
const CURVE_CENTER_RADIUS: f32 = 3.0;

struct Viewport {
    scale: f32,
    offset_x: f32,
    offset_y: f32,
    height: f32,
}

fn calculate_viewport(coeffs: &na::DMatrix<f64>, width: f32, height: f32) -> Viewport {
    let x_min = coeffs.column(0).min();
    let x_max = coeffs.column(0).max();
    let y_min = coeffs.column(1).min();
    let y_max = coeffs.column(1).max();

    let data_width = (x_max - x_min) as f32;
    let data_height = (y_max - y_min) as f32;

    let margin = CURVE_SIZE / 2.0 + POINT_RADIUS;
    let scale_x = (width - 2.0 * margin) / data_width;
    let scale_y = (height - 2.0 * margin) / data_height;
    let scale = scale_x.min(scale_y);

    let center_x = (x_min + x_max) / 2.0;
    let center_y = (y_min + y_max) / 2.0;

    Viewport {
        scale,
        offset_x: width / 2.0 - (center_x as f32 * scale),
        offset_y: height / 2.0 - (center_y as f32 * scale),
        height,
    }
}

fn draw_point(d: &mut RaylibDrawHandle, viewport: &Viewport, x: f64, y: f64, color: Color) {
    let screen_x = (x as f32 * viewport.scale + viewport.offset_x) as i32;
    let screen_y = (viewport.height - (y as f32 * viewport.scale + viewport.offset_y)) as i32;
    d.draw_circle(screen_x, screen_y, POINT_RADIUS, color);
}

fn normalize_curve(curve: &[Point2]) -> Vec<Point2> {
    // Find the bounding box
    let min_x = curve.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
    let max_x = curve.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
    let min_y = curve.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
    let max_y = curve.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);

    let width = max_x - min_x;
    let height = max_y - min_y;

    // Calculate the center of the curve
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;

    // Calculate the scaling factor
    let scale = (CURVE_SIZE - 4.0) / width.max(height);

    // Normalize and center the curve
    curve
        .iter()
        .map(|p| Point2 {
            x: (p.x - center_x) * scale,
            y: (p.y - center_y) * scale,
        })
        .collect()
}

fn draw_curve(d: &mut RaylibDrawHandle, viewport: &Viewport, curve: &[Point2], x: f64, y: f64) {
    let normalized = normalize_curve(curve);
    let screen_x = (x as f32 * viewport.scale + viewport.offset_x) as f32;
    let screen_y = (viewport.height - (y as f32 * viewport.scale + viewport.offset_y)) as f32;

    for window in normalized.windows(2) {
        let start = &window[0];
        let end = &window[1];

        d.draw_line(
            (start.x + screen_x) as i32,
            (start.y + screen_y) as i32,
            (end.x + screen_x) as i32,
            (end.y + screen_y) as i32,
            Color::BLUE,
        );
    }

    // Draw a small green circle at the center of the curve
    d.draw_circle(
        screen_x as i32,
        screen_y as i32,
        CURVE_CENTER_RADIUS,
        Color::GREEN,
    );
}
