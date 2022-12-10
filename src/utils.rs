// Importing Libraries
// use onnxruntime::ndarray::{Array, Array1, Array2, ArrayView, ArrayViewMut, Axis};
// use ndarray_linalg::Norm;
// use opencv::core::{Mat}; // , MatTrait, Scalar

use opencv::{
    core::{Mat, Rect, CV_32F},
    imgcodecs,
    highgui,
    imgproc,
    prelude::*,
};


// use std::error::Error;
// use onnxruntime::ndarray::{Array, ArrayView};
// use opencv::prelude::*;
// use opencv::{
//     highgui, imgcodecs,
//     imgproc::{self, resize, InterpolationFlags},
// };
// use std::cmp::{max, min};
// use std::f32::consts::SQRT_2;

// use std::f64::consts::PI;

use std::f64::consts::{FRAC_PI_2, PI};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////// TDDFA ////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn _parse_param(
    param: &[f32;62],
) -> Result<([[f32; 3]; 3], [[f32; 1]; 3], [[f32; 1]; 40], [[f32; 1]; 10]), &'static str> {
    let n = param.len();

    let (trans_dim, shape_dim, exp_dim) = match n {
        62 => (12, 40, 10),
        72 => (12, 40, 20),
        141 => (12, 100, 29),
        _ => return Err("Undefined templated param parsing rule"),
    };

    let R_ = [
        [param[0], param[1], param[2], param[3]],
        [param[4], param[5], param[6], param[7]],
        [param[8], param[9], param[10], param[11]],
    ];

    let R = [
        [R_[0][0], R_[0][1], R_[0][2]],
        [R_[1][0], R_[1][1], R_[1][2]],
        [R_[2][0], R_[2][1], R_[2][2]],
    ];

    let offset = [[R_[0][3]], [R_[1][3]], [R_[2][3]]];

    let mut alpha_shp = [[0.0; 1]; 40];
    for i in 0..40 {
        alpha_shp[i][0] = param[trans_dim + i];
    }

    let mut alpha_exp = [[0.0; 1]; 10];
    for i in 0..10 {
        alpha_exp[i][0] = param[trans_dim + shape_dim + i];
    }

    Ok((R, offset, alpha_shp, alpha_exp))
}

// fn _parse_param(param: &[f64]) -> Result<(Vec<Vec<f64>>, Vec<Vec<f64>>, Vec<Vec<f64>>, Vec<Vec<f64>>), &'static str> {
//     let n = param.len();

//     let (trans_dim, shape_dim, exp_dim) = match n {
//         62 => (12, 40, 10),
//         72 => (12, 40, 20),
//         141 => (12, 100, 29),
//         _ => return Err("Undefined templated param parsing rule")
//     };

//     let R_ = param[..trans_dim].chunks(4).map(|chunk| chunk.to_vec()).collect::<Vec<_>>();
//     let R = R_.iter().map(|row| row[..3].to_vec()).collect::<Vec<_>>();
//     let offset = R_.iter().map(|row| vec![row[3]]).collect::<Vec<_>>();

//     let alpha_shp = param[trans_dim..trans_dim+shape_dim].chunks(1).map(|chunk| chunk.to_vec()).collect::<Vec<_>>();
//     let alpha_exp = param[trans_dim+shape_dim..].chunks(1).map(|chunk| chunk.to_vec()).collect::<Vec<_>>();

//     Ok((R, offset, alpha_shp, alpha_exp))
//     }

// #[test]
// fn test_parse_param() {
//     let param = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0, 30.0, 31.0, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0, 38.0, 39.0, 40.0, 41.0, 42.0, 43.0, 44.0, 45.0, 46.0, 47.0, 48.0, 49.0, 50.0, 51.0, 52.0, 53.0, 54.0, 55.0, 56.0, 57.0, 58.0, 59.0, 60.0, 61.0];
//     let (R, offset, alpha_shp, alpha_exp) = _parse_param(&param).unwrap();

