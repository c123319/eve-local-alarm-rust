import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

interface ConfigStatus {
  path: string;
  exists: boolean;
  valid: boolean;
  last_modified: number;
}

interface AlertConfig {
  enabled: boolean;
  sound_enabled: boolean;
  sound_path: string;
  toast_enabled: boolean;
  cooldown_ms: number;
}

interface DebugConfig {
  enabled: boolean;
  dump_hsv_masks: boolean;
  dump_overlays: boolean;
  debug_dir: string;
}

interface RoiConfig {
  id: string;
  name: string;
  capture_mode: string;
  region: { x: number; y: number; width: number; height: number };
}

interface MonitorConfig {
  targets: unknown[];
  rois: RoiConfig[];
  capture_fps: number;
  alert: AlertConfig;
  debug: DebugConfig;
}

interface DpiInfo {
  display_id: string;
  scale_factor: number;
}

type MonitoringStatus = 'Idle' | 'Starting' | 'Running' | 'Stopping' | 'Stopped' | 'Error';

interface MonitoringSnapshot {
  status: MonitoringStatus;
  last_error: string | null;
  capture_fps: number;
  last_frame_at_ms: number | null;
}

const createFallbackConfig = (): MonitorConfig => ({
  targets: [],
  rois: [],
  capture_fps: 5,
  alert: {
    enabled: true,
    sound_enabled: true,
    sound_path: '',
    toast_enabled: true,
    cooldown_ms: 3000,
  },
  debug: {
    enabled: false,
    dump_hsv_masks: false,
    dump_overlays: false,
    debug_dir: 'debug',
  },
});

const getErrorMessage = (error: unknown): string => {
  if (typeof error === 'string') {
    return error;
  }

  if (error instanceof Error) {
    return error.message;
  }

  return String(error);
};

