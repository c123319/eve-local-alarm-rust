use crate::models::Rect;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// 捕获的帧数据
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapturedFrame {
    /// ROI ID
    pub roi_id: String,
    /// 捕获区域（物理像素）
    pub region: Rect,
    /// 捕获时间戳（毫秒）
    pub captured_at_ms: u128,
    /// 帧宽度（像素）
    pub width: u32,
    /// 帧高度（像素）
    pub height: u32,
    /// RGBA 像素数据
    pub rgba: Vec<u8>,
}

/// 监控状态
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MonitoringStatus {
    /// 空闲
    Idle,
    /// 启动中
    Starting,
    /// 运行中
    Running,
    /// 停止中
    Stopping,
    /// 已停止
    Stopped,
    /// 错误
    Error,
}

/// 监控快照
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitoringSnapshot {
    /// 当前状态
    pub status: MonitoringStatus,
    /// 最后的错误消息
    pub last_error: Option<String>,
    /// 捕获帧率
    pub capture_fps: u32,
    /// 最后一帧的捕获时间（毫秒）
    pub last_frame_at_ms: Option<u128>,
}

/// MSS 捕获工作器
pub struct MssCaptureWorker {
    /// 取消标志
    cancellation: Arc<AtomicBool>,
    /// 工作线程句柄
    thread_handle: Option<thread::JoinHandle<()>>,
    /// 最新帧存储（最新帧获胜）
    latest_frame: Arc<Mutex<Option<CapturedFrame>>>,
    /// 配置参数
    roi_id: String,
    region: Rect,
    capture_fps: u32,
}

impl MssCaptureWorker {
    /// 创建新的捕获工作器
    pub fn new(roi_id: String, region: Rect, capture_fps: u32) -> Self {
        Self {
            cancellation: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
            latest_frame: Arc::new(Mutex::new(None)),
            roi_id,
            region,
            capture_fps,
        }
    }

    /// 启动捕获工作器
    pub fn start(&mut self) -> Result<(), String> {
        // 验证 ROI
        validate_roi(&self.region)?;

        // 验证 FPS
        let validated_fps = validate_capture_fps(self.capture_fps)?;

        // 检查是否已经在运行
        if self.thread_handle.is_some() {
            return Err("监控已在运行中".to_string());
        }

        // 重置取消标志
        self.cancellation.store(false, Ordering::SeqCst);

        // 克隆必要的所有权
        let cancellation = Arc::clone(&self.cancellation);
        let latest_frame = Arc::clone(&self.latest_frame);
        let roi_id = self.roi_id.clone();
        let region = self.region.clone();
        let capture_fps = validated_fps;

        // 启动捕获线程
        let handle = thread::spawn(move || {
            Self::capture_loop(cancellation, latest_frame, roi_id, region, capture_fps);
        });

        self.thread_handle = Some(handle);
        Ok(())
    }

    /// 停止捕获工作器
    pub fn stop(&mut self) -> Result<(), String> {
        // 设置取消标志
        self.cancellation.store(true, Ordering::SeqCst);

        // 等待线程结束
        if let Some(handle) = self.thread_handle.take() {
            handle.join().map_err(|e| {
                format!("等待捕获线程结束失败: {:?}", e)
            })?;
        }

        Ok(())
    }

    /// 获取最新帧
    pub fn get_latest_frame(&self) -> Option<CapturedFrame> {
        self.latest_frame.lock().unwrap().clone()
    }

    /// 捕获循环
    fn capture_loop(
        cancellation: Arc<AtomicBool>,
        latest_frame: Arc<Mutex<Option<CapturedFrame>>>,
        roi_id: String,
        region: Rect,
        capture_fps: u32,
    ) {
        let frame_duration = Duration::from_millis(1000 / capture_fps as u64);

        loop {
            // 检查取消标志
            if cancellation.load(Ordering::SeqCst) {
                break;
            }

            let start_time = Instant::now();

            // 执行捕获
            match Self::capture_frame(&roi_id, &region) {
                Ok(frame) => {
                    // 存储最新帧（最新帧获胜）
                    let mut guard = latest_frame.lock().unwrap();
                    *guard = Some(frame);
                    drop(guard);
                }
                Err(e) => {
                    // 记录错误但继续尝试
                    eprintln!("捕获失败: {}", e);
                }
            }

            // 计算剩余等待时间
            let elapsed = start_time.elapsed();
            if elapsed < frame_duration {
                thread::sleep(frame_duration - elapsed);
            }
        }
    }

    /// 执行单帧捕获
    fn capture_frame(roi_id: &str, region: &Rect) -> Result<CapturedFrame, String> {
        // 验证坐标非负
        if region.x < 0 || region.y < 0 {
            return Err("ROI 坐标必须为非负值".to_string());
        }

        // 使用 XCap 捕获区域
        let monitors = xcap::Monitor::all()
            .map_err(|e| format!("获取显示器列表失败: {}", e))?;

        let monitor = monitors.into_iter().next()
            .ok_or_else(|| "未找到可用显示器".to_string())?;

        let image = monitor.capture_region(
            region.x as u32,
            region.y as u32,
            region.width,
            region.height
        )
            .map_err(|e| format!("MSS 区域捕获失败: {}", e))?;

        // ImageBuffer 已经是 RGBA 格式，提取原始字节
        let rgba = image.as_raw().to_vec();

        Ok(CapturedFrame {
            roi_id: roi_id.to_string(),
            region: region.clone(),
            captured_at_ms: now_millis(),
            width: region.width,
            height: region.height,
            rgba,
        })
    }
}

impl Drop for MssCaptureWorker {
    fn drop(&mut self) {
        // 确保 Drop 时停止捕获
        if self.thread_handle.is_some() {
            let _ = self.stop();
        }
    }
}

/// 验证捕获帧率
pub fn validate_capture_fps(fps: u32) -> Result<u32, String> {
    if (1..=30).contains(&fps) {
        Ok(fps)
    } else {
        Err("捕获帧率必须在 1 到 30 FPS 之间".to_string())
    }
}

/// 验证 ROI 区域
pub fn validate_roi(region: &Rect) -> Result<(), String> {
    if region.width == 0 || region.height == 0 {
        Err("ROI 宽度和高度必须大于 0".to_string())
    } else {
        Ok(())
    }
}

/// 获取当前时间（毫秒）
pub fn now_millis() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_positive_roi_dimensions() {
        let region = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 80,
        };
        assert!(validate_roi(&region).is_ok());
    }

    #[test]
    fn rejects_zero_width_roi() {
        let region = Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 80,
        };
        let result = validate_roi(&region);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ROI 宽度和高度必须大于 0"));
    }

    #[test]
    fn rejects_zero_height_roi() {
        let region = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 0,
        };
        let result = validate_roi(&region);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ROI 宽度和高度必须大于 0"));
    }

    #[test]
    fn validates_fps_range() {
        assert!(validate_capture_fps(1).is_ok());
        assert!(validate_capture_fps(5).is_ok());
        assert!(validate_capture_fps(30).is_ok());
    }

    #[test]
    fn rejects_invalid_fps_range() {
        let result = validate_capture_fps(0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("捕获帧率必须在 1 到 30 FPS 之间"));

        let result = validate_capture_fps(31);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("捕获帧率必须在 1 到 30 FPS 之间"));
    }

    #[test]
    fn monitoring_snapshot_defaults_to_idle() {
        let snapshot = MonitoringSnapshot {
            status: MonitoringStatus::Idle,
            last_error: None,
            capture_fps: 5,
            last_frame_at_ms: None,
        };
        assert_eq!(snapshot.status, MonitoringStatus::Idle);
    }
}
