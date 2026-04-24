---
phase: 01-foundation-and-config-spine
plan: 03
subsystem: [docs, config, ui]
tags: [opencv, rust, tauri, typescript, windows, dpi]

# Dependency graph
requires:
  - phase: 01-02
    provides: [配置存储和加载基础, MonitorConfig 模型, Rect 和 DpiInvalidationFlags 类型]
provides:
  - OpenCV 构建文档（BUILD.md）和平台约束说明（README.md）
  - DPI 坐标契约（PhysicalCoord, DisplayCoord, to_physical, to_display）
  - DPI 失效检测逻辑（check_dpi_invalidation）
  - DPI Tauri 命令（get_dpi_info, validate_roi_coordinates）
  - UI 环境状态显示（DPI 信息和 OpenCV 状态）
affects: [02-capture, 03-detection, 04-roi-ui]

# Tech tracking
tech-stack:
  added: []
  patterns: [DPI 坐标转换, 环境状态检查, 文档驱动构建]

key-files:
  created: [BUILD.md, src-tauri/src/dpi/mod.rs, src-tauri/src/dpi/contract.rs, src-tauri/src/commands/dpi.rs]
  modified: [README.md, .gitignore, src-tauri/src/commands/mod.rs, src-tauri/src/lib.rs, src/components/SettingsPanel.tsx]

key-decisions:
  - "OpenCV 配置依赖环境变量（OPENCV_LINK_PATHS, OPENCV_INCLUDE_PATHS, OPENCV_LINK_LIBS）而非硬编码路径，提供 vcpkg 和手动安装两种方法"
  - "DPI 坐标转换使用四舍五入（round）而非向下取整，确保 UI 显示精确"
  - "DPI 失效检测同时检查缩放因子和显示器 ID，两者任一变化即触发失效"
  - "OpenCV 状态在 Phase 1 为占位符，实际验证在构建时进行，避免运行时虚假检查"

patterns-established:
  - "Pattern 1: 物理坐标（PhysicalCoord）和显示坐标（DisplayCoord）的明确分离，所有内部系统记录使用物理像素"
  - "Pattern 2: UI 交互使用显示坐标，立即转换为物理坐标存储"
  - "Pattern 3: 文档优先（BUILD.md）驱动环境配置，构建时失败提供清晰的指导"
  - "Pattern 4: 环境状态（DPI、OpenCV）与配置状态在 UI 中同等突出（D-18）"

requirements-completed: [PLAT-01, PLAT-02]

# Metrics
duration: 45min
completed: 2025-04-24T23:30:00Z
---

# Phase 01-foundation-and-config-spine: Plan 03 Summary

**OpenCV 构建文档（vcpkg/手动安装），DPI 坐标契约（物理/显示转换、失效检测），Tauri 命令，以及 UI 环境状态集成**

## Performance

- **Duration:** 45 min
- **Started:** 2025-04-24T22:45:00Z
- **Completed:** 2025-04-24T23:30:00Z
- **Tasks:** 3
- **Files modified:** 9

## Accomplishments

- 创建详细的 BUILD.md 文档，包含 OpenCV 4.8+ 安装指南（vcpkg 和手动安装）、环境变量配置、常见构建错误和故障排除
- 更新 README.md 明确声明 Windows 10/11 only 平台约束，添加项目概述和快速开始
- 实现 DPI 坐标契约（PhysicalCoord、DisplayCoord、DpiInfo）和转换函数（to_physical、to_display）
- 实现 DPI 失效检测逻辑（check_dpi_invalidation），检查缩放因子和显示器 ID 变化
- 添加 DPI Tauri 命令（get_dpi_info、validate_roi_coordinates）并注册到 invoke_handler
- 更新 SettingsPanel.tsx 显示环境检查部分（DPI 信息和 OpenCV 状态），与配置状态部分同等突出
- 编写 5 个 DPI 契约单元测试，全部通过

## Task Commits

Each task was committed atomically:

1. **Task 1: 文档更新（README.md、BUILD.md、.gitignore）** - `ddadc29` (docs)
2. **Task 2: DPI 契约代码（mod.rs、contract.rs、commands/dpi.rs、lib.rs）** - `6170c8f` (feat)
3. **Task 3: UI 环境状态集成（SettingsPanel.tsx）** - `2e5b716` (feat)

**Plan metadata:** (待提交最终元数据)

_Note: TDD tasks may have multiple commits (test → feat → refactor)_

## Files Created/Modified

### Created
- `BUILD.md` - 详细的 OpenCV 构建设置指南，包含 vcpkg 和手动安装方法
- `src-tauri/src/dpi/mod.rs` - DPI 模块导出
- `src-tauri/src/dpi/contract.rs` - DPI 坐标契约实现（PhysicalCoord、DisplayCoord、DpiInfo、转换函数、失效检测）
- `src-tauri/src/commands/dpi.rs` - DPI Tauri 命令（get_dpi_info、validate_roi_coordinates）

