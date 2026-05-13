//! Flutter-Rust Bridge 包装层
//!
//! 为 Flutter 前端提供类型安全、异步友好的 Rust API。
//! 现有 C API (`capi/mod.rs`) 完全保留，此模块仅服务于 Flutter 前端。

use std::sync::Mutex;

use crate::session::*;
use crate::vm::vm::CideVM;
use crate::compiler::lexer::Lexer;
use crate::compiler::parser::Parser;
use crate::compiler::type_checker::TypeChecker;
use crate::compiler::bytecode_gen::BytecodeGen;

// ========== 全局 Session ==========

use once_cell::sync::Lazy;

static SESSION: Lazy<Mutex<Session>> = Lazy::new(|| {
    Mutex::new(Session::default())
});

// ========== 编译错误 trait（与 capi 共享逻辑） ==========

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

// ========== 公开数据结构 ==========

#[derive(Debug, Clone)]
pub struct CompileResult {
    pub success: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub algorithm_matches: Vec<AlgorithmMatch>,
}

#[derive(Debug, Clone)]
pub struct RunResult {
    pub success: bool,
    pub output: String,
    pub waiting_input: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub status: StepStatus,
    pub current_line: i32,
    pub output: String,
    pub waiting_input: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Paused,
    WaitingInput,
    Finished,
    Trap,
}

// ========== 辅助函数 ==========

fn push_diagnostics<T: CompileError>(session: &mut Session, errors: &[T], source: &str) {
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

fn push_warnings<T: CompileError>(session: &mut Session, warnings: &[T], source: &str) {
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

fn push_hints<T: CompileError>(session: &mut Session, hints: &[T], source: &str) {
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

fn setup_vm(vm: &mut CideVM, session: &Session) {
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
            unsafe {
                let dst = slice::from_raw_parts_mut(mem.add(a), bytes.len() + 1);
                dst[..bytes.len()].copy_from_slice(bytes);
                dst[bytes.len()] = 0;
            }
        }
    }
}

fn collect_algorithm_matches(_session: &mut Session, _program: &crate::compiler::ast::ProgramNode) {
    // 算法模式识别暂不可用（algorithm_detector 模块不存在）
    // TODO: 如需算法检测，需在 compiler 模块中添加 algorithm_detector
    _session.compile.algorithm_matches.clear();
}

// ========== 公开 API ==========

/// 设置源码并编译
pub fn compile(source: String) -> CompileResult {
    let mut session = SESSION.lock().unwrap();

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

    // 设置编译单元
    session.compile.compile_units.clear();
    session.compile.compile_units.push(CompileUnit {
        filename: "main.c".to_string(),
        source: source.clone(),
    });

    // 拼接源码
    let mut full_source = String::new();
    for unit in &session.compile.compile_units {
        full_source.push_str(&unit.source);
        if !unit.source.ends_with('\n') {
            full_source.push('\n');
        }
    }

    // 1. Lexer
    let (tokens, lex_errors) = Lexer::new(full_source.clone()).tokenize();
    if !lex_errors.is_empty() {
        push_diagnostics(&mut session, &lex_errors, &full_source);
        return CompileResult {
            success: false,
            diagnostics: session.compile.diagnostics.clone(),
            algorithm_matches: Vec::new(),
        };
    }

    // 2. Parser
    let (maybe_program, parse_errors) = Parser::new(tokens).parse();
    if !parse_errors.is_empty() {
        push_diagnostics(&mut session, &parse_errors, &full_source);
        return CompileResult {
            success: false,
            diagnostics: session.compile.diagnostics.clone(),
            algorithm_matches: Vec::new(),
        };
    }

    let mut program = match maybe_program {
        Some(p) => p,
        None => {
            session.compile.errors = "解析失败：无法生成 AST".to_string();
            session.compile.errors_buffer = session.compile.errors.clone();
            return CompileResult {
                success: false,
                diagnostics: session.compile.diagnostics.clone(),
                algorithm_matches: Vec::new(),
            };
        }
    };

    // 3. TypeChecker
    let (type_errors, type_warnings, type_hints) = TypeChecker::new().check(&mut program);
    if !type_errors.is_empty() {
        push_diagnostics(&mut session, &type_errors, &full_source);
        return CompileResult {
            success: false,
            diagnostics: session.compile.diagnostics.clone(),
            algorithm_matches: Vec::new(),
        };
    }
    if !type_warnings.is_empty() {
        push_warnings(&mut session, &type_warnings, &full_source);
    }
    if !type_hints.is_empty() {
        push_hints(&mut session, &type_hints, &full_source);
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
            return CompileResult {
                success: false,
                diagnostics: session.compile.diagnostics.clone(),
                algorithm_matches: Vec::new(),
            };
        }
    };

    // 填充编译结果
    session.compile.bytecode = output.code;
    session.compile.globals_init = output.globals_init;
    session.compile.source_map = output
        .source_map
        .into_iter()
        .map(|(ip, loc)| (ip, crate::compiler::ast::SourceLoc { line: loc.line, column: loc.column }))
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
    collect_algorithm_matches(&mut session, &program);

    session.compile.compiled = true;
    session.compile.errors.clear();
    session.compile.errors_buffer.clear();

    CompileResult {
        success: true,
        diagnostics: session.compile.diagnostics.clone(),
        algorithm_matches: session.compile.algorithm_matches.clone(),
    }
}