//     assert_eq!(R, vec![vec![0.0, 1.0, 2.0], vec![4.0, 5.0, 6.0], vec![8.0, 9.0, 10.0]]);
//     assert_eq!(offset, vec![vec![3.0], vec![7.0], vec![11.0]]);
//     assert_eq!(alpha_shp, vec![vec![12.0], vec![13.0], vec![14.0], vec![15.0], vec![16.0], vec![17.0], vec![18.0], vec![19.0], vec![20.0], vec![21.0], vec![22.0], vec![23.0], vec![24.0], vec![25.0], vec![26.0], vec![27.0], vec![28.0], vec![29.0], vec![30.0], vec![31.0], vec![32.0], vec![33.0], vec![34.0], vec![35.0], vec![36.0], vec![37.0], vec![38.0], vec![39.0], vec![40.0], vec![41.0], vec![42.0], vec![43.0], vec![44.0], vec![45.0], vec![46.0], vec![47.0], vec![48.0], vec![49.0], vec![50.0], vec![51.0]]);
//     assert_eq!(alpha_exp, vec![vec![52.0], vec![53.0], vec![54.0], vec![55.0], vec![56.0], vec![57.0], vec![58.0], vec![59.0], vec![60.0], vec![61.0]]);
// }

#[test]
fn test_parse_param() {
    let param = [
        0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0, 30.0, 31.0,
        32.0, 33.0, 34.0, 35.0, 36.0, 37.0, 38.0, 39.0, 40.0, 41.0, 42.0, 43.0, 44.0, 45.0, 46.0,
        47.0, 48.0, 49.0, 50.0, 51.0, 52.0, 53.0, 54.0, 55.0, 56.0, 57.0, 58.0, 59.0, 60.0, 61.0,
    ];

    // println!("{:?}", param);
    let result = _parse_param(&param);

    assert!(result.is_ok());

    let (R, offset, alpha_shp, alpha_exp) = result.unwrap();

    let expected = Ok((
        [[0.0, 1.0, 2.0], [4.0, 5.0, 6.0], [8.0, 9.0, 10.0]],
        [[3.0], [7.0], [11.0]],
        [
            [12.0],
            [13.0],
            [14.0],
            [15.0],
            [16.0],
            [17.0],
            [18.0],
            [19.0],
            [20.0],
            [21.0],
            [22.0],
            [23.0],
            [24.0],
            [25.0],
            [26.0],
            [27.0],
            [28.0],
            [29.0],
            [30.0],
            [31.0],
            [32.0],
            [33.0],
            [34.0],
            [35.0],
            [36.0],
            [37.0],
            [38.0],
            [39.0],
            [40.0],
            [41.0],
            [42.0],
            [43.0],
            [44.0],
            [45.0],
            [46.0],
            [47.0],
            [48.0],
            [49.0],
            [50.0],
            [51.0],
        ],
        [
            [52.0],
            [53.0],
            [54.0],
            [55.0],
            [56.0],
            [57.0],
            [58.0],
            [59.0],
            [60.0],
            [61.0],
        ],
    ));

    let result = _parse_param(&param);
    assert_eq!(result, expected);
}

pub fn similar_transform(mut pts3d: Vec<Vec<f32>>, roi_box: [f32; 4], size: i32) -> Vec<Vec<f32>> {
    // pts3d shape - ( 3, 68 )
    // roi_box example - [1, 2, 3, 4]
    // size example - 120

    pts3d[0].iter_mut().for_each(|p| *p -= 1.0);
    pts3d[2].iter_mut().for_each(|p| *p -= 1.0);
    pts3d[1].iter_mut().for_each(|p| *p = size as f32 - *p);

    let sx = roi_box[0];
    let sy = roi_box[1];
    let ex = roi_box[2];
    let ey = roi_box[3];
    let scale_x = (ex - sx) / size as f32;
    let scale_y = (ey - sy) / size as f32;
    pts3d[0].iter_mut().for_each(|p| *p = *p * scale_x + sx);
    pts3d[1].iter_mut().for_each(|p| *p = *p * scale_y + sy);
    let s = (scale_x + scale_y) / 2.0;
    pts3d[2].iter_mut().for_each(|p| *p = *p * s);
    pts3d[2].sort_by(|a, b| a.partial_cmp(b).unwrap());
    let min_z = pts3d[2][0];
    pts3d[2].iter_mut().for_each(|p| *p -= min_z);

    pts3d
}

// use onnxruntime::ndarray::{ArrayBase, OwnedRepr, Dim};

