use super::host_funcs::execute_host_func;
use super::instruction::{Instruction, SourceLoc};
use super::jit_templates::{execute_trace_bulk, CompiledTrace};
use super::jit_trace::{JitStats, TraceRecorder, JIT_THRESHOLD};
use super::opcode::OpCode;
use crate::session::{Session, VisEvent};
use crate::shared::type_utils::base_kind;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub const MEM_SIZE: u32 = 1024 * 1024;
pub const NULL_TRAP_SIZE: u32 = 0x1000;
pub const GLOBAL_START: u32 = 0x1000;
pub const HEAP_START: u32 = 0x5000;
pub const STACK_START: u32 = MEM_SIZE;
pub const SNAPSHOT_INTERVAL: i32 = 100_000;
pub const MAX_STACK_DEPTH: usize = 10_000;

/// Epsilon for approximate float comparison (f32).
const EPS_F32: f32 = 1e-6;
/// Epsilon for approximate double comparison (f64).
/// Using 1e-6 (same as f32) because Cide's float literals default to f32
/// and are promoted to double in contexts, leading to larger rounding deltas.
const EPS_F64: f64 = 1e-6;

pub use crate::shared::{func_meta::FuncMeta, symbol::Symbol as VMSymbol};

#[derive(Debug, Clone)]
pub struct FreedRegionInfo {
    pub addr: u32,
    pub size: u32,
    pub alloc_line: i32,
    pub freed_line: i32,
    /// Step index when the memory was allocated (for unified-mode timeline).
    pub alloc_step: i32,
    /// Step index when the memory was freed (for unified-mode timeline).
    pub freed_step: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallFrame {
    pub return_ip: usize,
    pub locals_base: u32,
    pub local_count: i32,
    pub func_name: String,
    pub original_stack_top: u32,
    pub caller_line: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StepResult {
    Ok,
    Paused,
    Finished,
    Trap,
    WaitingInput,
}

#[derive(Debug, Clone)]
pub struct ArrayConstructionGuard {
    /// `new[]` 分配返回给用户的指针之前 4 字节的 base 地址（存储 count）。
    pub base_addr: u32,
    /// 设置 guard 时的 call_stack 深度，用于确认 guard 设置者的栈帧仍然存在。
    pub frame_depth: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    Read,
    Write,
}

#[derive(Debug, Clone)]
pub struct VariableAccess {
    pub name: String,
    pub access_type: AccessType,
}

pub struct CideVM {
    code: Vec<Instruction>,
    ip: usize,
    memory: Vec<u8>,
    stack: Vec<u64>,
    mem_stack_top: u32,
    global_count: usize,
    call_stack: Vec<CallFrame>,
    func_table: Vec<FuncMeta>,
    func_names: Vec<String>,
    symbols: Vec<VMSymbol>,
    vis_event_lines: Vec<(i32, i32, String)>,
    vis_event_queue: Vec<VisEvent>,
    breakpoints: HashSet<i32>,
    paused: bool,
    cancelled: bool,
    step_event_hit: bool,
    step_count: i32,
    max_steps: i32,
    current_line: i32,
    error: String,
    last_snapshot_step: i32,
    snapshot_vars: HashMap<String, u64>,
    finished: bool,
    exit_code: i32,
    qsort_depth: i32,
    f64_constants: Vec<f64>,
    i64_constants: Vec<i64>,
    last_accessed_vars: Vec<VariableAccess>,
    local_sym_map: HashMap<i32, String>,
    global_sym_map: HashMap<i32, String>,
    pub(crate) freed_logs: Vec<FreedRegionInfo>,
    /// 当前未完成的 `new T[n]` 构造守卫。若构造过程中 trap，用于回滚释放内存。
    pub pending_array_construction: Option<ArrayConstructionGuard>,
    /// `main(int argc, char *argv[])` 的 argc 值。
    argc: i32,
    /// `main(int argc, char *argv[])` 的 argv 数组在 VM 内存中的起始地址。
    argv_addr: u32,
    // --- 增量快照脏页追踪 ---
    dirty_pages: [u64; 4], // 256 页 bitmap（4 × 64bit）
    // --- JIT ---
    ip_hits: HashMap<usize, u64>,
    trace_recorder: TraceRecorder,
    jit_traces: HashMap<usize, Arc<CompiledTrace>>,
    jit_stats: JitStats,
}

impl Default for CideVM {
    fn default() -> Self {
        Self::new()
    }
}

impl CideVM {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            ip: 0,
            memory: vec![0; MEM_SIZE as usize],
            stack: Vec::new(),
            mem_stack_top: STACK_START,
            global_count: 0,
            call_stack: Vec::new(),
            func_table: Vec::new(),
            func_names: Vec::new(),
            symbols: Vec::new(),
            vis_event_lines: Vec::new(),
            vis_event_queue: Vec::new(),
            breakpoints: HashSet::new(),
            paused: false,
            cancelled: false,
            step_event_hit: false,
            step_count: 0,
            max_steps: 10_000_000,
            current_line: 0,
            error: String::new(),
            last_snapshot_step: 0,
            snapshot_vars: HashMap::new(),
            finished: false,
            exit_code: 0,
            qsort_depth: 0,
            f64_constants: Vec::new(),
            i64_constants: Vec::new(),
            last_accessed_vars: Vec::new(),
            local_sym_map: HashMap::new(),
            global_sym_map: HashMap::new(),
            freed_logs: Vec::new(),
            pending_array_construction: None,
            argc: 0,
            argv_addr: 0,
            dirty_pages: [0; 4],
            ip_hits: HashMap::new(),
            trace_recorder: TraceRecorder::new(),
            jit_traces: HashMap::new(),
            jit_stats: JitStats::default(),
        }
    }