export function SettingsPanel() {
  const [configStatus, setConfigStatus] = useState<ConfigStatus | null>(null);
  const [config, setConfig] = useState<MonitorConfig>(createFallbackConfig);
  const [message, setMessage] = useState('');

  const [dpiInfo, setDpiInfo] = useState<DpiInfo | null>(null);
  const [opencvStatus, setOpencvStatus] = useState<string>('待检查');

  // Monitoring State
  const [monitoringSnapshot, setMonitoringSnapshot] = useState<MonitoringSnapshot | null>(null);
  const [monitoringError, setMonitoringError] = useState<string | null>(null);

  useEffect(() => {
    // D-01: Auto-load last config on startup
    loadConfig();
    fetchConfigStatus();

    // 检查环境状态
    checkEnvironment();

    // Fetch initial monitoring status
    fetchMonitoringStatus();

    let unlistenStatus: UnlistenFn | undefined;
    let unlistenError: UnlistenFn | undefined;

    const setupListeners = async () => {
      unlistenStatus = await listen<MonitoringSnapshot>('monitoring-status', (event) => {
        setMonitoringSnapshot(event.payload);
        if (event.payload.status === 'Running' || event.payload.status === 'Idle' || event.payload.status === 'Stopped') {
           setMonitoringError(null);
        }
      });

      unlistenError = await listen<string>('monitoring-error', (event) => {
        setMonitoringError(event.payload);
        setMonitoringSnapshot(prev => prev ? { ...prev, status: 'Error', last_error: event.payload } : { status: 'Error', last_error: event.payload, capture_fps: 0, last_frame_at_ms: null });
      });
    };

    setupListeners();

    return () => {
      if (unlistenStatus) unlistenStatus();
      if (unlistenError) unlistenError();
    };
  }, []);

  const checkEnvironment = async () => {
    // 检查 DPI
    try {
      const dpi = await invoke<DpiInfo>('get_dpi_info');
      setDpiInfo(dpi);
    } catch (err) {
      console.error('Failed to get DPI info:', err);
    }

    // OpenCV 在 Phase 1 仅完成文档与构建约定，不做运行时探测
    setOpencvStatus('OpenCV 需按 BUILD.md 手动配置；当前界面不做运行时探测');
  };

  const fetchConfigStatus = async () => {
    try {
      const status = await invoke<ConfigStatus>('get_config_status');
      setConfigStatus(status);
    } catch (err) {
      console.error('Failed to fetch config status:', err);
    }
  };

  const fetchMonitoringStatus = async () => {
    try {
      const status = await invoke<MonitoringSnapshot>('get_monitoring_status');
      setMonitoringSnapshot(status);
    } catch (err) {
      console.error('Failed to fetch monitoring status:', err);
    }
  };

  const loadConfig = async () => {
    try {
      const loadedConfig = await invoke<MonitorConfig>('load_config');
      setConfig(loadedConfig);
      setMessage('配置已加载');
      fetchConfigStatus();
    } catch (err: unknown) {
      const fallbackConfig = await invoke<MonitorConfig>('get_default_config');
      setConfig(fallbackConfig);

      // D-04: Show clear warning for missing/corrupted config and fall back to defaults
      const errorMsg = getErrorMessage(err);
      if (errorMsg.includes('not found')) {
        setMessage('配置文件未找到，已使用默认配置');
      } else {
        setMessage(`配置文件无效，已回退默认配置：${errorMsg}`);
      }

      fetchConfigStatus();
    }
  };

  const saveConfig = async () => {
    try {
      await invoke('save_config', { config });
      setMessage('配置已保存');
      fetchConfigStatus();
    } catch (err: unknown) {
      setMessage(`保存配置失败: ${getErrorMessage(err)}`);
    }
  };

  const loadDefaults = async () => {
    try {
      const defaultConfig = await invoke<MonitorConfig>('get_default_config');
      setConfig(defaultConfig);
      setMessage('已加载默认配置');
    } catch (err: unknown) {
      setMessage(`加载默认配置失败: ${getErrorMessage(err)}`);
    }
  };

  const handleStartMonitoring = async () => {
    try {
      setMonitoringError(null);
      if (monitoringSnapshot) {
        setMonitoringSnapshot({ ...monitoringSnapshot, status: 'Starting' });
      } else {
        setMonitoringSnapshot({ status: 'Starting', last_error: null, capture_fps: config.capture_fps, last_frame_at_ms: null });
      }
      
      const snapshot = await invoke<MonitoringSnapshot>('start_monitoring', { config });
      setMonitoringSnapshot(snapshot);
    } catch (err: unknown) {
      const errorMsg = getErrorMessage(err);
      setMonitoringError(errorMsg);
      if (monitoringSnapshot) {
        setMonitoringSnapshot({ ...monitoringSnapshot, status: 'Error', last_error: errorMsg });
      }
    }
  };

  const handleStopMonitoring = async () => {
    if (!window.confirm('停止监控：确定要停止当前监控吗？')) {
      return;
    }
    
    try {
      if (monitoringSnapshot) {
        setMonitoringSnapshot({ ...monitoringSnapshot, status: 'Stopping' });
      }
      
      const snapshot = await invoke<MonitoringSnapshot>('stop_monitoring');
      setMonitoringSnapshot(snapshot);
    } catch (err: unknown) {
      const errorMsg = getErrorMessage(err);
      setMonitoringError(errorMsg);
    }
  };

  const getStatusTextAndColor = (status: MonitoringStatus | undefined) => {
    switch (status) {
      case 'Idle': return { text: '未启动', color: '#0f0f0f' };
      case 'Starting': return { text: '启动中...', color: '#e67e22' };
      case 'Running': return { text: '运行中', color: '#28a745' };
      case 'Stopping': return { text: '停止中...', color: '#e67e22' };
      case 'Stopped': return { text: '已停止', color: '#0f0f0f' };
      case 'Error': return { text: '错误', color: '#dc3545' };
      default: return { text: '未启动', color: '#0f0f0f' };
    }
  };

  const statusInfo = getStatusTextAndColor(monitoringSnapshot?.status);
  const isMonitoring = monitoringSnapshot?.status === 'Running';
  const isStarting = monitoringSnapshot?.status === 'Starting';
  const isStopping = monitoringSnapshot?.status === 'Stopping';
  const displayFps = monitoringSnapshot?.capture_fps ?? config.capture_fps ?? 0;

  return (
    <div style={{ padding: '20px', fontFamily: 'sans-serif' }}>
      <h1>EVE 本地警报 - 设置</h1>

      {/* D-18: Configuration status and environment health equally prominent */}
      <section style={{ marginBottom: '30px' }}>
        <h2>配置状态</h2>
        {configStatus ? (
          <div>
            <div style={{ marginBottom: '10px' }}>
              <strong>配置文件路径：</strong> {configStatus.path}
            </div>
            <div style={{ marginBottom: '10px' }}>
              <strong>文件存在：</strong> {configStatus.exists ? '是' : '否'}
            </div>
            <div style={{ marginBottom: '10px' }}>
              <strong>配置有效：</strong> {configStatus.valid ? '是' : '否'}
            </div>
            {configStatus.last_modified > 0 && (
              <div style={{ marginBottom: '10px' }}>
                <strong>最后修改：</strong>{' '}
                {new Date(configStatus.last_modified * 1000).toLocaleString('zh-CN')}
              </div>
            )}
          </div>
        ) : (
          <div>加载中...</div>
        )}
      </section>

      <section style={{ marginBottom: '30px' }}>
        <h2>环境检查</h2>
        <div className="environment-status">
          <div style={{ marginBottom: '10px' }}>
            <strong>OpenCV：</strong> {opencvStatus}
          </div>
          {dpiInfo && (
            <div>
              <div style={{ marginBottom: '10px' }}>
                <strong>DPI 缩放：</strong> {(dpiInfo.scale_factor * 100).toFixed(0)}%
              </div>
              <div style={{ marginBottom: '10px' }}>
                <strong>显示器 ID：</strong> {dpiInfo.display_id}
              </div>
              <div style={{ marginBottom: '10px', color: '#856404' }}>
                当前 DPI 信息为 Phase 1 基线占位；Windows API 实时读取将在后续捕获/ROI 阶段接入。
              </div>
            </div>
          )}
        </div>
      </section>

      <section style={{ marginBottom: '24px' }}>
        <h2>监控控制</h2>
        <div style={{ marginBottom: '16px' }} aria-live="polite">
          <strong>当前状态：</strong>
          <span style={{ color: statusInfo.color }}>{statusInfo.text}</span>
        </div>

        <div style={{ marginBottom: '16px', color: '#856404' }}>
          ⚠️ MSS 模式仅捕获屏幕可见区域。请确保监控区域不被遮挡。
        </div>

        <div style={{ marginBottom: '16px' }}>
          <strong>捕获帧率：</strong> {displayFps} FPS
        </div>

        <div>
          <button
            onClick={handleStartMonitoring}
            disabled={isMonitoring || isStarting || isStopping}
            style={{ marginRight: '10px', padding: '8px 16px' }}
          >
            开始监控
          </button>
          <button
            onClick={handleStopMonitoring}
            disabled={!isMonitoring || isStarting || isStopping}
            style={{ padding: '8px 16px' }}
          >
            停止监控
          </button>
        </div>

        {monitoringError && (
          <div style={{ marginTop: '16px', padding: '10px', backgroundColor: '#f8d7da', borderRadius: '4px', color: '#721c24' }}>
            监控失败：{monitoringError}。请检查配置后重试。
          </div>
        )}
      </section>

      <section>
        <h2>配置操作</h2>
        <div style={{ marginBottom: '10px' }}>
          <strong>当前默认冷却：</strong> {config.alert.cooldown_ms} ms
        </div>
        <div style={{ marginBottom: '10px' }}>
          <strong>Toast 提醒：</strong> {config.alert.toast_enabled ? '开启' : '关闭'}
        </div>
        <div style={{ marginBottom: '10px' }}>
          <strong>调试输出目录：</strong> {config.debug.debug_dir}
        </div>
        <div>
          <button
            onClick={saveConfig}
            style={{ marginRight: '10px', padding: '8px 16px' }}
          >
            保存配置
          </button>
          <button
            onClick={loadConfig}
            style={{ marginRight: '10px', padding: '8px 16px' }}
          >
            加载配置
          </button>
          <button
            onClick={loadDefaults}
            style={{ padding: '8px 16px' }}
          >
            恢复默认
          </button>
        </div>
      </section>

      {message && (
        <div style={{ marginTop: '20px', padding: '10px', backgroundColor: '#f0f0f0', borderRadius: '4px' }}>
          {message}
        </div>
      )}
    </div>
  );
}
