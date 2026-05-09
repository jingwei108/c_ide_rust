# 移动端与平板适配设计

> 核心问题：用户使用平板时，前端如何支持？不同屏幕大小下 UI 怎么变化？

---

## 1. 设备形态分析

### 1.1 目标设备矩阵

| 设备类型 | 典型尺寸 | 使用场景 | 交互方式 | 优先级 |
|:---|:---|:---|:---|:---|
| **手机（竖屏）** | 360×800 ~ 414×896 | 碎片化学习、代码阅读 | 单指触控、虚拟键盘 | P0 |
| **手机（横屏）** | 800×360 ~ 896×414 | 游戏/视频模式，偶尔编程 | 双手持握、虚拟键盘 | P1 |
| **小平板（竖屏）** | 600×1024 ~ 768×1024 | 课堂学习、算法演示 | 双手触控、可选外接键盘 | P0 |
| **小平板（横屏）** | 1024×600 ~ 1024×768 | 主学习场景、代码编写 | 双手触控、建议外接键盘 | P0 |
| **大平板/折叠屏** | 1024×1366 ~ 1200×1600 | 主力编程设备 | 双手触控、外接键盘/触控笔 | P1 |
| **桌面（调试用）** | 1280×720+ | 开发调试 | 鼠标+键盘 | P1 |

### 1.2 关键差异

```
手机竖屏（窄长）          平板横屏（宽扁）           桌面（超宽）
┌─────────┐              ┌──────────────┐         ┌──────────────────────┐
│ 编辑器   │              │ 编辑器 │ 可视化 │       │ 编辑器 │ 可视化 │ 内存  │
│ （全屏）  │              │ （60%）│ （40%）│      │ （45%）│ （35%）│ （20%）│
│         │              │        │        │       │        │        │       │
│         │              │        │        │       │        │        │       │
├─────────┤              │        │        │       │        │        │       │
│ 控制台   │              ├────────┴────────┤      ├────────┴────────┴───────┤
│ （半屏）  │              │    控制台        │      │        控制台            │
└─────────┘              └─────────────────┘       └─────────────────────────┘

特点：                    特点：                   特点：
• 一屏一功能              • 左右分屏，边写边看        • 三栏布局，信息密度高
• 频繁切换                • 减少切换                • 专业效率
• 触控优先                • 触控+外接键盘           • 键鼠优先
```

---

## 2. 响应式布局架构

### 2.1 Avalonia 响应式工具链

Avalonia 提供三层响应式能力：

```
┌─────────────────────────────────────────────┐
│ Layer 1: OnFormFactor（设备类型）            │
│  • Desktop / Mobile / Tablet                │
│  • 启动时解析，运行时不变                     │
├─────────────────────────────────────────────┤
│ Layer 2: 容器查询（Container Queries）        │
│  • 基于控件自身尺寸自适应                     │
│  • 适合可复用组件                             │
├─────────────────────────────────────────────┤
│ Layer 3: 断点驱动（Breakpoint VM）           │
│  • 基于窗口宽度的实时响应                     │
│  • 最灵活，支持横竖屏切换                     │
└─────────────────────────────────────────────┘
```

### 2.2 断点定义

```csharp
// Core/Responsive/Breakpoints.cs
public enum LayoutBreakpoint {
    Compact,     // < 600px  → 手机
    Medium,      // 600~1024px → 小平板/手机横屏
    Expanded,    // 1024~1280px → 大平板
    Wide         // > 1280px → 桌面/大平板横屏
}

public class ResponsiveLayoutViewModel : ObservableObject {
    [ObservableProperty]
    private LayoutBreakpoint _currentBreakpoint;
    
    [ObservableProperty]
    private bool _isPortrait;     // 是否竖屏
    
    [ObservableProperty]
    private bool _isPhone;        // 是否是手机（Compact + 竖屏）
    
    [ObservableProperty]
    private bool _isTablet;       // 是否是平板（Medium/Expanded）
    
    [ObservableProperty]
    private bool _isDesktop;      // 是否是桌面（Wide）
    
    public void UpdateLayout(double width, double height) {
        CurrentBreakpoint = width switch {
            < 600 => LayoutBreakpoint.Compact,
            < 1024 => LayoutBreakpoint.Medium,
            < 1280 => LayoutBreakpoint.Expanded,
            _ => LayoutBreakpoint.Wide
        };
        
        IsPortrait = height > width;
        IsPhone = CurrentBreakpoint == LayoutBreakpoint.Compact;
        IsTablet = CurrentBreakpoint is LayoutBreakpoint.Medium or LayoutBreakpoint.Expanded;
        IsDesktop = CurrentBreakpoint == LayoutBreakpoint.Wide;
    }
}
```

