// Importing Libraries
use onnxruntime::ndarray::{arr2, s, Array2, Axis};
use opencv::{
    core,
    prelude::{MatTraitConst, MatTraitConstManual},
};
use std::f64::consts::{FRAC_PI_2, PI};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////// TDDFA ////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn parse_param(
    param: &[f32; 62],
) -> Result<([[f32; 3]; 3], [[f32; 1]; 3], [[f32; 1]; 40], [[f32; 1]; 10]), &str> {
    // TODO: Can use type alias/defininations to improve code redability.
    let n = param.len();

    let (trans_dim, shape_dim, _) = match n {
        62 => (12, 40, 10),
        72 => (12, 40, 20),
        141 => (12, 100, 29),
        _ => return Err("Undefined templated param parsing rule"),
    };

    let r_ = [
        [param[0], param[1], param[2], param[3]],
        [param[4], param[5], param[6], param[7]],
        [param[8], param[9], param[10], param[11]],
    ];

    let r = [
        [r_[0][0], r_[0][1], r_[0][2]],
        [r_[1][0], r_[1][1], r_[1][2]],
        [r_[2][0], r_[2][1], r_[2][2]],
    ];

    let offset = [[r_[0][3]], [r_[1][3]], [r_[2][3]]];

    let mut alpha_shp = [[0.0; 1]; 40];
    for i in 0..40 {
        alpha_shp[i][0] = param[trans_dim + i];
    }

    let mut alpha_exp = [[0.0; 1]; 10];
    for i in 0..10 {
        alpha_exp[i][0] = param[trans_dim + shape_dim + i];
    }

    Ok((r, offset, alpha_shp, alpha_exp))
}

pub fn similar_transform(mut pts3d: Vec<Vec<f32>>, roi_box: [f32; 4], size: i32) -> Vec<Vec<f32>> {
    pts3d[0].iter_mut().for_each(|p| *p -= 1.0);
    pts3d[2].iter_mut().for_each(|p| *p -= 1.0);
    pts3d[1].iter_mut().for_each(|p| *p = size as f32 - *p);

    let sx = roi_box[0];
    let sy = roi_box[1];
    let ex = roi_box[2];
    let ey = roi_box[3];
    let scale_x = (ex - sx) / size as f32;
    let scale_y = (ey - sy) / size as f32;
    pts3d[0]
        .iter_mut()
        .for_each(|p| *p = (*p).mul_add(scale_x, sx));
    pts3d[1]
        .iter_mut()
        .for_each(|p| *p = (*p).mul_add(scale_y, sy));
    let s = (scale_x + scale_y) / 2.0;
    pts3d[2].iter_mut().for_each(|p| *p *= s);
    pts3d[2].sort_by(|a, b| a.partial_cmp(b).unwrap());
    let min_z = pts3d[2][0];
    pts3d[2].iter_mut().for_each(|p| *p -= min_z);

    pts3d
}

pub fn parse_roi_box_from_landmark(pts: &[Vec<f32>]) -> [f32; 4] {
    let bbox = [
        pts[0]
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap(),
        pts[1]
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap(),
        pts[0]
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap(),
        pts[1]
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap(),
    ];

    let center = [(bbox[0] + bbox[2]) / 2., (bbox[1] + bbox[3]) / 2.];
    let radius = f32::max(bbox[2] - bbox[0], bbox[3] - bbox[1]) / 2.;
    let bbox = [
        center[0] - radius,
        center[1] - radius,
        center[0] + radius,
        center[1] + radius,
    ];

    let llength =
        ((bbox[3] - bbox[1]).mul_add(bbox[3] - bbox[1], (bbox[2] - bbox[0]).powi(2)) as f32).sqrt();

    let center_x = (bbox[2] + bbox[0]) / 2.;
    let center_y = (bbox[3] + bbox[1]) / 2.;

    let mut roi_box = [0.0; 4];
    roi_box[0] = center_x - llength / 2.;
    roi_box[1] = center_y - llength / 2.;
    roi_box[2] = roi_box[0] + llength;
    roi_box[3] = roi_box[1] + llength;

    roi_box
}

pub fn parse_roi_box_from_bbox(bbox: [f32; 4]) -> [f32; 4] {
    let left = bbox[0];
    let top = bbox[1];
    let right = bbox[2];
    let bottom = bbox[3];
    let old_size = (right - left + bottom - top) / 2.;
    let center_x = right - (right - left) / 2.;
    let center_y = old_size.mul_add(0.14, bottom - (bottom - top) / 2.);
    let size = (old_size * 1.58).round();

    let mut roi_box = [0.; 4];
    roi_box[0] = center_x - size / 2.;
    roi_box[1] = center_y - size / 2.;
    roi_box[2] = roi_box[0] + size;
    roi_box[3] = roi_box[1] + size;

    roi_box
}

