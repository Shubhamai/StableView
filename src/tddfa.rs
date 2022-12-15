use crate::{
    model::OnnxSessionsManager,
    utils::{
        crop_img, parse_param, parse_roi_box_from_bbox, parse_roi_box_from_landmark,
        similar_transform,
    },
};
use std::error::Error;
use std::sync::Mutex;

use onnxruntime::tensor::OrtOwnedTensor;
use onnxruntime::{environment::Environment, session::Session};
use serde::{Deserialize, Serialize};

use once_cell::sync::Lazy;
use opencv::{
    core::{Size, Vec3b},
    imgproc,
    prelude::{Mat, MatTraitConstManual},
};

use onnxruntime::ndarray::{arr1, arr2, s, Array2, Array4, ArrayBase, Axis, Dim, Order, OwnedRepr};
use std::ops::Deref;

#[derive(Serialize, Deserialize)]
struct DataStruct {
    mean: Vec<f32>,
    std: Vec<f32>,
    u_base: Vec<Vec<f32>>,
    w_shp_base: Vec<Vec<f32>>,
    w_exp_base: Vec<Vec<f32>>,
}

#[allow(clippy::upper_case_acronyms)]
pub struct TDDFA {
    landmark_model: Mutex<Session<'static>>,
    size: i32,
    mean_array: [f32; 62],
    std_array: [f32; 62],
    u_base_array: ArrayBase<OwnedRepr<f32>, Dim<[usize; 2]>>,
    w_shp_base_array: ArrayBase<OwnedRepr<f32>, Dim<[usize; 2]>>,
    w_exp_base_array: ArrayBase<OwnedRepr<f32>, Dim<[usize; 2]>>,
}

impl TDDFA {
    pub fn new(
        data_fp: &str,
        landmark_model_path: &str,
        size: i32,
    ) -> Result<Self, Box<dyn Error>> {
        static ENV: Lazy<Environment> =
            Lazy::new(|| OnnxSessionsManager::get_environment("Landmark Detection").unwrap());

        let landmark_model =
            OnnxSessionsManager::initialize_model(&ENV, landmark_model_path.to_string(), 1)?;
        let landmark_model = Mutex::new(landmark_model);

        let data = {
            let data = std::fs::read_to_string(data_fp).unwrap();
            serde_json::from_str::<DataStruct>(&data).unwrap()
        };

        let mean_array: [f32; 62] = data.mean.as_slice().try_into().unwrap();
        let std_array: [f32; 62] = data.std.as_slice().try_into().unwrap();

        let mut u_base_array = Array2::<f32>::default((204, 1));
        for (i, mut row) in u_base_array.axis_iter_mut(Axis(0)).enumerate() {
            for (j, col) in row.iter_mut().enumerate() {
                *col = data.u_base[i][j];
            }
        }

        let mut w_shp_base_array = Array2::<f32>::default((204, 40));
        for (i, mut row) in w_shp_base_array.axis_iter_mut(Axis(0)).enumerate() {
            for (j, col) in row.iter_mut().enumerate() {
                *col = data.w_shp_base[i][j];
            }
        }

        let mut w_exp_base_array = Array2::<f32>::default((204, 10));
        for (i, mut row) in w_exp_base_array.axis_iter_mut(Axis(0)).enumerate() {
            for (j, col) in row.iter_mut().enumerate() {
                *col = data.w_exp_base[i][j];
            }
        }

        Ok(Self {
            landmark_model,
            size,
            mean_array,
            std_array,
            u_base_array,
            w_shp_base_array,
            w_exp_base_array,
        })
    }

