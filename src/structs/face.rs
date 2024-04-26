use rust_faces::FaceDetector;

pub struct FaceDetect {
    pub face_detector: Box<dyn FaceDetector>,
}
