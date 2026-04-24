# 构建说明

本文档提供在 Windows 上构建 EVE Local Alert 的详细说明，包括必需的 OpenCV 设置。

## 前置条件

### 1. 操作系统
- **Windows 10 或 11**（64位）
- Windows SDK 已安装（通常随 Visual Studio 安装）

### 2. Rust 工具链
```bash
# 安装 Rust（如果尚未安装）
# 从 https://rustup.rs/ 下载安装程序

# 设置为 MSVC 目标（Windows 开发标准）
rustup default stable-x86_64-pc-windows-msvc

# 验证安装
rustc --version
cargo --version
```

### 3. Node.js
- **Node.js 20.19+**（当前 Vite 7 要求；建议使用最新 LTS）
- 从 https://nodejs.org/ 下载并安装 LTS 版本

```bash
# 验证安装
node --version
npm --version
```

### 4. OpenCV 4.8+（必需）
本项目的图像检测功能后续会依赖 OpenCV。**Phase 1 先固定构建约定与环境变量；当前仓库尚未接入 `opencv` crate。**

## OpenCV 设置

### 方法 1: vcpkg（推荐）

vcpkg 是微软的 C++ 包管理器，可以自动处理 OpenCV 的依赖关系。

#### 步骤 1: 安装 vcpkg

```bash
# 克隆 vcpkg 仓库
git clone https://github.com/Microsoft/vcpkg.git C:\vcpkg
cd C:\vcpkg

# 运行 bootstrap 脚本
.\bootstrap-vcpkg.bat

# 集成到系统
.\vcpkg integrate install
```

#### 步骤 2: 安装 OpenCV

```bash
# 安装 OpenCV（64位静态链接库）
.\vcpkg install opencv:x64-windows-static

# 安装过程可能需要 10-20 分钟，取决于网络速度
```

#### 步骤 3: 配置环境变量

在 Windows 系统环境变量中添加以下变量：

**系统环境变量设置：**
1. 右键"此电脑" → 属性 → 高级系统设置 → 环境变量
2. 在"系统变量"部分点击"新建"

添加以下变量：

| 变量名 | 变量值 |
|--------|--------|
| `OPENCV_LINK_PATHS` | `C:\vcpkg\installed\x64-windows-static\lib` |
| `OPENCV_INCLUDE_PATHS` | `C:\vcpkg\installed\x64-windows-static\include` |
| `OPENCV_LINK_LIBS` | `opencv_world480`（或相应版本号） |

**注意：** 如果使用不同版本的 OpenCV，请相应调整 `OPENCV_LINK_LIBS`（例如 OpenCV 4.10.0 使用 `opencv_world4100`）。

#### 步骤 4: 添加 DLL 到 PATH（运行时需要）

将 OpenCV DLL 添加到系统 PATH：

| 变量 | 值 |
|------|-----|
| `Path`（编辑） | 添加 `C:\vcpkg\installed\x64-windows-static\bin` |

**重要：** 设置环境变量后，**必须重启**命令提示符或 IDE 才能生效。

---

### 方法 2: 手动安装

如果 vcpkg 安装失败，可以手动下载预编译的 OpenCV 二进制文件。

#### 步骤 1: 下载 OpenCV

1. 访问 https://opencv.org/releases/
2. 下载 **OpenCV 4.8.0 Windows**（或更高版本）
3. 将下载的文件提取到 `C:\opencv`

#### 步骤 2: 配置环境变量

| 变量名 | 变量值 |
|--------|--------|
| `OPENCV_LINK_PATHS` | `C:\opencv\build\x64\vc16\lib` |
| `OPENCV_INCLUDE_PATHS` | `C:\opencv\build\include` |
| `OPENCV_LINK_LIBS` | `opencv_world480` |

#### 步骤 3: 添加 DLL 到 PATH

编辑 `Path` 环境变量，添加：`C:\opencv\build\x64\vc16\bin`

**重要：** 重启命令提示符或 IDE。

---

## 验证 OpenCV 设置

### 步骤 1: 验证环境变量

```bash
# 在新的命令提示符窗口中检查
echo %OPENCV_LINK_PATHS%
echo %OPENCV_INCLUDE_PATHS%
echo %OPENCV_LINK_LIBS%
```

应该显示你设置的路径和库名称。

### 步骤 2: 验证环境准备

```bash
# 在项目根目录
cargo build --manifest-path src-tauri/Cargo.toml
```

