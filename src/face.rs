use std::ops::Deref;

use onnxruntime::environment::Environment;
use onnxruntime::ndarray::{Array4, ArrayBase, Axis, Dim, OwnedRepr};
use onnxruntime::tensor::OrtOwnedTensor;
use onnxruntime::GraphOptimizationLevel;
use opencv::prelude::MatTraitConstManual;
use opencv::{
    core::{Mat, Size, Vec3b},
    imgproc,
};

use itertools::Itertools;
use once_cell::sync::Lazy;

use anyhow::Result;

use crate::consts::BLAZE_FACE_MODEL;
use crate::structs::face::FaceDetect;

use itertools::iproduct;

use std::collections::HashMap;

/// Face detection result.
#[derive(Debug, Clone)]
pub struct Face {
    /// Face's bounding rectangle.
    pub rect: Rect,
    /// Confidence of the detection.
    pub confidence: f32,
    /// Landmarks of the face.
    pub landmarks: Option<Vec<(f32, f32)>>,
}

/// Non-maximum suppression.
#[derive(Copy, Clone, Debug)]
pub struct Nms {
    pub iou_threshold: f32,
}

impl Default for Nms {
    fn default() -> Self {
        Self { iou_threshold: 0.3 }
    }
}

impl Nms {
    /// Suppress non-maxima faces.
    ///
    /// # Arguments
    ///
    /// * `faces` - Faces to suppress.
    ///
    /// # Returns
    ///
    /// * `Vec<Face>` - Suppressed faces.
    pub fn suppress_non_maxima(&self, mut faces: Vec<Face>) -> Vec<Face> {
        faces.sort_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap());

        let mut faces_map = HashMap::new();
        faces.iter().rev().enumerate().for_each(|(i, face)| {
            faces_map.insert(i, face);
        });

        let mut nms_faces = Vec::with_capacity(faces.len());
        let mut count = 0;
        while !faces_map.is_empty() {
            if let Some((_, face)) = faces_map.remove_entry(&count) {
                nms_faces.push(face.clone());
                //faces_map.retain(|_, face2| face.rect.iou(&face2.rect) < self.iou_threshold);
                faces_map.retain(|_, face2| face.rect.iou(&face2.rect) < self.iou_threshold);
            }
            count += 1;
        }

        nms_faces
    }

    /// Suppress non-maxima faces.
    ///
    /// # Arguments
    ///
    /// * `faces` - Faces to suppress.
    ///
    /// # Returns
    ///
    /// * `Vec<Face>` - Suppressed faces.
    pub fn suppress_non_maxima_min(&self, mut faces: Vec<Face>) -> Vec<Face> {
        faces.sort_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap());

        let mut faces_map = HashMap::new();
        faces.iter().rev().enumerate().for_each(|(i, face)| {
            faces_map.insert(i, face);
        });

        let mut nms_faces = Vec::with_capacity(faces.len());
        let mut count = 0;
        while !faces_map.is_empty() {
            if let Some((_, face)) = faces_map.remove_entry(&count) {
                nms_faces.push(face.clone());
                //faces_map.retain(|_, face2| face.rect.iou(&face2.rect) < self.iou_threshold);
                faces_map.retain(|_, face2| face.rect.iou_min(&face2.rect) < self.iou_threshold);
            }
            count += 1;
        }

        nms_faces
    }
}

#[derive(Debug, Clone)]
pub struct PriorBoxesParams {
    min_sizes: Vec<Vec<usize>>,
    steps: Vec<usize>,
    variance: (f32, f32),
}

impl Default for PriorBoxesParams {
    fn default() -> Self {
        Self {
            min_sizes: vec![vec![8, 11], vec![14, 19, 26, 38, 64, 149]],
            steps: vec![8, 16],
            variance: (0.1, 0.2),
        }
    }
}

pub struct PriorBoxes {
    pub anchors: Vec<(f32, f32, f32, f32)>,
    variances: (f32, f32),
}