    pub fn reset(&mut self) {
        self.code.clear();
        self.ip = 0;
        self.stack.clear();
        self.call_stack.clear();
        self.func_table.clear();
        self.func_names.clear();
        self.symbols.clear();
        self.vis_event_lines.clear();
        self.vis_event_queue.clear();
        self.breakpoints.clear();
        self.paused = false;
        self.cancelled = false;
        self.step_event_hit = false;
        self.step_count = 0;
        self.max_steps = 10_000_000;
        self.current_line = 0;
        self.error.clear();
        self.global_count = 0;
        self.last_snapshot_step = 0;
        self.snapshot_vars.clear();
        self.finished = false;
        self.exit_code = 0;
        self.i64_constants.clear();
        self.last_accessed_vars.clear();
        self.qsort_depth = 0;
        self.local_sym_map.clear();
        self.global_sym_map.clear();
        self.freed_logs.clear();
        self.argc = 0;
        self.argv_addr = 0;
        self.dirty_pages = [0; 4];
        self.ip_hits.clear();
        self.trace_recorder.reset();
        self.jit_traces.clear();
        self.jit_stats = JitStats::default();
        self.memory.fill(0);
        self.mem_stack_top = STACK_START;
    }

    pub fn load_program(&mut self, code: Vec<Instruction>) {
        self.code = code;
        self.ip = 0;
    }

    pub fn set_argc(&mut self, argc: i32) {
        self.argc = argc;
    }

    pub fn set_argv_addr(&mut self, addr: u32) {
        self.argv_addr = addr;
    }

    pub fn set_globals_32(&mut self, globals: &[(u32, i32)]) {
        for &(offset, v) in globals {
            let addr = GLOBAL_START + offset;
            self.store_i32(addr, v, &SourceLoc::default());
        }
    }

    pub fn set_globals_64(&mut self, globals: &[(u32, u64)]) {
        for &(offset, v) in globals {
            let addr = GLOBAL_START + offset;
            self.store_i64(addr, v, &SourceLoc::default());
        }
    }

    /// 在全局数据区之后为 `main(int argc, char *argv[])` 分配 argv 内存。
    /// 指针数组紧随其后存放字符串指针，字符串数据跟在其后。
    pub fn setup_argv(&mut self, argc: i32, argv: &[String]) {
        if argc <= 0 || argv.is_empty() {
            self.argc = 0;
            self.argv_addr = 0;
            return;
        }
        let array_addr = GLOBAL_START + (self.global_count as u32) * 4;
        let mut string_addr = array_addr + (argc as u32) * 4;
        let count = (argc as usize).min(argv.len());
        for (i, s) in argv.iter().enumerate().take(count) {
            let next = string_addr + s.len() as u32 + 1;
            if next > HEAP_START {
                self.trap("setup_argv: 命令行参数过长，超出全局数据区", &SourceLoc::default());
                return;
            }
            self.write_cstring(string_addr, s);
            self.store_i32(array_addr + (i as u32) * 4, string_addr as i32, &SourceLoc::default());
            string_addr = next;
        }
        self.argc = argc;
        self.argv_addr = array_addr;
    }

    pub fn register_function(&mut self, idx: u32, meta: FuncMeta) {
        let idx = idx as usize;
        if idx >= self.func_table.len() {
            self.func_table.resize(idx + 1, FuncMeta::default());
        }
        self.func_table[idx] = meta;
    }

    pub fn register_function_name(&mut self, idx: u32, name: String) {
        let idx = idx as usize;
        if idx >= self.func_names.len() {
            self.func_names.resize(idx + 1, String::new());
        }
        self.func_names[idx] = name;
    }

    pub fn set_symbols(&mut self, symbols: Vec<VMSymbol>) {
        self.global_sym_map.clear();
        for sym in &symbols {
            if !sym.is_local {
                self.global_sym_map.insert(sym.addr as i32, sym.name.clone());
            }
        }
        self.symbols = symbols;
    }

    pub fn set_f64_constants(&mut self, constants: Vec<f64>) {
        self.f64_constants = constants;
    }

    pub fn set_i64_constants(&mut self, constants: Vec<i64>) {
        self.i64_constants = constants;
    }

    pub fn set_vis_event_lines(&mut self, lines: Vec<(i32, i32, String)>) {
        self.vis_event_lines = lines;
    }

    pub fn take_vis_events(&mut self) -> Vec<VisEvent> {
        std::mem::take(&mut self.vis_event_queue)
    }

    pub fn add_breakpoint(&mut self, line: i32) {
        self.breakpoints.insert(line);
    }

    pub fn remove_breakpoint(&mut self, line: i32) {
        self.breakpoints.remove(&line);
    }

