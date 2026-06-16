//! Flutter-Rust Bridge 包装层
//!
//! 为 Flutter 前端提供类型安全、异步友好的 Rust API。
//! 现有 C API (`capi/mod.rs`) 完全保留，此模块仅服务于 Flutter 前端。
// TODO(#D10): 统一模式大状态仍通过同步 API 返回，未来应评估完整 Stream + 差分路径。

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::session::{CodeFile, *};
use crate::unified::engine::UnifiedEngine;
use crate::unified::types::*;
use crate::vm::core::NULL_TRAP_SIZE;

// ========== 多 Session 管理 ==========

use std::sync::LazyLock;

/// Mutex poison 发生次数。用于工程健康度监控与问题定位。
static POISON_COUNT: AtomicU64 = AtomicU64::new(0);

/// 锁定 Arc<Mutex<T>>；若 Mutex 已被 poison，则重置为默认值后返回 guard，
/// 避免继续使用可能已损坏的状态。
// NOTE(#D12): 当前策略仍为恢复默认值以保证前端不崩溃，但已增加调用位置与全局计数，
// 不再静默处理。未来若监控显示 poison 频繁发生，应升级为 panic 或上报崩溃报告。
#[track_caller]
fn lock_or_reset<T: Default>(arc: &Arc<Mutex<T>>) -> std::sync::MutexGuard<'_, T> {
    arc.lock().unwrap_or_else(|e| {
        let count = POISON_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
        let caller = std::panic::Location::caller();
        eprintln!(
            "[cide] Mutex poison detected (total={}) at {}. Resetting state to default; original panic context is lost.",
            count, caller
        );
        let mut guard = e.into_inner();
        *guard = T::default();
        guard
    })
}

static SESSIONS: LazyLock<Mutex<HashMap<u64, Arc<Mutex<Session>>>>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    let session = Arc::new(Mutex::new(Session::default()));
    map.insert(0, session);
    Mutex::new(map)
});

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);
static CURRENT_SESSION_ID: AtomicU64 = AtomicU64::new(0);

static UNIFIED_ENGINES: LazyLock<Mutex<HashMap<u64, Arc<Mutex<UnifiedEngine>>>>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    let engine = Arc::new(Mutex::new(UnifiedEngine::new()));
    map.insert(0, engine);
    Mutex::new(map)
});

fn current_unified_engine() -> Arc<Mutex<UnifiedEngine>> {
    let id = CURRENT_SESSION_ID.load(Ordering::SeqCst);
    let mut engines = UNIFIED_ENGINES.lock().unwrap_or_else(|e| e.into_inner());
    let engine_ref: Arc<Mutex<UnifiedEngine>> =
        engines.get(&id).or_else(|| engines.get(&0)).cloned().unwrap_or_else(|| {
            let e = Arc::new(Mutex::new(UnifiedEngine::new()));
            engines.insert(id, e.clone());
            e
        });
    engine_ref
}

/// 创建新 Session，返回唯一 ID
pub fn create_session() -> u64 {
    let id = NEXT_SESSION_ID.fetch_add(1, Ordering::SeqCst);
    let session = Arc::new(Mutex::new(Session::default()));
    let mut sessions = SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
    sessions.insert(id, session);
    id
}

/// 销毁指定 Session
pub fn destroy_session(session_id: u64) {
    let mut sessions = SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
    sessions.remove(&session_id);
    let mut engines = UNIFIED_ENGINES.lock().unwrap_or_else(|e| e.into_inner());
    engines.remove(&session_id);
}

/// 切换当前操作的 Session ID
pub fn set_current_session_id(session_id: u64) {
    CURRENT_SESSION_ID.store(session_id, Ordering::SeqCst);
}

/// 获取当前 Session ID
pub fn get_current_session_id() -> u64 {
    CURRENT_SESSION_ID.load(Ordering::SeqCst)
}

fn current_session() -> Arc<Mutex<Session>> {
    let id = CURRENT_SESSION_ID.load(Ordering::SeqCst);
    let mut sessions = SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
    let session_ref: Arc<Mutex<Session>> =
        sessions.get(&id).or_else(|| sessions.get(&0)).cloned().unwrap_or_else(|| {
            let s = Arc::new(Mutex::new(Session::default()));
            sessions.insert(id, s.clone());
            s
        });
    session_ref
}

