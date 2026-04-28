use crate::models::ColorMatchConfig;

/// 验证颜色匹配配置的阈值参数
///
/// 规则（per D-13）:
/// - min_pixels > 0
/// - min_ratio in (0.0, 1.0]
/// - hsv_lower <= hsv_upper per channel
pub fn validate_color_match_config(config: &ColorMatchConfig) -> Result<(), String> {
    if config.min_pixels == 0 {
        return Err("最小像素数必须大于 0".to_string());
    }
    if config.min_ratio <= 0.0 || config.min_ratio > 1.0 {
        return Err("最小像素比例必须在 (0.0, 1.0] 范围内".to_string());
    }
    for ch in 0..3 {
        if config.hsv_lower[ch] > config.hsv_upper[ch] {
            return Err(format!(
                "HSV 下界不能大于上界 (通道 {}): {} > {}",
                ch, config.hsv_lower[ch], config.hsv_upper[ch]
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_valid_config() -> ColorMatchConfig {
        ColorMatchConfig::default_hostile_marker()
    }

    #[test]
    fn test_valid_config_passes() {
        let config = make_valid_config();
        assert!(validate_color_match_config(&config).is_ok());
    }

    #[test]
    fn test_min_pixels_zero_rejected() {
        let mut config = make_valid_config();
        config.min_pixels = 0;
        let result = validate_color_match_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("最小像素数必须大于 0"),
            "Error message should contain expected Chinese text, got: {}",
            err
        );
    }

    #[test]
    fn test_min_ratio_zero_rejected() {
        let mut config = make_valid_config();
        config.min_ratio = 0.0;
        let result = validate_color_match_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("最小像素比例必须在 (0.0, 1.0] 范围内"),
            "Error message should contain expected Chinese text, got: {}",
            err
        );
    }

    #[test]
    fn test_min_ratio_above_one_rejected() {
        let mut config = make_valid_config();
        config.min_ratio = 1.5;
        let result = validate_color_match_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("最小像素比例必须在 (0.0, 1.0] 范围内"),
            "Error message should contain expected Chinese text, got: {}",
            err
        );
    }

    #[test]
    fn test_min_ratio_one_accepted() {
        let mut config = make_valid_config();
        config.min_ratio = 1.0;
        assert!(validate_color_match_config(&config).is_ok());
    }

    #[test]
    fn test_hsv_lower_h_greater_than_upper_rejected() {
        let mut config = make_valid_config();
        config.hsv_lower = [20, 120, 120];
        config.hsv_upper = [15, 255, 255];
        let result = validate_color_match_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("HSV 下界不能大于上界"),
            "Error message should contain expected Chinese text, got: {}",
            err
        );
    }

    #[test]
    fn test_hsv_lower_s_greater_than_upper_rejected() {
        let mut config = make_valid_config();
        config.hsv_lower = [0, 200, 120];
        config.hsv_upper = [15, 150, 255];
        let result = validate_color_match_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("HSV 下界不能大于上界"),
            "Error message should contain expected Chinese text, got: {}",
            err
        );
    }

    #[test]
    fn test_hsv_lower_v_greater_than_upper_rejected() {
        let mut config = make_valid_config();
        config.hsv_lower = [0, 120, 200];
        config.hsv_upper = [15, 255, 150];
        let result = validate_color_match_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("HSV 下界不能大于上界"),
            "Error message should contain expected Chinese text, got: {}",
            err
        );
    }
}
