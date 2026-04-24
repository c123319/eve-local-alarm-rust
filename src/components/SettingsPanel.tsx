import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface ConfigStatus {
  path: string;
  lastModified: string;
}

interface EnvironmentCheck {
  opencvStatus: string;
}

export default function SettingsPanel() {
  const [configStatus, setConfigStatus] = useState<ConfigStatus>({
    path: '未加载',
    lastModified: '未加载'
  });
  const [_envCheck, _setEnvCheck] = useState<EnvironmentCheck>({
    opencvStatus: '待检查（Plan 03 实现）'
  });
  const [isLoading, setIsLoading] = useState(false);

  const handleSave = async () => {
    setIsLoading(true);
    try {
      await invoke('save_config', { config: {} });
      setConfigStatus({
        ...configStatus,
        lastModified: new Date().toLocaleString('zh-CN')
      });
    } catch (error) {
      console.error('保存配置失败:', error);
      alert('保存配置失败');
    } finally {
      setIsLoading(false);
    }
  };

  const handleLoad = async () => {
    setIsLoading(true);
    try {
      const config = await invoke('save_config', { config: {} });
      console.log('配置已加载:', config);
      setConfigStatus({
        path: '配置文件路径（Plan 02 实现）',
        lastModified: new Date().toLocaleString('zh-CN')
      });
    } catch (error) {
      console.error('加载配置失败:', error);
      alert('加载配置失败');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div style={{ padding: '20px', fontFamily: 'sans-serif' }}>
      <h1>EVE 本地警报 - 设置</h1>

      {/* 配置状态 Section */}
      <section style={{ marginBottom: '30px' }}>
        <h2>配置状态</h2>
        <div style={{ marginBottom: '10px' }}>
          <strong>配置文件路径：</strong> {configStatus.path}
        </div>
        <div style={{ marginBottom: '10px' }}>
          <strong>最后修改时间：</strong> {configStatus.lastModified}
        </div>
        <div>
          <button
            onClick={handleSave}
            disabled={isLoading}
            style={{ marginRight: '10px', padding: '8px 16px' }}
          >
            保存配置
          </button>
          <button
            onClick={handleLoad}
            disabled={isLoading}
            style={{ padding: '8px 16px' }}
          >
            加载配置
          </button>
        </div>
      </section>

      {/* 环境检查 Section */}
      <section>
        <h2>环境检查</h2>
        <div>
          <strong>OpenCV 状态：</strong> 待检查（Plan 03 实现）
        </div>
      </section>

      {isLoading && (
        <div style={{ marginTop: '20px', color: '#666' }}>
          处理中...
        </div>
      )}
    </div>
  );
}
