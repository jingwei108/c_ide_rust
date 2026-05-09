using Avalonia;
using Avalonia.Data.Converters;
using Avalonia.Media;
using System;
using System.Globalization;

namespace Cide.Client.ViewModels;

/// <summary>
/// Converts diagnostic severity to a background brush color.
/// 0 = error (dark red), 1 = warning (dark yellow), 2 = hint (dark blue)
/// Adapts to current theme (Dark/Light).
/// </summary>
public class SeverityToBrushConverter : IValueConverter
{
    public static readonly SeverityToBrushConverter Instance = new();

    // Pre-create and freeze brushes to avoid repeated allocations and GC pressure.
    private static readonly SolidColorBrush ErrorDark = CreateBrush("#3C1E1E");
    private static readonly SolidColorBrush ErrorLight = CreateBrush("#FFE5E5");
    private static readonly SolidColorBrush WarningDark = CreateBrush("#3A3A1E");
    private static readonly SolidColorBrush WarningLight = CreateBrush("#FFF8E1");
    private static readonly SolidColorBrush HintDark = CreateBrush("#1E3A3A");
    private static readonly SolidColorBrush HintLight = CreateBrush("#E3F2FD");
    private static readonly SolidColorBrush DefaultDark = CreateBrush("#2D2D30");
    private static readonly SolidColorBrush DefaultLight = CreateBrush("#EFEFEF");

    private static SolidColorBrush CreateBrush(string color)
    {
        return new SolidColorBrush(Color.Parse(color));
    }

    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        bool isDark = Application.Current?.ActualThemeVariant == Avalonia.Styling.ThemeVariant.Dark;

        if (value is int severity)
        {
            return severity switch
            {
                0 => isDark ? ErrorDark : ErrorLight,
                1 => isDark ? WarningDark : WarningLight,
                2 => isDark ? HintDark : HintLight,
                _ => isDark ? DefaultDark : DefaultLight,
            };
        }
        return isDark ? DefaultDark : DefaultLight;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotImplementedException();
    }
}
