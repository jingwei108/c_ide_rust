//! 会话操作的语言中立层：`capi` / `cide_cli serve` / wasm 共用同一套入口语义。
//!
//! 架构纪律（主计划 §2.2 纪律 2）：**三个出口只做薄包装**。此前 capi 第一批把
//! "运行 → JSON"的构造直接写在 C 导出函数内部；若 serve 再写一份，就会重蹈
//! "typeck 与 codegen 双轨语义"的覆辙。
//!
//! 分工：
//! - 本模块：业务语义（何时算 trap、`status` 枚举、诊断字段映射、窗口裁剪、
//!   断点集合写入）——**只此一份**；
//! - 出口：参数编解码 + 所有权（capi 的 rust-alloc C 字符串 / serve 的 NDJSON 行）。
//!
//! 返回 `serde_json::Value` 而非字符串：序列化时机与所有权归出口决定。

use crate::engine::compile_pipeline::{run_multi_file_pipeline, setup_vm};
use crate::engine::session_ops::{execute_run, inject_preset_files, reset_runtime_for_step};
use crate::session::Session;
use crate::unified::engine::UnifiedEngine;
use serde_json::{json, Value};

/// 错误 JSON 的统一形状（与成功帧同构：都有 `ok`，失败时带 `error`）。
pub fn error_json(message: impl Into<String>) -> Value {
    json!({ "ok": false, "error": message.into() })
}

/// 诊断 severity 数值 → schema 枚举字符串。
pub fn severity_name(severity: i32) -> &'static str {
    match severity {
        0 => "error",
        1 => "warning",
        2 => "hint",
        _ => "info",
    }
}

/// 编译当前会话的编译单元，返回诊断 JSON。
///
/// `{"ok":bool,"diagnostics":[{code,error_code,severity,line,column,end_line,end_column,message,fix_suggestion,filename}]}`
pub fn compile(session: &mut Session) -> Value {
    let units = session.compile.compile_units.clone();
    if units.is_empty() {
        return error_json("尚未提供编译单元：请先调用 cide_compile_unit");
    }
    let ok = run_multi_file_pipeline(session, units, false).is_ok();
    let diagnostics: Vec<Value> = session
        .compile
        .diagnostics
        .iter()
        .map(|d| {
            json!({
                "code": format!("E{}", d.error_code),
                "error_code": d.error_code,
                "severity": severity_name(d.severity),
                "line": d.line,
                "column": d.column,
                // 精确跨度需动三处错误结构体；Phase 1 先给"起点 + 1"退化值
                "end_line": d.line,
                "end_column": d.column + 1,
                "message": d.message,
                "fix_suggestion": d.fix_suggestion,
                "filename": d.filename,
            })
        })
        .collect();
    json!({
        "ok": ok,
        "diagnostics": diagnostics,
        // E2 白箱教学层：宏展开链 + #if 分支选择原因（容量封顶，additive）
        "preprocessor_trace": session.compile.preprocessor_trace,
    })
}

/// 全速运行并返回结果 JSON。
///
/// `{"ok":bool,"status":"finished|trap|waiting_input|not_compiled","return_value":n,"trap":"...","waiting_input":bool,"steps_executed":n}`
pub fn run(session: &mut Session) -> Value {
    if !session.compile.compiled {
        session.runtime.error = "程序尚未编译。请先编译代码。".to_string();
        return json!({
            "ok": false,
            "status": "not_compiled",
            "return_value": 0,
            "trap": session.runtime.error,
            "waiting_input": false,
            "steps_executed": 0,
        });
    }

    let (return_value, waiting_input) = match execute_run(session) {
        Ok((code, waiting)) => (code, waiting),
        Err(_) => (
            session.vm.as_ref().map(|vm| vm.exit_code()).unwrap_or(-1),
            false,
        ),
    };
    let steps_executed = session.vm.as_ref().map(|vm| vm.get_step_count()).unwrap_or(0);
    let trap = session.runtime.error.clone();
    let status = if !trap.is_empty() {
        "trap"
    } else if waiting_input {
        "waiting_input"
    } else {
        "finished"
    };
    json!({
        "ok": trap.is_empty(),
        "status": status,
        "return_value": return_value,
        "trap": trap,
        "waiting_input": waiting_input,
        "steps_executed": steps_executed,
    })
}

/// 自 `cursor`（字节偏移）起的输出增量（**展示视图**：含引擎附注，兼容既有消费方）。
///
/// `{"delta":"...","cursor":<新游标>,"total":<总字节>,"stream":"display"}`。
///
/// 需要纯净程序 stdout（判分 / Shadow 比对）的消费方请用 [`output_delta_on`] 并传
/// `"stdout"`——E-P1-5 之前这里只有混装输出，消费方只能靠正则清洗。
pub fn output_delta(session: &Session, cursor: i32) -> Value {
    output_delta_on(session, cursor, "display")
}

