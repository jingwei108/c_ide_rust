# 面板拖拽手势设计文档

## 1. 设计目标

- **纯交换（Swap Only）**：所有拖拽交互仅执行交换，不改变两区元素数量。
- **精确交换**：用户拖到哪个元素，就与该元素交换位置。
- **视觉反馈**：hover 时目标元素蓝色边框+阴影，原位置半透明，feedback 带发光效果。
- **防误触**：同区域（悬浮球内部）拖拽不触发交换，避免误操作。
- **边缘兜底**：拖到无目标区域时，松手后 SnackBar 提示，不静默失败。

## 2. 手势区域划分

```
┌─────────────────────────────────────┐
│                                     │
│           编辑器区域                 │
│                                     │
│                                     │
│                              ┌─────┐│
│                              │ 球体 ││  ← 悬浮球（可拖拽吸附）
│                              └─────┘│
│                             ┌──────┐│
│                             │ 输出  ││  ← 底部 Tab 区域（最多3个）
│                             │ 诊断  ││
│                             │ 算法  ││
│                             └──────┘│
└─────────────────────────────────────┘

悬浮球展开后：
┌─────────────────────────────────────┐
│                             ┌──────┐│
│                             │知识  ││  ← 菜单项（每个独立 DragTarget）
│                             │指针  ││
│                             │数组  ││
│                             │内存  ││
│                             │ ...  ││
│                             └──────┘│
│                              ┌─────┐│
│                              │ 球体 ││
└─────────────────────────────────────┘
```

## 3. 拖拽交互规则

### 3.1 底部 Tab 区域

| 拖拽方向 | 触发元素 | 目标元素 | 行为 |
|---------|---------|---------|------|
| 底部 Tab → 底部 Tab | `DraggablePanelTab` | 另一个 `DraggablePanelTab` 的 `DragTarget` | 同区交换位置 |
| 底部 Tab → 悬浮球菜单项 | `DraggablePanelTab` | `MenuItemDragTarget` | 与目标菜单项交换 |
| 底部 Tab → 悬浮球边缘/球体 | `DraggablePanelTab` | `MenuDragTarget` / `OrbDragTarget` | SnackBar 提示"未识别到" |

### 3.2 悬浮球区域

| 拖拽方向 | 触发元素 | 目标元素 | 行为 |
|---------|---------|---------|------|
| 悬浮球菜单项 → 底部 Tab | `MenuItemDraggable` | `DraggablePanelTab` 的 `DragTarget` | 与目标底部 Tab 交换 |
| 悬浮球菜单项 → 悬浮球菜单项 | `MenuItemDraggable` | `MenuItemDragTarget` | **不响应**（同区过滤） |
| 悬浮球菜单项 → 悬浮球边缘 | `MenuItemDraggable` | `MenuDragTarget` | **不响应**（同区过滤） |

## 4. 交换逻辑实现

### 4.1 同区交换

```dart
// 底部 Tab A 拖到 底部 Tab B
notifier.swapBottomPanels(indexA, indexB);
```

### 4.2 跨区域交换（精确位置）

```dart
// 底部 Tab "诊断" 拖到 悬浮球第 2 个菜单项 "指针"
notifier.swapBottomWithFloatingItem('diagnostics', 2);
// 结果: bottomSlots[?] = 'pointer', floatingSlots[2] = 'diagnostics'

// 悬浮球 "内存" 拖到 底部第 1 个 Tab "输出"
notifier.swapFloatingWithBottomItem('memory', 0);
// 结果: floatingSlots[?] = 'output', bottomSlots[0] = 'memory'
```

### 4.3 关键约束

- 两区元素总数始终不变。
- 不执行增删操作，仅交换 list 中的元素值。
- 交换后 activeIndex 跟随被移动的元素，确保用户视角连续性。

## 5. 视觉反馈设计

### 5.1 Hover 效果（目标元素）

