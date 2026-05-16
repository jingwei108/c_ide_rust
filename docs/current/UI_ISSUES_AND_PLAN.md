# C IDE Android 端 UI 问题分析与优化计划

## 更新记录

- **2026-04-28** 初始版本：Phase 1~4 规划
- **2026-04-28** 更新：完成响应式系统铺设、工具栏优化、底部标签栏修复、悬浮球重做与拖拽吸附
- **2026-05-08** 更新：MAUI 端悬浮球扇形质感增强 + 工具栏UI重制（参考图一）

---

## 一、已完成功能

### ✅ 响应式系统铺设
- 以 400px 逻辑宽度为基准，`UpdateResponsiveMetrics()` 统一生成 30+ 个 `DynamicResource`
- XAML 中零硬编码尺寸，全部绑定 `DynamicResource`
- 覆盖：工具栏、编辑器、行号、底部面板、模板栏、卡片、调试面板、模态框、悬浮球

### ✅ CornerRadius 类型安全修复
- `system:Double` → `CornerRadius` 原生类型，解决 Android 闪退

### ✅ 工具栏优化
- 顶部安全区防状态栏遮挡（`MobileToolbarPadding` 顶部 28px）
- 运行/停止按钮合一（`ToggleRunStopCommand`）
- Slider 放回工具栏
- 动态边距与字体（`MobileBtnFontSize`、`MobileStepBtnFontSize` 等）

### ✅ 底部标签栏修复
- 移除自定义 `ControlTemplate`，恢复 Tab 切换功能
- 标签去 Emoji，改为纯文字（"输出" / "诊断" / "算法"）
- 均分屏幕宽度 `width / 3.0`

### ✅ 悬浮球重做（Phase 2 完成）
- **遮挡修复**：覆盖区从 160×360 缩小到 64×64；`Grid` 去 `Margin` 紧贴屏幕
- **样式美化**：微渐变（`#0A84FF` → `#005BB5`）+ 阴影 + 圆角，尺寸 56×56
- **扇形菜单**：7 个按钮改为中文单字图标（栈/表/变/存/组/针/知）+ 毛玻璃质感（`#CC2D2D30`）
- **自由拖动**：`PointerPressed` → `PointerMoved` → `PointerReleased`，8px 阈值区分点击与拖拽
- **贴边吸附**：释放后自动吸附到左/右边缘，保留约 1/4 圆在屏内；Y 限制在安全区（顶部 48px ~ 底部 8px）
- **初始位置**：从右下角贴底改为右侧中间偏下（`h * 0.65`）
- **事件穿透修复**：去掉覆盖层 Grid 的 `Background="Transparent"`；给 `FabButtonBorder` 补上默认渐变背景，确保命中测试正常

### ✅ MAUI 端悬浮球质感增强（2026-05-08）
- **扇形背景光晕**：展开时从球心扩散 240px 半透明蓝色径向渐变圆形背景
- **扇形弧线装饰**：新增 200px 半透明弧线，增强扇形轮廓感
- **Stagger 弹出动画**：8 个菜单项逐个延迟 25ms 依次弹出，弹性贝塞尔曲线 `cubic-bezier(0.34, 1.56, 0.64, 1)`
- **3D 球体质感**：
  - 多层阴影（外发光 `#0A84FF` + 深投影）
  - `::before` 顶部高光伪元素（模拟球面光泽）
  - `::after` 底部内阴影（增强凹陷感）
  - 按压 `scale(0.94)` 反馈，拖拽时 `scale(1.12)`
- **菜单项质感**：增强毛玻璃 `blur(12px) saturate(1.2)` + 多层精致阴影 + 顶部微高光 + 按压反馈

### ✅ MAUI 端工具栏重制（2026-05-08，参考图一）
- **播放按钮**：蓝色 `#0A84FF`，圆角矩形，精简为单图标 `▶`
- **停止按钮**：红色 `#FF453A`，仅在运行时显示，单图标 `■`
- **下一步按钮**：深灰 `#3A3A3C` 背景，显示 `⏭ 下一步`
- **新增滑块**：灰色轨道 + 蓝色圆点，绑定执行速度 `ExecutionSpeed`
- **状态框**：深色背景 `#1E1E1E`，青色文字 `#9CDCFE`，显示 `StepStatusText` 和运行指示灯
- **主题按钮**：改为文字"亮"/"暗"，带细边框的圆角样式
- **移除底部独立状态栏**：状态信息整合进工具栏状态框，消除重复显示