// fn similar_transform(
//     pts3d: ArrayBase<OwnedRepr<f32>, Dim<[usize; 2]>>,
//     roi_box: [f32; 4],
//     size: i32,
// ) -> ArrayBase<OwnedRepr<f32>, Dim<[usize; 2]>> {
//     pts3d[[0, ..]] -= 1;
//     pts3d[[2, ..]] -= 1;
//     pts3d[[1, ..]] = size - pts3d[1, ..];

//     let (sx, sy, ex, ey) = roi_box;
//     let scale_x = (ex - sx) / size;
//     let scale_y = (ey - sy) / size;
//     pts3d[[0, ..]] = pts3d[0, ..] * scale_x + sx;
//     pts3d[[1, ..]] = pts3d[1, ..] * scale_y + sy;
//     let s = (scale_x + scale_y) / 2;
//     pts3d[[2, ..]] *= s;
//     pts3d[[2, ..]] -= pts3d[2, ..].min();
//     pts3d
// }


#[test]
fn test_similar_transform() {
    let mut pts3d = vec![
        vec![0.0, 1.0, 2.0],
        vec![3.0, 4.0, 5.0],
        vec![6.0, 7.0, 8.0],
    ];
    let roi_box = [1., 2., 3., 4.];
    let size = 120;

    let result = similar_transform(pts3d, roi_box, size);

    println!("{:?}", result);
    // assert_eq!(
    //     result,
    //     &vec![
    //         vec![0.9833333492279053, 1.0, 1.0166666507720947],
    //         vec![3.950000047683716, 3.933333396911621, 3.9166667461395264],
    //         vec![0.0, 0.01666666753590107, 0.03333333507180214]
    //     ]
    // );
}

// fn parse_roi_box_from_landmark(pts: Array1<f32>) -> [i32; 4] {
//     let bbox = [
//         pts.slice(s![0, ..]).min().unwrap(),
//         pts.slice(s![1, ..]).min().unwrap(),
//         pts.slice(s![0, ..]).max().unwrap(),
//         pts.slice(s![1, ..]).max().unwrap(),
//     ];
//     let center = [(bbox[0] + bbox[2]) / 2.0, (bbox[1] + bbox[3]) / 2.0];
//     let radius = max(bbox[2] - bbox[0], bbox[3] - bbox[1]) / 2.0;
//     let bbox = [
//         center[0] - radius,
//         center[1] - radius,
//         center[0] + radius,
//         center[1] + radius,
//     ];

//     let llength = ((bbox[2] - bbox[0]).powi(2) + (bbox[3] - bbox[1]).powi(2)).sqrt();
//     let center_x = (bbox[2] + bbox[0]) / 2.0;
//     let center_y = (bbox[3] + bbox[1]) / 2.0;

//     let mut roi_box = [0; 4];
//     roi_box[0] = (center_x - llength / 2.0) as i32;
//     roi_box[1] = (center_y - llength / 2.0) as i32;
//     roi_box[2] = (roi_box[0] as f32 + llength) as i32;
//     roi_box[3] = (roi_box[1] as f32 + llength) as i32;

//     roi_box
// }

pub fn parse_roi_box_from_landmark(pts: Vec<Vec<f32>>) -> [f32; 4] {
    let bbox = vec![
        pts[0].iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        pts[1].iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        pts[0].iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        pts[1].iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
    ];
    let center = vec![
    (bbox[0] + bbox[2]) / 2.,
    (bbox[1] + bbox[3]) / 2.,
    ];
    let radius = f32::max(bbox[2] - bbox[0], bbox[3] - bbox[1]) / 2.;
    let bbox = vec![
    center[0] - radius,
    center[1] - radius,
    center[0] + radius,
    center[1] + radius,
    ];
    

    let llength = (((bbox[2] - bbox[0]).powf(2.) + (bbox[3] - bbox[1]).powf(2.)) as f32).sqrt();


    let center_x = ((bbox[2] + bbox[0]) / 2.) as f32;
    let center_y = ((bbox[3] + bbox[1]) / 2.) as f32;
    
    let mut roi_box = [0.0; 4];
    roi_box[0] = center_x - llength / 2.;
    roi_box[1] = center_y - llength / 2.;
    roi_box[2] = roi_box[0] + llength;
    roi_box[3] = roi_box[1] + llength;
    
    return roi_box;

    }