    pub fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
        self.step_event_hit = false;
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    /// 创建全量快照。
    ///
    /// 调用者应确保 `session.compile` 中的编译产物与当前 VM 加载的程序一致。
    pub fn snapshot(&self, session: &Session) -> super::snapshot::VMSnapshot {
        super::snapshot::VMSnapshot {
            memory: super::snapshot::MemoryImage::Full(self.memory.clone()),
            stack: self.stack.clone(),
            call_stack: self.call_stack.clone(),
            ip: self.ip,
            mem_stack_top: self.mem_stack_top,
            step_count: self.step_count,
            current_line: self.current_line,
            finished: self.finished,
            exit_code: self.exit_code,
            error: self.error.clone(),
            paused: self.paused,
            cancelled: self.cancelled,
            step_event_hit: self.step_event_hit,
            last_snapshot_step: self.last_snapshot_step,
            snapshot_vars: self.snapshot_vars.clone(),
            qsort_depth: self.qsort_depth,
            vis_event_queue: self.vis_event_queue.clone(),
            breakpoints: self.breakpoints.clone(),
            global_count: self.global_count,
            freed_logs: self.freed_logs.clone(),
            runtime: super::snapshot::RuntimeSnapshot::from(&session.runtime),
            memory_state: super::snapshot::MemorySnapshot::from(&session.memory),
        }
    }

    /// 将当前 VM 状态写入已有的 `VMSnapshot`，复用其内存 buffer。
    ///
    /// 当 `target.memory` 为 `MemoryImage::Full` 且长度匹配时，仅执行 `copy_from_slice`，
    /// 避免每步分配新的 1MB Vec。若类型或长度不匹配，则回退到 `clone()`。
    pub fn snapshot_into(&self, session: &Session, target: &mut super::snapshot::VMSnapshot) {
        match &mut target.memory {
            super::snapshot::MemoryImage::Full(buf) if buf.len() == self.memory.len() => {
                buf.copy_from_slice(&self.memory);
            }
            _ => {
                target.memory = super::snapshot::MemoryImage::Full(self.memory.clone());
            }
        }
        target.stack = self.stack.clone();
        target.call_stack = self.call_stack.clone();
        target.ip = self.ip;
        target.mem_stack_top = self.mem_stack_top;
        target.step_count = self.step_count;
        target.current_line = self.current_line;
        target.finished = self.finished;
        target.exit_code = self.exit_code;
        target.error = self.error.clone();
        target.paused = self.paused;
        target.cancelled = self.cancelled;
        target.step_event_hit = self.step_event_hit;
        target.last_snapshot_step = self.last_snapshot_step;
        target.snapshot_vars = self.snapshot_vars.clone();
        target.qsort_depth = self.qsort_depth;
        target.vis_event_queue = self.vis_event_queue.clone();
        target.breakpoints = self.breakpoints.clone();
        target.global_count = self.global_count;
        target.freed_logs = self.freed_logs.clone();
        target.runtime = super::snapshot::RuntimeSnapshot::from(&session.runtime);
        target.memory_state = super::snapshot::MemorySnapshot::from(&session.memory);
    }

    /// 标记脏页（ addr 为起始地址，len 为字节数 ）。
    fn mark_dirty_page(&mut self, addr: u32, len: u32) {
        if len == 0 {
            return;
        }
        let start_page = (addr as usize) / super::snapshot::PAGE_SIZE;
        let end_page = ((addr as usize) + (len as usize) - 1) / super::snapshot::PAGE_SIZE;
        for p in start_page..=end_page {
            if p < super::snapshot::PAGE_COUNT {
                let word = p / 64;
                let bit = p % 64;
                self.dirty_pages[word] |= 1u64 << bit;
            }
        }
    }

    /// 清空脏页记录。
    pub fn clear_dirty_pages(&mut self) {
        self.dirty_pages = [0; 4];
    }

    /// 基于脏页生成增量快照。
    pub fn snapshot_incremental(&self, session: &Session, base_step: i32) -> super::snapshot::VMSnapshot {
        let mut pages = Vec::new();
        for word in 0..4 {
            let mut bitmap = self.dirty_pages[word];
            while bitmap != 0 {
                let bit = bitmap.trailing_zeros() as usize;
                bitmap &= !(1u64 << bit);
                let page_idx = word * 64 + bit;
                if page_idx >= super::snapshot::PAGE_COUNT {
                    continue;
                }
                let offset = page_idx * super::snapshot::PAGE_SIZE;
                let page_data = self.memory[offset..offset + super::snapshot::PAGE_SIZE].to_vec();
                pages.push((page_idx as u16, page_data));
            }
        }
        super::snapshot::VMSnapshot {
            memory: super::snapshot::MemoryImage::Delta { base_step, pages },
            stack: self.stack.clone(),
            call_stack: self.call_stack.clone(),
            ip: self.ip,
            mem_stack_top: self.mem_stack_top,
            step_count: self.step_count,
            current_line: self.current_line,
            finished: self.finished,
            exit_code: self.exit_code,
            error: self.error.clone(),
            paused: self.paused,
            cancelled: self.cancelled,
            step_event_hit: self.step_event_hit,
            last_snapshot_step: self.last_snapshot_step,
            snapshot_vars: self.snapshot_vars.clone(),
            qsort_depth: self.qsort_depth,
            vis_event_queue: self.vis_event_queue.clone(),
            breakpoints: self.breakpoints.clone(),
            global_count: self.global_count,
            freed_logs: self.freed_logs.clone(),
            runtime: super::snapshot::RuntimeSnapshot::from(&session.runtime),
            memory_state: super::snapshot::MemorySnapshot::from(&session.memory),
        }
    }