/// 全速运行已编译的程序
pub fn run_code() -> RunResult {
    let mut session = SESSION.lock().unwrap();

    if !session.compile.compiled {
        return RunResult {
            success: false,
            output: String::new(),
            waiting_input: false,
            error: Some("程序尚未编译。请先编译代码。".to_string()),
        };
    }

    let is_resume = session.runtime.waiting_input;

    if !is_resume {
        session.runtime.output_lines.clear();
        session.runtime.error.clear();
        session.runtime.trace.clear();
        session.memory.regions.clear();
        session.memory.free_list.clear();
        session.memory.heap_offset = 0x5000;
        session.memory.alloc_counter = 0;
        session.runtime.running = true;
    }
    session.runtime.step_mode = false;
    session.runtime.waiting_input = false;

    let mut vm = session.vm.take().unwrap_or_default();
    if !is_resume {
        setup_vm(&mut vm, &session);
    } else {
        vm.resume();
    }

    let ret = vm.run(&mut session);

    let result = if vm.has_error() {
        session.runtime.error = vm.get_error().to_string();
        session.runtime.running = false;
        RunResult {
            success: false,
            output: session.runtime.output_lines.join("\n"),
            waiting_input: false,
            error: Some(session.runtime.error.clone()),
        }
    } else if session.runtime.waiting_input {
        session.vm = Some(vm);
        return RunResult {
            success: true,
            output: session.runtime.output_lines.join("\n"),
            waiting_input: true,
            error: None,
        };
    } else {
        session.runtime.output_lines.push(format!("程序运行完成，返回值：{}\n", ret));
        session.runtime.running = false;
        RunResult {
            success: true,
            output: session.runtime.output_lines.join("\n"),
            waiting_input: false,
            error: None,
        }
    };
    session.vm = Some(vm);
    result
}

/// 单步执行
pub fn step_next() -> StepResult {
    let mut session = SESSION.lock().unwrap();

    if !session.compile.compiled {
        return StepResult {
            status: StepStatus::Trap,
            current_line: 0,
            output: String::new(),
            waiting_input: false,
        };
    }

    let mut vm = session.vm.take().unwrap_or_default();
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

        setup_vm(&mut vm, &session);
        vm.pause();

        session.runtime.waiting_input = false;
        loop {
            match vm.step(&mut session) {
                crate::vm::vm::StepResult::Ok => {
                    // 继续执行
                }
                crate::vm::vm::StepResult::Paused => {
                    session.runtime.current_line = vm.get_current_line();
                    break StepResult {
                        status: StepStatus::Paused,
                        current_line: session.runtime.current_line,
                        output: session.runtime.output_lines.join("\n"),
                        waiting_input: false,
                    };
                }
                crate::vm::vm::StepResult::WaitingInput => {
                    session.runtime.current_line = vm.get_current_line();
                    break StepResult {
                        status: StepStatus::WaitingInput,
                        current_line: session.runtime.current_line,
                        output: session.runtime.output_lines.join("\n"),
                        waiting_input: true,
                    };
                }
                crate::vm::vm::StepResult::Finished => {
                    session.runtime.running = false;
                    session.runtime.current_line = vm.get_current_line();
                    break StepResult {
                        status: StepStatus::Finished,
                        current_line: session.runtime.current_line,
                        output: session.runtime.output_lines.join("\n"),
                        waiting_input: false,
                    };
                }
                crate::vm::vm::StepResult::Trap => {
                    session.runtime.error = vm.get_error().to_string();
                    session.runtime.running = false;
                    session.runtime.current_line = vm.get_current_line();
                    break StepResult {
                        status: StepStatus::Trap,
                        current_line: session.runtime.current_line,
                        output: session.runtime.output_lines.join("\n"),
                        waiting_input: false,
                    };
                }
            }
        }
    } else {
        match vm.step(&mut session) {
            crate::vm::vm::StepResult::Ok => {
                session.runtime.current_line = vm.get_current_line();
                StepResult {
                    status: StepStatus::Paused,
                    current_line: session.runtime.current_line,
                    output: session.runtime.output_lines.join("\n"),
                    waiting_input: false,
                }
            }
            crate::vm::vm::StepResult::Paused => {
                session.runtime.current_line = vm.get_current_line();
                StepResult {
                    status: StepStatus::Paused,
                    current_line: session.runtime.current_line,
                    output: session.runtime.output_lines.join("\n"),
                    waiting_input: false,
                }
            }
            crate::vm::vm::StepResult::WaitingInput => {
                session.runtime.current_line = vm.get_current_line();
                StepResult {
                    status: StepStatus::WaitingInput,
                    current_line: session.runtime.current_line,
                    output: session.runtime.output_lines.join("\n"),
                    waiting_input: true,
                }
            }
            crate::vm::vm::StepResult::Finished => {
                session.runtime.running = false;
                session.runtime.current_line = vm.get_current_line();
                StepResult {
                    status: StepStatus::Finished,
                    current_line: session.runtime.current_line,
                    output: session.runtime.output_lines.join("\n"),
                    waiting_input: false,
                }
            }
            crate::vm::vm::StepResult::Trap => {
                session.runtime.error = vm.get_error().to_string();
                session.runtime.running = false;
                session.runtime.current_line = vm.get_current_line();
                StepResult {
                    status: StepStatus::Trap,
                    current_line: session.runtime.current_line,
                    output: session.runtime.output_lines.join("\n"),
                    waiting_input: false,
                }
            }
        }
    };
    session.vm = Some(vm);
    result
}

