use super::ErrorInfo;

pub(crate) fn entries() -> [(i32, ErrorInfo); 7] {
    [
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
    ]
}
