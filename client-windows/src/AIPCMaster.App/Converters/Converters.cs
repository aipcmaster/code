// XAML 值转换器：健康分/严重度 → 颜色/文本。

using System.Globalization;
using System.Windows.Data;
using System.Windows.Media;
using AIPCMaster.Core.Diagnosis;

namespace AIPCMaster.App.Converters;

/// 健康分 → 环颜色（≥80 绿 / ≥60 黄 / <60 红）。
public sealed class ScoreColorConverter : IValueConverter
{
    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        int score = value is int i ? i : 100;
        return score >= 80 ? new SolidColorBrush(Color.FromRgb(0x2E, 0xCC, 0x71))
            : score >= 60 ? new SolidColorBrush(Color.FromRgb(0xF1, 0xC4, 0x0F))
            : new SolidColorBrush(Color.FromRgb(0xE7, 0x4C, 0x3C));
    }

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

/// 严重度 → 标签底色。
public sealed class SeverityBrushConverter : IValueConverter
{
    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        return value switch
        {
            Severity.Critical => new SolidColorBrush(Color.FromRgb(0xC0, 0x39, 0x2B)),
            Severity.High => new SolidColorBrush(Color.FromRgb(0xE6, 0x7E, 0x22)),
            Severity.Medium => new SolidColorBrush(Color.FromRgb(0xF1, 0xC4, 0x0F)),
            Severity.Low => new SolidColorBrush(Color.FromRgb(0x27, 0xAE, 0x60)),
            _ => new SolidColorBrush(Color.FromRgb(0x7F, 0x8C, 0x8D)),
        };
    }

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

/// 严重度 → 中文显示。
public sealed class SeverityDisplayConverter : IValueConverter
{
    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
        => value is Severity s ? s.Display() : "";

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

/// 类别 → 中文显示。
public sealed class CategoryDisplayConverter : IValueConverter
{
    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
        => value is Category c ? $"# {c.Display()}" : "";

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

/// 空集合 → Collapsed；非空 → Visible。
public sealed class EmptyCountConverter : IValueConverter
{
    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
        => value is int count && count > 0 ? Visibility.Collapsed : Visibility.Visible;

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

/// bool 空 → Collapsed/Visible。
public sealed class InverseBoolToVisibilityConverter : IValueConverter
{
    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
        => value is true ? Visibility.Collapsed : Visibility.Visible;

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => value is Visibility v && v == Visibility.Collapsed;
}

/// 非空字符串 → Visible；空 → Collapsed。
public sealed class HasTextConverter : IValueConverter
{
    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
        => string.IsNullOrEmpty(value as string) ? Visibility.Collapsed : Visibility.Visible;

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

/// 风险级 → 标签底色。
public sealed class RiskBrushConverter : IValueConverter
{
    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
        => value switch
        {
            Advisor.RiskLevel.High => new SolidColorBrush(Color.FromRgb(0xC0, 0x39, 0x2B)),
            Advisor.RiskLevel.Medium => new SolidColorBrush(Color.FromRgb(0xE6, 0x7E, 0x22)),
            _ => new SolidColorBrush(Color.FromRgb(0x27, 0xAE, 0x60)),
        };

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

/// 风险级 → 中文显示。
public sealed class RiskDisplayConverter : IValueConverter
{
    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
        => value is Advisor.RiskLevel r ? r.Display() : "";

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

/// 动作类型 → 中文显示。
public sealed class ActionDisplayConverter : IValueConverter
{
    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
        => value switch
        {
            Advisor.ActionType.CleanMemory => "内存清理",
            Advisor.ActionType.Startup => "启动项优化",
            Advisor.ActionType.Disk => "磁盘清理",
            Advisor.ActionType.Generic => "安全优化",
            _ => "",
        };

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}

/// 高危需确认提示。
public sealed class ConfirmHintConverter : IValueConverter
{
    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
        => value is true ? "⚠ 高风险动作，执行需确认，将创建系统还原点" : "低风险，可直接执行";

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}