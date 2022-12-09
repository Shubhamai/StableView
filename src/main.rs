mod model;
mod filter;
mod utils;

use std::{
    ops::Deref,
};
use model::use_onnxruntime;
use onnxruntime::ndarray::{Array, Array4};
use onnxruntime::tensor::OrtOwnedTensor;
use opencv::{core::{Size, Vec3b}, highgui, imgproc, prelude::*, videoio, imgcodecs};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = use_onnxruntime::get_environment(&"Landmark Detection")?;
    let mut model =
        use_onnxruntime::initialize_model(&env, "./assets/mb05_120x120.onnx".to_string(), 1)?;

    let window = "video capture";
    highgui::named_window(window, highgui::WINDOW_AUTOSIZE)?;

    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?; // videoio::CAP_ANY, CAP_V4L2, // 0 is the default camera
    let opened = videoio::VideoCapture::is_opened(&cam)?;

    if !opened {
        panic!("Unable to open default camera!");
    }

    loop {
        let start_time = Instant::now();
        
        // Reading frame
        let mut frame = Mat::default();
        cam.read(&mut frame)?;

        // imgcodecs::imread("test.jpg", 1)
        //     .map(|m| frame = m)
        //     .unwrap();
        

        // If frame is valid
        if frame.size()?.width > 0 {

            
            let mut rgb_frame = Mat::default();
            imgproc::cvt_color(&frame, &mut rgb_frame, imgproc::COLOR_BGR2RGB, 0)?;

            
            // let cropped_image = Mat::roi(&frame, opencv::core::Rect {
            //     x: 366,
            //     y: 173,    
            //     width: 274,  
            //     height: 274,
            // }).unwrap();
            // [366.3932577409803, 173.43238127613657, 640.2714151105822, 447.31053864573846]

            // Resizing the frame
            let mut resized_frame = Mat::default();
            imgproc::resize(
                &rgb_frame,
                &mut resized_frame,
                Size {
                    width: 120,
                    height: 120,
                },
                0.0,
                0.0,
                imgproc::INTER_LINEAR // INTER_AREA, // https://stackoverflow.com/a/51042104 | Speed -> https://stackoverflow.com/a/44278268
            )?;

            let vec = Mat::data_typed::<Vec3b>(&resized_frame).unwrap();

            let array = Array4::from_shape_fn((1, 3, 120, 120), |(_, c, y, x)| {
                (Vec3b::deref(&vec[x + y * 120 as usize])[c] as f32 - 127.5) / 128.0
            })
            .into();


            // println!("{:?}", resized_frame.at::<Vec3b>(0).unwrap());
            // Processing the frame into a valid input of model
            // TODO : Inefficient
            // let frame_data_bytes = Mat::data_bytes(&resized_frame)?;
            // let float_image_vector: Vec<f32> = frame_data_bytes
            //     .to_vec()
            //     .iter()
            //     .map(|&e| (e as f32 - 127.5) / 128.0) //128.0 - 127.5/128.0
            //     .collect();
            // println!("{:?}", float_image_vector[0]);
            // let test_image_vector = vec![(0. as f32);43200];
            // let array = Array::from_vec(float_image_vector).into_shape([1, 3, 120, 120])?;
            // println!("{:?}", array.get((0,2,0,0)));
            let input_tensor_values = vec![array];

            // Inference
            let outputs: Vec<OrtOwnedTensor<f32, _>> = model.run(input_tensor_values)?;
            assert_eq!(outputs[0].shape(), [1, 62].as_slice());

            // println!("{:?}", outputs);
            // println!("==========================================================");

            // Showing the output
            highgui::imshow(window, &mut resized_frame)?;
        }

        let key = highgui::wait_key(10)?;
        if key > 0 && key != 255 {
            break;
        }

        // let elapsed_time = start_time.elapsed();
        // let fps = cam.get(opencv::videoio::CAP_PROP_FPS).unwrap();
        // println!("{}x{} @ {} FPS @ {} ms", frame.cols(), frame.rows(), fps, elapsed_time.as_millis());
    }
    Ok(())
}
