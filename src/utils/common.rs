/// Common utility functions used by multiple modules
// Importing Libraries
use crate::enums::extreme::Extreme;
use onnxruntime::ndarray::{Array2, ArrayBase, Axis, Dim, OwnedRepr};

// Get minimum or maximum value from a vector containing floats
pub fn get_extreme_value(vec: &[f32], extreme: Extreme) -> f32 {
    let output = match extreme {
        Extreme::Min => vec.iter().min_by(|a, b| a.total_cmp(b)),
        Extreme::Max => vec.iter().max_by(|a, b| a.total_cmp(b)),
    };

    match output {
        Some(value) => *value,
        None => {
            tracing::error!("Unable to get extreme value. Default to 0");
            0.
        }
    }
}

// Converting a vector to ndarray
pub fn get_ndarray(
    vec: Vec<Vec<f32>>,
    shape: (usize, usize),
) -> ArrayBase<OwnedRepr<f32>, Dim<[usize; 2]>> {
    let mut array = Array2::<f32>::default(shape);
    for (i, mut row) in array.axis_iter_mut(Axis(0)).enumerate() {
        for (j, col) in row.iter_mut().enumerate() {
            *col = vec[i][j];
        }
    }

    array
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ndarray() {
        let _result = get_ndarray(vec![vec![1., 2.], vec![2., 3.], vec![3., 4.]], (3, 2));
    }

    #[test]
    fn test_get_extreme() {
        assert_eq!(get_extreme_value(&[1., 2., 3.], Extreme::Max), 3.);
        assert_eq!(get_extreme_value(&[1., 2., 3.], Extreme::Min), 1.)
    }
}
