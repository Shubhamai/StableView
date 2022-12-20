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
    core::Mat::roi(img, roi).unwrap() // ! Need to deal with this
}