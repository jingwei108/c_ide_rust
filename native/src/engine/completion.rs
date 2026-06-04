//! 智能补全引擎 v2
//!
//! 利用现有编译管线的 AST 快照 + 轻量级 Token 增量扫描，
//! 提供语义感知的代码补全：成员访问、类型上下文、表达式上下文、
//! 格式字符串、预处理指令等。

use crate::compiler::ast::ProgramNode;
use crate::compiler::lexer::{Lexer, Token, TokenType};
use crate::session::Session;

// ============================================================================
// 数据类型
// ============================================================================

/// 补全候选（FRB 友好，将在 api/cide.rs 中重新包装）
#[derive(Debug, Clone)]
pub struct CompletionCandidate {
    pub label: String,
    pub kind: CompletionKind,
    pub detail: String,
    pub documentation: String,
    pub insert_text: String,
    pub sort_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Variable,
    Function,
    Struct,
    Union,
    Enum,
    Typedef,
    Field,
    Keyword,
    Macro,
    Snippet,
    FormatSpecifier,
}

impl CompletionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            CompletionKind::Variable => "variable",
            CompletionKind::Function => "function",
            CompletionKind::Struct => "struct",
            CompletionKind::Union => "union",
            CompletionKind::Enum => "enum",
            CompletionKind::Typedef => "typedef",
            CompletionKind::Field => "field",
            CompletionKind::Keyword => "keyword",
            CompletionKind::Macro => "macro",
            CompletionKind::Snippet => "snippet",
            CompletionKind::FormatSpecifier => "format",
        }
    }
}

