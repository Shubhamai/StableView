use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Jsondata {
    pub mean: Vec<f32>,
    pub std: Vec<f32>,
    pub u_base: Vec<Vec<f32>>,
    pub w_shp_base: Vec<Vec<f32>>,
    pub w_exp_base: Vec<Vec<f32>>,
}