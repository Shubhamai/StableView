
/// Processing the head pose (filters, etc.) and generating the x,y,z of the head.
use crate::enums::crop_policy::CropPolicy;
use crate::structs::face::FaceDetect;
use crate::structs::{pose::ProcessHeadPose, tddfa::Tddfa};
use crate::utils::headpose::{calc_pose, gen_point2d};
use anyhow::{Context, Result};
use opencv::prelude::Mat;
use opencv::prelude::MatTraitConst;

impl ProcessHeadPose {
    pub fn new(image_size: i32) -> Result<Self> {
        let tddfa = Tddfa::new(image_size).context("Unable to create tddfa")?;
        let face_detector = FaceDetect::new().unwrap();

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
        // let mut rng = rand::thread_rng();
        // if rng.gen::<f64>() > 0.9 {
        let face_detected = self.face_detector.detect(frame.clone()).unwrap();

        if face_detected[0] < 1. {
            return Ok(return_data);
        }
        self.face_box = [
            face_detected[0] - 50.,
            face_detected[1] - 50.,
            face_detected[0] + face_detected[2] + 50.,
            face_detected[1] + face_detected[3] + 50.,
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

    let (tx, rx) = crossbeam_channel::unbounded::<Mat>();
    let mut thr_cam = ThreadedCamera::start_camera_thread(tx, 0, "Test Camera".to_owned())?;

    let mut face_detector = FaceDetect::new().unwrap();
    let mut head_pose = ProcessHeadPose::new(120)?;

    let window = "video capture";
    highgui::named_window(window, highgui::WINDOW_AUTOSIZE)?;

    let mut frame = rx.recv()?;

    loop {
        frame = match rx.try_recv() {
            Ok(result) => result,
            Err(_) => frame.clone(),
        };

        let data = head_pose.single_iter(&frame)?;

        // frame = draw_landmark(
        //     frame,
        //     vec![
        //         head_pose.pts_3d[0][28..48].to_vec(),
        //         head_pose.pts_3d[1][28..48].to_vec(),
        //         head_pose.pts_3d[2][28..48].to_vec(),
        //     ],
        //     head_pose.roi_box,
        //     (0., 255., 0.),
        //     1,
        // )?;

        // let detected_faces = face_detector.detect(frame.clone()).unwrap();

        // // draw point
        // imgproc::circle(
        //     &mut frame,
        //     opencv::core::Point::new(
        //         (detected_faces[0]).try_into().unwrap(),
        //         (detected_faces[1]).try_into().unwrap(),
        //     ),
        //     5,
        //     opencv::core::Scalar::from((0.0, 0.0, 255.0)),
        //     2,
        //     imgproc::LINE_4,
        //     0,
        // )?;

        // imgproc::circle(
        //     &mut frame,
        //     opencv::core::Point::new(
        //         (detected_faces[0] + detected_faces[2]).try_into().unwrap(),
        //         (detected_faces[1] + detected_faces[3]).try_into().unwrap(),
        //     ),
        //     5,
        //     opencv::core::Scalar::from((0.0, 0.0, 255.0)),
        //     2,
        //     imgproc::LINE_4,
        //     0,
        // )?;

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
    }
    thr_cam.shutdown();
    Ok(())
}
