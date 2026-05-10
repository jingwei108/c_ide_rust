using Avalonia;
using Avalonia.Styling;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using System.Collections.ObjectModel;
using System.Text;
using Cide.Client.Shared.Core;
using Cide.Client.Shared.ViewModels;

namespace Cide.Client.ViewModels;

public partial class MainViewModel : ViewModelBase, IDisposable
{
    public MainViewModel()
    {
        Console.WriteLine("[CIDE_VM] MainViewModel constructor START");
        try
        {
            // Ensure CurrentKnowledgeCard is never null to avoid NRE in compiled bindings
            // (MainView.axaml binds to CurrentKnowledgeCard.IsVisible).
            CurrentKnowledgeCard = new KnowledgeCardViewModel();
            Console.WriteLine("[CIDE_VM] CurrentKnowledgeCard initialized");

            // Initialize default code templates
            _templates = new ObservableCollection<CodeTemplate>(
                CodeTemplate.GetDefaultTemplates().Select(t => new CodeTemplate(t)));
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
    private ResponsiveLayoutViewModel _responsive = new();

    [ObservableProperty]
    private bool _isDarkMode = true;

    partial void OnIsDarkModeChanged(bool value)
    {
        if (Application.Current is not null)
        {
            Application.Current.RequestedThemeVariant = value
                ? ThemeVariant.Dark
                : ThemeVariant.Light;
        }
    }

    [RelayCommand]
    private void ToggleTheme()
    {
        IsDarkMode = !IsDarkMode;
    }

    [ObservableProperty]
    private int _selectedTabIndex = 0;

    [RelayCommand]
    private void NextTab()
    {
        // Mobile bottom panel has 3 tabs: Output(0), Diagnostics(1), Algorithm(2)
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
        #include <stdio.h>

        int main() {
            int a;
            int b;
            scanf("%d", &a);
            scanf("%d", &b);
            printf("%d", a + b);
            return 0;
        }
        """;

    [ObservableProperty]
    private string _consoleOutput = "";

    [ObservableProperty]
    private string _inputText = "";

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
    private int _executionSpeed = 0; // 0 = full speed, >0 = animation delay in ms

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
        AlgorithmMatches.Clear();
        var debugService = new DebugDataService(Compiler);
        foreach (var match in debugService.LoadAlgorithmMatches()) AlgorithmMatches.Add(match);
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

        // Rebuild the visualization without flash
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
            CallStackFrames.Add(new CallStackFrame(frame));
        }
    }

    [RelayCommand]
    private void JumpToCallStackFrame(CallStackFrame? frame)
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

    /// <summary>
    /// Evaluate a simple watch expression: variable name, array index arr[i], or pointer dereference *p.
    /// </summary>
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
        foreach (var p in result.Pointers) PointerVariables.Add(new PointerViewModel(p));
        foreach (var a in result.Arrays) ArrayVisualizations.Add(new ArrayVisualization(a));

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

        // Linked list graph visualization
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
    private async Task RunCodeAsync()
    {
        // Guard against concurrent execution (rapid button clicks) using Interlocked
        if (System.Threading.Interlocked.CompareExchange(ref _isRunInProgressFlag, 1, 0) != 0)
            return;

        RunCodeCommand.NotifyCanExecuteChanged();
        ResetExecutionState();

        try
        {
            if (!EnsureCompiled())
            {
                PresentCompileError(Compiler.GetCompileErrors() ?? "未知编译错误", setIsRunning: true);
                return;
            }

            // Sync breakpoints before run
            Compiler.ClearBreakpoints();
            foreach (int bpLine in BreakpointLines)
            {
                Compiler.AddBreakpoint(bpLine);
            }

            // Set program input for scanf
            Compiler.SetInput(InputText);

            if (ExecutionSpeed <= 0)
            {
                // Full speed mode
                var execService = new ExecutionService(Compiler);
                var runResult = execService.RunFullSpeed();
                if (!runResult.Success)
                {
                    PresentRuntimeError(runResult.RuntimeError!, addDiagnostic: true);
                    return;
                }

                var sb = new StringBuilder();
                sb.AppendLine(runResult.Output);

                // Load memory regions, algorithm matches, variables, call stack
                var debugService = new DebugDataService(Compiler);
                MemoryRegions.Clear();
                foreach (var region in debugService.LoadMemoryRegions()) MemoryRegions.Add(region);
                AlgorithmMatches.Clear();
                foreach (var match in debugService.LoadAlgorithmMatches()) AlgorithmMatches.Add(match);
                LoadVariables(runResult.VisEvents);
                LoadCallStack();

                // Append vis event summary for full-speed run
                if (runResult.VisEvents.Count > 0)
                {
                    sb.AppendLine($"\n--- 执行轨迹 ({runResult.VisEvents.Count} 个事件) ---");
                    foreach (var ev in runResult.VisEvents)
                    {
                        string evName = ev.Type == 1 ? "🔍 比较" : ev.Type == 2 ? "🔃 交换" : "📝 更新";
                        sb.AppendLine($"[{evName}] 第 {ev.Line} 行");
                    }
                }

                ConsoleOutput = TruncateOutput(sb.ToString());
            }
            else
            {
                // Animation mode: step through with delay
                _runCts = new CancellationTokenSource();
                while (IsRunning)
                {
                    if (!DoSingleStep()) break;
                    await Task.Delay(ExecutionSpeed, _runCts.Token);
                }
            }
        }
        catch (System.Exception ex)
        {
            ConsoleOutput = TruncateOutput("异常：" + ex.Message);
            Errors.Add(ex.Message);
        }
        finally
        {
            FinishExecution();
        }
    }

    public bool CanRunCodeAsync => System.Threading.Interlocked.CompareExchange(ref _isRunInProgressFlag, 0, 0) == 0;

    /// <summary>
    /// Execute a single step and update UI. Returns false if execution finished or error.
    /// </summary>
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

            // Load memory regions, variables, call stack
            var debugService = new DebugDataService(Compiler);
            MemoryRegions.Clear();
            foreach (var region in debugService.LoadMemoryRegions()) MemoryRegions.Add(region);
            LoadVariables(result.VisEvents);
            LoadCallStack();
            RefreshWatchExpressions();

            // Append VM vis event log
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
        catch (System.Exception ex)
        {
            ConsoleOutput = TruncateOutput("异常：" + ex.Message);
            Errors.Add(ex.Message);
            IsRunning = false;
            return false;
        }
    }

    [RelayCommand]
    private void StepNext()
    {
        if (!EnsureCompiled())
        {
            PresentCompileError(Compiler.GetCompileErrors() ?? "未知编译错误");
            return;
        }

        // Sync breakpoints before step (in case user toggled during pause)
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
    private void ApplyFix(Diagnostic diag)
    {
        var result = CodeFixService.TryApplyFix(SourceCode, diag);
        if (result.Applied)
        {
            SourceCode = result.NewSourceCode!;
            ConsoleOutput = TruncateOutput(result.Message);
            Diagnostics.Clear();
            Errors.Clear();
            _session.Reset();
        }
        else if (!string.IsNullOrEmpty(result.Message))
        {
            ConsoleOutput = TruncateOutput(result.Message);
        }
    }

    private void PresentCompileError(string err, bool setIsRunning = false)
    {
        ConsoleOutput = TruncateOutput("编译错误：\n" + err);
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
        ConsoleOutput = TruncateOutput("运行时错误：\n" + err);
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
    private void ValidateAlgorithm(AlgorithmMatch match)
    {
        CurrentValidationResult = AlgorithmValidator.Validate(SourceCode, match);
        ConsoleOutput = TruncateOutput(CurrentValidationResult.Value.Message);
    }

    [RelayCommand]
    private void StopExecution()
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
        if (disposing)
        {
            StopExecution();
            _session.Dispose();
            _flashCts?.Dispose();
            _flashCts = null;
            _runCts?.Dispose();
            _runCts = null;
        }
    }

    public void UpdateLayout(double width, double height)
    {
        Responsive.UpdateLayout(width, height);
    }
}
