# 移动端悬浮球UI优化全记录

## 问题背景
- **目标**：将悬浮球从简单的发光圆球升级为"呼吸水泡"质感
- **设备**：OPPO PKT110 (Android, Vulkan/Impeller) + Windows Desktop
- **渲染**：Flutter CustomPainter + `MaskFilter.blur` + 径向渐变

---

## 第一阶段：修复不显示问题

### 问题：移动端悬浮球完全不可见
- **根因**：`initState` 的 `addPostFrameCallback` 中 `MediaQuery.of(context).size` 返回 `Size.zero`，`_pos` 被算成 `(-72, 0)` 定位到屏幕外
- **修复**：在 `didChangeDependencies` 中初始化位置（context 已挂载），`build` 中兜底保护

```dart
@override
void didChangeDependencies() {
  super.didChangeDependencies();
  if (!_initialized) {
    final size = MediaQuery.of(context).size;
    _pos = Offset(size.width - _orbSize - 16, size.height * 0.62);
    _initialized = true;
  }
}

@override
Widget build(BuildContext context) {
  final size = MediaQuery.of(context).size;
  if (!_initialized || _pos.dx < 0 || _pos.dy < 0) {
    _pos = Offset(size.width - _orbSize - 16, size.height * 0.62);
    _initialized = true;
  }
  // ...
}
```

---

## 第二阶段：水泡效果多轮迭代

### Round 1：硬边描边（失败）
使用 `SweepGradient` + `PaintingStyle.stroke` 绘制彩色圆环，结果：
- 圆环像贴图，跟球体衔接突兀
- 用户反馈："更丑了，环不要跟悬浮球衔接很突兀"

### Round 2：弥散光晕替代描边（方向正确）
- 删除所有 `stroke` 描边
- 用 `RadialGradient` 弥散发光代替圆环
- 边缘发光从球体内部自然透出
- 用户反馈："这个方向可以"

### Round 3：整体提亮（接近参考图）
- 中心 `alpha` 从 0.30 → 0.50 → 0.65
- 深色区域全部变浅
- 高光 `alpha` 0.55 → 0.70
- 外光晕 `blur` 加大
- 用户反馈：很好，但"球里面的光晕有流动感就更好了"

### Round 4：液体流动感（最终版）

#### 1. 拖尾效果
主光斑后面跟一个更淡、更大、更弥散的拖尾：
```dart
// 主光斑
_drawFlowOval(..., alpha: 0.58, blur: 24);
// 拖尾（相位差 -0.4）
_drawFlowOval(..., alpha: 0.22, blur: 30, baseW: 0.75, baseH: 0.18);
```

#### 2. 光斑变形（长短轴交替伸缩）
```dart
final deform = math.sin(angle * deformSpeed) * deformAmount;
final w = r * baseW * (1.0 + deform);
final h = r * baseH * (1.0 - deform);
```

#### 3. 轨迹摆动（非纯圆周）
```dart
final wobble = math.sin(angle * wobbleFreq) * wobbleAmp;
final d = dist + wobble; // 半径轻微变化
```

#### 4. 微小气泡（4个快速闪烁点）
```dart
_drawFlowCircle(..., radius: 0.045, wobbleAmp: 0.12, wobbleFreq: 4.0);
```

#### 5. 中心亮芯脉动
```dart
final corePulse = 0.90 + math.sin(t * 2.5) * 0.10;
canvas.drawCircle(center, r * 0.18 * corePulse, corePaint);
```

---

## 第三阶段：手势重构（GestureDetector → Listener）

### 问题：菜单展开状态下拖动悬浮球只能移动极小距离
- **根因**：`GestureDetector` 参与 Flutter 手势竞技场，悬浮球在 `Overlay` 中，下方编辑器/ScrollView 的 `GestureDetector` 竞争获胜，导致 pan 手势被中断
- **表现**：移动端几乎无法发现球体在移动，PC 端也只有 DPI 6000 鼠标用力甩时才明显

### 修复：`Listener` 绕过手势竞技场
`Listener` 直接接收原始指针事件（`PointerDown/Move/Up/Cancel`），不参与手势竞技场，不会被任何其他 `GestureDetector` 打断。