**预期结果：**
- ✅ 当前 Phase 1 应至少能完成普通 Rust/Tauri 构建，并保留后续接入 OpenCV 所需的文档约定
- ⚠ 当后续阶段引入 `opencv` crate 后，如果看到链接器错误（如 `cannot find -lopencv_core`），请检查：
  - 环境变量是否正确设置
  - OpenCV 架构是否与 Rust 目标匹配（必须都是 x64）
  - 是否重启了命令提示符
  - OpenCV 版本是否为 4.8+

### 常见构建错误

| 错误 | 原因 | 解决方法 |
|------|------|----------|
| `cannot find -lopencv_core` | 环境变量未设置或错误 | 检查 `OPENCV_LINK_PATHS` 和 `OPENCV_LINK_LIBS` |
| `unresolved external symbol` | OpenCV 版本/架构不匹配 | 确保 x64 架构，检查 OpenCV 版本 |
| `LNK1112: 模块计算机类型 'x64' 与目标计算机类型 'X86' 冲突` | 架构不匹配 | 使用 `rustup default stable-x86_64-pc-windows-msvc` |
| `无法打开包括文件: 'opencv2/...''` | 包含路径错误 | 检查 `OPENCV_INCLUDE_PATHS` |
| `DPI 缩放问题` | 没有正确处理 DPI | 代码中已实现 DPI 坐标转换（见 `src-tauri/src/dpi/contract.rs`） |

> 注意：当前 `src-tauri/src/dpi/contract.rs` 中的 `get_current_dpi()` 仍返回 Phase 1 基线占位值（`default`, `1.0`），真实的 Windows API DPI 读取将在后续阶段补上。

## 构建项目

### 安装前端依赖

```bash
# 在项目根目录
npm install
```

### 开发模式（热重载）

```bash
cargo tauri dev
```

这将：
1. 编译 Rust 后端
2. 启动前端开发服务器
3. 打开 Tauri 应用窗口
4. 启用热重载（代码更改后自动重新加载）

### 发布构建

```bash
cargo tauri build
```

构建产物位于：
- Windows 安装程序：`src-tauri\target\release\bundle\msi\*.msi`
- 独立可执行文件：`src-tauri\target\release\eve_local_alarm.exe`

## 运行测试

```bash
# 运行所有测试
cargo test --manifest-path src-tauri/Cargo.toml

# 运行特定测试
cargo test --manifest-path src-tauri/Cargo.toml dpi
```

## 故障排除

### OpenCV 相关问题

**问题：** `vcpkg install opencv:x64-windows-static` 失败或卡住

**解决方案：**
1. 确保网络连接正常
2. 尝试使用代理或镜像（参考 vcpkg 文档）
3. 如果持续失败，使用方法 2（手动安装）

**问题：** 构建成功但运行时提示缺少 DLL

**解决方案：**
- 确保将 OpenCV `bin` 目录添加到系统 `Path`
- 重启应用或 IDE

**问题：** OpenCV 版本检测失败

**解决方案：**
- OpenCV 4.8.0+ 是稳定版本，推荐使用
- 如果使用其他版本，请相应调整 `OPENCV_LINK_LIBS`：
  - OpenCV 4.10.0 → `opencv_world4100`
  - OpenCV 4.8.0 → `opencv_world480`
  - OpenCV 4.5.0 → `opencv_world450`

### 其他问题

**问题：** `cargo tauri dev` 命令不存在

**解决方案：**
```bash
# 安装 Tauri CLI
cargo install tauri-cli --version "^2.0.0"
```

**问题：** Node.js 依赖安装失败

**解决方案：**
```bash
# 清除缓存
npm cache clean --force

# 删除 node_modules 并重新安装
rmdir /s /q node_modules
npm install
```

## 已知问题和注意事项

参考 [`.planning/research/PITFALLS.md`](.planning/research/PITFALLS.md) 了解已知的技术陷阱和解决方案。

主要已知问题：
- **PITFALLS.md #1**: DPI 缩放问题（已在 Phase 1 解决）
- **PITFALLS.md #2**: OpenCV Rust 绑定构建复杂性（本文档解决）

## 下一步

构建成功后，请参考：
- [README.md](README.md) - 项目概述和快速开始
- [`.planning/PROJECT.md`](.planning/PROJECT.md) - 详细的项目文档和架构
- [`.planning/ROADMAP.md`](.planning/ROADMAP.md) - 开发路线图

## 版本信息

- Rust 工具链：stable-x86_64-pc-windows-msvc
- OpenCV：4.8.0 或更高版本（推荐，用于后续接入 `opencv` crate）
- Node.js：20.19+ LTS
- Tauri：2.x
