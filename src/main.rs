mod model;
mod filter;
mod utils;
mod tddfa;
use std::{
    ops::Deref,
};
use model::OnnxSessionsManager;
use onnxruntime::ndarray::{Array, Array4};
use onnxruntime::tensor::OrtOwnedTensor;
use opencv::{core::{Size, Vec3b}, highgui, imgproc, prelude::*, videoio, imgcodecs};
use tddfa::TDDFA;
use std::time::Instant;

use crate::utils::calc_pose;


use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::mem::transmute;



fn main() -> Result<(), Box<dyn std::error::Error>> {

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4242);
let mut buf = [0; 8 * 6];

let socket_network = UdpSocket::bind("0.0.0.0:0").expect("failed to bind UDP socket");




    let env = OnnxSessionsManager::get_environment(&"Landmark Detection")?;
    let mut model =
    OnnxSessionsManager::initialize_model(&env, "./assets/mb05_120x120.onnx".to_string(), 1)?;

    // let window = "video capture";
    // highgui::named_window(window, highgui::WINDOW_AUTOSIZE)?;

    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?; // videoio::CAP_ANY, CAP_V4L2, // 0 is the default camera
    let opened = videoio::VideoCapture::is_opened(&cam)?;

    if !opened {
        panic!("Unable to open default camera!");
    }

    let tddfa = TDDFA::new(
        // bfm_onnx_fp,
        "./assets/data.json",
        "./assets/mb05_120x120.onnx",
        120,
     
        )?; 
    let face_box = [150., 150., 400., 400.];

    
    
    // Reading frame
    let mut frame = Mat::default();
    cam.read(&mut frame)?;
    
    let (param, roi_box) = tddfa.run(&frame, face_box, vec![vec![1., 2., 3.], vec![4., 5., 6.], vec![7., 8., 9.]], "box").unwrap();
    let pts_3d = tddfa.recon_vers(param, roi_box);

    
    let (param, roi_box) = tddfa.run(&frame, face_box, pts_3d, "landmark").unwrap();
    let pts_3d = tddfa.recon_vers(param, roi_box);

    loop {
        let start_time = Instant::now();
        
        // Reading frame
        let mut frame = Mat::default();
        cam.read(&mut frame)?;

        // If frame is valid
        if frame.size()?.width > 0 {

            let (param, roi_box) = tddfa.run(&frame, face_box, pts_3d.clone(), "landmark").unwrap();
            let pts_3d = tddfa.recon_vers(param, roi_box);
            
            
            let (P, pose) = calc_pose(&param);

            println!("{:?}", pose);

            // Showing the output
            // highgui::imshow(window, &mut frame)?;

            // let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
            let data = [1.0, 2.0, 3.0, pose[0] as f64, -pose[1] as f64, pose[2] as f64];

// sensivity_data
unsafe {
    let ptr = buf.as_mut_ptr() as *mut [f64; 6];
    *ptr = data;
}

socket_network.send_to(&buf, &address).expect("failed to send data");

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
