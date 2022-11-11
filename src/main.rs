mod model;
mod webcam;

use model::use_onnxruntime;
use webcam::use_nokhwa;

use onnxruntime::tensor::OrtOwnedTensor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Camera
    let mut camera = use_nokhwa::initialize_camera(0, 120, 120, 30)?;

    // Model
    let env = use_onnxruntime::get_environment(&"Landmark Detection")?;
    let mut model =
        use_onnxruntime::initialize_model(&env, "./assets/mb1_120x120.onnx".to_string(), 1)?;

    loop {
        // New frame
        let frame = camera.frame()?;

        // Processing the inputs
        let input = use_nokhwa::frame2ndarray(
            frame.to_vec(),
            model.inputs[0].dimensions().map(|d| d.unwrap()).collect(),
        )?;

        // Generating the outputs
        let outputs: Vec<OrtOwnedTensor<f32, _>> = model.run(vec![input])?;

        println!("{:?}", outputs)
    }
}
