use opencv::{
    core::{Mat, Point, Point2i, Scalar},
    imgproc::{circle, line, LINE_8},
};

pub fn draw_landmark(
    frame: Mat,
    pts_3d: Vec<Vec<f32>>,
    face_box: [f32; 4],
    color: (f64, f64, f64),
    size: i32,
) -> Mat {
    let mut img = frame.clone();
    let n = pts_3d[0].len();
    if n <= 106 {
        for i in 0..n {
            circle(
                &mut img,
                Point::new(pts_3d[0][i] as i32, pts_3d[1][i] as i32),
                size,
                Scalar::new(color.0, color.1, color.2, 0.),
                -1,
                LINE_8,
                0,
            )
            .unwrap()
        }
    } else {
        let sep = 1;
        for i in (0..n).step_by(sep) {
            circle(
                &mut img,
                Point::new(pts_3d[0][i] as i32, pts_3d[1][i] as i32),
                size,
                Scalar::new(color.0, color.1, color.2, 0.),
                1,
                LINE_8,
                0,
            )
            .unwrap()
        }
    };

    let left = face_box[0].round() as i32;
    let top = face_box[1].round() as i32;
    let right = face_box[2].round() as i32;
    let bottom = face_box[3].round() as i32;
    let left_top = Point2i::new(left, top);
    let right_top = Point2i::new(right, top);
    let right_bottom = Point2i::new(right, bottom);
    let left_bottom = Point2i::new(left, bottom);
    line(
        &mut img,
        left_top,
        right_top,
        Scalar::new(0., 0., 255., 0.),
        1,
        LINE_8,
        0,
    )
    .unwrap();
    line(
        &mut img,
        right_top,
        right_bottom,
        Scalar::new(0., 0., 255., 0.),
        1,
        LINE_8,
        0,
    )
    .unwrap();
    line(
        &mut img,
        right_bottom,
        left_bottom,
        Scalar::new(0., 0., 255., 0.),
        1,
        LINE_8,
        0,
    )
    .unwrap();
    line(
        &mut img,
        left_bottom,
        left_top,
        Scalar::new(0., 0., 255., 0.),
        1,
        LINE_8,
        0,
    )
    .unwrap();

    img
}
