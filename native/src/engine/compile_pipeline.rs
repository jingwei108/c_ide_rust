//! 共享编译管线辅助函数
//!
//! 为 `flutter_bridge.rs` 和 `capi/mod.rs` 提供统一的诊断推送、VM 初始化逻辑，
//! 消除两端的代码重复。

use crate::compiler::ast::{self, SourceLoc as AstSourceLoc};
use crate::compiler::codegen::BytecodeGen;
use crate::compiler::lexer::Lexer;
use crate::compiler::parser::Parser;
use crate::compiler::typeck::TypeChecker;
use crate::engine::completion::update_completion_snapshot;
use crate::session::*;
use crate::vm::vm::CideVM;

// ---------- 辅助函数：根据类型定义计算类型大小 ----------

// ========== 编译错误 trait ==========

pub trait CompileError {
    fn line(&self) -> i32;
    fn column(&self) -> i32;
    fn code(&self) -> i32;
    fn message(&self) -> &str;
}

impl CompileError for crate::compiler::lexer::LexerError {
    fn line(&self) -> i32 {
        self.line
    }
    fn column(&self) -> i32 {
        self.column
    }
    fn code(&self) -> i32 {
        self.code
    }
    fn message(&self) -> &str {
        &self.message
    }
}

impl CompileError for crate::compiler::parser::ParseError {
    fn line(&self) -> i32 {
        self.line
    }
    fn column(&self) -> i32 {
        self.column
    }
    fn code(&self) -> i32 {
        self.code
    }
    fn message(&self) -> &str {
        &self.message
    }
}

impl CompileError for crate::compiler::typeck::TypeError {
    fn line(&self) -> i32 {
        self.line
    }
    fn column(&self) -> i32 {
        self.column
    }
    fn code(&self) -> i32 {
        self.code
    }
    fn message(&self) -> &str {
        &self.message
    }
}

// ========== 多文件行号映射 ==========

pub struct FileRange {
    pub filename: String,
    pub start_line: i32,
    pub end_line: i32,
}

fn resolve_filename(line: i32, file_ranges: &[FileRange]) -> Option<String> {
    file_ranges
        .iter()
        .find(|r| line >= r.start_line && line <= r.end_line)
        .map(|r| r.filename.clone())
}

// ========== 诊断推送 ==========

fn push_one<T: CompileError>(
    session: &mut Session,
    item: &T,
    severity: i32,
    source_lines: &[&str],
    file_ranges: Option<&[FileRange]>,
) {
    let info = crate::diagnostics::error_catalog::lookup_error_info(item.code());
    let (fix_suggestion, fix_kind, rsl, rsc, rel, rec, rt) = crate::diagnostics::error_catalog::generate_fix(
        item.code(),
        item.line(),
        item.column(),
        item.message(),
        source_lines,
    );

    let filename = file_ranges
        .and_then(|ranges| resolve_filename(item.line(), ranges))
        .unwrap_or_else(|| "main.c".to_string());

    session.compile.diagnostics.push(Diagnostic {
        line: item.line(),
        column: item.column(),
        error_code: item.code(),
        severity,
        message: item.message().to_string(),
        fix_suggestion: if fix_suggestion.is_empty() {
            info.map(|i| i.explanation.to_string()).unwrap_or_default()
        } else {
            fix_suggestion.clone()
        },
        fix_kind,
        replace_start_line: rsl,
        replace_start_column: rsc,
        replace_end_line: rel,
        replace_end_column: rec,
        replacement_text: rt,
        filename,
    });
}

pub fn push_diagnostics<T: CompileError>(
    session: &mut Session,
    errors: &[T],
    source: &str,
    file_ranges: Option<&[FileRange]>,
) {
    let source_lines: Vec<&str> = source.lines().collect();
    let mut err_str = String::new();
    for e in errors {
        let info = crate::diagnostics::error_catalog::lookup_error_info(e.code());
        let enriched_msg = if let Some(ref i) = info {
            format!(
                "{} {} — 错误 E{} (第{}行, 第{}列): {}",
                i.emoji,
                i.title,
                e.code(),
                e.line(),
                e.column(),
                e.message()
            )
        } else {
            format!("错误 E{} (第{}行, 第{}列): {}", e.code(), e.line(), e.column(), e.message())
        };
        push_one(session, e, 0, &source_lines, file_ranges);
        err_str.push_str(&enriched_msg);
        err_str.push('\n');
    }
    session.compile.errors = err_str;
}

