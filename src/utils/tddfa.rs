/// Utility function used by tddfa module
/// The code is mostly converted from python to rust with assistance from ChatGPT.
/// Python source - https://github.com/cleardusk/3DDFA_V2/blob/master/utils/tddfa_util.py
/// https://github.com/cleardusk/3DDFA_V2/blob/master/utils/functions.py#L65
use crate::enums::extreme::Extreme;
use crate::utils::common::get_extreme_value;

pub fn parse_param(
    param: &[f32; 62],
) -> ([[f32; 3]; 3], [[f32; 1]; 3], [[f32; 1]; 40], [[f32; 1]; 10]) {
    // TODO: Can use type alias/defininations to improve code redability.
    let n = param.len();

    let (trans_dim, shape_dim, _) = match n {
        62 => (12, 40, 10),
        72 => (12, 40, 20),
        141 => (12, 100, 29),
        invalid_size => {
            tracing::error!("Undefined templated param parsing rule : {invalid_size}");
            panic!()
        }
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

    (r, offset, alpha_shp, alpha_exp)
}

pub fn similar_transform(mut pts3d: Vec<Vec<f32>>, roi_box: [f32; 4], size: f32) -> Vec<Vec<f32>> {
    pts3d[0].iter_mut().for_each(|p| *p -= 1.0);
    pts3d[2].iter_mut().for_each(|p| *p -= 1.0);
    pts3d[1].iter_mut().for_each(|p| *p = size - *p);

    let sx = roi_box[0];
    let sy = roi_box[1];
    let ex = roi_box[2];
    let ey = roi_box[3];

    let scale_x = (ex - sx) / size;
    let scale_y = (ey - sy) / size;
    pts3d[0]
        .iter_mut()
        .for_each(|p| *p = (*p).mul_add(scale_x, sx));
    pts3d[1]
        .iter_mut()
        .for_each(|p| *p = (*p).mul_add(scale_y, sy));

    let s = (scale_x + scale_y) / 2.0;
    pts3d[2].iter_mut().for_each(|p| *p *= s);

    let min_z = get_extreme_value(&pts3d[2], Extreme::Min);

    pts3d[2].iter_mut().for_each(|p| *p -= min_z);

    pts3d
}

pub fn parse_roi_box_from_landmark(pts: &[Vec<f32>]) -> [f32; 4] {
    let bbox = [
        get_extreme_value(&pts[0], Extreme::Min),
        get_extreme_value(&pts[1], Extreme::Min),
        get_extreme_value(&pts[0], Extreme::Max),
        get_extreme_value(&pts[1], Extreme::Max),
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
        ((bbox[3] - bbox[1]).mul_add(bbox[3] - bbox[1], (bbox[2] - bbox[0]).powi(2))).sqrt();

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

        let expected = (
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
        );

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
        let size = 120.;

        let result = similar_transform(pts3d, roi_box, size);

        assert_eq!(
            result,
            vec![
                vec![0.9833333492279053, 1.0, 1.0166666507720947],
                vec![3.950000047683716, 3.933333396911621, 3.9166667461395264],
                vec![0.0, 0.0166666668, 0.03333334]
            ]
        );
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
}