### 2.3 主窗口响应式布局

```xml
<!-- MainWindow.axaml -->
<Window xmlns="https://github.com/avaloniaui"
        x:Class="Cide.Client.Views.MainWindow"
        x:DataType="vm:MainViewModel">

  <Design.DataContext>
    <vm:MainViewModel />
  </Design.DataContext>

  <!-- 手机布局：底部导航 + 全屏页面切换 -->
  <Panel IsVisible="{Binding Responsive.IsPhone}">
    <Grid RowDefinitions="*, Auto">
      <!-- 主内容区 -->
      <ContentControl Grid.Row="0" Content="{Binding CurrentPage}" />
      
      <!-- 底部导航栏 -->
      <TabControl Grid.Row="1" 
                  SelectedIndex="{Binding SelectedTabIndex}"
                  Classes="mobile">
        <TabItem Header="✏️ 代码" />
        <TabItem Header="▶️ 运行" />
        <TabItem Header="📊 内存" />
        <TabItem Header="⚠️ 错误" />
      </TabControl>
    </Grid>
  </Panel>

  <!-- 平板布局：侧边栏 + 双栏/三栏 -->
  <Panel IsVisible="{Binding Responsive.IsTablet}">
    <SplitView IsPaneOpen="{Binding IsSidebarOpen}"
               DisplayMode="CompactOverlay"
               CompactPaneLength="56"
               OpenPaneLength="240">
      <SplitView.Pane>
        <!-- 侧边导航 -->
        <StackPanel>
          <Button Content="☰" Command="{Binding ToggleSidebarCommand}"
                  Width="56" Height="56" />
          <ListBox ItemsSource="{Binding MenuItems}"
                   SelectedItem="{Binding SelectedMenuItem}">
            <ListBox.ItemTemplate>
              <DataTemplate>
                <StackPanel Orientation="Horizontal" Spacing="12">
                  <TextBlock Text="{Binding Icon}" FontSize="20" />
                  <TextBlock Text="{Binding Title}" 
                             IsVisible="{Binding $parent[SplitView].IsPaneOpen}" />
                </StackPanel>
              </DataTemplate>
            </ListBox.ItemTemplate>
          </ListBox>
        </StackPanel>
      </SplitView.Pane>
      
      <SplitView.Content>
        <!-- 平板主内容：根据方向调整布局 -->
        <ContentControl Content="{Binding TabletLayout}" />
      </SplitView.Content>
    </SplitView>
  </Panel>

  <!-- 桌面布局：三栏固定 -->
  <Panel IsVisible="{Binding Responsive.IsDesktop}">
    <Grid ColumnDefinitions="280, *, 320">
      <!-- 左栏：文件/导航 -->
      <Border Grid.Column="0" Classes="sidebar">
        <views:FileExplorerView />
      </Border>
      
      <!-- 中栏：代码编辑器 -->
      <Grid Grid.Column="1" RowDefinitions="*, Auto">
        <views:CodeEditorView Grid.Row="0" />
        <views:ConsoleView Grid.Row="1" Height="200" />
      </Grid>
      
      <!-- 右栏：可视化/内存/错误 -->
      <Grid Grid.Column="2">
        <TabControl>
          <TabItem Header="内存">
            <views:MemoryView />
          </TabItem>
          <TabItem Header="指针">
            <views:PointerView />
          </TabItem>
          <TabItem Header="错误">
            <views:ErrorPanelView />
          </TabItem>
        </TabControl>
      </Grid>
    </Grid>
  </Panel>

</Window>
```

