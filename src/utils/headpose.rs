/// Utility function used by headpose module
/// The code is mostly converted from python to rust with assistance from ChatGPT. 
/// Python source - https://github.com/cleardusk/3DDFA_V2/blob/fa8dfc479b46c218e7d375706c673d5823ddb464/utils/pose.py

// Imporing Libraries
use crate::enums::extreme::Extreme;
use crate::utils::common::{get_extreme_value, get_ndarray};
use onnxruntime::ndarray::{arr2, s, Axis};
use std::f32::consts::{FRAC_PI_2, PI};

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
    let (x, y, z);

    if r[2][0] > 0.998 {
        z = 0.0;
        x = FRAC_PI_2;
        y = z + -r[0][1].atan2(-r[0][2]);
    } else if r[2][0] < -0.998 {
        z = 0.0;
        x = -FRAC_PI_2;
        y = -z + r[0][1].atan2(r[0][2]);
    } else {
        x = r[2][0].asin();
        y = (r[2][1] / x.cos()).atan2(r[2][2] / x.cos());
        z = (r[1][0] / x.cos()).atan2(r[0][0] / x.cos());
    }

    (x, y, z)
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
        pose.0 * 180.0 / PI,
        pose.1 * 180.0 / PI,
        pose.2 * 180.0 / PI,
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

    let mut front_size = (4. / 3. * rear_size).ceil();
    let mut front_depth = (4. / 3. * rear_size).ceil();

    // ? Subtracting by -1 because in python, the int conversion returns the greatest integer instead of rounding the values to integer
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
        get_extreme_value(&pts[0], Extreme::Min),
        get_extreme_value(&pts[1], Extreme::Min),
        get_extreme_value(&pts[0], Extreme::Max),
        get_extreme_value(&pts[1], Extreme::Max),
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

// TODO : Cleaning this up
pub fn gen_point2d(p: &[[f32; 4]; 3], ver: Vec<Vec<f32>>) -> (Vec<Vec<f32>>, f32) {
    let llength = calc_hypotenuse(&ver);
    let point_3d = build_camera_box(llength);

    let point_3d_homo: Vec<[f32; 4]> = point_3d
        .into_iter()
        .map(|p| [p[0], p[1], p[2], 1.])
        .collect();

    let mut binding = arr2(&point_3d_homo).dot(&arr2(p).t());
    let mut point_2d = binding.slice_mut(s![.., ..2]);

    let sliced_ver_mean = get_ndarray(ver, (3, 20))
        .slice_mut(s![..2, ..])
        .mean_axis(Axis(1))
        .expect("Unable to calculate sliced_ver_mean in gen_point2d");

    // point_2d.slice(s![.., 1]).map_inplace(|x| *x = -*x);
    let sliced_point2d_mean = point_2d
        .to_owned()
        .slice_mut(s![..4, ..2])
        .mean_axis(Axis(0))
        .expect("Unable to calculate sliced_point2d_mean in gen_point2d");

    let point_2d = point_2d.slice_mut(s![.., ..2]).map_axis(Axis(1), |x| {
        (&x - &sliced_point2d_mean + &sliced_ver_mean).to_vec()
    });

    (point_2d.to_vec(), llength)
}

#[cfg(test)]
mod tests {
    use super::*;

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
