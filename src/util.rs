//! Utility structures and functions.

pub fn enable_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};

    let format = fmt::format()
        .without_time()
        .with_target(false)
        .with_source_location(true)
        .compact();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .event_format(format)
        .init();
}

pub fn fuck_ref<'a, T>(ptr: &T) -> &'a T {
    unsafe { &*(ptr as *const T) }
}

pub fn fuck_mut<'a, T>(ptr: &mut T) -> &'a mut T {
    unsafe { &mut *(ptr as *mut T) }
}

// TODO: Stable since Rust version 1.62.0
pub fn total_cmp(lhs: &f32, rhs: &f32) -> std::cmp::Ordering {
    let mut lhs = lhs.to_bits() as i32;
    let mut rhs = rhs.to_bits() as i32;
    lhs ^= (((lhs >> 31) as u32) >> 1) as i32;
    rhs ^= (((rhs >> 31) as u32) >> 1) as i32;
    lhs.cmp(&rhs)
}

pub fn linear_from_srgb_f64(r: f64, g: f64, b: f64) -> [f64; 3] {
    let cutoff = [r < 0.04045, g < 0.04045, b < 0.04045];
    let lower = [r / 12.92, g / 12.92, b / 12.92];
    let higher = [
        ((r + 0.055) / 1.055).powf(2.4),
        ((g + 0.055) / 1.055).powf(2.4),
        ((b + 0.055) / 1.055).powf(2.4),
    ];
    [
        if cutoff[0] { lower[0] } else { higher[0] },
        if cutoff[1] { lower[1] } else { higher[1] },
        if cutoff[2] { lower[2] } else { higher[2] },
    ]
}

pub fn expand_to_pixel(mut rect: egui::Rect, ppi: f32) -> egui::Rect {
    rect.min = map_to_pixel_pos(rect.min, ppi, f32::floor);
    rect.max = map_to_pixel_pos(rect.max, ppi, f32::ceil);
    rect
}

pub fn shrink_to_pixel(mut rect: egui::Rect, ppi: f32) -> egui::Rect {
    rect.min = map_to_pixel_pos(rect.min, ppi, f32::ceil);
    rect.max = map_to_pixel_pos(rect.max, ppi, f32::floor);
    rect
}

pub fn round_to_pixel(mut rect: egui::Rect, pixels_per_point: f32) -> egui::Rect {
    rect.min = map_to_pixel_pos(rect.min, pixels_per_point, f32::round);
    rect.max = map_to_pixel_pos(rect.max, pixels_per_point, f32::round);
    rect
}

fn map_to_pixel_pos(mut pos: egui::Pos2, ppi: f32, map: fn(f32) -> f32) -> egui::Pos2 {
    pos.x = map_to_pixel(pos.x, ppi, map);
    pos.y = map_to_pixel(pos.y, ppi, map);
    pos
}

#[inline(always)]
pub fn map_to_pixel(point: f32, ppi: f32, map: fn(f32) -> f32) -> f32 {
    map(point * ppi) / ppi
}
