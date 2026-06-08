//! Structured error catalog: metadata and auto-fix generation for every error code.
//!
//! Each entry provides:
//! - Emoji + title (for L1 perceptual layer)
//! - Explanation (for L2 understanding layer)
//! - Common causes (checklist)
//! - Structured fix data (insert / replace / delete coordinates)

use std::collections::HashMap;
use std::sync::LazyLock;

/// Metadata for a single error code.
#[derive(Clone, Copy)]
pub struct ErrorInfo {
    pub code: i32,
    pub emoji: &'static str,
    pub title: &'static str,
    pub explanation: &'static str,
    pub common_causes: &'static [&'static str],
}

static ERROR_INFO_MAP: LazyLock<HashMap<i32, ErrorInfo>> = LazyLock::new(|| {
    HashMap::from([
        (1002, ErrorInfo {
            code: 1002,
            emoji: "💬",
            title: "字符串未闭合",
            explanation: "字符串的双引号只写了一个开头，没有找到配对的结尾。字符串必须在一对双引号之间。",
            common_causes: &["忘记写结尾双引号", "字符串跨行（应使用 \\n 或拼接）"],
        }),
        (1003, ErrorInfo {
            code: 1003,
            emoji: "💬",
            title: "字符串跨行",
            explanation: "C 语言的字符串不能直接在代码中换行。如果需要多行文本，请使用 \\n 转义或分成多个字符串拼接。",
            common_causes: &["字符串字面量中直接按了回车"],
        }),
        (1004, ErrorInfo {
            code: 1004,
            emoji: "🔧",
            title: "不支持的操作符",
            explanation: "检测到可能是逻辑运算符的误写。在 C 语言中，逻辑或写为 ||，逻辑与写为 &&。单个 | 和 & 是位运算符，很少在条件里使用。",
            common_causes: &["把 || 写成了 |", "把 && 写成了 &"],
        }),
        (1005, ErrorInfo {
            code: 1005,
            emoji: "📋",
            title: "无效的宏定义",
            explanation: "#define 宏定义格式不正确。简单宏应该是 #define NAME value 的形式。",
            common_causes: &["宏名称缺失", "宏值包含不支持的语法"],
        }),
        (1006, ErrorInfo {
            code: 1006,
            emoji: "🚧",
            title: "暂不支持的功能",
            explanation: "你使用了当前 C IDE 子集尚未支持的语法。请查阅支持列表，使用替代写法。",
            common_causes: &["使用了当前编译器未实现的 C 特性"],
        }),
        (1007, ErrorInfo {
            code: 1007,
            emoji: "🌀",
            title: "声明过于复杂",
            explanation: "此声明符的嵌套层数超出了当前编译器的直接支持范围。当声明中出现多层括号交叉（如 (*(*fp)[2])(int)），代码会变得难以阅读。建议拆分为 typedef 链。",
            common_causes: &["多层括号与指针交叉嵌套", "函数指针数组直接声明"],
        }),
        (1010, ErrorInfo {
            code: 1010,
            emoji: "💬",
            title: "块注释未闭合",
            explanation: "块注释 /* 开始后没有找到配对的 */。请检查是否遗漏了结束标记。",
            common_causes: &["忘记写 */", "注释嵌套导致不匹配"],
        }),
        (2002, ErrorInfo {
            code: 2002,
            emoji: "📐",
            title: "预期数组大小",
            explanation: "声明数组时方括号内需要一个常量大小（如 int arr[5]）。当前子集不支持变量长度数组（VLA）。",
            common_causes: &["数组方括号为空", "用变量当数组大小"],
        }),
        (2003, ErrorInfo {
            code: 2003,
            emoji: "🧮",
            title: "预期表达式",
            explanation: "这里需要一个表达式（如变量、数字、计算式）。请检查是否遗漏了操作数或写错了运算符。",
            common_causes: &["运算符后面缺少操作数", "括号不匹配导致表达式解析失败"],
        }),
        (2004, ErrorInfo {
            code: 2004,
            emoji: "🔀",
            title: "预期 case 或 default",
            explanation: "switch 语句内部的标签必须是 case 或 default。请检查是否拼写错误或遗漏了关键字。",
            common_causes: &["case 拼写错误", "switch 内写了普通语句作为标签"],
        }),
        (2005, ErrorInfo {
            code: 2005,
            emoji: "⏹️",
            title: "预期分号",
            explanation: "C 语言的每条语句末尾都需要分号 ; 作为结束标志。就像中文一句话末尾需要句号。",
            common_causes: &["语句末尾忘记写分号", "上一行的分号写在了注释里"],
        }),
        (2006, ErrorInfo {
            code: 2006,
            emoji: "🗂️",
            title: "预期右花括号",
            explanation: "代码块由一对花括号 { } 包裹。编译器读到了块的末尾，但没有找到配对的 }。",
            common_causes: &["忘记写右花括号", "花括号嵌套层次过多导致遗漏"],
        }),
        (2007, ErrorInfo {
            code: 2007,
            emoji: "🗂️",
            title: "预期右圆括号",
            explanation: "圆括号 ( ) 必须成对出现。常见于 if/while/for 的条件表达式或函数调用参数列表。",
            common_causes: &["if/while/for 的条件后缺少 )", "函数调用参数后缺少 )"],
        }),
        (2008, ErrorInfo {
            code: 2008,
            emoji: "🗂️",
            title: "预期右方括号",
            explanation: "方括号 [ ] 必须成对出现。常见于数组声明和数组索引访问。",
            common_causes: &["数组声明时缺少 ]", "数组索引访问时缺少 ]"],
        }),
        (3002, ErrorInfo {
            code: 3002,
            emoji: "🔄",
            title: "结构体重复定义",
            explanation: "同一个结构体类型名只能定义一次。如果需要在多处使用，请在开头定义一次，之后直接声明变量。",
            common_causes: &["struct 定义写在头文件并被多次包含", "同一个 .c 文件里重复写 struct"],
        }),
        (3003, ErrorInfo {
            code: 3003,
            emoji: "🔄",
            title: "函数重复定义",
            explanation: "同一个函数名只能有一个定义（函数体）。如果需要提前让编译器知道函数签名，请使用前向声明（只写函数头加分号）。",
            common_causes: &["函数定义写了两次", "忘记用分号结束函数原型声明"],
        }),
        (3004, ErrorInfo {
            code: 3004,
            emoji: "🔀",
            title: "类型不匹配",
            explanation: "赋值或传参时，左右两边的类型不一致。C 语言对类型检查比较严格，int* 和 int 不能直接混用。",
            common_causes: &["把指针赋给了整数变量", "把整数赋给了指针变量", "struct 类型不一致"],
        }),
        (3005, ErrorInfo {
            code: 3005,
            emoji: "📦",
            title: "数组初始化项过多",
            explanation: "大括号里的初始化值数量超过了数组声明的大小。例如 int arr[3] = {1,2,3,4} 就是错误的。",
            common_causes: &["初始化列表里的数字个数比数组大小多"],
        }),
        (3006, ErrorInfo {
            code: 3006,
            emoji: "📦",
            title: "数组初始化类型不匹配",
            explanation: "数组初始化列表中的某个值与数组元素类型不兼容。例如 int 数组里放了一个字符串。",
            common_causes: &["初始化列表中混入了字符串", "类型混用"],
        }),
        (3007, ErrorInfo {
            code: 3007,
            emoji: "📦",
            title: "字符串初始化非字符数组",
            explanation: "只有 char 类型的数组才能用字符串初始化。例如 int arr[] = \"hello\" 是错误的。",
            common_causes: &["非 char 数组用双引号字符串初始化"],
        }),
        (3008, ErrorInfo {
            code: 3008,
            emoji: "📦",
            title: "字符串太长",
            explanation: "用于初始化的字符串长度超过了数组声明的大小。注意字符串末尾还有一个隐含的 \\0 终止符。",
            common_causes: &["char s[5] = \"hello\"（需要至少 6 个字节）"],
        }),
        (3009, ErrorInfo {
            code: 3009,
            emoji: "📦",
            title: "无效的数组初始化",
            explanation: "数组初始化列表格式不正确。请确保使用大括号 {} 包裹初始化值，且值之间用逗号分隔。",
            common_causes: &["初始化列表格式错误", "缺少大括号"],
        }),
        (3010, ErrorInfo {
            code: 3010,
            emoji: "🛑",
            title: "break 在循环外",
            explanation: "break 语句只能用在循环（for/while/do-while）或 switch 语句内部，用于跳出当前结构。",
            common_causes: &["break 写在了循环外面", "花括号不匹配导致 break 跑到了错误的位置"],
        }),
        (3011, ErrorInfo {
            code: 3011,
            emoji: "🛑",
            title: "continue 在循环外",
            explanation: "continue 语句只能用在循环内部，用于跳过本次循环的剩余代码，进入下一次循环。",
            common_causes: &["continue 写在了循环外面"],
        }),
        (3012, ErrorInfo {
            code: 3012,
            emoji: "↩️",
            title: "void 函数返回了值",
            explanation: "void 函数表示'不返回任何值'，所以不能在函数体里写 return 某个值。只能写 return; 或不写。",
            common_causes: &["void 函数里写了 return x;"],
        }),
        (3013, ErrorInfo {
            code: 3013,
            emoji: "↩️",
            title: "非 void 函数缺少返回值",
            explanation: "声明了返回类型（如 int）的函数，必须在所有执行路径上都返回一个对应类型的值。",
            common_causes: &["函数末尾缺少 return", "if 分支里 return 了但 else 分支没有"],
        }),
        (3014, ErrorInfo {
            code: 3014,
            emoji: "↩️",
            title: "返回值类型不匹配",
            explanation: "return 后面的表达式类型与函数声明的返回类型不一致。",
            common_causes: &["函数声明返回 int 但 return 了指针", "忘记写 return 值"],
        }),
        (3015, ErrorInfo {
            code: 3015,
            emoji: "❓",
            title: "条件表达式不合法",
            explanation: "if/while/for 的条件位置需要一个能判断真假的表达式。常见错误是把赋值 = 误写成了比较 ==。",
            common_causes: &["条件中使用了赋值 = 而不是比较 ==", "条件表达式类型不是整数或指针"],
        }),
        (3016, ErrorInfo {
            code: 3016,
            emoji: "🧮",
            title: "算术运算类型错误",
            explanation: "加减乘除等算术运算要求操作数是整数或浮点数类型。指针和结构体不能直接做算术运算（指针只能加减整数）。",
            common_causes: &["对指针使用了 * 或 /", "对结构体使用了算术运算符"],
        }),
        (3017, ErrorInfo {
            code: 3017,
            emoji: "🧮",
            title: "比较运算类型错误",
            explanation: "== 和 != 比较要求两边类型兼容。不能比较指针和整数，也不能比较不相关的结构体类型。",
            common_causes: &["用 == 比较指针和整数", "比较了不兼容的结构体"],
        }),
        (3018, ErrorInfo {
            code: 3018,
            emoji: "🧮",
            title: "关系运算类型错误",
            explanation: "< <= > >= 要求操作数是整数或浮点数（或同类型指针）。不能混用整数和指针进行大小比较。",
            common_causes: &["整数和指针比大小", "结构体比大小"],
        }),
        (3019, ErrorInfo {
            code: 3019,
            emoji: "🧮",
            title: "逻辑运算类型错误",
            explanation: "&& 和 || 要求操作数是整数或指针类型（0 为假，非 0 为真）。不能对结构体或数组使用逻辑运算。",
            common_causes: &["对结构体使用 && 或 ||", "对数组名使用逻辑运算"],
        }),
        (3020, ErrorInfo {
            code: 3020,
            emoji: "🧮",
            title: "单目运算类型错误",
            explanation: "! - ~ 等单目运算符要求操作数是整数或指针（! 和 ~）或整数（-）。",
            common_causes: &["对结构体取反", "对数组使用负号"],
        }),
        (3021, ErrorInfo {
            code: 3021,
            emoji: "📍",
            title: "对非指针解引用",
            explanation: "*p 表示访问指针 p 指向的内存。如果 p 不是指针类型，就不能用 *。",
            common_causes: &["对 int 变量使用 *", "对结构体变量使用 *"],
        }),
        (3022, ErrorInfo {
            code: 3022,
            emoji: "➕",
            title: "自增自减类型错误",
            explanation: "++ 和 -- 只能用于整数变量或指针变量。不能用于结构体、数组名或常量。",
            common_causes: &["对结构体使用 ++", "对数组名使用 ++"],
        }),
        (3023, ErrorInfo {
            code: 3023,
            emoji: "❓",
            title: "变量未声明",
            explanation: "在使用变量之前，必须先声明它的类型（如 int x;）。C 语言不会自动创建变量。",
            common_causes: &["忘记写 int/char 等类型声明", "变量名拼写错误", "变量声明在使用之后且没有前向声明"],
        }),
        (3024, ErrorInfo {
            code: 3024,
            emoji: "🧱",
            title: "malloc 参数个数错误",
            explanation: "malloc 函数只需要一个参数：要分配的字节数。常见写法是 malloc(sizeof(int))。",
            common_causes: &["malloc() 传了太多参数", "漏写了 sizeof"],
        }),
        (3025, ErrorInfo {
            code: 3025,
            emoji: "🧱",
            title: "malloc 参数类型错误",
            explanation: "malloc 的参数应该是整数（表示字节数）。请使用 sizeof 计算大小，如 malloc(sizeof(int) * n)。",
            common_causes: &["把指针传给了 malloc", "传了字符串给 malloc"],
        }),
        (3026, ErrorInfo {
            code: 3026,
            emoji: "🧱",
            title: "free 参数个数错误",
            explanation: "free 函数只需要一个参数：要释放的内存指针。",
            common_causes: &["free() 传了多个参数"],
        }),
        (3027, ErrorInfo {
            code: 3027,
            emoji: "🧱",
            title: "free 参数类型错误",
            explanation: "free 的参数必须是指针类型（由 malloc 返回的地址）。",
            common_causes: &["把整数传给了 free", "传了未初始化的指针给 free"],
        }),
        (3028, ErrorInfo {
            code: 3028,
            emoji: "📞",
            title: "内置函数参数个数错误",
            explanation: "printf/scanf/strlen/strcpy/strcmp 等内置函数有固定的参数个数要求。请查阅它们的正确用法。",
            common_causes: &["printf 缺少格式字符串或参数", "strcpy 缺少 src 或 dest"],
        }),
        (3029, ErrorInfo {
            code: 3029,
            emoji: "📞",
            title: "内置函数参数类型错误",
            explanation: "内置函数的某个参数类型不符合要求。例如 printf 第一个参数必须是字符串，scanf 后面必须传指针。",
            common_causes: &["printf 第一个参数不是字符串", "scanf 参数没有加 &"],
        }),
        (3030, ErrorInfo {
            code: 3030,
            emoji: "🖨️",
            title: "printf 参数个数不匹配",
            explanation: "printf 的格式字符串中的占位符（如 %d %s）数量与实际传入的参数数量不一致。",
            common_causes: &["格式占位符比参数多", "参数比格式占位符多"],
        }),
        (3031, ErrorInfo {
            code: 3031,
            emoji: "🖨️",
            title: "printf 第一个参数必须是字符串",
            explanation: "printf(\"...\", ...) 的第一个参数必须是格式字符串（用双引号包裹）。",
            common_causes: &["printf 第一个参数是变量或数字"],
        }),
        (3032, ErrorInfo {
            code: 3032,
            emoji: "🖨️",
            title: "printf 参数类型不匹配",
            explanation: "格式占位符与实际参数类型不对应。例如 %d 对应 int，%s 对应字符串（char*），%f 对应 float/double。",
            common_causes: &["%d 但传了字符串", "%s 但传了 int", "%f 但传了 int"],
        }),
        (3033, ErrorInfo {
            code: 3033,
            emoji: "⌨️",
            title: "scanf 参数个数不匹配",
            explanation: "scanf 的格式字符串中的占位符数量与实际传入的参数数量不一致。",
            common_causes: &["格式占位符比参数多", "参数比格式占位符多"],
        }),
        (3034, ErrorInfo {
            code: 3034,
            emoji: "⌨️",
            title: "scanf 第一个参数必须是字符串",
            explanation: "scanf(\"...\", ...) 的第一个参数必须是格式字符串。",
            common_causes: &["scanf 第一个参数是变量"],
        }),
        (3035, ErrorInfo {
            code: 3035,
            emoji: "⌨️",
            title: "scanf 参数类型错误",
            explanation: "scanf 的参数必须是变量的地址（加 & 符号）。例如 scanf(\"%d\", &a);。如果参数已经是指针，则不需要 &。",
            common_causes: &["scanf 参数忘记加 &", "类型不匹配（如 %d 但变量是 char）"],
        }),
        (3036, ErrorInfo {
            code: 3036,
            emoji: "📞",
            title: "调用了未定义的函数",
            explanation: "调用函数前，编译器需要知道函数的签名。请在调用前添加函数原型声明（函数头加分号），或将函数定义移到调用之前。",
            common_causes: &["函数定义在使用之后", "函数名拼写错误", "忘记 #include 头文件"],
        }),
        (3037, ErrorInfo {
            code: 3037,
            emoji: "📞",
            title: "函数参数个数不匹配",
            explanation: "调用函数时传入的参数个数与函数声明的参数个数不一致。",
            common_causes: &["参数个数太多", "参数个数太少", "漏写了某个参数"],
        }),
        (3038, ErrorInfo {
            code: 3038,
            emoji: "📞",
            title: "函数参数类型不匹配",
            explanation: "调用函数时传入的参数类型与函数声明的形参类型不兼容。",
            common_causes: &["传了 int 但需要指针", "传了 float 但需要 int", "struct 类型不匹配"],
        }),
        (3039, ErrorInfo {
            code: 3039,
            emoji: "📦",
            title: "数组索引类型错误",
            explanation: "数组下标必须是整数类型（int、char 等）。不能用浮点数或指针作为下标。",
            common_causes: &["数组下标是 float/double", "数组下标是字符串"],
        }),
        (3040, ErrorInfo {
            code: 3040,
            emoji: "📦",
            title: "对非数组使用索引",
            explanation: "只有数组和指针才能用 [ ] 访问元素。普通变量和结构体不能用方括号。",
            common_causes: &["对 int 变量使用了 []", "对结构体使用了 []"],
        }),
        (3041, ErrorInfo {
            code: 3041,
            emoji: "🏗️",
            title: "对非结构体使用成员访问",
            explanation: ". 和 -> 只能用于结构体类型。请检查变量是否为结构体，以及是否该用 ->（指针）或 .（变量）。",
            common_causes: &["对普通变量使用 .", "对数组使用 .", "指针应该用 -> 而不是 ."],
        }),
        (3042, ErrorInfo {
            code: 3042,
            emoji: "🏗️",
            title: "未知的结构体成员",
            explanation: "访问的结构体成员名称不存在。请检查成员名拼写，以及是否使用了正确的结构体类型。",
            common_causes: &["成员名拼写错误", "混淆了不同的结构体类型"],
        }),
        (3043, ErrorInfo {
            code: 3043,
            emoji: "🔒",
            title: "向右值赋值",
            explanation: "赋值号 = 左边必须是可修改的变量（左值）。不能给常量、表达式结果或数组名赋值。",
            common_causes: &["给常量赋值", "给表达式结果赋值（如 a+b=5）", "给数组名赋值"],
        }),
        (3044, ErrorInfo {
            code: 3044,
            emoji: "🔀",
            title: "赋值类型不匹配",
            explanation: "赋值号左右两边的类型不兼容。例如 int 变量不能接收 float* 指针。",
            common_causes: &["整数和指针混用", "不兼容的结构体类型互相赋值"],
        }),
        (3045, ErrorInfo {
            code: 3045,
            emoji: "🔀",
            title: "复合赋值类型错误",
            explanation: "+= -= *= /= 等复合赋值要求操作数类型兼容。",
            common_causes: &["对指针使用了 *= 或 /=", "类型不兼容的复合赋值"],
        }),
        (3046, ErrorInfo {
            code: 3046,
            emoji: "🔀",
            title: "switch 条件类型错误",
            explanation: "switch 的条件表达式必须是整数类型（int、char 等）。",
            common_causes: &["switch 条件使用了浮点数", "switch 条件使用了字符串"],
        }),
        (3047, ErrorInfo {
            code: 3047,
            emoji: "🔢",
            title: "case 标签不是常量",
            explanation: "case 后面的值必须是编译期常量（如数字或枚举值），不能是变量。",
            common_causes: &["case 后面写了变量", "case 后面写了表达式"],
        }),
        (3048, ErrorInfo {
            code: 3048,
            emoji: "🧮",
            title: "位运算类型错误",
            explanation: "& | ^ ~ << >> 是位运算符，要求操作数是整数类型。不能对指针、浮点数或结构体使用位运算。",
            common_causes: &["对浮点数使用位运算", "对指针使用位运算", "混淆了 &（位与）和 &&（逻辑与）"],
        }),
        (3051, ErrorInfo {
            code: 3051,
            emoji: "🔢",
            title: "Off-by-One 错误",
            explanation: "循环条件可能是 <=，导致数组访问越界。大小为 n 的数组，有效下标是 0 ~ n-1。",
            common_causes: &["循环条件写成了 <= 而不是 <", "数组大小和循环边界混淆"],
        }),
        (3052, ErrorInfo {
            code: 3052,
            emoji: "📍",
            title: "数组退化为指针",
            explanation: "数组名在大多数表达式中会自动退化为指向首元素的指针。这可能导致 sizeof 结果与预期不符。",
            common_causes: &["函数参数中数组退化为指针", "sizeof(arr) 在参数中变为 sizeof(int*)"],
        }),
        (3053, ErrorInfo {
            code: 3053,
            emoji: "⚠️",
            title: "隐式标量转换",
            explanation: "赋值时发生了隐式类型转换，可能导致数据截断或精度丢失。例如 int 赋值给 char 会截断高位。",
            common_causes: &["int 赋值给 char（截断）", "float 赋值给 int（丢失小数）"],
        }),
        (3054, ErrorInfo {
            code: 3054,
            emoji: "⚠️",
            title: "整数转指针",
            explanation: "把整数直接转换为指针类型是危险的，因为整数可能不代表有效的内存地址。",
            common_causes: &["(int*)123 这类硬编码地址", "NULL 未用 0 表示"],
        }),
        (3055, ErrorInfo {
            code: 3055,
            emoji: "⚠️",
            title: "void 指针转换",
            explanation: "void* 可以隐式转换为任何指针类型，这是 C 标准允许的，但建议显式转换以增加代码可读性。",
            common_causes: &["malloc 返回值没有显式强制转换"],
        }),
        (3056, ErrorInfo {
            code: 3056,
            emoji: "⚠️",
            title: "无符号类型提示",
            explanation: "当前编译器将 unsigned int 映射为 int，暂不支持完整的无符号语义。请确保数值范围在有符号 int 内。",
            common_causes: &["使用了 unsigned 关键字", "需要无符号运算但未实现"],
        }),
        (3057, ErrorInfo {
            code: 3057,
            emoji: "💡",
            title: "隐式类型提升",
            explanation: "代码中发生了安全的隐式类型转换（如 char → int、int → float 或 void* → 具体指针）。这在 C 语言中是允许的，但如果你希望代码更明确，可以使用显式强制转换。",
            common_causes: &["char 自动提升为 int 参与运算", "int 自动提升为 float 参与运算", "malloc 返回值未显式转换"],
        }),
        (3060, ErrorInfo {
            code: 3060,
            emoji: "💥",
            title: "使用已释放的内存 (Use-After-Free)",
            explanation: "你正在读取或写入一块已经被 free() 的内存。这块内存可能还保留着旧数据，但它已经不再属于你。继续使用它会导致不可预测的行为。",
            common_causes: &["指针在 free 后没有置为 NULL，后续又解引用", "多个指针指向同一块内存，其中一个释放了，另一个还在用"],
        }),
        (3061, ErrorInfo {
            code: 3061,
            emoji: "🔁",
            title: "重复释放内存 (Double-Free)",
            explanation: "同一块内存被 free() 了两次。这会破坏内存管理器的内部数据结构，可能导致程序崩溃或后续分配出错。",
            common_causes: &["free(p) 后没有置 NULL，再次 free(p)", "两个指针指向同一地址，都执行了 free"],
        })
    ])
});