impl PriorBoxes {
    pub fn new(params: &PriorBoxesParams, image_size: (usize, usize)) -> Self {
        let feature_map_sizes: Vec<(usize, usize)> = params
            .steps
            .iter()
            .map(|&step| (image_size.0 / step, image_size.1 / step))
            .collect();

        let mut anchors = Vec::new();

        for ((f, min_sizes), step) in feature_map_sizes
            .iter()
            .zip(params.min_sizes.iter())
            .zip(params.steps.iter())
        {
            let step = *step;
            for (i, j) in iproduct!(0..f.1, 0..f.0) {
                for min_size in min_sizes {
                    let s_kx = *min_size as f32 / image_size.0 as f32;
                    let s_ky = *min_size as f32 / image_size.1 as f32;
                    let cx = (j as f32 + 0.5) * step as f32 / image_size.0 as f32;
                    let cy = (i as f32 + 0.5) * step as f32 / image_size.1 as f32;
                    anchors.push((cx, cy, s_kx, s_ky));
                }
            }
        }

        Self {
            anchors,
            variances: params.variance,
        }
    }

    pub fn decode_box(&self, prior: &(f32, f32, f32, f32), pred: &(f32, f32, f32, f32)) -> Rect {
        let (anchor_cx, anchor_cy, s_kx, s_ky) = prior;
        let (x1, y1, x2, y2) = pred;

        let cx = anchor_cx + x1 * self.variances.0 * s_kx;
        let cy = anchor_cy + y1 * self.variances.0 * s_ky;
        let width = s_kx * (x2 * self.variances.1).exp();
        let height = s_ky * (y2 * self.variances.1).exp();
        let x_start = cx - width / 2.0;
        let y_start = cy - height / 2.0;
        Rect::at(x_start, y_start).ending_at(width + x_start, height + y_start)
    }

    pub fn decode_landmark(
        &self,
        prior: &(f32, f32, f32, f32),
        landmark: (f32, f32),
    ) -> (f32, f32) {
        let (anchor_cx, anchor_cy, s_kx, s_ky) = prior;
        let (x, y) = landmark;
        let x = anchor_cx + x * self.variances.0 * s_kx;
        let y = anchor_cy + y * self.variances.0 * s_ky;
        (x, y)
    }
}

use std::fmt::Display;

/// Rectangle.
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    /// X coordinate of the top-left corner.
    pub x: f32,
    /// Y coordinate of the top-left corner.
    pub y: f32,
    /// Width of the rectangle.
    pub width: f32,
    /// Height of the rectangle.
    pub height: f32,
}

/// Rectangle position used for chaining constructors.
pub struct RectPosition {
    pub x: f32,
    pub y: f32,
}

impl RectPosition {
    /// Makes a rectangle with the given size.
    pub fn with_size(&self, width: f32, height: f32) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            width,
            height,
        }
    }

    /// Makes a rectangle with the given end point.
    pub fn ending_at(&self, x: f32, y: f32) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            width: x - self.x,
            height: y - self.y,
        }
    }
}

impl Rect {
    /// Starts a rectangle with the given position.
    pub fn at(x: f32, y: f32) -> RectPosition {
        RectPosition { x, y }
    }

    /// Right end of the rectangle.
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Bottom end of the rectangle.
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Unites two rectangles.
    ///
    /// # Arguments
    ///
    /// * `other` - Other rectangle to unite with.
    ///
    /// # Returns
    ///
    /// * `Rect` - United rectangle.
    pub fn union(&self, other: &Rect) -> Rect {
        let left = self.x.min(other.x);
        let right = self.right().max(other.right());
        let top = self.y.min(other.y);
        let bottom = self.bottom().max(other.bottom());

        Rect {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        }
    }

    /// Intersects two rectangles.
    ///
    /// # Arguments
    ///
    /// * `other` - Other rectangle to intersect with.
    ///
    /// # Returns
    ///
    /// * `Rect` - Intersected rectangle.
    pub fn intersection(&self, other: &Rect) -> Rect {
        let left = self.x.max(other.x);
        let right = self.right().min(other.right());
        let top = self.y.max(other.y);
        let bottom = self.bottom().min(other.bottom());

        Rect {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        }
    }

    /// Clamps the rectangle to the given rect.
    /// If the rectangle is larger than the given size, it will be shrunk.
    ///
    /// # Arguments
    ///
    /// * `width` - Width to clamp to.
    /// * `height` - Height to clamp to.
    pub fn clamp(&self, width: f32, height: f32) -> Rect {
        let left = self.x.max(0.0);
        let right = self.right().min(width);
        let top = self.y.max(0.0);
        let bottom = self.bottom().min(height);

        Rect {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        }
    }

