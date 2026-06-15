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

// ========== 错误码 ==========
/// C-compatible error code enumeration. Keep in sync with native/src/diagnostics/error_codes.rs.

typedef enum {
    CIDE_E1001_UnknownChar        = 1001,
    CIDE_E1002_UnterminatedString = 1002,
    CIDE_E1003_StringCrossLine    = 1003,
    CIDE_E1004_UnsupportedOp      = 1004,
    CIDE_E1005_InvalidDefine      = 1005,
    CIDE_E1006_UnsupportedFeature = 1006,
    CIDE_E1007_ComplexDeclarator = 1007,
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
CIDE_API const char* cide_get_compile_errors(CideSession* s);

// ========== 命令行参数 ==========

/// Set command-line arguments for `main(int argc, char *argv[])`.
CIDE_API void cide_set_argv(CideSession* s, int argc, const char** argv);

// ========== 执行 ==========

/// Run the compiled program. Returns 0 on success, -1 on runtime error.
CIDE_API int cide_run(CideSession* s);

/// Get runtime error message. Returns nullptr if no error.
/// Note: The returned pointer may become invalid after the next run/step call.
CIDE_API const char* cide_get_runtime_error(CideSession* s);

// ========== 输入 ==========

/// Set input lines for scanf (newline-separated lines).
CIDE_API void cide_set_input(CideSession* s, const char* input);

/// Set input mode: 0 = interactive (default), non-zero = batch.
/// In batch mode getchar returns EOF immediately when input is exhausted.
CIDE_API void cide_set_input_mode(CideSession* s, int is_batch);

/// Returns 1 if the program is waiting for input, 0 otherwise.
CIDE_API int cide_is_waiting_input(CideSession* s);

/// Provide a single input line and resume execution.
CIDE_API int cide_provide_input_line(CideSession* s, const char* line);

// ========== 输出 ==========

/// Get the length of the console output.
CIDE_API int cide_get_output_length(CideSession* s);

/// Copy console output into the provided buffer (max_len includes null terminator).
CIDE_API void cide_get_output(CideSession* s, char* buf, int max_len);

#ifdef __cplusplus
}
#endif
