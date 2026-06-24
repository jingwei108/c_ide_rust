use super::ErrorInfo;

pub(crate) fn entries() -> [(i32, ErrorInfo); 7] {
    [
        (
            2002,
            ErrorInfo {
                code: 2002,
                emoji: "📐",
                title: "预期数组大小",
                explanation: "声明数组时方括号内需要一个常量大小（如 int arr[5]）。当前子集不支持变量长度数组（VLA）。",
                common_causes: &["数组方括号为空", "用变量当数组大小"],
            },
        ),
        (
            2003,
            ErrorInfo {
                code: 2003,
                emoji: "🧮",
                title: "预期表达式",
                explanation: "这里需要一个表达式（如变量、数字、计算式）。请检查是否遗漏了操作数或写错了运算符。",
                common_causes: &["运算符后面缺少操作数", "括号不匹配导致表达式解析失败"],
            },
        ),
        (
            2004,
            ErrorInfo {
                code: 2004,
                emoji: "🔀",
                title: "预期 case 或 default",
                explanation: "switch 语句内部的标签必须是 case 或 default。请检查是否拼写错误或遗漏了关键字。",
                common_causes: &["case 拼写错误", "switch 内写了普通语句作为标签"],
            },
        ),
        (
            2005,
            ErrorInfo {
                code: 2005,
                emoji: "⏹️",
                title: "预期分号",
                explanation: "C 语言的每条语句末尾都需要分号 ; 作为结束标志。就像中文一句话末尾需要句号。",
                common_causes: &["语句末尾忘记写分号", "上一行的分号写在了注释里"],
            },
        ),
        (
            2006,
            ErrorInfo {
                code: 2006,
                emoji: "🗂️",
                title: "预期右花括号",
                explanation: "代码块由一对花括号 { } 包裹。编译器读到了块的末尾，但没有找到配对的 }。",
                common_causes: &["忘记写右花括号", "花括号嵌套层次过多导致遗漏"],
            },
        ),
        (
            2007,
            ErrorInfo {
                code: 2007,
                emoji: "🗂️",
                title: "预期右圆括号",
                explanation: "圆括号 ( ) 必须成对出现。常见于 if/while/for 的条件表达式或函数调用参数列表。",
                common_causes: &["if/while/for 的条件后缺少 )", "函数调用参数后缺少 )"],
            },
        ),
        (
            2008,
            ErrorInfo {
                code: 2008,
                emoji: "🗂️",
                title: "预期右方括号",
                explanation: "方括号 [ ] 必须成对出现。常见于数组声明和数组索引访问。",
                common_causes: &["数组声明时缺少 ]", "数组索引访问时缺少 ]"],
            },
        ),
    ]
}