```dart
Container(
  decoration: BoxDecoration(
    borderRadius: BorderRadius.circular(8),
    border: isHovering
        ? Border.all(color: Colors.blueAccent.withValues(alpha: 0.6), width: 1.5)
        : null,
    boxShadow: isHovering
        ? [BoxShadow(color: Colors.blueAccent.withValues(alpha: 0.2), blurRadius: 6, spreadRadius: 1)]
        : null,
  ),
)
```

### 5.2 拖拽中效果（原位置）

```dart
childWhenDragging: Opacity(opacity: 0.5, child: menuItem),
```

### 5.3 Feedback（跟随物）

```dart
feedback: Material(
  elevation: 8,
  child: Container(
    decoration: BoxDecoration(
      border: Border.all(color: Colors.blueAccent.withValues(alpha: 0.6)),
      boxShadow: [BoxShadow(color: Colors.black.withValues(alpha: 0.5), blurRadius: 12)],
    ),
  ),
),
```

## 6. 防误触与过滤

### 6.1 同区过滤

悬浮球菜单项的 `DragTarget` 仅接受来自 `PanelLocation.bottom` 的拖拽：

```dart
DragTarget<PanelDragData>(
  onWillAcceptWithDetails: (details) {
    return details.data.fromLocation == PanelLocation.bottom;
  },
)
```

这样悬浮球内部拖拽不会显示 hover 效果，也不会触发交换。

### 6.2 边缘兜底

菜单面板整体保留一个 `DragTarget`，仅接收底部→悬浮球的拖拽，用于处理 padding 区域：

```dart
onAcceptWithDetails: (details) {
  // 拖到边缘，未落在具体菜单项上
  ScaffoldMessenger.of(context).showSnackBar(
    const SnackBar(content: Text('未识别到可交换的目标位置')),
  );
}
```

## 7. 悬浮球菜单展开方向

**优先向上展开**，方便底部 Tab 向上拖拽到菜单项：

```dart
bool get _menuGoesUp {
  final menuHeight = items.length * _menuItemHeight + 16;
  // 上方空间 >= 菜单高度 + 间距8 + 余量20
  return _pos.dy >= menuHeight + 28;
}
```

- 只要球体不贴顶，菜单向上展开，拖拽路径最短。
- 上方空间不够时，才向下展开。

## 8. 菜单宽度自适应

菜单栏宽度**由最多文字的长度动态决定**，不硬编码：

```dart
double _calcMenuWidth(List<PanelItem> items) {
  const textStyle = TextStyle(
    color: Color(0xFFE0E0F0),
    fontSize: 13,
    fontWeight: FontWeight.w500,
  );
  double maxTextWidth = 0;
  for (final item in items) {
    final tp = TextPainter(
      text: TextSpan(text: item.label, style: textStyle),
      textDirection: TextDirection.ltr,
    );
    tp.layout();
    if (tp.width > maxTextWidth) maxTextWidth = tp.width;
  }
  // icon(16) + SizedBox(10) + 左右padding(14*2) + 余量(8)
  return 16 + 10 + maxTextWidth + 28 + 8;
}
```

- 移除所有 `width: 148` 硬编码。
- 菜单项 `Row` 中移除 `Expanded`，让文字自然宽度决定整体宽度。
- `feedback` 同样移除固定宽度，保持与菜单项一致。

## 9. 文件改动清单

| 文件 | 改动 |
|------|------|
| `lib/providers/ide_notifier.dart` | 新增 `swapBottomWithFloatingItem`、`swapFloatingWithBottomItem` |
| `lib/widgets/floating_orb_widget.dart` | 项级 `DragTarget`、同区过滤、菜单优先向上展开 |
| `lib/widgets/draggable_panel_tab.dart` | 底部 Tab 拖拽 feedback + hover 效果 |
| `lib/screens/ide_screen.dart` | 桥接回调：精确交换 + 边缘 SnackBar 提示 |