/// 按输出通道取增量（E-P1-5）。
///
/// `view` 取值：`"display"`（默认，全部按序拼接，与旧行为一致）/ `"stdout"`（纯程序
/// stdout）/ `"stderr"`（程序 stderr）/ `"note"`（引擎附注：运行完成提示、泄漏报告、
/// 教学诊断）。未知取值退化为 `"display"`，保证不带该参数或传旧值的客户端行为不变。
///
/// 游标越界按末尾处理；落入多字节字符中间时前移到下一个字符边界，保证 `delta` 为合法 UTF-8。
pub fn output_delta_on(session: &Session, cursor: i32, view: &str) -> Value {
    let (stream, all) = match view.trim().to_ascii_lowercase().as_str() {
        "stdout" => ("stdout", session.runtime.stdout()),
        "stderr" => ("stderr", session.runtime.stderr()),
        "note" | "notes" => ("note", session.runtime.notes()),
        _ => ("display", session.runtime.display()),
    };
    let total = all.len();
    let mut start = if cursor < 0 { 0 } else { (cursor as usize).min(total) };
    while start < total && !all.is_char_boundary(start) {
        start += 1;
    }
    let delta = all.get(start..).unwrap_or("").to_string();
    json!({ "delta": delta, "cursor": total, "total": total, "stream": stream })
}

/// 初始化统一模式（时间旅行）会话。返回 0 成功；-2 表示尚未编译成功。
pub fn step_begin(session: &mut Session) -> i32 {
    if !session.compile.compiled {
        return -2;
    }
    let mut engine = UnifiedEngine::new();
    engine.reset();
    let mut vm = session.vm.take().unwrap_or_default();
    reset_runtime_for_step(session);
    setup_vm(&mut vm, session);
    inject_preset_files(&mut vm, session);
    session.runtime.running = true;
    engine.checkpoints.save(0, &mut vm, &mut session.as_vm_context());
    session.vm = Some(vm);
    session.unified = Some(engine);
    0
}

/// 单步推进一次，返回该步的 `AutoStepResult` 形状 JSON（见 schema 文档 §6.1）。
pub fn step_next(session: &mut Session) -> Result<Value, String> {
    let Some(mut engine) = session.unified.take() else {
        return Err("统一模式会话未初始化：请先调用 cide_step_begin".to_string());
    };
    let mut vm = session.vm.take().unwrap_or_default();
    let outcome = engine.run_batch(&mut vm, session, 1);
    session.vm = Some(vm);
    session.unified = Some(engine);
    match outcome {
        Ok(result) => serde_json::to_value(&result).map_err(|e| format!("JSON 序列化失败：{}", e)),
        Err(e) => Err(e),
    }
}

