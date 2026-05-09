namespace Cide.Client.ViewModels;

public record CallStackFrame(string FunctionName, int ReturnLine, bool IsCurrentFrame)
    : Cide.Client.Shared.ViewModels.CallStackFrame(FunctionName, ReturnLine, IsCurrentFrame)
{
    public CallStackFrame(Cide.Client.Shared.ViewModels.CallStackFrame f)
        : this(f.FunctionName, f.ReturnLine, f.IsCurrentFrame) { }
}

public record WatchExpression(string Expression, string Value, bool IsValid)
    : Cide.Client.Shared.ViewModels.WatchExpression(Expression, Value, IsValid)
{
    public WatchExpression(Cide.Client.Shared.ViewModels.WatchExpression w)
        : this(w.Expression, w.Value, w.IsValid) { }
}

public record ArrayVisualization(string Name, uint BaseAddress, Cide.Client.Shared.ViewModels.ArrayElementVisual[] Elements)
    : Cide.Client.Shared.ViewModels.ArrayVisualization(Name, BaseAddress, Elements)
{
    public ArrayVisualization(Cide.Client.Shared.ViewModels.ArrayVisualization a)
        : this(a.Name, a.BaseAddress, a.Elements) { }
}

public record ArrayElementVisual(int Value, double HeightPercent, string BackgroundColor, string BorderColor, bool IsFlashing, bool IsCompareHighlight = false)
    : Cide.Client.Shared.ViewModels.ArrayElementVisual(Value, HeightPercent, BackgroundColor, BorderColor, IsFlashing, IsCompareHighlight)
{
    public ArrayElementVisual(Cide.Client.Shared.ViewModels.ArrayElementVisual e)
        : this(e.Value, e.HeightPercent, e.BackgroundColor, e.BorderColor, e.IsFlashing, e.IsCompareHighlight) { }
}

public record PointerViewModel(string VariableName, uint Address, uint TargetAddress, string TargetName)
    : Cide.Client.Shared.ViewModels.PointerViewModel(VariableName, Address, TargetAddress, TargetName)
{
    public PointerViewModel(Cide.Client.Shared.ViewModels.PointerViewModel p)
        : this(p.VariableName, p.Address, p.TargetAddress, p.TargetName) { }
}

public record TraceEntry(int Line, string Operation)
    : Cide.Client.Shared.ViewModels.TraceEntry(Line, Operation)
{
    public TraceEntry(Cide.Client.Shared.ViewModels.TraceEntry t)
        : this(t.Line, t.Operation) { }
}
