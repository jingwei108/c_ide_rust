using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Templates;
using Avalonia.Input;
using Avalonia.Media;
using Cide.Client.ViewModels;

namespace Cide.Client.Views;

public partial class MainView : UserControl
{
    private bool _isFabOpen = false;
    private ComboBox? _templatePicker;
    private CodeEditor? _codeEditor;

    // FAB 拖拽状态
    private bool _isFabDragging = false;
    private Point _fabDragStartPos;
    private Point _fabDragOffset;
    private const double FabDragThreshold = 8.0;
    private bool _fabUserPositioned = false;

    public MainView()
    {
        Console.WriteLine("[CIDE_MAINVIEW] Constructor START");
        try
        {
            InitializeComponent();
            Console.WriteLine("[CIDE_MAINVIEW] InitializeComponent done");
        }
        catch (Exception ex)
        {
            Console.WriteLine($"[CIDE_MAINVIEW] InitializeComponent FAILED: {ex}");
            throw;
        }

        try
        {
            _templatePicker = this.FindControl<ComboBox>("TemplatePicker");
            if (_templatePicker != null)
            {
                _templatePicker.SelectionChanged += OnTemplateSelected;
            }
            Console.WriteLine("[CIDE_MAINVIEW] Constructor END");
        }
        catch (Exception ex)
        {
            Console.WriteLine($"[CIDE_MAINVIEW] Post-init FAILED: {ex}");
            throw;
        }

        this.Loaded += OnLoaded;
        this.SizeChanged += OnSizeChanged;
    }

    private Point _swipeStart;
    private bool _swipeHandled;
    private const double SwipeThreshold = 60.0;
    private const double SwipeMaxVertical = 40.0;

    private void OnLoaded(object? sender, global::Avalonia.Interactivity.RoutedEventArgs e)
    {
        UpdateResponsiveLayout();

        // Hook swipe gesture on the code editor area for mobile tab switching
        _codeEditor = this.FindControl<CodeEditor>("CodeEditor");
        if (_codeEditor != null)
        {
            _codeEditor.PointerPressed += OnEditorSwipePressed;
            _codeEditor.PointerMoved += OnEditorSwipeMoved;
            _codeEditor.PointerReleased += OnEditorSwipeReleased;
        }
    }

    protected override void OnDetachedFromVisualTree(VisualTreeAttachmentEventArgs e)
    {
        base.OnDetachedFromVisualTree(e);

        if (_templatePicker != null)
        {
            _templatePicker.SelectionChanged -= OnTemplateSelected;
            _templatePicker = null;
        }

        if (_codeEditor != null)
        {
            _codeEditor.PointerPressed -= OnEditorSwipePressed;
            _codeEditor.PointerMoved -= OnEditorSwipeMoved;
            _codeEditor.PointerReleased -= OnEditorSwipeReleased;
            _codeEditor = null;
        }

        this.Loaded -= OnLoaded;
        this.SizeChanged -= OnSizeChanged;

        if (DataContext is IDisposable disposable)
        {
            disposable.Dispose();
        }
    }

    private void OnEditorSwipePressed(object? sender, PointerPressedEventArgs e)
    {
        _swipeStart = e.GetPosition(this);
        _swipeHandled = false;
        e.Pointer.Capture(sender as Avalonia.Input.InputElement);
    }

    private void OnEditorSwipeMoved(object? sender, PointerEventArgs e)
    {
        if (_swipeHandled) return;
        if (DataContext is not MainViewModel vm) return;
        if (!vm.Responsive.IsPhone) return; // Only enable on phone

        var currentPos = e.GetPosition(this);
        double dx = currentPos.X - _swipeStart.X;
        double dy = currentPos.Y - _swipeStart.Y;

        if (Math.Abs(dx) > SwipeThreshold && Math.Abs(dy) < SwipeMaxVertical)
        {
            _swipeHandled = true;
            e.Pointer.Capture(null);
            if (dx < 0)
            {
                vm.NextTabCommand.Execute(null); // swipe left → next tab
            }
            else
            {
                vm.PreviousTabCommand.Execute(null); // swipe right → previous tab
            }
        }
    }

