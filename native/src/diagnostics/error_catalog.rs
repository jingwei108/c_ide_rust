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
#[derive(Clone, Copy, Debug)]
pub struct ErrorInfo {
    pub code: i32,
    pub emoji: &'static str,
    pub title: &'static str,
    pub explanation: &'static str,
    pub common_causes: &'static [&'static str],
}

mod cpp;
mod lexer;
mod parser;
mod semantic;

static ERROR_INFO_MAP: LazyLock<HashMap<i32, ErrorInfo>> = LazyLock::new(|| {
    let mut map = HashMap::with_capacity(75);
    for (code, info) in lexer::entries()
        .into_iter()
        .chain(parser::entries())
        .chain(semantic::entries())
        .chain(cpp::entries())
    {
        map.insert(code, info);
    }
    map
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
        4100 => (
            "new 与 delete 必须成对出现：构造函数中 new 的资源应在析构函数中 delete；必要时使用 unique_ptr 自动管理。".to_string(),
            4,
            0,
            0,
            0,
            0,
            String::new(),
        ),
        4101 => (
            "避免返回局部变量的引用；如需返回引用，请确保被引用对象的生命周期覆盖引用的使用范围（如返回成员变量或全局对象）。".to_string(),
            4,
            0,
            0,
            0,
            0,
            String::new(),
        ),
        4102 => (
            "把基类值对象改为基类指针/引用，或使用虚函数与多态；按值传递派生类时会发生对象切片。".to_string(),
            4,
            0,
            0,
            0,
            0,
            String::new(),
        ),
        4103 => (
            "unique_ptr 不能被拷贝，只能被 move；move 后不要再访问原 unique_ptr，也不要手动 delete 其管理的对象。".to_string(),
            4,
            0,
            0,
            0,
            0,
            String::new(),
        ),
        4104 => (
            "std::move 后把源对象视为'已释放'，除非显式重新赋值，否则不要再读取或使用它的原值。".to_string(),
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