    /// 从快照恢复 VM 和 Session 运行时状态。
    ///
    /// 恢复前必须先调用 `setup_vm()` 加载编译产物，否则 `code`、`func_table` 等为空，
    /// 恢复后的 VM 将无法继续执行。
    pub fn restore(&mut self, snap: &super::snapshot::VMSnapshot, session: &mut Session) {
        // VM 内存（1MB）
        snap.memory.apply_to(&mut self.memory);

        // VM 栈与调用帧
        self.stack = snap.stack.clone();
        self.call_stack = snap.call_stack.clone();

        // VM 执行指针与计数器
        self.ip = snap.ip;
        self.mem_stack_top = snap.mem_stack_top;
        self.step_count = snap.step_count;
        self.current_line = snap.current_line;

        // VM 状态标志
        self.finished = snap.finished;
        self.exit_code = snap.exit_code;
        self.error = snap.error.clone();
        self.paused = snap.paused;
        self.cancelled = snap.cancelled;
        self.step_event_hit = snap.step_event_hit;
        self.last_snapshot_step = snap.last_snapshot_step;
        self.snapshot_vars = snap.snapshot_vars.clone();
        self.qsort_depth = snap.qsort_depth;

        // 可视化与调试
        self.vis_event_queue = snap.vis_event_queue.clone();
        self.breakpoints = snap.breakpoints.clone();
        self.global_count = snap.global_count;

        // Session 运行时状态
        session.runtime.output_lines = snap.runtime.output_lines.clone();
        session.runtime.trace = snap.runtime.trace.clone();
        session.runtime.current_line = snap.runtime.current_line;
        session.runtime.input_index = snap.runtime.input_index;
        session.runtime.input_char_offset = snap.runtime.input_char_offset;
        session.runtime.waiting_input = snap.runtime.waiting_input;
        session.runtime.rand_seed = snap.runtime.rand_seed;
        session.runtime.vis_event_cache = snap.runtime.vis_event_cache.clone();
        session.runtime.ungetc_char = snap.runtime.ungetc_char;

        // Session 内存管理状态
        session.memory.regions = snap.memory_state.regions.clone();
        session.memory.free_list = snap.memory_state.free_list.clone();
        session.memory.heap_offset = snap.memory_state.heap_offset;
        session.memory.alloc_counter = snap.memory_state.alloc_counter;

        self.freed_logs = snap.freed_logs.clone();
    }

    /// Call a user-defined function from a host function context.
    /// Used by host_qsort to invoke the user-supplied comparison function.
    /// Returns the return value on success, or None if the function trapped.
    pub fn call_user_function(
        &mut self,
        session: &mut Session,
        func_idx: u32,
        args: &[i32],
        max_steps: i32,
    ) -> Option<i32> {
        let idx = func_idx as usize;
        if idx >= self.func_table.len() || self.func_table[idx].ip == 0 {
            return None;
        }
        let meta = self.func_table[idx].clone();
        let frame_size = meta.local_count as u64;
        if frame_size > MEM_SIZE as u64 || frame_size > self.mem_stack_top as u64 {
            return None;
        }
        let frame_size_u32 = frame_size as u32;
        if self.mem_stack_top < NULL_TRAP_SIZE + frame_size_u32 {
            return None;
        }

        // Save state
        let saved_ip = self.ip;
        let saved_call_stack = self.call_stack.clone();
        let saved_mem_stack_top = self.mem_stack_top;
        let saved_stack = self.stack.clone();
        let saved_error = self.error.clone();
        let saved_finished = self.finished;
        let saved_step_event_hit = self.step_event_hit;
        let saved_current_line = self.current_line;
        let saved_vis_event_queue = std::mem::take(&mut self.vis_event_queue);
        let saved_breakpoints = std::mem::take(&mut self.breakpoints);
        let start_step = self.step_count;

        // Setup call frame
        let original_stack_top = self.mem_stack_top;
        self.mem_stack_top -= frame_size_u32;
        let locals_base = self.mem_stack_top;
        // Arguments: args[0] is first param, args[n-1] is last param.
        // VM Call convention: first param is at locals_base + 0
        // 当前 call_user_function 仅用于 qsort 回调（参数均为 4 字节指针）。
        // 若未来扩展为 8 字节参数（double/long long），需按 type_size 选择 store_i32/store_i64。
        assert!(
            meta.param_sizes.iter().all(|&sz| sz == 1),
            "call_user_function 暂不支持非 4 字节参数（param_sizes {:?}）",
            meta.param_sizes
        );
        for i in 0..meta.param_count {
            let arg = if (i as usize) < args.len() { args[i as usize] } else { 0 };
            let arg_addr = (locals_base as u64) + (i as u64) * 4;
            self.store_i32(arg_addr as u32, arg, &SourceLoc::default());
        }
        let arg_bytes = meta.param_count as u32 * 4;
        let zero_start = locals_base + arg_bytes;
        let zero_end = locals_base + meta.local_count as u32;
        if zero_end > zero_start {
            for addr in zero_start..zero_end {
                self.store_i8(addr, 0, &SourceLoc::default());
            }
        }
        let func_name = if idx < self.func_names.len() {
            self.func_names[idx].clone()
        } else {
            format!("func_{}", func_idx)
        };
        const HOST_CALLBACK_SENTINEL: usize = usize::MAX;
        self.call_stack.push(CallFrame {
            return_ip: HOST_CALLBACK_SENTINEL,
            locals_base,
            local_count: meta.local_count,
            func_name,
            original_stack_top,
            caller_line: self.current_line,
        });
        self.rebuild_local_sym_map();
        self.ip = meta.ip;

        // Execute until return or trap
        let mut result = None;
        loop {
            if self.step_count - start_step >= max_steps {
                break;
            }
            let step_result = self.step(session);
            match step_result {
                StepResult::Finished => {
                    result = self.stack.pop().map(|v| v as i32);
                    break;
                }
                StepResult::Trap => {
                    break; // result stays None
                }
                StepResult::Paused => {
                    self.paused = false;
                }
                StepResult::WaitingInput => {
                    break;
                }
                StepResult::Ok => {}
            }
        }

        // Restore state
        self.ip = saved_ip;
        self.call_stack = saved_call_stack;
        self.mem_stack_top = saved_mem_stack_top;
        self.stack = saved_stack;
        self.error = saved_error;
        self.finished = saved_finished;
        self.step_event_hit = saved_step_event_hit;
        self.rebuild_local_sym_map();
        self.current_line = saved_current_line;
        self.vis_event_queue = saved_vis_event_queue;
        self.breakpoints = saved_breakpoints;

        result
    }

