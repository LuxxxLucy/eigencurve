use crate::{sample_curve, Curve, Point2};
use nalgebra as na;

pub fn encode_curve(curve: &Curve, basis: &na::DMatrix<f32>) -> na::DVector<f32> {
    let sampled_points = sample_curve(curve);
    let mut flattened = Vec::with_capacity(sampled_points.len() * 2);
    for point in sampled_points {
        flattened.push(point.x);
        flattened.push(point.y);
    }

    let flattened_vector = na::DVector::from_vec(flattened);
    basis.transpose() * flattened_vector
}

pub fn reconstruct_curve(coefficients: &na::DVector<f32>, basis: &na::DMatrix<f32>) -> Vec<Point2> {
    let reconstructed = basis * coefficients;
    (0..reconstructed.nrows() / 2)
        .map(|i| Point2 {
            x: reconstructed[2 * i],
            y: reconstructed[2 * i + 1],
        })
        .collect()
}

fn create_curve_matrix(curves: &[Curve]) -> na::DMatrix<f32> {
    let sampled_curves: Vec<Vec<Point2>> = curves.iter().map(sample_curve).collect();
    let num_curves = curves.len();
    let points_per_curve = sampled_curves[0].len();

    let mut matrix = na::DMatrix::zeros(num_curves, points_per_curve * 2);

    for (i, curve) in sampled_curves.iter().enumerate() {
        for (j, point) in curve.iter().enumerate() {
            matrix[(i, j * 2)] = point.x;
            matrix[(i, j * 2 + 1)] = point.y;
        }
    }

    matrix
}

pub fn perform_svd_on_curves(
    curves: &[Curve],
    k: usize,
) -> (na::DMatrix<f32>, Vec<na::DVector<f32>>) {
    let matrix = create_curve_matrix(curves);

    // Perform SVD
    let svd = na::SVD::new(matrix.transpose(), true, true);
    let u = svd.u.unwrap();

    // Get the first k left singular vectors as the basis
    let basis = u.columns(0, k).into_owned();

    // Project each curve onto the basis to get coefficients
    let coefficients: Vec<na::DVector<f32>> = matrix
        .row_iter()
        .map(|row| basis.transpose() * row.transpose())
        .collect();

    (basis, coefficients)
}