```dart
Listener(
  onPointerDown: _onPointerDown,
  onPointerMove: _onPointerMove,
  onPointerUp: _onPointerUp,
  onPointerCancel: _onPointerCancel,
  behavior: HitTestBehavior.translucent,
  child: _buildOrb(),
)
```

### 坐标系陷阱：局部坐标 vs 全局坐标
- **根因**：`PointerEvent.position` 是**局部坐标**（相对于 `Listener`），而 `_pos` 是**全局坐标**（相对于屏幕）。用 `event.position - _pos` 计算 `_dragOffset`，导致坐标系混乱
- **表现**：拖动时球体只能移动极小距离，偶尔跳到右下角等异常位置
- **修复**：拖动判定仍用局部坐标（`event.position - _pointerDownPos`），位置更新改用 `event.delta`（全局位移）

```dart
void _onPointerMove(PointerMoveEvent event) {
  // 局部坐标判定拖动
  final moveDist = (event.position - _pointerDownPos).distance;
  if (!_hasDragged && moveDist > 8.0) {
    _hasDragged = true;
  }
  // delta 是全局位移，不受 Listener 坐标系影响
  if (_hasDragged) {
    setState(() => _pos += event.delta);
  }
}
```

### 点击/拖动判定
- **拖动阈值**：8px（比 `GestureDetector` 默认更宽松，过滤手指轻微抖动）
- **点击判定**：移动 <8px 且按下时间 <250ms
- **拖动判定**：移动 ≥8px 即判定为拖动，立即关闭菜单并跟随手指

```dart
void _onPointerMove(PointerMoveEvent event) {
  final moveDist = (event.position - _pointerDownPos).distance;
  if (!_hasDragged && moveDist > 8.0) {
    _hasDragged = true;
    if (widget.isMenuOpen) widget.onCloseMenu();
  }
  if (_hasDragged) {
    setState(() => _pos = event.position - _dragOffset);
  }
}
```

### 后续扩展基础
`Listener` 方案为后续"悬浮球元素与底部栏元素拖拽交换"打下基础：
- 直接控制 `onPointerDown/Move/Up`，可自由定义拖拽开始/结束时机
- 可在 `_hasDragged` 分支中扩展数据交换逻辑

---

## 第四阶段：吸附动画 listener 泄漏修复 + build 位置保护陷阱

### 问题 A：listener 泄漏（已修复）
- **根因**：`_snapToEdge()` 每次调用都创建新的 `Animation` 并 `addListener`，旧 listener 未清理
- **修复**：`initState` 中只注册一次 listener，`_snapToEdge` 仅更新起止坐标

### 问题 B：中间松手时球体跳到右下角（真正根因）
- **初判**：`PointerEvent.position` 局部坐标与 `_pos` 全局坐标混用
- **逐帧日志分析后**：拖动日志正常，吸附动画 `t=1.086` 时 `_pos.dx=-2.3`（easeOutBack 正常 overshoot）
- **真正根因**：`build` 中的位置保护检测到 `_pos.dx < 0`，瞬间将 `_pos` 重置到默认右下角！
- **表现**：
  - 图1：手指按下，球体在中间
  - 图2：吸附动画 overshoot 到负数，`build` 保护触发重置 → 球体跳到右下角
  - 图3~5：动画反弹回正常值，球体向左吸附

### 修复
1. **`build` 位置保护去敏** — 不再检查 `_pos.dx/dy < 0`，只在未初始化时修正
2. **`_onSnapTick` 末尾 clamp** — 动画结束后将 `_pos` 限制在屏幕内

```dart
// build：只初始化时修正，不拦截 overshoot
if (!_initialized) {
  _pos = Offset(size.width - _orbSize - 16, size.height * 0.62);
  _initialized = true;
}

// _onSnapTick：动画完成后 clamp 到屏幕内
void _onSnapTick() {
  final t = Curves.easeOutBack.transform(_snapController.value);
  final newPos = Offset.lerp(_snapBegin, _snapTarget, t)!;
  final size = MediaQuery.of(context).size;
  final clamped = Offset(
    newPos.dx.clamp(0.0, size.width - _orbSize),
    newPos.dy.clamp(0.0, size.height - _orbSize),
  );
  setState(() => _pos = clamped);
}
```

