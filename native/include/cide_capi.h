#pragma once

#ifdef __cplusplus
extern "C" {
#endif

#ifdef _WIN32
    #ifdef CIDE_EXPORTS
        #define CIDE_API __declspec(dllexport)
    #else
        #define CIDE_API __declspec(dllimport)
    #endif
#else
    #define CIDE_API __attribute__((visibility("default")))
#endif

// Opaque session handle
typedef struct CideSession CideSession;

// ========== 会话管理 ==========

CIDE_API CideSession* cide_session_create();
CIDE_API void cide_session_destroy(CideSession* s);

/// Save session state (compile units + bytecode + runtime) to a file.
/// Returns 0 on success, -1 on error.
CIDE_API int cide_session_save(CideSession* s, const char* filepath);

/// Load session state from a file. Replaces current state.
/// Returns 0 on success, -1 on error.
CIDE_API int cide_session_load(CideSession* s, const char* filepath);

// ========== 错误码 ==========
/// C-compatible error code enumeration. Keep in sync with native/src/diagnostics/ErrorCodes.hpp.

typedef enum {
    CIDE_E1001_UnknownChar        = 1001,
    CIDE_E1002_UnterminatedString = 1002,
    CIDE_E1003_StringCrossLine    = 1003,
    CIDE_E1004_UnsupportedOp      = 1004,
    CIDE_E1005_InvalidDefine      = 1005,
    CIDE_E1006_UnsupportedFeature = 1006,
    CIDE_E1010_UnterminatedComment = 1010,

    CIDE_E2001_ExpectedType       = 2001,
    CIDE_E2002_ExpectedArraySize  = 2002,
    CIDE_E2003_ExpectedExpr       = 2003,
    CIDE_E2004_ExpectedCaseOrDefault = 2004,
    CIDE_E2005_ExpectedSemicolon  = 2005,
    CIDE_E2006_ExpectedClosingBrace = 2006,
    CIDE_E2007_ExpectedClosingParen = 2007,
    CIDE_E2008_ExpectedClosingBracket = 2008,

    CIDE_E3001_VarRedeclared      = 3001,
    CIDE_E3002_StructRedeclared   = 3002,
    CIDE_E3003_FuncRedeclared     = 3003,
    CIDE_E3004_TypeMismatch       = 3004,
    CIDE_E3005_ArrayInitTooMany   = 3005,
    CIDE_E3006_ArrayInitTypeMismatch = 3006,
    CIDE_E3007_StringInitNonCharArray = 3007,
    CIDE_E3008_StringTooLong      = 3008,
    CIDE_E3009_InvalidArrayInit   = 3009,
    CIDE_E3010_BreakOutsideLoop   = 3010,
    CIDE_E3011_ContinueOutsideLoop = 3011,
    CIDE_E3012_VoidFuncReturnValue = 3012,
    CIDE_E3013_MissingReturnValue = 3013,
    CIDE_E3014_ReturnTypeMismatch = 3014,
    CIDE_E3015_InvalidCondition   = 3015,
    CIDE_E3016_ArithmeticTypeError = 3016,
    CIDE_E3017_ComparisonTypeError = 3017,
    CIDE_E3018_RelationTypeError  = 3018,
    CIDE_E3019_LogicTypeError     = 3019,
    CIDE_E3020_UnaryTypeError     = 3020,
    CIDE_E3021_DerefNonPointer    = 3021,
    CIDE_E3022_IncDecTypeError    = 3022,
    CIDE_E3023_UndeclaredVar      = 3023,
    CIDE_E3024_MallocArgCount     = 3024,
    CIDE_E3025_MallocArgType      = 3025,
    CIDE_E3026_FreeArgCount       = 3026,
    CIDE_E3027_FreeArgType        = 3027,
    CIDE_E3028_BuiltInArgCount    = 3028,
    CIDE_E3029_BuiltInArgType     = 3029,
    CIDE_E3030_PrintfArgCount     = 3030,
    CIDE_E3031_PrintfFirstArg     = 3031,
    CIDE_E3032_PrintfArgType      = 3032,
    CIDE_E3033_ScanfArgCount      = 3033,
    CIDE_E3034_ScanfFirstArg      = 3034,
    CIDE_E3035_ScanfArgType       = 3035,
    CIDE_E3036_UndefinedFunc      = 3036,
    CIDE_E3037_FuncArgCount       = 3037,
    CIDE_E3038_FuncArgType        = 3038,
    CIDE_E3039_ArrayIndexType     = 3039,
    CIDE_E3040_IndexNonArray      = 3040,
    CIDE_E3041_MemberNonStruct    = 3041,
    CIDE_E3042_UnknownMember      = 3042,
    CIDE_E3043_AssignToRValue     = 3043,
    CIDE_E3044_AssignTypeMismatch = 3044,
    CIDE_E3045_CompoundAssignType = 3045,
    CIDE_E3046_SwitchCondType     = 3046,
    CIDE_E3047_CaseNotConstant    = 3047,
    CIDE_E3048_BitOpTypeError     = 3048,
    CIDE_E3049_AssignToConst      = 3049,

    CIDE_W3050_AssignInCondition  = 3050,
    CIDE_W3051_ArrayBoundOffByOne = 3051,
    CIDE_W3052_ArrayToPointerDecay = 3052,
    CIDE_W3053_ImplicitScalarConversion = 3053,
    CIDE_W3054_IntToPointerCast   = 3054,
    CIDE_W3055_VoidPointerCast    = 3055,
    CIDE_W3056_UnsignedToInt      = 3056,
    CIDE_H3057_ImplicitConversionHint = 3057,
} CideErrorCode;

// ========== 编译 ==========

/// Compile C source code. Returns 0 on success, -1 on error.
/// This clears any previously added compile units.
/// Note: The `source` string pointer is only valid for the duration of this call.
CIDE_API int cide_compile(CideSession* s, const char* source);

/// Add a compile unit (multi-file support). Does not compile yet.
CIDE_API int cide_compile_unit(CideSession* s, const char* filename, const char* source);

/// Compile all added units. Returns 0 on success, -1 on error.
CIDE_API int cide_compile_all(CideSession* s);

/// Get compilation errors as a UTF-8 string. Returns nullptr if no errors.
/// Note: The returned pointer may become invalid after the next compile call.
/// Use cide_get_compile_errors_buf for safe copy-out.
CIDE_API const char* cide_get_compile_errors(CideSession* s);

/// Copy compilation errors into the provided buffer.
/// Returns the number of bytes copied (excluding null terminator), or -1 on error.
CIDE_API int cide_get_compile_errors_buf(CideSession* s, char* buf, int max_len);

// ========== 执行 ==========

/// Run the compiled program. Returns 0 on success, -1 on runtime error.
CIDE_API int cide_run(CideSession* s);

/// Execute a single step. Returns 0 on success, -1 if finished or error.
CIDE_API int cide_step_next(CideSession* s);

/// Get the current source line during stepping. Returns 0 if not stepping.
CIDE_API int cide_get_current_line(CideSession* s);

// ========== 调用栈 ==========

/// Get the number of frames in the current call stack.
CIDE_API int cide_callstack_count(CideSession* s);

/// Get a call stack frame by index (0 = oldest/main, count-1 = current).
/// name: function name buffer.
/// line: source line number of the return point (0 if unknown).
CIDE_API void cide_callstack_get(
    CideSession* s, int index,
    char* name, int name_size,
    int* line);

// ========== 断点调试 ==========

/// Add a breakpoint at the given source line.
CIDE_API void cide_breakpoint_add(CideSession* s, int line);

/// Remove a breakpoint at the given source line.
CIDE_API void cide_breakpoint_remove(CideSession* s, int line);

/// Clear all breakpoints.
CIDE_API void cide_breakpoint_clear(CideSession* s);

/// Get runtime error message. Returns nullptr if no error.
/// Note: The returned pointer may become invalid after the next run/step call.
/// Use cide_get_runtime_error_buf for safe copy-out.
CIDE_API const char* cide_get_runtime_error(CideSession* s);

/// Copy runtime error into the provided buffer.
/// Returns the number of bytes copied (excluding null terminator), or -1 on error.
CIDE_API int cide_get_runtime_error_buf(CideSession* s, char* buf, int max_len);

/// Set input lines for scanf (newline-separated lines).
CIDE_API void cide_set_input(CideSession* s, const char* input);

/// Get the number of available input lines.
CIDE_API int cide_input_count(CideSession* s);

// ========== 输出 ==========

/// Get the length of the console output.
CIDE_API int cide_get_output_length(CideSession* s);

/// Copy console output into the provided buffer (max_len includes null terminator).
CIDE_API void cide_get_output(CideSession* s, char* buf, int max_len);

// ========== 内存视图 ==========

/// Get the number of memory regions.
CIDE_API int cide_memory_region_count(CideSession* s);

/// Get information about a memory region by index.
CIDE_API void cide_memory_region_get(
    CideSession* s, int index,
    unsigned int* addr, int* size,
    char* name, int name_size,
    char* type, int type_size,
    int* is_heap, int* is_freed);

/// Read an int32 value from the given memory address.
CIDE_API int cide_memory_get_value(CideSession* s, unsigned int addr, int* out_val);

/// Read the target address of a pointer at the given address.
CIDE_API int cide_memory_get_pointer_target(CideSession* s, unsigned int addr, unsigned int* out_target);

// ========== 诊断与修复 ==========

/// Get the number of diagnostics.
CIDE_API int cide_diagnostic_count(CideSession* s);

/// Get a diagnostic by index.
/// severity: 0=error, 1=warning, 2=hint
CIDE_API void cide_diagnostic_get(
    CideSession* s, int index,
    int* line, int* column, int* error_code, int* severity,
    char* message, int msg_size,
    char* fix_suggestion, int fix_size);

/// Get structured fix data for a diagnostic by index.
/// fix_kind: 0=None, 1=ReplaceText, 2=InsertText, 3=DeleteText, 4=ManualHint
CIDE_API void cide_diagnostic_get_fix(
    CideSession* s, int index,
    int* fix_kind,
    int* start_line, int* start_column,
    int* end_line, int* end_column,
    char* replacement_text, int replacement_size);

/// Lookup source location from bytecode offset.
/// Returns 0 if found, -1 if not.
CIDE_API int cide_sourcemap_lookup(
    CideSession* s, unsigned int bytecode_offset,
    int* out_line, int* out_column);

// ========== 执行轨迹 ==========

/// Get the number of trace entries.
CIDE_API int cide_trace_count(CideSession* s);

/// Get a trace entry by index.
CIDE_API void cide_trace_get(
    CideSession* s, int index,
    int* line, char* operation, int op_size);

// ========== 变量面板 (Stage 3) ==========

/// Get the number of visible variables at the current execution point.
CIDE_API int cide_variable_count(CideSession* s);

/// Get variable information by index.
/// is_array: 0=scalar, 1=array
/// array_size: number of elements (0 if scalar)
CIDE_API void cide_variable_get(
    CideSession* s, int index,
    char* name, int name_size,
    unsigned int* addr,
    int* is_local, int* is_array, int* array_size,
    int* value);

/// Get the type string of a variable (e.g. "int", "struct Node*", "int[5]")
/// Returns the length of the type string, or -1 on error.
CIDE_API int cide_variable_get_type(
    CideSession* s, int index,
    char* type_buf, int type_buf_size);

/// Find the variable name that contains the given address.
/// Returns 0 if found, -1 if not. offset is bytes from variable start.
CIDE_API int cide_variable_find_by_addr(
    CideSession* s, unsigned int addr,
    char* name, int name_size,
    int* offset);

/// Get struct field information by variable index and field index.
/// The variable type must be a struct or pointer-to-struct.
/// Returns 0 on success, -1 if variable/field not found.
/// offset: byte offset of the field within the struct.
/// name: field name buffer (null-terminated).
CIDE_API int cide_variable_get_field(
    CideSession* s, int var_index, int field_index,
    int* offset, char* name, int name_size);

// ========== 运行时可视化事件 (Stage 4) ==========

/// Get the number of pending vis events.
/// type: 1=Compare, 2=Swap, 3=Update
CIDE_API int cide_vis_event_count(CideSession* s);

/// Get a vis event by index.
CIDE_API void cide_vis_event_get(CideSession* s, int index, int* type, int* line);

/// Get a vis event by index (extended with extra payload for graph/tree support).
CIDE_API void cide_vis_event_get_ex(CideSession* s, int index,
                                      int* type, int* line,
                                      int* extra0, int* extra1, int* extra2);

/// Clear all vis events.
CIDE_API void cide_vis_event_clear(CideSession* s);

// ========== 算法模式识别 (Phase 4) ==========

/// Get the number of detected algorithm patterns.
CIDE_API int cide_algorithm_match_count(CideSession* s);

/// Get an algorithm match by index.
CIDE_API void cide_algorithm_match_get(
    CideSession* s, int index,
    char* name, int name_size,
    char* display_name, int display_name_size,
    char* func_name, int func_name_size,
    int* confidence,
    char* suggestion, int suggestion_size,
    int* line);

/// Get the number of vis events for an algorithm match.
CIDE_API int cide_algorithm_match_vis_event_count(CideSession* s, int match_index);

/// Get a vis event for an algorithm match by index.
/// type: 1=Compare, 2=Swap, 3=Update
/// context: index expressions separated by ':', e.g. "j:j+1" or "mid"
CIDE_API void cide_algorithm_match_vis_event_get(
    CideSession* s, int match_index, int event_index,
    int* type, int* line, char* context, int context_size);

#ifdef __cplusplus
}
#endif