### ✅ MAUI 端模板快捷栏（2026-05-08，参考图一）
- **位置**：代码编辑器与底部面板之间，横向滚动
- **数据源**：绑定 `VM.Templates`（冒泡排序、二分查找、链表节点、快速排序、递归阶乘、斐波那契、数组遍历、指针交换）
- **样式**：深灰 `#3A3A3C` 圆角矩形标签按钮，白色/浅色文字，按压时变主题蓝 `#0A84FF`
- **交互**：点击后在当前光标位置插入模板代码（通过 CodeMirror 6 `dispatch` 实现）
- **实现**：JS interop 新增 `insertTemplate(id, text)`，支持在光标处插入文本不覆盖现有代码

### ✅ MAUI 端面板与模态框质感增强（2026-05-08）
- **底部面板高度压缩**：桌面端 200px → 移动端 140px，给编辑器释放更多垂直空间
- **诊断卡片优化**：增加圆角（8px）、微阴影、半透明背景层次，错误/警告/提示三色边框
- **算法卡片优化**：增加圆角（8px）、微阴影，与诊断卡片视觉统一
- **空状态美化**：增大 padding 和字体，降低文字亮度，视觉更柔和
- **模态面板增强**：
  - 遮罩模糊从 4px 提升到 8px，背景更深
  - 内容区顶部圆角从 16px 提升到 20px，增加顶部拖动手柄指示条
  - 入场动画时间从 0.25s 优化到 0.3s
  - 添加下滑关闭手势（下滑超过 100px 自动关闭 Modal）
  - 关闭按钮增加按压反馈

---

## 二、当前问题梳理

### 1. 功能缺失/异常

| 问题 | 现象 | 根因分析 |
|------|------|----------|
| **行点击无虚拟键盘** | 点击代码编辑区软键盘不弹出 | AvaloniaEdit 11.1.0 在 Android 上未实现 `ITextInputMethodClient`，平台不会自动唤起输入法。需升级库或添加平台代码手动控制。 |
| **光标可见性待验证** | 编辑区看不到闪烁光标 | Caret 颜色已设为亮白色 `#FFFFFF`，待实机验证是否可见。 |

### 2. UI 美观与布局问题

| 问题 | 详细描述 | 优化方向 |
|------|----------|----------|
| **行号未贴近边缘** | 行号区左右 `Margin` 过大（`Margin="2,0,2,0"` + `Padding="0,2,8,2"`），浪费空间 | 精简行号区宽度，去除多余内边距，使数字贴近左边框 |
| **横屏布局不合理** | 横屏时底部面板（输出/诊断/算法）高度固定 140，占用过多；工具栏按钮拥挤 | 横屏时启用响应式布局：底部面板改为侧栏或可调高度；工具栏支持滚动或折叠 |
| **整体配色单调** | 大面积深灰 `#1E1E1E`、`#252526` 缺乏层次，按钮无渐变/圆角统一规范 | 引入主题色（主色 `#0A84FF`、成功 `#30D158`、警告 `#FF9F0A`、错误 `#FF453A`），统一圆角、间距、阴影规范 |

---

## 三、剩余优化计划

### Phase 3: 编辑器体验修复
- [ ] **虚拟键盘**：AvaloniaEdit 11.1.0 在 Android 上未实现 `ITextInputMethodClient`，需后续专项处理（方案：升级库或添加 Android 平台代码）
- [x] **光标可见**：Caret 颜色已设为亮白色 `#FFFFFF`，待实机验证
- [x] **模板交互**：已修复（见下方详情）

**模板交互修复详情：**
- **根因**：`Templates` 集合在 `MainViewModel` 中始终为空，没有默认模板数据；同时移动端 `ScrollViewer` 在 Android 上会拦截 `Button` 的 `PointerPressed` 事件，导致 `Click` 无法触发。
- **数据填充**：在 `MainViewModel` 构造函数中初始化 8 个常用 C 语言模板（冒泡排序、二分查找、链表节点、快速排序、递归阶乘、斐波那契、数组遍历、指针交换）。
- **事件修复**：给模板 `Button` 增加 `PointerPressed` 事件处理，直接调用 `InsertTemplate` 并设置 `e.Handled = true`，阻止 `ScrollViewer` 继续捕获指针。
- **视觉反馈**：给模板 `Button` 补充 `pointerover`（深灰 `#3E3E42`）和 `pressed`（主题蓝 `#0A84FF` + 白字）状态样式。

