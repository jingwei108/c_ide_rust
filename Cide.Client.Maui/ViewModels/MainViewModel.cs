using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using System.Collections.ObjectModel;
using System.Text;
using Cide.Client.Shared.Core;
using Cide.Client.Shared.ViewModels;
using Microsoft.Maui.ApplicationModel;

namespace Cide.Client.Maui.ViewModels;

public partial class MainViewModel : ObservableObject, IDisposable
{
    private bool _disposed;
    public MainViewModel()
    {
        Console.WriteLine("[CIDE_VM] MainViewModel constructor START");
        try
        {
            CurrentKnowledgeCard = new KnowledgeCardViewModel();

            _templates = new ObservableCollection<CodeTemplate>(CodeTemplate.GetDefaultTemplates());
            Console.WriteLine($"[CIDE_VM] Templates initialized: {_templates.Count}");
        }
        catch (Exception ex)
        {
            Console.WriteLine($"[CIDE_VM] Constructor FAILED: {ex}");
            throw;
        }
        Console.WriteLine("[CIDE_VM] MainViewModel constructor END");
    }

    [ObservableProperty]
    private bool _isDarkMode = true;

    [ObservableProperty]
    private int _selectedTabIndex = 0;

    [RelayCommand]
    private void NextTab()
    {
        SelectedTabIndex = Math.Min(SelectedTabIndex + 1, 2);
    }

    [RelayCommand]
    private void PreviousTab()
    {
        SelectedTabIndex = Math.Max(SelectedTabIndex - 1, 0);
    }

    [ObservableProperty]
    private bool _isRunning = false;

    [ObservableProperty]
    private string _sourceCode = """
        int main() {
            int sum = 0;
            for (int i = 1; i <= 5; i = i + 1) {
                sum = sum + i;
            }
            printf("%d", sum);
            return sum;
        }
        """;

    [ObservableProperty]
    private string _consoleOutput = "";

    [ObservableProperty]
    private ObservableCollection<string> _errors = new();

    [ObservableProperty]
    private ObservableCollection<Diagnostic> _diagnostics = new();

    [ObservableProperty]
    private ObservableCollection<int> _errorLines = new();

    [ObservableProperty]
    private ObservableCollection<int> _warningLines = new();

    [ObservableProperty]
    private ObservableCollection<TraceEntry> _traceEntries = new();

    [ObservableProperty]
    private ObservableCollection<MemoryRegion> _memoryRegions = new();

    [ObservableProperty]
    private ObservableCollection<AlgorithmMatch> _algorithmMatches = new();

    [ObservableProperty]
    private AlgorithmValidationResult? _currentValidationResult;

    [ObservableProperty]
    private ObservableCollection<VariableSnapshot> _variables = new();

    [ObservableProperty]
    private ObservableCollection<PointerViewModel> _pointerVariables = new();

    [ObservableProperty]
    private ObservableCollection<ArrayVisualization> _arrayVisualizations = new();

    [ObservableProperty]
    private KnowledgeCardViewModel? _currentKnowledgeCard;

    [ObservableProperty]
    private int _currentStepIndex = -1;

    [ObservableProperty]
    private int _highlightedLine = -1;

    [ObservableProperty]
    private string _stepStatusText = "等待执行...";

    [ObservableProperty]
    private ObservableCollection<int> _breakpointLines = new();

    [ObservableProperty]
    private ObservableCollection<CodeTemplate> _templates = new();

    [ObservableProperty]
    private ObservableCollection<CallStackFrame> _callStackFrames = new();

    [ObservableProperty]
    private ObservableCollection<WatchExpression> _watchExpressions = new();

    [ObservableProperty]
    private ObservableCollection<GraphNodeViewModel> _graphNodes = new();

    [ObservableProperty]
    private string _newWatchExpression = "";

    [ObservableProperty]
    private string _inputText = "";

    [ObservableProperty]
    private int _executionSpeed = 0;

    private readonly CompilerSessionService _session = new();
    private CancellationTokenSource? _flashCts;
    private CancellationTokenSource? _runCts;

    private CompilerService Compiler => _session.Compiler!;

    private bool EnsureCompiled()
    {
        bool ok = _session.EnsureCompiled(SourceCode, BreakpointLines);
        if (!ok)
        {
            LoadDiagnostics();
            return false;
        }
        // Load algorithm matches after successful compilation
        var debugService = new DebugDataService(Compiler);
        AlgorithmMatches = new ObservableCollection<AlgorithmMatch>(debugService.LoadAlgorithmMatches());
        return true;
    }

