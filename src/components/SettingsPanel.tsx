import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface ConfigStatus {
  path: string;
  exists: boolean;
  valid: boolean;
  last_modified: number;
}

export function SettingsPanel() {
  const [configStatus, setConfigStatus] = useState<ConfigStatus | null>(null);
  const [message, setMessage] = useState('');

  useEffect(() => {
    // D-01: Auto-load last config on startup
    loadConfig();
    fetchConfigStatus();
  }, []);

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
      await invoke('load_config');
      setMessage('配置已加载');
      fetchConfigStatus();
    } catch (err: any) {
      // D-04: Show clear warning for missing/corrupted config
      const errorMsg = err as string;
      if (errorMsg.includes('not found')) {
        setMessage('配置文件未找到，已使用默认配置');
      } else {
        setMessage(`加载配置失败: ${errorMsg}`);
      }
    }
  };

  const saveConfig = async () => {
    try {
      await invoke('save_config', { config: {} }); // Placeholder for actual config
      setMessage('配置已保存');
      fetchConfigStatus();
    } catch (err: any) {
      setMessage(`保存配置失败: ${err}`);
    }
  };

  const loadDefaults = async () => {
    try {
      await invoke('get_default_config');
      setMessage('已加载默认配置');
    } catch (err: any) {
      setMessage(`加载默认配置失败: ${err}`);
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
        {/* Placeholder for Plan 03 - OpenCV status */}
        <div>
          <strong>OpenCV 状态：</strong> 待检查 (Phase 1-03)
        </div>
      </section>

      <section>
        <h2>配置操作</h2>
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