### Phase 4: 布局与视觉优化
- [x] **行号区域精简**：`LineNumberItemMargin` 右侧改为 0，`LineNumberTextPadding` 右侧从 3-10px 减至 1-3px，数字更贴近右边缘
- [x] **横屏适配**：`BottomPanelHeight` 已在 `UpdateResponsiveMetrics` 中按横竖屏动态计算（竖屏 140*scale / 横屏 100*scale）
- [x] **整体视觉统一**：诊断卡片加阴影+半透明边框；工具栏/修复按钮统一 Hover/Pressed 反馈色
- [x] **平板横屏侧栏布局**：当 `!IsPortrait && IsTablet` 时，底部 TabControl 隐藏，在编辑器右侧显示 280px 侧栏面板（输出/诊断/算法），释放垂直空间

**视觉统一详情：**
- **诊断卡片**：添加 `BoxShadow="0 1 4 0 #20000000"` 和 `BorderBrush="#15FFFFFF"`，增加层次感和边界感
- **应用修复按钮**：主题蓝 `#0A84FF` → Hover `#1E90FF` → Pressed `#005BB5`
- **移动端运行按钮**：蓝色 `#0A84FF` → Hover `#1E90FF` → Pressed `#005BB5`
- **移动端停止按钮**：红色 `#FF453A` → Hover `#FF6B6B` → Pressed `#CC352D`
- **移动端下一步按钮**：深灰 `#3A3A3C` → Hover `#4A4A4C` → Pressed `#2A2A2C`

---

## 四、当前截图关键测量数据

- 状态栏高度：约 24–32 dp（系统区域）
- 工具栏顶部 `Padding`：`28`（已有安全区）
- 行号区宽度：`48`
- 行号内部右 `Padding`：`8`
- 底部面板高度：`140`
- 悬浮球尺寸：`56×56`
- 悬浮球初始位置：右侧中间偏下（`h * 0.65`）
- 悬浮球收起态：保留约 1/4 圆在屏幕内（`fabX = w + halfFab / 2`）

---

---

## 五、主题切换（Dark / Light）

### 已实现
- **主题资源字典**：`App.axaml` 中定义 `Dark` / `Light` 两套 `ThemeDictionaries`，包含：
  - `AppBackgroundBrush`、`AppPanelBrush`、`AppCardBrush`、`AppPopupBrush`
  - `AppBorderBrush`、`AppTextBrush`、`AppTextSecondaryBrush`、`AppTextMutedBrush`
  - `AppButtonBrush`、`AppButtonHoverBrush`、`AppButtonSecondaryBrush`
  - `AppFanBrush`、`AppFanBorderBrush`、`AppFanTextBrush`
  - `AppAlgorithmBgBrush`、`AppWrongCodeBgBrush`、`AppCorrectCodeBgBrush`
  - `AppErrorBgColor`、`AppWarningBgColor`、`AppHintBgColor`
- **XAML 颜色替换**：`MainView.axaml`、`CodeEditor.axaml` 中所有结构色（背景/面板/卡片/边框/文本）已改为 `DynamicResource`
- **诊断卡片主题感知**：`SeverityToBrushConverter` 根据 `ActualThemeVariant` 自动返回深色或浅色背景
- **TextMate 语法主题**：`CodeEditor` 构造时根据当前主题选择 `DarkPlus` 或 `LightPlus`
- **主题切换按钮**：移动端工具栏新增 "亮"/"暗" 按钮，绑定 `ToggleThemeCommand`
- **默认主题**：深色（与项目原有风格一致）

### 已知限制
- **TextMate 动态切换**：AvaloniaEdit 的 TextMate 主题在运行时无法热切换，切换主题后编辑器语法高亮需重启应用才能完全生效（面板/按钮/卡片等 UI 元素会即时切换）

---

*文档更新时间: 2026-04-28*
*下一步: 用户验收主题切换效果，或继续其他功能*