/// Look up human-readable metadata for an error code.
pub fn lookup_error_info(code: i32) -> Option<ErrorInfo> {
    ERROR_INFO_MAP.get(&code).copied()
}

/// Generate structured fix data for a diagnostic.
/// Returns: (fix_suggestion, fix_kind, start_line, start_col, end_line, end_col, replacement_text)
///
/// fix_kind: 0=None, 1=ReplaceText, 2=InsertText, 3=DeleteText, 4=ManualHint
pub fn generate_fix(
    code: i32,
    line: i32,
    column: i32,
    message: &str,
    source_lines: &[&str],
) -> (String, i32, i32, i32, i32, i32, String) {
    let line_idx = (line as usize).saturating_sub(1);
    let line_text = source_lines.get(line_idx).unwrap_or(&"");
    let trimmed_len = line_text.trim_end().len() as i32;

    // Helper: try to find a token in the line and return its byte position.
    // column is 1-based and points *after* the problematic token for Parser errors.
    // For replace operations we need 0-based positions.
    let col0 = (column - 1).max(0) as usize;

    match code {
        // ---- Lexer fixes ----
        1007 => {
            // Complex declarator: manual hint, no automatic replacement
            (
                "建议将复杂声明拆分为 typedef 链：\n1. 先定义函数指针类型\n2. 用类型别名声明变量".to_string(),
                4,
                0,
                0,
                0,
                0,
                String::new(),
            )
        }
        1002 => {
            // Unterminated string: insert closing quote at end of line
            (
                "字符串引号未闭合，建议在行末添加双引号".to_string(),
                2,
                line,
                trimmed_len,
                line,
                trimmed_len,
                "\"".to_string(),
            )
        }
        1004 => {
            // Unsupported op: | -> ||, & -> &&
            // column points after the consumed char, so the char is at col0-1 (0-based before the char).
            // Lexer column semantics: after advance(), column is post-char.
            // We look around the error position for single | or &.
            let mut found_pos = None;
            let mut replacement = String::new();
            if col0 >= 1 {
                let bytes = line_text.as_bytes();
                // Search backwards a few characters for | or &
                for i in (0..=col0.saturating_sub(1)).rev().take(3) {
                    if i + 1 < bytes.len() && bytes[i + 1] == b'|' && bytes[i] != b'|' {
                        found_pos = Some(i);
                        replacement = "||".to_string();
                        break;
                    }
                    if i + 1 < bytes.len() && bytes[i + 1] == b'&' && bytes[i] != b'&' {
                        found_pos = Some(i);
                        replacement = "&&".to_string();
                        break;
                    }
                }
            }
            if let Some(pos) = found_pos {
                (
                    format!(
                        "位运算符 '{}' 在条件中很少使用，建议改为逻辑运算符 '{}'",
                        if replacement == "||" { "|" } else { "&" },
                        replacement
                    ),
                    1,
                    line,
                    pos as i32,
                    line,
                    (pos + 1) as i32,
                    replacement,
                )
            } else {
                (
                    "检测到不支持的操作符，建议检查是否误写 | 或 &".to_string(),
                    4,
                    0,
                    0,
                    0,
                    0,
                    String::new(),
                )
            }
        }

        // ---- Parser fixes ----
        2005 => (
            "语句末尾缺少分号，建议添加 ';'".to_string(),
            2,
            line,
            trimmed_len,
            line,
            trimmed_len,
            ";".to_string(),
        ),
        2006 => (
            "代码块缺少右花括号，建议添加 '}'".to_string(),
            2,
            line,
            trimmed_len,
            line,
            trimmed_len,
            "}".to_string(),
        ),
        2007 => (
            "缺少右圆括号，建议添加 ')'".to_string(),
            2,
            line,
            trimmed_len,
            line,
            trimmed_len,
            ")".to_string(),
        ),
        2008 => (
            "缺少右方括号，建议添加 ']'".to_string(),
            2,
            line,
            trimmed_len,
            line,
            trimmed_len,
            "]".to_string(),
        ),

        // ---- TypeChecker fixes ----
        3013 => (
            "非 void 函数缺少返回值，建议在函数末尾添加 'return 0;'".to_string(),
            2,
            line,
            trimmed_len,
            line,
            trimmed_len,
            "return 0;".to_string(),
        ),
        3023 => ("变量未声明，建议先声明变量再使用".to_string(), 4, 0, 0, 0, 0, String::new()),
        3015 => (
            "条件表达式不合法，建议检查是否误用 '=' 代替 '=='".to_string(),
            4,
            0,
            0,
            0,
            0,
            String::new(),
        ),
        3035 => {
            // Scanf arg type: likely missing &
            if message.contains("&") || message.contains("指针") {
                (
                    "scanf 参数需要传入变量的地址，建议在变量名前添加 '&'".to_string(),
                    4,
                    0,
                    0,
                    0,
                    0,
                    String::new(),
                )
            } else {
                (
                    "scanf 参数类型不匹配，请检查格式符与变量类型".to_string(),
                    4,
                    0,
                    0,
                    0,
                    0,
                    String::new(),
                )
            }
        }
        3036 => (
            "函数未声明，建议在调用前添加函数原型声明".to_string(),
            4,
            0,
            0,
            0,
            0,
            String::new(),
        ),
        3041 => {
            // Member access on non-struct: suggest . <-> -> swap if applicable
            if message.contains("->") {
                (
                    "结构体变量应使用 '.' 而不是 '->'，建议将 '->' 改为 '.'".to_string(),
                    4,
                    0,
                    0,
                    0,
                    0,
                    String::new(),
                )
            } else {
                (
                    "只有结构体类型才能使用成员访问，请检查变量类型".to_string(),
                    4,
                    0,
                    0,
                    0,
                    0,
                    String::new(),
                )
            }
        }
        3043 => (
            "不能给表达式或常量赋值，请确认左侧是可修改的变量".to_string(),
            4,
            0,
            0,
            0,
            0,
            String::new(),
        ),
        3044 => (
            "赋值两边类型不匹配，建议检查类型或使用强制类型转换".to_string(),
            4,
            0,
            0,
            0,
            0,
            String::new(),
        ),

        // ---- Warning fixes ----
        3050 => {
            // Assignment in condition -> ==
            // Try to locate a lone '=' inside the condition on this line.
            if let Some((start, end)) = find_single_equals_in_condition(line_text) {
                (
                    "条件中使用了赋值 =，建议改为比较 ==".to_string(),
                    1,
                    line,
                    start as i32,
                    line,
                    end as i32,
                    "==".to_string(),
                )
            } else {
                (
                    "条件中使用了赋值 =，建议检查是否应使用 ==".to_string(),
                    4,
                    0,
                    0,
                    0,
                    0,
                    String::new(),
                )
            }
        }
        3051 => {
            // Off-by-one: <= -> <
            if let Some(pos) = line_text.find("<=") {
                (
                    "循环条件使用了 <=，可能导致数组越界，建议改为 <".to_string(),
                    1,
                    line,
                    pos as i32,
                    line,
                    (pos + 2) as i32,
                    "<".to_string(),
                )
            } else {
                (
                    "循环条件可能导致数组越界，建议检查边界".to_string(),
                    4,
                    0,
                    0,
                    0,
                    0,
                    String::new(),
                )
            }
        }
        3053 => (
            "隐式类型转换可能导致数据截断，建议显式强制转换".to_string(),
            4,
            0,
            0,
            0,
            0,
            String::new(),
        ),
        3054 => (
            "整数直接转指针可能不安全，请确保地址有效".to_string(),
            4,
            0,
            0,
            0,
            0,
            String::new(),
        ),
        3055 => (
            "void* 转换是允许的，但建议显式写 (int*)malloc(...) 以增强可读性".to_string(),
            4,
            0,
            0,
            0,
            0,
            String::new(),
        ),
        3056 => (
            "unsigned 类型暂映射为 int，请确保数值在有符号范围内".to_string(),
            4,
            0,
            0,
            0,
            0,
            String::new(),
        ),
        3057 => (
            "隐式类型提升是安全的，如需更明确可添加显式强制转换".to_string(),
            4,
            0,
            0,
            0,
            0,
            String::new(),
        ),
        3060 | 3061 => (
            "free(p) 后建议立即执行 p = NULL;，并检查是否还有其他指针指向这块内存。".to_string(),
            4,
            0,
            0,
            0,
            0,
            String::new(),
        ),

        // Default: no fix
        _ => (String::new(), 0, 0, 0, 0, 0, String::new()),
    }
}

/// Find a single '=' (not ==, <=, >=, !=) inside a condition on the line.
/// Returns (start_byte, end_byte) of the '=' if found.
fn find_single_equals_in_condition(line: &str) -> Option<(usize, usize)> {
    let mut in_parens = false;
    let mut prev_char = '\0';
    for (idx, c) in line.char_indices() {
        if c == '(' {
            in_parens = true;
            prev_char = c;
            continue;
        }
        if c == ')' {
            in_parens = false;
            prev_char = c;
            continue;
        }
        if in_parens && c == '=' {
            let next_char = line[idx..].chars().nth(1).unwrap_or('\0');
            if prev_char != '=' && prev_char != '!' && prev_char != '<' && prev_char != '>' && next_char != '=' {
                return Some((idx, idx + c.len_utf8()));
            }
        }
        prev_char = c;
    }
    None
}
