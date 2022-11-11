use {nokhwa, onnxruntime};

pub mod use_nokhwa {

    use super::onnxruntime::ndarray::{Array, ArrayBase, Dim, IxDynImpl, OwnedRepr};

    use super::nokhwa::{Camera, CameraFormat, CameraInfo, FrameFormat, NokhwaError};
    #[cfg(unix)]
    use super::nokhwa::CameraIndex;


    // To get all the camera info
    pub fn _get_cameras_info() -> Result<Vec<CameraInfo>, NokhwaError> {
        nokhwa::query_devices(nokhwa::CaptureAPIBackend::Auto)
    }

    // Initializing a camera
    pub fn initialize_camera(
        index: u32,
        height: u32,
        width:u32,
        fps: u32,
    ) -> Result<Camera, Box<dyn std::error::Error>> {

        #[cfg(unix)]
        let camera_index = CameraIndex::Index(index);

        #[cfg(windows)]
        let camera_index:usize = index.try_into().unwrap();

        let mut camera = Camera::new(
            &camera_index,
            Some(CameraFormat::new_from(width, height, FrameFormat::MJPEG, fps)),
        )?;
        camera.open_stream()?;

        Ok(camera)
    }

    // Processing the input
    pub fn frame2ndarray(
        frame: Vec<u8>,
        input_shape: Vec<usize>,
    ) -> Result<ArrayBase<OwnedRepr<f32>, Dim<IxDynImpl>>, Box<dyn std::error::Error>> {
        let frame_array: Vec<f32> = frame.iter().map(|&e| e as f32).collect();

        Ok(Array::from_vec(frame_array).into_shape(input_shape)?)
    }
}
