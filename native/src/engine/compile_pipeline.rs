//! 共享编译管线辅助函数
//!
//! 为 `flutter_bridge.rs` 和 `capi/mod.rs` 提供统一的诊断推送、VM 初始化逻辑，
//! 消除两端的代码重复。

use crate::compiler::ast::SourceLoc as AstSourceLoc;
use crate::compiler::bytecode_gen::BytecodeGen;
use crate::compiler::lexer::Lexer;
use crate::compiler::parser::Parser;
use crate::compiler::type_checker::TypeChecker;
use crate::session::*;
use crate::vm::vm::CideVM;
use std::slice;

// ========== 编译错误 trait ==========

pub trait CompileError {
    fn line(&self) -> i32;
    fn column(&self) -> i32;
    fn code(&self) -> i32;
    fn message(&self) -> &str;
}

impl CompileError for crate::compiler::lexer::LexerError {
    fn line(&self) -> i32 { self.line }
    fn column(&self) -> i32 { self.column }
    fn code(&self) -> i32 { self.code }
    fn message(&self) -> &str { &self.message }
}

impl CompileError for crate::compiler::parser::ParseError {
    fn line(&self) -> i32 { self.line }
    fn column(&self) -> i32 { self.column }
    fn code(&self) -> i32 { self.code }
    fn message(&self) -> &str { &self.message }
}

impl CompileError for crate::compiler::type_checker::TypeError {
    fn line(&self) -> i32 { self.line }
    fn column(&self) -> i32 { self.column }
    fn code(&self) -> i32 { self.code }
    fn message(&self) -> &str { &self.message }
}

// ========== 诊断推送 ==========

fn push_one<T: CompileError>(session: &mut Session, item: &T, severity: i32, source_lines: &[&str]) {
    let info = crate::diagnostics::error_catalog::lookup_error_info(item.code());
    let (fix_suggestion, fix_kind, rsl, rsc, rel, rec, rt) =
        crate::diagnostics::error_catalog::generate_fix(
            item.code(), item.line(), item.column(), item.message(), source_lines,
        );

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
    });
}

pub fn push_diagnostics<T: CompileError>(session: &mut Session, errors: &[T], source: &str) {
    let source_lines: Vec<&str> = source.lines().collect();
    let mut err_str = String::new();
    for e in errors {
        let info = crate::diagnostics::error_catalog::lookup_error_info(e.code());
        let enriched_msg = if let Some(ref i) = info {
            format!(
                "{} {} — 错误 E{} (第{}行, 第{}列): {}",
                i.emoji, i.title, e.code(), e.line(), e.column(), e.message()
            )
        } else {
            format!(
                "错误 E{} (第{}行, 第{}列): {}",
                e.code(), e.line(), e.column(), e.message()
            )
        };
        push_one(session, e, 0, &source_lines);
        err_str.push_str(&enriched_msg);
        err_str.push('\n');
    }
    session.compile.errors = err_str.clone();
    session.compile.errors_buffer = err_str;
}

pub fn push_warnings<T: CompileError>(session: &mut Session, warnings: &[T], source: &str) {
    let source_lines: Vec<&str> = source.lines().collect();
    for w in warnings {
        push_one(session, w, 1, &source_lines);
    }
}

pub fn push_hints<T: CompileError>(session: &mut Session, hints: &[T], source: &str) {
    let source_lines: Vec<&str> = source.lines().collect();
    for h in hints {
        push_one(session, h, 2, &source_lines);
    }
}

// ========== VM 初始化 ==========

pub fn setup_vm(vm: &mut CideVM, session: &Session) {
    use crate::vm::vm::{FuncMeta, VMSymbol};

    vm.reset();
    vm.load_program(session.compile.bytecode.clone());
    vm.set_globals(&session.compile.globals_init);
    vm.set_max_steps(10_000_000);

    for (name, meta) in &session.compile.func_table {
        if let Some(&idx) = session.compile.func_index.get(name) {
            vm.register_function(idx as u32, FuncMeta {
                ip: meta.ip,
                arg_count: meta.arg_count,
                local_count: meta.local_count,
            });
            vm.register_function_name(idx as u32, name.clone());
        }
    }

    let symbols: Vec<VMSymbol> = session.compile.symbols.iter().map(|s| VMSymbol {
        name: s.name.clone(),
        addr: s.addr,
        is_local: s.is_local,
        ty: s.ty.clone(),
        scope_depth: s.scope_depth,
    }).collect();
    vm.set_symbols(symbols);

    let mut vis_lines = Vec::new();
    for m in &session.compile.algorithm_matches {
        for &(line, ty, _) in &m.vis_events {
            vis_lines.push((line, ty));
        }
    }
    vm.set_vis_event_lines(vis_lines);

    // 写入字符串数据到 VM 内存
    let mem = vm.get_memory();
    let mem_size = vm.get_memory_size() as usize;
    for &(addr, ref s) in &session.compile.string_data {
        unsafe {
            write_string_to_vm_memory(mem, mem_size, addr, s);
        }
    }
}

