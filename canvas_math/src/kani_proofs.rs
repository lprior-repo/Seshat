use super::*;

#[kani::proof]
fn kani_safe_zoom() {
    let zoom: f64 = kani::any();
    let result = safe_zoom(zoom);
    if zoom.is_finite() && zoom > f64::EPSILON {
        kani::assert(result == Some(zoom), "Valid zoom should be accepted");
    } else {
        kani::assert(result.is_none(), "Invalid zoom should be rejected");
    }
}

#[kani::proof]
fn kani_within() {
    let sx: f64 = kani::any();
    let sy: f64 = kani::any();
    let sw: f64 = kani::any();
    let sh: f64 = kani::any();
    let nx: f64 = kani::any();
    let ny: f64 = kani::any();
    let nw: f64 = kani::any();
    let nh: f64 = kani::any();

    kani::assume(sx.is_finite() && sx.abs() < 1e100);
    kani::assume(sy.is_finite() && sy.abs() < 1e100);
    kani::assume(sw.is_finite() && sw.abs() < 1e100);
    kani::assume(sh.is_finite() && sh.abs() < 1e100);
    kani::assume(nx.is_finite() && nx.abs() < 1e100);
    kani::assume(ny.is_finite() && ny.abs() < 1e100);
    kani::assume(nw.is_finite() && nw.abs() < 1e100);
    kani::assume(nh.is_finite() && nh.abs() < 1e100);

    let subgraph = (sx, sy, sw, sh);
    let node = (nx, ny, nw, nh);

    let result = within(subgraph, node);
    let expected = nx >= sx && ny >= sy && nx + nw <= sx + sw && ny + nh <= sy + sh;
    kani::assert(
        result == expected,
        "within must match the exact bound calculation",
    );
}

#[kani::proof]
fn kani_sanitize_zoom() {
    let zoom: f64 = kani::any();
    let min: f64 = kani::any();
    let max: f64 = kani::any();

    kani::assume(min.is_finite() && max.is_finite() && min <= max);
    kani::assume(min > f64::EPSILON);

    let result = sanitize_zoom(zoom, min, max);
    if zoom.is_finite() && zoom > f64::EPSILON {
        kani::assert(result.is_some(), "Valid zoom should return Some");
        if let Some(z) = result {
            kani::assert(
                z >= min && z <= max,
                "Zoom must be clamped within min and max",
            );
        }
    } else {
        kani::assert(result.is_none(), "Invalid zoom should return None");
    }
}

#[kani::proof]
fn kani_roundtrip_screen_canvas() {
    let client_x: f64 = kani::any();
    let client_y: f64 = kani::any();
    let camera_x: f64 = kani::any();
    let camera_y: f64 = kani::any();
    let zoom: f64 = kani::any();

    kani::assume(client_x.is_finite() && client_x.abs() < 1e4);
    kani::assume(client_y.is_finite() && client_y.abs() < 1e4);
    kani::assume(camera_x.is_finite() && camera_x.abs() < 1e4);
    kani::assume(camera_y.is_finite() && camera_y.abs() < 1e4);
    kani::assume(zoom.is_finite() && zoom > 0.1 && zoom < 10.0);

    if let Some((cx, cy)) = screen_to_canvas(client_x, client_y, camera_x, camera_y, zoom) {
        if let Some((sx, sy)) = canvas_to_screen(cx, cy, camera_x, camera_y, zoom) {
            let diff_x = (sx - client_x).abs();
            let diff_y = (sy - client_y).abs();
            kani::assert(diff_x < 1e-4, "X should be inverse");
            kani::assert(diff_y < 1e-4, "Y should be inverse");
        }
    }
}

#[kani::proof]
fn kani_roundtrip_canvas_screen() {
    let world_x: f64 = kani::any();
    let world_y: f64 = kani::any();
    let camera_x: f64 = kani::any();
    let camera_y: f64 = kani::any();
    let zoom: f64 = kani::any();

    kani::assume(world_x.is_finite() && world_x.abs() < 1e4);
    kani::assume(world_y.is_finite() && world_y.abs() < 1e4);
    kani::assume(camera_x.is_finite() && camera_x.abs() < 1e4);
    kani::assume(camera_y.is_finite() && camera_y.abs() < 1e4);
    kani::assume(zoom.is_finite() && zoom > 0.1 && zoom < 10.0);

    if let Some((sx, sy)) = canvas_to_screen(world_x, world_y, camera_x, camera_y, zoom) {
        if let Some((cx, cy)) = screen_to_canvas(sx, sy, camera_x, camera_y, zoom) {
            let diff_x = (cx - world_x).abs();
            let diff_y = (cy - world_y).abs();
            kani::assert(diff_x < 1e-4, "X should be inverse");
            kani::assert(diff_y < 1e-4, "Y should be inverse");
        }
    }
}