#[test]
fn test_parse_roi_box_from_landmark() {
    let pts = vec![vec![1., 2., 3.], vec![4., 5., 6.], vec![7., 8., 9.]];
    let result = parse_roi_box_from_landmark(pts);
    
    assert_eq!(
            result,
            [0.58578646, 3.5857863, 3.4142137, 6.414213]
        );
}

pub fn parse_roi_box_from_bbox(bbox: [f32; 4]) -> [f32; 4] {
    let left = bbox[0];
    let top = bbox[1];
    let right = bbox[2];
    let bottom = bbox[3];
    let old_size = (right - left + bottom - top) / 2.;
    let center_x = right - (right - left) / 2.;
    let center_y = bottom - (bottom - top) / 2. + old_size * 0.14;
    let size = (old_size * 1.58).round();

    let mut roi_box = [0.; 4];
    roi_box[0] = center_x - size / 2.;
    roi_box[1] = center_y - size / 2.;
    roi_box[2] = roi_box[0] + size;
    roi_box[3] = roi_box[1] + size;

    roi_box
}

#[test]
fn test_parse_roi_box_from_bbox() {
    let bbox = [1., 2., 3., 4.];
    let roi_box = parse_roi_box_from_bbox(bbox);

    
    assert_eq!(
        roi_box,
        [0.5, 1.78, 3.5, 4.7799997]
    );
    
}

// fn crop_img(img: &Mat, roi_box: [i32; 4]) -> Mat {
//     let h = img.rows();
//     let w = img.cols();

//     let (sx, sy, ex, ey) = (roi_box[0], roi_box[1], roi_box[2], roi_box[3]);
//     let (dh, dw) = (ey - sy, ex - sx);

//     let mut res = Mat::default().unwrap();
//     if img.channels() == 3 {
//         res = Mat::zeros(dh, dw, img.typ()).unwrap();
//     } else {
//         res = Mat::zeros(dh, dw, img.typ()).unwrap();
//     }

//     let (sx, dsx) = if sx < 0 { (0, -sx) } else { (sx, 0) };
//     let (ex, dex) = if ex > w { (w, dw - (ex - w)) } else { (ex, dw) };
//     let (sy, dsy) = if sy < 0 { (0, -sy) } else { (sy, 0) };
//     let (ey, dey) = if ey > h { (h, dh - (ey - h)) } else { (ey, dh) };

//     let mut src = img.row_range(sy, ey).col_range(sx, ex).to_mat();
//     let mut dst = res.row_range_mut(dsy, dey).col_range_mut(dsx, dex);
//     src.copy_to(&mut dst).unwrap();

//     res
// }



pub fn crop_img(img: &Mat, roi_box: [f32; 4]) -> Mat {
    let h = img.size().unwrap().height;
    let w = img.size().unwrap().width;

    let sx = roi_box[0].round() as i32;
    let sy = roi_box[1].round() as i32;
    let ex = roi_box[2].round() as i32;
    let ey = roi_box[3].round() as i32;

    let dh = ey - sy;
    let dw = ex - sx;


    let (sx, dsx) = if sx < 0 { (0, -sx) } else { (sx, 0) };
    let (ex, dex) = if ex > w { (w, dw - (ex - w)) } else { (ex, dw) };
    let (sy, dsy) = if sy < 0 { (0, -sy) } else { (sy, 0) };
    let (ey, dey) = if ey > h { (h, dh - (ey - h)) } else { (ey, dh) };


    let roi = Rect::new(sx, sy,ex-sx , ey-sy);
    let res = Mat::roi(&img, roi).unwrap();

    res
}


#[test]
fn test_crop_img() {

    let mut frame = Mat::default();
        // cam.read(&mut frame)?;

    imgcodecs::imread("test.jpg", 1)
    .map(|m| frame = m)
    .unwrap();
    let roi_box = [77.5, 112.5, 472.5, 507.5];

    let mut result = crop_img(&frame, roi_box);


    let window = "video capture";
    highgui::named_window(window, highgui::WINDOW_AUTOSIZE).unwrap();

    highgui::imshow(window, &mut result).unwrap();

    let key = highgui::wait_key(100000).unwrap();

}


// use opencv::{
//     core::{
//     Point,
//     Scalar,
//     Vec3b,
//     },
//     imgcodecs,
//     prelude::*,
//     };
    
