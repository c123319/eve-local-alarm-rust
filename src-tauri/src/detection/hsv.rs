use crate::models::ColorMatchConfig;

/// 将 RGBA 像素转换为 HSV 颜色空间（OpenCV 半范围约定：H:0-179, S:0-255, V:0-255）
#[inline]
pub fn rgba_pixel_to_hsv(_r: u8, _g: u8, _b: u8) -> (u8, u8, u8) {
    todo!()
}

/// 统计 RGBA 缓冲区内落在指定 HSV 范围内的像素数量
pub fn count_matching_pixels(_rgba: &[u8], _rule: &ColorMatchConfig) -> usize {
    todo!()
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
        // OpenCV half-range: H = 60/2 = ~30
        assert!(h >= 28 && h <= 32, "Green hue should be near 30, got {}", h);
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