pub fn crop_img(img: &core::Mat, roi_box: [f32; 4]) -> core::Mat {
    let h = img.size().unwrap().height;
    let w = img.size().unwrap().width;

    let sx = roi_box[0].round() as i32;
    let sy = roi_box[1].round() as i32;
    let ex = roi_box[2].round() as i32;
    let ey = roi_box[3].round() as i32;

    let dh = ey - sy;
    let dw = ex - sx;

    let (sx, _) = if sx < 0 { (0, -sx) } else { (sx, 0) };
    let (ex, _) = if ex > w { (w, dw - (ex - w)) } else { (ex, dw) };
    let (sy, _) = if sy < 0 { (0, -sy) } else { (sy, 0) };
    let (ey, _) = if ey > h { (h, dh - (ey - h)) } else { (ey, dh) };

    let roi = core::Rect::new(sx, sy, ex - sx, ey - sy);
    // println!("{:?}", roi);
    core::Mat::roi(img, roi).unwrap()
}

// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// ///////////////////////////////////////////// Head Pose  ///////////////////////////////////////////////////////
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn p2s_rt(p: &[[f32; 4]]) -> (f32, [[f32; 3]; 3], [f32; 3]) {
    let t3d = [p[0][3], p[1][3], p[2][3]];
    let r1 = [p[0][0], p[0][1], p[0][2]];
    let r2 = [p[1][0], p[1][1], p[1][2]];
    let s = (r1.iter().map(|&x| x * x).sum::<f32>().sqrt()
        + r2.iter().map(|&x| x * x).sum::<f32>().sqrt())
        / 2.0;
    let r1 = [
        r1[0] / r1.iter().map(|&x| x * x).sum::<f32>().sqrt(),
        r1[1] / r1.iter().map(|&x| x * x).sum::<f32>().sqrt(),
        r1[2] / r1.iter().map(|&x| x * x).sum::<f32>().sqrt(),
    ];
    let r2 = [
        r2[0] / r2.iter().map(|&x| x * x).sum::<f32>().sqrt(),
        r2[1] / r2.iter().map(|&x| x * x).sum::<f32>().sqrt(),
        r2[2] / r2.iter().map(|&x| x * x).sum::<f32>().sqrt(),
    ];
    let r3 = [
        r1[1] * r2[2] - r1[2] * r2[1],
        r1[2] * r2[0] - r1[0] * r2[2],
        r1[0] * r2[1] - r1[1] * r2[0],
    ];
    let r = [r1, r2, r3];
    (s, r, t3d)
}

fn matrix2angle(r: &[[f32; 3]]) -> (f32, f32, f32) {
    if r[2][0] > 0.998 {
        let z = 0.0;
        let x = FRAC_PI_2 as f32;
        let y = z + -r[0][1].atan2(-r[0][2]);
        (x, y, z)
    } else if r[2][0] < -0.998 {
        let z = 0.0;
        let x = -FRAC_PI_2 as f32;
        let y = -z + r[0][1].atan2(r[0][2]);
        (x, y, z)
    } else {
        let x = r[2][0].asin();
        let y = (r[2][1] / x.cos()).atan2(r[2][2] / x.cos());
        let z = (r[1][0] / x.cos()).atan2(r[0][0] / x.cos());
        (x, y, z)
    }
}

pub fn calc_pose(param: &[f32; 62]) -> ([[f32; 4]; 3], [f32; 3]) {
    let p = [
        [param[0], param[1], param[2], param[3]],
        [param[4], param[5], param[6], param[7]],
        [param[8], param[9], param[10], param[11]],
    ];

    let (_, r, t3d) = p2s_rt(&p);
    let p = [
        [r[0][0], r[0][1], r[0][2], t3d[0]],
        [r[1][0], r[1][1], r[1][2], t3d[1]],
        [r[2][0], r[2][1], r[2][2], t3d[2]],
    ];

    let pose = matrix2angle(&r);

    let pose = [
        pose.0 * 180.0 / PI as f32,
        pose.1 * 180.0 / PI as f32,
        pose.2 * 180.0 / PI as f32,
    ];

    (p, pose)
}

