mod webcam;

use webcam::use_nokhwa;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // println!("Hello, world!");
    let mut camera = use_nokhwa::initialize_camera(0, 120, 120)?;

    loop {

        let frame = camera.frame()?;

    }

}