    private void LoadDiagnostics()
    {
        Diagnostics.Clear();
        ErrorLines.Clear();
        WarningLines.Clear();
        CurrentKnowledgeCard = new KnowledgeCardViewModel();
        if (_session.Compiler == null) return;

        var result = DiagnosticService.LoadDiagnostics(Compiler, SourceCode);
        foreach (var d in result.Diagnostics) Diagnostics.Add(d);
        foreach (var el in result.ErrorLines) ErrorLines.Add(el);
        foreach (var wl in result.WarningLines) WarningLines.Add(wl);
        if (result.FirstCard != null)
        {
            result.FirstCard.IsVisible = true;
            CurrentKnowledgeCard = result.FirstCard;
        }
    }

    private readonly Dictionary<string, int[]> _lastArrayValues = new();

    /// <summary>
    /// Cancel all pending animations and immediately snap visual state to final (no flash).
    /// Call this before switching tabs, stopping execution, or loading new data.
    /// </summary>
    private void CancelAllAnimationsAndSnap()
    {
        _flashCts?.Cancel();
        _flashCts?.Dispose();
        _flashCts = null;

        // Snap: immediately rebuild all array visualizations without flash
        for (int idx = 0; idx < ArrayVisualizations.Count; idx++)
        {
            var av = ArrayVisualizations[idx];
            var values = av.Elements.Select(e => e.Value).ToArray();
            var elements = VisualizationService.BuildArrayElements(values, Array.Empty<int>());
            ArrayVisualizations[idx] = new ArrayVisualization(av.Name, av.BaseAddress, elements);
        }
    }

    private async Task ClearFlashAsync(string arrayName, int[] flashIndices, CancellationToken ct)
    {
        try
        {
            await Task.Delay(500, ct);
        }
        catch (OperationCanceledException)
        {
            // Snap is handled by CancelAllAnimationsAndSnap; just exit here.
            return;
        }

        for (int idx = 0; idx < ArrayVisualizations.Count; idx++)
        {
            var av = ArrayVisualizations[idx];
            if (av.Name == arrayName)
            {
                var values = av.Elements.Select(e => e.Value).ToArray();
                var elements = VisualizationService.BuildArrayElements(values, Array.Empty<int>());
                ArrayVisualizations[idx] = new ArrayVisualization(av.Name, av.BaseAddress, elements);
                break;
            }
        }
    }

    private void LoadCallStack()
    {
        CallStackFrames.Clear();
        if (_session.Compiler == null) return;
        var service = new DebugDataService(Compiler);
        foreach (var frame in service.LoadCallStack())
        {
            CallStackFrames.Add(frame);
        }
    }

    [RelayCommand]
    public void JumpToCallStackFrame(CallStackFrame? frame)
    {
        if (frame == null || frame.ReturnLine <= 0) return;
        HighlightedLine = frame.ReturnLine;
    }

    [RelayCommand]
    private void AddWatchExpression()
    {
        if (string.IsNullOrWhiteSpace(NewWatchExpression)) return;
        string value = EvaluateWatchExpression(NewWatchExpression.Trim());
        WatchExpressions.Add(new WatchExpression(NewWatchExpression.Trim(), value, value != "未知表达式" && !value.StartsWith("错误")));
        NewWatchExpression = "";
    }

    [RelayCommand]
    private void RemoveWatchExpression(WatchExpression? expr)
    {
        if (expr == null) return;
        WatchExpressions.Remove(expr);
    }

    private void RefreshWatchExpressions()
    {
        if (_session.Compiler == null) return;
        var service = new DebugDataService(Compiler);
        for (int i = 0; i < WatchExpressions.Count; i++)
        {
            var expr = WatchExpressions[i];
            string value = service.EvaluateWatchExpression(expr.Expression, Variables.ToList());
            WatchExpressions[i] = expr with { Value = value, IsValid = value != "未知表达式" && !value.StartsWith("错误") };
        }
    }

    private string EvaluateWatchExpression(string expr)
    {
        if (_session.Compiler == null) return "未运行";
        var service = new DebugDataService(Compiler);
        return service.EvaluateWatchExpression(expr, Variables.ToList());
    }

    private void LoadVariables(List<VisEventEx>? visEvents = null)
    {
        Variables.Clear();
        PointerVariables.Clear();
        ArrayVisualizations.Clear();
        if (_session.Compiler == null) return;

        var service = new DebugDataService(Compiler);
        var algoMatches = AlgorithmMatches.ToList();
        var result = service.LoadVariables(_lastArrayValues, visEvents, algoMatches);

        foreach (var v in result.Variables) Variables.Add(v);
        foreach (var p in result.Pointers) PointerVariables.Add(p);
        foreach (var a in result.Arrays) ArrayVisualizations.Add(a);

        _lastArrayValues.Clear();
        foreach (var kvp in result.UpdatedArrayValues)
        {
            _lastArrayValues[kvp.Key] = kvp.Value;
        }

        foreach (var (arrayName, flashIndices) in result.FlashRequests)
        {
            CancelAllAnimationsAndSnap();
            _flashCts = new CancellationTokenSource();
            _ = ClearFlashAsync(arrayName, flashIndices, _flashCts.Token);
        }

        LoadLinkedListGraph(visEvents);
    }