//     fn crop_img(img: Mat, roi_box: Vec<i32>) -> Mat {
//     let (h, w) = img.size();
    
//     let sx = roi_box[0].round() as i32;
//     let sy = roi_box[1].round() as i32;
//     let ex = roi_box[2].round() as i32;
//     let ey = roi_box[3].round() as i32;
//     let dh = ey - sy;
//     let dw = ex - sx;
    
//     let mut res = Mat::zeros_size(dh, dw, img.type()).unwrap();
    
//     let (dsx, dex, dsy, dey) = if sx < 0 {
//         (0, -sx, 0, dh)
//     } else if ex > w {
//         (0, dw - (ex - w), 0, dh)
//     } else if sy < 0 {
//         (0, dw, 0, -sy)
//     } else if ey > h {
//         (0, dw, 0, dh - (ey - h))
//     } else {
//         (0, dw, 0, dh)
//     };
    
//     let roi = Rect::new(sx, sy, dw, dh);
//     let _roi = res.roi(roi).unwrap();
//     img.copy_to(&mut _roi, None);
    
//     return res;
//     }
    
//     fn main() {
//     let img = imgcodecs::imread("test.jpg").unwrap();
//     let roi_box = vec![1, 2, 3, 4];
//     let result = crop_img(img, roi_box);
//     imgcodecs::imwrite("test_cropped.jpg", &result).unwrap();
//     }


// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// ///////////////////////////////////////////// Head Pose  ///////////////////////////////////////////////////////
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// fn P2sRt(P: Array2<f64>) -> (f64, Array2<f64>, Array1<f64>) {
//     let t3d = P.slice(s![.., 3]);
//     let R1 = P.slice(s![0, ..3]);
//     let R2 = P.slice(s![1, ..3]);
//     let s = (R1.norm() + R2.norm()) / 2.0;
//     let r1 = &R1 / R1.norm();
//     let r2 = &R2 / R2.norm();
//     let r3 = r1.cross(&r2);

//     let mut R = Array2::zeros((3, 3));
//     R.slice_mut(s![0, ..]).assign(&r1);
//     R.slice_mut(s![1, ..]).assign(&r2);
//     R.slice_mut(s![2, ..]).assign(&r3);

//     (s, R, t3d)
// }


fn P2sRt(P: &[[f32; 4]]) -> (f32, [[f32; 3]; 3], [f32; 3]) {
    let t3d = [P[0][3], P[1][3], P[2][3]];
    let R1 = [P[0][0], P[0][1], P[0][2]];
    let R2 = [P[1][0], P[1][1], P[1][2]];
    let s = (R1.iter().map(|&x| x * x).sum::<f32>().sqrt() + R2.iter().map(|&x| x * x).sum::<f32>().sqrt()) / 2.0;
    let r1 = [R1[0] / R1.iter().map(|&x| x * x).sum::<f32>().sqrt(), R1[1] / R1.iter().map(|&x| x * x).sum::<f32>().sqrt(), R1[2] / R1.iter().map(|&x| x * x).sum::<f32>().sqrt()];
    let r2 = [R2[0] / R2.iter().map(|&x| x * x).sum::<f32>().sqrt(), R2[1] / R2.iter().map(|&x| x * x).sum::<f32>().sqrt(), R2[2] / R2.iter().map(|&x| x * x).sum::<f32>().sqrt()];
    let r3 = [r1[1] * r2[2] - r1[2] * r2[1], r1[2] * r2[0] - r1[0] * r2[2], r1[0] * r2[1] - r1[1] * r2[0]];
    let R = [r1, r2, r3];
    (s, R, t3d)
}


#[test]
fn test_P2sRt() {
    let P = [[1.0, 2.0, 3.0, 4.0],
             [5.0, 6.0, 7.0, 8.0],
             [9.0, 10.0, 11.0, 12.0]];
    let (s, R, t3d) = P2sRt(&P);
    assert_eq!(s, 7.114872934237728);
    assert_eq!(R, [[0.2672612419124244, 0.5345224838248488, 0.8017837257372732],
                    [ 0.47673129, 0.57207755,  0.66742381],
                    [-0.10192944,  0.20385888, -0.10192944]]);
    assert_eq!(t3d, [4.0, 8.0, 12.0]);
}