    fn get_model_input(
        &self,
        input_frame: &Mat,
        roi_box: [f32; 4],
    ) -> Vec<ArrayBase<OwnedRepr<f32>, Dim<[usize; 4]>>> {

        // let mut rgb_frame = Mat::default();
        // imgproc::cvt_color(&input_frame, &mut rgb_frame, imgproc::COLOR_BGR2RGB, 0).unwrap();

        let cropped_image = crop_img(input_frame, roi_box);

        // Resizing the frame
        let mut resized_frame = Mat::default();
        imgproc::resize(
            &cropped_image,
            &mut resized_frame,
            Size {
                width: self.size,
                height: self.size,
            },
            0.0,
            0.0,
            imgproc::INTER_LINEAR, //*INTER_AREA, // https://stackoverflow.com/a/51042104 | Speed -> https://stackoverflow.com/a/44278268
        )
        .unwrap();

        let vec = Mat::data_typed::<Vec3b>(&resized_frame).unwrap();

        let array = Array4::from_shape_fn(
            (1, 3, self.size as usize, self.size as usize),
            |(_, c, y, x)| {
                (f32::from(Vec3b::deref(&vec[x + y * self.size as usize])[c]) - 127.5) / 128.0
            },
        );

        vec![array]
    }

    pub fn run(
        &self,
        input_frame: &Mat,
        face_box: [f32; 4],
        ver: Vec<Vec<f32>>,
        crop_policy: &str,
    ) -> Result<([f32; 62], [f32; 4]), Box<dyn Error>> {
        let mut roi_box = [0.; 4];
        if crop_policy == "box" {
            roi_box = parse_roi_box_from_bbox(face_box);
        } else if crop_policy == "landmark" {
            roi_box = parse_roi_box_from_landmark(ver);
        } else {
            tracing::error!("Invalid crop policy : {crop_policy}");
        }

        let model_input = self.get_model_input(input_frame, roi_box);

        // Inference
        let mut landmark_model = self.landmark_model.lock().unwrap();
        let param: Vec<OrtOwnedTensor<f32, _>> = landmark_model.run(model_input).unwrap();
        let param: [f32; 62] = param[0].as_slice().unwrap().try_into().unwrap();

        // Rescaling the output by multiplying with standard deviation and adding mean
        let processed_param = arr1(&param) * arr1(&self.std_array) + arr1(&self.mean_array);
        let processed_param: [f32; 62] = processed_param.as_slice().unwrap().try_into().unwrap();
        Ok((processed_param, roi_box))
    }

    pub fn recon_vers(&self, param: [f32; 62], roi_box: [f32; 4]) -> Vec<Vec<f32>> {
        let (r, offset, alpha_shp, alpha_exp) = parse_param(&param).unwrap();

        let pts3d = &self.u_base_array
            + (&self.w_shp_base_array.dot(&arr2(&alpha_shp)))
            + (&self.w_exp_base_array.dot(&arr2(&alpha_exp)));
        let pts3d = pts3d.to_shape(((3, 68), Order::ColumnMajor)).unwrap();
        let pts3d = arr2(&r).dot(&pts3d) + arr2(&offset);
        let vec_pts_3d = vec![
            pts3d.slice(s![0, ..]).to_vec(),
            pts3d.slice(s![1, ..]).to_vec(),
            pts3d.slice(s![2, ..]).to_vec(),
        ];
        similar_transform(vec_pts_3d, roi_box, self.size)
    }
}

#[test]
pub fn test() {
    use opencv::core::{Scalar, CV_8UC3};

    let data_fp = "./assets/data.json";
    let landmark_model_path = "./assets/mb05_120x120.onnx";
    let size = 120;

    let bfm = TDDFA::new(data_fp, landmark_model_path, size).unwrap();

    let frame =
        Mat::new_rows_cols_with_default(120, 120, CV_8UC3, Scalar::new(255., 0., 0., 0.)).unwrap();

    let face_box = [150., 150., 400., 400.];
    let (param, roi_box) = bfm
        .run(
            &frame,
            face_box,
            vec![vec![1., 2., 3.], vec![4., 5., 6.], vec![7., 8., 9.]],
            "box",
        )
        .unwrap();
    let pts_3d = bfm.recon_vers(param, roi_box);

    let (param, roi_box) = bfm.run(&frame, face_box, pts_3d, "landmark").unwrap();
    let _pts_3d = bfm.recon_vers(param, roi_box);
}
