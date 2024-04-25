use super::{face::FaceDetect, tddfa::Tddfa};

pub struct ProcessHeadPose {
    pub tddfa: Tddfa,
    pub face_detector: FaceDetect,
    pub pts_3d: Vec<Vec<f32>>,
    pub face_box: [f32; 4],
    pub first_iteration: bool,
    pub param: [f32; 62],
    pub roi_box: [f32; 4],
}