---

## 3. 平板专用布局设计

### 3.1 平板横屏（主学习场景）

```
┌─────────────────────────────────────────────────────────┐
│ ☰  C IDE                              [▶] [⏸] [⏭]    │  ← 顶部工具栏
├──────────────┬──────────────────────────┬───────────────┤
│              │                          │               │
│  📁 文件      │   1 void bubbleSort(     │   [内存视图]   │
│  ─────────   │   2     int arr[],       │   ┌─┬─┬─┬─┐  │
│  ▶ main.c    │   3     int n) {         │   │5│3│8│1│  │
│  □ utils.c   │   4     for (int i...    │   └─┴─┴─┴─┘  │
│              │   5         for (int...   │               │
│  📚 模板      │   6             if...    │   [指针视图]   │
│  ─────────   │   7                 ...  │   ┌───┐      │
│  冒泡排序     │   8             }        │   │ p │───→  │
│  二分查找     │   9         }            │   └───┘      │
│  链表操作     │  10     }                │               │
│              │                          │               │
├──────────────┴──────────────────────────┤───────────────┤
│ ▶ 运行  │ 输出: 排序完成 [1,2,3,5,8]     │ [变量面板]    │
│          │                               │  i=2, j=1... │
└──────────┴───────────────────────────────┴───────────────┘
         ↑                    ↑                 ↑
      240px                 自适应              300px
      文件树               代码编辑器           可视化面板
```

**设计要点**：
- 左侧文件/模板面板可折叠（`SplitView`）
- 代码编辑器占据主要空间（50%~60%）
- 右侧可视化面板实时展示内存/指针状态
- 底部控制台可折叠

### 3.2 平板竖屏

```
┌─────────────────────────────┐
│ ☰  C IDE         [▶] [⏸]  │
├─────────────────────────────┤
│                             │
│   1 void bubbleSort(        │
│   2     int arr[],          │
│   3     int n) {            │
│   4     for (int i...       │
│   5         for (int...     │
│   6             if...       │
│   7                 ...     │
│   8             }           │
│   9         }               │
│  10     }                   │
│                             │
├─────────────────────────────┤
│ 📊 内存 │ 📍 指针 │ ⚠️ 错误  │  ← Tab 切换
├─────────────────────────────┤
│ ┌─┬─┬─┬─┐                   │
│ │5│3│8│1│  比较中...         │
│ └─┴─┴─┴─┘                   │
├─────────────────────────────┤
│ ▶ 运行  │ 输出: ...          │
└─────────────────────────────┘
```

**设计要点**：
- 代码编辑器全宽
- 可视化面板移至底部，通过 Tab 切换
- 文件面板折叠为抽屉（从左侧滑出）
- 更适合阅读代码，不适合边写边看

---

## 4. 手机专用布局设计

### 4.1 手机竖屏（碎片化学习）

```
┌─────────────────┐
│ ✏️ 代码    ≡    │  ← 标题栏 + 菜单按钮
├─────────────────┤
│                 │
│ 1 void bubble...│
│ 2     int a...  │
│ 3     int n...  │
│ 4     for...    │
│ 5         for...│
│ 6             if│
│ 7               │
│ 8             } │
│ 9         }     │
│ 10    }         │
│                 │
├─────────────────┤
│ [▶ 运行]        │
├─────────────────┤
│ 📊  ▶️  📍  ⚠️  │  ← 底部导航（4 个 Tab）
└─────────────────┘
```

**底部导航 Tab**：
- **✏️ 代码**：代码编辑器
- **▶️ 运行**：控制台输出 + 运行控制
- **📊 内存**：内存视图（简化版）
- **⚠️ 错误**：错误列表

### 4.2 手机横屏

