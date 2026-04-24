# Phase 1: Foundation and Config Spine - Discussion Log

**Gathered:** 2026-04-24
**Status:** Completed

## Scope Discussed

- Configuration lifecycle and startup behavior
- OpenCV / native dependency strategy
- DPI and ROI coordinate contract
- Phase 1 delivery shape and UI emphasis

## Question and Answer Record

### 1. Configuration startup behavior

**Question:** 应用启动时，配置怎么加载最合适？

**Options presented:**
1. 自动加载上次配置，但不自动开始监控
2. 自动加载上次配置，并自动恢复上次监控状态
3. 每次启动都进入空白/向导状态
4. 其他

**User selection:** 1

**Captured decision:** 启动时自动加载上次配置，但不自动开始监控。

### 2. Configuration file model

**Question:** 配置文件本身，你更希望怎么管理？

**Options presented:**
1. 固定单配置文件
2. 支持多个命名配置档
3. 单主配置 + 导入/导出
4. 其他

**User selection:** 1

**Captured decision:** v1.0 使用固定单配置文件。

### 3. Config mutation while monitoring

**Question:** 当监控正在运行时，如果用户修改配置，应该怎么处理？

**Options presented:**
1. 禁止热更新，必须停止监控后再生效
2. 允许修改，但提示“下次启动监控时生效”
3. 允许部分配置热更新
4. 其他

**User selection:** 1

**Captured decision:** 监控运行中禁止配置热更新，必须停止监控后才生效。

### 4. Broken config recovery

**Question:** 如果主配置文件损坏、缺失或解析失败，应用应该怎么做？

**Options presented:**
1. 自动回退到默认配置，并明确提示用户
2. 阻止进入主界面，要求用户先修复配置
3. 保留当前界面为空白状态，让用户手动重新配置
4. 其他

**User selection:** 1

**Captured decision:** 配置损坏/缺失时自动回退默认配置，并明确提示用户。

### 5. OpenCV goal for Phase 1

**Question:** Phase 1 对 OpenCV 的目标，你更偏向哪种？

**Options presented:**
1. 先确保开发机可稳定构建，文档写清楚安装步骤
2. 从一开始就要求尽量自动化安装/配置依赖
3. 先做可替换接口，OpenCV 先不真正接入
4. 其他

**User selection:** 1

**Captured decision:** Phase 1 优先保证开发机可稳定构建，并写清楚安装步骤。

### 6. OpenCV install style

**Question:** 开发环境的依赖方式，你更倾向哪种？

**Options presented:**
1. 先接受手动安装 OpenCV + 环境变量配置
2. 优先走包管理器方案（如 vcpkg）
3. 两者都支持，但文档主推其中一种
4. 其他

**User selection:** 1

**Captured decision:** 接受手动安装 OpenCV + 环境变量配置。

### 7. OpenCV version policy

**Question:** OpenCV 版本策略，你更希望怎么定？

**Options presented:**
1. 明确锁定一个推荐版本范围，并按它写文档
2. 尽量兼容多个 OpenCV 4.x 版本
3. Phase 1 只要求“能装上就行”
4. 其他

**User selection:** 1

**Captured decision:** 锁定明确的推荐 OpenCV 版本范围，并按它写文档。

### 8. Failure mode for missing OpenCV

**Question:** 如果开发机没有装好 OpenCV，应用/项目层面你希望怎么表现？

**Options presented:**
1. 在构建阶段就明确失败，并给出清晰指引
2. 尽量让项目能先编过去，运行到检测功能时再报错
3. 允许无 OpenCV 模式启动，只是禁用检测相关功能
4. 其他

**User selection:** 1

**Captured decision:** OpenCV 环境未就绪时，在构建阶段直接失败并给出清晰指引。

### 9. Internal coordinate system

**Question:** 内部统一坐标系，你更希望选哪种？

**Options presented:**
1. 内部一律使用物理像素坐标，UI 负责显示换算
2. 内部一律使用用户看到的显示坐标
3. 两套都保留
4. 其他

**User selection:** 1

**Captured decision:** 内部统一使用物理像素坐标，UI 负责显示换算。

