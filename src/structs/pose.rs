use super::tddfa::Tddfa;

pub struct ProcessHeadPose {
    pub tddfa: Tddfa,
    pub pts_3d: Vec<Vec<f32>>,
    pub face_box: [f32; 4],
}