```
┌─────────────────────────────────┐
│ ✏️ 代码            [▶] [⏸] [≡] │
├────────────────┬────────────────┤
│                │                │
│ 1 void bub...  │ 输出:          │
│ 2     int...   │ 排序完成       │
│ 3     int...   │ [1,2,3,5,8]    │
│ 4     for...   │                │
│ 5         f... │                │
│ 6             i│                │
│ 7              │                │
│ 8             }│                │
│                │                │
├────────────────┴────────────────┤
│ 📊  ▶️  📍  ⚠️                   │
└─────────────────────────────────┘
```

**设计要点**：
- 左右分屏：代码 + 输出
- 底部导航保留（但图标更小）

---

## 5. 代码编辑器触控适配

### 5.1 触控优化

```xml
<!-- CodeEditor.axaml -->
<Grid>
  <!-- 行号区 -->
  <Border Width="{OnFormFactor Desktop=40, Mobile=32, Tablet=36}">
    <ItemsControl ItemsSource="{Binding LineNumbers}">
      <ItemsControl.ItemTemplate>
        <DataTemplate>
          <TextBlock Text="{Binding}" 
                     FontSize="{OnFormFactor Desktop=12, Mobile=10, Tablet=11}"
                     Foreground="Gray"
                     HorizontalAlignment="Right"
                     Padding="{OnFormFactor Desktop='0,4', Mobile='0,6', Tablet='0,5'}" />
        </DataTemplate>
      </ItemsControl.ItemTemplate>
    </ItemsControl>
  </Border>
  
  <!-- 代码编辑区 -->
  <TextBox Classes="code-editor"
           FontSize="{OnFormFactor Desktop=14, Mobile=16, Tablet=15}"
           Padding="{OnFormFactor Desktop=4, Mobile=8, Tablet=6}"
           AcceptsReturn="True"
           TextWrapping="NoWrap">
    <TextBox.Styles>
      <!-- 移动端增大触控区域 -->
      <Style Selector="TextBox.code-editor">
        <Setter Property="MinHeight" Value="{OnFormFactor Desktop=20, Mobile=32, Tablet=24}" />
      </Style>
    </TextBox.Styles>
  </TextBox>
  
  <!-- 触控工具栏（移动端显示） -->
  <Border IsVisible="{Binding Responsive.IsPhone}"
          VerticalAlignment="Bottom"
          Background="#F0F0F0">
    <StackPanel Orientation="Horizontal" Spacing="8">
      <Button Content="{" Command="{Binding InsertBraceCommand}" MinWidth="44" MinHeight="44" />
      <Button Content="}" Command="{Binding InsertCloseBraceCommand}" MinWidth="44" MinHeight="44" />
      <Button Content=";" Command="{Binding InsertSemicolonCommand}" MinWidth="44" MinHeight="44" />
      <Button Content="=" Command="{Binding InsertEqualsCommand}" MinWidth="44" MinHeight="44" />
      <Button Content="Tab" Command="{Binding InsertTabCommand}" MinWidth="44" MinHeight="44" />
    </StackPanel>
  </Border>
</Grid>
```

### 5.2 触控手势

参考 2048 项目的滑动处理经验：

