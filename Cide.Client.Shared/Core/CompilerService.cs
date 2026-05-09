using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

namespace Cide.Client.Shared.Core;

public record MemoryRegion
{
    public uint Address { get; init; }
    public int Size { get; init; }
    public string Name { get; init; } = "";
    public string Type { get; init; } = "";
    public bool IsHeap { get; init; }
    public bool IsFreed { get; init; }
    public int Value { get; init; }
}

public enum FixKind
{
    None = 0,
    ReplaceText = 1,
    InsertText = 2,
    DeleteText = 3,
    ManualHint = 4
}

public record Diagnostic
{
    public int Line { get; init; }
    public int Column { get; init; }
    public int ErrorCode { get; init; }
    public int Severity { get; init; } // 0=error, 1=warning, 2=hint
    public string Message { get; init; } = "";
    public string FixSuggestion { get; init; } = "";
    public string CodeSnippet { get; init; } = "";

    // Structured fix data (populated when backend provides it)
    public FixKind FixKind { get; init; } = FixKind.None;
    public int ReplaceStartLine { get; init; }
    public int ReplaceStartColumn { get; init; }
    public int ReplaceEndLine { get; init; }
    public int ReplaceEndColumn { get; init; }
    public string ReplacementText { get; init; } = "";
}

public record AlgorithmVisEvent(int Line, int Type, string Context);

public record AlgorithmMatch
{
    public string Name { get; init; } = "";
    public string DisplayName { get; init; } = "";
    public string FuncName { get; init; } = "";
    public int Confidence { get; init; }
    public string Suggestion { get; init; } = "";
    public int Line { get; init; }
    public List<AlgorithmVisEvent> VisEvents { get; init; } = new();
}

public record VariableSnapshot
{
    public string Name { get; init; } = "";
    public string TypeName { get; init; } = "";
    public uint Address { get; init; }
    public bool IsLocal { get; init; }
    public bool IsArray { get; init; }
    public int ArraySize { get; init; }
    public int Value { get; init; }
}

/// <summary>
/// High-level wrapper around the native C IDE compiler and runtime.
/// </summary>
public class CompilerService : IDisposable
{
    private IntPtr _session;
    private volatile int _disposed;

    public bool IsDisposed => _disposed != 0;

    public CompilerService()
    {
        _session = NativeMethods.cide_session_create();
        if (_session == IntPtr.Zero)
            throw new InvalidOperationException("Failed to create C IDE session.");
    }

    /// <summary>
    /// Compile the given C source code.
    /// </summary>
    /// <returns>true if compilation succeeded.</returns>
    public bool Compile(string source)
    {
        EnsureNotDisposed();
        return NativeMethods.cide_compile(_session, source) == 0;
    }

    public string? GetCompileErrors()
    {
        EnsureNotDisposed();
        byte[] buf = new byte[4096];
        int len = NativeMethods.cide_get_compile_errors_buf(_session, buf, buf.Length);
        if (len < 0) return null;
        if (len == 0) return null;
        return Encoding.UTF8.GetString(buf, 0, len);
    }

    public bool Run()
    {
        EnsureNotDisposed();
        return NativeMethods.cide_run(_session) == 0;
    }

    public bool StepNext()
    {
        EnsureNotDisposed();
        return NativeMethods.cide_step_next(_session) == 0;
    }

    public int GetCurrentLine()
    {
        EnsureNotDisposed();
        return NativeMethods.cide_get_current_line(_session);
    }

    public int GetCallStackCount()
    {
        EnsureNotDisposed();
        return NativeMethods.cide_callstack_count(_session);
    }

    public (string name, int line) GetCallStackFrame(int index)
    {
        EnsureNotDisposed();
        byte[] name = new byte[64];
        int line;
        NativeMethods.cide_callstack_get(_session, index, name, name.Length, out line);
        return (Encoding.UTF8.GetString(name).TrimEnd('\0'), line);
    }

    public void AddBreakpoint(int line)
    {
        EnsureNotDisposed();
        NativeMethods.cide_breakpoint_add(_session, line);
    }

    public void RemoveBreakpoint(int line)
    {
        EnsureNotDisposed();
        NativeMethods.cide_breakpoint_remove(_session, line);
    }

    public void ClearBreakpoints()
    {
        EnsureNotDisposed();
        NativeMethods.cide_breakpoint_clear(_session);
    }

    public int GetDiagnosticCount()
    {
        EnsureNotDisposed();
        return NativeMethods.cide_diagnostic_count(_session);
    }

