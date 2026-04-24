# EVE Local Alert (Rust + Tauri)

Windows 桌面应用程序，监控 EVE Online 的"本地"聊天列表，并在检测到敌对（红色/声望）标记时发出警报。

这是现有 Python/PyQt5 应用程序的完整重写，使用 **Rust + Tauri v2**（React + TypeScript 前端）。

**平台：** Windows 10/11 仅限（无跨平台支持）
**技术栈：** Rust + Tauri v2 + React + TypeScript + OpenCV 4.x

## 项目概述

EVE Online 的"本地"频道显示同一恒星系中的所有飞行员。敌对飞行员在其名称旁边显示红色/橙色的声望标记。该工具捕获本地成员列表屏幕区域并检测这些彩色标记以警告用户。

- UI 语言默认为**中文**，从一开始就支持国际化架构
- DingTalk Webhook **超出范围** — 已替换为 Windows Toast 通知

## 快速开始

请参阅 [BUILD.md](BUILD.md) 获取详细的构建说明，包括必需的 OpenCV 设置。

> 当前 Phase 1 只建立了 OpenCV 的构建文档与环境变量约定；仓库尚未接入 `opencv` crate，也没有运行时 OpenCV 自检。

## 前置条件

- **Windows 10 或 11**（64位）
- Rust 工具链（MSVC）
- Node.js 20.19+（Vite 7 当前要求，建议使用最新 LTS）
- OpenCV 4.8+（参见 BUILD.md）
- Windows SDK

## 开发

```bash
# 安装前端依赖
npm install

# 开发模式（热重载）
cargo tauri dev

# 发布构建
cargo tauri build

# 运行测试
cargo test

# 运行单个测试
cargo test test_name

# 代码检查
cargo clippy
cargo fmt --check
```

## 架构

两种捕获模式运行独立的管道：

- **WGC 模式**：每个 EVE 客户端窗口都有自己的捕获 → 检测管道。支持多窗口。窗口被枚举，每个都有专用的捕获和检测。
- **MSS 模式**：更简单的桌面区域捕获。窗口必须保持可见。

每个 ROI 的检测管道：HSV 颜色匹配（可配置范围、min_pixels/min_ratio）+ 模板匹配（OpenCV matchTemplate，多模板、缩放搜索）。结果结合去抖动/冷却逻辑。

## 配置模型

- `MonitorConfig` — 全局设置
- `TargetConfig` — 每个 WGC 窗口
- `RoiConfig` — 每个 ROI 区域
- `ColorMatchConfig` — 每个颜色规则（HSV 范围）
- `TemplateMatchConfig` — 每个模板规则

配置作为 JSON 保存/加载，并在启动时冻结运行时（深度复制）。

## 项目文档

详细的文档位于 `.planning/` 目录：

- [PROJECT.md](.planning/PROJECT.md) - 完整的项目文档和架构
- [ROADMAP.md](.planning/ROADMAP.md) - 开发路线图
- [REQUIREMENTS.md](.planning/REQUIREMENTS.md) - 项目需求
- [BUILD.md](BUILD.md) - 详细的构建说明

## 开发进度

当前阶段：**Phase 1 - Foundation and Config Spine**

查看 [ROADMAP.md](.planning/ROADMAP.md) 了解完整的开发进度。

## 已知问题和注意事项

参考 [`.planning/research/PITFALLS.md`](.planning/research/PITFALLS.md) 了解已知的技术陷阱。

## 贡献

本项目正在积极开发中。欢迎反馈和问题报告！

## 许可证

见 [LICENSE](LICENSE) 文件。
