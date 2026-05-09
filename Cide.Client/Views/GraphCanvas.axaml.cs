using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Shapes;
using Avalonia.Media;
using Avalonia.Interactivity;
using System.Collections.Generic;
using System.Linq;
using Cide.Client.ViewModels;

namespace Cide.Client.Views;

public partial class GraphCanvas : UserControl
{
    public static readonly StyledProperty<IList<GraphNodeViewModel>> NodesProperty =
        AvaloniaProperty.Register<GraphCanvas, IList<GraphNodeViewModel>>(nameof(Nodes), new List<GraphNodeViewModel>());

    public IList<GraphNodeViewModel> Nodes
    {
        get => GetValue(NodesProperty);
        set => SetValue(NodesProperty, value);
    }

    private Canvas _rootCanvas = null!;
    private ItemsControl _nodeItemsControl = null!;

    private static readonly SolidColorBrush EdgeBrushGrey = new(Color.Parse("#808080"));
    private static readonly SolidColorBrush EdgeBrushBlue = new(Color.Parse("#4FC1FF"));

    public GraphCanvas()
    {
        InitializeComponent();
        this.Loaded += OnGraphLoaded;
        this.Unloaded += OnGraphUnloaded;
    }

    private void OnGraphLoaded(object? sender, RoutedEventArgs e)
    {
        _rootCanvas = this.FindControl<Canvas>("RootCanvas")!;
        _nodeItemsControl = this.FindControl<ItemsControl>("NodeItemsControl")!;
        this.PropertyChanged += OnPropertyChanged;
        UpdateEdges();
    }

    private void OnGraphUnloaded(object? sender, RoutedEventArgs e)
    {
        this.PropertyChanged -= OnPropertyChanged;
        this.Loaded -= OnGraphLoaded;
        this.Unloaded -= OnGraphUnloaded;
    }

    private void OnPropertyChanged(object? sender, AvaloniaPropertyChangedEventArgs e)
    {
        if (e.Property == NodesProperty)
        {
            UpdateEdges();
        }
    }

    private void UpdateEdges()
    {
        if (_rootCanvas == null) return;

        // Remove previous edge lines (they are direct children of RootCanvas)
        var oldEdges = _rootCanvas.Children.OfType<Line>().ToList();
        foreach (var line in oldEdges)
        {
            _rootCanvas.Children.Remove(line);
        }

        if (Nodes == null || Nodes.Count == 0) return;

        var nodeMap = Nodes.GroupBy(n => n.Address).ToDictionary(g => g.Key, g => g.First());

        foreach (var node in Nodes)
        {
            DrawEdge(node, node.NextAddr, nodeMap, "#808080");   // list: grey
            DrawEdge(node, node.LeftAddr, nodeMap, "#4FC1FF");   // tree left: blue
            DrawEdge(node, node.RightAddr, nodeMap, "#4FC1FF");  // tree right: blue
        }
    }

    private void DrawEdge(GraphNodeViewModel fromNode, uint? toAddr,
                          Dictionary<uint, GraphNodeViewModel> nodeMap, string color)
    {
        if (toAddr == null || !nodeMap.TryGetValue(toAddr.Value, out var toNode)) return;

        // Source: bottom-center of node (width=64, height=36)
        double x1 = fromNode.X + 32.0;
        double y1 = fromNode.Y + 36.0;

        // Target: top-center of node
        double x2 = toNode.X + 32.0;
        double y2 = toNode.Y;

        var line = new Line
        {
            StartPoint = new Point(x1, y1),
            EndPoint = new Point(x2, y2),
            Stroke = color == "#808080" ? EdgeBrushGrey : EdgeBrushBlue,
            StrokeThickness = 1.5,
            IsHitTestVisible = false
        };

        // Ensure edges are drawn behind nodes
        _rootCanvas.Children.Insert(0, line);
    }
}
