namespace Cide.Client.Shared.ViewModels;

public record TraceEntry(int Line, string Operation);

public record PointerViewModel(
    string VariableName,
    uint Address,
    uint TargetAddress,
    string TargetName);

public record ArrayElementVisual(
    int Value,
    double HeightPercent,
    string BackgroundColor,
    string BorderColor,
    bool IsFlashing,
    bool IsCompareHighlight = false);

public record ArrayVisualization(
    string Name,
    uint BaseAddress,
    ArrayElementVisual[] Elements);

public record CallStackFrame(
    string FunctionName,
    int ReturnLine,
    bool IsCurrentFrame);

public record WatchExpression(
    string Expression,
    string Value,
    bool IsValid);