    pub fn set_max_steps(&mut self, max: i32) {
        self.max_steps = max;
    }

    pub fn has_error(&self) -> bool {
        !self.error.is_empty()
    }

    pub fn get_error(&self) -> &str {
        &self.error
    }

    pub fn code_len(&self) -> usize {
        self.code.len()
    }

    pub fn code_ref(&self) -> &[Instruction] {
        &self.code
    }

    pub fn set_ip(&mut self, ip: usize) {
        self.ip = ip;
    }

    pub fn get_ip(&self) -> usize {
        self.ip
    }

    pub fn get_current_line(&self) -> i32 {
        self.current_line
    }

    pub fn get_executed_steps(&self) -> i32 {
        self.step_count
    }

    pub fn jit_stats(&self) -> &crate::vm::jit_trace::JitStats {
        &self.jit_stats
    }

    pub fn jit_traces_mut(
        &mut self,
    ) -> &mut std::collections::HashMap<usize, std::sync::Arc<crate::vm::jit_templates::CompiledTrace>> {
        &mut self.jit_traces
    }

    /// JIT trace 批量执行时调用：增加 step_count 并检查全局限制。
    /// 返回 `true` 表示可以继续执行，`false` 表示已 trap。
    pub(crate) fn bulk_step_check(&mut self, steps: i32) -> bool {
        self.step_count = self.step_count.saturating_add(steps);
        if self.step_count >= self.max_steps {
            let msg = self.format_infinite_loop_error();
            self.trap(&msg, &SourceLoc::default());
            return false;
        }
        if self.cancelled {
            self.trap("执行已取消。", &SourceLoc::default());
            return false;
        }
        true
    }

    pub fn set_finished(&mut self, code: i32) {
        self.finished = true;
        self.exit_code = code;
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }

    /// 将 C 风格字符串安全写入 VM 内存的指定地址（含 null 终止符）。
    /// 若目标地址超出边界则静默跳过。
    ///
    /// 注意：边界检查 `a + bytes.len() < len` 已隐含为末尾的 null 终止符预留了 1 字节空间，
    /// 因此当 `addr + bytes.len() == MEM_SIZE` 时会正确拒绝写入，避免越界。
    pub fn write_cstring(&mut self, addr: u32, s: &str) {
        let a = addr as usize;
        let bytes = s.as_bytes();
        let total = bytes.len() + 1;
        // 统一边界检查：NULL 区、上界、UAF。
        if !self.check_mem_access(addr, total as u32, &SourceLoc::default(), true) {
            return;
        }
        if let Some(log) = self.check_uaf(addr, total as u32) {
            let msg = self.format_uaf_message(log, true);
            self.trap(&msg, &SourceLoc::default());
            return;
        }
        self.memory[a..a + bytes.len()].copy_from_slice(bytes);
        self.memory[a + bytes.len()] = 0;
        self.mark_dirty_page(addr, total as u32);
    }

    pub(crate) fn call_stack_len(&self) -> usize {
        self.call_stack.len()
    }

    pub fn memory_ref(&self) -> &[u8] {
        &self.memory
    }

    pub fn memory_ref_mut(&mut self) -> &mut [u8] {
        &mut self.memory
    }

    /// 若存在未完成的 `new T[n]` 构造守卫，释放其底层内存块。
    /// 在 VM trap 或运行结束时调用，防止构造失败导致内存泄漏。
    pub fn rollback_pending_array_construction(&mut self, session: &mut Session) {
        if let Some(guard) = self.pending_array_construction.take() {
            // 仅当设置 guard 的栈帧仍然存在时才执行回滚（避免跨函数边界误释放）。
            if self.call_stack.len() >= guard.frame_depth {
                self.free_memory(session, guard.base_addr);
            }
        }
    }