/// 普通 VM 单步（非统一模式；cide_cli step 子命令同款语义）。
///
/// 首次调用（`running == false`）先初始化步进环境并推进到第一个 step 事件；
/// 之后每次调用推进一条指令。R2：自 flutter_bridge 收口而来——原实现挂在全局
/// 单例上，语义本体在此（语言中立层），出口（CLI/capi）只做薄包装。
pub fn vm_step(session: &mut Session) -> crate::session::StepResult {
    use crate::session::StepStatus;
    use crate::vm::core::StepResult as VmStep;

    if !session.compile.compiled {
        return crate::session::StepResult {
            status: StepStatus::Trap,
            current_line: 0,
            output: String::new(),
            waiting_input: false,
        };
    }

    let mut vm = session.vm.take().unwrap_or_default();
    let result = if !session.runtime.running {
        reset_runtime_for_step(session);
        setup_vm(&mut vm, session);
        inject_preset_files(&mut vm, session);
        vm.pause();
        session.runtime.waiting_input = false;
        loop {
            match vm.step(&mut session.as_vm_context()) {
                VmStep::Ok => {
                    // 首次运行：遇到第一个 StepEvent 后暂停，避免无断点时持续执行到 max_steps
                    if vm.was_step_event_hit() {
                        session.runtime.current_line = vm.get_current_line();
                        break crate::session::StepResult {
                            status: StepStatus::Paused,
                            current_line: session.runtime.current_line,
                            output: session.runtime.output(),
                            waiting_input: false,
                        };
                    }
                }
                VmStep::Paused => {
                    session.runtime.current_line = vm.get_current_line();
                    break crate::session::StepResult {
                        status: StepStatus::Paused,
                        current_line: session.runtime.current_line,
                        output: session.runtime.output(),
                        waiting_input: false,
                    };
                }
                VmStep::WaitingInput => {
                    session.runtime.current_line = vm.get_current_line();
                    break crate::session::StepResult {
                        status: StepStatus::WaitingInput,
                        current_line: session.runtime.current_line,
                        output: session.runtime.output(),
                        waiting_input: true,
                    };
                }
                VmStep::Finished => {
                    session.runtime.running = false;
                    session.runtime.current_line = vm.get_current_line();
                    break crate::session::StepResult {
                        status: StepStatus::Finished,
                        current_line: session.runtime.current_line,
                        output: session.runtime.output(),
                        waiting_input: false,
                    };
                }
                VmStep::Trap => {
                    session.runtime.error = vm.get_error().to_string();
                    session.runtime.running = false;
                    session.runtime.current_line = vm.get_current_line();
                    break crate::session::StepResult {
                        status: StepStatus::Trap,
                        current_line: session.runtime.current_line,
                        output: session.runtime.output(),
                        waiting_input: false,
                    };
                }
            }
        }
    } else {
        match vm.step(&mut session.as_vm_context()) {
            VmStep::Ok | VmStep::Paused => {
                session.runtime.current_line = vm.get_current_line();
                step_out(session, StepStatus::Paused, false)
            }
            VmStep::WaitingInput => {
                session.runtime.current_line = vm.get_current_line();
                step_out(session, StepStatus::WaitingInput, true)
            }
            VmStep::Finished => {
                session.runtime.running = false;
                session.runtime.current_line = vm.get_current_line();
                step_out(session, StepStatus::Finished, false)
            }
            VmStep::Trap => {
                session.runtime.error = vm.get_error().to_string();
                session.runtime.running = false;
                session.runtime.current_line = vm.get_current_line();
                step_out(session, StepStatus::Trap, false)
            }
        }
    };
    session.vm = Some(vm);
    result
}

fn step_out(session: &Session, status: crate::session::StepStatus, waiting: bool) -> crate::session::StepResult {
    crate::session::StepResult {
        status,
        current_line: session.runtime.current_line,
        output: session.runtime.output(),
        waiting_input: waiting,
    }
}

/// 当前栈帧局部变量快照（cide_cli step 的 `p` 命令；教学变量面板同源）。
pub fn variables(session: &Session) -> Vec<cide_runtime::VariableSnapshotData> {
    match session.vm.as_ref() {
        Some(vm) => vm.get_variable_snapshot(),
        None => Vec::new(),
    }
}

/// 取 `[start, end)` 步区间的 StepPayload（裁剪到当前 frameCache 窗口内）。
pub fn payloads(session: &Session, start: i32, end: i32) -> Result<Value, String> {
    let Some(engine) = session.unified.as_ref() else {
        return Err("统一模式会话未初始化：请先调用 cide_step_begin".to_string());
    };
    Ok(json!({
        "payloads": engine.get_payloads(start, end),
        "cache_start_step": engine.frame_cache_start_step,
        "max_collected_step": engine.max_collected_step(),
    }))
}

/// Seek 到指定步（窗口内直接命中；越窗走检查点恢复 + 正向重放）。
pub fn seek(session: &mut Session, target: i32) -> Result<Value, String> {
    let Some(mut engine) = session.unified.take() else {
        return Err("统一模式会话未初始化：请先调用 cide_step_begin".to_string());
    };
    let mut vm = session.vm.take().unwrap_or_default();
    let result = engine.seek_to(target, &mut vm, session);
    session.vm = Some(vm);
    session.unified = Some(engine);
    serde_json::to_value(&result).map_err(|e| format!("JSON 序列化失败：{}", e))
}

/// 写入断点行号集合（会先清空旧断点）。非正行号忽略。
pub fn set_breakpoints(session: &mut Session, lines: &[i32]) -> i32 {
    if let Some(vm) = session.vm.as_mut() {
        vm.clear_breakpoints();
        for &line in lines {
            if line > 0 {
                vm.add_breakpoint(line);
            }
        }
    }
    0
}

/// 内存视图（堆决议 §3 的三色语义：已分配 / 隔离中 / 可复用）。
///
/// ⚠️ 过渡形态：capi 第二批将把区域查询定型为 `kind: global|stack|heap` +
/// `status` + `alloc_line` 的地道 schema；serve 先按现有字段暴露，第二批落地后对齐。
pub fn memory_regions(session: &Session) -> Value {
    json!({
        "regions": session.memory.regions,
        "free_list": session.memory.free_list,
        "quarantine": {
            "bytes": session.memory.quarantine_bytes,
            "budget": session.memory.quarantine_budget,
            "blocks": session.memory.quarantine.len(),
        },
        "heap_base": session.memory.heap_base,
        "heap_offset": session.memory.heap_offset,
        "alloc_counter": session.memory.alloc_counter,
    })
}