// fn matrix2angle(R: Array2<f64>) -> (f64, f64, f64) {
//     let cos_x = R[(2, 0)].cos();
//     let (x, y, z) = if R[(2, 0)] > 0.998 {
//         (PI / 2.0, 0.0, 0.0)
//     } else if R[(2, 0)] < -0.998 {
//         (-PI / 2.0, 0.0, 0.0)
//     } else {
//         (R[(2, 0)].asin(), 0.0, 0.0)
//     };
//     y = R[(2, 1)] / cos_x.cos();
//     z = R[(1, 0)] / cos_x.cos();

//     (x, y, z)
// }


fn matrix2angle(R: &[[f32; 3]]) -> (f32, f32, f32) {
    if R[2][0] > 0.998 {
        let z = 0.0;
        let x = FRAC_PI_2 as f32;
        let y = z + -R[0][1].atan2(-R[0][2]);
        (x, y, z)
    } else if R[2][0] < -0.998 {
        let z = 0.0;
        let x = -FRAC_PI_2 as f32;
        let y = -z + R[0][1].atan2(R[0][2]);
        (x, y, z)
    } else {
        let x = R[2][0].asin();
        let y = (R[2][1] / x.cos()).atan2(R[2][2] / x.cos());
        let z = (R[1][0] / x.cos()).atan2(R[0][0] / x.cos());
        (x, y, z)
    }
}


#[test]
fn test_matrix2angle() {
    let R = [[1.0, 2.0, 3.0],
             [4.0, 5.0, 6.0],
             [7.0, 8.0, 9.0]];
    let (x, y, z) = matrix2angle(&R);

    assert_eq!(x, 1.5707963267948966);
    assert_eq!(y, -2.5535900500422257);
    assert_eq!(z, 0.0);
}


// fn calc_pose(param: Array1<f64>) -> (Array2<f64>, [f64; 3]) {
//     let P = param.slice(s![..12]).to_owned().into_shape((3, 4)).unwrap();
//     let (s, R, t3d) = P2sRt(P);
//     let mut P = Array2::zeros((3, 4));
//     P.slice_mut(s![.., ..3]).assign(&R);
//     P.slice_mut(s![.., 3]).assign(&t3d);

//     let pose = matrix2angle(R);
//     // let pose = [p * 180.0 / PI for p in pose];

//     (P, pose)
// }


pub fn calc_pose(param: &[f32; 62]) ->([[f32; 4]; 3], [f32; 3]) { // ([[f64; 4]; 3], [f64; 3])
    let P = [[param[0], param[1], param[2], param[3]],
    [param[4], param[5], param[6], param[7]],
    [param[8], param[9], param[10], param[11]]];
    
    // [&[f64; 4]; 3
    // println!("{:?}", P);
    let (s, R, t3d) = P2sRt(&P);
    let P = [        [R[0][0], R[0][1], R[0][2], t3d[0]],
        [R[1][0], R[1][1], R[1][2], t3d[1]],
        [R[2][0], R[2][1], R[2][2], t3d[2]],
    ];

    let pose = matrix2angle(&R);
    // println!("{:?}", pose);
    // let pose = [p * 180.0 / PI for p in pose];
    // pose.iter().map(|&p| p * 180 / PI);

    let pose = [        pose.0 * 180.0 / PI as f32,
        pose.1 * 180.0 / PI as f32,
        pose.2 * 180.0 / PI as f32,
    ];
    
    (P, pose)
}


#[test]
fn test_calc_pose() {
    let param = [
        0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0, 30.0, 31.0,
        32.0, 33.0, 34.0, 35.0, 36.0, 37.0, 38.0, 39.0, 40.0, 41.0, 42.0, 43.0, 44.0, 45.0, 46.0,
        47.0, 48.0, 49.0, 50.0, 51.0, 52.0, 53.0, 54.0, 55.0, 56.0, 57.0, 58.0, 59.0, 60.0, 61.0,
    ];

    let (P, updated_pose) = calc_pose(&param);

    assert_eq!(P, [[0.0, 0.4472135954999579, 0.8944271909999159, 3.0], [0.4558423058385518, 0.5698028822981898, 0.6837634587578276, 7.0], [-0.2038588765750503, 0.4077177531501004, -0.2038588765750502, 11.0]]);
    assert_eq!(updated_pose, [-11.76270692571231, 116.56505117707799, 90.0]);

}

