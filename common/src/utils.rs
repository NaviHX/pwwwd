pub fn rgb_u8_to_f64((r, g, b): (u8, u8, u8)) -> (f64, f64, f64) {
    (r as f64 / 255., g as f64 / 255., b as f64 / 255.)
}

pub fn rgb_f64_to_u8((r, g, b): (f64, f64, f64)) -> (u8, u8, u8) {
    ((r * 255.) as u8, (g * 255.) as u8, (b * 255.) as u8)
}
