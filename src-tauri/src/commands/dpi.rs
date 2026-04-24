use crate::dpi::{DpiInfo, DpiInvalidationResult, get_current_dpi, check_dpi_invalidation};
use crate::models::Rect;

/// 获取当前显示器的 DPI 信息
#[tauri::command]
pub async fn get_dpi_info() -> Result<DpiInfo, String> {
    get_current_dpi().ok_or("无法获取 DPI 信息".to_string())
}

/// 验证 ROI 坐标是否因 DPI 变化而失效
#[tauri::command]
pub async fn validate_roi_coordinates(
    roi_rect: Rect,
    stored_dpi_scale: f64,
    stored_display_id: Option<String>,
) -> Result<DpiInvalidationResult, String> {
    let current_dpi = get_current_dpi()
        .ok_or("无法获取当前 DPI".to_string())?;

    let stored_flags = crate::models::DpiInvalidationFlags {
        invalid: false,
        last_dpi_scale: stored_dpi_scale,
        last_display_id: stored_display_id,
    };

    let result = check_dpi_invalidation(&roi_rect, &current_dpi, &stored_flags);
    Ok(result)
}
