/// Model inference and generating the head pose
/// Python source - https://github.com/cleardusk/3DDFA_V2/blob/master/TDDFA.py


// Importing Libraries
use crate::{utils::{
    common::get_ndarray,
    image::crop_img,
    tddfa::{parse_param, parse_roi_box_from_bbox, parse_roi_box_from_landmark, similar_transform},
}, structs::{tddfa::Tddfa, data::Jsondata}, enums::crop_policy::CropPolicy};

use onnxruntime::{
    environment::Environment,
    ndarray::{arr1, arr2, s, Array4, ArrayBase, Dim, Order, OwnedRepr},
    tensor::OrtOwnedTensor,
    GraphOptimizationLevel,
};
use std::{
    error::Error,
    ops::Deref,
    sync::{Arc, Mutex},
};

use once_cell::sync::Lazy;
use opencv::{
    core::{Size, Vec3b},
    imgproc,
    prelude::{Mat, MatTraitConstManual},
};

 
static ENVIRONMENT: Lazy<Environment> = Lazy::new(|| {
    Environment::builder()
        .with_name("Landmark Detection")
        .with_log_level(onnxruntime::LoggingLevel::Warning)
        .build()
        .unwrap()
});

impl Tddfa {
    pub fn new(size: i32) -> Result<Self, Box<dyn Error>> {

        let landmark_model = ENVIRONMENT
            .new_session_builder()?
            .with_optimization_level(GraphOptimizationLevel::All)?
            .with_number_threads(1)?
            .with_model_from_memory(include_bytes!("../assets/mb05_120x120.onnx"))?;
        let landmark_model = Arc::new(Mutex::new(landmark_model));

        let data =
            serde_json::from_slice::<Jsondata>(include_bytes!("../assets/data.json")).unwrap();

        let mean_array: [f32; 62] = data.mean.as_slice().try_into().unwrap();
        let std_array: [f32; 62] = data.std.as_slice().try_into().unwrap();

        let u_base_array = get_ndarray(data.u_base, (204, 1));
        let w_shp_base_array = get_ndarray(data.w_shp_base, (204, 40));
        let w_exp_base_array = get_ndarray(data.w_exp_base, (204, 10));

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
        roi_box: &[f32; 4],
    ) -> Vec<ArrayBase<OwnedRepr<f32>, Dim<[usize; 4]>>> {
        // let mut rgb_frame = Mat::default();
        // imgproc::cvt_color(&input_frame, &mut rgb_frame, imgproc::COLOR_BGR2RGB, 0).unwrap();

        // Cropping the image
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
        .unwrap(); // ! Error handling here

        let vec = Mat::data_typed::<Vec3b>(&resized_frame)
            .expect("Unable to convert the image to vector");

        let array = Array4::from_shape_fn(
            (1, 3, self.size as usize, self.size as usize),
            |(_, c, y, x)| {
                (f32::from(Vec3b::deref(&vec[x + y * self.size as usize])[c]) - 127.5) / 128.0
            },
        );

        vec![array]
    }

    pub fn run(
        &mut self,
        input_frame: &Mat,
        face_box: [f32; 4],
        ver: &[Vec<f32>],
        crop_policy: CropPolicy,
    ) -> Result<([f32; 62], [f32; 4]), Box<dyn Error>> {

        let roi_box = match crop_policy {
            CropPolicy::Box => parse_roi_box_from_bbox(face_box),
            CropPolicy::Landmark => parse_roi_box_from_landmark(ver)
        };

        let model_input = self.get_model_input(input_frame, &roi_box);

        // Inference
        let mut landmark_model = self.landmark_model.try_lock().unwrap(); // * unblocking lock
        let param: Vec<OrtOwnedTensor<f32, _>> = landmark_model.run(model_input).unwrap();
        let param: [f32; 62] = param[0].as_slice().unwrap().try_into().unwrap();

        // Rescaling the output by multiplying with standard deviation and adding mean
        let processed_param = arr1(&param) * arr1(&self.std_array) + arr1(&self.mean_array);
        let processed_param: [f32; 62] = processed_param.as_slice().unwrap().try_into().unwrap();
        Ok((processed_param, roi_box))
    }

    pub fn recon_vers(&self, param: [f32; 62], roi_box: [f32; 4]) -> Vec<Vec<f32>> {
        let (r, offset, alpha_shp, alpha_exp) = parse_param(&param);

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
        similar_transform(vec_pts_3d, roi_box, self.size as f32)
    }
}

#[test]
pub fn test() {
    use opencv::core::{Scalar, CV_8UC3};

    let size = 120;

    let mut bfm = Tddfa::new(size).unwrap();

    let frame =
        Mat::new_rows_cols_with_default(120, 120, CV_8UC3, Scalar::new(255., 0., 0., 0.)).unwrap();

    let face_box = [150., 150., 400., 400.];
    let (param, roi_box) = bfm
        .run(
            &frame,
            face_box,
            &[vec![1., 2., 3.], vec![4., 5., 6.], vec![7., 8., 9.]],
            CropPolicy::Box,
        )
        .unwrap();
    let pts_3d = bfm.recon_vers(param, roi_box);

    let (param, roi_box) = bfm.run(&frame, face_box, &pts_3d, CropPolicy::Landmark).unwrap();
    let _pts_3d = bfm.recon_vers(param, roi_box);
}