### 10. ROI drag semantics

**Question:** ROI 选择器里，用户拖出来的框，应该怎么理解？

**Options presented:**
1. 用户按屏幕上看到的位置拖拽，系统立刻换算成内部物理像素
2. 用户直接操作物理像素意义上的区域
3. 界面里同时显示两套坐标
4. 其他

**User selection:** 1

**Captured decision:** 用户按显示坐标拖拽，系统立即换算成内部物理像素坐标。

### 11. Behavior after DPI/display changes

**Question:** 如果用户换了显示缩放、分辨率，或者把窗口拖到另一块 DPI 不同的显示器上，v1.0 你希望怎么处理？

**Options presented:**
1. 把当前 ROI 标记为可能失效，要求用户重新确认/调整
2. 尝试自动重新映射 ROI
3. 继续照旧运行，不做特别处理
4. 其他

**User selection:** 1

**Captured decision:** DPI / 分辨率 / 显示器变化后，将 ROI 标记为可能失效。

### 12. Monitoring behavior with DPI risk

**Question:** 当 ROI 因 DPI 变化被判定为可能失效时，你更希望产品怎么提示？

**Options presented:**
1. 阻止直接开始监控，先弹出明确提示让用户确认 ROI
2. 允许开始监控，但显示明显警告
3. 只在设置页标记异常，不主动拦截
4. 其他

**User selection:** 2

**Captured decision:** 允许继续开始监控，但必须显示明显警告。

### 13. Warning presentation style

**Question:** ROI 被标记为“可能因 DPI 变化失效”时，警告最合适怎么呈现？

**Options presented:**
1. 开始监控时弹一次明确警告，同时在界面里持续标黄/标红
2. 只做持续警告，不弹窗
3. 每次开始监控都弹窗提醒
4. 其他

**User selection:** 1

**Captured decision:** 开始监控时弹一次明确警告，同时界面里持续标黄/标红。

### 14. Warning granularity

**Question:** 这个 DPI 风险标记，你希望是按什么粒度存在？

**Options presented:**
1. 按 ROI 单独标记
2. 按整个配置全局标记
3. 按监控目标（窗口/模式）标记
4. 其他

**User selection:** 1

**Captured decision:** DPI 风险按 ROI 单独标记。

### 15. Phase 1 milestone output

**Question:** Phase 1 结束时，你最希望看到哪种结果？

**Options presented:**
1. 项目骨架 + 配置系统 + 环境依赖打通，但先不做完整 UI
2. 项目骨架 + 配置系统 + 一个可操作的基础中文设置界面
3. 在 2 的基础上，再带一个很轻量的监控占位流程
4. 其他

**User selection:** 2

**Captured decision:** Phase 1 结束时需要项目骨架、配置系统，以及一个可操作的基础中文设置界面。

### 16. Scope of the Phase 1 UI

**Question:** 这个基础中文设置界面，Phase 1 里你希望它做到什么程度？

**Options presented:**
1. 只覆盖 Phase 1 相关内容
2. 提前把后续主要配置页框架也搭出来
3. 做成一个单页设置面板
4. 其他

**User selection:** 1

**Captured decision:** Phase 1 基础中文设置界面只覆盖 Phase 1 相关内容。

### 17. UI style direction

**Question:** 你希望这个 Phase 1 界面更偏哪种风格？

**Options presented:**
1. 朴素实用型
2. 偏桌面工具型
3. 偏现代面板型
4. 其他

**User selection:** 1

**Captured decision:** Phase 1 界面风格走朴素实用型。

### 18. Homepage emphasis

**Question:** Phase 1 完成时，你希望用户一打开应用，首页最重要的是什么？

**Options presented:**
1. 当前配置状态 + 保存/加载操作
2. 环境/依赖检查状态
3. 两者并列同等重要
4. 其他

**User selection:** 3

**Captured decision:** 首页同时突出当前配置状态/保存加载与环境/依赖检查状态。

## Deferred Ideas

None.

---

*Phase: 01-foundation-and-config-spine*
*Discussion logged: 2026-04-24*
