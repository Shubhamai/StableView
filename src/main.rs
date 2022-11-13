mod model;

use model::use_onnxruntime;
use onnxruntime::ndarray::Array;
use onnxruntime::tensor::OrtOwnedTensor;
use opencv::{core, highgui, imgproc, prelude::*, videoio};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = use_onnxruntime::get_environment(&"Landmark Detection")?;
    let mut model =
        use_onnxruntime::initialize_model(&env, "./assets/mb05_120x120.onnx".to_string(), 1)?;

    // let window = "video capture";
    // highgui::named_window(window, highgui::WINDOW_AUTOSIZE)?;

    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_V4L2)?; // videoio::CAP_ANY, CAP_V4L2, // 0 is the default camera
    let opened = videoio::VideoCapture::is_opened(&cam)?;

    if !opened {
        panic!("Unable to open default camera!");
    }

    loop {
        let start_time = Instant::now();
        
        // Reading frame
        let mut frame = Mat::default();
        cam.read(&mut frame)?;

        // If frame is valid
        if frame.size()?.width > 0 {

            // Resizingt he frame
            let mut resized_frame = Mat::default();
            imgproc::resize(
                &frame,
                &mut resized_frame,
                core::Size {
                    width: 120,
                    height: 120,
                },
                0.0,
                0.0,
                imgproc::INTER_AREA, // https://stackoverflow.com/a/51042104 | Speed -> https://stackoverflow.com/a/44278268
            )?;

            // Processing the frame into a valid input of model
            // TODO : Inefficient
            let frame_data_bytes = Mat::data_bytes(&resized_frame)?;
            let float_image_vector: Vec<f32> = frame_data_bytes
                .to_vec()
                .iter()
                .map(|&e| e as f32)
                .collect();
            let array = Array::from_vec(float_image_vector).into_shape([1, 3, 120, 120])?;
            let input_tensor_values = vec![array];

            // Inference
            let outputs: Vec<OrtOwnedTensor<f32, _>> = model.run(input_tensor_values)?;
            assert_eq!(outputs[0].shape(), [1, 62].as_slice());

            // Showing the output
            // highgui::imshow(window, &mut frame)?;
        }

        // let key = highgui::wait_key(10)?;
        // if key > 0 && key != 255 {
        //     break;
        // }

        // let elapsed_time = start_time.elapsed();
        // let fps = cam.get(opencv::videoio::CAP_PROP_FPS).unwrap();
        // println!("{}x{} @ {} FPS @ {} ms", frame.cols(), frame.rows(), fps, elapsed_time.as_millis());
    }
    Ok(())
}
