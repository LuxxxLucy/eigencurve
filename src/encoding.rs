use crate::{Curve, Point2};
use nalgebra as na;

pub trait CurveEncoder {
    type Embedding;

    fn encode(&self, curve: &Curve) -> Self::Embedding;
    fn encode_batch(&self, curves: &[Curve]) -> Vec<Self::Embedding> {
        curves.iter().map(|curve| self.encode(curve)).collect()
    }
}

pub trait CurveDecoder {
    type Embedding;

    fn decode(&self, embedding: &Self::Embedding) -> Vec<Curve>;
    fn decode_batch(&self, embeddings: &[Self::Embedding]) -> Vec<Vec<Curve>> {
        embeddings.iter().map(|emb| self.decode(emb)).collect()
    }
}

pub struct SVDEncoder {
    basis: na::DMatrix<f32>,
    num_points: usize,
}

impl SVDEncoder {
    pub fn get_basis(&self) -> &na::DMatrix<f32> {
        &self.basis
    }

    pub fn num_basis_vectors(&self) -> usize {
        self.basis.ncols()
    }

    pub fn num_sample_points(&self) -> usize {
        self.num_points
    }

    pub fn train(curves: &[Curve], num_points: usize) -> Self {
        let matrix = Self::create_curve_matrix(curves, num_points);
        let svd = na::SVD::new(matrix.transpose(), true, true);
        let u = svd.u.unwrap();
        let singular_values = svd.singular_values;

        // Determine k based on non-zero singular values
        let k = singular_values.iter().take_while(|&&sv| sv > 1e-10).count();
        let basis = u.columns(0, k).into_owned();

        Self { basis, num_points }
    }

    fn create_curve_matrix(curves: &[Curve], num_points: usize) -> na::DMatrix<f32> {
        let sampled_curves: Vec<Vec<Point2>> = curves
            .iter()
            .map(|c| crate::sample_curve(c, num_points))
            .collect();
        let num_curves = curves.len();

        let mut matrix = na::DMatrix::zeros(num_curves, num_points * 2);

        for (i, curve) in sampled_curves.iter().enumerate() {
            for (j, point) in curve.iter().enumerate() {
                matrix[(i, j * 2)] = point.x;
                matrix[(i, j * 2 + 1)] = point.y;
            }
        }

        matrix
    }
}

impl CurveEncoder for SVDEncoder {
    type Embedding = na::DVector<f32>;

    fn encode(&self, curve: &Curve) -> Self::Embedding {
        let sampled_points = crate::sample_curve(curve, self.num_points);
        let mut flattened = Vec::with_capacity(sampled_points.len() * 2);
        for point in sampled_points {
            flattened.push(point.x);
            flattened.push(point.y);
        }

        let flattened_vector = na::DVector::from_vec(flattened);
        self.basis.transpose() * flattened_vector
    }
}

impl CurveDecoder for SVDEncoder {
    type Embedding = na::DVector<f32>;

    fn decode(&self, embedding: &Self::Embedding) -> Vec<Curve> {
        let reconstructed = &self.basis * embedding;
        let points: Vec<Point2> = (0..self.num_points)
            .map(|i| Point2 {
                x: reconstructed[2 * i],
                y: reconstructed[2 * i + 1],
            })
            .collect();

        // Create a series of line segments from the points
        points
            .windows(2)
            .map(|window| Curve::Line {
                start: window[0],
                end: window[1],
            })
            .collect()
    }
}
