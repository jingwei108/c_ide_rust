use super::*;

pub fn complete_members(
    _session: &Session,
    snapshot: &CompletionSnapshot,
    source: &str,
    line: usize,
    column: usize,
    expr: &str,
    _is_pointer: bool,
) -> Vec<CompletionCandidate> {
    let mut candidates = Vec::new();

    // 1. 将链式表达式拆分为路径段（如 "a.b.c" -> ["a", "b", "c"]）
    let segments: Vec<&str> = expr.split("->").flat_map(|s| s.split('.')).filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return candidates;
    }

    // 2. 逐级解析类型
    let mut current_type = find_variable_type(snapshot, source, line, column, segments[0]);

    for seg in segments.iter().skip(1) {
        let ty = match current_type {
            Some(ref t) => t.clone(),
            None => return candidates,
        };
        // 去掉指针/struct/union 前缀
        let clean = ty
            .trim_end_matches(" *")
            .trim_end_matches('*')
            .trim_start_matches("struct ")
            .trim_start_matches("union ")
            .trim()
            .to_string();
        // 查找 clean 类型中 seg 字段的类型
        let mut found = None;
        for s in &snapshot.structs {
            if s.name == clean {
                found = s.fields.iter().find(|(n, _)| n == seg).map(|(_, t)| t.clone());
                break;
            }
        }
        if found.is_none() {
            for u in &snapshot.unions {
                if u.name == clean {
                    found = u.fields.iter().find(|(n, _)| n == seg).map(|(_, t)| t.clone());
                    break;
                }
            }
        }
        current_type = found;
    }

    let type_name = match current_type {
        Some(ty) => ty.trim_end_matches(" *").trim_end_matches('*').trim().to_string(),
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

pub fn complete_types(snapshot: &CompletionSnapshot, prefix: &str) -> Vec<CompletionCandidate> {
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

pub fn complete_expression(
    _session: &Session,
    snapshot: &CompletionSnapshot,
    source: &str,
    line: usize,
    column: usize,
    prefix: &str,
    hint: ExprHint,
) -> Vec<CompletionCandidate> {
    let mut candidates: Vec<CompletionCandidate> = Vec::new();

    // 上下文感知 Snippet（高优先级）
    if prefix.is_empty() || "for".starts_with(prefix) {
        candidates.push(CompletionCandidate {
            label: "for (i < n)".to_string(),
            kind: CompletionKind::Snippet,
            detail: "for 循环模板".to_string(),
            documentation: String::new(),
            insert_text: "for (int i = 0; i < n; i++) {\n\t\n}".to_string(),
            sort_text: "z0_for_snippet".to_string(),
        });
    }
    if prefix.is_empty() || "while".starts_with(prefix) {
        candidates.push(CompletionCandidate {
            label: "while (cond)".to_string(),
            kind: CompletionKind::Snippet,
            detail: "while 循环模板".to_string(),
            documentation: String::new(),
            insert_text: "while (condition) {\n\t\n}".to_string(),
            sort_text: "z0_while_snippet".to_string(),
        });
    }
    if prefix.is_empty() || "if".starts_with(prefix) {
        candidates.push(CompletionCandidate {
            label: "if (cond)".to_string(),
            kind: CompletionKind::Snippet,
            detail: "if 分支模板".to_string(),
            documentation: String::new(),
            insert_text: "if (condition) {\n\t\n}".to_string(),
            sort_text: "z0_if_snippet".to_string(),
        });
    }
    if hint == ExprHint::MallocArg && (prefix.is_empty() || "sizeof".starts_with(prefix)) {
        candidates.push(CompletionCandidate {
            label: "sizeof(type)".to_string(),
            kind: CompletionKind::Snippet,
            detail: "计算类型大小".to_string(),
            documentation: String::new(),
            insert_text: "sizeof()".to_string(),
            sort_text: "z0_sizeof_snippet".to_string(),
        });
    }
    if hint == ExprHint::IfCondition {
        for op in ["==", "!=", "<", ">", "<=", ">="] {
            candidates.push(CompletionCandidate {
                label: op.to_string(),
                kind: CompletionKind::Operator,
                detail: "比较运算符".to_string(),
                documentation: String::new(),
                insert_text: op.to_string(),
                sort_text: format!("z1_{}", op),
            });
        }
    }

    // 1. 局部变量（增量扫描）
    let locals = scan_local_symbols(source, line, column);
    for local in &locals {
        // 上下文感知排序微调
        let sort_prefix = match hint {
            ExprHint::ForCondition => {
                if is_likely_loop_var(&local.name) {
                    "0a"
                } else {
                    "0b"
                }
            }
            ExprHint::IfCondition => {
                if local.ty.contains("int") || local.ty.contains("char") || local.ty.contains("bool") {
                    "0a"
                } else {
                    "0b"
                }
            }
            _ => "0",
        };
        candidates.push(CompletionCandidate {
            label: local.name.clone(),
            kind: CompletionKind::Variable,
            detail: local.ty.clone(),
            documentation: String::new(),
            insert_text: local.name.clone(),
            sort_text: format!("{}_{}", sort_prefix, local.name),
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
        "return", "if", "else", "while", "for", "do", "break", "continue", "switch", "case", "default", "sizeof",
        "NULL", "struct", "union", "enum", "typedef", "int", "char", "float", "double", "void", "long", "short",
        "signed", "unsigned", "const", "static", "extern",
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

pub fn complete_preprocessor(prefix: &str) -> Vec<CompletionCandidate> {
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
    let headers = ["stdio.h", "stdlib.h", "string.h", "math.h", "ctype.h", "time.h"];
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

pub fn complete_format_string(func_name: &str, prefix: &str) -> Vec<CompletionCandidate> {
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

pub fn find_variable_type(
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
    pub fn filter_text(&self) -> String {
        self.label.clone()
    }
}

// ============================================================================
// 为 CompileState 提供便捷方法
// ============================================================================