    private void OnEditorSwipeReleased(object? sender, PointerReleasedEventArgs e)
    {
        e.Pointer.Capture(null);
    }

    private void OnSizeChanged(object? sender, SizeChangedEventArgs e)
    {
        UpdateResponsiveLayout();
    }

    private void UpdateResponsiveLayout()
    {
        if (DataContext is not MainViewModel vm) return;
        if (RootGrid == null) return;

        var topLevel = TopLevel.GetTopLevel(this);
        var clientSize = topLevel?.ClientSize ?? new Size(0, 0);
        var boundsSize = this.Bounds.Size;

        Console.WriteLine($"[CIDE_MAINVIEW] UpdateResponsiveLayout: ClientSize={clientSize}, Bounds={boundsSize}, DataContext={(vm != null)}");

        var size = clientSize.Width > 0 ? clientSize : boundsSize;
        if (size.Width > 0 && size.Height > 0 && vm != null)
        {
            vm.UpdateLayout(size.Width, size.Height);
        }

        UpdateResponsiveMetrics(size.Width, size.Height);
        PositionFabElements();
    }

    /// <summary>
    /// 全站响应式度量计算：根据屏幕逻辑宽高动态缩放所有设计令牌。
    /// 以 400px 逻辑宽度为基准，所有宏观尺寸按等比缩放，并做上下限保护防止过小或过大。
    /// </summary>
    private void UpdateResponsiveMetrics(double width, double height)
    {
        if (width <= 0) return;
        try
        {
            double scale = Math.Clamp(width / 400.0, 0.85, 1.45);
            bool isPortrait = height > width;

            void SetResource(string key, object value)
            {
                if (Resources.ContainsKey(key))
                    Resources[key] = value;
                else
                    Resources.Add(key, value);
            }

        // ===== 工具栏 =====
        double toolbarTop = Math.Clamp(28 * scale, 20, 40);
        double toolbarBottom = Math.Clamp(6 * scale, 4, 10);
        SetResource("MobileToolbarPadding", new Thickness(8, toolbarTop, 8, toolbarBottom));
        SetResource("MobileBtnPadding", new Thickness(Math.Clamp(14 * scale, 10, 22), Math.Clamp(6 * scale, 4, 10)));
        SetResource("MobileBtnFontSize", Math.Clamp(14 * scale, 12, 18));
        SetResource("MobileStepBtnFontSize", Math.Clamp(13 * scale, 11, 17));
        SetResource("MobileSliderWidth", Math.Clamp(80 * scale, 60, 120));
        SetResource("MobileStatusPadding", new Thickness(Math.Clamp(8 * scale, 6, 14), Math.Clamp(4 * scale, 2, 8)));
        SetResource("MobileStatusFontSize", Math.Clamp(11 * scale, 10, 14));

        // ===== 编辑器区域 =====
        double editorMargin = Math.Clamp(4 * scale, 2, 8);
        SetResource("EditorMargin", new Thickness(editorMargin));
        SetResource("EditorFontSize", Math.Clamp(14 * scale, 12, 18));
        SetResource("EditorPadding", new Thickness(Math.Clamp(8 * scale, 6, 14), Math.Clamp(4 * scale, 2, 8)));
        // 行号区：竖屏更紧凑，横屏略宽；左边距清零让数字贴近边缘
        SetResource("LineNumberWidth", Math.Clamp(isPortrait ? 34 * scale : 38 * scale, 30, 52));
        // 行号区精简：去除多余右侧内边距，让数字贴近右边缘
        SetResource("LineNumberMargin", new Thickness(0, Math.Clamp(4 * scale, 2, 8), 0, 0));
        SetResource("LineNumberItemMargin", new Thickness(0, 0, 0, 0));
        SetResource("LineNumberEllipseMargin", new Thickness(Math.Clamp(2 * scale, 1, 4), 0, 0, 0));
        SetResource("LineNumberTextPadding", new Thickness(0, Math.Clamp(2 * scale, 1, 4), Math.Clamp(2 * scale, 1, 3), Math.Clamp(2 * scale, 1, 4)));
        SetResource("LineNumberFontSize", Math.Clamp(13 * scale, 11, 17));

        // ===== 底部面板 =====
        double bottomHeight = isPortrait ? Math.Clamp(140 * scale, 100, 200) : Math.Clamp(100 * scale, 80, 160);
        SetResource("BottomPanelHeight", bottomHeight);
        SetResource("BottomPanelMargin", new Thickness(editorMargin));
        SetResource("ConsoleOutputMargin", new Thickness(Math.Clamp(8 * scale, 6, 14)));
        SetResource("ConsoleOutputFontSize", Math.Clamp(14 * scale, 12, 18));

        // ===== 模板栏 =====
        SetResource("TemplateBarMargin", new Thickness(editorMargin));
        SetResource("TemplateBarItemMargin", new Thickness(Math.Clamp(4 * scale, 2, 8), 0));
        SetResource("TemplateBtnPadding", new Thickness(Math.Clamp(8 * scale, 6, 14), Math.Clamp(4 * scale, 2, 8)));
        SetResource("TemplateBtnFontSize", Math.Clamp(12 * scale, 10, 16));
        SetResource("TemplateBtnCornerRadius", new CornerRadius(Math.Clamp(12 * scale, 8, 18)));

        // ===== 诊断/算法卡片 =====
        double cardPad = Math.Clamp(8 * scale, 6, 14);
        double cardMargin = Math.Clamp(4 * scale, 2, 8);
        SetResource("CardPadding", new Thickness(cardPad));
        SetResource("CardMargin", new Thickness(0, cardMargin));
        SetResource("CardCornerRadius", new CornerRadius(Math.Clamp(4 * scale, 3, 8)));
        SetResource("CardInnerPadding", new Thickness(Math.Clamp(6 * scale, 4, 10)));
        SetResource("CardInnerCornerRadius", new CornerRadius(Math.Clamp(2 * scale, 2, 6)));
        SetResource("CardFontSize", Math.Clamp(11 * scale, 10, 14));
        SetResource("CardTitleFontSize", Math.Clamp(12 * scale, 10, 16));

        // ===== 桌面调试面板 =====
        SetResource("DebugPanelWidth", Math.Clamp(300 * scale, 240, 400));
        SetResource("DebugPanelMargin", new Thickness(editorMargin, 0, 0, 0));
        SetResource("DebugPanelHeaderMargin", new Thickness(Math.Clamp(8 * scale, 6, 14), Math.Clamp(8 * scale, 6, 14), Math.Clamp(8 * scale, 6, 14), Math.Clamp(4 * scale, 2, 8)));
        SetResource("DebugPanelHeaderFontSize", Math.Clamp(14 * scale, 12, 18));
        SetResource("DebugItemFontSize", Math.Clamp(12 * scale, 10, 16));
        SetResource("DebugItemPadding", new Thickness(Math.Clamp(6 * scale, 4, 10)));
        SetResource("DebugItemMargin", new Thickness(0, Math.Clamp(2 * scale, 1, 4)));
        SetResource("DebugItemCornerRadius", new CornerRadius(Math.Clamp(3 * scale, 2, 6)));
        SetResource("DebugItemDetailFontSize", Math.Clamp(11 * scale, 10, 14));

        // ===== 移动端模态面板 =====
        SetResource("ModalWidth", Math.Clamp(360 * scale, 280, 480));
        SetResource("ModalHeight", Math.Clamp(520 * scale, 400, 680));
        SetResource("ModalCornerRadius", new CornerRadius(Math.Clamp(16 * scale, 12, 24)));
        SetResource("ModalCloseBtnMargin", new Thickness(Math.Clamp(8 * scale, 6, 14)));
        SetResource("ModalCloseBtnFontSize", Math.Clamp(16 * scale, 14, 22));

        // ===== 悬浮球 =====
        // 横屏时以高度为基准限制尺寸，避免过宽屏幕上悬浮球过大
        double refScale = isPortrait ? scale : Math.Clamp(height / 400.0, 0.85, 1.35);
        double fabSize = Math.Clamp(56 * refScale, 44, 64);
        double fanSize = Math.Clamp(44 * refScale, 36, 52);
        SetResource("FabAreaWidth", Math.Clamp(160 * scale, 120, 220));
        SetResource("FabAreaHeight", Math.Clamp(360 * scale, 280, 480));
        SetResource("FabSize", fabSize);
        SetResource("FabCornerRadius", new CornerRadius(fabSize / 2));
        SetResource("FabFontSize", Math.Clamp(22 * scale, 18, 30));
        SetResource("FanBtnSize", fanSize);
        SetResource("FanBtnCornerRadius", new CornerRadius(fanSize / 2));
        SetResource("FanBtnFontSize", Math.Clamp(16 * scale, 12, 22));
        SetResource("FabRadius", Math.Clamp(90 * scale, 70, 130));

        // 底部标签栏：三键均分屏幕宽度，无硬编码
        SetResource("TabHeaderFontSize", Math.Clamp(12 * scale, 10, 16));
        SetResource("TabHeaderPadding", new Thickness(Math.Clamp(8 * scale, 6, 14), Math.Clamp(4 * scale, 2, 8)));
        SetResource("TabItemWidth", width / 3.0);

        // 悬浮球边距：底部留出 BottomPanelHeight 空间，避免遮挡底部面板
        SetResource("FabAreaMargin", new Thickness(0, 0, 0, bottomHeight));
        }
        catch (Exception ex)
        {
            Console.WriteLine($"[CIDE_RESP] UpdateResponsiveMetrics FAILED: {ex}");
        }
    }

