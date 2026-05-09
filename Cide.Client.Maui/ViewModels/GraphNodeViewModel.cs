namespace Cide.Client.Maui.ViewModels;

public class GraphNodeViewModel : Cide.Client.Shared.ViewModels.GraphNodeViewModel
{
    public GraphNodeViewModel() { }

    public GraphNodeViewModel(Cide.Client.Shared.ViewModels.GraphNodeViewModel n)
    {
        Address = n.Address;
        Label = n.Label;
        X = n.X;
        Y = n.Y;
        NextAddr = n.NextAddr;
        LeftAddr = n.LeftAddr;
        RightAddr = n.RightAddr;
        IsHighlighted = n.IsHighlighted;
    }
}