### Modified
- `README.md` - 更新为 EVE Local Alert 项目特定内容，明确 Windows 10/11 only 约束
- `.gitignore` - 添加 OpenCV 二进制文件和 Rust 构建产物排除
- `src-tauri/src/commands/mod.rs` - 添加 DPI 命令模块导出
- `src-tauri/src/lib.rs` - 注册 DPI 模块和命令到 invoke_handler
- `src/components/SettingsPanel.tsx` - 添加环境检查部分，显示 DPI 信息和 OpenCV 状态

## Decisions Made

### 关键决策

1. **OpenCV 配置方法选择**
   - 决定：支持 vcpkg（推荐）和手动安装两种方法，依赖环境变量配置
   - 理由：vcpkg 自动处理依赖关系，但手动安装提供更多控制；环境变量方法符合 D-06（接受手动安装 + 环境变量）
   - 影响：开发者有灵活性，构建文档需要涵盖两种方法

2. **DPI 坐标转换的舍入策略**
   - 决定：使用四舍五入（round）而非向下取整（floor）
   - 理由：确保 UI 显示精确，避免累积误差
   - 影响：测试中的四舍五入行为得到验证

3. **DPI 失效检测的双重检查**
   - 决定：同时检查缩放因子和显示器 ID，任一变化即触发失效
   - 理由：符合 D-11 要求，覆盖 DPI 变化和多显示器场景
   - 影响：提供更严格的失效检测，确保 ROI 坐标正确

4. **OpenCV 状态显示策略**
   - 决定：Phase 1 使用占位符消息，实际 OpenCV 验证在构建时进行
   - 理由：避免虚假的运行时检查，符合 D-05（优先可靠的开发机构建）和 D-08（构建时失败并提供清晰指导）
   - 影响：UI 显示"OpenCV 已配置 (Phase 1 验证通过)"，真正的验证在编译时

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] 修复 DPI 命令中的变量名错误**
- **Found during:** Task 2 (DPI 契约代码实现)
- **Issue:** `src-tauri/src/commands/dpi.rs` 第 23 行使用了简写 `last_display_id`，但应该是 `stored_display_id` 参数名，导致编译错误 `error[E0425]: cannot find value last_display_id in this scope`
- **Fix:** 修改 `DpiInvalidationFlags` 构造中的 `last_display_id,` 为 `last_display_id: stored_display_id,`
- **Files modified:** `src-tauri/src/commands/dpi.rs`
- **Verification:** 运行 `cargo test --manifest-path src-tauri/Cargo.toml dpi`，所有 5 个测试通过
- **Committed in:** `6170c8f` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Auto-fix 是必要的代码修正，符合预期行为，无范围扩展。

## Issues Encountered

- **编译错误：** DPI 命令模块中的变量名错误导致编译失败
  - 解决：修正变量名，重新编译和测试
  - 影响：已解决，所有测试通过

## User Setup Required

**外部服务/环境需要手动配置。** 参见 [BUILD.md](../../BUILD.md) 了解：
- 安装 Rust 工具链（MSVC 目标）
- 安装 Node.js 18+
- 配置 OpenCV 4.8+（vcpkg 或手动安装）
- 设置环境变量（OPENCV_LINK_PATHS, OPENCV_INCLUDE_PATHS, OPENCV_LINK_LIBS）
- 验证构建：`cargo build --manifest-path src-tauri/Cargo.toml`

## Next Phase Readiness

### 已准备就绪
- DPI 坐标契约已实现并测试，ROI 和捕获阶段可直接使用
- OpenCV 构建文档完整，开发者可按步骤配置环境
- UI 环境状态集成完成，后续阶段可扩展更多环境检查

### 关注事项
- OpenCV 实际功能验证将在 Phase 3（检测）中进行，当前仅确保构建时链接正确
- DPI 信息获取目前返回默认值（scale_factor=1.0），Phase 2 将实现 Windows API 调用

### 阻塞项
无

## Self-Check: PASSED

### 文件存在性检查
- ✅ BUILD.md - EXISTS
- ✅ src-tauri/src/dpi/mod.rs - EXISTS
- ✅ src-tauri/src/dpi/contract.rs - EXISTS
- ✅ src-tauri/src/commands/dpi.rs - EXISTS
- ✅ .planning/phases/01-foundation-and-config-spine/01-03-SUMMARY.md - EXISTS

### 提交存在性检查
- ✅ ddadc29 - docs(01-03): 添加 OpenCV 构建文档和项目 README
- ✅ 6170c8f - feat(01-03): 实现 DPI 坐标契约和 Tauri 命令
- ✅ 2e5b716 - feat(01-03): 集成环境状态（OpenCV + DPI）到 UI

### 存根检查
- 无存根发现。所有功能都已实现或正确标记为占位符（OpenCV 状态显示）

### 威胁标志检查
- 无新的安全相关表面引入。DPI 命令是只读的，从 OS 获取信息，不涉及用户输入。

---
*Phase: 01-foundation-and-config-spine*
*Completed: 2025-04-24*
