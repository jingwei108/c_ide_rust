#![allow(clippy::missing_safety_doc)]

use crate::session::*;
use crate::vm::vm::{CideVM, FuncMeta, VMSymbol};
use crate::compiler::lexer::Lexer;
use crate::compiler::parser::Parser;
use crate::compiler::type_checker::TypeChecker;
use crate::compiler::bytecode_gen::BytecodeGen;
use std::ffi::{c_char, c_int, CStr};
use std::ptr;
use std::slice;

/// Trait to abstract over LexerError, ParseError, and TypeError.
trait CompileError {
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

unsafe fn cstr_to_str(s: *const c_char) -> Option<&'static str> {
    if s.is_null() {
        return None;
    }
    CStr::from_ptr(s).to_str().ok()
}

unsafe fn write_str(dst: *mut c_char, dst_size: c_int, src: &str) {
    if dst.is_null() || dst_size <= 0 {
        return;
    }
    let len = src.len().min((dst_size - 1) as usize);
    let slice = slice::from_raw_parts_mut(dst as *mut u8, len);
    slice.copy_from_slice(&src.as_bytes()[..len]);
    *dst.add(len) = 0;
}

#[no_mangle]
pub extern "C" fn cide_session_create() -> *mut Session {
    Box::into_raw(Box::new(Session::default()))
}

