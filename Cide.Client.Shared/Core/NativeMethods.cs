using System;
using System.Runtime.InteropServices;
using System.Text;

namespace Cide.Client.Shared.Core;

/// <summary>
/// P/Invoke declarations for the C IDE native backend (cide_native.dll / .so).
/// </summary>
public static class NativeMethods
{
    private const string LibName = "cide_native";

    // ========== 会话管理 ==========

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr cide_session_create();

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void cide_session_destroy(IntPtr session);

    // ========== 编译 ==========

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_compile(IntPtr session, [MarshalAs(UnmanagedType.LPUTF8Str)] string source);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_get_compile_errors_buf(IntPtr session, byte[] buf, int maxLen);

    // ========== 执行 ==========

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_run(IntPtr session);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_step_next(IntPtr session);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_get_runtime_error_buf(IntPtr session, byte[] buf, int maxLen);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void cide_set_input(IntPtr session, [MarshalAs(UnmanagedType.LPUTF8Str)] string input);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_input_count(IntPtr session);

    // ========== 输出 ==========

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_get_output_length(IntPtr session);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void cide_get_output(IntPtr session, byte[] buf, int maxLen);

    // ========== 内存视图 ==========

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_memory_region_count(IntPtr session);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void cide_memory_region_get(
        IntPtr session, int index,
        out uint addr, out int size,
        byte[] name, int nameSize,
        byte[] type, int typeSize,
        out int isHeap, out int isFreed);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_memory_get_value(IntPtr session, uint addr, out int outVal);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_memory_get_pointer_target(IntPtr session, uint addr, out uint outTarget);

    // ========== 诊断与修复 ==========

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_diagnostic_count(IntPtr session);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void cide_diagnostic_get(
        IntPtr session, int index,
        out int line, out int column, out int errorCode, out int severity,
        byte[] message, int msgSize,
        byte[] fixSuggestion, int fixSize);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void cide_diagnostic_get_fix(
        IntPtr session, int index,
        out int fixKind,
        out int startLine, out int startColumn,
        out int endLine, out int endColumn,
        byte[] replacementText, int replacementSize);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_sourcemap_lookup(
        IntPtr session, uint bytecodeOffset,
        out int outLine, out int outColumn);

    // ========== 执行轨迹 ==========

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_get_current_line(IntPtr session);

    // ========== 调用栈 ==========

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_callstack_count(IntPtr session);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void cide_callstack_get(
        IntPtr session, int index,
        byte[] name, int nameSize,
        out int line);

    // ========== 断点调试 ==========

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void cide_breakpoint_add(IntPtr session, int line);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void cide_breakpoint_remove(IntPtr session, int line);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void cide_breakpoint_clear(IntPtr session);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_trace_count(IntPtr session);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void cide_trace_get(
        IntPtr session, int index, out int line,
        byte[] operation, int opSize);

    // ========== 变量面板 (Stage 3) ==========

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_variable_count(IntPtr session);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void cide_variable_get(
        IntPtr session, int index,
        byte[] name, int nameSize,
        out uint addr,
        out int isLocal, out int isArray, out int arraySize,
        out int value);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_variable_get_type(
        IntPtr session, int index,
        byte[] typeBuf, int typeBufSize);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_variable_find_by_addr(
        IntPtr session, uint addr,
        byte[] name, int nameSize,
        out int offset);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_variable_get_field(
        IntPtr session, int varIndex, int fieldIndex,
        out int offset, byte[] name, int nameSize);

    // ========== 运行时可视化事件 (Stage 4) ==========

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_vis_event_count(IntPtr session);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void cide_vis_event_get(IntPtr session, int index, out int type, out int line);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void cide_vis_event_get_ex(IntPtr session, int index,
                                                      out int type, out int line,
                                                      out int extra0, out int extra1, out int extra2);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void cide_vis_event_clear(IntPtr session);

    // ========== 算法模式识别 (Phase 4) ==========

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_algorithm_match_count(IntPtr session);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void cide_algorithm_match_get(
        IntPtr session, int index,
        byte[] name, int nameSize,
        byte[] displayName, int displayNameSize,
        byte[] funcName, int funcNameSize,
        out int confidence,
        byte[] suggestion, int suggestionSize,
        out int line);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int cide_algorithm_match_vis_event_count(IntPtr session, int matchIndex);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void cide_algorithm_match_vis_event_get(
        IntPtr session, int matchIndex, int eventIndex,
        out int type, out int line,
        byte[] context, int contextSize);

    // ========== 辅助方法 ==========

    public static string? PtrToStringUtf8(IntPtr ptr)
    {
        if (ptr == IntPtr.Zero) return null;
        return Marshal.PtrToStringUTF8(ptr);
    }
}
