//! HSV color matching detection engine.
//!
//! Evaluates captured frames against color rules using OR logic across rules
//! (any rule match triggers detection) and AND logic within each rule
//! (min_pixels AND min_ratio must both be met).

use crate::capture::CapturedFrame;
use crate::models::ColorMatchConfig;
use super::hsv::count_matching_pixels;
use serde::{Deserialize, Serialize};

/// 单条颜色规则匹配结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleMatchResult {
    /// 规则名称
    pub rule_name: String,
    /// 该规则是否匹配
    pub matched: bool,
    /// 匹配的像素数量
    pub pixel_count: u32,
    /// 匹配像素占总像素的比例
    pub ratio: f64,
}

/// 单帧检测结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetectionResult {
    /// ROI ID
    pub roi_id: String,
    /// 是否检测到目标（任一规则匹配即为 true）
    pub detected: bool,
    /// 每条规则的评价结果
    pub rule_results: Vec<RuleMatchResult>,
    /// 源帧时间戳（毫秒）
    pub frame_timestamp_ms: u128,
    /// 评价完成时间戳（毫秒）
    pub evaluated_at_ms: u128,
}

/// HSV 颜色匹配检测引擎
#[derive(Clone)]
pub struct DetectionEngine {
    color_rules: Vec<ColorMatchConfig>,
}

impl DetectionEngine {
    /// 创建检测引擎
    pub fn new(color_rules: Vec<ColorMatchConfig>) -> Self {
        Self { color_rules }
    }