pub fn push_warnings<T: CompileError>(
    session: &mut Session,
    warnings: &[T],
    source: &str,
    file_ranges: Option<&[FileRange]>,
) {
    let source_lines: Vec<&str> = source.lines().collect();
    for w in warnings {
        push_one(session, w, 1, &source_lines, file_ranges);
    }
}

pub fn push_hints<T: CompileError>(
    session: &mut Session,
    hints: &[T],
    source: &str,
    file_ranges: Option<&[FileRange]>,
) {
    let source_lines: Vec<&str> = source.lines().collect();
    for h in hints {
        push_one(session, h, 2, &source_lines, file_ranges);
    }
}

// ========== VM 初始化 ==========

pub fn setup_vm(vm: &mut CideVM, session: &Session) {
    use crate::vm::bytecode_libc_loader::load_artifact;
    use crate::vm::opcode::OpCode;
    use crate::vm::vm::{FuncMeta, VMSymbol};

    vm.reset();

    // ── 1. 加载 Bytecode Libc 预编译产物 ──
    let libc = load_artifact();
    let libc_code_len = libc.code.len();

    // ── 2. 重定位用户代码中的 Jump 指令 ──
    let mut user_code = session.compile.bytecode.clone();
    for instr in &mut user_code {
        match instr.op {
            OpCode::Jump | OpCode::JumpIfZero | OpCode::JumpIfNotZero => {
                instr.operand += libc_code_len as i32;
            }
            _ => {}
        }
    }

    // ── 3. 合并代码：Bytecode Libc + 重定位后的用户代码 ──
    let mut full_code = libc.code.clone();
    full_code.extend_from_slice(&user_code);
    vm.load_program(full_code);

    // ── 4. 注册 Bytecode Libc 函数 ──
    use crate::vm::bytecode_libc_index::bytecode_libc_index;

    // 4a. 注册原始索引（供 Bytecode Libc 内部调用使用）
    for (name, meta) in &libc.func_table {
        if let Some(&raw_idx) = libc.func_index.get(name) {
            vm.register_function(
                raw_idx as u32,
                FuncMeta {
                    ip: meta.ip,
                    arg_count: meta.arg_count,
                    param_count: meta.param_count,
                    local_count: meta.local_count,
                    param_sizes: meta.param_sizes.clone(),
                },
            );
            vm.register_function_name(raw_idx as u32, name.clone());
        }
    }

    // 4b. 注册固定索引（供用户代码调用使用）
    for (name, meta) in &libc.func_table {
        if let Some(idx) = bytecode_libc_index(name) {
            vm.register_function(
                idx as u32,
                FuncMeta {
                    ip: meta.ip,
                    arg_count: meta.arg_count,
                    param_count: meta.param_count,
                    local_count: meta.local_count,
                    param_sizes: meta.param_sizes.clone(),
                },
            );
            vm.register_function_name(idx as u32, name.clone());
        }
    }

    // ── 5. 注册用户函数（IP 偏移 libc_code_len） ──
    for (name, meta) in &session.compile.func_table {
        if let Some(&idx) = session.compile.func_index.get(name) {
            vm.register_function(
                idx as u32,
                FuncMeta {
                    ip: meta.ip + libc_code_len,
                    arg_count: meta.arg_count,
                    param_count: meta.param_count,
                    local_count: meta.local_count,
                    param_sizes: meta.param_sizes.clone(),
                },
            );
            vm.register_function_name(idx as u32, name.clone());
        }
    }

    // ── 6. 初始化全局变量 ──
    vm.set_globals_32(&libc.globals_init_32);
    vm.set_globals_64(&libc.globals_init_64);
    vm.set_globals_32(&session.compile.globals_init);
    vm.set_globals_64(&session.compile.globals_init_64);

    // ── 7. 合并常量池 ──
    let mut f64_constants = libc.f64_constants.clone();
    f64_constants.extend_from_slice(&session.compile.f64_constants);
    vm.set_f64_constants(f64_constants);

    let mut i64_constants = libc.i64_constants.clone();
    i64_constants.extend_from_slice(&session.compile.i64_constants);
    vm.set_i64_constants(i64_constants);

    vm.set_max_steps(10_000_000);

    // ── 8. 符号表 ──
    let symbols: Vec<VMSymbol> = session
        .compile
        .symbols
        .iter()
        .map(|s| VMSymbol {
            name: s.name.clone(),
            addr: s.addr,
            is_local: s.is_local,
            ty: s.ty.clone(),
            scope_depth: s.scope_depth,
            func_name: s.func_name.clone(),
        })
        .collect();
    vm.set_symbols(symbols);

    // ── 9. 可视化事件 ──
    let mut vis_lines = Vec::new();
    for m in &session.compile.algorithm_matches {
        for ev in &m.vis_events {
            vis_lines.push((ev.line, ev.ty, ev.context.clone()));
        }
    }
    vm.set_vis_event_lines(vis_lines);

    // ── 10. 写入字符串数据到 VM 内存 ──
    for &(addr, ref s) in &libc.string_data {
        vm.write_cstring(addr, s);
    }
    for &(addr, ref s) in &session.compile.string_data {
        vm.write_cstring(addr, s);
    }

    // ── 11. VM 入口设为用户代码起始位置 ──
    vm.set_ip(libc_code_len);
}

