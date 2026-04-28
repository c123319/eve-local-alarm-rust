use crate::models::ColorMatchConfig;

/// 将 RGBA 像素转换为 HSV 颜色空间（OpenCV 半范围约定：H:0-179, S:0-255, V:0-255）
#[inline]
pub fn rgba_pixel_to_hsv(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let delta = max - min;

    // Value
    let v = max;

    // Saturation
    let s = if max == 0.0 { 0.0 } else { delta / max };

    // Hue
    let h_deg = if delta == 0.0 {
        0.0
    } else if max == rf {
        60.0 * (((gf - bf) / delta) % 6.0)
    } else if max == gf {
        60.0 * (((bf - rf) / delta) + 2.0)
    } else {
        60.0 * (((rf - gf) / delta) + 4.0)
    };

    // Normalize hue to [0, 360)
    let h_deg = if h_deg < 0.0 { h_deg + 360.0 } else { h_deg };

    // OpenCV half-range: H in [0, 179], S in [0, 255], V in [0, 255]
    let h = (h_deg / 2.0) as u8;
    let s = (s * 255.0) as u8;
    let v = (v * 255.0) as u8;

    (h, s, v)
}

/// 统计 RGBA 缓冲区内落在指定 HSV 范围内的像素数量
pub fn count_matching_pixels(rgba: &[u8], rule: &ColorMatchConfig) -> usize {
    let [h_lo, s_lo, v_lo] = rule.hsv_lower;
    let [h_hi, s_hi, v_hi] = rule.hsv_upper;

    rgba
        .chunks_exact(4)
        .filter(|pixel| {
            let (h, s, v) = rgba_pixel_to_hsv(pixel[0], pixel[1], pixel[2]);
            h >= h_lo as u8
                && h <= h_hi as u8
                && s >= s_lo as u8
                && s <= s_hi as u8
                && v >= v_lo as u8
                && v <= v_hi as u8
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_hostile_marker_config() -> ColorMatchConfig {
        ColorMatchConfig {
            name: "敌对标记".to_string(),
            hsv_lower: [0, 120, 120],
            hsv_upper: [15, 255, 255],
            min_pixels: 12,
            min_ratio: 0.02,
        }
    }

    fn make_rgba(pixels: &[(u8, u8, u8, u8)]) -> Vec<u8> {
        let mut buf = Vec::with_capacity(pixels.len() * 4);
        for &(r, g, b, a) in pixels {
            buf.extend_from_slice(&[r, g, b, a]);
        }
        buf
    }

    #[test]
    fn test_pure_red_to_hsv() {
        let (h, s, v) = rgba_pixel_to_hsv(255, 0, 0);
        // OpenCV half-range: H near 0
        assert!(h <= 2, "Red hue should be near 0, got {}", h);
        assert_eq!(s, 255);
        assert_eq!(v, 255);
    }

    #[test]
    fn test_pure_green_to_hsv() {
        let (h, s, v) = rgba_pixel_to_hsv(0, 255, 0);
        // OpenCV half-range: H = 120/2 = ~60
        assert!(h >= 58 && h <= 62, "Green hue should be near 60, got {}", h);
        assert_eq!(s, 255);
        assert_eq!(v, 255);
    }

    #[test]
    fn test_pure_blue_to_hsv() {
        let (h, s, v) = rgba_pixel_to_hsv(0, 0, 255);
        // OpenCV half-range: H = 240/2 = ~120
        assert!(h >= 118 && h <= 122, "Blue hue should be near 120, got {}", h);
        assert_eq!(s, 255);
        assert_eq!(v, 255);
    }

    #[test]
    fn test_black_to_hsv() {
        let (h, s, v) = rgba_pixel_to_hsv(0, 0, 0);
        assert_eq!(h, 0);
        assert_eq!(s, 0);
        assert_eq!(v, 0);
    }

    #[test]
    fn test_white_to_hsv() {
        let (h, s, v) = rgba_pixel_to_hsv(255, 255, 255);
        assert_eq!(h, 0);
        assert_eq!(s, 0);
        assert_eq!(v, 255);
    }

    #[test]
    fn test_count_matching_pixels_all_red() {
        let rule = make_hostile_marker_config();
        // 4x4 all-red frame: 16 pixels
        let pixels: Vec<(u8, u8, u8, u8)> = (0..16).map(|_| (255, 0, 0, 255)).collect();
        let rgba = make_rgba(&pixels);
        let count = count_matching_pixels(&rgba, &rule);
        assert_eq!(count, 16);
    }

    #[test]
    fn test_count_matching_pixels_all_black() {
        let rule = make_hostile_marker_config();
        // 4x4 all-black frame
        let pixels: Vec<(u8, u8, u8, u8)> = (0..16).map(|_| (0, 0, 0, 255)).collect();
        let rgba = make_rgba(&pixels);
        let count = count_matching_pixels(&rgba, &rule);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_matching_pixels_mixed() {
        let rule = make_hostile_marker_config();
        // 4x4 mixed: first 8 red, last 8 black
        let mut pixels: Vec<(u8, u8, u8, u8)> = (0..8).map(|_| (255, 0, 0, 255)).collect();
        pixels.extend((0..8).map(|_| (0, 0, 0, 255)));
        let rgba = make_rgba(&pixels);
        let count = count_matching_pixels(&rgba, &rule);
        assert_eq!(count, 8);
    }
}
