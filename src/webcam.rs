use { nokhwa };

pub mod use_nokhwa {

    use super::nokhwa::{Camera, CameraFormat, CameraIndex, CameraInfo, FrameFormat, NokhwaError};

    // To get all the camera info
    pub fn get_cameras_info() -> Result<Vec<CameraInfo>, NokhwaError> {
        nokhwa::query_devices(nokhwa::CaptureAPIBackend::Auto)
    }

    // Initializing a camera
    pub fn initialize_camera(
        index: u32,
        size: u32,
        fps: u32,
    ) -> Result<Camera, Box<dyn std::error::Error>> {
        let mut camera = Camera::new(
            &CameraIndex::Index(index),
            // 0,
            Some(CameraFormat::new_from(size, size, FrameFormat::MJPEG, fps)),
        )?;
        camera.open_stream()?;

        Ok(camera)
    }

}