/// 获取诊断信息
pub fn get_diagnostics() -> Vec<Diagnostic> {
    let session = SESSION.lock().unwrap();
    session.compile.diagnostics.clone()
}

/// 获取算法匹配
pub fn get_algorithm_matches() -> Vec<AlgorithmMatch> {
    let session = SESSION.lock().unwrap();
    session.compile.algorithm_matches.clone()
}

/// 获取变量列表
pub fn get_variables() -> Vec<VariableSnapshot> {
    let session = SESSION.lock().unwrap();
    session.runtime.variable_snapshot.clone()
}

/// 获取内存区域
pub fn get_memory_regions() -> Vec<MemoryRegion> {
    let session = SESSION.lock().unwrap();
    session.memory.regions.clone()
}

/// 获取调用栈
pub fn get_callstack() -> Vec<TraceEntry> {
    let session = SESSION.lock().unwrap();
    session.runtime.trace.clone()
}

/// 获取输出
pub fn get_output() -> String {
    let session = SESSION.lock().unwrap();
    session.runtime.output_lines.join("\n")
}

/// 获取当前行
pub fn get_current_line() -> i32 {
    let session = SESSION.lock().unwrap();
    session.runtime.current_line
}

/// 是否等待输入
pub fn is_waiting_input() -> bool {
    let session = SESSION.lock().unwrap();
    session.runtime.waiting_input
}

/// 添加断点
pub fn add_breakpoint(line: i32) {
    let mut session = SESSION.lock().unwrap();
    if let Some(ref mut vm) = session.vm {
        vm.add_breakpoint(line);
    }
}

/// 清除所有断点
pub fn clear_breakpoints() {
    let mut session = SESSION.lock().unwrap();
    if let Some(ref mut vm) = session.vm {
        vm.clear_breakpoints();
    }
}

/// 设置输入（用于 scanf）
pub fn set_input(input: String) {
    let mut session = SESSION.lock().unwrap();
    session.runtime.input_lines = input
        .lines()
        .map(|l| l.trim_end_matches('\r').to_string())
        .collect();
    session.runtime.input_index = 0;
    session.runtime.input_char_offset = 0;
}

/// 提供单行输入（恢复执行）
pub fn provide_input_line(line: String) {
    let mut session = SESSION.lock().unwrap();
    session.runtime.input_lines.push(line);
    session.runtime.waiting_input = false;
    if let Some(ref mut vm) = session.vm {
        vm.resume();
    }
}

/// 获取可视化事件
pub fn get_vis_events() -> Vec<VisEvent> {
    let session = SESSION.lock().unwrap();
    session.runtime.vis_event_cache.clone()
}

/// 清除可视化事件
pub fn clear_vis_events() {
    let mut session = SESSION.lock().unwrap();
    session.runtime.vis_event_cache.clear();
}

/// 读取 VM 内存（按 i32 数组返回）
pub fn read_memory(addr: u32, count: u32) -> Vec<i32> {
    let session = SESSION.lock().unwrap();
    if let Some(ref vm) = session.vm {
        let mem = vm.memory_ref();
        let mut result = Vec::new();
        for i in 0..count {
            let offset = (addr + i * 4) as usize;
            if offset + 4 <= mem.len() {
                let bytes = &mem[offset..offset + 4];
                let val = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                result.push(val);
            } else {
                result.push(0);
            }
        }
        result
    } else {
        Vec::new()
    }
}

/// 重置会话
pub fn reset_session() {
    let mut session = SESSION.lock().unwrap();
    *session = Session::default();
}
