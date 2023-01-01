/// Utility function for processing image
/// Python source - https://github.com/cleardusk/3DDFA/blob/d5c1f6a647a89070b1f9ea4e88c910b743a1a87a/utils/inference.py#L20
use opencv::{
    core::{Mat, Rect},
    prelude::MatTraitConstManual,
};

pub fn crop_img(img: &Mat, roi_box: &[f32; 4]) -> Mat {
    let h = img.size().unwrap().height;
    let w = img.size().unwrap().width;

    let sx = roi_box[0].round() as i32;
    let sy = roi_box[1].round() as i32;
    let ex = roi_box[2].round() as i32;
    let ey = roi_box[3].round() as i32;

    let dh = ey - sy;
    let dw = ex - sx;

    let (mut sx, _) = if sx < 0 { (0, -sx) } else { (sx, 0) };
    let (ex, _) = if ex > w { (w, dw - (ex - w)) } else { (ex, dw) };
    let (mut sy, _) = if sy < 0 { (0, -sy) } else { (sy, 0) };
    let (ey, _) = if ey > h { (h, dh - (ey - h)) } else { (ey, dh) };

    let mut width = ex - sx;
    let mut height = ey - sy;
    if ex - sx < 0 {
        width = 1;
    }
    if ey - sy < 0 {
        height = 1;
    }
    if sy > h - 1 {
        sy = h - 1;
    }
    if sx > w - 1 {
        sx = w - 1;
    }

    let roi = Rect::new(sx, sy, width, height);
    Mat::roi(img, roi).unwrap() // ! Need to deal with this
}

#[test]
fn test_crop_img() {
    use opencv::{
        core::{Scalar, CV_8UC3},
        prelude::MatTraitConst,
    };

    let frame =
        Mat::new_rows_cols_with_default(120, 120, CV_8UC3, Scalar::new(255., 0., 0., 0.)).unwrap();
    let roi_box = [50., 60., 100., 120.];
    let result = crop_img(&frame, &roi_box);

    assert_eq!(result.rows() as f32, roi_box[3] - roi_box[1]);
    assert_eq!(result.cols() as f32, roi_box[2] - roi_box[0]);

    let roi_box = [50., 60., 400., 400.];
    let result = crop_img(&frame, &roi_box);

    assert_eq!(result.rows() as f32, 60.);
    assert_eq!(result.cols() as f32, 70.);
}
