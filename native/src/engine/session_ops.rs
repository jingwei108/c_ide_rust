//! Session 操作公共函数
//!
//! 为 `flutter_bridge.rs` 和 `capi/mod.rs` 提供统一的运行/单步核心逻辑，
//! 消除两端的重复代码。

use crate::engine::compile_pipeline::setup_vm;
use crate::session::Session;
use crate::vm::vm::CideVM;

/// 初始化程序运行环境（非 resume 场景）
pub fn reset_runtime(session: &mut Session) {
    session.runtime.output_lines.clear();
    session.runtime.error.clear();
    session.runtime.trace.clear();
    session.memory.regions.clear();
    session.memory.free_list.clear();
    session.memory.heap_offset = 0x5000;
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

    if vm.has_error() {
        session.runtime.error = vm.get_error().to_string();
        session.runtime.running = false;
        session.vm = Some(vm);
        Err(session.runtime.error.clone())
    } else if session.runtime.waiting_input {
        session.vm = Some(vm);
        Ok((ret, true))
    } else {
        session.runtime.output_lines.push(format!("程序运行完成，返回值：{}\n", ret));
        session.runtime.running = false;
        session.vm = Some(vm);
        Ok((ret, false))
    }
}
