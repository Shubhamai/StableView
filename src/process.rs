use std::ops::Deref;

/// Processing the head pose (filters, etc.) and generating the x,y,z of the head.
use crate::enums::crop_policy::CropPolicy;
use crate::structs::face::FaceDetect;
use crate::structs::{pose::ProcessHeadPose, tddfa::Tddfa};
use crate::utils::headpose::{calc_pose, gen_point2d};
use crate::utils::image::crop_img;
use anyhow::{Context, Result};
use onnxruntime::ndarray::{Array3, Array4};
use opencv::core::{MatTraitConstManual, Scalar, Size, ToOutputArray, Vec3b};
use opencv::imgproc::{self, rectangle, LINE_4};
use opencv::prelude::Mat;
use opencv::prelude::MatTraitConst;

impl ProcessHeadPose {
    pub fn new(image_size: i32) -> Result<Self> {
        let tddfa = Tddfa::new(image_size).context("Unable to create tddfa")?;
        let face_detector = FaceDetect::new();

        Ok(Self {
            tddfa,
            face_detector,
            pts_3d: vec![vec![1., 2., 3.], vec![4., 5., 6.], vec![7., 8., 9.]],
            face_box: [150., 150., 400., 400.],
            first_iteration: true,
            param: [0.; 62],
            roi_box: [150., 150., 400., 400.],
        })
    }

    // Get the X,Y,Z coordinates of the head
    fn get_coordintes_and_depth(
        &self,
        pose: [f32; 3],
        mut distance: f32,
        _point2d: Vec<Vec<f32>>,
        roi_box: &[f32; 4],
    ) -> ([f32; 2], f32) {
        distance -= 56.;
        distance += (pose[0] * 0.2).abs();

        // let x = [point2d[0][0], point2d[1][0], point2d[2][0], point2d[3][0]];
        // let y = [point2d[0][1], point2d[1][1], point2d[2][1], point2d[3][1]];

        let mut centroid = [
            // x.iter().sum::<f32>() / (x.len()) as f32,
            // y.iter().sum::<f32>() / (y.len()) as f32,
            ((roi_box[2] + roi_box[0]) / 20.) - 40.,
            ((roi_box[3] + roi_box[1]) / 20.) - 15.,
        ];
        // * disbling the multiplying pose with distance (pose[0]*(distance/31), pose[1]*(distance/27)), it seems to causing jitting even when blinking eyes or smiling
        // centroid[0] += pose[0]; // * When very close to the camera, the head pose invariant seems to does't work, to miltgate the issue, we use this
        centroid[1] -= pose[1] * 0.15; // * 31 & 27 represent the distance where head pose invariant is fully solved, and we use this ratio to make it work for closer distance
                                       // if pose[2] > 0. {
                                       //     centroid[0] += pose[2].abs()
                                       // } else {
                                       //     centroid[0] -= pose[2].abs()
                                       // }

        (centroid, distance)
    }

    pub fn single_iter(&mut self, frame: &Mat) -> Result<[f32; 6]> {
        // ! A very tuff bug laying around somewhere here, resulting in out of ordinary roi box values when moving to camera border

        let mut return_data = [0.; 6];

        if self.first_iteration {
            (self.param, self.roi_box) =
                self.tddfa
                    .run(frame, self.face_box, &self.pts_3d, CropPolicy::Box)?;
            self.pts_3d = self.tddfa.recon_vers(self.param, self.face_box);

            (self.param, self.roi_box) =
                self.tddfa
                    .run(frame, self.face_box, &self.pts_3d, CropPolicy::Landmark)?;
            self.pts_3d = self.tddfa.recon_vers(self.param, self.face_box);

            self.first_iteration = false;
        } else {
            (self.param, self.roi_box) =
                self.tddfa
                    .run(frame, self.face_box, &self.pts_3d, CropPolicy::Landmark)?;

            if (self.roi_box[2] - self.roi_box[0]).abs() * (self.roi_box[3] - self.roi_box[1]).abs()
                < 2020.
            {
                (self.param, self.roi_box) =
                    self.tddfa
                        .run(frame, self.face_box, &self.pts_3d, CropPolicy::Box)?;
            }

            // make sure the roi_box is not out of the frame
            if self.roi_box[0] < 0. {
                self.roi_box[0] = 0.;
            }
            if self.roi_box[1] < 0. {
                self.roi_box[1] = 0.;
            }
            if self.roi_box[2] > frame.size()?.width as f32 {
                self.roi_box[2] = frame.size()?.width as f32;
            }
            if self.roi_box[3] > frame.size()?.height as f32 {
                self.roi_box[3] = frame.size()?.height as f32;
            }

            self.pts_3d = self.tddfa.recon_vers(self.param, self.roi_box);
        }
        let (p, pose) = calc_pose(&self.param);

        let (point2d, distance) = gen_point2d(
            &p,
            vec![
                self.pts_3d[0][28..48].to_vec(),
                self.pts_3d[1][28..48].to_vec(),
                self.pts_3d[2][28..48].to_vec(),
            ],
        );

        let (centroid, distance) =
            self.get_coordintes_and_depth(pose, distance, point2d, &self.roi_box);

        // detect any faces, if there are no faces, return the previous values
        let faces = self.face_detector.detect(frame.clone());

        if faces.is_empty() {
            return Ok(return_data);
        }

        // update the face box with a little bit of bigger box
        self.face_box = [
            faces[0].rect.x as f32 - 50.,
            faces[0].rect.y as f32 - 50.,
            faces[0].rect.x as f32 + faces[0].rect.width as f32 + 50.,
            faces[0].rect.y as f32 + faces[0].rect.height as f32 + 50.,
        ];

        return_data = [
            centroid[0],
            -centroid[1],
            distance,
            pose[0],
            -pose[1],
            pose[2],
        ];

        Ok(return_data)
    }
}