```csharp
// Views/CodeEditor.axaml.cs
public partial class CodeEditorView : UserControl {
    private Point _touchStart;
    private DateTime _touchStartTime;
    
    protected override void OnPointerPressed(PointerPressedEventArgs e) {
        base.OnPointerPressed(e);
        _touchStart = e.GetPosition(this);
        _touchStartTime = DateTime.Now;
        e.Pointer.Capture(this);
    }
    
    protected override void OnPointerMoved(PointerEventArgs e) {
        base.OnPointerMoved(e);
        if (!e.Pointer.Captured == this) return;
        
        var pos = e.GetPosition(this);
        var delta = pos - _touchStart;
        
        // 水平滑动：切换代码/输出/内存视图（手机端）
        if (Math.Abs(delta.X) > 50 && Math.Abs(delta.Y) < 30) {
            if (Responsive.IsPhone) {
                if (delta.X < 0) {
                    // 向左滑动 → 下一个 Tab
                    ViewModel.NextTab();
                } else {
                    // 向右滑动 → 上一个 Tab
                    ViewModel.PreviousTab();
                }
            }
        }
    }
    
    protected override void OnPointerReleased(PointerReleasedEventArgs e) {
        base.OnPointerReleased(e);
        e.Pointer.Capture(null);
        
        var pos = e.GetPosition(this);
        var delta = pos - _touchStart;
        var duration = DateTime.Now - _touchStartTime;
        
        // 长按：显示上下文菜单（复制/粘贴/快速修复）
        if (duration.TotalMilliseconds > 500 && delta.Length < 10) {
            ShowContextMenu(pos);
        }
        
        // 双击：选中当前单词
        if (e.ClickCount == 2) {
            SelectCurrentWord(pos);
        }
    }
}
```

### 5.3 虚拟键盘适配

```csharp
// 监听软键盘弹出事件
public class KeyboardAwareLayout : Grid {
    private double _keyboardHeight = 0;
    
    protected override void OnAttachedToVisualTree(VisualTreeAttachmentEventArgs e) {
        base.OnAttachedToVisualTree(e);
        
        // Android 软键盘监听
        if (OperatingSystem.IsAndroid()) {
            SubscribeToKeyboardEvents();
        }
    }
    
    private void OnKeyboardHeightChanged(double height) {
        _keyboardHeight = height;
        
        // 键盘弹出时：
        if (height > 0) {
            // 1. 调整编辑器高度，确保光标可见
            ScrollToCursor();
            
            // 2. 手机端隐藏底部导航栏
            if (Responsive.IsPhone) {
                BottomNavigation.IsVisible = false;
            }
            
            // 3. 平板端缩小可视化面板
            if (Responsive.IsTablet) {
                VisualizationPanel.Height = Math.Min(200, height * 0.5);
            }
        } else {
            // 键盘收起时恢复
            BottomNavigation.IsVisible = true;
            VisualizationPanel.Height = double.NaN; // Auto
        }
    }
}
```

---

## 6. 内存视图与指针视图触控适配

### 6.1 内存视图触控交互

```csharp
// Views/MemoryCanvas.axaml.cs
public class MemoryCanvas : Control {
    private Point _panStart;
    private float _zoom = 1.0f;
    private Point _panOffset;
    
    // 双指缩放
    protected override void OnPointerWheelChanged(PointerWheelEventArgs e) {
        base.OnPointerWheelChanged(e);
        // 鼠标滚轮 / 双指捏合
        var factor = e.Delta.Y > 0 ? 1.1f : 0.9f;
        _zoom = Math.Clamp(_zoom * factor, 0.5f, 3.0f);
        InvalidateVisual();
    }
    
    // 单指平移
    protected override void OnPointerPressed(PointerPressedEventArgs e) {
        base.OnPointerPressed(e);
        if (e.Pointer.Type == PointerType.Touch) {
            _panStart = e.GetPosition(this);
            e.Pointer.Capture(this);
        }
    }
    
    protected override void OnPointerMoved(PointerEventArgs e) {
        base.OnPointerMoved(e);
        if (e.Pointer.Captured == this && e.Pointer.Type == PointerType.Touch) {
            var pos = e.GetPosition(this);
            var delta = pos - _panStart;
            _panOffset = new Point(_panOffset.X + delta.X, _panOffset.Y + delta.Y);
            _panStart = pos;
            InvalidateVisual();
        }
    }
    
    // 点击内存格子显示详情
    protected override void OnPointerReleased(PointerReleasedEventArgs e) {
        base.OnPointerReleased(e);
        e.Pointer.Capture(null);
        
        var pos = e.GetPosition(this);
        var cell = HitTestCell(pos);
        if (cell != null) {
            ShowCellDetailPopup(cell, pos);
        }
    }
    
    // 渲染（带缩放和平移）
    public override void Render(DrawingContext context) {
        context.PushTransform(new MatrixTransform(
            new Matrix(_zoom, 0, 0, _zoom, _panOffset.X, _panOffset.Y)));
        
        // 绘制内存格子...
        
        context.Pop();
    }
}
```