    private void PositionFabElements()
    {
        try
        {
            if (FabCanvas == null || FabButtonBorder == null) return;

            double w = FabCanvas.Bounds.Width;
            double h = FabCanvas.Bounds.Height;
            if (w <= 0 || h <= 0) return;

            double fabSize = Resources.TryGetValue("FabSize", out var fs) && fs is double d1 ? d1 : 56;
            double fanSize = Resources.TryGetValue("FanBtnSize", out var fns) && fns is double d2 ? d2 : 44;
            double radius = Resources.TryGetValue("FabRadius", out var fr) && fr is double d3 ? d3 : 90;
            double halfFab = fabSize / 2;
            double halfFan = fanSize / 2;

            // 主悬浮球默认位置：右侧中间偏下（用户要求）
            double fabX, fabY;
            if (_fabUserPositioned)
            {
                // 用户已拖拽过，使用当前位置
                fabX = Canvas.GetLeft(FabButtonBorder) + halfFab;
                fabY = Canvas.GetTop(FabButtonBorder) + halfFab;
            }
            else
            {
                fabX = w - halfFab;
                fabY = h * 0.65; // 右侧中间偏下
            }

            // 收起状态贴边吸附：保留约 3/4 在屏幕外
            if (!_isFabOpen)
            {
                fabX = w + halfFab / 2;
            }

            Canvas.SetLeft(FabButtonBorder, fabX - halfFab);
            Canvas.SetTop(FabButtonBorder, fabY - halfFab);

            // 扇形按钮位置（现在包裹在 Border 内）
            var fans = new[]
            {
                FanCallStackBorder, FanWatchBorder, FanVariablesBorder, FanMemoryBorder,
                FanArrayBorder, FanPointerBorder, FanKnowledgeBorder
            };

            double startAngle = 120;
            double endAngle = 240;
            double step = fans.Length > 1 ? (endAngle - startAngle) / (fans.Length - 1) : 0;

            for (int i = 0; i < fans.Length; i++)
            {
                var btn = fans[i];
                if (btn == null) continue;

                if (_isFabOpen)
                {
                    double angle = (startAngle + step * i) * Math.PI / 180;
                    double fx = fabX + radius * Math.Cos(angle);
                    double fy = fabY - radius * Math.Sin(angle);
                    Canvas.SetLeft(btn, fx - halfFan);
                    Canvas.SetTop(btn, fy - halfFan);
                    btn.Opacity = 1;
                    btn.IsHitTestVisible = true;
                }
                else
                {
                    Canvas.SetLeft(btn, fabX - halfFan);
                    Canvas.SetTop(btn, fabY - halfFan);
                    btn.Opacity = 0;
                    btn.IsHitTestVisible = false;
                }
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"[CIDE_FAB] PositionFabElements FAILED: {ex}");
        }
    }

