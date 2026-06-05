//! Session 操作公共函数
//!
//! 为 `flutter_bridge.rs` 和 `capi/mod.rs` 提供统一的运行/单步核心逻辑，
//! 消除两端的重复代码。

use crate::engine::compile_pipeline::setup_vm;
use crate::session::Session;
use crate::vm::vm::CideVM;

/// 生成内存泄漏报告并追加到输出。
///
/// 遍历所有未释放的堆区域，按分配行号排序，
/// 输出形如 "第 12 行的 malloc 未被 free" 的提示。
pub fn append_leak_report(session: &mut Session) {
    let leaks: Vec<_> = session
        .memory
        .regions
        .iter()
        .filter(|r| r.is_heap && !r.is_freed && r.alloc_by != "vfs")
        .collect();

    if leaks.is_empty() {
        return;
    }

    let mut lines: Vec<String> = Vec::new();
    lines.push("\n===== 内存泄漏检测报告 =====".to_string());

    let mut sorted = leaks.clone();
    sorted.sort_by_key(|r| r.alloc_line);

    let total_leaked: i32 = sorted.iter().map(|r| r.size).sum();
    lines.push(format!("发现 {} 处未释放的堆内存，共 {} 字节：", sorted.len(), total_leaked));

    for r in &sorted {
        let by = if r.alloc_by.is_empty() { "malloc" } else { &r.alloc_by };
        if r.alloc_line > 0 {
            lines.push(format!("  • 第 {} 行的 {} 分配了 {} 字节 (addr=0x{:04X})，未被 free", r.alloc_line, by, r.size, r.addr));
        } else {
            lines.push(format!("  • {} 分配了 {} 字节 (addr=0x{:04X})，未被 free", by, r.size, r.addr));
        }
    }

    lines.push("💡 提示：在 C 语言中，malloc 分配的内存需要对应 free 释放，否则会造成内存泄漏。".to_string());
    lines.push("==============================".to_string());

    session.runtime.output_lines.extend(lines);
}

/// 初始化程序运行环境（非 resume 场景）
pub fn reset_runtime(session: &mut Session) {
    session.runtime.output_lines.clear();
    session.runtime.error.clear();
    session.runtime.trace.clear();
    session.memory.regions.clear();
    session.memory.free_list.clear();
    session.memory.heap_offset = crate::vm::vm::HEAP_START;
    session.memory.alloc_counter = 0;
    session.vfs = crate::vm::vfs::VirtualFileSystem::new();
    session.runtime.running = true;
}

/// 初始化单步执行环境
pub fn reset_runtime_for_step(session: &mut Session) {
    reset_runtime(session);
    session.runtime.step_count = 0;
    session.runtime.step_mode = true;
}

/// 为 VM 注入预设测试文件
pub fn inject_preset_files(vm: &mut CideVM, session: &mut Session) {
    let mut vfs = std::mem::take(&mut session.vfs);
    vfs.inject_preset_file("test.txt", b"hello\nworld\n", vm, &mut session.memory);
    vfs.inject_preset_file("numbers.txt", b"1 2 3 4 5\n", vm, &mut session.memory);
    session.vfs = vfs;
}

/// 执行全速运行。
///
/// 调用前需确保 session 已编译且 VM 可用。
/// 返回 `(运行返回值, 是否等待输入)`；若运行出错返回 `Err`。
pub fn execute_run(session: &mut Session) -> Result<(i32, bool), String> {
    let is_resume = session.runtime.waiting_input;

    if !is_resume {
        reset_runtime(session);
    }
    session.runtime.step_mode = false;
    session.runtime.waiting_input = false;

    let mut vm = session.vm.take().unwrap_or_default();
    if !is_resume {
        setup_vm(&mut vm, session);
        inject_preset_files(&mut vm, session);
    } else {
        vm.resume();
    }

    let ret = vm.run(session);

    // 收集 JIT 统计（在 vm 移入 session 之前）
    let jit_stats = vm.jit_stats().clone();

    if vm.has_error() {
        session.runtime.error = vm.get_error().to_string();
        session.runtime.running = false;
        session.vm = Some(vm);
        Err(session.runtime.error.clone())
    } else if session.runtime.waiting_input {
        session.vm = Some(vm);
        Ok((ret, true))
    } else {
        if jit_stats.traces_compiled > 0 {
            session.runtime.output_lines.push(format!(
                "[JIT] 已编译 {} 条 trace，加速执行 {} 步",
                jit_stats.traces_compiled,
                jit_stats.steps_accelerated
            ));
        }
        session.runtime.output_lines.push(format!("程序运行完成，返回值：{}\n", ret));
        append_leak_report(session);
        session.runtime.running = false;
        session.vm = Some(vm);
        Ok((ret, false))
    }
}