### 6.2 指针视图触控交互

```csharp
// Views/PointerCanvas.axaml.cs
public class PointerCanvas : Control {
    // 长按节点显示详情
    private DateTime _pressTime;
    private Point _pressPos;
    
    protected override void OnPointerPressed(PointerPressedEventArgs e) {
        base.OnPointerPressed(e);
        _pressTime = DateTime.Now;
        _pressPos = e.GetPosition(this);
    }
    
    protected override void OnPointerReleased(PointerReleasedEventArgs e) {
        base.OnPointerReleased(e);
        var duration = DateTime.Now - _pressTime;
        var pos = e.GetPosition(this);
        
        if (duration.TotalMilliseconds > 500 && (pos - _pressPos).Length < 10) {
            // 长按：显示节点详情菜单
            var node = HitTestNode(pos);
            if (node != null) {
                ShowNodeContextMenu(node, pos);
            }
        } else if (e.ClickCount == 1) {
            // 单击：选中节点
            SelectNode(HitTestNode(pos));
        }
    }
    
    // 双指缩放（捏合）
    // Avalonia 12 支持多指触控
    protected override void OnPointerMoved(PointerEventArgs e) {
        // 处理双指捏合缩放...
    }
}
```

---

## 7. 横竖屏切换处理

### 7.1 响应式布局切换

```csharp
// ViewModels/MainViewModel.cs
public partial class MainViewModel : ViewModelBase {
    [ObservableProperty]
    private Control _currentLayout;
    
    partial void OnResponsiveChanged(ResponsiveLayoutViewModel value) {
        UpdateLayout();
    }
    
    private void UpdateLayout() {
        if (Responsive.IsPortrait) {
            // 竖屏：堆叠布局
            CurrentLayout = new PortraitLayoutView {
                DataContext = this
            };
        } else {
            // 横屏：分栏布局
            if (Responsive.IsPhone) {
                // 手机横屏：左右分栏
                CurrentLayout = new PhoneLandscapeLayoutView {
                    DataContext = this
                };
            } else {
                // 平板/桌面横屏：三栏布局
                CurrentLayout = new TabletLandscapeLayoutView {
                    DataContext = this
                };
            }
        }
    }
}
```

### 7.2 状态保持

```csharp
// 横竖屏切换时保持状态
public class LayoutStateManager {
    private Dictionary<string, object> _state = new();
    
    public void SaveState(string key, object value) {
        _state[key] = value;
    }
    
    public T? LoadState<T>(string key) {
        if (_state.TryGetValue(key, out var value) && value is T t) {
            return t;
        }
        return default;
    }
    
    // 切换布局前保存状态
    public void BeforeLayoutChange(MainViewModel vm) {
        SaveState("cursorLine", vm.Editor.CursorLine);
        SaveState("cursorColumn", vm.Editor.CursorColumn);
        SaveState("scrollOffset", vm.Editor.ScrollOffset);
        SaveState("selectedTab", vm.SelectedTabIndex);
        SaveState("isRunning", vm.IsRunning);
    }
    
    // 切换布局后恢复状态
    public void AfterLayoutChange(MainViewModel vm) {
        vm.Editor.CursorLine = LoadState<int>("cursorLine");
        vm.Editor.CursorColumn = LoadState<int>("cursorColumn");
        vm.Editor.ScrollOffset = LoadState<double>("scrollOffset");
        vm.SelectedTabIndex = LoadState<int>("selectedTab");
    }
}
```

---

## 8. 触控优化细节

### 8.1 最小触控区域