    private void OnFabPointerPressed(object? sender, PointerPressedEventArgs e)
    {
        _isFabDragging = false;
        _fabDragStartPos = e.GetPosition(FabCanvas);
        _fabDragOffset = new Point(Canvas.GetLeft(FabButtonBorder), Canvas.GetTop(FabButtonBorder));
        e.Pointer.Capture(FabButtonBorder);
        e.Handled = true;
    }

    private void OnFabPointerMoved(object? sender, PointerEventArgs e)
    {
        if (FabCanvas == null || FabButtonBorder == null) return;
        if (!e.Pointer.Captured?.Equals(FabButtonBorder) ?? true) return;

        var currentPos = e.GetPosition(FabCanvas);
        double dx = currentPos.X - _fabDragStartPos.X;
        double dy = currentPos.Y - _fabDragStartPos.Y;

        if (!_isFabDragging && (Math.Abs(dx) > FabDragThreshold || Math.Abs(dy) > FabDragThreshold))
        {
            _isFabDragging = true;
            // 开始拖拽时，如果扇形菜单打开则收起
            if (_isFabOpen)
            {
                ToggleFab();
            }
        }

        if (_isFabDragging)
        {
            double newLeft = _fabDragOffset.X + dx;
            double newTop = _fabDragOffset.Y + dy;
            Canvas.SetLeft(FabButtonBorder, newLeft);
            Canvas.SetTop(FabButtonBorder, newTop);
        }
    }

