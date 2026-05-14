//! 共享编译管线辅助函数
//!
//! 为 `flutter_bridge.rs` 和 `capi/mod.rs` 提供统一的诊断推送、VM 初始化逻辑，
//! 消除两端的代码重复。

use crate::session::*;
use crate::vm::vm::CideVM;

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

pub fn push_diagnostics<T: CompileError>(session: &mut Session, errors: &[T], source: &str) {
    let source_lines: Vec<&str> = source.lines().collect();
    let mut err_str = String::new();
    for e in errors {
        let info = crate::diagnostics::error_catalog::lookup_error_info(e.code());
        let (fix_suggestion, fix_kind, rsl, rsc, rel, rec, rt) =
            crate::diagnostics::error_catalog::generate_fix(
                e.code(), e.line(), e.column(), e.message(), &source_lines,
            );

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

        session.compile.diagnostics.push(Diagnostic {
            line: e.line(),
            column: e.column(),
            error_code: e.code(),
            severity: 0,
            message: e.message().to_string(),
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
        err_str.push_str(&enriched_msg);
        err_str.push('\n');
    }
    session.compile.errors = err_str.clone();
    session.compile.errors_buffer = err_str;
}

pub fn push_warnings<T: CompileError>(session: &mut Session, warnings: &[T], source: &str) {
    let source_lines: Vec<&str> = source.lines().collect();
    for w in warnings {
        let info = crate::diagnostics::error_catalog::lookup_error_info(w.code());
        let (fix_suggestion, fix_kind, rsl, rsc, rel, rec, rt) =
            crate::diagnostics::error_catalog::generate_fix(
                w.code(), w.line(), w.column(), w.message(), &source_lines,
            );

        session.compile.diagnostics.push(Diagnostic {
            line: w.line(),
            column: w.column(),
            error_code: w.code(),
            severity: 1,
            message: w.message().to_string(),
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
}

pub fn push_hints<T: CompileError>(session: &mut Session, hints: &[T], source: &str) {
    let source_lines: Vec<&str> = source.lines().collect();
    for h in hints {
        let info = crate::diagnostics::error_catalog::lookup_error_info(h.code());
        let (fix_suggestion, fix_kind, rsl, rsc, rel, rec, rt) =
            crate::diagnostics::error_catalog::generate_fix(
                h.code(), h.line(), h.column(), h.message(), &source_lines,
            );

        session.compile.diagnostics.push(Diagnostic {
            line: h.line(),
            column: h.column(),
            error_code: h.code(),
            severity: 2,
            message: h.message().to_string(),
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
}

// ========== VM 初始化 ==========

pub fn setup_vm(vm: &mut CideVM, session: &Session) {
    use crate::vm::vm::{FuncMeta, VMSymbol};
    use std::slice;

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
    for &(addr, ref str) in &session.compile.string_data {
        let a = addr as usize;
        let bytes = str.as_bytes();
        if a + bytes.len() < mem_size {
            // Safety: mem.add(a) is within vm memory bounds (checked by a + bytes.len() < mem_size)
            // dst length equals bytes.len() + 1, copy is valid u8 to u8
            unsafe {
                let dst = slice::from_raw_parts_mut(mem.add(a), bytes.len() + 1);
                dst[..bytes.len()].copy_from_slice(bytes);
                dst[bytes.len()] = 0;
            }
        }
    }
}
