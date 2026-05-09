#pragma once

namespace cide {

// ============================================================================
// Unified Error Codes
// ============================================================================
// Format: EXYYY where X = phase, YYY = number
// E1xxx = Lexer errors
// E2xxx = Parser errors
// E3xxx = TypeChecker errors
// E4xxx = BytecodeGen errors
// E5xxx = Reserved

enum class ErrorCode : int {
    // Lexer errors (E1xxx)
    E1001_UnknownChar       = 1001,  // 无法识别的字符
    E1002_UnterminatedString= 1002,  // 字符串未闭合
    E1003_StringCrossLine   = 1003,  // 字符串不能跨行
    E1004_UnsupportedOp     = 1004,  // 暂不支持的操作符
    E1005_InvalidDefine     = 1005,  // #define 语法错误

    // Parser errors (E2xxx)
    E2001_ExpectedType      = 2001,  // 预期类型名称
    E2002_ExpectedArraySize = 2002,  // 预期数组大小或 ']'
    E2003_ExpectedExpr      = 2003,  // 预期表达式
    E2004_ExpectedCaseOrDefault = 2004, // 预期 'case' 或 'default'
    E2005_ExpectedSemicolon = 2005,  // 预期 ';'
    E2006_ExpectedClosingBrace = 2006, // 预期 '}'
    E2007_ExpectedClosingParen = 2007, // 预期 ')'
    E2008_ExpectedClosingBracket = 2008, // 预期 ']'

    // TypeChecker errors (E3xxx)
    E3001_VarRedeclared     = 3001,  // 变量重复声明
    E3002_StructRedeclared  = 3002,  // 结构体重复定义
    E3003_FuncRedeclared    = 3003,  // 函数重复定义
    E3004_TypeMismatch      = 3004,  // 类型不匹配
    E3005_ArrayInitTooMany  = 3005,  // 初始化列表元素数量超过数组大小
    E3006_ArrayInitTypeMismatch = 3006, // 数组初始化元素类型不匹配
    E3007_StringInitNonCharArray = 3007, // 字符串字面量只能用于初始化 char 数组
    E3008_StringTooLong     = 3008,  // 字符串字面量长度超过数组大小
    E3009_InvalidArrayInit  = 3009,  // 数组初始化必须使用初始化列表或字符串字面量
    E3010_BreakOutsideLoop  = 3010,  // break 只能在循环或 switch 体内使用
    E3011_ContinueOutsideLoop = 3011, // continue 只能在循环体内使用
    E3012_VoidFuncReturnValue = 3012, // void 函数不能有返回值
    E3013_MissingReturnValue = 3013, // 非 void 函数必须返回一个值
    E3014_ReturnTypeMismatch = 3014, // 返回类型不匹配
    E3015_InvalidCondition  = 3015,  // 条件必须是整数或指针类型
    E3016_ArithmeticTypeError = 3016, // 算术运算类型错误
    E3017_ComparisonTypeError = 3017, // 比较运算类型不兼容
    E3018_RelationTypeError = 3018,  // 关系运算要求 int 类型
    E3019_LogicTypeError    = 3019,  // 逻辑运算要求 int 类型
    E3020_UnaryTypeError    = 3020,  // 一元运算要求 int 类型
    E3021_DerefNonPointer   = 3021,  // 解引用要求指针类型
    E3022_IncDecTypeError   = 3022,  // 自增/自减要求 int 类型
    E3023_UndeclaredVar     = 3023,  // 未声明的变量
    E3024_MallocArgCount    = 3024,  // malloc 参数数量错误
    E3025_MallocArgType     = 3025,  // malloc 参数类型错误
    E3026_FreeArgCount      = 3026,  // free 参数数量错误
    E3027_FreeArgType       = 3027,  // free 参数类型错误
    E3028_BuiltInArgCount   = 3028,  // 内置函数参数数量错误
    E3029_BuiltInArgType    = 3029,  // 内置函数参数类型错误
    E3030_PrintfArgCount    = 3030,  // printf 参数数量错误
    E3031_PrintfFirstArg    = 3031,  // printf 第一个参数必须是字符串
    E3032_PrintfArgType     = 3032,  // printf 参数类型错误
    E3033_ScanfArgCount     = 3033,  // scanf 参数数量错误
    E3034_ScanfFirstArg     = 3034,  // scanf 第一个参数必须是字符串
    E3035_ScanfArgType      = 3035,  // scanf 参数必须是指针
    E3036_UndefinedFunc     = 3036,  // 未定义的函数
    E3037_FuncArgCount      = 3037,  // 函数参数数量不匹配
    E3038_FuncArgType       = 3038,  // 函数参数类型不匹配
    E3039_ArrayIndexType    = 3039,  // 数组索引必须是 int 类型
    E3040_IndexNonArray     = 3040,  // 不能对非数组/指针类型进行索引
    E3041_MemberNonStruct   = 3041,  // '.' 和 '->' 只能用于结构体类型
    E3042_UnknownMember     = 3042,  // 结构体没有该成员
    E3043_AssignToRValue    = 3043,  // 赋值左边必须是可修改的左值
    E3044_AssignTypeMismatch = 3044,// 赋值类型不匹配
    E3045_CompoundAssignType = 3045, // 复合赋值类型错误
    E3046_SwitchCondType    = 3046,  // switch 条件必须是整数类型
    E3047_CaseNotConstant   = 3047,  // case 标签必须是整数常量

    // TypeChecker warnings (W3xxx)
    W3050_AssignInCondition = 3050,  // 条件中使用了赋值运算符，可能是想使用 ==
    W3051_ArrayBoundOffByOne = 3051, // 循环条件可能是 <=，数组访问可能越界

    // BytecodeGen errors (E4xxx)
    E4001_NoMainFunc        = 4001,  // 找不到 main 函数
    E4002_StringTooMany     = 4002,  // 字符串字面量过多，超出内存限制
    E4003_UndeclaredId      = 4003,  // 未声明的标识符
    E4004_AddrNotSupported  = 4004,  // 取地址暂不支持此表达式
    E4005_IncDecNotSupported = 4005, // 自增/自减暂只支持简单变量
    E4006_PrintfTooManyArgs = 4006,  // printf 最多支持 2 个额外参数
    E4007_UndefinedFunc     = 4007,  // 未定义的函数
    E4008_GlobalStructNotSupported = 4008, // 全局结构体暂不支持
    E4009_ComplexStructExpr = 4009,  // 复杂结构体表达式暂不支持
    E4010_CompoundAssignArray = 4010, // 复合赋值暂不支持数组索引
    E4011_CompoundAssignDeref = 4011, // 复合赋值暂不支持指针解引用
    E4012_CompoundAssignMember = 4012,// 复合赋值暂不支持结构体成员
    E4013_AssignTargetUnsupported = 4013, // 赋值目标不支持
    E4014_InitListNotInArray = 4014, // 初始化列表只能在数组声明中使用

    Unknown = 0,
};

} // namespace cide
