use std::ops::Deref;

use onnxruntime::environment::Environment;
use onnxruntime::ndarray::{Array3, Array4, ArrayBase, Dim, OwnedRepr};
use onnxruntime::tensor::OrtOwnedTensor;
use onnxruntime::GraphOptimizationLevel;
use opencv::prelude::MatTraitConstManual;
use opencv::{
    core::{Mat, Size, Vec3b},
    imgproc,
};

use once_cell::sync::Lazy;

// use rust_faces::{
//     BlazeFaceParams, Face, FaceDetection, FaceDetectorBuilder, InferParams, Provider,
// };

use anyhow::{anyhow, Result};

use crate::structs::face::FaceDetect;

pub const BLAZE_FACE_MODEL: &[u8] = include_bytes!("../assets/model/blazeface-320.onnx");

impl FaceDetect {
    pub fn new() -> Result<Self> {
        static ENVIRONMENT: Lazy<Environment> = Lazy::new(|| {
            match Environment::builder()
                .with_name("Face Detector")
                .with_log_level(onnxruntime::LoggingLevel::Warning)
                .build()
            {
                Ok(environment) => environment,
                Err(error) => {
                    tracing::error!("Unable to create environment : {:?}", error);
                    std::process::exit(1);
                }
            }
        });

        let face_detector = ENVIRONMENT
            .new_session_builder()?
            .with_optimization_level(GraphOptimizationLevel::All)?
            .with_number_threads(1)?
            .with_model_from_memory(BLAZE_FACE_MODEL)?;

        Ok(Self { face_detector })
    }

    pub fn preprocess_frame(
        &self,
        frame: Mat,
    ) -> Result<ArrayBase<OwnedRepr<f32>, Dim<[usize; 4]>>> {
        // bgr to rgb on new frame
        let mut bgr_frame = Mat::default();
        imgproc::cvt_color(&frame, &mut bgr_frame, imgproc::COLOR_BGR2RGB, 0)?;

        // let cropped_image = crop_img(&bgr_frame, &[150., 150., 400., 400.])?;

        // Resizing the frame
        let mut resized_frame = Mat::default();
        imgproc::resize(
            &bgr_frame,
            &mut resized_frame,
            Size {
                width: 320,
                height: 320,
            },
            0.0,
            0.0,
            imgproc::INTER_LINEAR, //*INTER_AREA, // https://stackoverflow.com/a/51042104 | Speed -> https://stackoverflow.com/a/44278268
        )?; // ! Error handling here

        let vec = Mat::data_typed::<Vec3b>(&resized_frame)?;

        // use the shape [height, width, channels] instead of [channels, height, width].
        // Ok(Array3::from_shape_fn((120, 120, 3), |(y, x, c)| {
        //     Vec3b::deref(&vec[x + y * 120])[c]
        // }))

        Ok(Array4::from_shape_fn((1, 3, 320, 320), |(_, c, y, x)| {
            f32::from(Vec3b::deref(&vec[x + y * 320])[c])
        }))
    }

    pub fn detect(&mut self, frame: Mat) -> Result<Vec<u8>> {
        let array = match self.preprocess_frame(frame) {
            Ok(array) => vec![array],
            Err(e) => {
                tracing::error!("Error preprocessing frame: {:?}", e);
                return Ok(vec![]);
            }
        };

        let output_tensors: Vec<OrtOwnedTensor<f32, _>> = self.face_detector.run(array)?;

        // let boxes: OrtOwnedTensor<f32, _> = output_tensors[0].try_extract()?;
        
        // match self.face_detector.detect(array.view().into_dyn()) {
        //     Ok(faces) => {
        //         // convert the rect from 120x120 to 640x480

        //         faces
        //             .iter()
        //             .map(|face| {
        //                 let mut rect = face.rect;
        //                 rect.x *= 640. / 120.;
        //                 rect.y *= 480. / 120.;
        //                 rect.width *= 640. / 120.;
        //                 rect.height *= 480. / 120.;
        //                 Face {
        //                     rect,
        //                     landmarks: face.landmarks.clone(),
        //                     confidence: face.confidence.clone(),
        //                 }
        //             })
        //             .collect()
        //     }
        //     Err(e) => {
        //         tracing::error!("Error detecting faces: {:?}", e);
        //         vec![]
        //     }
        // }

        Ok(vec![])
    }
}