    private void OnFabPointerReleased(object? sender, PointerReleasedEventArgs e)
    {
        if (FabCanvas == null || FabButtonBorder == null) return;
        e.Pointer.Capture(null);

        if (_isFabDragging)
        {
            _fabUserPositioned = true;
            SnapFabToEdge();
            _isFabDragging = false;
        }
        else
        {
            ToggleFab();
        }
        e.Handled = true;
    }

    private void SnapFabToEdge()
    {
        if (FabCanvas == null || FabButtonBorder == null) return;

        double w = FabCanvas.Bounds.Width;
        double h = FabCanvas.Bounds.Height;
        if (w <= 0 || h <= 0) return;

        double fabSize = Resources.TryGetValue("FabSize", out var fs) && fs is double d1 ? d1 : 56;
        double halfFab = fabSize / 2;

        double left = Canvas.GetLeft(FabButtonBorder);
        double top = Canvas.GetTop(FabButtonBorder);
        double centerX = left + halfFab;
        double centerY = top + halfFab;

        // X 吸附到左边缘或右边缘，保留约 1/4 圆在屏内
        double targetX;
        if (centerX < w / 2)
        {
            targetX = -halfFab / 2;
        }
        else
        {
            targetX = w - halfFab * 1.5;
        }

        // Y 限制在安全区域内（顶部留出状态栏余量，底部留出底部面板余量）
        double topMargin = 48;   // 状态栏上方安全区
        double bottomMargin = 8; // 底部间隙
        double minY = topMargin - halfFab;
        double maxY = h - halfFab * 1.5 - bottomMargin;
        double targetY = Math.Clamp(top, minY, maxY);

        Canvas.SetLeft(FabButtonBorder, targetX);
        Canvas.SetTop(FabButtonBorder, targetY);
    }

    // Cached brushes for FAB to avoid GC pressure on frequent toggles
    private static readonly SolidColorBrush FabOpenBrush = new(Color.Parse("#FF6B6B"));
    private static readonly LinearGradientBrush FabClosedBrush = new()
    {
        StartPoint = new RelativePoint(0, 0, RelativeUnit.Relative),
        EndPoint = new RelativePoint(1, 1, RelativeUnit.Relative),
        GradientStops = new GradientStops
        {
            new GradientStop { Offset = 0, Color = Color.Parse("#0A84FF") },
            new GradientStop { Offset = 1, Color = Color.Parse("#005BB5") }
        }
    };