    private void LoadLinkedListGraph(List<VisEventEx>? visEvents = null)
    {
        GraphNodes.Clear();
        if (_session.Compiler == null) return;

        var service = new DebugDataService(Compiler);
        var nodes = service.LoadLinkedListGraph(Variables.ToList(), visEvents);
        foreach (var node in nodes)
        {
            GraphNodes.Add(new GraphNodeViewModel(node));
        }
    }

    private int _isRunInProgressFlag = 0;

    private void ResetExecutionState()
    {
        ConsoleOutput = "";
        Errors.Clear();
        Diagnostics.Clear();
        TraceEntries.Clear();
        CurrentStepIndex = -1;
        HighlightedLine = -1;
        IsRunning = true;
    }

    private void FinishExecution()
    {
        IsRunning = false;
        System.Threading.Interlocked.Exchange(ref _isRunInProgressFlag, 0);
        RunCodeCommand.NotifyCanExecuteChanged();
        _runCts?.Dispose();
        _runCts = null;
    }

    private const int MaxConsoleOutputLength = 50000; // ~50KB

    private string TruncateOutput(string output)
    {
        if (output.Length <= MaxConsoleOutputLength) return output;
        return output.Substring(0, MaxConsoleOutputLength) + "\n... [输出过长，已截断]";
    }

    [RelayCommand(CanExecute = nameof(CanRunCodeAsync))]
    public async Task RunCodeAsync()
    {
        if (System.Threading.Interlocked.CompareExchange(ref _isRunInProgressFlag, 1, 0) != 0)
            return;

        RunCodeCommand.NotifyCanExecuteChanged();
        ResetExecutionState();

        try
        {
            // Run compilation and execution on background thread to avoid blocking UI
            await Task.Run(async () =>
            {
                if (!EnsureCompiled())
                {
                    await MainThread.InvokeOnMainThreadAsync(() =>
                    {
                        PresentCompileError(Compiler.GetCompileErrors() ?? "未知编译错误", setIsRunning: true);
                    });
                    return;
                }

                Compiler.ClearBreakpoints();
                foreach (int bpLine in BreakpointLines)
                {
                    Compiler.AddBreakpoint(bpLine);
                }

                // Set program input for scanf
                Compiler.SetInput(InputText);

                if (ExecutionSpeed <= 0)
                {
                    var execService = new ExecutionService(Compiler);
                    var runResult = execService.RunFullSpeed();
                    if (!runResult.Success)
                    {
                        await MainThread.InvokeOnMainThreadAsync(() =>
                        {
                            PresentRuntimeError(runResult.RuntimeError!, addDiagnostic: true);
                        });
                        return;
                    }

                    var sb = new StringBuilder();
                    sb.AppendLine(runResult.Output);

                    var debugService = new DebugDataService(Compiler);
                    var memRegions = debugService.LoadMemoryRegions();
                    var algoMatches = debugService.LoadAlgorithmMatches();
                    var visEvents = runResult.VisEvents;

                    // Append vis event summary for full-speed run
                    if (visEvents.Count > 0)
                    {
                        sb.AppendLine($"\n--- 执行轨迹 ({visEvents.Count} 个事件) ---");
                        foreach (var ev in visEvents)
                        {
                            string evName = ev.Type == 1 ? "🔍 比较" : ev.Type == 2 ? "🔃 交换" : "📝 更新";
                            sb.AppendLine($"[{evName}] 第 {ev.Line} 行");
                        }
                    }

                    string output = TruncateOutput(sb.ToString());

                    await MainThread.InvokeOnMainThreadAsync(() =>
                    {
                        MemoryRegions.Clear();
                        foreach (var region in memRegions) MemoryRegions.Add(region);
                        AlgorithmMatches = new ObservableCollection<AlgorithmMatch>(algoMatches);
                        LoadVariables(visEvents);
                        LoadCallStack();
                        ConsoleOutput = output;
                    });
                }
                else
                {
                    _runCts?.Dispose();
                    _runCts = new CancellationTokenSource();
                    while (IsRunning)
                    {
                        if (!DoSingleStep()) break;
                        await Task.Delay(ExecutionSpeed, _runCts.Token);
                    }
                }
            });
        }
        catch (Exception ex)
        {
            ConsoleOutput = "异常：" + ex.Message;
            Errors.Add(ex.Message);
        }
        finally
        {
            FinishExecution();
        }
    }

    public bool CanRunCodeAsync => System.Threading.Interlocked.CompareExchange(ref _isRunInProgressFlag, 0, 0) == 0;