    /// 评价单帧检测结果
    ///
    /// OR 逻辑（D-04）：任一规则匹配即判定为检测到目标。
    /// AND 逻辑（D-05）：单条规则内 min_pixels 和 min_ratio 必须同时满足。
    pub fn evaluate_frame(&self, frame: &CapturedFrame) -> DetectionResult {
        let total_pixels = (frame.width as usize)
            .checked_mul(frame.height as usize)
            .unwrap_or(0);

        // 防御性检查：零面积帧直接返回未检测
        if total_pixels == 0 {
            return DetectionResult {
                roi_id: frame.roi_id.clone(),
                detected: false,
                rule_results: Vec::new(),
                frame_timestamp_ms: frame.captured_at_ms,
                evaluated_at_ms: crate::capture::now_millis(),
            };
        }

        // 防御性检查：RGBA 缓冲区大小不匹配
        let expected_len = total_pixels * 4;
        if frame.rgba.len() < expected_len {
            return DetectionResult {
                roi_id: frame.roi_id.clone(),
                detected: false,
                rule_results: Vec::new(),
                frame_timestamp_ms: frame.captured_at_ms,
                evaluated_at_ms: crate::capture::now_millis(),
            };
        }

        let mut rule_results = Vec::with_capacity(self.color_rules.len());
        let mut any_matched = false;

        for rule in &self.color_rules {
            let matched_count = count_matching_pixels(&frame.rgba, rule);
            let ratio = matched_count as f64 / total_pixels as f64;
            let pixel_threshold_met = matched_count >= rule.min_pixels as usize;
            let ratio_threshold_met = ratio >= rule.min_ratio;
            let rule_matched = pixel_threshold_met && ratio_threshold_met;

            if rule_matched {
                any_matched = true;
            }

            rule_results.push(RuleMatchResult {
                rule_name: rule.name.clone(),
                matched: rule_matched,
                pixel_count: matched_count as u32,
                ratio,
            });
        }

        DetectionResult {
            roi_id: frame.roi_id.clone(),
            detected: any_matched,
            rule_results,
            frame_timestamp_ms: frame.captured_at_ms,
            evaluated_at_ms: crate::capture::now_millis(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Rect;

    fn make_captured_frame(width: u32, height: u32, pixels: &[(u8, u8, u8, u8)]) -> CapturedFrame {
        let mut rgba = Vec::with_capacity(pixels.len() * 4);
        for &(r, g, b, a) in pixels {
            rgba.extend_from_slice(&[r, g, b, a]);
        }
        CapturedFrame {
            roi_id: "test-roi".to_string(),
            region: Rect { x: 0, y: 0, width, height },
            captured_at_ms: 1000,
            width,
            height,
            rgba,
        }
    }

    fn hostile_marker_config() -> ColorMatchConfig {
        ColorMatchConfig::default_hostile_marker()
    }

    fn make_all_red_frame(w: u32, h: u32) -> CapturedFrame {
        let total = (w as usize) * (h as usize);
        let pixels: Vec<(u8, u8, u8, u8)> = (0..total).map(|_| (255, 0, 0, 255)).collect();
        make_captured_frame(w, h, &pixels)
    }

    fn make_all_black_frame(w: u32, h: u32) -> CapturedFrame {
        let total = (w as usize) * (h as usize);
        let pixels: Vec<(u8, u8, u8, u8)> = (0..total).map(|_| (0, 0, 0, 255)).collect();
        make_captured_frame(w, h, &pixels)
    }

    #[test]
    fn test_all_red_frame_detected() {
        let engine = DetectionEngine::new(vec![hostile_marker_config()]);
        let frame = make_all_red_frame(4, 4);
        let result = engine.evaluate_frame(&frame);
        assert!(result.detected, "All-red frame should be detected");
        assert_eq!(result.rule_results.len(), 1);
        assert_eq!(result.rule_results[0].pixel_count, 16);
        assert!(result.rule_results[0].matched);
    }

    #[test]
    fn test_all_black_frame_not_detected() {
        let engine = DetectionEngine::new(vec![hostile_marker_config()]);
        let frame = make_all_black_frame(4, 4);
        let result = engine.evaluate_frame(&frame);
        assert!(!result.detected, "All-black frame should not be detected");
        assert_eq!(result.rule_results.len(), 1);
        assert!(!result.rule_results[0].matched);
    }

    #[test]
    fn test_mixed_frame_detected() {
        // 4x4 frame: first 12 pixels red, last 4 black
        // min_pixels=12, so 12 >= 12 should trigger
        let mut pixels: Vec<(u8, u8, u8, u8)> = (0..12).map(|_| (255, 0, 0, 255)).collect();
        pixels.extend((0..4).map(|_| (0, 0, 0, 255)));
        let frame = make_captured_frame(4, 4, &pixels);
        let engine = DetectionEngine::new(vec![hostile_marker_config()]);
        let result = engine.evaluate_frame(&frame);
        assert!(result.detected, "Mixed frame with 12 red pixels should be detected (12 >= min_pixels=12)");
    }

    #[test]
    fn test_sparse_red_not_detected() {
        // 4x4 frame: 6 red pixels, 10 black pixels
        // min_pixels=12, so 6 < 12 should NOT trigger
        let mut pixels: Vec<(u8, u8, u8, u8)> = (0..6).map(|_| (255, 0, 0, 255)).collect();
        pixels.extend((0..10).map(|_| (0, 0, 0, 255)));
        let frame = make_captured_frame(4, 4, &pixels);
        let engine = DetectionEngine::new(vec![hostile_marker_config()]);
        let result = engine.evaluate_frame(&frame);
        assert!(!result.detected, "Sparse red frame (6 pixels) should not be detected (6 < min_pixels=12)");
    }

    #[test]
    fn test_or_logic_first_fails_second_succeeds() {
        // Rule 1: hostile-marker (red), won't match on blue
        // Rule 2: blue rule that matches blue pixels
        let blue_rule = ColorMatchConfig {
            name: "蓝色规则".to_string(),
            hsv_lower: [100, 120, 120],
            hsv_upper: [130, 255, 255],
            min_pixels: 1,
            min_ratio: 0.01,
        };
        let engine = DetectionEngine::new(vec![hostile_marker_config(), blue_rule]);

        // All-blue frame
        let total = 16usize;
        let pixels: Vec<(u8, u8, u8, u8)> = (0..total).map(|_| (0, 0, 255, 255)).collect();
        let frame = make_captured_frame(4, 4, &pixels);

        let result = engine.evaluate_frame(&frame);
        assert!(result.detected, "Blue frame should be detected by blue rule (OR logic)");
        assert!(!result.rule_results[0].matched, "Red rule should NOT match blue frame");
        assert!(result.rule_results[1].matched, "Blue rule SHOULD match blue frame");
    }

    #[test]
    fn test_or_logic_all_fail() {
        let blue_rule = ColorMatchConfig {
            name: "蓝色规则".to_string(),
            hsv_lower: [100, 120, 120],
            hsv_upper: [130, 255, 255],
            min_pixels: 1,
            min_ratio: 0.01,
        };
        let engine = DetectionEngine::new(vec![hostile_marker_config(), blue_rule]);
        let frame = make_all_black_frame(4, 4);
        let result = engine.evaluate_frame(&frame);
        assert!(!result.detected, "All-black frame should not be detected by any rule");
        assert!(result.rule_results.iter().all(|r| !r.matched));
    }

    #[test]
    fn test_zero_area_frame_safe() {
        let engine = DetectionEngine::new(vec![hostile_marker_config()]);
        let frame = CapturedFrame {
            roi_id: "zero-roi".to_string(),
            region: Rect { x: 0, y: 0, width: 0, height: 0 },
            captured_at_ms: 1000,
            width: 0,
            height: 0,
            rgba: Vec::new(),
        };
        let result = engine.evaluate_frame(&frame);
        assert!(!result.detected, "Zero-area frame should not be detected");
        assert!(result.rule_results.is_empty(), "Zero-area frame should have no rule results");
    }

    #[test]
    fn test_roi_id_carried_through() {
        let engine = DetectionEngine::new(vec![hostile_marker_config()]);
        let mut frame = make_all_red_frame(4, 4);
        frame.roi_id = "my-special-roi".to_string();
        let result = engine.evaluate_frame(&frame);
        assert_eq!(result.roi_id, "my-special-roi");
    }
}
