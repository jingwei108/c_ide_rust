namespace Cide.Client.Shared.ViewModels;

/// <summary>
/// Represents a node in a graph or tree visualization (linked list, binary tree, etc.)
/// </summary>
public class GraphNodeViewModel
{
    public uint Address { get; set; }
    public string Label { get; set; } = string.Empty;
    public int X { get; set; }
    public int Y { get; set; }
    public uint? NextAddr { get; set; }
    public uint? LeftAddr { get; set; }
    public uint? RightAddr { get; set; }
    public bool IsHighlighted { get; set; }
    public string BackgroundColor => IsHighlighted ? "#CE9178" : "#2D2D30";
    public string BorderColor => IsHighlighted ? "#F48771" : "#555555";

    /// <summary>Flash color from vis event (NodeCreate/NodeAccess/NodeDelete). Empty = no flash.</summary>
    public string FlashColor { get; set; } = string.Empty;
    public bool IsFlashing => !string.IsNullOrEmpty(FlashColor);
    public string EffectiveBackgroundColor => IsFlashing ? FlashColor : BackgroundColor;
    public string EffectiveBorderColor => IsFlashing ? "#FFFFFF" : BorderColor;
}
