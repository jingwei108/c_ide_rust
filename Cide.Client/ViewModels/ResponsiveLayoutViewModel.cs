using CommunityToolkit.Mvvm.ComponentModel;

namespace Cide.Client.ViewModels;

public enum LayoutBreakpoint
{
    Compact,     // < 600px  -> 手机
    Medium,      // 600~1024px -> 小平板/手机横屏
    Expanded,    // 1024~1280px -> 大平板
    Wide         // > 1280px -> 桌面/大平板横屏
}

public partial class ResponsiveLayoutViewModel : ViewModelBase
{
    [ObservableProperty]
    private LayoutBreakpoint _currentBreakpoint = LayoutBreakpoint.Wide;

    [ObservableProperty]
    private bool _isPortrait = true;

    [ObservableProperty]
    private bool _isPhone = false;

    [ObservableProperty]
    private bool _isTablet = false;

    [ObservableProperty]
    private bool _isDesktop = true;

    [ObservableProperty]
    private bool _isLandscapeTablet = false;

    public void UpdateLayout(double width, double height)
    {
        CurrentBreakpoint = width switch
        {
            < 600 => LayoutBreakpoint.Compact,
            < 1024 => LayoutBreakpoint.Medium,
            < 1280 => LayoutBreakpoint.Expanded,
            _ => LayoutBreakpoint.Wide
        };

        IsPortrait = height > width;
        IsPhone = CurrentBreakpoint == LayoutBreakpoint.Compact;
        IsTablet = CurrentBreakpoint is LayoutBreakpoint.Medium or LayoutBreakpoint.Expanded;
        IsDesktop = CurrentBreakpoint == LayoutBreakpoint.Wide;
        IsLandscapeTablet = !IsPortrait && IsTablet;

        Console.WriteLine($"[CIDE_RESP] UpdateLayout({width:F1}x{height:F1}) -> {CurrentBreakpoint}, IsPhone={IsPhone}, IsTablet={IsTablet}, IsDesktop={IsDesktop}, IsLandscapeTablet={IsLandscapeTablet}");
    }
}