/// 会话级配置读取（判分/回放场景需要确认两边配置一致）。
///
/// 2026-09-11 补齐：此前只能 `config.set` 而读不回 `max_steps` / `call_depth_limit`，
/// 消费方无法确认保险丝真的生效（实测暴露过"设置成功但程序跑到默认上限"的静默失败）。
/// VM 尚未创建时两项返回 `null`（表示"尚未落到 VM 上"，与会话字段类配置区分）。
pub fn config(session: &Session) -> Value {
    let (max_steps, call_depth_limit) = match session.vm.as_ref() {
        Some(vm) => (json!(vm.max_steps()), json!(vm.call_depth_limit())),
        None => (Value::Null, Value::Null),
    };
    json!({
        "deterministic": session.runtime.deterministic,
        "quarantine_budget": session.memory.quarantine_budget,
        "input_mode_batch": matches!(session.runtime.input_mode, crate::session::InputMode::Batch),
        "compiled": session.compile.compiled,
        "max_steps": max_steps,
        "call_depth_limit": call_depth_limit,
    })
}

/// `session.reset` 语义单源：清空编译/运行状态，**保留会话级配置**
/// （隔离预算、判分确定性、argv），与引擎 `reset_runtime` 的"配置保留、运行
/// 清空"语义一致。serve 与后续出口共用本入口（R3 自 cide_cli 收口）。
pub fn reset_session_preserving_config(session: &mut Session) {
    let quarantine_budget = session.memory.quarantine_budget;
    let deterministic = session.runtime.deterministic;
    let argc = session.runtime.argc;
    let argv = std::mem::take(&mut session.runtime.argv);
    *session = Session::default();
    session.memory.quarantine_budget = quarantine_budget;
    session.runtime.deterministic = deterministic;
    session.runtime.argc = argc;
    session.runtime.argv = argv;
}

/// E2：引擎能力清单（机器可读真实能力；"版本宏当能力探测"的三层配套之一）。
///
/// 口径（C23 锚定决议）：`__STDC_VERSION__=202311L` 是**名义锚点**，真实能力
/// 以本清单为准；内存模型常量从 `cide_runtime` 单源引用，禁止在此复刻数值。
pub fn capabilities() -> Value {
    use cide_runtime::{GLOBAL_REGION_LIMIT, GLOBAL_START, HEAP_START, MEM_SIZE, NULL_TRAP_SIZE};
    json!({
        "engine": "cide",
        "abi_version": crate::capi::CIDE_ABI_VERSION,
        "languages": {
            "c": {
                "anchor": "ISO C23 (ISO/IEC 9899:2024) 教学子集",
                "stdc_version_macro_nominal": "202311L",
                "spec": "docs/current/C_SUBSET_SPEC.md",
                "predefined_macros": {
                    "__STDC_VERSION__": "202311L",
                    "__CIDE_SUBSET__": "1",
                },
                "preprocessor": {
                    "object_macros": true,
                    "function_macros": true,
                    "stringize": true,
                    "token_paste": true,
                    "paste_result_must_be_single_token": true,
                    "conditionals": ["#if", "#ifdef", "#ifndef", "#elif", "#else", "#endif"],
                    "defined": true,
                    "has_include": true,
                    "include_once": true,
                    "include_cycle_detection": true,
                    "expand_depth_fuse": 64,
                    "teaching_layer": [
                        "macro_shadowing_warning",
                        "macro_arg_side_effect_warning",
                        "expansion_trace",
                        "branch_reason",
                    ],
                    "dropped": [
                        "自引用宏 trick（展开栈查重直接停止）",
                        "## 动态拼标识符的元编程（结果必须为单个合法 token）",
                        "X-macro 高级用法",
                        "宏拼接 include 路径",
                    ],
                },
            },
            "cpp": {
                "anchor": "C++ 教学子集（Phase 31+，Stage 0~6）",
                "spec": "docs/current/CPP_SUBSET_SPEC.md",
            },
        },
        "memory_model": {
            "mem_size": MEM_SIZE,
            "null_trap_size": NULL_TRAP_SIZE,
            "global_start": GLOBAL_START,
            "global_region_limit": GLOBAL_REGION_LIMIT,
            "heap_start_default": HEAP_START,
            "heap_start": "动态：max(HEAP_START, align4(global_data_end))",
        },
    })
}