```xml
<!-- 所有可交互元素的最小触控区域 -->
<Style Selector="Button">
  <Setter Property="MinWidth" Value="{OnFormFactor Desktop=80, Mobile=48, Tablet=56}" />
  <Setter Property="MinHeight" Value="{OnFormFactor Desktop=32, Mobile=48, Tablet=40}" />
</Style>

<Style Selector="ListBoxItem">
  <Setter Property="MinHeight" Value="{OnFormFactor Desktop=28, Mobile=48, Tablet=36}" />
  <Setter Property="Padding" Value="{OnFormFactor Desktop='8,4', Mobile='16,12', Tablet='12,8'}" />
</Style>

<Style Selector="TextBox">
  <Setter Property="MinHeight" Value="{OnFormFactor Desktop=24, Mobile=44, Tablet=32}" />
</Style>
```

### 8.2 字体大小适配

```xml
<Style Selector="TextBlock.body">
  <Setter Property="FontSize" Value="{OnFormFactor Desktop=14, Mobile=16, Tablet=15}" />
</Style>

<Style Selector="TextBlock.title">
  <Setter Property="FontSize" Value="{OnFormFactor Desktop=20, Mobile=24, Tablet=22}" />
</Style>

<Style Selector="TextBlock.code">
  <Setter Property="FontSize" Value="{OnFormFactor Desktop=13, Mobile=15, Tablet=14}" />
  <!-- 移动端使用等宽字体，提高可读性 -->
  <Setter Property="FontFamily" Value="Consolas, Courier New, monospace" />
</Style>
```

### 8.3 间距适配

```xml
<Style Selector="StackPanel.content">
  <Setter Property="Spacing" Value="{OnFormFactor Desktop=8, Mobile=16, Tablet=12}" />
  <Setter Property="Margin" Value="{OnFormFactor Desktop='16', Mobile='12', Tablet='14'}" />
</Style>
```

---

## 9. 外接键盘/触控笔支持

### 9.1 外接键盘检测

```csharp
public class InputDeviceDetector {
    // 检测是否有外接键盘
    public bool HasHardwareKeyboard {
        get {
            if (OperatingSystem.IsAndroid()) {
                // Android: 检查 Configuration.keyboard
                return AndroidConfiguration.Keyboard != AndroidKeyboard.Nokeys;
            }
            return false;
        }
    }
    
    // 检测是否有触控笔
    public bool HasStylus =>
        PointerDevice.GetPointerDevices().Any(d => d.Type == PointerDeviceType.Pen);
}

// 根据输入设备调整 UI
public void AdaptToInputDevices() {
    if (InputDeviceDetector.HasHardwareKeyboard) {
        // 有外接键盘：显示更多功能
        ShowShortcutHints = true;
        ToolbarHeight = 40;  // 更小的工具栏
        ShowVirtualKeyboardButton = false;
    } else {
        // 纯触控：增大触控区域
        ShowShortcutHints = false;
        ToolbarHeight = 56;
        ShowVirtualKeyboardButton = true;
    }
    
    if (InputDeviceDetector.HasStylus) {
        // 支持触控笔：启用精确选择
        EnableStylusPrecisionMode = true;
    }
}
```

### 9.2 快捷键提示

```xml
<!-- 有外接键盘时显示快捷键提示 -->
<StackPanel IsVisible="{Binding HasHardwareKeyboard}">
  <TextBlock Text="快捷键:" FontSize="12" Foreground="Gray" />
  <TextBlock Text="▶ F5 运行" FontSize="12" />
  <TextBlock Text="⏸ F6 暂停" FontSize="12" />
  <TextBlock Text="⏭ F8 下一步" FontSize="12" />
</StackPanel>
```

---

## 10. 性能优化

### 10.1 移动端渲染优化

