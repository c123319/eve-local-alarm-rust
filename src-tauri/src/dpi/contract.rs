use serde::{Serialize, Deserialize};

/// 内部坐标：物理（设备）像素
/// 这是捕获 API 返回的，也是我们在配置中存储的
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PhysicalCoord {
    pub x: i32,
    pub y: i32,
}

/// 显示坐标：用户可见（DPI 缩放）的像素
/// 这是 UI 显示的，也是用户交互的
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayCoord {
    pub x: i32,
    pub y: i32,
}

/// 显示器的 DPI 信息
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DpiInfo {
    pub display_id: String,
    pub scale_factor: f64, // 1.0 = 100%, 1.5 = 150%, 2.0 = 200%
}

/// 将显示坐标转换为物理坐标
pub fn to_physical(display: DisplayCoord, scale: f64) -> PhysicalCoord {
    PhysicalCoord {
        x: (display.x as f64 * scale).round() as i32,
        y: (display.y as f64 * scale).round() as i32,
    }
}

/// 将物理坐标转换为显示坐标
pub fn to_display(physical: PhysicalCoord, scale: f64) -> DisplayCoord {
    DisplayCoord {
        x: (physical.x as f64 / scale).round() as i32,
        y: (physical.y as f64 / scale).round() as i32,
    }
}

/// 检查 ROI 坐标是否需要 DPI 失效
pub fn check_dpi_invalidation(
    _roi_physical_rect: &crate::models::Rect,
    current_dpi: &DpiInfo,
    stored_flags: &crate::models::DpiInvalidationFlags,
) -> DpiInvalidationResult {
    // D-11: 检查 DPI 缩放是否已更改
    let scale_changed = (stored_flags.last_dpi_scale - current_dpi.scale_factor).abs() > 0.01;

    // D-11: 检查显示器是否已更改
    let display_changed = stored_flags.last_display_id.as_ref()
        .map(|id| id != &current_dpi.display_id)
        .unwrap_or(true);

    let invalid = scale_changed || display_changed;

    DpiInvalidationResult {
        invalid,
        reason: if invalid {
            if scale_changed { Some("DPI 缩放已更改".to_string()) }
            else { Some("显示器已更改".to_string()) }
        } else {
            None
        },
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DpiInvalidationResult {
    pub invalid: bool,
    pub reason: Option<String>, // 中文原因消息
}

/// 获取当前显示器的 DPI 信息
pub fn get_current_dpi() -> Option<DpiInfo> {
    // 实现将在 Phase 2 中使用 Windows API
    // 目前返回默认值
    Some(DpiInfo {
        display_id: "default".to_string(),
        scale_factor: 1.0, // 默认为 100%
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_physical() {
        let display = DisplayCoord { x: 100, y: 100 };
        let physical = to_physical(display, 1.5); // 150% DPI
        assert_eq!(physical.x, 150);
        assert_eq!(physical.y, 150);
    }

    #[test]
    fn test_to_display() {
        let physical = PhysicalCoord { x: 150, y: 150 };
        let display = to_display(physical, 1.5);
        assert_eq!(display.x, 100);
        assert_eq!(display.y, 100);
    }

    #[test]
    fn test_dpi_invalidation_detection() {
        let roi_rect = crate::models::Rect::default();
        let current_dpi = DpiInfo {
            display_id: "display1".to_string(),
            scale_factor: 1.5,
        };

        // DPI 缩放已更改
        let stored_flags = crate::models::DpiInvalidationFlags {
            invalid: false,
            last_dpi_scale: 1.0,
            last_display_id: Some("display1".to_string()),
        };
        let result = check_dpi_invalidation(&roi_rect, &current_dpi, &stored_flags);
        assert!(result.invalid);
        assert_eq!(result.reason, Some("DPI 缩放已更改".to_string()));

        // 显示器已更改
        let stored_flags = crate::models::DpiInvalidationFlags {
            invalid: false,
            last_dpi_scale: 1.5,
            last_display_id: Some("display2".to_string()),
        };
        let result = check_dpi_invalidation(&roi_rect, &current_dpi, &stored_flags);
        assert!(result.invalid);
        assert_eq!(result.reason, Some("显示器已更改".to_string()));

        // 无更改
        let stored_flags = crate::models::DpiInvalidationFlags {
            invalid: false,
            last_dpi_scale: 1.5,
            last_display_id: Some("display1".to_string()),
        };
        let result = check_dpi_invalidation(&roi_rect, &current_dpi, &stored_flags);
        assert!(!result.invalid);
        assert_eq!(result.reason, None);
    }

    #[test]
    fn test_to_physical_rounding() {
        // 测试四舍五入行为
        let display = DisplayCoord { x: 100, y: 100 };
        let physical = to_physical(display, 1.25); // 125% DPI
        assert_eq!(physical.x, 125);
        assert_eq!(physical.y, 125);
    }

    #[test]
    fn test_to_display_rounding() {
        // 测试四舍五入行为
        let physical = PhysicalCoord { x: 125, y: 125 };
        let display = to_display(physical, 1.25);
        assert_eq!(display.x, 100);
        assert_eq!(display.y, 100);
    }
}
