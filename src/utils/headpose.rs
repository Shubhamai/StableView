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
        get_extreme_value(&pts[0], Extreme::MIN),
        get_extreme_value(&pts[1], Extreme::MIN),
        get_extreme_value(&pts[0], Extreme::MAX),
        get_extreme_value(&pts[1], Extreme::MAX),
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
        .unwrap();

    // point_2d.slice(s![.., 1]).map_inplace(|x| *x = -*x);
    let sliced_point2d_mean = point_2d
        .to_owned()
        .slice_mut(s![..4, ..2])
        .mean_axis(Axis(0))
        .unwrap();

    let point_2d = point_2d.slice_mut(s![.., ..2]).map_axis(Axis(1), |x| {
        (&x - &sliced_point2d_mean + &sliced_ver_mean).to_vec()
    });

    (point_2d.to_vec(), llength)
}