// ========== 辅助函数 ==========

use crate::engine::compile_pipeline::{run_multi_file_pipeline, setup_vm};
use crate::engine::session_ops::{execute_run, inject_preset_files, reset_runtime_for_step};

// ========== 公开 API ==========

/// 设置源码并编译（单文件，向后兼容）
pub fn compile(source: String) -> CompileResult {
    compile_multi(vec![CodeFile {
        filename: "main.c".to_string(),
        source,
    }])
}

/// 多文件编译
pub fn compile_multi(files: Vec<CodeFile>) -> CompileResult {
    let session_arc_l111 = current_session();
    let mut session = lock_or_reset(&session_arc_l111);

    // 设置编译单元
    session.compile.compile_units.clear();
    for file in &files {
        session.compile.compile_units.push(CompileUnit {
            filename: file.filename.clone(),
            source: file.source.clone(),
        });
    }

    // 运行多文件编译管线
    let units = session.compile.compile_units.clone();
    if run_multi_file_pipeline(&mut session, units, false).is_err() {
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

/// 设置命令行参数（供 `main(int argc, char *argv[])` 使用）。
/// 应在 `compile` 之后、`run_code`/`step_next` 之前调用。
pub fn set_argv(argv: Vec<String>) {
    let session_arc = current_session();
    let mut session = lock_or_reset(&session_arc);
    session.runtime.argc = argv.len() as i32;
    session.runtime.argv = argv;
}

/// 全速运行已编译的程序
pub fn run_code() -> RunResult {
    let session_arc_l141 = current_session();
    let mut session = lock_or_reset(&session_arc_l141);

    if !session.compile.compiled {
        return RunResult {
            success: false,
            output: String::new(),
            waiting_input: false,
            error: Some("程序尚未编译。请先编译代码。".to_string()),
        };
    }

    match execute_run(&mut session) {
        Ok((_, true)) => RunResult {
            success: true,
            output: session.runtime.output(),
            waiting_input: true,
            error: None,
        },
        Ok((_, false)) => RunResult {
            success: true,
            output: session.runtime.output(),
            waiting_input: false,
            error: None,
        },
        Err(e) => RunResult {
            success: false,
            output: session.runtime.output(),
            waiting_input: false,
            error: Some(e),
        },
    }
}

/// 单步执行
pub fn step_next() -> StepResult {
    let session_arc_l176 = current_session();
    let mut session = lock_or_reset(&session_arc_l176);

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
        reset_runtime_for_step(&mut session);
        setup_vm(&mut vm, &session);
        inject_preset_files(&mut vm, &mut session);
        vm.pause();
        session.runtime.waiting_input = false;
        loop {
            match vm.step(&mut session) {
                crate::vm::core::StepResult::Ok => {
                    // 首次运行：遇到第一个 StepEvent 后暂停，避免无断点时持续执行到 max_steps
                    if vm.was_step_event_hit() {
                        session.runtime.current_line = vm.get_current_line();
                        break StepResult {
                            status: StepStatus::Paused,
                            current_line: session.runtime.current_line,
                            output: session.runtime.output(),
                            waiting_input: false,
                        };
                    }
                }
                crate::vm::core::StepResult::Paused => {
                    session.runtime.current_line = vm.get_current_line();
                    break StepResult {
                        status: StepStatus::Paused,
                        current_line: session.runtime.current_line,
                        output: session.runtime.output(),
                        waiting_input: false,
                    };
                }
                crate::vm::core::StepResult::WaitingInput => {
                    session.runtime.current_line = vm.get_current_line();
                    break StepResult {
                        status: StepStatus::WaitingInput,
                        current_line: session.runtime.current_line,
                        output: session.runtime.output(),
                        waiting_input: true,
                    };
                }
                crate::vm::core::StepResult::Finished => {
                    session.runtime.running = false;
                    session.runtime.current_line = vm.get_current_line();
                    break StepResult {
                        status: StepStatus::Finished,
                        current_line: session.runtime.current_line,
                        output: session.runtime.output(),
                        waiting_input: false,
                    };
                }
                crate::vm::core::StepResult::Trap => {
                    session.runtime.error = vm.get_error().to_string();
                    session.runtime.running = false;
                    session.runtime.current_line = vm.get_current_line();
                    break StepResult {
                        status: StepStatus::Trap,
                        current_line: session.runtime.current_line,
                        output: session.runtime.output(),
                        waiting_input: false,
                    };
                }
            }
        }
    } else {
        match vm.step(&mut session) {
            crate::vm::core::StepResult::Ok => {
                session.runtime.current_line = vm.get_current_line();
                StepResult {
                    status: StepStatus::Paused,
                    current_line: session.runtime.current_line,
                    output: session.runtime.output(),
                    waiting_input: false,
                }
            }
            crate::vm::core::StepResult::Paused => {
                session.runtime.current_line = vm.get_current_line();
                StepResult {
                    status: StepStatus::Paused,
                    current_line: session.runtime.current_line,
                    output: session.runtime.output(),
                    waiting_input: false,
                }
            }
            crate::vm::core::StepResult::WaitingInput => {
                session.runtime.current_line = vm.get_current_line();
                StepResult {
                    status: StepStatus::WaitingInput,
                    current_line: session.runtime.current_line,
                    output: session.runtime.output(),
                    waiting_input: true,
                }
            }
            crate::vm::core::StepResult::Finished => {
                session.runtime.running = false;
                session.runtime.current_line = vm.get_current_line();
                StepResult {
                    status: StepStatus::Finished,
                    current_line: session.runtime.current_line,
                    output: session.runtime.output(),
                    waiting_input: false,
                }
            }
            crate::vm::core::StepResult::Trap => {
                session.runtime.error = vm.get_error().to_string();
                session.runtime.running = false;
                session.runtime.current_line = vm.get_current_line();
                StepResult {
                    status: StepStatus::Trap,
                    current_line: session.runtime.current_line,
                    output: session.runtime.output(),
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
    let session_arc_l307 = current_session();
    let session = lock_or_reset(&session_arc_l307);
    session.compile.diagnostics.clone()
}

/// 获取算法匹配
pub fn get_algorithm_matches() -> Vec<AlgorithmMatch> {
    let session_arc_l313 = current_session();
    let session = lock_or_reset(&session_arc_l313);
    session.compile.algorithm_matches.clone()
}

/// 获取变量列表
pub fn get_variables() -> Vec<VariableSnapshot> {
    let session_arc_l319 = current_session();
    let session = lock_or_reset(&session_arc_l319);
    session.vm.as_ref().map(|vm| vm.get_variable_snapshot()).unwrap_or_default()
}

/// 获取内存区域
pub fn get_memory_regions() -> Vec<MemoryRegion> {
    let session_arc_l325 = current_session();
    let session = lock_or_reset(&session_arc_l325);
    session.memory.regions.clone()
}

/// 获取当前空闲碎片块（外部碎片）
pub fn get_memory_fragments() -> Vec<crate::session::MemoryFragment> {
    let session_arc_l331 = current_session();
    let session = lock_or_reset(&session_arc_l331);
    session
        .memory
        .free_list
        .iter()
        .map(|b| crate::session::MemoryFragment { addr: b.addr, size: b.size })
        .collect()
}

/// 获取堆内存统计信息（总空间、已分配、碎片、碎片率）
pub fn get_heap_stats() -> crate::session::HeapStats {
    let session_arc_l342 = current_session();
    let session = lock_or_reset(&session_arc_l342);
    let mem = &session.memory;
    let heap_start = crate::vm::core::HEAP_START as i32;
    let total_heap = (mem.heap_offset as i32).saturating_sub(heap_start);
    let allocated: i32 = mem.regions.iter().filter(|r| r.is_heap && !r.is_freed).map(|r| r.size).sum();
    let fragmented: i32 = mem.free_list.iter().map(|b| b.size).sum();
    let fragmentation_rate = if total_heap > 0 {
        ((fragmented as f64 / total_heap as f64) * 100.0) as i32
    } else {
        0
    };
    crate::session::HeapStats {
        total_heap,
        allocated,
        fragmented,
        fragmentation_rate,
    }
}

/// 获取 VM 内存总大小（字节）
pub fn get_memory_size() -> u32 {
    let session_arc_l363 = current_session();
    let session = lock_or_reset(&session_arc_l363);
    session.vm.as_ref().map(|v| v.get_memory_size()).unwrap_or(0)
}

/// 获取调用栈
pub fn get_callstack() -> Vec<TraceEntry> {
    let session_arc_l369 = current_session();
    let session = lock_or_reset(&session_arc_l369);
    session.runtime.trace.clone()
}

/// 获取输出
pub fn get_output() -> String {
    let session_arc_l375 = current_session();
    let session = lock_or_reset(&session_arc_l375);
    session.runtime.output()
}

/// 获取当前行
pub fn get_current_line() -> i32 {
    let session_arc_l381 = current_session();
    let session = lock_or_reset(&session_arc_l381);
    session.runtime.current_line
}

/// 是否等待输入
pub fn is_waiting_input() -> bool {
    let session_arc_l387 = current_session();
    let session = lock_or_reset(&session_arc_l387);
    session.runtime.waiting_input
}

/// 添加断点
pub fn add_breakpoint(line: i32) {
    let session_arc_l393 = current_session();
    let mut session = lock_or_reset(&session_arc_l393);
    if let Some(ref mut vm) = session.vm {
        vm.add_breakpoint(line);
    }
}

/// 清除所有断点
pub fn clear_breakpoints() {
    let session_arc_l401 = current_session();
    let mut session = lock_or_reset(&session_arc_l401);
    if let Some(ref mut vm) = session.vm {
        vm.clear_breakpoints();
    }
}

/// 批量设置断点（清除后重新添加）
pub fn set_breakpoints(lines: Vec<i32>) {
    let session_arc_l409 = current_session();
    let mut session = lock_or_reset(&session_arc_l409);
    if let Some(ref mut vm) = session.vm {
        vm.clear_breakpoints();
        for line in lines {
            vm.add_breakpoint(line);
        }
    }
}

/// 设置输入（用于 scanf）
pub fn set_input(input: String) {
    let session_arc_l420 = current_session();
    let mut session = lock_or_reset(&session_arc_l420);
    session.runtime.input_lines = input.lines().map(|l| l.trim_end_matches('\r').to_string()).collect();
    session.runtime.input_index = 0;
    session.runtime.input_char_offset = 0;
}

/// 提供单行输入（恢复执行）
pub fn provide_input_line(line: String) {
    let session_arc_l428 = current_session();
    let mut session = lock_or_reset(&session_arc_l428);
    session.runtime.input_lines.push(line);
    session.runtime.waiting_input = false;
    if let Some(ref mut vm) = session.vm {
        vm.resume();
    }
}

/// 获取可视化事件
pub fn get_vis_events() -> Vec<VisEvent> {
    let session_arc_l438 = current_session();
    let session = lock_or_reset(&session_arc_l438);
    session.runtime.vis_event_cache.clone()
}

/// 清除可视化事件
pub fn clear_vis_events() {
    let session_arc_l444 = current_session();
    let mut session = lock_or_reset(&session_arc_l444);
    session.runtime.vis_event_cache.clear();
}

/// 读取 VM 内存（按 i32 数组返回）
pub fn read_memory(addr: u32, count: u32) -> Vec<i32> {
    let session_arc_l450 = current_session();
    let session = lock_or_reset(&session_arc_l450);
    if let Some(ref vm) = session.vm {
        let mem = vm.memory_ref();
        let mut result = Vec::new();
        for i in 0..count {
            let word_addr = addr + i * 4;
            // 统一 NULL 区检查；越界返回 0。
            if word_addr >= NULL_TRAP_SIZE && word_addr as u64 + 4 <= mem.len() as u64 {
                let offset = word_addr as usize;
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
    let session_arc_l474 = current_session();
    let session = lock_or_reset(&session_arc_l474);
    session.compile.struct_fields.get(&name).cloned().unwrap_or_default()
}

/// 重置会话
pub fn reset_session() {
    let session_arc_l480 = current_session();
    let mut session = lock_or_reset(&session_arc_l480);
    *session = Session::default();
    let engine_arc_l482 = current_unified_engine();
    let mut engine = lock_or_reset(&engine_arc_l482);
    engine.reset();
}

// ========== 统一模式 API ==========

/// 编译并初始化统一模式执行环境（单文件，向后兼容）
pub fn compile_and_run(source: String) -> UnifiedRunResult {
    compile_and_run_multi(vec![CodeFile {
        filename: "main.c".to_string(),
        source,
    }])
}

/// 多文件编译并初始化统一模式执行环境
pub fn compile_and_run_multi(files: Vec<CodeFile>) -> UnifiedRunResult {
    let session_arc_l498 = current_session();
    let mut session = lock_or_reset(&session_arc_l498);

    // 编译
    session.compile.compile_units.clear();
    for file in &files {
        session.compile.compile_units.push(CompileUnit {
            filename: file.filename.clone(),
            source: file.source.clone(),
        });
    }

    let units = session.compile.compile_units.clone();
    if run_multi_file_pipeline(&mut session, units, false).is_err() {
        return UnifiedRunResult {
            success: false,
            error: Some(session.compile.errors.clone()),
            total_steps: 0,
            finished: false,
        };
    }

    // 重置统一模式引擎
    let engine_arc_l520 = current_unified_engine();
    let mut engine = lock_or_reset(&engine_arc_l520);
    engine.reset();

    // 初始化 VM
    let mut vm = session.vm.take().unwrap_or_default();
    reset_runtime_for_step(&mut session);
    setup_vm(&mut vm, &session);
    inject_preset_files(&mut vm, &mut session);
    session.runtime.running = true;

    // 保存初始检查点（第 0 步）
    engine.checkpoints.save(0, &mut vm, &session);

    session.vm = Some(vm);

    UnifiedRunResult {
        success: true,
        error: None,
        total_steps: 0,
        finished: false,
    }
}

/// 批量自动执行。
pub fn run_auto_steps(batch_size: i32) -> AutoStepResult {
    let session_arc_l545 = current_session();
    let mut session = lock_or_reset(&session_arc_l545);
    let engine_arc_l546 = current_unified_engine();
    let mut engine = lock_or_reset(&engine_arc_l546);

    let mut vm = session.vm.take().unwrap_or_default();
    let result = match engine.run_batch(&mut vm, &mut session, batch_size) {
        Ok(r) => r,
        Err(e) => {
            let line = vm.get_current_line();
            session.vm = Some(vm);
            return AutoStepResult {
                payloads: Vec::new(),
                finished: false,
                trapped: true,
                waiting_input: false,
                paused: false,
                current_line: line,
                trap_message: Some(e),
                cache_start_step: engine.frame_cache_start_step(),
            };
        }
    };

    session.vm = Some(vm);
    result
}

/// Seek 到指定步。
pub fn seek_to_step(target: i32) -> SeekResult {
    let session_arc_l572 = current_session();
    let mut session = lock_or_reset(&session_arc_l572);
    let engine_arc_l573 = current_unified_engine();
    let mut engine = lock_or_reset(&engine_arc_l573);

    let mut vm = session.vm.take().unwrap_or_default();
    let result = engine.seek_to(target, &mut vm, &mut session);

    session.vm = Some(vm);
    result
}

/// 单步执行（统一模式）。
pub fn step_next_unified() -> Option<StepPayload> {
    let session_arc_l584 = current_session();
    let mut session = lock_or_reset(&session_arc_l584);
    let engine_arc_l585 = current_unified_engine();
    let mut engine = lock_or_reset(&engine_arc_l585);

    let mut vm = session.vm.take().unwrap_or_default();
    let step = vm.get_executed_steps();

    let payload = match vm.step(&mut session) {
        crate::vm::core::StepResult::Ok
        | crate::vm::core::StepResult::Paused
        | crate::vm::core::StepResult::WaitingInput
        | crate::vm::core::StepResult::Finished
        | crate::vm::core::StepResult::Trap => {
            let p = crate::unified::collector::StepCollector::collect(&mut vm, &session, step);
            if let Some(idx) = engine.frame_cache_index(step) {
                // 目标步在当前窗口内：替换
                engine.frame_cache[idx] = p.clone();
            } else if step == engine.max_collected_step() + 1 {
                // 目标步是窗口下一帧：追加并可能触发截断
                engine.frame_cache.push(p.clone());
                engine.trim_frame_cache();
            } else {
                // 其他情况（如窗口外的旧步）：重置窗口为仅包含当前步。
                engine.frame_cache_start_step = step;
                engine.frame_cache.clear();
                engine.frame_cache.push(p.clone());
            }
            Some(p)
        }
    };

    session.vm = Some(vm);
    payload
}

/// 暂停自动执行。
pub fn pause_execution() {
    let engine_arc_l612 = current_unified_engine();
    let mut engine = lock_or_reset(&engine_arc_l612);
    engine.pause();
}

/// 恢复自动执行。
pub fn resume_execution() {
    let engine_arc_l618 = current_unified_engine();
    let mut engine = lock_or_reset(&engine_arc_l618);
    engine.resume();
}

/// 获取执行热力图。
pub fn get_heatmap() -> HeatmapData {
    let session_arc_l624 = current_session();
    let session = lock_or_reset(&session_arc_l624);
    let line_counts: Vec<(i32, u64)> = session.runtime.heatmap.line_counts.iter().map(|(&k, &v)| (k, v)).collect();
    let max_count = session.runtime.heatmap.max_count();
    HeatmapData { line_counts, max_count }
}

/// 获取指定范围的 StepPayload。
///
/// 为避免一次性返回过大窗口导致前端卡顿，单次请求上限为 1000 步。
pub fn get_step_payloads(start: i32, end: i32) -> Vec<StepPayload> {
    const MAX_WINDOW: i32 = 1000;
    let start = start.max(0);
    let mut end = end.max(start);
    if end - start > MAX_WINDOW {
        end = start + MAX_WINDOW;
    }
    let engine_arc_l632 = current_unified_engine();
    let engine = lock_or_reset(&engine_arc_l632);
    engine.get_payloads(start, end)
}

/// 获取当前 frame_cache 窗口起始步号。
pub fn get_frame_cache_start_step() -> i32 {
    let engine_arc_l640 = current_unified_engine();
    let engine = lock_or_reset(&engine_arc_l640);
    engine.frame_cache_start_step()
}

/// Stream 模式批量自动执行。
pub fn run_auto_steps_stream(
    sink: crate::frb_generated::StreamSink<crate::unified::stream::StepStreamBatch>,
    batch_size: i32,
) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::spawn(move || {
            run_auto_steps_stream_loop(sink, batch_size);
        });
    }

    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(async move {
            run_auto_steps_stream_loop(sink, batch_size);
        });
    }
}

fn run_auto_steps_stream_loop(
    sink: crate::frb_generated::StreamSink<crate::unified::stream::StepStreamBatch>,
    batch_size: i32,
) {
    loop {
        let result = run_auto_steps(batch_size);
        let should_stop = result.finished || result.trapped || result.waiting_input || result.paused;

        // 将 payloads 编码为优化后的 StepStreamBatch
        let cache_start_step = {
            let engine_arc_l546 = current_unified_engine();
            let engine = lock_or_reset(&engine_arc_l546);
            engine.frame_cache_start_step()
        };
        let mut batch = crate::unified::stream::encode_payloads(&result.payloads, cache_start_step);
        batch.finished = result.finished;
        batch.trapped = result.trapped;
        batch.waiting_input = result.waiting_input;
        batch.paused = result.paused;
        batch.current_line = result.current_line;
        batch.trap_message = result.trap_message;

        if sink.add(batch).is_err() {
            break;
        }
        if should_stop {
            break;
        }
    }
}

/// 从指定步继续执行。
pub fn continue_from_step(step: i32) -> UnifiedRunResult {
    let session_arc_l667 = current_session();
    let mut session = lock_or_reset(&session_arc_l667);
    let engine_arc_l668 = current_unified_engine();
    let mut engine = lock_or_reset(&engine_arc_l668);

    let mut vm = session.vm.take().unwrap_or_default();

    let seek_result = engine.seek_to(step, &mut vm, &mut session);
    if !seek_result.success {
        session.vm = Some(vm);
        return UnifiedRunResult {
            success: false,
            error: seek_result.error,
            total_steps: 0,
            finished: false,
        };
    }

    engine.resume();

    session.vm = Some(vm);
    UnifiedRunResult {
        success: true,
        error: None,
        total_steps: engine.max_collected_step(),
        finished: false,
    }
}

// ========== 智能补全 API ==========

use crate::engine::completion::CompletionCandidate;

/// 获取光标位置的语义补全候选
pub fn get_completion_candidates(source: String, line: i32, column: i32, prefix: String) -> Vec<CompletionCandidate> {
    let session_arc_l700 = current_session();
    let session = lock_or_reset(&session_arc_l700);
    crate::engine::completion::get_completion_candidates(&session, &source, line as usize, column as usize, &prefix)
}