fn build_camera_box(rear_size: f32) -> Vec<[f32; 3]> {
    let mut point_3d:Vec<[f32; 3]> = Vec::new(); //Vec<[f32; 3]>
    let rear_depth = 0.;
    point_3d.push([-rear_size, -rear_size, rear_depth]);
    point_3d.push([-rear_size, rear_size, rear_depth]);
    point_3d.push([rear_size, rear_size, rear_depth]);
    point_3d.push([rear_size, -rear_size, rear_depth]);
    point_3d.push([-rear_size, -rear_size, rear_depth]);

    let front_size = (4. * rear_size) / 3.;
    let front_depth = (4. * rear_size) / 3.;
    point_3d.push([-front_size, -front_size, front_depth]);
    point_3d.push([-front_size, front_size, front_depth]);
    point_3d.push([front_size, front_size, front_depth]);
    point_3d.push([front_size, -front_size, front_depth]);
    point_3d.push([-front_size, -front_size, front_depth]);

    point_3d
}

#[test]
fn test_build_camera_box() {
    let point_3d = build_camera_box(90.);
    assert_eq!(point_3d, [[-90., -90., 0.],
                          [-90., 90., 0.],
                          [90., 90., 0.],
                          [90., -90., 0.],
                          [-90., -90., 0.],
                          [-120., -120., 120.],
                          [-120., 120., 120.],
                          [120., 120., 120.],
                          [120., -120., 120.],
                          [-120., -120., 120.]])
            }

// fn calc_hypotenuse(pts: Array2<f64>) -> f64 {
//     let bbox = [
//         pts.slice(s![0, ..]).min().unwrap(),
//         pts.slice(s![1, ..]).min().unwrap(),
//         pts.slice(s![0, ..]).max().unwrap(),
//         pts.slice(s![1, ..]).max().unwrap(),
//     ];
//     let center = [(bbox[0] + bbox[2]) / 2.0, (bbox[1] + bbox[3]) / 2.0];
//     let radius = (bbox[2] - bbox[0]).max(bbox[3] - bbox[1]) / 2.0;
//     let bbox = [
//         center[0] - radius,
//         center[1] - radius,
//         center[0] + radius,
//         center[1] + radius,
//     ];
//     let llength = ((bbox[2] - bbox[0]).powi(2) + (bbox[3] - bbox[1]).powi(2)).sqrt();
//     llength / 3.0
// }


fn calc_hypotenuse(pts: &[[f32; 20]])  -> f32 {
    let bbox = [pts[0].iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        pts[1].iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        pts[0].iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        pts[1].iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
    ];

    let center = [(bbox[0] + bbox[2]) / 2.0, (bbox[1] + bbox[3]) / 2.0];
    let radius = f32::max(bbox[2] - bbox[0], bbox[3] - bbox[1]) / 2.0;
    let bbox = [        center[0] - radius,
        center[1] - radius,
        center[0] + radius,
        center[1] + radius,
    ];
    let llength = ((bbox[2] - bbox[0]).powi(2) + (bbox[3] - bbox[1]).powi(2)).sqrt();
    llength / 3.0
}


#[test]
fn test_calc_hypotenuse() {
    let pts = [
        [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0],
        [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0],
        [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0],
    ];
    let expected = 8.95;

    let result = calc_hypotenuse(&pts);
    // assert_eq!(result, expected);

}




// fn gen_point2d(P: Vec<[f32; 4]>, ver: Vec<[f32; 20]>) -> (Vec<[i32; 2]>, f32) {
//     let llength = calc_hypotenuse(&ver);
//     let point_3d = build_camera_box(llength);

//     println!("{:?}", point_3d);

//     let mut point_3d_homo = Vec::new();
//     for point in point_3d {
//         let mut homogeneous_point = [0.0, 0.0, 0.0, 0.0];
//         for i in 0..3 {
//             for j in 0..3 {
//                 homogeneous_point[i] += point[j] * P[j][i];
//             }
//         }
//         point_3d_homo.push(homogeneous_point);
//     }

//     // println!("{:?}", point_3d_homo);