    private void ToggleFab()
    {
        _isFabOpen = !_isFabOpen;
        if (FabButton != null)
        {
            FabButton.Content = _isFabOpen ? "✕" : "调";
        }
        if (FabButtonBorder != null)
        {
            FabButtonBorder.Background = _isFabOpen ? FabOpenBrush : FabClosedBrush;
        }
        PositionFabElements();
    }

    private void ShowDebugTab(int tabIndex)
    {
        if (MobileDebugTabs == null) return;
        MobileDebugTabs.SelectedIndex = tabIndex;
        if (ModalBackdrop != null)
        {
            ModalBackdrop.IsVisible = true;
        }
    }

    private void OnFanCallStackClick(object? sender, global::Avalonia.Interactivity.RoutedEventArgs e)
    {
        ShowDebugTab(0);
    }

    private void OnFanWatchClick(object? sender, global::Avalonia.Interactivity.RoutedEventArgs e)
    {
        ShowDebugTab(1);
    }

    private void OnFanVariablesClick(object? sender, global::Avalonia.Interactivity.RoutedEventArgs e)
    {
        ShowDebugTab(2);
    }

    private void OnFanMemoryClick(object? sender, global::Avalonia.Interactivity.RoutedEventArgs e)
    {
        ShowDebugTab(3);
    }

    private void OnFanArrayClick(object? sender, global::Avalonia.Interactivity.RoutedEventArgs e)
    {
        ShowDebugTab(4);
    }

    private void OnFanPointerClick(object? sender, global::Avalonia.Interactivity.RoutedEventArgs e)
    {
        ShowDebugTab(5);
    }

    private void OnFanKnowledgeClick(object? sender, global::Avalonia.Interactivity.RoutedEventArgs e)
    {
        ShowDebugTab(6);
    }

    private void OnCloseModalClick(object? sender, global::Avalonia.Interactivity.RoutedEventArgs e)
    {
        if (ModalBackdrop != null)
        {
            ModalBackdrop.IsVisible = false;
        }
    }

    private void OnModalBackdropPressed(object? sender, PointerPressedEventArgs e)
    {
        if (ModalBackdrop != null)
        {
            ModalBackdrop.IsVisible = false;
        }
    }

    private void OnModalContentPressed(object? sender, PointerPressedEventArgs e)
    {
        // 阻止事件冒泡，点击内容区不关闭模态面板
        e.Handled = true;
    }

    private void OnTemplateSelected(object? sender, SelectionChangedEventArgs e)
    {
        if (sender is not ComboBox comboBox) return;
        if (comboBox.SelectedItem is not ViewModels.CodeTemplate template) return;

        var codeEditor = this.FindControl<CodeEditor>("CodeEditor");
        if (codeEditor != null)
        {
            codeEditor.InsertTemplate(template.Key);
        }

        comboBox.SelectedIndex = -1;
    }

    private void OnTemplateButtonClick(object? sender, global::Avalonia.Interactivity.RoutedEventArgs e)
    {
        if (sender is not Button button) return;
        if (button.Tag is not string key) return;

        var codeEditor = this.FindControl<CodeEditor>("CodeEditor");
        if (codeEditor != null)
        {
            codeEditor.InsertTemplate(key);
        }
    }

    private void OnTemplateButtonPressed(object? sender, PointerPressedEventArgs e)
    {
        // On Android, ScrollViewer may intercept PointerPressed for scrolling,
        // preventing Button.Click from firing. Handle it directly here.
        if (sender is not Button button) return;
        if (button.Tag is not string key) return;

        var codeEditor = this.FindControl<CodeEditor>("CodeEditor");
        if (codeEditor != null)
        {
            codeEditor.InsertTemplate(key);
        }

        // Stop the event from bubbling to ScrollViewer so it doesn't start scrolling
        e.Handled = true;
    }

    private void OnCallStackFramePressed(object? sender, Avalonia.Input.PointerPressedEventArgs e)
    {
        if (sender is not Border border) return;
        if (border.Tag is not CallStackFrame frame) return;

        if (DataContext is ViewModels.MainViewModel vm)
        {
            vm.JumpToCallStackFrameCommand.Execute(frame);
        }
    }
}