    public Diagnostic GetDiagnostic(int index)
    {
        EnsureNotDisposed();
        int line, column, errorCode, severity;
        byte[] message = new byte[512];
        byte[] fixSuggestion = new byte[512];
        NativeMethods.cide_diagnostic_get(_session, index,
            out line, out column, out errorCode, out severity,
            message, message.Length,
            fixSuggestion, fixSuggestion.Length);

        int fixKind, startLine, startColumn, endLine, endColumn;
        byte[] replacementText = new byte[512];
        NativeMethods.cide_diagnostic_get_fix(_session, index,
            out fixKind, out startLine, out startColumn,
            out endLine, out endColumn,
            replacementText, replacementText.Length);

        return new Diagnostic
        {
            Line = line,
            Column = column,
            ErrorCode = errorCode,
            Severity = severity,
            Message = Encoding.UTF8.GetString(message).TrimEnd('\0'),
            FixSuggestion = Encoding.UTF8.GetString(fixSuggestion).TrimEnd('\0'),
            FixKind = (FixKind)fixKind,
            ReplaceStartLine = startLine,
            ReplaceStartColumn = startColumn,
            ReplaceEndLine = endLine,
            ReplaceEndColumn = endColumn,
            ReplacementText = Encoding.UTF8.GetString(replacementText).TrimEnd('\0')
        };
    }

    public string? GetRuntimeError()
    {
        EnsureNotDisposed();
        byte[] buf = new byte[2048];
        int len = NativeMethods.cide_get_runtime_error_buf(_session, buf, buf.Length);
        if (len < 0) return null;
        if (len == 0) return null;
        return Encoding.UTF8.GetString(buf, 0, len);
    }

    public string GetOutput()
    {
        EnsureNotDisposed();
        int len = NativeMethods.cide_get_output_length(_session);
        if (len <= 0) return string.Empty;

        byte[] buf = new byte[len + 1];
        NativeMethods.cide_get_output(_session, buf, len + 1);
        return Encoding.UTF8.GetString(buf).TrimEnd('\0');
    }

    public void SetInput(string input)
    {
        EnsureNotDisposed();
        NativeMethods.cide_set_input(_session, input);
    }

    public int GetAlgorithmMatchCount()
    {
        EnsureNotDisposed();
        return NativeMethods.cide_algorithm_match_count(_session);
    }

    public AlgorithmMatch GetAlgorithmMatch(int index)
    {
        EnsureNotDisposed();
        byte[] name = new byte[64];
        byte[] displayName = new byte[64];
        byte[] funcName = new byte[64];
        byte[] suggestion = new byte[512];
        int confidence, line;
        NativeMethods.cide_algorithm_match_get(_session, index,
            name, name.Length,
            displayName, displayName.Length,
            funcName, funcName.Length,
            out confidence,
            suggestion, suggestion.Length,
            out line);

        var visEvents = new List<AlgorithmVisEvent>();
        int veCount = NativeMethods.cide_algorithm_match_vis_event_count(_session, index);
        for (int i = 0; i < veCount; i++)
        {
            byte[] ctx = new byte[32];
            int type, veLine;
            NativeMethods.cide_algorithm_match_vis_event_get(_session, index, i,
                out type, out veLine, ctx, ctx.Length);
            visEvents.Add(new AlgorithmVisEvent(veLine, type, Encoding.UTF8.GetString(ctx).TrimEnd('\0')));
        }

        return new AlgorithmMatch
        {
            Name = Encoding.UTF8.GetString(name).TrimEnd('\0'),
            DisplayName = Encoding.UTF8.GetString(displayName).TrimEnd('\0'),
            FuncName = Encoding.UTF8.GetString(funcName).TrimEnd('\0'),
            Confidence = confidence,
            Suggestion = Encoding.UTF8.GetString(suggestion).TrimEnd('\0'),
            Line = line,
            VisEvents = visEvents
        };
    }

    public int GetTraceCount()
    {
        EnsureNotDisposed();
        return NativeMethods.cide_trace_count(_session);
    }

    public (int line, string operation) GetTraceEntry(int index)
    {
        EnsureNotDisposed();
        int line;
        byte[] op = new byte[32];
        NativeMethods.cide_trace_get(_session, index, out line, op, op.Length);
        string operation = Encoding.UTF8.GetString(op).TrimEnd('\0');
        return (line, operation);
    }

    public int GetMemoryRegionCount()
    {
        EnsureNotDisposed();
        return NativeMethods.cide_memory_region_count(_session);
    }

    public MemoryRegion GetMemoryRegion(int index)
    {
        EnsureNotDisposed();
        uint addr;
        int size;
        byte[] name = new byte[64];
        byte[] type = new byte[32];
        int isHeap, isFreed;
        NativeMethods.cide_memory_region_get(_session, index, out addr, out size,
            name, name.Length, type, type.Length, out isHeap, out isFreed);
        return new MemoryRegion
        {
            Address = addr,
            Size = size,
            Name = Encoding.UTF8.GetString(name).TrimEnd('\0'),
            Type = Encoding.UTF8.GetString(type).TrimEnd('\0'),
            IsHeap = isHeap != 0,
            IsFreed = isFreed != 0,
            Value = ReadMemoryValue(addr)
        };
    }