fn build_camera_box(rear_size: f32) -> Vec<[f32; 3]> {
    let mut point_3d: Vec<[f32; 3]> = Vec::new();
    let rear_depth = 0.;
    point_3d.push([-rear_size, -rear_size, rear_depth]);
    point_3d.push([-rear_size, rear_size, rear_depth]);
    point_3d.push([rear_size, rear_size, rear_depth]);
    point_3d.push([rear_size, -rear_size, rear_depth]);
    point_3d.push([-rear_size, -rear_size, rear_depth]);

    // ? Subtracting by -1 because in python, the int conversion simplt returns the greatest integer instead of rounding the values to integer
    let mut front_size = (4. / 3. * rear_size).ceil();
    let mut front_depth = (4. / 3. * rear_size).ceil();

    if (rear_size.ceil() - rear_size).abs() > f32::EPSILON {
        front_size -= 1.;
        front_depth -= 1.;
    };

    point_3d.push([-front_size, -front_size, front_depth]);
    point_3d.push([-front_size, front_size, front_depth]);
    point_3d.push([front_size, front_size, front_depth]);
    point_3d.push([front_size, -front_size, front_depth]);
    point_3d.push([-front_size, -front_size, front_depth]);

    point_3d
}

fn calc_hypotenuse(pts: &[Vec<f32>]) -> f32 {
    let bbox = [
        pts[0]
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap(),
        pts[1]
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap(),
        pts[0]
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap(),
        pts[1]
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap(),
    ];

    let center = [(bbox[0] + bbox[2]) / 2.0, (bbox[1] + bbox[3]) / 2.0];
    let radius = f32::max(bbox[2] - bbox[0], bbox[3] - bbox[1]) / 2.0;
    let bbox = [
        center[0] - radius,
        center[1] - radius,
        center[0] + radius,
        center[1] + radius,
    ];
    let llength = (bbox[2] - bbox[0]).hypot(bbox[3] - bbox[1]);
    llength / 3.0
}