/// 将 C 风格字符串安全写入 VM 内存的指定地址。
///
/// 内部验证边界条件：要求 `mem` 指向至少 `mem_size` 字节的有效内存，
/// 且 `addr + s.len() < mem_size` 时才执行写入。
/// 自动追加 null 终止符。
/// # Safety
/// `mem` 必须指向至少 `mem_size` 字节的有效连续内存。
pub unsafe fn write_string_to_vm_memory(mem: *mut u8, mem_size: usize, addr: u32, s: &str) {
    let a = addr as usize;
    let bytes = s.as_bytes();
    if a + bytes.len() < mem_size {
        unsafe {
            let dst = slice::from_raw_parts_mut(mem.add(a), bytes.len() + 1);
            dst[..bytes.len()].copy_from_slice(bytes);
            dst[bytes.len()] = 0;
        }
    }
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
    session.compile.diagnostics.clear();
    session.compile.source_map.clear();
    session.compile.func_table.clear();
    session.compile.func_index.clear();
    session.compile.string_data.clear();
    session.compile.symbols.clear();
    session.compile.algorithm_matches.clear();
    session.compile.struct_fields.clear();
    session.compile.errors.clear();
    session.compile.errors_buffer.clear();
    session.compile.compiled = false;

    // 1. Lexer
    let (tokens, lex_errors) = Lexer::new(full_source.to_string()).tokenize();
    if !lex_errors.is_empty() {
        push_diagnostics(session, &lex_errors, full_source);
        return Err("词法错误".to_string());
    }

    // 2. Parser
    let (maybe_program, parse_errors) = Parser::new(tokens).parse();
    if !parse_errors.is_empty() {
        push_diagnostics(session, &parse_errors, full_source);
        return Err("语法错误".to_string());
    }

    let mut program = match maybe_program {
        Some(p) => p,
        None => {
            session.compile.errors = "解析失败：无法生成 AST".to_string();
            session.compile.errors_buffer = session.compile.errors.clone();
            return Err("解析失败".to_string());
        }
    };

    // 3. TypeChecker
    let (type_errors, type_warnings, type_hints) = TypeChecker::default().check(&mut program);
    if !type_errors.is_empty() {
        push_diagnostics(session, &type_errors, full_source);
        return Err("类型错误".to_string());
    }
    if !type_warnings.is_empty() {
        push_warnings(session, &type_warnings, full_source);
    }
    if !type_hints.is_empty() {
        push_hints(session, &type_hints, full_source);
    }

    // 4. BytecodeGen
    let gen = BytecodeGen::new();
    let output = match gen.generate(&mut program) {
        Ok(o) => o,
        Err(gen_errors) => {
            let err_str: String = gen_errors.iter().map(|e| format!("生成错误: {}\n", e)).collect();
            session.compile.errors = err_str.clone();
            session.compile.errors_buffer = err_str;
            return Err("字节码生成错误".to_string());
        }
    };

    // 填充编译结果
    session.compile.bytecode = output.code;
    session.compile.globals_init = output.globals_init;
    session.compile.source_map = output
        .source_map
        .into_iter()
        .map(|(ip, loc)| (ip, AstSourceLoc { line: loc.line, column: loc.column }))
        .collect();
    session.compile.func_index = output.func_index;

    for (name, meta) in output.func_table {
        session.compile.func_table.insert(
            name,
            FuncMeta {
                ip: meta.ip,
                arg_count: meta.arg_count,
                local_count: meta.local_count,
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
        });
    }

    for (name, fields) in output.struct_defs {
        let converted: Vec<(String, i32)> = fields
            .into_iter()
            .enumerate()
            .map(|(i, f)| (f.name, i as i32 * 4))
            .collect();
        session.compile.struct_fields.insert(name, converted);
    }

    // 算法模式识别
    session.compile.algorithm_matches =
        crate::compiler::algorithm_detector::detect_algorithms(&program);

    session.compile.compiled = true;
    session.compile.errors.clear();
    session.compile.errors_buffer.clear();

    Ok(())
}