//     let mut point_2d = Vec::new();
//     for point in point_3d_homo {
//         let mut homogeneous_point = [0.0, 0.0];
//         for i in 0..2 {
//             for j in 0..3 {
//                 homogeneous_point[i] += point[j] * P[j][i];
//             }
//         }
//         point_2d.push(homogeneous_point);
//     }

//     let mut point_2d = point_2d.into_iter()
//         .map(|point| [point[0] as i32, -point[1] as i32])
//         .collect::<Vec<[i32; 2]>>();

//     let mean_2d = point_2d[..4].iter().fold([0, 0], |acc, point| [acc[0] + point[0], acc[1] + point[1]]);
//     let mean_2d = [mean_2d[0] / 4, mean_2d[1] / 4];
//     let mean_ver = ver[..2].iter().flat_map(|row| row[..20].iter()).sum::<f32>() / 20.0;

//     for point in &mut point_2d {
//         point[0] = (point[0] - mean_2d[0]) as i32 + mean_ver as i32;
//         point[1] = (point[1] - mean_2d[1]) as i32 + mean_ver as i32;
//     }

//     (point_2d, llength)
// }


// fn gen_point2d(P: Vec<Vec<f32>>, ver: Vec<[f32; 20]>) -> (Vec<[f32; 2]>, f32) {
//     let llength = calc_hypotenuse(&ver);
//     let point_3d = build_camera_box(llength);

//     // Map to 2D image points
//     let point_3d_homo: Vec<[f32; 3]> = point_3d
//         .into_iter()
//         .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32])
//         .collect();

    

//     let point_2d: Vec<[f32; 2]> = point_3d_homo
//         .into_iter()
//         .map(|p| [            p[0] * P[0][0] + p[1] * P[0][1] + p[2] * P[0][2],
//             p[0] * P[1][0] + p[1] * P[1][1] + p[2] * P[1][2],
//         ])
//         .collect();

    

//     let mut point_2d: Vec<[f32; 2]> = point_2d
//         .into_iter()
//         .map(|p| [p[0], -p[1]])
//         .collect();

        

//     let mean = [        ver[0][0..20].iter().sum::<f32>() / 20.0,
//         ver[1][0..20].iter().sum::<f32>() / 20.0,
//     ];
//     for p in point_2d.iter_mut() {
//         p[0] = (p[0] - mean[0]);
//         p[1] = (p[1] - mean[1]);
//     }

//     (point_2d, llength)
// }


// fn gen_point2d(p: &[[f32; 4];3], ver: &[[f32; 20]]) { // -> (Vec<[i32; 2]>, i32)
//     let llength = calc_hypotenuse(ver);
//     let point_3d = build_camera_box(llength);
//     let point_3d_homo = point_3d
//         .iter()
//         .map(|pt| [pt[0], pt[1], pt[2], 1.0])
//         .collect::<Vec<_>>();
//     let point_2d = point_3d_homo
//         .iter()
//         .map(|pt| {
//             let x = pt[0] * p[0][0] + pt[1] * p[1][0] + pt[2] * p[2][0] + pt[3] * p[3][0];
//             let y = pt[0] * p[0][1] + pt[1] * p[1][1] + pt[2] * p[2][1] + pt[3] * p[3][1];
//             [-y, x]
//         })
//         .collect::<Vec<_>>();
    // let point_2d = point_2d
    //     .iter()
    //     .map(|pt| {
    //         let x = pt[0] - mean(&point_2d[0..4], 0) + mean(&ver[0..2], 1);
    //         let y = pt[1] - mean(&point_2d[0..4], 0) + mean(&ver[0..2], 1);
    //         [x, y]
    //     })
    //     .map(|pt| [pt[0] as i32, pt[1] as i32])
    //     .collect::<Vec<_>>();
    // (point_2d, llength)
// }

// #[test]
// fn test_gen_point2d() {
//     let P = [[1.0, 2.0, 3.0, 4.0], [5.0, 6.0, 7.0, 8.0], [9.0, 10.0, 11.0, 12.0]];
// let ver = [[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0],
// [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0], [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0]];

//                 gen_point2d(P, ver); //let (point_2d, llength) = 

//                 // println!("{:?}", point_2d);

//                 // assert_eq!(point_2d, vec![[1, 2], [3, 4], [5, 6], [7, 8], [9, 10], [11, 12], [13, 14], [15, 16]]);
//                 // assert_eq!(llength, 17.0);
                

// }