---

## 最终绘制层次（从外到内）

1. **最外层 bloom** — `blur 65`，范围 3.2r，极淡紫
2. **中层 bloom** — `blur 42`，范围 2.3r，淡紫
3. **近层 bloom** — `blur 26`，范围 1.55r，中紫
4. **球体主体** — 径向渐变：中心白亮 → 淡紫 → 边缘柔和淡出
5. **暖色上层叠加** — 左上偏移，模拟主光源照射
6. **冷色下层叠加** — 右下偏移，模拟环境光
7. **主暖光 + 拖尾** — 大弥散椭圆，带变形和相位拖尾
8. **蓝色冷光** — 不同轨道速度，带变形
9. **淡粉/青绿微光** — 小尺寸高频率变形
10. **边缘流动光斑 × 2** — 带轨迹摆动
11. **微小气泡 × 4** — 快速闪烁+大振幅摆动
12. **中心亮芯** — 2.5倍速脉动
13. **顶部弥散高光区** — 代替生硬白点，径向渐变自然过渡
14. **底部暖反光** — 环境光反射

---

## 关键渲染参数

| 参数 | 值 | 说明 |
|------|-----|------|
| 球体尺寸 | 64px | `_orbSize = 64` |
| 呼吸周期 | 3.5s | `AnimationController` duration |
| 呼吸缩放 | 0.92~1.00 | `0.92 + sin(2πt) * 0.08` |
| 主暖光 blur | 24 | 大范围弥散 |
| 拖尾 blur | 30 | 比主光斑更弥散 |
| 变形幅度 | ±15% | `deformAmount = 0.15` |
| 轨迹摆动 | ±3%~14% | `wobbleAmp` 按光斑大小递减 |
| 微小气泡 | 4个 | 半径 0.028~0.045r |

---

## 核心教训

### 渲染
1. **不要描边/圆环** — `SweepGradient stroke` 在 Flutter 中效果很假，全部用 `MaskFilter.blur` + 径向渐变实现弥散发光
2. **Flutter 的 blur 是渲染瓶颈** — 每层 blur 都有性能开销，移动端控制在 10 层以内
3. **高光要大而弥散** — blur 20~30 才能模拟液体内部大范围折射，blur 5~8 只会像白点贴图
4. **自发光 > 被照亮** — 球体本身应该是光源，不要用深色基底+高光，而是用亮色半透明叠加
5. **菲涅尔边缘** — 径向渐变的边缘 stop 可以比中间更亮，模拟玻璃边缘反光
6. **流动感 = 变形 + 摆动 + 拖尾** — 纯圆周运动不够，要加入长短轴变形、半径摆动、相位拖尾
7. **微小气泡增加生命力** — 4~6个快速闪烁的小点让球体看起来是"活的"

### 手势
8. **GestureDetector 参与手势竞技场会被打断** — 悬浮球在 Overlay 中时，下方编辑器/ScrollView 的 GestureDetector 会竞争并获胜，导致 pan 手势中断
9. **Listener 绕过竞技场** — 直接接收 PointerEvent，不会被任何 GestureDetector 打断，适合需要"抢夺"手势的场景
10. **拖动阈值要适配移动端** — 1.5px 太敏感，手指轻微抖动就会触发；8px 更合理

### 动画
11. **Animation listener 必须清理** — `addListener` 每次调用都新增回调，旧回调未移除会导致多 listener 竞争，出现位置跳变帧
12. **单一 listener + 缓存状态** — `initState` 中注册一次 listener，动画触发时只更新起止坐标，避免泄漏

---

## 参考对比

| | 初始版 | 最终版 |
|--|--------|--------|
| 质感 | 实心紫色圆球 | 通透发光水泡 |
| 边缘 | 无 / 硬边圆环 | 柔和 bloom 弥散 |
| 高光 | 简单白点 | 变形+拖尾+摆动的流动光斑 |
| 内部 | 纯色 | 冷暖交织+微小气泡 |
| 动态 | 仅缩放 | 缩放+变形+摆动+脉动+闪烁 |
| 手势 | GestureDetector（被竞争打断） | Listener（绕过竞技场） |
| 吸附 | 多 listener 泄漏跳变 | 单一 listener 平滑 |
