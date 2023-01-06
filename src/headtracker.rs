/// Processing the head pose (filters, etc.) and generating the x,y,z of the head.
use crate::enums::crop_policy::CropPolicy;
use crate::structs::{pose::ProcessHeadPose, tddfa::Tddfa};
use crate::utils::headpose::{calc_pose, gen_point2d};

use opencv::prelude::Mat;

impl ProcessHeadPose {
    pub fn new(image_size: i32) -> Self {
        let tddfa = Tddfa::new(image_size).unwrap();

        Self {
            tddfa,
            pts_3d: vec![vec![1., 2., 3.], vec![4., 5., 6.], vec![7., 8., 9.]],
            face_box: [150., 150., 400., 400.],
        }
    }

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

    pub fn single_iter(&mut self, frame: &Mat) -> [f32; 6] {
        let (mut param, mut roi_box) = self
            .tddfa
            .run(frame, self.face_box, &self.pts_3d, CropPolicy::Landmark)
            .unwrap();

        if (roi_box[2] - roi_box[0]).abs() * (roi_box[3] - roi_box[1]).abs() < 2020. {
            (param, roi_box) = self
                .tddfa
                .run(frame, self.face_box, &self.pts_3d, CropPolicy::Box)
                .unwrap();
        }

        self.pts_3d = self.tddfa.recon_vers(param, roi_box); // ? Commenting this code still seems to output the pose perfectly

        let (p, pose) = calc_pose(&param);

        let (point2d, distance) = gen_point2d(
            &p,
            vec![
                self.pts_3d[0][28..48].to_vec(),
                self.pts_3d[1][28..48].to_vec(),
                self.pts_3d[2][28..48].to_vec(),
            ],
        );

        let (centroid, distance) = self.get_coordintes_and_depth(pose, distance, point2d, &roi_box);

        let data = [
            centroid[0],
            -centroid[1],
            distance,
            pose[0],
            -pose[1],
            pose[2],
        ];

        data
    }
}

#[test]
#[ignore = "Can only test this offline since it requires webcam, run cargo test -- --ignored"]
pub fn test_process_head_pose() {
    use crate::filter::EuroDataFilter;
    use crate::structs::{camera::ThreadedCamera, network::SocketNetwork};
    use std::sync::mpsc;

    let _euro_filter = EuroDataFilter::new(0.0025, 0.01);
    let _socket_network = SocketNetwork::new("127.0.0.1".to_owned(), "4242".to_owned());

    let (tx, _rx) = mpsc::channel();

    let mut thr_cam = ThreadedCamera::start_camera_thread(tx, 0);

    let _head_pose = ProcessHeadPose::new(120);

    thr_cam.shutdown();
}