```csharp
public class MobileRenderOptimizer {
    public void OptimizeForMobile(Control root) {
        if (!Responsive.IsPhone && !Responsive.IsTablet) return;
        
        // 1. 降低动画帧率（省电）
        Animation.GlobalClockRate = 0.5;  // 30fps instead of 60fps
        
        // 2. 禁用不必要的特效
        root.Styles.Add(new Style(x => x.Is<Border>()) {
            Setters = {
                new Setter(BoxShadowProperty, null),  // 禁用阴影
            }
        });
        
        // 3. 内存视图简化渲染
        if (Responsive.IsPhone) {
            MemoryCanvas.MaxVisibleCells = 64;  // 手机端最多显示 64 个内存格子
        }
        
        // 4. 指针视图简化
        if (Responsive.IsPhone) {
            PointerCanvas.MaxVisibleNodes = 20;  // 手机端最多显示 20 个节点
        }
    }
}
```

### 10.2 参考 2048 的闪退修复经验

```csharp
// 参考 2048 的 CancelAllAnimations + SnapToGrid 模式
public class TouchSafeAnimator {
    private readonly List<DispatcherTimer> _activeTimers = new();
    
    public void StartAnimation(Action frameAction) {
        // 开始新动画前：取消旧动画
        CancelAllAnimations();
        
        var timer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(16) };
        _activeTimers.Add(timer);
        timer.Tick += (_, _) => {
            frameAction();
            InvalidateVisual();
        };
        timer.Start();
    }
    
    public void CancelAllAnimations() {
        foreach (var timer in _activeTimers.ToList()) {
            timer.Stop();
        }
        _activeTimers.Clear();
    }
    
    // 快速切换时：同步到最终状态
    public void SnapToFinalState() {
        CancelAllAnimations();
        // 同步所有 UI 状态到最终值...
    }
}
```

---

## 11. 实施优先级

### Phase 1：核心适配（2-3 周）
- [ ] 响应式布局框架（Breakpoint VM + OnFormFactor）
- [ ] 手机竖屏布局（底部导航 + 全屏 Tab）
- [ ] 平板横屏布局（SplitView 侧边栏 + 双栏）
- [ ] 代码编辑器触控优化（增大触控区域、虚拟键盘适配）
- [ ] 内存视图触控交互（缩放、平移、点击查看详情）

### Phase 2：体验优化（2 周）
- [ ] 横竖屏切换状态保持
- [ ] 手势操作（滑动切换 Tab、捏合缩放）
- [ ] 外接键盘检测与适配
- [ ] 性能优化（降帧率、简化渲染）

### Phase 3：进阶适配（1-2 周）
- [ ] 折叠屏适配
- [ ] 触控笔支持
- [ ] 分屏/多窗口支持（Android 多任务）

---

## 12. 总结

### 平板布局核心策略

| 场景 | 布局策略 | 关键交互 |
|:---|:---|:---|
| **平板横屏**（主力场景） | 三栏/双栏：编辑器 + 可视化 + 文件 | 触控 + 外接键盘、侧边栏折叠 |
| **平板竖屏** | 编辑器全宽 + 底部 Tab 切换可视化 | 纯触控、抽屉式文件面板 |
| **手机竖屏** | 底部导航 + 全屏页面 | 滑动切换、长按菜单、虚拟键盘 |
| **手机横屏** | 左右分栏：代码 + 输出 | 双手持握 |

### 核心原则

1. **一屏一功能（手机）**：不要在小屏幕上挤太多内容
2. **边写边看（平板横屏）**：利用宽度优势，代码和可视化并排
3. **触控优先**：所有可交互元素 ≥ 48dp 触控区域
4. **状态保持**：横竖屏切换不丢失编辑位置和运行状态
5. **渐进增强**：根据设备能力（键盘、触控笔）动态调整功能

### 与 2048 的参考对比

| 维度 | 2048 | 本项目 C IDE |
|:---|:---|:---|
| 设备支持 | Android 手机 | Android 手机 + 平板 + 桌面 |
| 布局复杂度 | 单页面游戏 | 多视图 IDE（编辑器/内存/指针/错误） |
| 交互方式 | 滑动 + 点击 | 滑动 + 点击 + 长按 + 捏合 + 键盘 |
| 横竖屏 | 基本适配 | 完全响应式，状态保持 |
| 虚拟键盘 | 不涉及 | 核心适配（代码编辑器） |
