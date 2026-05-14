# 悬浮球 UI 设计文档

> 创建时间: 2026-05-11

## 平台说明

| 平台 | 状态 | 说明 |
|------|------|------|
| **移动端 (MAUI Blazor)** | ✅ 生产中 | 当前唯一活跃实现，`FloatingActionButton.razor` + `Home.razor` |
| **桌面端 (Avalonia)** | 🚧 已废弃，仅用于测试 | `MainView.axaml` 中的扇形 FAB 保留作交互原型验证，不随产品发布 |

> **注意**：桌面端悬浮球代码（`Cide.Client/Views/MainView.axaml` 及 `.axaml.cs` 中的 FAB 相关逻辑）已停止维护，仅作为早期交互实验留存。所有功能迭代、视觉更新、Bug 修复均以移动端 `FloatingActionButton.razor` 为准。

## 设计目标

解决原悬浮球的三个问题：
1. **小球重叠** — 扇形展开时 8 个 44px 圆球挤在 120° 扇区内，互相遮挡
2. **信息不清晰** — 每个菜单项只有一个中文字（栈/表/变/存/组/针/知/算），用户无法直观理解功能
3. **自身 UI 单调** — 蓝色渐变圆形缺乏辨识度，视觉吸引力不足

## 最终方案：星空星球 + 垂直堆叠菜单

### 悬浮球本体

- **无字、无图标**，纯视觉呈现
- 56px 发光星球体，多层径向渐变模拟球面光影：
  - 顶部金橙色高光（`#FBBF60`）
  - 右侧紫蓝辉光（`#8B5CF6`）
  - 主体深紫到暗蓝（`#5B3CC4` → `#140F23`）
  - 底部暗部阴影
- **动态效果**：
  - `::before` 旋转 conic-gradient 光晕（6s 线性无限循环）
  - `::after` 内部光斑缓慢漂移（8s ease-in-out 交替）
  - 外层紫色 box-shadow 辉光
- **交互反馈**：
  - 展开时球体渐变为红紫关闭色
  - 拖拽时放大 1.15x 并增强发光
  - 按下时缩小 0.92x

### 菜单展开方式

- 放弃扇形分布，改为**垂直堆叠长条按钮**
- 根据悬浮球在屏幕的位置自动判断展开方向：
  - 球在屏幕上半部 → 向下展开
  - 球在屏幕下半部 → 向上展开
- 118×40px 圆角长条，有足够空间显示完整文字

### 菜单项内容

去掉底部已重复的"算法"项，共 7 项：

| 索引 | 文字 |
|------|------|
| 0 | 调用栈 |
| 1 | 监视变量 |
| 2 | 局部变量 |
| 3 | 内存区域 |
| 4 | 数组可视化 |
| 5 | 指针视图 |
| 6 | 知识卡片 |

### 过渡动画

1. **背景条**：`scale(0.5)` → `scale(1)` + `opacity: 0` → `1`，0.4s，弹性贝塞尔
2. **菜单项**：逐个延迟 40ms 弹出，`scale(0.85)` → `scale(1)` + `opacity` 变化，0.35s
3. 曲线：`cubic-bezier(0.34, 1.56, 0.64, 1)`（带过冲回弹）

## 关键文件

| 文件 | 说明 |
|------|------|
| `Cide.Client.Maui/Components/Editor/FloatingActionButton.razor` | Blazor 组件逻辑：垂直堆叠位置计算、展开方向自适应 |
| `Cide.Client.Maui/Components/Editor/FloatingActionButton.razor.css` | 星球球体样式、光晕动画、菜单长条样式 |

## 核心 CSS 片段

```css
/* 星球球体 — 多层径向渐变 */
.fab-main {
    background:
        /* 顶部金橙高光 */
        radial-gradient(ellipse 65% 55% at 35% 25%,
            rgba(251, 191, 96, 0.75) 0%, transparent 55%),
        /* 右侧紫蓝辉光 */
        radial-gradient(ellipse 70% 70% at 75% 45%,
            rgba(139, 92, 246, 0.6) 0%, transparent 60%),
        /* 主球体深紫 */
        radial-gradient(ellipse 90% 90% at 45% 55%,
            rgba(91, 60, 196, 0.9) 0%,
            rgba(15, 10, 35, 0.95) 100%);
    box-shadow:
        0 0 20px rgba(139, 92, 246, 0.35),
        0 4px 12px rgba(0, 0, 0, 0.4);
}

/* 旋转光晕 */
.fab-main::before {
    background: conic-gradient(
        from 0deg,
        transparent 0%,
        rgba(139, 92, 246, 0.4) 15%,
        rgba(96, 165, 250, 0.35) 30%,
        rgba(251, 191, 96, 0.3) 45%,
        transparent 60%,
        rgba(139, 92, 246, 0.25) 75%,
        transparent 100%
    );
    filter: blur(6px);
    animation: planetGlow 6s linear infinite;
}
```

## 截图参考

悬浮球视觉参考联想 AI 快开样式：
- 深色背景上的发光球体
- 紫/蓝/橙三色在球面自然过渡
- 无文字、无图标，纯形态识别