    private bool DoSingleStep()
    {
        try
        {
            var execService = new ExecutionService(Compiler);
            var result = execService.StepNext();

            if (!result.Continue)
            {
                if (!string.IsNullOrEmpty(result.RuntimeError))
                {
                    PresentRuntimeError(result.RuntimeError, addDiagnostic: false);
                }
                else
                {
                    ConsoleOutput = TruncateOutput("程序执行完成。\n" + result.Output);
                    HighlightedLine = -1;
                    StepStatusText = "程序执行完成";
                    IsRunning = false;
                }
                return false;
            }

            int line = result.CurrentLine;
            HighlightedLine = line;
            StepStatusText = $"当前行: {line}";

            var sb = new StringBuilder();
            sb.Append(result.Output);

            var debugService = new DebugDataService(Compiler);
            MemoryRegions.Clear();
            foreach (var region in debugService.LoadMemoryRegions()) MemoryRegions.Add(region);
            LoadVariables(result.VisEvents);
            LoadCallStack();
            RefreshWatchExpressions();

            foreach (var ev in result.VisEvents)
            {
                string evName = ev.Type == 1 ? "🔍 比较" : ev.Type == 2 ? "🔃 交换" : "📝 更新";
                sb.AppendLine($"[{evName}] 第 {ev.Line} 行");
            }
            sb.AppendLine($"[单步] 第 {line} 行");
            ConsoleOutput = TruncateOutput(sb.ToString());
            IsRunning = true;
            return true;
        }
        catch (Exception ex)
        {
            ConsoleOutput = "异常：" + ex.Message;
            Errors.Add(ex.Message);
            IsRunning = false;
            return false;
        }
    }

    [RelayCommand]
    public void StepNext()
    {
        if (!EnsureCompiled())
        {
            PresentCompileError(Compiler.GetCompileErrors() ?? "未知编译错误");
            return;
        }

        Compiler.ClearBreakpoints();
        foreach (int bpLine in BreakpointLines)
        {
            Compiler.AddBreakpoint(bpLine);
        }

        // Set program input for scanf
        Compiler.SetInput(InputText);

        DoSingleStep();
    }

    [RelayCommand]
    public void ApplyFix(Diagnostic diag)
    {
        var result = CodeFixService.TryApplyFix(SourceCode, diag);
        if (result.Applied)
        {
            SourceCode = result.NewSourceCode!;
            ConsoleOutput = result.Message;
            Diagnostics.Clear();
            Errors.Clear();
            _session.Reset();
        }
        else if (!string.IsNullOrEmpty(result.Message))
        {
            ConsoleOutput = result.Message;
        }
    }

    private void PresentCompileError(string err, bool setIsRunning = false)
    {
        ConsoleOutput = "编译错误：\n" + err;
        foreach (var diag in Diagnostics)
        {
            Errors.Add($"[{diag.ErrorCode}] 第{diag.Line}行: {diag.Message}");
        }
        if (Errors.Count == 0)
            Errors.Add(err);
        if (Diagnostics.Count > 0 && Diagnostics[0].Line > 0)
        {
            HighlightedLine = Diagnostics[0].Line;
            StepStatusText = $"错误在第 {Diagnostics[0].Line} 行";
        }
        if (setIsRunning)
            IsRunning = false;
    }

    private void PresentRuntimeError(string err, bool addDiagnostic)
    {
        ConsoleOutput = "运行时错误：\n" + err;
        Errors.Add(err);
        if (addDiagnostic)
        {
            Diagnostics.Add(new Diagnostic
            {
                Line = 0,
                Column = 0,
                ErrorCode = -1,
                Severity = 0,
                Message = err,
                FixSuggestion = ""
            });
        }
        var card = KnowledgeCardViewModel.FromErrorCode(-1, err, "");
        if (card != null)
        {
            card.IsVisible = true;
            CurrentKnowledgeCard = card;
        }
        IsRunning = false;
    }

    [RelayCommand]
    public void ValidateAlgorithm(AlgorithmMatch match)
    {
        CurrentValidationResult = AlgorithmValidator.Validate(SourceCode, match);
        ConsoleOutput = CurrentValidationResult.Value.Message;
    }

    [RelayCommand]
    public void StopExecution()
    {
        IsRunning = false;
        CurrentStepIndex = -1;
        HighlightedLine = -1;
        StepStatusText = "等待执行...";
        CancelAllAnimationsAndSnap();
        _runCts?.Cancel();
        _runCts?.Dispose();
        _runCts = null;
        _session.Reset();
    }

    public void Dispose()
    {
        Dispose(true);
        GC.SuppressFinalize(this);
    }

    protected virtual void Dispose(bool disposing)
    {
        if (_disposed) return;
        if (disposing)
        {
            StopExecution();
            _session.Dispose();
            _flashCts?.Dispose();
            _flashCts = null;
            _runCts?.Dispose();
            _runCts = null;
        }
        _disposed = true;
    }
}