    fn free_memory(&mut self, session: &mut Session, addr: u32) {
        if addr == 0 {
            return;
        }
        // 避免 double-free 记录重复（已释放则静默跳过）。
        if self.freed_logs.iter().any(|log| log.addr == addr) {
            return;
        }
        for r in &mut session.memory.regions {
            if r.addr == addr && !r.is_freed {
                r.is_freed = true;
                let aligned_size = ((r.size as u32) + 3) & !3;
                self.freed_logs.push(FreedRegionInfo {
                    addr: r.addr,
                    size: aligned_size,
                    alloc_line: r.alloc_line,
                    freed_line: self.get_current_line(),
                    alloc_step: 0,
                    freed_step: self.get_executed_steps(),
                });
                session.memory.free_list.push(crate::session::FreeBlock {
                    addr: r.addr,
                    size: aligned_size as i32,
                });
                session.memory.merge_free_list();
                break;
            }
        }
    }

    pub fn qsort_depth(&self) -> i32 {
        self.qsort_depth
    }

    pub fn set_qsort_depth(&mut self, depth: i32) {
        self.qsort_depth = depth;
    }

    pub fn get_memory_size(&self) -> u32 {
        MEM_SIZE
    }

    /// 获取当前已释放的内存区域日志（用于 UAF/Double-Free 检测诊断）。
    pub fn get_freed_logs(&self) -> &[FreedRegionInfo] {
        &self.freed_logs
    }

    /// 根据函数名查找其在 VM 函数表中的索引。
    pub fn get_func_index(&self, name: &str) -> Option<u32> {
        self.func_names.iter().position(|n| n == name).map(|i| i as u32)
    }

    /// 安全写入字节切片到 VM 内存（带 NULL Trap 和边界检查）
    pub fn write_memory(&mut self, addr: u32, data: &[u8]) -> bool {
        let end = addr as usize + data.len();
        if addr < NULL_TRAP_SIZE || end > self.memory.len() {
            return false;
        }
        if let Some(log) = self.check_uaf(addr, data.len() as u32) {
            let msg = self.format_uaf_message(log, true);
            self.trap(&msg, &SourceLoc::default());
            return false;
        }
        self.memory[addr as usize..end].copy_from_slice(data);
        self.mark_dirty_page(addr, data.len() as u32);
        true
    }

    /// 安全读取字节切片从 VM 内存到外部缓冲区
    pub fn read_memory_to(&self, addr: u32, buf: &mut [u8]) -> bool {
        let end = addr as usize + buf.len();
        if addr < NULL_TRAP_SIZE || end > self.memory.len() {
            return false;
        }
        buf.copy_from_slice(&self.memory[addr as usize..end]);
        true
    }

    /// 安全地在 VM 内存内部复制（src → dst），带完整边界检查
    pub fn copy_memory(&mut self, dst: u32, src: u32, len: usize) -> bool {
        let dst_end = dst as usize + len;
        let src_end = src as usize + len;
        if dst < NULL_TRAP_SIZE || src < NULL_TRAP_SIZE || dst_end > self.memory.len() || src_end > self.memory.len() {
            return false;
        }
        let len_u32 = len as u32;
        if let Some(log) = self.check_uaf(dst, len_u32) {
            let msg = self.format_uaf_message(log, true);
            self.trap(&msg, &SourceLoc::default());
            return false;
        }
        if let Some(log) = self.check_uaf(src, len_u32) {
            let msg = self.format_uaf_message(log, false);
            self.trap(&msg, &SourceLoc::default());
            return false;
        }
        // 用临时 buffer 避免重叠问题
        let tmp = self.memory[src as usize..src_end].to_vec();
        self.memory[dst as usize..dst_end].copy_from_slice(&tmp);
        self.mark_dirty_page(dst, len as u32);
        true
    }

    pub fn get_stack(&self) -> &[u64] {
        &self.stack
    }

    pub fn get_symbols(&self) -> &[VMSymbol] {
        &self.symbols
    }

    fn rebuild_local_sym_map(&mut self) {
        self.local_sym_map.clear();
        if let Some(frame) = self.call_stack.last() {
            for sym in &self.symbols {
                if sym.is_local && sym.func_name == frame.func_name {
                    self.local_sym_map.insert(sym.addr as i32, sym.name.clone());
                }
            }
        }
    }

    pub fn get_call_stack(&self) -> &[CallFrame] {
        &self.call_stack
    }

    pub fn get_last_accessed_vars(&self) -> &[VariableAccess] {
        &self.last_accessed_vars
    }

    pub fn was_step_event_hit(&self) -> bool {
        self.step_event_hit
    }

    // --- Stack helpers ---

    pub fn pop(&mut self) -> u64 {
        match self.stack.pop() {
            Some(v) => v,
            None => {
                self.trap("运行时错误：栈下溢", &SourceLoc::default());
                0
            }
        }
    }

    pub fn push(&mut self, val: u64) {
        if self.stack.len() >= MAX_STACK_DEPTH {
            self.trap(
                "值栈溢出：栈深度超过限制。请检查是否有无限递归或过多嵌套表达式。",
                &SourceLoc::default(),
            );
            return;
        }
        self.stack.push(val);
    }

