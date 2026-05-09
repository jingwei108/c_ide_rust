namespace Cide.Client.Shared.Core;

/// <summary>
/// Manages the compiler session lifecycle: creation, compilation, breakpoint sync, and disposal.
/// </summary>
public sealed class CompilerSessionService : IDisposable
{
    private CompilerService? _compiler;
    private string? _lastCompiledCode;

    /// <summary>
    /// The current compiler instance, or null if not yet compiled.
    /// </summary>
    public CompilerService? Compiler => _compiler;

    /// <summary>
    /// Ensures the compiler is compiled and ready for the given source code.
    /// Returns true if compilation succeeded, false if compilation failed.
    /// </summary>
    public bool EnsureCompiled(string sourceCode, IEnumerable<int> breakpointLines)
    {
        if (_compiler != null && !_compiler.IsDisposed && _lastCompiledCode == sourceCode)
            return true;

        _compiler?.Dispose();
        _compiler = new CompilerService();
        _lastCompiledCode = null;

        bool ok = _compiler.Compile(sourceCode);
        if (!ok)
            return false;

        _compiler.ClearBreakpoints();
        foreach (int line in breakpointLines)
            _compiler.AddBreakpoint(line);

        _lastCompiledCode = sourceCode;
        return true;
    }

    /// <summary>
    /// Resets the session, disposing the compiler and clearing cached state.
    /// </summary>
    public void Reset()
    {
        _compiler?.Dispose();
        _compiler = null;
        _lastCompiledCode = null;
    }

    public void Dispose()
    {
        Reset();
        GC.SuppressFinalize(this);
    }
}