#[no_mangle]
pub unsafe extern "C" fn cide_session_destroy(s: *mut Session) {
    if !s.is_null() {
        drop(Box::from_raw(s));
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SessionSnapshot {
    compile: CompileState,
    runtime: RuntimeState,
    memory: MemoryState,
}

#[no_mangle]
pub unsafe extern "C" fn cide_session_save(s: *mut Session, filepath: *const c_char) -> c_int {
    if s.is_null() || filepath.is_null() {
        return -1;
    }
    let session = &*s;
    let snapshot = SessionSnapshot {
        compile: session.compile.clone(),
        runtime: session.runtime.clone(),
        memory: session.memory.clone(),
    };
    let path = CStr::from_ptr(filepath).to_string_lossy().into_owned();
    match serde_json::to_string_pretty(&snapshot) {
        Ok(json) => {
            if std::fs::write(&path, json).is_ok() {
                0
            } else {
                -1
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn cide_session_load(s: *mut Session, filepath: *const c_char) -> c_int {
    if s.is_null() || filepath.is_null() {
        return -1;
    }
    let session = &mut *s;
    let path = CStr::from_ptr(filepath).to_string_lossy().into_owned();
    match std::fs::read_to_string(&path) {
        Ok(json) => {
            match serde_json::from_str::<SessionSnapshot>(&json) {
                Ok(snapshot) => {
                    session.compile = snapshot.compile;
                    session.runtime = snapshot.runtime;
                    session.memory = snapshot.memory;
                    session.vm = Some(CideVM::default());
                    0
                }
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn cide_compile(s: *mut Session, source: *const c_char) -> c_int {
    if s.is_null() || source.is_null() {
        return -1;
    }
    let session = &mut *s;
    let src = match cstr_to_str(source) {
        Some(v) => v,
        None => return -1,
    };
    session.compile.compile_units.clear();
    session.compile.compile_units.push(CompileUnit {
        filename: "main.c".to_string(),
        source: src.to_string(),
    });
    cide_compile_all(s)
}

#[no_mangle]
pub unsafe extern "C" fn cide_compile_unit(
    s: *mut Session,
    filename: *const c_char,
    source: *const c_char,
) -> c_int {
    if s.is_null() || filename.is_null() || source.is_null() {
        return -1;
    }
    let session = &mut *s;
    let fname = match cstr_to_str(filename) {
        Some(v) => v,
        None => return -1,
    };
    let src = match cstr_to_str(source) {
        Some(v) => v,
        None => return -1,
    };
    session.compile.compile_units.push(CompileUnit {
        filename: fname.to_string(),
        source: src.to_string(),
    });
    0
}

#[no_mangle]
pub unsafe extern "C" fn cide_compile_all(s: *mut Session) -> c_int {
    if s.is_null() {
        return -1;
    }
    let session = &mut *s;

    // 清空之前的编译状态
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

    // 拼接所有编译单元（避免末尾多余换行导致行号偏移）
    let mut full_source = String::new();
    for unit in &session.compile.compile_units {
        full_source.push_str(&unit.source);
        if !unit.source.ends_with('\n') {
            full_source.push('\n');
        }
    }

    // Helper: 将编译错误同时填充到 diagnostics 和 errors 字符串
    fn push_diagnostics<T: CompileError>(session: &mut Session, errors: &[T], source: &str) {
        let source_lines: Vec<&str> = source.lines().collect();
        let mut err_str = String::new();
        for e in errors {
            let info = crate::diagnostics::error_catalog::lookup_error_info(e.code());
            let (fix_suggestion, fix_kind, rsl, rsc, rel, rec, rt) =
                crate::diagnostics::error_catalog::generate_fix(e.code(), e.line(), e.column(), e.message(), &source_lines);

            let enriched_msg = if let Some(ref i) = info {
                format!("{} {} — 错误 E{} (第{}行, 第{}列): {}", i.emoji, i.title, e.code(), e.line(), e.column(), e.message())
            } else {
                format!("错误 E{} (第{}行, 第{}列): {}", e.code(), e.line(), e.column(), e.message())
            };

            session.compile.diagnostics.push(crate::session::Diagnostic {
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

    fn push_warnings<T: CompileError>(session: &mut Session, warnings: &[T], source: &str) {
        let source_lines: Vec<&str> = source.lines().collect();
        for w in warnings {
            let info = crate::diagnostics::error_catalog::lookup_error_info(w.code());
            let (fix_suggestion, fix_kind, rsl, rsc, rel, rec, rt) =
                crate::diagnostics::error_catalog::generate_fix(w.code(), w.line(), w.column(), w.message(), &source_lines);

            session.compile.diagnostics.push(crate::session::Diagnostic {
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

    fn push_hints<T: CompileError>(session: &mut Session, hints: &[T], source: &str) {
        let source_lines: Vec<&str> = source.lines().collect();
        for h in hints {
            let info = crate::diagnostics::error_catalog::lookup_error_info(h.code());
            let (fix_suggestion, fix_kind, rsl, rsc, rel, rec, rt) =
                crate::diagnostics::error_catalog::generate_fix(h.code(), h.line(), h.column(), h.message(), &source_lines);

            session.compile.diagnostics.push(crate::session::Diagnostic {
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

    // 1. Lexer
    let (tokens, lex_errors) = Lexer::new(full_source.clone()).tokenize();
    if !lex_errors.is_empty() {
        push_diagnostics(session, &lex_errors, &full_source);
        return -1;
    }

    // 2. Parser
    let (maybe_program, parse_errors) = Parser::new(tokens).parse();
    if !parse_errors.is_empty() {
        push_diagnostics(session, &parse_errors, &full_source);
        return -1;
    }

    let mut program = match maybe_program {
        Some(p) => p,
        None => {
            session.compile.errors = "解析失败：无法生成 AST".to_string();
            session.compile.errors_buffer = session.compile.errors.clone();
            return -1;
        }
    };

    // 3. TypeChecker
    let (type_errors, type_warnings, type_hints) = TypeChecker::new().check(&mut program);
    if !type_errors.is_empty() {
        push_diagnostics(session, &type_errors, &full_source);
        return -1;
    }
    if !type_warnings.is_empty() {
        push_warnings(session, &type_warnings, &full_source);
    }
    if !type_hints.is_empty() {
        push_hints(session, &type_hints, &full_source);
    }

    // 4. BytecodeGen
    let gen = BytecodeGen::new();
    let output = match gen.generate(&mut program) {
        Ok(o) => o,
        Err(gen_errors) => {
            let mut err_str = String::new();
            for e in &gen_errors {
                err_str.push_str(&format!("生成错误: {}\n", e));
            }
            session.compile.errors = err_str.clone();
            session.compile.errors_buffer = err_str;
            return -1;
        }
    };

    // 填充编译结果
    session.compile.bytecode = output.code;
    session.compile.globals_init = output.globals_init;
    session.compile.source_map = output.source_map.into_iter()
        .map(|(ip, loc)| (ip, crate::compiler::ast::SourceLoc { line: loc.line, column: loc.column }))
        .collect();
    session.compile.func_index = output.func_index;

    // func_table 转换：bytecode_gen::FuncMeta -> session::FuncMeta
    for (name, meta) in output.func_table {
        session.compile.func_table.insert(name, crate::session::FuncMeta {
            ip: meta.ip,
            arg_count: meta.arg_count,
            local_count: meta.local_count,
        });
    }

    session.compile.string_data = output.string_data;

    // symbols 转换：VMSymbol -> Symbol
    for sym in output.symbols {
        session.compile.symbols.push(crate::session::Symbol {
            name: sym.name,
            addr: sym.addr,
            is_local: sym.is_local,
            ty: sym.ty,
            scope_depth: sym.scope_depth,
        });
    }

    // struct_fields 转换
    for (name, fields) in output.struct_defs {
        let converted: Vec<(String, i32)> = fields.into_iter()
            .enumerate()
            .map(|(i, f)| (f.name, i as i32 * 4))
            .collect();
        session.compile.struct_fields.insert(name, converted);
    }

    session.compile.compiled = true;
    session.compile.errors.clear();
    session.compile.errors_buffer.clear();
    0
}

/// 返回指向编译错误字符串的指针。
/// 
/// # 安全性
/// 返回的指针仅在下次调用 `cide_compile` / `cide_compile_all` 之前有效。
/// 调用方应立即复制数据，不要长期保存此指针。
#[no_mangle]
pub unsafe extern "C" fn cide_get_compile_errors(s: *mut Session) -> *const c_char {
    if s.is_null() {
        return ptr::null();
    }
    let session = &mut *s;
    if session.compile.errors.is_empty() {
        return ptr::null();
    }
    if session.compile.errors_buffer != session.compile.errors {
        session.compile.errors_buffer = session.compile.errors.clone();
    }
    session.compile.errors_buffer.as_ptr() as *const c_char
}

#[no_mangle]
pub unsafe extern "C" fn cide_get_compile_errors_buf(
    s: *mut Session,
    buf: *mut c_char,
    max_len: c_int,
) -> c_int {
    if s.is_null() || buf.is_null() || max_len <= 0 {
        return -1;
    }
    let session = &*s;
    if session.compile.errors.is_empty() {
        *buf = 0;
        return 0;
    }
    let copy_len = session.compile.errors.len().min((max_len - 1) as usize);
    let slice = slice::from_raw_parts_mut(buf as *mut u8, copy_len);
    slice.copy_from_slice(&session.compile.errors.as_bytes()[..copy_len]);
    *buf.add(copy_len) = 0;
    copy_len as c_int
}

unsafe fn setup_vm(vm: &mut CideVM, session: &Session) {
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

    let mem = vm.get_memory();
    let mem_size = vm.get_memory_size() as usize;
    for &(addr, ref str) in &session.compile.string_data {
        let a = addr as usize;
        let bytes = str.as_bytes();
        if a + bytes.len() < mem_size {
            let dst = slice::from_raw_parts_mut(mem.add(a), bytes.len() + 1);
            dst[..bytes.len()].copy_from_slice(bytes);
            dst[bytes.len()] = 0;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cide_run(s: *mut Session) -> c_int {
    if s.is_null() || !(*s).compile.compiled {
        if !s.is_null() {
            (*s).runtime.error = "程序尚未编译。请先编译代码。".to_string();
        }
        return -1;
    }
    let session = &mut *s;
    session.runtime.output_lines.clear();
    session.runtime.error.clear();
    session.runtime.trace.clear();
    session.memory.regions.clear();
    session.memory.free_list.clear();
    session.memory.heap_offset = 0x5000;
    session.memory.alloc_counter = 0;
    session.runtime.running = true;
    session.runtime.step_mode = false;

    let mut vm = session.vm.take().unwrap();
    setup_vm(&mut vm, session);

    let ret = vm.run(session);

    let result = if vm.has_error() {
        session.runtime.error = vm.get_error().to_string();
        session.runtime.running = false;
        -1
    } else {
        session.runtime.output_lines.push(format!("程序运行完成，返回值：{}\n", ret));
        session.runtime.running = false;
        0
    };
    session.vm = Some(vm);
    result
}

#[no_mangle]
pub unsafe extern "C" fn cide_step_next(s: *mut Session) -> c_int {
    if s.is_null() || !(*s).compile.compiled {
        if !s.is_null() {
            (*s).runtime.error = "程序尚未编译。".to_string();
        }
        return -1;
    }
    let session = &mut *s;

    let mut vm = session.vm.take().unwrap();
    let result = if !session.runtime.running {
        session.runtime.output_lines.clear();
        session.runtime.error.clear();
        session.runtime.trace.clear();
        session.memory.regions.clear();
        session.memory.free_list.clear();
        session.memory.heap_offset = 0x5000;
        session.memory.alloc_counter = 0;
        session.runtime.step_count = 0;
        session.runtime.step_mode = true;
        session.runtime.running = true;

        setup_vm(&mut vm, session);
        vm.pause();

        let ret;
        loop {
            match vm.step(session) {
                crate::vm::vm::StepResult::Paused => {
                    session.runtime.current_line = vm.get_current_line();
                    ret = 0;
                    break;
                }
                crate::vm::vm::StepResult::Finished => {
                    session.runtime.running = false;
                    session.runtime.current_line = vm.get_current_line();
                    ret = -1;
                    break;
                }
                crate::vm::vm::StepResult::Trap => {
                    session.runtime.error = vm.get_error().to_string();
                    session.runtime.running = false;
                    session.runtime.current_line = vm.get_current_line();
                    ret = -1;
                    break;
                }
                _ => {}
            }
        }
        ret
    } else {
        vm.resume();
        let ret;
        loop {
            match vm.step(session) {
                crate::vm::vm::StepResult::Paused => {
                    session.runtime.current_line = vm.get_current_line();
                    ret = 0;
                    break;
                }
                _ if vm.was_step_event_hit() => {
                    vm.pause();
                    session.runtime.current_line = vm.get_current_line();
                    ret = 0;
                    break;
                }
                crate::vm::vm::StepResult::Finished => {
                    session.runtime.running = false;
                    session.runtime.current_line = vm.get_current_line();
                    ret = -1;
                    break;
                }
                crate::vm::vm::StepResult::Trap => {
                    session.runtime.error = vm.get_error().to_string();
                    session.runtime.running = false;
                    session.runtime.current_line = vm.get_current_line();
                    ret = -1;
                    break;
                }
                _ => {}
            }
        }
        ret
    };
    session.vm = Some(vm);
    result
}

#[no_mangle]
pub unsafe extern "C" fn cide_get_current_line(s: *mut Session) -> c_int {
    if s.is_null() {
        return 0;
    }
    (*s).runtime.current_line
}

#[no_mangle]
pub unsafe extern "C" fn cide_callstack_count(s: *mut Session) -> c_int {
    if s.is_null() {
        return 0;
    }
    if let Some(ref vm) = (*s).vm {
        vm.get_call_stack().len() as c_int
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn cide_callstack_get(
    s: *mut Session,
    index: c_int,
    name: *mut c_char,
    name_size: c_int,
    line: *mut c_int,
) {
    let stack_len = if s.is_null() {
        0
    } else if let Some(ref vm) = (*s).vm {
        vm.get_call_stack().len() as c_int
    } else {
        0
    };
    if s.is_null() || index < 0 || index >= stack_len {
        if !name.is_null() && name_size > 0 { *name = 0; }
        if !line.is_null() { *line = 0; }
        return;
    }
    let session = &*s;
    let frame = &session.vm.as_ref().unwrap().get_call_stack()[index as usize];
    write_str(name, name_size, &frame.func_name);

    let mut best_line = 0;
    if !session.compile.source_map.is_empty() && frame.return_ip > 0 {
        let ret_ip = frame.return_ip as u32;
        let map = &session.compile.source_map;
        let mut best = None;
        for &(ip, ref loc) in map {
            if ip <= ret_ip {
                best = Some(loc.line);
            } else {
                break;
            }
        }
        if let Some(l) = best { best_line = l; }
    }
    if !line.is_null() { *line = best_line; }
}

#[no_mangle]
pub unsafe extern "C" fn cide_breakpoint_add(s: *mut Session, line: c_int) {
    if !s.is_null() && line > 0 {
        if let Some(ref mut vm) = (*s).vm {
            vm.add_breakpoint(line);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cide_breakpoint_remove(s: *mut Session, line: c_int) {
    if !s.is_null() && line > 0 {
        if let Some(ref mut vm) = (*s).vm {
            vm.remove_breakpoint(line);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cide_breakpoint_clear(s: *mut Session) {
    if !s.is_null() {
        if let Some(ref mut vm) = (*s).vm {
            vm.clear_breakpoints();
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cide_get_runtime_error(s: *mut Session) -> *const c_char {
    if s.is_null() {
        return ptr::null();
    }
    let session = &*s;
    if session.runtime.error.is_empty() {
        return ptr::null();
    }
    session.runtime.error.as_ptr() as *const c_char
}

#[no_mangle]
pub unsafe extern "C" fn cide_get_runtime_error_buf(
    s: *mut Session,
    buf: *mut c_char,
    max_len: c_int,
) -> c_int {
    if s.is_null() || buf.is_null() || max_len <= 0 {
        return -1;
    }
    let session = &*s;
    if session.runtime.error.is_empty() {
        *buf = 0;
        return 0;
    }
    let copy_len = session.runtime.error.len().min((max_len - 1) as usize);
    let slice = slice::from_raw_parts_mut(buf as *mut u8, copy_len);
    slice.copy_from_slice(&session.runtime.error.as_bytes()[..copy_len]);
    *buf.add(copy_len) = 0;
    copy_len as c_int
}

#[no_mangle]
pub unsafe extern "C" fn cide_set_input(s: *mut Session, input: *const c_char) {
    if s.is_null() {
        return;
    }
    let session = &mut *s;
    session.runtime.input_lines.clear();
    session.runtime.input_index = 0;
    let input_str = match cstr_to_str(input) {
        Some(v) => v,
        None => return,
    };
    for line in input_str.lines() {
        session.runtime.input_lines.push(line.trim_end_matches('\r').to_string());
    }
}

#[no_mangle]
pub unsafe extern "C" fn cide_input_count(s: *mut Session) -> c_int {
    if s.is_null() {
        return 0;
    }
    (*s).runtime.input_lines.len() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn cide_get_output_length(s: *mut Session) -> c_int {
    if s.is_null() {
        return 0;
    }
    (*s).runtime.output_lines.iter().map(|l| l.len()).sum::<usize>() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn cide_get_output(s: *mut Session, buf: *mut c_char, max_len: c_int) {
    if s.is_null() || buf.is_null() || max_len <= 0 {
        return;
    }
    let session = &*s;
    let all: String = session.runtime.output_lines.concat();
    let copy_len = all.len().min((max_len - 1) as usize);
    let slice = slice::from_raw_parts_mut(buf as *mut u8, copy_len);
    slice.copy_from_slice(&all.as_bytes()[..copy_len]);
    *buf.add(copy_len) = 0;
}

#[no_mangle]
pub unsafe extern "C" fn cide_memory_region_count(s: *mut Session) -> c_int {
    if s.is_null() {
        return 0;
    }
    (*s).memory.regions.len() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn cide_memory_region_get(
    s: *mut Session,
    index: c_int,
    addr: *mut u32,
    size: *mut c_int,
    name: *mut c_char,
    name_size: c_int,
    ty: *mut c_char,
    type_size: c_int,
    is_heap: *mut c_int,
    is_freed: *mut c_int,
) {
    if s.is_null()
        || index < 0
        || index >= (*s).memory.regions.len() as c_int
    {
        if !addr.is_null() { *addr = 0; }
        if !size.is_null() { *size = 0; }
        if !is_heap.is_null() { *is_heap = 0; }
        if !is_freed.is_null() { *is_freed = 0; }
        return;
    }
    let r = &(&(*s).memory.regions)[index as usize];
    if !addr.is_null() { *addr = r.addr; }
    if !size.is_null() { *size = r.size; }
    if !is_heap.is_null() { *is_heap = if r.is_heap { 1 } else { 0 }; }
    if !is_freed.is_null() { *is_freed = if r.is_freed { 1 } else { 0 }; }
    write_str(name, name_size, &r.name);
    write_str(ty, type_size, &r.ty);
}

#[no_mangle]
pub unsafe extern "C" fn cide_memory_get_value(
    s: *mut Session,
    addr: u32,
    out_val: *mut c_int,
) -> c_int {
    if s.is_null() || out_val.is_null() {
        return -1;
    }
    let session = &*s;
    if let Some(ref vm) = session.vm {
        let mem = vm.memory_ref();
        if addr as u64 + 4 <= mem.len() as u64 {
            let val = i32::from_le_bytes([
                mem[addr as usize],
                mem[addr as usize + 1],
                mem[addr as usize + 2],
                mem[addr as usize + 3],
            ]);
            *out_val = val;
            return 0;
        }
    }
    *out_val = 0;
    -1
}

#[no_mangle]
pub unsafe extern "C" fn cide_memory_get_pointer_target(
    s: *mut Session,
    addr: u32,
    out_target: *mut u32,
) -> c_int {
    if s.is_null() || out_target.is_null() {
        return -1;
    }
    let session = &*s;
    if let Some(ref vm) = session.vm {
        let mem = vm.memory_ref();
        if addr as u64 + 4 <= mem.len() as u64 {
            let target = i32::from_le_bytes([
                mem[addr as usize],
                mem[addr as usize + 1],
                mem[addr as usize + 2],
                mem[addr as usize + 3],
            ]);
            if target >= 0 {
                *out_target = target as u32;
                return 0;
            }
        }
    }
    *out_target = 0;
    -1
}

#[no_mangle]
pub unsafe extern "C" fn cide_diagnostic_count(s: *mut Session) -> c_int {
    if s.is_null() {
        return 0;
    }
    (*s).compile.diagnostics.len() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn cide_diagnostic_get(
    s: *mut Session,
    index: c_int,
    line: *mut c_int,
    column: *mut c_int,
    error_code: *mut c_int,
    severity: *mut c_int,
    message: *mut c_char,
    msg_size: c_int,
    fix_suggestion: *mut c_char,
    fix_size: c_int,
) {
    if s.is_null()
        || index < 0
        || index >= (*s).compile.diagnostics.len() as c_int
    {
        if !line.is_null() { *line = 0; }
        if !column.is_null() { *column = 0; }
        if !error_code.is_null() { *error_code = 0; }
        if !severity.is_null() { *severity = 0; }
        return;
    }
    let d = &(&(*s).compile.diagnostics)[index as usize];
    if !line.is_null() { *line = d.line; }
    if !column.is_null() { *column = d.column; }
    if !error_code.is_null() { *error_code = d.error_code; }
    if !severity.is_null() { *severity = d.severity; }
    write_str(message, msg_size, &d.message);
    write_str(fix_suggestion, fix_size, &d.fix_suggestion);
}

#[no_mangle]
pub unsafe extern "C" fn cide_diagnostic_get_fix(
    s: *mut Session,
    index: c_int,
    fix_kind: *mut c_int,
    start_line: *mut c_int,
    start_column: *mut c_int,
    end_line: *mut c_int,
    end_column: *mut c_int,
    replacement_text: *mut c_char,
    replacement_size: c_int,
) {
    if s.is_null()
        || index < 0
        || index >= (*s).compile.diagnostics.len() as c_int
    {
        if !fix_kind.is_null() { *fix_kind = 0; }
        if !start_line.is_null() { *start_line = 0; }
        if !start_column.is_null() { *start_column = 0; }
        if !end_line.is_null() { *end_line = 0; }
        if !end_column.is_null() { *end_column = 0; }
        return;
    }
    let d = &(&(*s).compile.diagnostics)[index as usize];
    if !fix_kind.is_null() { *fix_kind = d.fix_kind; }
    if !start_line.is_null() { *start_line = d.replace_start_line; }
    if !start_column.is_null() { *start_column = d.replace_start_column; }
    if !end_line.is_null() { *end_line = d.replace_end_line; }
    if !end_column.is_null() { *end_column = d.replace_end_column; }
    write_str(replacement_text, replacement_size, &d.replacement_text);
}

#[no_mangle]
pub unsafe extern "C" fn cide_sourcemap_lookup(
    s: *mut Session,
    bytecode_offset: u32,
    out_line: *mut c_int,
    out_column: *mut c_int,
) -> c_int {
    if s.is_null() || out_line.is_null() || out_column.is_null() {
        return -1;
    }
    let session = &*s;
    let map = &session.compile.source_map;
    if map.is_empty() {
        return -1;
    }
    let mut best = None;
    for (ip, loc) in map.iter() {
        if *ip <= bytecode_offset {
            best = Some(loc);
        } else {
            break;
        }
    }
    if let Some(loc) = best {
        *out_line = loc.line;
        *out_column = loc.column;
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn cide_trace_count(s: *mut Session) -> c_int {
    if s.is_null() {
        return 0;
    }
    (*s).runtime.trace.len() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn cide_trace_get(
    s: *mut Session,
    index: c_int,
    line: *mut c_int,
    operation: *mut c_char,
    op_size: c_int,
) {
    if s.is_null()
        || index < 0
        || index >= (*s).runtime.trace.len() as c_int
    {
        if !line.is_null() { *line = 0; }
        return;
    }
    let t = &(&(*s).runtime.trace)[index as usize];
    if !line.is_null() { *line = t.line; }
    write_str(operation, op_size, &t.operation);
}

#[no_mangle]
pub unsafe extern "C" fn cide_variable_count(s: *mut Session) -> c_int {
    if s.is_null() {
        return 0;
    }
    (*s).runtime.variable_snapshot.len() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn cide_variable_get(
    s: *mut Session,
    index: c_int,
    name: *mut c_char,
    name_size: c_int,
    addr: *mut u32,
    is_local: *mut c_int,
    is_array: *mut c_int,
    array_size: *mut c_int,
    value: *mut c_int,
) {
    if s.is_null()
        || index < 0
        || index >= (*s).runtime.variable_snapshot.len() as c_int
    {
        if !name.is_null() && name_size > 0 { *name = 0; }
        if !addr.is_null() { *addr = 0; }
        if !is_local.is_null() { *is_local = 0; }
        if !is_array.is_null() { *is_array = 0; }
        if !array_size.is_null() { *array_size = 0; }
        if !value.is_null() { *value = 0; }
        return;
    }
    let v = &(&(*s).runtime.variable_snapshot)[index as usize];
    write_str(name, name_size, &v.name);
    if !addr.is_null() { *addr = v.addr; }
    if !is_local.is_null() { *is_local = if v.is_local { 1 } else { 0 }; }
    let is_arr = matches!(v.ty.kind, crate::compiler::ast::TypeKind::Array);
    if !is_array.is_null() { *is_array = if is_arr { 1 } else { 0 }; }
    if !array_size.is_null() { *array_size = if is_arr { v.ty.array_size } else { 0 }; }
    if !value.is_null() { *value = v.value; }
}

#[no_mangle]
pub unsafe extern "C" fn cide_variable_get_type(
    s: *mut Session,
    index: c_int,
    type_buf: *mut c_char,
    type_buf_size: c_int,
) -> c_int {
    if s.is_null()
        || index < 0
        || index >= (*s).runtime.variable_snapshot.len() as c_int
    {
        if !type_buf.is_null() && type_buf_size > 0 { *type_buf = 0; }
        return -1;
    }
    let v = &(&(*s).runtime.variable_snapshot)[index as usize];
    let type_str = format_type(&v.ty);
    write_str(type_buf, type_buf_size, &type_str);
    type_str.len() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn cide_variable_find_by_addr(
    s: *mut Session,
    addr: u32,
    name: *mut c_char,
    name_size: c_int,
    offset: *mut c_int,
) -> c_int {
    if s.is_null() || name.is_null() || name_size <= 0 {
        return -1;
    }
    let vars = &(*s).runtime.variable_snapshot;
    for v in vars.iter() {
        let size = if matches!(v.ty.kind, crate::compiler::ast::TypeKind::Array) {
            (v.ty.array_size as u32) * 4
        } else {
            4
        };
        if addr >= v.addr && addr < v.addr + size {
            write_str(name, name_size, &v.name);
            if !offset.is_null() { *offset = (addr - v.addr) as c_int; }
            return 0;
        }
    }
    *name = 0;
    if !offset.is_null() { *offset = 0; }
    -1
}

#[no_mangle]
pub unsafe extern "C" fn cide_variable_get_field(
    s: *mut Session,
    var_index: c_int,
    field_index: c_int,
    out_offset: *mut c_int,
    name: *mut c_char,
    name_size: c_int,
) -> c_int {
    if s.is_null() || var_index < 0 || field_index < 0 {
        return -1;
    }
    let session = &*s;
    if var_index >= session.runtime.variable_snapshot.len() as c_int {
        return -1;
    }
    let v = &session.runtime.variable_snapshot[var_index as usize];
    let struct_name = match v.ty.kind {
        crate::compiler::ast::TypeKind::Struct => v.ty.name.clone(),
        crate::compiler::ast::TypeKind::Pointer if matches!(v.ty.base_kind, crate::compiler::ast::TypeKind::Struct) => v.ty.name.clone(),
        _ => return -1,
    };
    let fields = match session.compile.struct_fields.get(&struct_name) {
        Some(f) => f,
        None => return -1,
    };
    if field_index >= fields.len() as c_int {
        return -1;
    }
    let (field_name, field_offset) = &fields[field_index as usize];
    if !out_offset.is_null() { *out_offset = *field_offset; }
    write_str(name, name_size, field_name);
    0
}

#[no_mangle]
pub unsafe extern "C" fn cide_vis_event_count(s: *mut Session) -> c_int {
    if s.is_null() {
        return 0;
    }
    (*s).runtime.vis_event_cache.len() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn cide_vis_event_get(
    s: *mut Session,
    index: c_int,
    ty: *mut c_int,
    line: *mut c_int,
) {
    if s.is_null()
        || index < 0
        || index >= (*s).runtime.vis_event_cache.len() as c_int
    {
        if !ty.is_null() { *ty = 0; }
        if !line.is_null() { *line = 0; }
        return;
    }
    let e = &(&(*s).runtime.vis_event_cache)[index as usize];
    if !ty.is_null() { *ty = e.ty; }
    if !line.is_null() { *line = e.line; }
}

#[no_mangle]
pub unsafe extern "C" fn cide_vis_event_get_ex(
    s: *mut Session,
    index: c_int,
    ty: *mut c_int,
    line: *mut c_int,
    extra0: *mut c_int,
    extra1: *mut c_int,
    extra2: *mut c_int,
) {
    if s.is_null()
        || index < 0
        || index >= (*s).runtime.vis_event_cache.len() as c_int
    {
        if !ty.is_null() { *ty = 0; }
        if !line.is_null() { *line = 0; }
        if !extra0.is_null() { *extra0 = 0; }
        if !extra1.is_null() { *extra1 = 0; }
        if !extra2.is_null() { *extra2 = 0; }
        return;
    }
    let e = &(&(*s).runtime.vis_event_cache)[index as usize];
    if !ty.is_null() { *ty = e.ty; }
    if !line.is_null() { *line = e.line; }
    if !extra0.is_null() { *extra0 = e.extra[0]; }
    if !extra1.is_null() { *extra1 = e.extra[1]; }
    if !extra2.is_null() { *extra2 = e.extra[2]; }
}

#[no_mangle]
pub unsafe extern "C" fn cide_vis_event_clear(s: *mut Session) {
    if s.is_null() {
        return;
    }
    (*s).runtime.vis_event_cache.clear();
}

#[no_mangle]
pub unsafe extern "C" fn cide_algorithm_match_count(s: *mut Session) -> c_int {
    if s.is_null() {
        return 0;
    }
    (*s).compile.algorithm_matches.len() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn cide_algorithm_match_get(
    s: *mut Session,
    index: c_int,
    name: *mut c_char,
    name_size: c_int,
    display_name: *mut c_char,
    display_name_size: c_int,
    func_name: *mut c_char,
    func_name_size: c_int,
    confidence: *mut c_int,
    suggestion: *mut c_char,
    suggestion_size: c_int,
    line: *mut c_int,
) {
    if s.is_null()
        || index < 0
        || index >= (*s).compile.algorithm_matches.len() as c_int
    {
        if !name.is_null() && name_size > 0 { *name = 0; }
        if !display_name.is_null() && display_name_size > 0 { *display_name = 0; }
        if !func_name.is_null() && func_name_size > 0 { *func_name = 0; }
        if !confidence.is_null() { *confidence = 0; }
        if !suggestion.is_null() && suggestion_size > 0 { *suggestion = 0; }
        if !line.is_null() { *line = 0; }
        return;
    }
    let m = &(&(*s).compile.algorithm_matches)[index as usize];
    write_str(name, name_size, &m.name);
    write_str(display_name, display_name_size, &m.display_name);
    write_str(func_name, func_name_size, &m.func_name);
    if !confidence.is_null() { *confidence = m.confidence; }
    write_str(suggestion, suggestion_size, &m.suggestion);
    if !line.is_null() { *line = m.line; }
}

#[no_mangle]
pub unsafe extern "C" fn cide_algorithm_match_vis_event_count(
    s: *mut Session,
    match_index: c_int,
) -> c_int {
    if s.is_null()
        || match_index < 0
        || match_index >= (*s).compile.algorithm_matches.len() as c_int
    {
        return 0;
    }
    (&(*s).compile.algorithm_matches)[match_index as usize]
        .vis_events
        .len() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn cide_algorithm_match_vis_event_get(
    s: *mut Session,
    match_index: c_int,
    event_index: c_int,
    ty: *mut c_int,
    line: *mut c_int,
    context: *mut c_char,
    context_size: c_int,
) {
    if s.is_null()
        || match_index < 0
        || match_index >= (*s).compile.algorithm_matches.len() as c_int
    {
        if !ty.is_null() { *ty = 0; }
        if !line.is_null() { *line = 0; }
        if !context.is_null() && context_size > 0 { *context = 0; }
        return;
    }
    let m = &(&(*s).compile.algorithm_matches)[match_index as usize];
    if event_index < 0 || event_index >= m.vis_events.len() as c_int {
        if !ty.is_null() { *ty = 0; }
        if !line.is_null() { *line = 0; }
        if !context.is_null() && context_size > 0 { *context = 0; }
        return;
    }
    let (ev_line, ev_ty, ctx) = &m.vis_events[event_index as usize];
    if !ty.is_null() { *ty = *ev_ty; }
    if !line.is_null() { *line = *ev_line; }
    write_str(context, context_size, ctx);
}

fn format_type(t: &crate::compiler::ast::Type) -> String {
    use crate::compiler::ast::TypeKind;
    match t.kind {
        TypeKind::Void => "void".to_string(),
        TypeKind::Int => "int".to_string(),
        TypeKind::Char => "char".to_string(),
        TypeKind::Float => "float".to_string(),
        TypeKind::Pointer => {
            let base = match t.base_kind {
                TypeKind::Struct => format!("struct {}", t.name),
                TypeKind::Char => "char".to_string(),
                _ => "int".to_string(),
            };
            format!("{}*", base)
        }
        TypeKind::Array => {
            let base = match t.base_kind {
                TypeKind::Struct => format!("struct {}", t.name),
                TypeKind::Char => "char".to_string(),
                _ => "int".to_string(),
            };
            if !t.dims.is_empty() {
                let mut s = base;
                for d in &t.dims {
                    s.push_str(&format!("[{}]", d));
                }
                s
            } else if t.array_size > 0 {
                format!("{}[{}]", base, t.array_size)
            } else {
                format!("{}[]", base)
            }
        }
        TypeKind::Struct => format!("struct {}", t.name),
    }
}