    // --- Memory helpers ---

    fn check_uaf(&self, addr: u32, size: u32) -> Option<&FreedRegionInfo> {
        self.freed_logs
            .iter()
            .find(|log| addr < log.addr + log.size && addr + size > log.addr)
    }

    fn format_uaf_message(&self, log: &FreedRegionInfo, is_write: bool) -> String {
        let action = if is_write { "写入" } else { "读取" };
        format!(
            "💥 Use-After-Free (E3060)：你正在向一块已经在第 {} 行（第 {} 步）被 free 的内存{}（由第 {} 行的 malloc/realloc 分配）。\n\n⏱️ 时间轴：分配 → 释放（第 {} 步）→ 继续访问（第 {} 步）。\n\n💡 原因：指针在 free 后没有置为 NULL，或者存在别名指针（另一个指针也指向同一块内存）。\n✅ 解决方法：free(p) 后立即写 p = NULL;，并确保不再通过其他指针访问这块内存。",
            log.freed_line, log.freed_step, action, log.alloc_line, log.freed_step, self.step_count
        )
    }

    fn check_mem_access(&mut self, addr: u32, size: u32, loc: &SourceLoc, is_write: bool) -> bool {
        if addr < NULL_TRAP_SIZE {
            let msg = if is_write {
                format!("向 NULL 指针区域写入（地址 0x{:04X}）。请确认指针已被正确初始化。", addr)
            } else {
                format!(
                    "访问了 NULL 指针区域（地址 0x{:04X}）。NULL 指针不能解引用。请确认指针已被正确初始化。",
                    addr
                )
            };
            self.trap(&msg, loc);
            return false;
        }
        if addr as u64 + size as u64 > MEM_SIZE as u64 {
            self.trap(&self.format_bounds_error(addr), loc);
            return false;
        }
        true
    }

    pub fn load_i32(&mut self, addr: u32, loc: &SourceLoc) -> i32 {
        if !self.check_mem_access(addr, 4, loc, false) {
            return 0;
        }
        if let Some(log) = self.check_uaf(addr, 4) {
            let msg = self.format_uaf_message(log, false);
            self.trap(&msg, loc);
            return 0;
        }
        i32::from_le_bytes([
            self.memory[addr as usize],
            self.memory[addr as usize + 1],
            self.memory[addr as usize + 2],
            self.memory[addr as usize + 3],
        ])
    }

    pub fn store_i32(&mut self, addr: u32, val: i32, loc: &SourceLoc) {
        if !self.check_mem_access(addr, 4, loc, true) {
            return;
        }
        if let Some(log) = self.check_uaf(addr, 4) {
            let msg = self.format_uaf_message(log, true);
            self.trap(&msg, loc);
            return;
        }
        let bytes = val.to_le_bytes();
        self.memory[addr as usize..addr as usize + 4].copy_from_slice(&bytes);
        self.mark_dirty_page(addr, 4);
    }

    pub fn load_i64(&mut self, addr: u32, loc: &SourceLoc) -> u64 {
        if !self.check_mem_access(addr, 8, loc, false) {
            return 0;
        }
        if let Some(log) = self.check_uaf(addr, 8) {
            let msg = self.format_uaf_message(log, false);
            self.trap(&msg, loc);
            return 0;
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.memory[addr as usize..addr as usize + 8]);
        u64::from_le_bytes(bytes)
    }

    pub fn store_i64(&mut self, addr: u32, val: u64, loc: &SourceLoc) {
        if !self.check_mem_access(addr, 8, loc, true) {
            return;
        }
        if let Some(log) = self.check_uaf(addr, 8) {
            let msg = self.format_uaf_message(log, true);
            self.trap(&msg, loc);
            return;
        }
        let bytes = val.to_le_bytes();
        self.memory[addr as usize..addr as usize + 8].copy_from_slice(&bytes);
        self.mark_dirty_page(addr, 8);
    }

    pub fn load_i8(&mut self, addr: u32, loc: &SourceLoc) -> i32 {
        if !self.check_mem_access(addr, 1, loc, false) {
            return 0;
        }
        if let Some(log) = self.check_uaf(addr, 1) {
            let msg = self.format_uaf_message(log, false);
            self.trap(&msg, loc);
            return 0;
        }
        self.memory[addr as usize] as i8 as i32
    }

    pub fn store_i8(&mut self, addr: u32, val: i32, loc: &SourceLoc) {
        if !self.check_mem_access(addr, 1, loc, true) {
            return;
        }
        if let Some(log) = self.check_uaf(addr, 1) {
            let msg = self.format_uaf_message(log, true);
            self.trap(&msg, loc);
            return;
        }
        self.memory[addr as usize] = val as u8;
        self.mark_dirty_page(addr, 1);
    }

    // --- Error formatting ---