// ========== 统一编译管线 ==========

/// 运行完整的编译管线：Lexer → Parser → TypeChecker → BytecodeGen
///
/// 成功时填充 `session.compile` 的所有编译产物字段，并返回 `Ok(())`。
/// 失败时推送诊断信息到 session，并返回错误消息。
pub fn run_compile_pipeline(session: &mut Session, full_source: &str) -> Result<(), String> {
    // 清空编译状态
    session.compile.bytecode.clear();
    session.compile.globals_init.clear();
    session.compile.globals_init_64.clear();
    session.compile.i64_constants.clear();
    session.compile.f64_constants.clear();
    session.compile.diagnostics.clear();
    session.compile.source_map.clear();
    session.compile.func_table.clear();
    session.compile.func_index.clear();
    session.compile.string_data.clear();
    session.compile.symbols.clear();
    session.compile.algorithm_matches.clear();
    session.compile.struct_fields.clear();
    session.compile.errors.clear();
    session.compile.compiled = false;

    // 1. Lexer
    let (tokens, lex_errors) = Lexer::new(full_source).tokenize();
    if !lex_errors.is_empty() {
        push_diagnostics(session, &lex_errors, full_source, None);
        return Err("词法错误".to_string());
    }

    // 2. Parser
    let (maybe_program, parse_errors) = Parser::new(tokens).parse();
    if !parse_errors.is_empty() {
        push_diagnostics(session, &parse_errors, full_source, None);
        return Err("语法错误".to_string());
    }

    let mut program = match maybe_program {
        Some(p) => p,
        None => {
            session.compile.errors = "解析失败：无法生成 AST".to_string();
            return Err("解析失败".to_string());
        }
    };

    // 3. TypeChecker
    let (type_errors, type_warnings, type_hints) = TypeChecker::default().check(&mut program);
    if !type_errors.is_empty() {
        push_diagnostics(session, &type_errors, full_source, None);
        return Err("类型错误".to_string());
    }
    if !type_warnings.is_empty() {
        push_warnings(session, &type_warnings, full_source, None);
    }
    if !type_hints.is_empty() {
        push_hints(session, &type_hints, full_source, None);
    }

    // 4. BytecodeGen
    let gen = BytecodeGen::new();
    let output = match gen.generate(&mut program) {
        Ok(o) => o,
        Err(gen_errors) => {
            let err_str: String = gen_errors.iter().map(|e| format!("生成错误: {}\n", e)).collect();
            session.compile.errors = err_str;
            return Err("字节码生成错误".to_string());
        }
    };

    // 填充编译结果
    session.compile.bytecode = output.code;
    session.compile.globals_init = output.globals_init_32;
    session.compile.globals_init_64 = output.globals_init_64;
    session.compile.f64_constants = output.f64_constants;
    session.compile.i64_constants = output.i64_constants;
    // 偏移 source_map 以匹配 Bytecode Libc 代码拼接后的 IP
    let libc_code_len = crate::vm::bytecode_libc_index::BYTECODE_LIBC_CODE_LEN as u32;
    session.compile.source_map = output
        .source_map
        .into_iter()
        .map(|(ip, loc)| {
            (
                ip + libc_code_len,
                AstSourceLoc {
                    line: loc.line,
                    column: loc.column,
                },
            )
        })
        .collect();
    session.compile.func_index = output.func_index;

    for (name, meta) in output.func_table {
        session.compile.func_table.insert(
            name,
            FuncMeta {
                ip: meta.ip,
                arg_count: meta.arg_count,
                param_count: meta.param_count,
                local_count: meta.local_count,
                param_sizes: meta.param_sizes,
            },
        );
    }

    session.compile.string_data = output.string_data;

    for sym in output.symbols {
        session.compile.symbols.push(Symbol {
            name: sym.name,
            addr: sym.addr,
            is_local: sym.is_local,
            ty: sym.ty,
            scope_depth: sym.scope_depth,
            func_name: sym.func_name,
        });
    }

    for (name, fields) in &output.struct_defs {
        let mut offset = 0;
        let converted: Vec<(String, i32)> = fields
            .iter()
            .map(|f| {
                let current = offset;
                let empty_class_map: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
                offset += ast::compute_type_size(&f.ty, &output.struct_defs, &output.union_defs, &empty_class_map);
                (f.name.clone(), current)
            })
            .collect();
        session.compile.struct_fields.insert(name.clone(), converted);
    }
    for (name, fields) in &output.union_defs {
        let converted: Vec<(String, i32)> = fields.iter().map(|f| (f.name.clone(), 0)).collect();
        session.compile.struct_fields.insert(name.clone(), converted);
    }

    // 算法模式识别
    session.compile.algorithm_matches = crate::compiler::algorithm_detector::detect_algorithms(&program);

    // 更新智能补全快照
    update_completion_snapshot(session, &program);

    session.compile.compiled = true;
    session.compile.errors.clear();

    Ok(())
}