    /// Calculates the intersection over union of two rectangles.
    ///
    /// # Arguments
    ///
    /// * `other` - Other rectangle to calculate the intersection over union with.
    ///
    /// # Returns
    ///
    /// * `f32` - Intersection over union.
    pub fn iou(&self, other: &Rect) -> f32 {
        let left = self.x.max(other.x);
        let right = (self.right()).min(other.right());
        let top = self.y.max(other.y);
        let bottom = (self.bottom()).min(other.bottom());

        let intersection = (right - left).max(0.0) * (bottom - top).max(0.0);
        let area_self = self.width * self.height;
        let area_other = other.width * other.height;

        intersection / (area_self + area_other - intersection)
    }

    /// Calculates the intersection over union of two rectangles.
    ///
    /// # Arguments
    ///
    /// * `other` - Other rectangle to calculate the intersection over union with.
    ///
    /// # Returns
    ///
    /// * `f32` - Intersection over union.
    pub fn iou_min(&self, other: &Rect) -> f32 {
        let left = self.x.max(other.x);
        let right = (self.right()).min(other.right());
        let top = self.y.max(other.y);
        let bottom = (self.bottom()).min(other.bottom());

        let intersection = (right - left).max(0.0) * (bottom - top).max(0.0);
        let area_self = self.width * self.height;
        let area_other = other.width * other.height;

        intersection / area_self.min(area_other)
    }

    /// Scales the rectangle.
    pub fn scale(&self, x_scale: f32, y_scale: f32) -> Rect {
        Rect {
            x: self.x * x_scale,
            y: self.y * y_scale,
            width: self.width * x_scale,
            height: self.height * y_scale,
        }
    }

    /// Gets the rectangle as a tuple of (x, y, width, height).
    pub fn to_xywh(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }
}

impl Display for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{x: {}, y: {}, width: {}, height: {}}}",
            self.x, self.y, self.width, self.height
        )
    }
}

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
        let bgr_frame = frame;

        // bgr to rgb on new frame
        // let mut bgr_frame = Mat::default();
        // imgproc::cvt_color(&frame, &mut bgr_frame, imgproc::COLOR_BGR2RGB, 0)?;

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

    pub fn detect(&mut self, frame: Mat) -> Result<[f32; 4]> {
        // -> Result<bool> {
        // -> Result<Vec<Face>> {
        let array = match self.preprocess_frame(frame) {
            Ok(array) => vec![array],
            Err(e) => {
                tracing::error!("Error preprocessing frame: {:?}", e);
                // return Ok(vec![]);
                panic!("Error preprocessing frame: {:?}", e);
            }
        };
        let output_tensors: Vec<OrtOwnedTensor<f32, _>> = self.face_detector.run(array)?;
        let boxes = &output_tensors[0];
        let scores = &output_tensors[1];
        let num_boxes = boxes.view().shape()[1];
        let input_width = 320;
        let input_height = 320;
        let priors = PriorBoxes::new(
            &PriorBoxesParams::default(),
            (input_width as usize, input_height as usize),
        );

        let ratio = 320. / 640.0;
        let scale_ratios = (input_width as f32 / ratio, input_height as f32 / ratio);

        // let faces: Vec<[f32; 4]> =
        let faces: Vec<Face> = boxes
            .view()
            .to_shape((num_boxes, 4))
            .unwrap()
            .axis_iter(Axis(0))
            .zip(priors.anchors.iter())
            .zip(
                scores
                    .view()
                    .to_shape((num_boxes, 2))
                    .unwrap()
                    .axis_iter(Axis(0)),
            )
            .filter_map(|((rect, prior), score)| {
                let score = score[1];

                if score > 0.5 {
                    let rect = priors.decode_box(prior, &(rect[0], rect[1], rect[2], rect[3]));
                    let rect = rect.scale(scale_ratios.0, scale_ratios.1);

                    // Some([rect.x, rect.y, rect.width, rect.height])
                    Some(Face {
                        rect,
                        landmarks: None,
                        confidence: score,
                    })
                } else {
                    None
                }
            })
            .collect_vec();

        let nms = Nms::default();
        let nms_faces = nms.suppress_non_maxima(faces);

        if !nms_faces.is_empty() {
            return Ok([
                (nms_faces[0].rect.x),      // * 640.) / 120.,
                (nms_faces[0].rect.y),      // * 480.) / 120.,
                (nms_faces[0].rect.width),  // * 640.) / 120.,
                (nms_faces[0].rect.height), // * 480.) / 120.,
            ]);
        } else {
            return Ok([0.0; 4]);
        }
    }
}