    fn read_variable(&self, sym: &VMSymbol) -> i64 {
        let vaddr = if sym.is_local {
            if let Some(frame) = self.call_stack.last() {
                frame.locals_base + sym.addr
            } else {
                return 0;
            }
        } else {
            GLOBAL_START + sym.addr
        };
        if vaddr + 4 > MEM_SIZE || vaddr < NULL_TRAP_SIZE {
            return 0;
        }
        if matches!(
            sym.ty.kind(),
            crate::compiler::ast::TypeKind::Double | crate::compiler::ast::TypeKind::LongLong
        ) {
            if vaddr + 8 > MEM_SIZE {
                return 0;
            }
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&self.memory[vaddr as usize..vaddr as usize + 8]);
            u64::from_le_bytes(bytes) as i64
        } else {
            i32::from_le_bytes([
                self.memory[vaddr as usize],
                self.memory[vaddr as usize + 1],
                self.memory[vaddr as usize + 2],
                self.memory[vaddr as usize + 3],
            ]) as i64
        }
    }

    pub fn get_variable_snapshot(&self) -> Vec<crate::session::VariableSnapshot> {
        self.symbols
            .iter()
            .filter_map(|sym| {
                let vaddr = if sym.is_local {
                    if let Some(frame) = self.call_stack.last() {
                        frame.locals_base + sym.addr
                    } else {
                        return None;
                    }
                } else {
                    GLOBAL_START + sym.addr
                };
                if vaddr + 4 > MEM_SIZE || vaddr < NULL_TRAP_SIZE {
                    return None;
                }
                let val = if matches!(sym.ty.kind(), crate::compiler::ast::TypeKind::Double) {
                    if vaddr + 8 > MEM_SIZE {
                        return None;
                    }
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&self.memory[vaddr as usize..vaddr as usize + 8]);
                    u64::from_le_bytes(bytes) as i64
                } else {
                    i32::from_le_bytes([
                        self.memory[vaddr as usize],
                        self.memory[vaddr as usize + 1],
                        self.memory[vaddr as usize + 2],
                        self.memory[vaddr as usize + 3],
                    ]) as i64
                };
                Some(crate::session::VariableSnapshot {
                    name: sym.name.clone(),
                    addr: vaddr,
                    is_local: sym.is_local,
                    ty: sym.ty.clone(),
                    value: val,
                })
            })
            .collect()
    }

    /// 获取所有数组变量的元素快照（用于算法可视化条形图）。
    pub fn get_array_snapshots(&self) -> Vec<crate::unified::types::ArraySnapshot> {
        use crate::compiler::ast::TypeKind;
        let mut result = Vec::new();
        for sym in &self.symbols {
            if sym.ty.kind() != TypeKind::Array {
                continue;
            }
            let vaddr = if sym.is_local {
                if let Some(frame) = self.call_stack.last() {
                    frame.locals_base + sym.addr
                } else {
                    continue;
                }
            } else {
                GLOBAL_START + sym.addr
            };
            let array_size = sym.ty.array_size();
            if array_size <= 0 {
                continue;
            }
            let base_kind = base_kind(&sym.ty);
            let elem_size = match base_kind {
                TypeKind::Char => 1,
                TypeKind::Int | TypeKind::Pointer | TypeKind::Float => 4,
                TypeKind::Double | TypeKind::LongLong => 8,
                _ => 4,
            };
            let mut elements = Vec::with_capacity(array_size as usize);
            for i in 0..array_size {
                let addr = vaddr + (i as u32) * (elem_size as u32);
                let elem_size_u32 = elem_size as u32;
                // NULL 区检查；上界在后续索引前已经判断。
                if addr < NULL_TRAP_SIZE || addr + elem_size_u32 > MEM_SIZE {
                    break;
                }
                let val_str = match base_kind {
                    TypeKind::Char => {
                        let b = self.memory[addr as usize];
                        if (32..=126).contains(&b) {
                            format!("'{}'", b as char)
                        } else {
                            format!("{}", b as i8)
                        }
                    }
                    TypeKind::Int => {
                        let bytes = [
                            self.memory[addr as usize],
                            self.memory[addr as usize + 1],
                            self.memory[addr as usize + 2],
                            self.memory[addr as usize + 3],
                        ];
                        i32::from_le_bytes(bytes).to_string()
                    }
                    TypeKind::Float => {
                        let bytes = [
                            self.memory[addr as usize],
                            self.memory[addr as usize + 1],
                            self.memory[addr as usize + 2],
                            self.memory[addr as usize + 3],
                        ];
                        format!("{:.2}", f32::from_le_bytes(bytes))
                    }
                    TypeKind::Double => {
                        let mut bytes = [0u8; 8];
                        bytes.copy_from_slice(&self.memory[addr as usize..addr as usize + 8]);
                        format!("{:.2}", f64::from_le_bytes(bytes))
                    }
                    TypeKind::LongLong => {
                        let mut bytes = [0u8; 8];
                        bytes.copy_from_slice(&self.memory[addr as usize..addr as usize + 8]);
                        i64::from_le_bytes(bytes).to_string()
                    }
                    _ => "?".to_string(),
                };
                elements.push(val_str);
            }
            let element_ty = match base_kind {
                TypeKind::Int => "int",
                TypeKind::Char => "char",
                TypeKind::Float => "float",
                TypeKind::Double => "double",
                TypeKind::LongLong => "long long",
                _ => "unknown",
            }
            .to_string();
            result.push(crate::unified::types::ArraySnapshot {
                name: sym.name.clone(),
                element_ty,
                elements,
            });
        }
        result
    }

    // --- Run ---

    #[cfg(test)]
    pub fn set_test_code(&mut self, code: Vec<Instruction>) {
        self.code = code;
        self.ip = 0;
    }
}

mod executor;
mod trap;
