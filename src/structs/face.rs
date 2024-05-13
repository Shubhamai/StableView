use onnxruntime::session::Session;
use rust_faces::FaceDetector;

pub struct FaceDetect {
    pub face_detector: Session<'static>,
}