    public int ReadMemoryValue(uint address)
    {
        EnsureNotDisposed();
        int value;
        NativeMethods.cide_memory_get_value(_session, address, out value);
        return value;
    }

    public uint? ReadPointerTarget(uint address)
    {
        EnsureNotDisposed();
        uint target;
        if (NativeMethods.cide_memory_get_pointer_target(_session, address, out target) == 0 && target != 0)
            return target;
        return null;
    }

    public int[] ReadArray(uint baseAddr, int size)
    {
        EnsureNotDisposed();
        int[] result = new int[size];
        for (int i = 0; i < size; i++)
        {
            NativeMethods.cide_memory_get_value(_session, baseAddr + (uint)(i * Constants.IntSize), out result[i]);
        }
        return result;
    }

    public int GetVisEventCount()
    {
        EnsureNotDisposed();
        return NativeMethods.cide_vis_event_count(_session);
    }

    public (int type, int line) GetVisEvent(int index)
    {
        EnsureNotDisposed();
        int type, line;
        NativeMethods.cide_vis_event_get(_session, index, out type, out line);
        return (type, line);
    }

    public (int type, int line, int extra0, int extra1, int extra2) GetVisEventEx(int index)
    {
        EnsureNotDisposed();
        int type, line, extra0, extra1, extra2;
        NativeMethods.cide_vis_event_get_ex(_session, index, out type, out line,
                                            out extra0, out extra1, out extra2);
        return (type, line, extra0, extra1, extra2);
    }

    public void ClearVisEvents()
    {
        EnsureNotDisposed();
        NativeMethods.cide_vis_event_clear(_session);
    }

    public int GetVariableCount()
    {
        EnsureNotDisposed();
        return NativeMethods.cide_variable_count(_session);
    }

    public VariableSnapshot GetVariable(int index)
    {
        EnsureNotDisposed();
        byte[] name = new byte[64];
        uint addr;
        int isLocal, isArray, arraySize, value;
        NativeMethods.cide_variable_get(_session, index,
            name, name.Length,
            out addr,
            out isLocal, out isArray, out arraySize,
            out value);
        byte[] typeName = new byte[64];
        NativeMethods.cide_variable_get_type(_session, index, typeName, typeName.Length);
        return new VariableSnapshot
        {
            Name = Encoding.UTF8.GetString(name).TrimEnd('\0'),
            TypeName = Encoding.UTF8.GetString(typeName).TrimEnd('\0'),
            Address = addr,
            IsLocal = isLocal != 0,
            IsArray = isArray != 0,
            ArraySize = arraySize,
            Value = value
        };
    }

    public string? FindVariableByAddr(uint addr)
    {
        EnsureNotDisposed();
        byte[] name = new byte[64];
        int offset;
        if (NativeMethods.cide_variable_find_by_addr(_session, addr, name, name.Length, out offset) == 0)
        {
            var varName = Encoding.UTF8.GetString(name).TrimEnd('\0');
            if (offset > 0)
                return $"{varName}[{offset / 4}]";
            return varName;
        }
        return null;
    }

    public (string name, int offset)? GetVariableField(int varIndex, int fieldIndex)
    {
        EnsureNotDisposed();
        int offset;
        byte[] name = new byte[64];
        if (NativeMethods.cide_variable_get_field(_session, varIndex, fieldIndex, out offset, name, name.Length) == 0)
        {
            return (Encoding.UTF8.GetString(name).TrimEnd('\0'), offset);
        }
        return null;
    }

    public (int line, int column) SourceMapLookup(uint bytecodeOffset)
    {
        EnsureNotDisposed();
        int line, column;
        int result = NativeMethods.cide_sourcemap_lookup(_session, bytecodeOffset, out line, out column);
        if (result != 0) return (0, 0);
        return (line, column);
    }

    public bool TryReadMemory(uint addr, out int value)
    {
        EnsureNotDisposed();
        value = 0;
        if (NativeMethods.cide_memory_get_value(_session, addr, out value) == 0)
            return true;
        return false;
    }

    private void EnsureNotDisposed()
    {
        if (_disposed != 0) throw new ObjectDisposedException(nameof(CompilerService));
    }

    public void Dispose()
    {
        Dispose(true);
        GC.SuppressFinalize(this);
    }

    protected virtual void Dispose(bool disposing)
    {
        if (Interlocked.Exchange(ref _disposed, 1) == 0)
        {
            NativeMethods.cide_session_destroy(_session);
            _session = IntPtr.Zero;
        }
    }

    ~CompilerService()
    {
        Dispose(false);
    }
}
