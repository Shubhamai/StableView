use std::ops::Deref;

use onnxruntime::ndarray::{Array3, ArrayBase, Dim, OwnedRepr};
use opencv::prelude::MatTraitConstManual;
use opencv::{
    core::{Mat, Size, Vec3b},
    imgproc,
};

use rust_faces::{
    BlazeFaceParams, Face, FaceDetection, FaceDetectorBuilder, InferParams, Provider,
};

use anyhow::{anyhow, Result};

use crate::structs::face::FaceDetect;

impl FaceDetect {
    pub fn new() -> Self {



        let face_detector =
            FaceDetectorBuilder::new(FaceDetection::BlazeFace320(BlazeFaceParams {
                score_threshold: 0.5,
                target_size: 320,

                ..Default::default()
            }))
            // .from_file(FACE_DETECTOR_MODEL)
            .download()
            .infer_params(InferParams {
                provider: Provider::OrtCuda(0),
                intra_threads: Some(5),

                ..Default::default()
            })
            .build()
            .expect("Fail to load the face detector.");

        Self { face_detector }
    }

    pub fn preprocess_frame(
        &self,
        frame: Mat,
    ) -> Result<ArrayBase<OwnedRepr<u8>, Dim<[usize; 3]>>> {
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
                width: 120,
                height: 120,
            },
            0.0,
            0.0,
            imgproc::INTER_LINEAR, //*INTER_AREA, // https://stackoverflow.com/a/51042104 | Speed -> https://stackoverflow.com/a/44278268
        )?; // ! Error handling here

        let vec = Mat::data_typed::<Vec3b>(&resized_frame)?;

        // use the shape [height, width, channels] instead of [channels, height, width].
        Ok(Array3::from_shape_fn((120, 120, 3), |(y, x, c)| {
            Vec3b::deref(&vec[x + y * 120])[c]
        }))
    }

    pub fn detect(&self, frame: Mat) -> Vec<Face> {
        let array = match self.preprocess_frame(frame) {
            Ok(array) => array,
            Err(e) => {
                tracing::error!("Error preprocessing frame: {:?}", e);
                return vec![];
            }
        };

        match self.face_detector.detect(array.view().into_dyn()) {
            Ok(faces) => {
                // convert the rect from 120x120 to 640x480

                faces
                    .iter()
                    .map(|face| {
                        let mut rect = face.rect;
                        rect.x *= 640. / 120.;
                        rect.y *= 480. / 120.;
                        rect.width *= 640. / 120.;
                        rect.height *= 480. / 120.;
                        Face {
                            rect,
                            landmarks: face.landmarks.clone(),
                            confidence: face.confidence.clone(),
                        }
                    })
                    .collect()
            }
            Err(e) => {
                tracing::error!("Error detecting faces: {:?}", e);
                vec![]
            }
        }
    }
}
