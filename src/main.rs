mod model;
mod webcam;

use image::{imageops::resize, ImageBuffer, Rgb};
use model::use_onnxruntime;
use webcam::{initialize_nokhwa_camera, initialize_camera_capture, frame2ndarray};

use onnxruntime::tensor::OrtOwnedTensor;

use std::time::{Instant};


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Camera
    // let mut camera = initialize_nokhwa_camera(0, 120, 120, 30)?;
    let mut camera = initialize_camera_capture(0, 720, 1280, 30.0)?;

    // Model
    let env = use_onnxruntime::get_environment(&"Landmark Detection")?;
    let mut model =
        use_onnxruntime::initialize_model(&env, "./assets/mb1_120x120.onnx".to_string(), 1)?;

    loop {
        let start_time = Instant::now();

        // New frame
        // let frame = camera.frame()?;
        let frame = camera.next().unwrap();

        // Resize
        let frame_buffer:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_vec(120, 120, frame.to_vec()).unwrap();
        let resized_frame = resize(&frame_buffer, 120, 120, image::imageops::FilterType::Nearest); 

        // Processing the inputs
        let input = frame2ndarray(
            resized_frame.to_vec(),
            model.inputs[0].dimensions().map(|d| d.unwrap()).collect(),
        )?;

        // Generating the outputs
        let outputs: Vec<OrtOwnedTensor<f32, _>> = model.run(vec![input])?;

        // println!("{:?}", outputs)

        let elapsed_time = start_time.elapsed();
        println!("delay : {}", elapsed_time.as_millis());
    }
}
