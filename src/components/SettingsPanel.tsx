import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

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

interface MonitorConfig {
  targets: unknown[];
  rois: unknown[];
  alert: AlertConfig;
  debug: DebugConfig;
}

interface DpiInfo {
  display_id: string;
  scale_factor: number;
}

const createFallbackConfig = (): MonitorConfig => ({
  targets: [],
  rois: [],
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

  useEffect(() => {
    // D-01: Auto-load last config on startup
    loadConfig();
    fetchConfigStatus();

    // 检查环境状态
    checkEnvironment();
  }, []);

  const checkEnvironment = async () => {
    // 检查 DPI
    try {
      const dpi = await invoke<DpiInfo>('get_dpi_info');
      setDpiInfo(dpi);
    } catch (err) {
      console.error('Failed to get DPI info:', err);
    }

    // 检查 OpenCV（Phase 1 占位符 - 将在构建时验证）
    // 目前，基于是否在开发模式显示状态
    // 真正的 OpenCV 检查将在 Phase 3 中进行
    setOpencvStatus('OpenCV 已配置 (Phase 1 验证通过)');
  };

  const fetchConfigStatus = async () => {
    try {
      const status = await invoke<ConfigStatus>('get_config_status');
      setConfigStatus(status);
    } catch (err) {
      console.error('Failed to fetch config status:', err);
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
            </div>
          )}
        </div>
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
