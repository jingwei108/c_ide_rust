namespace Cide.Client.Shared.Core;

/// <summary>
/// Extended vis event with payload (extra0/1/2 from VM).
/// type: 1=Compare, 2=Swap, 3=Update, 4=NodeCreate, 5=EdgeConnect, 6=NodeAccess, 7=NodeDelete
/// </summary>
public readonly record struct VisEventEx(int Type, int Line, int Extra0, int Extra1, int Extra2);

/// <summary>
/// Result of a single step execution.
/// </summary>
public readonly record struct StepResult(
    bool Continue,
    int CurrentLine,
    string Output,
    string? RuntimeError,
    List<VisEventEx> VisEvents,
    bool WaitingInput = false);

/// <summary>
/// Result of a full-speed run.
/// </summary>
public readonly record struct RunResult(
    bool Success,
    string Output,
    string? RuntimeError,
    List<VisEventEx> VisEvents,
    bool WaitingInput = false);

/// <summary>
/// Encapsulates pure execution logic (stepping and full-speed run) against the compiler VM.
/// </summary>
public class ExecutionService
{
    private readonly CompilerService _compiler;

    public ExecutionService(CompilerService compiler)
    {
        _compiler = compiler;
    }

    /// <summary>
    /// Execute a single VM step and return structured results.
    /// </summary>
    public StepResult StepNext()
    {
        bool ok = _compiler.StepNext();
        bool waitingInput = _compiler.IsWaitingInput();

        // Collect vis events (Compare/Swap/Update) with extended payload
        var visEvents = new List<VisEventEx>();
        int visCount = _compiler.GetVisEventCount();
        for (int i = 0; i < visCount; i++)
        {
            var (type, line, e0, e1, e2) = _compiler.GetVisEventEx(i);
            visEvents.Add(new VisEventEx(type, line, e0, e1, e2));
        }
        _compiler.ClearVisEvents();

        string? runtimeError = null;
        string output = "";
        int currentLine = 0;

        if (!ok && !waitingInput)
        {
            runtimeError = _compiler.GetRuntimeError();
            output = _compiler.GetOutput();
        }
        else
        {
            currentLine = _compiler.GetCurrentLine();
            output = _compiler.GetOutput();
        }

        return new StepResult(ok, currentLine, output, runtimeError, visEvents, waitingInput);
    }

    /// <summary>
    /// Run the compiled program at full speed and return structured results.
    /// Also collects any vis events emitted during execution.
    /// </summary>
    public RunResult RunFullSpeed()
    {
        bool ok = _compiler.Run();
        bool waitingInput = _compiler.IsWaitingInput();
        string output = _compiler.GetOutput();
        string? runtimeError = (ok && !waitingInput) ? null : (_compiler.GetRuntimeError() ?? "未知运行时错误");

        // Collect vis events that accumulated during full-speed run
        var visEvents = new List<VisEventEx>();
        int visCount = _compiler.GetVisEventCount();
        for (int i = 0; i < visCount; i++)
        {
            var (type, line, e0, e1, e2) = _compiler.GetVisEventEx(i);
            visEvents.Add(new VisEventEx(type, line, e0, e1, e2));
        }
        _compiler.ClearVisEvents();

        return new RunResult(ok, output, runtimeError, visEvents, waitingInput);
    }
}
