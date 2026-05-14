//! Flutter-Rust Bridge 包装层
//!
//! 为 Flutter 前端提供类型安全、异步友好的 Rust API。
//! 现有 C API (`capi/mod.rs`) 完全保留，此模块仅服务于 Flutter 前端。

use std::sync::Mutex;

use crate::session::*;

// ========== 全局 Session ==========

use std::sync::LazyLock;

static SESSION: LazyLock<Mutex<Session>> = LazyLock::new(|| {
    Mutex::new(Session::default())
});

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

use crate::engine::compile_pipeline::{run_compile_pipeline, setup_vm};

// ========== 公开 API ==========

/// 设置源码并编译
pub fn compile(source: String) -> CompileResult {
    let mut session = SESSION.lock().unwrap_or_else(|e| e.into_inner());

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

    // 运行编译管线
    if run_compile_pipeline(&mut session, &full_source).is_err() {
        return CompileResult {
            success: false,
            diagnostics: session.compile.diagnostics.clone(),
            algorithm_matches: Vec::new(),
        };
    }

    CompileResult {
        success: true,
        diagnostics: session.compile.diagnostics.clone(),
        algorithm_matches: session.compile.algorithm_matches.clone(),
    }
}

/// 全速运行已编译的程序
pub fn run_code() -> RunResult {
    let mut session = SESSION.lock().unwrap_or_else(|e| e.into_inner());

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
    let mut session = SESSION.lock().unwrap_or_else(|e| e.into_inner());

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
    let session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    session.compile.diagnostics.clone()
}

/// 获取算法匹配
pub fn get_algorithm_matches() -> Vec<AlgorithmMatch> {
    let session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    session.compile.algorithm_matches.clone()
}

/// 获取变量列表
pub fn get_variables() -> Vec<VariableSnapshot> {
    let session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    session.runtime.variable_snapshot.clone()
}

/// 获取内存区域
pub fn get_memory_regions() -> Vec<MemoryRegion> {
    let session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    session.memory.regions.clone()
}

/// 获取调用栈
pub fn get_callstack() -> Vec<TraceEntry> {
    let session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    session.runtime.trace.clone()
}

/// 获取输出
pub fn get_output() -> String {
    let session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    session.runtime.output_lines.join("\n")
}

/// 获取当前行
pub fn get_current_line() -> i32 {
    let session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    session.runtime.current_line
}

/// 是否等待输入
pub fn is_waiting_input() -> bool {
    let session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    session.runtime.waiting_input
}

/// 添加断点
pub fn add_breakpoint(line: i32) {
    let mut session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref mut vm) = session.vm {
        vm.add_breakpoint(line);
    }
}

/// 清除所有断点
pub fn clear_breakpoints() {
    let mut session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref mut vm) = session.vm {
        vm.clear_breakpoints();
    }
}

/// 设置输入（用于 scanf）
pub fn set_input(input: String) {
    let mut session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    session.runtime.input_lines = input
        .lines()
        .map(|l| l.trim_end_matches('\r').to_string())
        .collect();
    session.runtime.input_index = 0;
    session.runtime.input_char_offset = 0;
}

/// 提供单行输入（恢复执行）
pub fn provide_input_line(line: String) {
    let mut session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    session.runtime.input_lines.push(line);
    session.runtime.waiting_input = false;
    if let Some(ref mut vm) = session.vm {
        vm.resume();
    }
}

/// 获取可视化事件
pub fn get_vis_events() -> Vec<VisEvent> {
    let session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    session.runtime.vis_event_cache.clone()
}

/// 清除可视化事件
pub fn clear_vis_events() {
    let mut session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    session.runtime.vis_event_cache.clear();
}

/// 读取 VM 内存（按 i32 数组返回）
pub fn read_memory(addr: u32, count: u32) -> Vec<i32> {
    let session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
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

/// 获取结构体字段定义（字段名, 偏移量）
pub fn get_struct_fields(name: String) -> Vec<(String, i32)> {
    let session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    session.compile.struct_fields.get(&name).cloned().unwrap_or_default()
}

/// 重置会话
pub fn reset_session() {
    let mut session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    *session = Session::default();
}