// ========== 多文件编译管线 ==========

fn merge_compile_units(units: &[CompileUnit]) -> (String, Vec<FileRange>) {
    let mut full_source = String::new();
    let mut file_ranges = Vec::new();
    let mut current_line = 1;

    for unit in units {
        let start_line = current_line;
        full_source.push_str(&unit.source);
        if !unit.source.ends_with('\n') {
            full_source.push('\n');
        }

        let line_count = if unit.source.is_empty() {
            0
        } else {
            let newline_count = unit.source.matches('\n').count() as i32;
            if unit.source.ends_with('\n') {
                newline_count
            } else {
                newline_count + 1
            }
        };

        if line_count > 0 {
            let end_line = start_line + line_count - 1;
            file_ranges.push(FileRange {
                filename: unit.filename.clone(),
                start_line,
                end_line,
            });
        }
        current_line = start_line + line_count;
    }

    (full_source, file_ranges)
}

/// 运行多文件编译管线：合并源码 → Lexer → Parser → TypeChecker → BytecodeGen
///
/// 与 `run_compile_pipeline` 的区别：
/// - 支持多个 CompileUnit（多文件）
/// - 诊断信息携带 `filename` 字段
/// - AST 节点附加 `source_file` 信息（供 TypeChecker static 隔离使用）
pub fn run_multi_file_pipeline(session: &mut Session, units: Vec<CompileUnit>) -> Result<(), String> {
    // 清空编译状态
    session.compile.bytecode.clear();
    session.compile.globals_init.clear();
    session.compile.globals_init_64.clear();
    session.compile.i64_constants.clear();
    session.compile.f64_constants.clear();
    session.compile.diagnostics.clear();
    session.compile.source_map.clear();
    session.compile.func_table.clear();
    session.compile.func_index.clear();
    session.compile.string_data.clear();
    session.compile.symbols.clear();
    session.compile.algorithm_matches.clear();
    session.compile.struct_fields.clear();
    session.compile.errors.clear();
    session.compile.compiled = false;

    let (full_source, file_ranges) = merge_compile_units(&units);

    // 根据文件扩展名检测 C++ 模式
    let is_cpp_mode = units.iter().any(|u| {
        let name = u.filename.to_lowercase();
        name.ends_with(".cpp") || name.ends_with(".cxx") || name.ends_with(".cidecpp")
    });

    // 1. Lexer
    let (tokens, lex_errors) = Lexer::with_mode(&full_source, is_cpp_mode).tokenize();
    if !lex_errors.is_empty() {
        push_diagnostics(session, &lex_errors, &full_source, Some(&file_ranges));
        return Err("词法错误".to_string());
    }

    // 2. Parser
    let (maybe_program, parse_errors) = Parser::with_mode(tokens, is_cpp_mode).parse();
    if !parse_errors.is_empty() {
        push_diagnostics(session, &parse_errors, &full_source, Some(&file_ranges));
        return Err("语法错误".to_string());
    }

    let mut program = match maybe_program {
        Some(p) => p,
        None => {
            session.compile.errors = "解析失败：无法生成 AST".to_string();
            return Err("解析失败".to_string());
        }
    };

    // 根据行号范围设置 AST 节点的 source_file
    for f in &mut program.funcs {
        f.source_file = resolve_filename(f.loc.line, &file_ranges).unwrap_or_else(|| "main.c".to_string());
    }
    for g in &mut program.globals {
        g.source_file = resolve_filename(g.loc.line, &file_ranges).unwrap_or_else(|| "main.c".to_string());
    }

    // 3. TypeChecker
    let (type_errors, type_warnings, type_hints) = TypeChecker::default().check(&mut program);
    if !type_errors.is_empty() {
        push_diagnostics(session, &type_errors, &full_source, Some(&file_ranges));
        return Err("类型错误".to_string());
    }
    if !type_warnings.is_empty() {
        push_warnings(session, &type_warnings, &full_source, Some(&file_ranges));
    }
    if !type_hints.is_empty() {
        push_hints(session, &type_hints, &full_source, Some(&file_ranges));
    }

    // 4. BytecodeGen
    let gen = BytecodeGen::new();
    let output = match gen.generate(&mut program) {
        Ok(o) => o,
        Err(gen_errors) => {
            let err_str: String = gen_errors.iter().map(|e| format!("生成错误: {}\n", e)).collect();
            session.compile.errors = err_str;
            return Err("字节码生成错误".to_string());
        }
    };

    // 填充编译结果（与 run_compile_pipeline 相同）
    session.compile.bytecode = output.code;
    session.compile.globals_init = output.globals_init_32;
    session.compile.globals_init_64 = output.globals_init_64;
    session.compile.f64_constants = output.f64_constants;
    session.compile.i64_constants = output.i64_constants;
    // 偏移 source_map 以匹配 Bytecode Libc 代码拼接后的 IP
    let libc_code_len = crate::vm::bytecode_libc_index::BYTECODE_LIBC_CODE_LEN as u32;
    session.compile.source_map = output
        .source_map
        .into_iter()
        .map(|(ip, loc)| {
            (
                ip + libc_code_len,
                AstSourceLoc {
                    line: loc.line,
                    column: loc.column,
                },
            )
        })
        .collect();
    session.compile.func_index = output.func_index;

    for (name, meta) in output.func_table {
        session.compile.func_table.insert(
            name,
            FuncMeta {
                ip: meta.ip,
                arg_count: meta.arg_count,
                param_count: meta.param_count,
                local_count: meta.local_count,
                param_sizes: meta.param_sizes,
            },
        );
    }

    session.compile.string_data = output.string_data;

    for sym in output.symbols {
        session.compile.symbols.push(Symbol {
            name: sym.name,
            addr: sym.addr,
            is_local: sym.is_local,
            ty: sym.ty,
            scope_depth: sym.scope_depth,
            func_name: sym.func_name,
        });
    }

    for (name, fields) in &output.struct_defs {
        let mut offset = 0;
        let converted: Vec<(String, i32)> = fields
            .iter()
            .map(|f| {
                let current = offset;
                let empty_class_map: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
                offset += ast::compute_type_size(&f.ty, &output.struct_defs, &output.union_defs, &empty_class_map);
                (f.name.clone(), current)
            })
            .collect();
        session.compile.struct_fields.insert(name.clone(), converted);
    }
    for (name, fields) in &output.union_defs {
        let converted: Vec<(String, i32)> = fields.iter().map(|f| (f.name.clone(), 0)).collect();
        session.compile.struct_fields.insert(name.clone(), converted);
    }

    // 算法模式识别
    session.compile.algorithm_matches = crate::compiler::algorithm_detector::detect_algorithms(&program);

    // 更新智能补全快照
    update_completion_snapshot(session, &program);

    session.compile.compiled = true;
    session.compile.errors.clear();

    Ok(())
}