pub fn gen_point2d(p: &[[f32; 4]; 3], ver: Vec<Vec<f32>>) -> (Vec<Vec<f32>>, f32) {
    let llength = calc_hypotenuse(&ver);
    let point_3d = build_camera_box(llength);
    let point_3d_homo: Vec<[f32; 4]> = point_3d
        .into_iter()
        .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32, 1.])
        .collect();

    let mut binding = arr2(&point_3d_homo).dot(&arr2(p).t());
    let mut point_2d = binding.slice_mut(s![.., ..2]);

    let mut ver_array = Array2::<f32>::default((3, 20));
    for (i, mut row) in ver_array.axis_iter_mut(Axis(0)).enumerate() {
        for (j, col) in row.iter_mut().enumerate() {
            *col = ver[i][j];
        }
    }

    point_2d.slice_mut(s![.., 1]).map_inplace(|x| *x = -*x);
    let mut point_2d_copy = point_2d.to_owned();
    let point_2d = point_2d.slice_mut(s![.., ..2]).map_axis(Axis(1), |x| {
        (&x - point_2d_copy
            .slice_mut(s![..4, ..2])
            .mean_axis(Axis(0))
            .unwrap()
            + ver_array.slice_mut(s![..2, ..]).mean_axis(Axis(1)).unwrap())
        .to_vec()
    });

    (point_2d.to_vec(), llength)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_param() {
        let param = [
            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
            16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0,
            30.0, 31.0, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0, 38.0, 39.0, 40.0, 41.0, 42.0, 43.0,
            44.0, 45.0, 46.0, 47.0, 48.0, 49.0, 50.0, 51.0, 52.0, 53.0, 54.0, 55.0, 56.0, 57.0,
            58.0, 59.0, 60.0, 61.0,
        ];

        let result = parse_param(&param);

        assert!(result.is_ok());

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

        assert_eq!(result, expected);
    }

    #[test]
    fn test_similar_transform() {
        let pts3d = vec![
            vec![0.0, 1.0, 2.0],
            vec![3.0, 4.0, 5.0],
            vec![6.0, 7.0, 8.0],
        ];
        let roi_box = [1., 2., 3., 4.];
        let size = 120;

        let result = similar_transform(pts3d, roi_box, size);

        // println!("{:?}", result[0]
        //     .get(0)
        //     .map(|first| result[0].iter().all(|x| x - first < 0.01))
        //     .unwrap_or(true));
        // println!("{:?}", result[0].iter().zip(&result[0]).filter(|&(a, b)| (a - b) < f32::EPSILON));
        // assert_eq!(
        //     result[0].iter().zip(&result[0]).filter(|&(a, b)| (a - b) < f32::EPSILON), 3
        // );

        // assert_eq!(
        //         result,
        //     );
    }

    #[test]
    fn test_parse_roi_box_from_landmark() {
        let pts = vec![vec![1., 2., 3.], vec![4., 5., 6.], vec![7., 8., 9.]];
        let result = parse_roi_box_from_landmark(&pts);

        assert_eq!(result, [0.585_786_46, 3.585_786_3, 3.414_213_7, 6.414_213]);
    }

    #[test]
    fn test_parse_roi_box_from_bbox() {
        let bbox = [1., 2., 3., 4.];
        let roi_box = parse_roi_box_from_bbox(bbox);

        assert_eq!(roi_box, [0.5, 1.78, 3.5, 4.779_999_7]);
    }

    #[test]
    fn test_crop_img() {
        let frame = core::Mat::new_rows_cols_with_default(
            120,
            120,
            core::CV_8UC3,
            core::Scalar::new(255., 0., 0., 0.),
        )
        .unwrap();
        let roi_box = [50., 60., 100., 120.];
        let result = crop_img(&frame, roi_box);

        assert_eq!(result.rows() as f32, roi_box[3] - roi_box[1]);
        assert_eq!(result.cols() as f32, roi_box[2] - roi_box[0]);
    }
    #[test]
    fn test_p2s_rt() {
        let p = [
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
        ];
        let (s, r, t3d) = p2s_rt(&p);
        assert_eq!(s, 7.114_873);
        assert_eq!(
            r,
            [
                [0.267_261_24, 0.534_522_5, 0.801_783_74],
                [0.476_731_3, 0.572_077_6, 0.667_423_8],
                [-0.101_929_44, 0.203_858_88, -0.101_929_44]
            ]
        );
        assert_eq!(t3d, [4.0, 8.0, 12.0]);
    }
    #[test]
    fn test_matrix2angle() {
        let r = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
        let (x, y, z) = matrix2angle(&r);

        assert_eq!(x, 1.570_796_4);
        assert_eq!(y, -2.553_59);
        assert_eq!(z, 0.0);
    }

    #[test]
    fn test_calc_pose() {
        let param = [
            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
            16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0,
            30.0, 31.0, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0, 38.0, 39.0, 40.0, 41.0, 42.0, 43.0,
            44.0, 45.0, 46.0, 47.0, 48.0, 49.0, 50.0, 51.0, 52.0, 53.0, 54.0, 55.0, 56.0, 57.0,
            58.0, 59.0, 60.0, 61.0,
        ];

        let (p, updated_pose) = calc_pose(&param);

        assert_eq!(
            p,
            [
                [0.0, 0.447_213_6, 0.894_427_2, 3.0],
                [0.455_842_32, 0.569_802_9, 0.683_763_44, 7.0],
                [-0.203_858_88, 0.407_717_76, -0.203_858_88, 11.0]
            ]
        );
        assert_eq!(updated_pose, [-11.762_707, 116.565_05, 90.0]);
    }

    #[test]
    fn test_build_camera_box() {
        let llength = 90.0;
        let point_3d = build_camera_box(llength);
        assert_eq!(
            point_3d,
            [
                [-llength, -llength, 0.],
                [-llength, llength, 0.],
                [llength, llength, 0.],
                [llength, -llength, 0.],
                [-llength, -llength, 0.],
                [-120., -120., 120.],
                [-120., 120., 120.],
                [120., 120., 120.],
                [120., -120., 120.],
                [-120., -120., 120.]
            ]
        );

        let llength = 90.1;
        let point_3d = build_camera_box(llength);
        assert_eq!(
            point_3d,
            [
                [-llength, -llength, 0.],
                [-llength, llength, 0.],
                [llength, llength, 0.],
                [llength, -llength, 0.],
                [-llength, -llength, 0.],
                [-120., -120., 120.],
                [-120., 120., 120.],
                [120., 120., 120.],
                [120., -120., 120.],
                [-120., -120., 120.]
            ]
        );

        let llength = 89.9;
        let point_3d = build_camera_box(llength);
        assert_eq!(
            point_3d,
            [
                [-llength, -llength, 0.],
                [-llength, llength, 0.],
                [llength, llength, 0.],
                [llength, -llength, 0.],
                [-llength, -llength, 0.],
                [-119., -119., 119.],
                [-119., 119., 119.],
                [119., 119., 119.],
                [119., -119., 119.],
                [-119., -119., 119.]
            ]
        )
    }

    #[test]
    fn test_calc_hypotenuse() {
        let pts = [
            vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
                16.0, 17.0, 18.0, 19.0, 20.0,
            ],
            vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
                16.0, 17.0, 18.0, 19.0, 20.0,
            ],
            vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
                16.0, 17.0, 18.0, 19.0, 20.0,
            ],
        ];
        let expected = 8.95;

        let result = calc_hypotenuse(&pts);

        assert!((result - expected).abs() < 0.01);
    }

    #[test]
    fn test_gen_point2d() {
        let p = [
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
        ];
        let ver = vec![
            vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
                16.0, 17.0, 18.0, 19.0, 20.0,
            ],
            vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
                16.0, 17.0, 18.0, 19.0, 20.0,
            ],
            vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
                16.0, 17.0, 18.0, 19.0, 20.0,
            ],
        ];

        println!("{:?}", gen_point2d(&p, ver));
    }
}