/// 编译快照：在每次成功编译时从 AST 提取并持久化到 Session
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CompletionSnapshot {
    pub functions: Vec<SnapshotFunc>,
    pub globals: Vec<SnapshotVar>,
    pub structs: Vec<SnapshotStruct>,
    pub unions: Vec<SnapshotStruct>,
    pub typedefs: Vec<SnapshotTypedef>,
    pub enums: Vec<SnapshotEnum>,
    pub macros: Vec<String>, // #define 定义的宏名称
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnapshotFunc {
    pub name: String,
    pub return_type: String,
    pub params: Vec<String>,
    pub is_static: bool,
    pub filename: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnapshotVar {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnapshotStruct {
    pub name: String,
    pub fields: Vec<(String, String)>, // (name, type_string)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnapshotTypedef {
    pub name: String,
    pub underlying: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnapshotEnum {
    pub name: String,
    pub variants: Vec<String>,
}

/// 轻量扫描发现的局部符号
#[derive(Debug, Clone)]
struct LocalSymbol {
    name: String,
    ty: String,
    scope_depth: usize,
    #[allow(dead_code)]
    is_param: bool,
}

/// 补全上下文
#[derive(Debug, Clone)]
enum CompletionContext {
    /// `expr.` 或 `expr->`
    MemberAccess { base_name: String, is_pointer: bool },
    /// 类型位置（如 `int |`, `struct |`）
    TypePosition,
    /// 预处理
    Preprocessor,
    /// printf / scanf 等格式字符串内
    FormatString { func_name: String },
    /// 一般表达式
    Expression,
}

// ============================================================================
// 公开 API
// ============================================================================

/// 从 AST 构建补全快照（在编译管线成功后调用）
pub fn build_snapshot(program: &ProgramNode) -> CompletionSnapshot {
    build_snapshot_impl(program)
}

/// 从当前源码实时解析提取符号快照
///
/// **利用自研 Parser 的错误恢复能力**：即使源码不完整/含语法错误，
/// 自研 Parser 的 `synchronize()` + checkpoint/rollback 机制仍会返回
/// 包含已成功解析部分的 `ProgramNode`。我们从中提取 struct/union/
/// function/global 符号，作为编译快照的实时补充。
///
/// 这比纯 Token 扫描更精确，因为 Parser 能正确识别嵌套类型、
/// 函数签名、数组维度等复杂语法结构。
pub fn build_snapshot_from_source(source: &str) -> CompletionSnapshot {
    let (tokens, _) = Lexer::new(source).tokenize();
    let (program, _) = crate::compiler::parser::Parser::new(tokens).parse();
    match program {
        Some(p) => build_snapshot_impl(&p),
        None => CompletionSnapshot::default(),
    }
}

fn build_snapshot_impl(program: &ProgramNode) -> CompletionSnapshot {
    let mut snapshot = CompletionSnapshot::default();

    // 结构体 / 联合体
    for s in &program.structs {
        let fields: Vec<(String, String)> = s
            .fields
            .iter()
            .map(|f| (f.name.clone(), f.ty.to_string()))
            .collect();
        snapshot.structs.push(SnapshotStruct {
            name: s.name.clone(),
            fields,
        });
    }
    for u in &program.unions {
        let fields: Vec<(String, String)> = u
            .fields
            .iter()
            .map(|f| (f.name.clone(), f.ty.to_string()))
            .collect();
        snapshot.unions.push(SnapshotStruct {
            name: u.name.clone(),
            fields,
        });
    }

    // 全局变量
    for g in &program.globals {
        snapshot.globals.push(SnapshotVar {
            name: g.name.clone(),
            ty: g.ty.to_string(),
        });
    }

    // 函数（含参数签名）
    for f in &program.funcs {
        let params: Vec<String> = f
            .params
            .iter()
            .map(|p| format!("{} {}", p.ty, p.name))
            .collect();
        snapshot.functions.push(SnapshotFunc {
            name: f.name.clone(),
            return_type: f.return_type.to_string(),
            params,
            is_static: f.is_static,
            filename: f.source_file.clone(),
        });
    }

    // 枚举（从 globals 中提取 enum 类型，目前 ast 中没有独立 enum decl，
    // 但 parser 会把 enum 当作 int typedef；我们扫描全局变量中的 enum 声明）
    // 注意：Parser 会把 `enum Color { RED, GREEN }` 解析为 typedef int Color + 全局常量
    // 因此这里不单独处理 enum 定义，而是在 scan_local_macros 中处理 #define

    snapshot
}

/// 合并两个快照：以 `compiled` 为优先，`live` 补充缺失的符号
fn merge_snapshots(
    compiled: &CompletionSnapshot,
    live: &CompletionSnapshot,
) -> CompletionSnapshot {
    let mut merged = compiled.clone();

    // 用 HashSet 去重（基于名称）
    let mut func_names: std::collections::HashSet<String> =
        merged.functions.iter().map(|f| f.name.clone()).collect();
    for f in &live.functions {
        if func_names.insert(f.name.clone()) {
            merged.functions.push(f.clone());
        }
    }

    let mut global_names: std::collections::HashSet<String> =
        merged.globals.iter().map(|g| g.name.clone()).collect();
    for g in &live.globals {
        if global_names.insert(g.name.clone()) {
            merged.globals.push(g.clone());
        }
    }

    let mut struct_names: std::collections::HashSet<String> =
        merged.structs.iter().map(|s| s.name.clone()).collect();
    for s in &live.structs {
        if struct_names.insert(s.name.clone()) {
            merged.structs.push(s.clone());
        }
    }

    let mut union_names: std::collections::HashSet<String> =
        merged.unions.iter().map(|u| u.name.clone()).collect();
    for u in &live.unions {
        if union_names.insert(u.name.clone()) {
            merged.unions.push(u.clone());
        }
    }

    // typedef / enum / macros：同样 live 补充
    let mut typedef_names: std::collections::HashSet<String> =
        merged.typedefs.iter().map(|t| t.name.clone()).collect();
    for t in &live.typedefs {
        if typedef_names.insert(t.name.clone()) {
            merged.typedefs.push(t.clone());
        }
    }

    let mut enum_names: std::collections::HashSet<String> =
        merged.enums.iter().map(|e| e.name.clone()).collect();
    for e in &live.enums {
        if enum_names.insert(e.name.clone()) {
            merged.enums.push(e.clone());
        }
    }

    let mut macro_names: std::collections::HashSet<String> =
        merged.macros.iter().cloned().collect();
    for m in &live.macros {
        if macro_names.insert(m.clone()) {
            merged.macros.push(m.clone());
        }
    }

    merged
}

/// 获取光标位置的补全候选
///
/// # 参数
/// - `session`: 当前 Session，用于读取编译快照
/// - `source`: 当前文件完整源码
/// - `line`: 0-based 行号
/// - `column`: 0-based 列号
/// - `prefix`: 当前已输入的前缀（由前端提取）
pub fn get_completion_candidates(
    session: &Session,
    source: &str,
    line: usize,
    column: usize,
    prefix: &str,
) -> Vec<CompletionCandidate> {
    // === 核心优化：实时解析源码获取符号快照 ===
    // 利用自研 Parser 的错误恢复能力，即使代码不完整也能提取部分符号。
    // 与编译快照合并，确保编译成功的精确信息优先，同时补充实时编辑中的新符号。
    let compiled_snapshot = &session.compile.completion_snapshot;
    let live_snapshot = build_snapshot_from_source(source);
    let snapshot = merge_snapshots(compiled_snapshot, &live_snapshot);

    let context = detect_context(source, line, column);

    let mut candidates: Vec<CompletionCandidate> = match &context {
        CompletionContext::MemberAccess {
            base_name,
            is_pointer,
        } => {
            complete_members(session, &snapshot, source, line, column, base_name, *is_pointer)
        }
        CompletionContext::TypePosition => complete_types(&snapshot, prefix),
        CompletionContext::Preprocessor => complete_preprocessor(prefix),
        CompletionContext::FormatString { func_name } => {
            complete_format_string(func_name, prefix)
        }
        CompletionContext::Expression => {
            complete_expression(session, &snapshot, source, line, column, prefix)
        }
    };

    // 统一按前缀匹配过滤（上下文相关的已经预过滤过，这里做安全兜底）
    if !prefix.is_empty() && !matches!(context, CompletionContext::FormatString { .. }) {
        let lower_prefix = prefix.to_lowercase();
        candidates.retain(|c| c.filter_text().to_lowercase().starts_with(&lower_prefix));
    }

    // 排序
    candidates.sort_by(|a, b| a.sort_text.cmp(&b.sort_text));

    // 限制数量（前端 UI 性能）
    candidates.truncate(50);

    candidates
}

// ============================================================================
// 上下文检测
// ============================================================================

fn detect_context(source: &str, line: usize, column: usize) -> CompletionContext {
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

    // 3. 成员访问上下文：识别 `identifier.` 或 `identifier->`
    if let Some((name, is_pointer)) = detect_member_access(&before) {
        return CompletionContext::MemberAccess {
            base_name: name,
            is_pointer,
        };
    }

    // 4. 类型上下文
    if detect_type_position(&before) {
        return CompletionContext::TypePosition;
    }

    CompletionContext::Expression
}

/// 提取光标前的文本（行内 + 前行末尾）
fn text_before_cursor(source: &str, line: usize, column: usize) -> String {
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
fn detect_format_function(before: &str) -> Option<String> {
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

/// 检测成员访问：`identifier.` 或 `identifier->`
fn detect_member_access(before: &str) -> Option<(String, bool)> {
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

    if start < i {
        let name = String::from_utf8_lossy(&bytes[start..i]).to_string();
        if !name.is_empty() {
            return Some((name, is_pointer));
        }
    }

    None
}

/// 检测是否在类型位置（简化 heuristic）
fn detect_type_position(before: &str) -> bool {
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

// ============================================================================
// 局部符号增量扫描
// ============================================================================

/// 轻量级 token 扫描，提取光标所在作用域的局部变量和参数
fn scan_local_symbols(source: &str, target_line: usize, _target_column: usize) -> Vec<LocalSymbol> {
    let (tokens, _) = Lexer::new(source).tokenize();
    let mut locals = Vec::new();
    let mut scope_depth: usize = 0;
    let mut i = 0;

    while i < tokens.len() {
        let tok = &tokens[i];

        // 只扫描到目标行（包含目标行）
        if tok.line > target_line as i32 + 1 {
            break;
        }

        match tok.ty {
            TokenType::LBrace => scope_depth += 1,
            TokenType::RBrace => {
                scope_depth = scope_depth.saturating_sub(1);
                // 清除离开作用域的局部变量
                locals.retain(|l: &LocalSymbol| l.scope_depth <= scope_depth);
            }
            TokenType::Int
            | TokenType::Char
            | TokenType::Float
            | TokenType::Double
            | TokenType::Void
            | TokenType::Long
            | TokenType::Short
            | TokenType::Signed
            | TokenType::Unsigned
            | TokenType::Const
            | TokenType::Struct
            | TokenType::Union
            | TokenType::Enum => {
                // 尝试解析类型 + 变量名 模式
                if let Some((ty_str, names, new_i, is_param)) =
                    try_parse_declaration(&tokens, i, scope_depth)
                {
                    for (name, _init) in names {
                        locals.push(LocalSymbol {
                            name,
                            ty: ty_str.clone(),
                            scope_depth,
                            is_param,
                        });
                    }
                    i = new_i;
                    continue;
                }
            }
            _ => {}
        }

        i += 1;
    }

    locals
}

/// 尝试从 token[i]（类型关键字）开始解析声明
/// 返回 (type_string, vec<(name, has_init)>, next_index, is_param)
#[allow(clippy::type_complexity)]
fn try_parse_declaration(
    tokens: &[Token],
    start: usize,
    _scope_depth: usize,
) -> Option<(String, Vec<(String, bool)>, usize, bool)> {
    let mut i = start;
    let mut type_parts = Vec::new();

    // 阶段 1：收集类型信息
    while i < tokens.len() {
        match tokens[i].ty {
            TokenType::Int
            | TokenType::Char
            | TokenType::Float
            | TokenType::Double
            | TokenType::Void
            | TokenType::Long
            | TokenType::Short
            | TokenType::Signed
            | TokenType::Unsigned
            | TokenType::Const
            => {
                type_parts.push(tokens[i].text.clone());
                i += 1;
            }
            TokenType::Struct | TokenType::Union | TokenType::Enum => {
                type_parts.push(tokens[i].text.clone());
                i += 1;
                if i < tokens.len() && tokens[i].ty == TokenType::Identifier {
                    type_parts.push(tokens[i].text.clone());
                    i += 1;
                }
            }
            TokenType::Star => {
                type_parts.push("*".to_string());
                i += 1;
            }
            TokenType::Identifier => {
                if type_parts.is_empty() {
                    // typedef 别名，如 `Point p;`
                    type_parts.push(tokens[i].text.clone());
                    i += 1;
                    break;
                }
                // type_parts 非空时，此 identifier 大概率是变量名，停止类型收集
                break;
            }
            _ => break,
        }
    }

    if type_parts.is_empty() {
        return None;
    }

    // 阶段 2：解析变量名列表（i 当前指向第一个变量名）
    if i >= tokens.len() || tokens[i].ty != TokenType::Identifier {
        return None;
    }

    let ty_str = type_parts.join(" ");
    let mut names = Vec::new();
    let mut is_param = false;

    // 解析逗号分隔的多变量：Type name1, name2, name3;
    loop {
        if i >= tokens.len() || tokens[i].ty != TokenType::Identifier {
            break;
        }
        let name = tokens[i].text.clone();
        i += 1;

        // 跳过数组维度 [N]
        while i < tokens.len() && tokens[i].ty == TokenType::LBracket {
            i += 1;
            while i < tokens.len() && tokens[i].ty != TokenType::RBracket {
                i += 1;
            }
            if i < tokens.len() {
                i += 1;
            }
        }

        // 检查是否有初始化 = ...
        let has_init =
            i < tokens.len() && (tokens[i].ty == TokenType::Assign || tokens[i].ty == TokenType::LParen);

        names.push((name, has_init));

        // 如果是函数定义/声明的参数列表
        if i < tokens.len() && tokens[i].ty == TokenType::Comma {
            i += 1;
            is_param = true;
            continue;
        }

        break;
    }

    // Consume until semicolon or RParen or LBrace
    while i < tokens.len() {
        match tokens[i].ty {
            TokenType::Semicolon | TokenType::RParen | TokenType::LBrace => {
                if tokens[i].ty == TokenType::RParen {
                    is_param = true;
                }
                i += 1;
                break;
            }
            _ => i += 1,
        }
    }

    Some((ty_str, names, i, is_param))
}

// ============================================================================
// 补全生成器
// ============================================================================

fn complete_members(
    _session: &Session,
    snapshot: &CompletionSnapshot,
    source: &str,
    line: usize,
    column: usize,
    base_name: &str,
    is_pointer: bool,
) -> Vec<CompletionCandidate> {
    let mut candidates = Vec::new();

    // 1. 确定 base_name 的类型
    let base_type = find_variable_type(snapshot, source, line, column, base_name);

    let type_name = match base_type {
        Some(ty) => {
            if is_pointer {
                // 去掉末尾的 *
                ty.trim_end_matches(" *").trim_end_matches('*').trim().to_string()
            } else {
                ty.trim().to_string()
            }
        }
        None => return candidates,
    };

    // 2. 在 struct/union 中查找字段
    let type_name_clean = type_name
        .trim_start_matches("struct ")
        .trim_start_matches("union ")
        .trim()
        .to_string();

    for s in &snapshot.structs {
        if s.name == type_name_clean {
            for (fname, fty) in &s.fields {
                candidates.push(CompletionCandidate {
                    label: fname.clone(),
                    kind: CompletionKind::Field,
                    detail: fty.clone(),
                    documentation: String::new(),
                    insert_text: fname.clone(),
                    sort_text: format!("0_{}", fname),
                });
            }
            break;
        }
    }

    for u in &snapshot.unions {
        if u.name == type_name_clean {
            for (fname, fty) in &u.fields {
                candidates.push(CompletionCandidate {
                    label: fname.clone(),
                    kind: CompletionKind::Field,
                    detail: fty.clone(),
                    documentation: String::new(),
                    insert_text: fname.clone(),
                    sort_text: format!("0_{}", fname),
                });
            }
            break;
        }
    }

    candidates
}

fn complete_types(snapshot: &CompletionSnapshot, prefix: &str) -> Vec<CompletionCandidate> {
    let mut candidates = Vec::new();

    // 基本类型与类型关键字
    let primitives = [
        ("int", "integer type"),
        ("char", "character type"),
        ("float", "single-precision float"),
        ("double", "double-precision float"),
        ("void", "no type"),
        ("long", "long modifier"),
        ("short", "short modifier"),
        ("signed", "signed modifier"),
        ("unsigned", "unsigned modifier"),
        ("const", "const qualifier"),
        ("static", "static storage"),
        ("extern", "external linkage"),
        ("struct", "struct type"),
        ("union", "union type"),
        ("enum", "enum type"),
    ];
    for (name, doc) in primitives {
        candidates.push(CompletionCandidate {
            label: name.to_string(),
            kind: CompletionKind::Keyword,
            detail: doc.to_string(),
            documentation: String::new(),
            insert_text: name.to_string(),
            sort_text: format!("1_{}", name),
        });
    }

    // struct / union 类型名
    for s in &snapshot.structs {
        candidates.push(CompletionCandidate {
            label: s.name.clone(),
            kind: CompletionKind::Struct,
            detail: format!("struct {} {{ ... }}", s.name),
            documentation: String::new(),
            insert_text: s.name.clone(),
            sort_text: format!("0_{}", s.name),
        });
    }
    for u in &snapshot.unions {
        candidates.push(CompletionCandidate {
            label: u.name.clone(),
            kind: CompletionKind::Union,
            detail: format!("union {} {{ ... }}", u.name),
            documentation: String::new(),
            insert_text: u.name.clone(),
            sort_text: format!("0_{}", u.name),
        });
    }

    // typedef 别名
    for t in &snapshot.typedefs {
        candidates.push(CompletionCandidate {
            label: t.name.clone(),
            kind: CompletionKind::Typedef,
            detail: t.underlying.clone(),
            documentation: String::new(),
            insert_text: t.name.clone(),
            sort_text: format!("0_{}", t.name),
        });
    }

    // enum 名
    for e in &snapshot.enums {
        candidates.push(CompletionCandidate {
            label: e.name.clone(),
            kind: CompletionKind::Enum,
            detail: format!("enum {{ {} }}", e.variants.join(", ")),
            documentation: String::new(),
            insert_text: e.name.clone(),
            sort_text: format!("0_{}", e.name),
        });
    }

    if !prefix.is_empty() {
        let lower = prefix.to_lowercase();
        candidates.retain(|c| c.filter_text().to_lowercase().starts_with(&lower));
    }

    candidates
}

fn complete_expression(
    _session: &Session,
    snapshot: &CompletionSnapshot,
    source: &str,
    line: usize,
    column: usize,
    _prefix: &str,
) -> Vec<CompletionCandidate> {
    let mut candidates: Vec<CompletionCandidate> = Vec::new();

    // 1. 局部变量（增量扫描）
    let locals = scan_local_symbols(source, line, column);
    for local in &locals {
        candidates.push(CompletionCandidate {
            label: local.name.clone(),
            kind: CompletionKind::Variable,
            detail: local.ty.clone(),
            documentation: String::new(),
            insert_text: local.name.clone(),
            sort_text: format!("0_{}", local.name), // 局部变量最优先
        });
    }

    // 2. 全局变量
    for g in &snapshot.globals {
        candidates.push(CompletionCandidate {
            label: g.name.clone(),
            kind: CompletionKind::Variable,
            detail: g.ty.clone(),
            documentation: String::new(),
            insert_text: g.name.clone(),
            sort_text: format!("1_{}", g.name),
        });
    }

    // 3. 函数
    for f in &snapshot.functions {
        let sig = if f.params.is_empty() {
            format!("{}()", f.return_type)
        } else {
            format!("{}({})", f.return_type, f.params.join(", "))
        };
        let insert = format!("{}()", f.name);
        candidates.push(CompletionCandidate {
            label: f.name.clone(),
            kind: CompletionKind::Function,
            detail: sig,
            documentation: if f.is_static {
                format!("static ({})", f.filename)
            } else {
                String::new()
            },
            insert_text: insert,
            sort_text: format!("2_{}", f.name),
        });
    }

    // 4. 结构体/联合体/typedef/enum 类型名（也作为表达式候选，用于 sizeof 等）
    for s in &snapshot.structs {
        candidates.push(CompletionCandidate {
            label: s.name.clone(),
            kind: CompletionKind::Struct,
            detail: format!("struct {}", s.name),
            documentation: String::new(),
            insert_text: s.name.clone(),
            sort_text: format!("3_{}", s.name),
        });
    }
    for u in &snapshot.unions {
        candidates.push(CompletionCandidate {
            label: u.name.clone(),
            kind: CompletionKind::Union,
            detail: format!("union {}", u.name),
            documentation: String::new(),
            insert_text: u.name.clone(),
            sort_text: format!("3_{}", u.name),
        });
    }
    for t in &snapshot.typedefs {
        candidates.push(CompletionCandidate {
            label: t.name.clone(),
            kind: CompletionKind::Typedef,
            detail: t.underlying.clone(),
            documentation: String::new(),
            insert_text: t.name.clone(),
            sort_text: format!("3_{}", t.name),
        });
    }

    // 5. 关键字（补充常用 C 关键字）
    let keywords = [
        "return", "if", "else", "while", "for", "do", "break", "continue",
        "switch", "case", "default", "sizeof", "NULL", "struct", "union",
        "enum", "typedef", "int", "char", "float", "double", "void",
        "long", "short", "signed", "unsigned", "const", "static", "extern",
    ];
    for kw in keywords {
        candidates.push(CompletionCandidate {
            label: kw.to_string(),
            kind: CompletionKind::Keyword,
            detail: String::new(),
            documentation: String::new(),
            insert_text: kw.to_string(),
            sort_text: format!("9_{}", kw),
        });
    }

    candidates
}

fn complete_preprocessor(prefix: &str) -> Vec<CompletionCandidate> {
    let items = [
        ("include", "#include <header>"),
        ("define", "#define NAME value"),
        ("ifdef", "#ifdef NAME"),
        ("ifndef", "#ifndef NAME"),
        ("endif", "#endif"),
        ("pragma", "#pragma ..."),
    ];

    let mut candidates: Vec<CompletionCandidate> = items
        .iter()
        .map(|(name, detail)| CompletionCandidate {
            label: name.to_string(),
            kind: CompletionKind::Keyword,
            detail: detail.to_string(),
            documentation: String::new(),
            insert_text: name.to_string(),
            sort_text: format!("0_{}", name),
        })
        .collect();

    // 标准头文件
    let headers = [
        "stdio.h", "stdlib.h", "string.h", "math.h", "ctype.h", "time.h",
    ];
    for h in headers {
        candidates.push(CompletionCandidate {
            label: h.to_string(),
            kind: CompletionKind::Macro,
            detail: format!("标准头文件 <{}>", h),
            documentation: String::new(),
            insert_text: h.to_string(),
            sort_text: format!("1_{}", h),
        });
    }

    if !prefix.is_empty() {
        let lower = prefix.to_lowercase();
        candidates.retain(|c| c.filter_text().to_lowercase().starts_with(&lower));
    }

    candidates
}

fn complete_format_string(func_name: &str, prefix: &str) -> Vec<CompletionCandidate> {
    let mut specs = Vec::new();

    let common = [
        ("%d", "signed int"),
        ("%f", "float/double"),
        ("%c", "char"),
        ("%s", "char* string"),
        ("%p", "pointer"),
        ("%x", "hex int"),
        ("%o", "octal int"),
        ("%ld", "long"),
        ("%lld", "long long"),
        ("%u", "unsigned int"),
        ("%zu", "size_t"),
        ("%%", "literal %"),
    ];

    let scanf_extra = [
        ("%d", "int*"),
        ("%f", "float*"),
        ("%lf", "double*"),
        ("%c", "char*"),
        ("%s", "char[] buffer"),
    ];

    let used = if func_name == "scanf" {
        &scanf_extra[..]
    } else {
        &common[..]
    };

    for (spec, detail) in used {
        specs.push(CompletionCandidate {
            label: spec.to_string(),
            kind: CompletionKind::FormatSpecifier,
            detail: detail.to_string(),
            documentation: String::new(),
            insert_text: spec.to_string(),
            sort_text: format!("0_{}", spec),
        });
    }

    if !prefix.is_empty() {
        let lower = prefix.to_lowercase();
        specs.retain(|c| c.filter_text().to_lowercase().starts_with(&lower));
    }

    specs
}

// ============================================================================
// 辅助函数
// ============================================================================

fn find_variable_type(
    snapshot: &CompletionSnapshot,
    source: &str,
    line: usize,
    column: usize,
    name: &str,
) -> Option<String> {
    // 1. 先查局部变量
    let locals = scan_local_symbols(source, line, column);
    for local in &locals {
        if local.name == name {
            return Some(local.ty.clone());
        }
    }

    // 2. 查全局变量
    for g in &snapshot.globals {
        if g.name == name {
            return Some(g.ty.clone());
        }
    }

    // 3. 查函数参数（从 snapshot functions 中查找）
    for f in &snapshot.functions {
        for p in &f.params {
            // params 格式: "Type name"
            let parts: Vec<&str> = p.rsplitn(2, ' ').collect();
            if parts.len() == 2 && parts[0] == name {
                return Some(parts[1].to_string());
            }
        }
    }

    None
}

impl CompletionCandidate {
    fn filter_text(&self) -> String {
        self.label.clone()
    }
}

// ============================================================================
// 为 CompileState 提供便捷方法
// ============================================================================

pub fn update_completion_snapshot(session: &mut Session, program: &ProgramNode) {
    session.compile.completion_snapshot = build_snapshot(program);
}
