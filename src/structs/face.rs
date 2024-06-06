use onnxruntime::session::Session;

pub struct FaceDetect {
    pub face_detector: Session<'static>,
}
