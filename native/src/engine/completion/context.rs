use super::*;

pub fn detect_context(source: &str, line: usize, column: usize) -> CompletionContext {
    // 提取光标前最多 200 个字符做上下文分析
    let before = text_before_cursor(source, line, column);
    let trimmed = before.trim();


    // 1. 预处理上下文
    if trimmed.starts_with('#') || trimmed.ends_with('#') {
        return CompletionContext::Preprocessor;
    }

    // 2. 格式字符串上下文
    if let Some(func_name) = detect_format_function(&before) {
        return CompletionContext::FormatString { func_name };
    }

    // 3. 成员访问上下文：识别 `expr.` 或 `expr->`（支持链式如 `a.b.c.`）
    if let Some((expr, is_pointer)) = detect_member_access(&before) {
        return CompletionContext::MemberAccess {
            expr,
            is_pointer,
        };
    }

    // 4. 类型上下文
    if detect_type_position(&before) {
        return CompletionContext::TypePosition;
    }

    // 5. 表达式子上下文
    let hint = detect_expression_hint(&before);
    CompletionContext::Expression { hint }
}

/// 判断名称是否为常见循环变量。
pub fn is_likely_loop_var(name: &str) -> bool {
    matches!(
        name,
        "i" | "j" | "k" | "idx" | "index" | "m" | "n" | "left" | "right" | "mid" | "low"
            | "high" | "pivot" | "gap" | "l" | "r"
    )
}

/// 根据光标前文本推断表达式上下文。
pub fn detect_expression_hint(before: &str) -> ExprHint {
    let trimmed = before.trim();
    // If condition: 最近未闭合的 `if (`
    if trimmed.rfind("if(").is_some() || trimmed.rfind("if (").is_some() {
        return ExprHint::IfCondition;
    }
    // For condition: 最近未闭合的 `for (`
    if trimmed.rfind("for(").is_some() || trimmed.rfind("for (").is_some() {
        return ExprHint::ForCondition;
    }
    // Assignment RHS: 最近有一个 `=` 且没有 `;` / `{` 隔断
    if let Some(eq_pos) = trimmed.rfind('=') {
        let after_eq = &trimmed[eq_pos + 1..];
        if !after_eq.contains(';') && !after_eq.contains('{') {
            return ExprHint::AssignRhs;
        }
    }
    // Malloc arg: 最近未闭合的 `malloc(` / `calloc(` / `realloc(`
    if trimmed.rfind("malloc(").is_some()
        || trimmed.rfind("calloc(").is_some()
        || trimmed.rfind("realloc(").is_some()
    {
        return ExprHint::MallocArg;
    }
    ExprHint::General
}

/// 提取光标前的文本（行内 + 前行末尾）
pub fn text_before_cursor(source: &str, line: usize, column: usize) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut result = String::new();

    // 前一行末尾最多 100 个字符
    if line > 0 {
        let prev = lines.get(line - 1).unwrap_or(&"");
        let start = prev.len().saturating_sub(100);
        result.push_str(&prev[start..]);
        result.push('\n');
    }

    // 当前行光标前
    if let Some(current) = lines.get(line) {
        let col = column.min(current.len());
        result.push_str(&current[..col]);
    }

    result
}

/// 检测是否在 printf/scanf/fprintf 的格式字符串内部
pub fn detect_format_function(before: &str) -> Option<String> {
    // 简单 heuristic：如果前面有 printf(" 或 scanf(" 且没有闭合的 "
    let s = before;
    // 找最后一个未闭合的字符串
    let mut in_string = false;
    for c in s.chars() {
        if c == '"' {
            in_string = !in_string;
        }
    }

    // 更简单的 heuristic：看 before 中是否包含 `printf("...` 且 `"` 未闭合
    for func in ["printf", "scanf", "fprintf"] {
        if let Some(pos) = s.rfind(&format!("{}(\"", func)) {
            let after = &s[pos + func.len() + 2..];
            let quote_count = after.chars().filter(|&c| c == '"').count();
            if quote_count % 2 == 0 {
                // 可能还在字符串内部（简化判断）
                // 更精确：找 after 中下一个 " 前面是否有 \
                let mut in_fmt = true;
                let mut chars = after.chars();
                while let Some(c) = chars.next() {
                    if c == '"' {
                        in_fmt = false;
                        break;
                    }
                    if c == '%' {
                        // 跳过格式说明符
                        let _ = chars.next();
                    }
                }
                if in_fmt {
                    return Some(func.to_string());
                }
            }
        }
    }

    None
}

/// 检测成员访问：`expr.` 或 `expr->`（支持链式如 `a.b.c.`）。
/// 返回 `(expr, is_pointer)`，其中 expr 为点号/箭头前的完整表达式。
pub fn detect_member_access(before: &str) -> Option<(String, bool)> {
    let bytes = before.as_bytes();
    let mut i = bytes.len();

    // 跳过尾部空白
    while i > 0 && bytes[i - 1].is_ascii_whitespace() {
        i -= 1;
    }

    // 检查尾部是 . 还是 ->
    let is_pointer = if i >= 2 && bytes[i - 2] == b'-' && bytes[i - 1] == b'>' {
        i -= 2;
        true
    } else if i > 0 && bytes[i - 1] == b'.' {
        i -= 1;
        false
    } else {
        return None;
    };

    // 跳过 . / -> 前的空白
    while i > 0 && bytes[i - 1].is_ascii_whitespace() {
        i -= 1;
    }

    // 向前扫描，收集完整的链式表达式（允许 identifier、.、->）
    let expr_end = i;
    let mut expect_ident = true;
    loop {
        if expect_ident {
            // 提取 identifier
            let mut start = i;
            while start > 0 {
                let c = bytes[start - 1] as char;
                if c.is_alphanumeric() || c == '_' {
                    start -= 1;
                } else {
                    break;
                }
            }
            if start == i {
                break; // 没有 identifier，结束
            }
            i = start;
            expect_ident = false;
        } else {
            // 期望 . 或 ->
            if i >= 2 && bytes[i - 2] == b'-' && bytes[i - 1] == b'>' {
                i -= 2;
                expect_ident = true;
            } else if i > 0 && bytes[i - 1] == b'.' {
                i -= 1;
                expect_ident = true;
            } else {
                break;
            }
            // 跳过空白
            while i > 0 && bytes[i - 1].is_ascii_whitespace() {
                i -= 1;
            }
        }
    }

    if i < expr_end {
        let expr = String::from_utf8_lossy(&bytes[i..expr_end]).to_string();
        if !expr.is_empty() {
            return Some((expr, is_pointer));
        }
    }

    None
}

/// 检测是否在类型位置（简化 heuristic）
pub fn detect_type_position(before: &str) -> bool {
    let trimmed = before.trim();
    // 以类型关键字结尾
    let type_keywords = [
        "int", "char", "float", "double", "void", "long", "short", "signed", "unsigned",
        "struct", "union", "enum", "const", "static", "extern", "typedef",
    ];
    for kw in &type_keywords {
        if trimmed.ends_with(kw) {
            return true;
        }
    }
    false
}

