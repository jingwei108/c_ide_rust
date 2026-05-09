using Avalonia.Data.Converters;
using System;
using System.Globalization;

namespace Cide.Client.ViewModels;

/// <summary>
/// Converts an int to bool. Returns true when the value is greater than the threshold (default 0).
/// </summary>
public class IntToBoolConverter : IValueConverter
{
    public static readonly IntToBoolConverter Instance = new();

    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value is not int i) return false;
        int threshold = 0;
        if (parameter is int p) threshold = p;
        else if (parameter is string s && int.TryParse(s, out var parsed)) threshold = parsed;
        return i > threshold;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotImplementedException();
    }
}