#[test]
#[ignore = "Can only test this offline since it requires webcam, run cargo test -- --ignored"]
#[allow(unused_variables)]
pub fn test_process_head_pose() -> Result<()> {
    use crate::structs::camera::ThreadedCamera;
    // use crate::utils::image::crop_img;
    use crate::utils::visualize::draw_landmark;
    use opencv::highgui;

    use rust_faces::{
        BlazeFaceParams, FaceDetection, FaceDetectorBuilder, InferParams, Provider, ToArray3,
        ToRgb8,
    };
    let face_detector = FaceDetectorBuilder::new(FaceDetection::BlazeFace320(BlazeFaceParams {
        score_threshold: 0.5,
        target_size: 320,
        ..Default::default()
    }))
    .download()
    .infer_params(InferParams {
        provider: Provider::OrtCuda(0),
        intra_threads: Some(5),
        ..Default::default()
    })
    .build()
    .expect("Fail to load the face detector.");

    let (tx, rx) = crossbeam_channel::unbounded::<Mat>();

    let mut thr_cam = ThreadedCamera::start_camera_thread(tx, 0, "Test Camera".to_owned())?;

    let mut head_pose = ProcessHeadPose::new(120)?;

    let window = "video capture";
    highgui::named_window(window, highgui::WINDOW_AUTOSIZE)?;

    let mut frame = rx.recv()?;
    let mut _data: [f32; 6];

    let mut frame_no = 0;
    loop {
        frame = match rx.try_recv() {
            Ok(result) => result,
            Err(_) => frame.clone(),
        };

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
        let array = Array3::from_shape_fn((120, 120, 3), |(y, x, c)| {
            Vec3b::deref(&vec[x + y * 120])[c]
        });

        let mut faces = vec![];
        // rrun detection every 10 frames
        if frame_no % 10 == 0 {
            faces = face_detector.detect(array.view().into_dyn()).unwrap();
        }

        _data = head_pose.single_iter(&frame)?;

        frame = draw_landmark(
            frame,
            vec![
                head_pose.pts_3d[0][28..48].to_vec(),
                head_pose.pts_3d[1][28..48].to_vec(),
                head_pose.pts_3d[2][28..48].to_vec(),
            ],
            head_pose.roi_box,
            (0., 255., 0.),
            1,
        )?;

        // add bbox of face detection
        for face in faces {
            let rect = face.rect;

            // convert the rect from 120x120 to 640x480
            let rect = opencv::core::Rect::new(
                (rect.x * 5.33) as i32,
                (rect.y * 4.) as i32,
                (rect.width * 5.33) as i32,
                (rect.height * 4.) as i32,
            );

            let landmarks = face.landmarks.unwrap();
            let confidence = face.confidence;

            // println!("Face detected: {:?}", rect);
            // println!("Confidence: {}", confidence);
            // println!("Landmarks: {:?}", landmarks);

            imgproc::rectangle(
                &mut frame,
                rect,
                opencv::core::Scalar::from((0.0, 255.0, 0.0)),
                2,
                imgproc::LINE_4,
                0,
            )?;
        }

        if frame.size()?.width > 0 {
            // let cropped_image = crop_img(&frame, &head_pose.roi_box)?;

            // Resizing the frame
            // let mut resized_frame = Mat::default();
            // imgproc::resize(
            //     &cropped_image,
            //     &mut resized_frame,
            //     Size {
            //         width: 120,
            //         height: 120,
            //     },
            //     0.0,
            //     0.0,
            //     imgproc::INTER_LINEAR, //*INTER_AREA, // https://stackoverflow.com/a/51042104 | Speed -> https://stackoverflow.com/a/44278268
            // )?; // ! Error handling here

            highgui::imshow(window, &frame)?;
        }
        let key = highgui::wait_key(30)?;
        if key > 0 && key != 255 {
            break;
        }

        frame_no += 1;
    }
    thr_cam.shutdown();
    Ok(())
}
