use std::sync::{Mutex, Arc};
use onnxruntime::{ndarray::{ArrayBase, Dim, OwnedRepr}, session::Session};

pub struct Tddfa {
    pub landmark_model: Arc<Mutex<Session<'static>>>,
    pub size: i32,
    pub mean_array: [f32; 62],
    pub std_array: [f32; 62],
    pub u_base_array: ArrayBase<OwnedRepr<f32>, Dim<[usize; 2]>>,
    pub w_shp_base_array: ArrayBase<OwnedRepr<f32>, Dim<[usize; 2]>>,
    pub w_exp_base_array: ArrayBase<OwnedRepr<f32>, Dim<[usize; 2]>>,
}