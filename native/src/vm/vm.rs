use super::host_funcs::execute_host_func;
use super::instruction::{Instruction, SourceLoc};
use super::jit_templates::{CompiledTrace, execute_trace_bulk};
use std::sync::Arc;
use super::jit_trace::{JIT_THRESHOLD, JitStats, TraceRecorder};
use super::opcode::OpCode;
use crate::compiler::ast::Type;
use crate::session::{Session, VisEvent};
use std::collections::{HashMap, HashSet};

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

fn base_kind(ty: &Type) -> crate::compiler::ast::TypeKind {
    match ty {
        Type::Pointer { pointee, .. } => pointee.kind(),
        Type::Array { element, .. } => base_kind(element),
        _ => ty.kind(),
    }
}

#[derive(Debug, Clone, Default)]
pub struct FuncMeta {
    pub ip: usize,
    /// 参数总 word 数（以 4-byte words 计），供 Call 指令弹栈使用。
    pub arg_count: i32,
    /// 参数个数（供 call_user_function 使用，与总 word 数不同）。
    pub param_count: i32,
    pub local_count: i32,
    pub param_sizes: Vec<i32>,
}

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
}

#[derive(Debug, Clone)]
pub struct VMSymbol {
    pub name: String,
    pub addr: u32,
    pub is_local: bool,
    pub ty: Type,
    pub scope_depth: i32,
    pub func_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StepResult {
    Ok,
    Paused,
    Finished,
    Trap,
    WaitingInput,
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
            memory: self.memory.clone(),
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
        let len = snap.memory.len().min(self.memory.len());
        self.memory[..len].copy_from_slice(&snap.memory[..len]);

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
    pub fn call_user_function(&mut self, session: &mut Session, func_idx: u32, args: &[i32], max_steps: i32) -> Option<i32> {
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
        self.mem_stack_top -= frame_size_u32;
        let locals_base = self.mem_stack_top;
        // Arguments: args[0] is first param, args[n-1] is last param.
        // VM Call convention: first param is at locals_base + 0
        // 当前 call_user_function 仅用于 qsort 回调（参数均为 4 字节指针）。
        // 若未来扩展为 8 字节参数（double/long long），需按 type_size 选择 store_i32/store_i64。
        debug_assert!(
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
        for addr in (locals_base + arg_bytes)..(locals_base + meta.local_count as u32) {
            self.memory[addr as usize] = 0;
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

    pub fn jit_traces_mut(&mut self) -> &mut std::collections::HashMap<usize, std::sync::Arc<crate::vm::jit_templates::CompiledTrace>> {
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

    /// 将 C 风格字符串安全写入 VM 内存的指定地址（含 null 终止符）。
    /// 若目标地址超出边界则静默跳过。
    /// 
    /// 注意：边界检查 `a + bytes.len() < len` 已隐含为末尾的 null 终止符预留了 1 字节空间，
    /// 因此当 `addr + bytes.len() == MEM_SIZE` 时会正确拒绝写入，避免越界。
    pub fn write_cstring(&mut self, addr: u32, s: &str) {
        let a = addr as usize;
        let bytes = s.as_bytes();
        if a + bytes.len() < self.memory.len() {
            self.memory[a..a + bytes.len()].copy_from_slice(bytes);
            self.memory[a + bytes.len()] = 0;
        }
    }

    pub fn memory_ref(&self) -> &[u8] {
        &self.memory
    }

    pub fn memory_ref_mut(&mut self) -> &mut [u8] {
        &mut self.memory
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

    /// 安全写入字节切片到 VM 内存（带 NULL Trap 和边界检查）
    pub fn write_memory(&mut self, addr: u32, data: &[u8]) -> bool {
        let end = addr as usize + data.len();
        if addr < NULL_TRAP_SIZE || end > self.memory.len() {
            return false;
        }
        self.memory[addr as usize..end].copy_from_slice(data);
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
        if dst < NULL_TRAP_SIZE
            || src < NULL_TRAP_SIZE
            || dst_end > self.memory.len()
            || src_end > self.memory.len()
        {
            return false;
        }
        // 用临时 buffer 避免重叠问题
        let tmp = self.memory[src as usize..src_end].to_vec();
        self.memory[dst as usize..dst_end].copy_from_slice(&tmp);
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
            self.trap("值栈溢出：栈深度超过限制。请检查是否有无限递归或过多嵌套表达式。", &SourceLoc::default());
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
                format!("访问了 NULL 指针区域（地址 0x{:04X}）。NULL 指针不能解引用。请确认指针已被正确初始化。", addr)
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
        let bytes = val.to_le_bytes();
        self.memory[addr as usize..addr as usize + 4].copy_from_slice(&bytes);
    }

    pub fn load_i64(&mut self, addr: u32, loc: &SourceLoc) -> u64 {
        if !self.check_mem_access(addr, 8, loc, false) {
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
        let bytes = val.to_le_bytes();
        self.memory[addr as usize..addr as usize + 8].copy_from_slice(&bytes);
    }

    pub fn load_i8(&mut self, addr: u32, loc: &SourceLoc) -> i32 {
        if !self.check_mem_access(addr, 1, loc, false) {
            return 0;
        }
        self.memory[addr as usize] as i8 as i32
    }

    pub fn store_i8(&mut self, addr: u32, val: i32, loc: &SourceLoc) {
        if !self.check_mem_access(addr, 1, loc, true) {
            return;
        }
        self.memory[addr as usize] = val as u8;
    }

    // --- Error formatting ---

    fn format_bounds_error(&self, addr: u32) -> String {
        let mut best_sym: Option<(&VMSymbol, u32, i32)> = None;
        let mut best_dist = i32::MAX;

        for sym in &self.symbols {
            if !matches!(sym.ty.kind(), crate::compiler::ast::TypeKind::Array) || sym.ty.array_size() <= 0 {
                continue;
            }
            let mut base = sym.addr;
            if sym.is_local {
                if let Some(frame) = self.call_stack.last() {
                    base = frame.locals_base + sym.addr;
                } else {
                    continue;
                }
            }
            let elem_size = match base_kind(&sym.ty) {
                crate::compiler::ast::TypeKind::Char => 1,
                crate::compiler::ast::TypeKind::Double => 8,
                _ => 4,
            };
            let size = (sym.ty.array_size() as u32) * elem_size as u32;
            let dist = if addr >= base && addr < base + size {
                0
            } else if addr >= base + size && addr < base + size + 64 {
                (addr - (base + size)) as i32
            } else if addr + 64 >= base && addr < base {
                (base - addr) as i32
            } else {
                continue;
            };
            if dist < best_dist {
                best_dist = dist;
                best_sym = Some((sym, base, dist));
            }
        }

        if let Some((sym, base, _)) = best_sym {
            let elem_size = match base_kind(&sym.ty) {
                crate::compiler::ast::TypeKind::Char => 1,
                crate::compiler::ast::TypeKind::Double => 8,
                _ => 4,
            };
            let index = ((addr as i64 - base as i64) / elem_size as i64) as i32;
            format!(
                "🚫 数组越界：你访问了 {}[{}]，但数组 '{}' 只有 {} 个元素，有效索引是 0~{}。\n\n📍 发生在第 {} 行\n💡 原因：数组索引超出了合法范围。\n✅ 检查方法：确认索引变量值在 0 到 {} 之间。",
                sym.name, index, sym.name, sym.ty.array_size(), sym.ty.array_size() - 1,
                self.current_line, sym.ty.array_size() - 1
            )
        } else {
            format!(
                "🚫 内存访问越界：你访问了地址 0x{:04X}，但合法内存范围是 0x{:04X} ~ 0x{:04X}。\n\n✅ 检查方法：\n  • 确认数组索引小于数组大小\n  • 确认指针已经指向有效的内存地址\n  • 确认没有使用已经 free 的指针",
                addr, NULL_TRAP_SIZE, MEM_SIZE
            )
        }
    }

    fn format_div_zero_error(&self, a: i32, _b: i32) -> String {
        let mut diag = format!("😵 除零错误：你试图用 {} 除以 0。\n\n", a);
        let zero_vars: Vec<String> = self.symbols.iter().filter_map(|sym| {
            if matches!(sym.ty.kind(), crate::compiler::ast::TypeKind::Array) {
                return None;
            }
            let mut vaddr = sym.addr;
            if sym.is_local {
                if let Some(frame) = self.call_stack.last() {
                    vaddr = frame.locals_base + sym.addr;
                } else {
                    return None;
                }
            }
            if vaddr + 4 <= MEM_SIZE && vaddr >= NULL_TRAP_SIZE {
                let val = i32::from_le_bytes([
                    self.memory[vaddr as usize],
                    self.memory[vaddr as usize + 1],
                    self.memory[vaddr as usize + 2],
                    self.memory[vaddr as usize + 3],
                ]);
                if val == 0 { Some(sym.name.clone()) } else { None }
            } else {
                None
            }
        }).collect();

        if !zero_vars.is_empty() {
            diag.push_str("🔍 当前作用域内值为 0 的变量：");
            diag.push_str(&zero_vars.join("、"));
            diag.push_str("。请检查除法表达式中是否使用了这些变量。\n\n");
        }
        diag.push_str("💡 原因：除数不能为 0。\n✅ 检查你的除法表达式，确保除数在被除之前不是 0。\n📝 示例：如果变量 b 可能为 0，先用 if 判断：\n    if (b != 0) {\n        result = a / b;\n    }");
        diag
    }

    fn format_infinite_loop_error(&self) -> String {
        let mut diag = format!("🔄 程序执行步数超过限制（{} 步），可能包含无限循环。\n\n", self.max_steps);
        let mut stale_vars = Vec::new();
        let mut changed_vars = Vec::new();
        for sym in &self.symbols {
            if matches!(sym.ty.kind(), crate::compiler::ast::TypeKind::Array) {
                continue;
            }
            let cur_val = self.read_variable(sym);
            if let Some(&old_val) = self.snapshot_vars.get(&sym.name) {
                if old_val == cur_val as u64 {
                    stale_vars.push(format!("{} = {}", sym.name, cur_val));
                } else {
                    changed_vars.push(format!("{} = {}", sym.name, cur_val));
                }
            }
        }
        if !stale_vars.is_empty() {
            diag.push_str("🔍 在最近 ");
            diag.push_str(&SNAPSHOT_INTERVAL.to_string());
            diag.push_str(" 步内没有变化的变量：");
            let shown: Vec<_> = stale_vars.iter().take(6).cloned().collect();
            diag.push_str(&shown.join("，"));
            if stale_vars.len() > 6 { diag.push_str(" 等"); }
            diag.push_str("。\n\n");
        }
        if !changed_vars.is_empty() {
            diag.push_str("🔍 发生变化的变量：");
            let shown: Vec<_> = changed_vars.iter().take(4).cloned().collect();
            diag.push_str(&shown.join("，"));
            if changed_vars.len() > 4 { diag.push_str(" 等"); }
            diag.push_str("。\n\n");
        }
        diag.push_str("💡 原因：程序执行了太多步数但没有结束。常见原因：\n  • 循环条件永远为真（如 while(1)）\n  • 循环变量没有更新（如忘了写 i++）\n  • 递归函数没有正确的终止条件\n✅ 检查方法：确认循环体中有改变循环条件的语句。");
        diag
    }

    pub fn trap(&mut self, msg: &str, loc: &SourceLoc) {
        if self.error.is_empty() {
            self.error = msg.to_string();
            let line = if loc.line > 0 { loc.line } else { self.current_line };
            if line > 0 {
                self.error.push_str(&format!("\n📍 发生在第 {} 行", line));
                if loc.column > 0 {
                    self.error.push_str(&format!(" 第 {} 列", loc.column));
                }
            }
        }
    }

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
        if matches!(sym.ty.kind(), crate::compiler::ast::TypeKind::Double | crate::compiler::ast::TypeKind::LongLong) {
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
        self.symbols.iter().filter_map(|sym| {
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
        }).collect()
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
                if addr + (elem_size as u32) > MEM_SIZE {
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

    pub fn run(&mut self, session: &mut Session) -> i32 {
        loop {
            // --- JIT fast path ---
            if let Some(trace) = self.jit_traces.get(&self.ip).cloned() {
                let (result, steps) = execute_trace_bulk(self, session, &trace);
                self.jit_stats.steps_accelerated += steps;
                if let Some(r) = result {
                    match r {
                        StepResult::Finished => {
                            return if self.finished { self.exit_code } else { self.stack.last().copied().unwrap_or(0) as i32 };
                        }
                        StepResult::Trap => return 0,
                        StepResult::Paused => {
                            self.trap("完整运行模式下遇到暂停状态（可能是断点配置不一致）", &SourceLoc::default());
                            return 0;
                        }
                        StepResult::WaitingInput => {
                            return 0;
                        }
                        StepResult::Ok => {}
                    }
                }
                // trace 正常退出（ip 已离开循环），继续外层调度
                continue;
            }

            let result = self.step(session);
            match result {
                StepResult::Finished => {
                    return if self.finished { self.exit_code } else { self.stack.last().copied().unwrap_or(0) as i32 };
                }
                StepResult::Trap => return 0,
                StepResult::Paused => {
                    self.trap("完整运行模式下遇到暂停状态（可能是断点配置不一致）", &SourceLoc::default());
                    return 0;
                }
                StepResult::WaitingInput => {
                    return 0;
                }
                StepResult::Ok => {}
            }
        }
    }



    fn execute_stack(&mut self, op: OpCode, operand: i32, loc: &SourceLoc) {
        match op {
                        OpCode::PushConst => {
                            self.push(operand as u64);
                        }
                        OpCode::Pop => {
                            self.pop();
                        }
                        OpCode::Dup => {
                            if let Some(&v) = self.stack.last() {
                                self.push(v);
                            } else {
                                self.trap("Dup: 栈空", loc);
                            }
                        }
                        OpCode::Swap => {
                            let len = self.stack.len();
                            if len >= 2 {
                                self.stack.swap(len - 1, len - 2);
                            } else {
                                self.trap("Swap: 栈不足", loc);
                            }
                        }
            _ => {}
        }
    }
    fn execute_local(&mut self, op: OpCode, operand: i32, loc: &SourceLoc) {
        match op {
                        OpCode::LoadLocal => {
                            let var_name = self.local_sym_map
                                .get(&operand)
                                .cloned()
                                .unwrap_or_else(|| format!("local_{}", operand));
                            self.last_accessed_vars.push(VariableAccess {
                                name: var_name,
                                access_type: AccessType::Read,
                            });
                            if let Some(frame) = self.call_stack.last() {
                                let addr = frame.locals_base + operand as u32;
                                if addr as u64 + 4 > MEM_SIZE as u64 || addr < NULL_TRAP_SIZE {
                                    self.trap("LoadLocal: 地址越界", loc);
                                } else {
                                    let val = self.load_i32(addr, loc);
                                    self.push(val as u64);
                                }
                            } else {
                                self.trap("LoadLocal: 无调用帧", loc);
                            }
                        }
                        OpCode::StoreLocal => {
                            let var_name = self.local_sym_map
                                .get(&operand)
                                .cloned()
                                .unwrap_or_else(|| format!("local_{}", operand));
                            self.last_accessed_vars.push(VariableAccess {
                                name: var_name,
                                access_type: AccessType::Write,
                            });
                            if let Some(frame) = self.call_stack.last() {
                                let addr = frame.locals_base + operand as u32;
                                if addr as u64 + 4 > MEM_SIZE as u64 || addr < NULL_TRAP_SIZE {
                                    self.trap("StoreLocal: 地址越界", loc);
                                } else {
                                    let val = self.pop() as i32;
                                    self.store_i32(addr, val, loc);
                                }
                            } else {
                                self.trap("StoreLocal: 无调用帧", loc);
                            }
                        }
                        OpCode::LoadLocalD => {
                            if let Some(frame) = self.call_stack.last() {
                                let addr = frame.locals_base + operand as u32;
                                if addr as u64 + 8 > MEM_SIZE as u64 || addr < NULL_TRAP_SIZE {
                                    self.trap("LoadLocalD: 地址越界", loc);
                                } else {
                                    let val = self.load_i64(addr, loc);
                                    self.push(val);
                                }
                            } else {
                                self.trap("LoadLocalD: 无调用帧", loc);
                            }
                        }
                        OpCode::StoreLocalD => {
                            if let Some(frame) = self.call_stack.last() {
                                let addr = frame.locals_base + operand as u32;
                                if addr as u64 + 8 > MEM_SIZE as u64 || addr < NULL_TRAP_SIZE {
                                    self.trap("StoreLocalD: 地址越界", loc);
                                } else {
                                    let val = self.pop();
                                    self.store_i64(addr, val, loc);
                                }
                            } else {
                                self.trap("StoreLocalD: 无调用帧", loc);
                            }
                        }
                        OpCode::GetFrameBase => {
                            if let Some(frame) = self.call_stack.last() {
                                self.push(frame.locals_base as u64);
                            } else {
                                self.trap("GetFrameBase: 无调用帧", loc);
                            }
                        }
                        OpCode::LoadLocalQ => {
                            if let Some(frame) = self.call_stack.last() {
                                let addr = frame.locals_base + operand as u32;
                                if addr as u64 + 8 > MEM_SIZE as u64 || addr < NULL_TRAP_SIZE {
                                    self.trap("LoadLocalQ: 地址越界", loc);
                                } else {
                                    let val = self.load_i64(addr, loc);
                                    self.push(val);
                                }
                            } else {
                                self.trap("LoadLocalQ: 无调用帧", loc);
                            }
                        }
                        OpCode::StoreLocalQ => {
                            if let Some(frame) = self.call_stack.last() {
                                let addr = frame.locals_base + operand as u32;
                                if addr as u64 + 8 > MEM_SIZE as u64 || addr < NULL_TRAP_SIZE {
                                    self.trap("StoreLocalQ: 地址越界", loc);
                                } else {
                                    let val = self.pop();
                                    self.store_i64(addr, val, loc);
                                }
                            } else {
                                self.trap("StoreLocalQ: 无调用帧", loc);
                            }
                        }
            _ => {}
        }
    }
    fn execute_global(&mut self, op: OpCode, operand: i32, loc: &SourceLoc) {
        match op {
                        OpCode::LoadGlobal => {
                            let var_name = self.global_sym_map
                                .get(&operand)
                                .cloned()
                                .unwrap_or_else(|| format!("global_{}", operand));
                            self.last_accessed_vars.push(VariableAccess {
                                name: var_name,
                                access_type: AccessType::Read,
                            });
                            let addr = GLOBAL_START + operand as u32;
                            if addr as u64 + 4 > MEM_SIZE as u64 || addr < NULL_TRAP_SIZE {
                                self.trap("LoadGlobal: 地址越界", loc);
                            } else {
                                let val = self.load_i32(addr, loc);
                                self.push(val as u64);
                            }
                        }
                        OpCode::StoreGlobal => {
                            let var_name = self.global_sym_map
                                .get(&operand)
                                .cloned()
                                .unwrap_or_else(|| format!("global_{}", operand));
                            self.last_accessed_vars.push(VariableAccess {
                                name: var_name,
                                access_type: AccessType::Write,
                            });
                            let addr = GLOBAL_START + operand as u32;
                            if addr as u64 + 4 > MEM_SIZE as u64 || addr < NULL_TRAP_SIZE {
                                self.trap("StoreGlobal: 地址越界", loc);
                            } else {
                                let val = self.pop() as i32;
                                self.store_i32(addr, val, loc);
                            }
                        }
                        OpCode::LoadGlobalD => {
                            let addr = GLOBAL_START + operand as u32;
                            if addr as u64 + 8 > MEM_SIZE as u64 || addr < NULL_TRAP_SIZE {
                                self.trap("LoadGlobalD: 地址越界", loc);
                            } else {
                                let val = self.load_i64(addr, loc);
                                self.push(val);
                            }
                        }
                        OpCode::StoreGlobalD => {
                            let addr = GLOBAL_START + operand as u32;
                            if addr as u64 + 8 > MEM_SIZE as u64 || addr < NULL_TRAP_SIZE {
                                self.trap("StoreGlobalD: 地址越界", loc);
                            } else {
                                let val = self.pop();
                                self.store_i64(addr, val, loc);
                            }
                        }
                        OpCode::LoadGlobalQ => {
                            let addr = GLOBAL_START + operand as u32;
                            if addr as u64 + 8 > MEM_SIZE as u64 || addr < NULL_TRAP_SIZE {
                                self.trap("LoadGlobalQ: 地址越界", loc);
                            } else {
                                let val = self.load_i64(addr, loc);
                                self.push(val);
                            }
                        }
                        OpCode::StoreGlobalQ => {
                            let addr = GLOBAL_START + operand as u32;
                            if addr as u64 + 8 > MEM_SIZE as u64 || addr < NULL_TRAP_SIZE {
                                self.trap("StoreGlobalQ: 地址越界", loc);
                            } else {
                                let val = self.pop();
                                self.store_i64(addr, val, loc);
                            }
                        }
            _ => {}
        }
    }
    fn execute_memory(&mut self, op: OpCode, _operand: i32, loc: &SourceLoc) {
        match op {
                        OpCode::LoadMem => {
                            let addr = self.pop() as u32;
                            if let Some(log) = self.check_uaf(addr, 4) {
                                let msg = self.format_uaf_message(log, false);
                                self.trap(&msg, loc);
                                return;
                            }
                            let val = self.load_i32(addr, loc);
                            self.push(val as u64);
                        }
                        OpCode::StoreMem => {
                            let val = self.pop() as i32;
                            let addr = self.pop() as u32;
                            if let Some(log) = self.check_uaf(addr, 4) {
                                let msg = self.format_uaf_message(log, true);
                                self.trap(&msg, loc);
                                return;
                            }
                            self.store_i32(addr, val, loc);
                        }
                        OpCode::LoadMemD => {
                            let addr = self.pop() as u32;
                            if let Some(log) = self.check_uaf(addr, 8) {
                                let msg = self.format_uaf_message(log, false);
                                self.trap(&msg, loc);
                                return;
                            }
                            let val = self.load_i64(addr, loc);
                            self.push(val);
                        }
                        OpCode::StoreMemD => {
                            let val = self.pop();
                            let addr = self.pop() as u32;
                            if let Some(log) = self.check_uaf(addr, 8) {
                                let msg = self.format_uaf_message(log, true);
                                self.trap(&msg, loc);
                                return;
                            }
                            self.store_i64(addr, val, loc);
                        }
                        OpCode::SplitD => {
                            let val = self.pop();
                            let low = (val & 0xFFFFFFFF) as i32;
                            let high = ((val >> 32) & 0xFFFFFFFF) as i32;
                            self.push(low as u64);
                            self.push(high as u64);
                        }
                        OpCode::LoadMemByte => {
                            let addr = self.pop() as u32;
                            if let Some(log) = self.check_uaf(addr, 1) {
                                let msg = self.format_uaf_message(log, false);
                                self.trap(&msg, loc);
                                return;
                            }
                            let val = self.load_i8(addr, loc);
                            self.push(val as u64);
                        }
                        OpCode::StoreMemByte => {
                            let val = self.pop() as i32;
                            let addr = self.pop() as u32;
                            if let Some(log) = self.check_uaf(addr, 1) {
                                let msg = self.format_uaf_message(log, true);
                                self.trap(&msg, loc);
                                return;
                            }
                            self.store_i8(addr, val, loc);
                        }
                        OpCode::LoadMemQ => {
                            let addr = self.pop() as u32;
                            if let Some(log) = self.check_uaf(addr, 8) {
                                let msg = self.format_uaf_message(log, false);
                                self.trap(&msg, loc);
                                return;
                            }
                            let val = self.load_i64(addr, loc);
                            self.push(val);
                        }
                        OpCode::StoreMemQ => {
                            let val = self.pop();
                            let addr = self.pop() as u32;
                            if let Some(log) = self.check_uaf(addr, 8) {
                                let msg = self.format_uaf_message(log, true);
                                self.trap(&msg, loc);
                                return;
                            }
                            self.store_i64(addr, val, loc);
                        }
                        OpCode::SplitQ => {
                            let val = self.pop();
                            let low = (val & 0xFFFFFFFF) as i32;
                            let high = ((val >> 32) & 0xFFFFFFFF) as i32;
                            self.push(low as u64);
                            self.push(high as u64);
                        }
            _ => {}
        }
    }
    fn execute_arithmetic(&mut self, op: OpCode, _operand: i32, loc: &SourceLoc) {
        match op {
                        OpCode::Add => {
                            let b = self.pop() as i32;
                            let a = self.pop() as i32;
                            let r = (a as i64) + (b as i64);
                            if r > i32::MAX as i64 || r < i32::MIN as i64 {
                                self.trap("整数加法溢出。两个很大的正数（或很小的负数）相加超出了 int 能表示的范围。", loc);
                            } else {
                                self.push(r as u64);
                            }
                        }
                        OpCode::Sub => {
                            let b = self.pop() as i32;
                            let a = self.pop() as i32;
                            let r = (a as i64) - (b as i64);
                            if r > i32::MAX as i64 || r < i32::MIN as i64 {
                                self.trap("整数减法溢出。被减数太小而减数太大，结果超出了 int 能表示的范围。", loc);
                            } else {
                                self.push(r as u64);
                            }
                        }
                        OpCode::Mul => {
                            let b = self.pop() as i32;
                            let a = self.pop() as i32;
                            let r = (a as i64) * (b as i64);
                            if r > i32::MAX as i64 || r < i32::MIN as i64 {
                                self.trap("整数乘法溢出。乘积太大，超出了 int 能表示的范围。", loc);
                            } else {
                                self.push(r as u64);
                            }
                        }
                        OpCode::Div => {
                            let b = self.pop() as i32;
                            let a = self.pop() as i32;
                            if b == 0 {
                                let msg = self.format_div_zero_error(a, b);
                                self.trap(&msg, loc);
                            } else if a == i32::MIN && b == -1 {
                                self.trap("整数除法溢出。INT_MIN / -1 的结果超出了 int 能表示的范围。", loc);
                            } else {
                                self.push((a / b) as u64);
                            }
                        }
                        OpCode::Mod => {
                            let b = self.pop() as i32;
                            let a = self.pop() as i32;
                            if b == 0 {
                                let msg = self.format_div_zero_error(a, b);
                                self.trap(&msg, loc);
                            } else {
                                self.push((a % b) as u64);
                            }
                        }
                        OpCode::Neg => {
                            let a = self.pop() as i32;
                            if a == i32::MIN {
                                self.trap("整数取反溢出。-INT_MIN 的结果超出了 int 能表示的范围。", loc);
                            } else {
                                self.push((-a) as u64);
                            }
                        }
            _ => {}
        }
    }
    fn execute_comparison(&mut self, op: OpCode, _operand: i32, _loc: &SourceLoc) {
        match op {
                        OpCode::Eq => { let b = self.pop() as i32; let a = self.pop() as i32; self.push(if a == b { 1 } else { 0 }); }
                        OpCode::Ne => { let b = self.pop() as i32; let a = self.pop() as i32; self.push(if a != b { 1 } else { 0 }); }
                        OpCode::Lt => { let b = self.pop() as i32; let a = self.pop() as i32; self.push(if a < b { 1 } else { 0 }); }
                        OpCode::Le => { let b = self.pop() as i32; let a = self.pop() as i32; self.push(if a <= b { 1 } else { 0 }); }
                        OpCode::Gt => { let b = self.pop() as i32; let a = self.pop() as i32; self.push(if a > b { 1 } else { 0 }); }
                        OpCode::Ge => { let b = self.pop() as i32; let a = self.pop() as i32; self.push(if a >= b { 1 } else { 0 }); }
                        OpCode::And => { let b = self.pop() as i32; let a = self.pop() as i32; self.push(if a != 0 && b != 0 { 1 } else { 0 }); }
                        OpCode::Or  => { let b = self.pop() as i32; let a = self.pop() as i32; self.push(if a != 0 || b != 0 { 1 } else { 0 }); }
                        OpCode::Not => { let a = self.pop() as i32; self.push(if a != 0 { 0 } else { 1 }); }
            _ => {}
        }
    }
    fn execute_bitwise(&mut self, op: OpCode, _operand: i32, loc: &SourceLoc) {
        match op {
                        OpCode::BitAnd => { let b = self.pop() as i32; let a = self.pop() as i32; self.push((a & b) as u64); }
                        OpCode::BitOr  => { let b = self.pop() as i32; let a = self.pop() as i32; self.push((a | b) as u64); }
                        OpCode::BitXor => { let b = self.pop() as i32; let a = self.pop() as i32; self.push((a ^ b) as u64); }
                        OpCode::BitNot => { let a = self.pop() as i32; self.push((!a) as u64); }
                        OpCode::Shl => {
                            let b = self.pop() as i32;
                            let a = self.pop() as i32;
                            if !(0..32).contains(&b) {
                                self.trap(&format!("Shl 移位量越界：{}（必须是 0~31）", b), loc);
                            } else {
                                self.push((a << b) as u64);
                            }
                        }
                        OpCode::Shr => {
                            let b = self.pop() as i32;
                            let a = self.pop() as i32;
                            if !(0..32).contains(&b) {
                                self.trap(&format!("Shr 移位量越界：{}（必须是 0~31）", b), loc);
                            } else {
                                self.push((a >> b) as u64);
                            }
                        }
            _ => {}
        }
    }
    fn execute_float(&mut self, op: OpCode, operand: i32, loc: &SourceLoc) {
        match op {
                        OpCode::PushConstF => { self.push(operand as u32 as u64); }
                        OpCode::AddF => {
                            let b = f32::from_bits(self.pop() as u32);
                            let a = f32::from_bits(self.pop() as u32);
                            let r = a + b;
                            self.push(r.to_bits() as u64);
                        }
                        OpCode::SubF => {
                            let b = f32::from_bits(self.pop() as u32);
                            let a = f32::from_bits(self.pop() as u32);
                            let r = a - b;
                            self.push(r.to_bits() as u64);
                        }
                        OpCode::MulF => {
                            let b = f32::from_bits(self.pop() as u32);
                            let a = f32::from_bits(self.pop() as u32);
                            let r = a * b;
                            self.push(r.to_bits() as u64);
                        }
                        OpCode::DivF => {
                            let b = f32::from_bits(self.pop() as u32);
                            let a = f32::from_bits(self.pop() as u32);
                            if b == 0.0 {
                                self.trap("浮点数除以零", loc);
                            } else {
                                let r = a / b;
                                self.push(r.to_bits() as u64);
                            }
                        }
                        OpCode::NegF => {
                            let a = f32::from_bits(self.pop() as u32);
                            let r = -a;
                            self.push(r.to_bits() as u64);
                        }
                        OpCode::EqF => { let b = f32::from_bits(self.pop() as u32); let a = f32::from_bits(self.pop() as u32); self.push(if (a - b).abs() < EPS_F32 { 1 } else { 0 }); }
                        OpCode::NeF => { let b = f32::from_bits(self.pop() as u32); let a = f32::from_bits(self.pop() as u32); self.push(if (a - b).abs() >= EPS_F32 { 1 } else { 0 }); }
                        OpCode::LtF => { let b = f32::from_bits(self.pop() as u32); let a = f32::from_bits(self.pop() as u32); self.push(if a + EPS_F32 < b { 1 } else { 0 }); }
                        OpCode::LeF => { let b = f32::from_bits(self.pop() as u32); let a = f32::from_bits(self.pop() as u32); self.push(if a < b + EPS_F32 { 1 } else { 0 }); }
                        OpCode::GtF => { let b = f32::from_bits(self.pop() as u32); let a = f32::from_bits(self.pop() as u32); self.push(if a > b + EPS_F32 { 1 } else { 0 }); }
                        OpCode::GeF => { let b = f32::from_bits(self.pop() as u32); let a = f32::from_bits(self.pop() as u32); self.push(if a + EPS_F32 > b { 1 } else { 0 }); }
                        OpCode::CastI2F => {
                            let a = self.pop() as i32;
                            self.push((a as f32).to_bits() as u64);
                        }
                        OpCode::CastF2I => {
                            let a = f32::from_bits(self.pop() as u32);
                            self.push(a as i32 as u64);
                        }
            _ => {}
        }
    }
    fn execute_double(&mut self, op: OpCode, operand: i32, loc: &SourceLoc) {
        match op {
                        OpCode::PushConstD => {
                            let idx = operand as usize;
                            let val = match self.f64_constants.get(idx) {
                                Some(&v) => v,
                                None => {
                                    self.trap(&format!("f64常量索引越界: {}", idx), loc);
                                    return;
                                }
                            };
                            self.push(val.to_bits());
                        }
                        OpCode::AddD => {
                            let b = f64::from_bits(self.pop());
                            let a = f64::from_bits(self.pop());
                            self.push((a + b).to_bits());
                        }
                        OpCode::SubD => {
                            let b = f64::from_bits(self.pop());
                            let a = f64::from_bits(self.pop());
                            self.push((a - b).to_bits());
                        }
                        OpCode::MulD => {
                            let b = f64::from_bits(self.pop());
                            let a = f64::from_bits(self.pop());
                            self.push((a * b).to_bits());
                        }
                        OpCode::DivD => {
                            let b = f64::from_bits(self.pop());
                            let a = f64::from_bits(self.pop());
                            if b == 0.0 {
                                self.trap("double 除以零", loc);
                            } else {
                                self.push((a / b).to_bits());
                            }
                        }
                        OpCode::NegD => {
                            let a = f64::from_bits(self.pop());
                            self.push((-a).to_bits());
                        }
                        OpCode::CastI2D => {
                            let a = self.pop() as i32;
                            self.push((a as f64).to_bits());
                        }
                        OpCode::CastF2D => {
                            let a = f32::from_bits(self.pop() as u32);
                            self.push((a as f64).to_bits());
                        }
                        OpCode::CastD2I => {
                            let a = f64::from_bits(self.pop());
                            self.push(a as i32 as u64);
                        }
                        OpCode::CastD2F => {
                            let a = f64::from_bits(self.pop());
                            self.push((a as f32).to_bits() as u64);
                        }
                        OpCode::EqD => { let b = f64::from_bits(self.pop()); let a = f64::from_bits(self.pop()); self.push(if (a - b).abs() < EPS_F64 { 1 } else { 0 }); }
                        OpCode::NeD => { let b = f64::from_bits(self.pop()); let a = f64::from_bits(self.pop()); self.push(if (a - b).abs() >= EPS_F64 { 1 } else { 0 }); }
                        OpCode::LtD => { let b = f64::from_bits(self.pop()); let a = f64::from_bits(self.pop()); self.push(if a + EPS_F64 < b { 1 } else { 0 }); }
                        OpCode::LeD => { let b = f64::from_bits(self.pop()); let a = f64::from_bits(self.pop()); self.push(if a < b + EPS_F64 { 1 } else { 0 }); }
                        OpCode::GtD => { let b = f64::from_bits(self.pop()); let a = f64::from_bits(self.pop()); self.push(if a > b + EPS_F64 { 1 } else { 0 }); }
                        OpCode::GeD => { let b = f64::from_bits(self.pop()); let a = f64::from_bits(self.pop()); self.push(if a + EPS_F64 > b { 1 } else { 0 }); }
            _ => {}
        }
    }
    fn execute_longlong(&mut self, op: OpCode, operand: i32, loc: &SourceLoc) {
        match op {
                        OpCode::PushConstQ => {
                            let idx = operand as usize;
                            let val = match self.i64_constants.get(idx) {
                                Some(&v) => v,
                                None => {
                                    self.trap(&format!("i64常量索引越界: {}", idx), loc);
                                    return;
                                }
                            };
                            self.push(val as u64);
                        }
                        OpCode::AddQ => {
                            let b = self.pop() as i64;
                            let a = self.pop() as i64;
                            self.push((a.wrapping_add(b)) as u64);
                        }
                        OpCode::SubQ => {
                            let b = self.pop() as i64;
                            let a = self.pop() as i64;
                            self.push((a.wrapping_sub(b)) as u64);
                        }
                        OpCode::MulQ => {
                            let b = self.pop() as i64;
                            let a = self.pop() as i64;
                            self.push((a.wrapping_mul(b)) as u64);
                        }
                        OpCode::DivQ => {
                            let b = self.pop() as i64;
                            let a = self.pop() as i64;
                            if b == 0 {
                                self.trap("long long 除以零", loc);
                            } else {
                                self.push((a / b) as u64);
                            }
                        }
                        OpCode::ModQ => {
                            let b = self.pop() as i64;
                            let a = self.pop() as i64;
                            if b == 0 {
                                self.trap("long long 取模除以零", loc);
                            } else {
                                self.push((a % b) as u64);
                            }
                        }
                        OpCode::NegQ => {
                            let a = self.pop() as i64;
                            self.push((-a) as u64);
                        }
                        OpCode::CastI2Q => {
                            let a = self.pop() as i32;
                            self.push(a as i64 as u64);
                        }
                        OpCode::CastQ2I => {
                            let a = self.pop() as i64;
                            self.push(a as i32 as u64);
                        }
                        OpCode::CastQ2D => {
                            let a = self.pop() as i64;
                            self.push((a as f64).to_bits());
                        }
                        OpCode::CastD2Q => {
                            let a = f64::from_bits(self.pop());
                            self.push(a as i64 as u64);
                        }
                        OpCode::EqQ => { let b = self.pop() as i64; let a = self.pop() as i64; self.push(if a == b { 1 } else { 0 }); }
                        OpCode::NeQ => { let b = self.pop() as i64; let a = self.pop() as i64; self.push(if a != b { 1 } else { 0 }); }
                        OpCode::LtQ => { let b = self.pop() as i64; let a = self.pop() as i64; self.push(if a < b { 1 } else { 0 }); }
                        OpCode::LeQ => { let b = self.pop() as i64; let a = self.pop() as i64; self.push(if a <= b { 1 } else { 0 }); }
                        OpCode::GtQ => { let b = self.pop() as i64; let a = self.pop() as i64; self.push(if a > b { 1 } else { 0 }); }
                        OpCode::GeQ => { let b = self.pop() as i64; let a = self.pop() as i64; self.push(if a >= b { 1 } else { 0 }); }
            _ => {}
        }
    }
    fn do_call(&mut self, func_idx: u32, loc: &SourceLoc, session: &mut Session, op_name: &str) {
        let idx = func_idx as usize;
        if idx >= self.func_table.len() || self.func_table[idx].ip == 0 {
            self.trap(&format!("{}: 未知函数索引 {}", op_name, func_idx), loc);
            return;
        }
        let meta = self.func_table[idx].clone();
        let frame_size = meta.local_count as u64;
        if frame_size > (STACK_START - NULL_TRAP_SIZE) as u64 || frame_size > self.mem_stack_top as u64 {
            self.trap(&format!("{}: 栈溢出", op_name), loc);
            return;
        }
        let frame_size_u32 = frame_size as u32;
        if self.mem_stack_top < NULL_TRAP_SIZE + frame_size_u32 {
            self.trap(&format!("{}: 栈溢出", op_name), loc);
            return;
        }
        let heap_limit = session.memory.heap_offset;
        if self.mem_stack_top - frame_size_u32 < heap_limit {
            self.trap(&format!("{}: 栈溢出（栈与堆发生碰撞）。请减少递归深度或动态内存分配。", op_name), loc);
            return;
        }
        self.mem_stack_top -= frame_size_u32;
        let locals_base = self.mem_stack_top;

        let mut word_offset = 0;
        for word_count in meta.param_sizes.iter() {
            let words = *word_count as u32;
            let addr = locals_base + word_offset * 4;
            for w in (0..words).rev() {
                let val = self.pop() as i32;
                self.store_i32(addr + w * 4, val, loc);
            }
            word_offset += words;
        }
        let arg_bytes = word_offset * 4;
        for addr in (locals_base + arg_bytes)..(locals_base + meta.local_count as u32) {
            self.memory[addr as usize] = 0;
        }
        let func_name = if idx < self.func_names.len() {
            self.func_names[idx].clone()
        } else {
            format!("func_{}", func_idx)
        };
        self.call_stack.push(CallFrame {
            return_ip: self.ip,
            locals_base,
            local_count: meta.local_count,
            func_name,
        });
        self.rebuild_local_sym_map();
        self.ip = meta.ip;
    }

    fn execute_control_flow(&mut self, op: OpCode, operand: i32, loc: &SourceLoc, session: &mut Session) -> Option<StepResult> {
        match op {
                        OpCode::Jump => {
                            let target = operand as usize;
                            if target >= self.code.len() {
                                self.trap(&format!("Jump 目标越界：{}（代码长度：{}）", target, self.code.len()), loc);
                            } else {
                                self.ip = target;
                            }
                            None
                        }
                        OpCode::JumpIfZero => {
                            let val = self.pop();
                            if val == 0 {
                                let target = operand as usize;
                                if target >= self.code.len() {
                                    self.trap(&format!("JumpIfZero 目标越界：{}（代码长度：{}）", target, self.code.len()), loc);
                                } else {
                                    self.ip = target;
                                }
                            }
                            None
                        }
                        OpCode::JumpIfNotZero => {
                            let val = self.pop();
                            if val != 0 {
                                let target = operand as usize;
                                if target >= self.code.len() {
                                    self.trap(&format!("JumpIfNotZero 目标越界：{}（代码长度：{}）", target, self.code.len()), loc);
                                } else {
                                    self.ip = target;
                                }
                            }
                            None
                        }
                        OpCode::Call => {
                            self.do_call(operand as u32, loc, session, "Call");
                            None
                        }
                        OpCode::CallPtr => {
                            if self.stack.is_empty() {
                                self.trap("CallPtr: 栈下溢（缺少函数索引）", loc);
                            } else {
                                let func_idx = self.pop() as u32;
                                if func_idx as usize >= self.func_table.len() {
                                    self.trap(&format!("CallPtr: 函数指针索引 {} 越界，可能指针未正确初始化", func_idx), loc);
                                } else {
                                    self.do_call(func_idx, loc, session, "CallPtr");
                                }
                            }
                            None
                        }
                        OpCode::CallHost => {
                            execute_host_func(self, session, operand as u32);
                            if session.runtime.waiting_input {
                                self.ip -= 1;
                                return Some(StepResult::WaitingInput);
                            }
                            None
                        }
                        OpCode::Ret => {
                            if self.call_stack.is_empty() {
                                return Some(StepResult::Finished);
                            }
                            let ret_val = self.pop();
                            let frame = self.call_stack.pop().unwrap();
                            const HOST_CALLBACK_SENTINEL: usize = usize::MAX;
                            if frame.return_ip == HOST_CALLBACK_SENTINEL {
                                self.mem_stack_top = frame.locals_base;
                                self.push(ret_val);
                                self.local_sym_map.clear();
                                return Some(StepResult::Finished);
                            }
                            self.ip = frame.return_ip;
                            self.mem_stack_top = frame.locals_base;
                            self.push(ret_val);
                            self.rebuild_local_sym_map();
                            None
                        }
                        OpCode::RetVoid => {
                            if self.call_stack.is_empty() {
                                return Some(StepResult::Finished);
                            }
                            let frame = self.call_stack.pop().unwrap();
                            const HOST_CALLBACK_SENTINEL: usize = usize::MAX;
                            if frame.return_ip == HOST_CALLBACK_SENTINEL {
                                self.mem_stack_top = frame.locals_base;
                                self.local_sym_map.clear();
                                return Some(StepResult::Finished);
                            }
                            self.ip = frame.return_ip;
                            self.mem_stack_top = frame.locals_base;
                            self.rebuild_local_sym_map();
                            None
                        }
            _ => None,
        }
    }
    fn execute_debug(&mut self, op: OpCode, operand: i32, loc: &SourceLoc) -> Option<StepResult> {
        match op {
                        OpCode::StepEvent => {
                            self.current_line = operand;
                            if self.breakpoints.contains(&self.current_line) {
                                self.paused = true;
                            }
                            self.step_event_hit = true;
                            for &(line, ty, ref ctx) in &self.vis_event_lines {
                                if line == operand {
                                    self.vis_event_queue.push(VisEvent {
                                        ty,
                                        line: operand,
                                        extra0: 0,
                                        extra1: 0,
                                        extra2: 0,
                                        context: ctx.clone(),
                                    });
                                }
                            }
                            if self.paused {
                                return Some(StepResult::Paused);
                            }
                            None
                        }
                        OpCode::TrapBounds => {
                            let index = if let Some(&val) = self.stack.last() {
                                val as i64
                            } else {
                                self.trap("TrapBounds: 值栈为空，无法获取索引", loc);
                                return Some(StepResult::Trap);
                            };
                            let mut name = "数组".to_string();
                            let mut size = 0;
                            if operand >= 0 {
                                let sym_idx = operand as usize;
                                if sym_idx < self.symbols.len() {
                                    let sym = &self.symbols[sym_idx];
                                    name = sym.name.clone();
                                    size = sym.ty.array_size();
                                }
                            } else {
                                size = -operand;
                            }
                            if index < 0 || index >= size as i64 {
                                let diag = if operand >= 0 {
                                    format!(
                                        "🚫 数组越界：你访问了 {}[{}]，但数组 '{}' 只有 {} 个元素，有效索引是 0~{}。\n\n💡 原因：数组索引超出了合法范围。\n✅ 检查方法：确认索引变量值在 0 到 {} 之间。",
                                        name, index, name, size, size.saturating_sub(1), size.saturating_sub(1)
                                    )
                                } else {
                                    format!(
                                        "🚫 数组越界：索引 {} 超出了合法范围 0~{}。\n\n💡 原因：数组索引超出了合法范围。",
                                        index, size.saturating_sub(1)
                                    )
                                };
                                self.trap(&diag, loc);
                            }
                            None
                        }
            _ => None,
        }
    }

    // --- Single instruction dispatch (used by both step() and JIT generic fallback) ---

    pub(crate) fn dispatch_single_instruction(
        &mut self,
        op: OpCode,
        operand: i32,
        loc: &SourceLoc,
        session: &mut Session,
    ) -> Option<StepResult> {
        match op {
            OpCode::Nop => {}

            OpCode::PushConst | OpCode::Pop | OpCode::Dup | OpCode::Swap => {
                self.execute_stack(op, operand, loc);
            }

            OpCode::LoadLocal | OpCode::StoreLocal | OpCode::LoadLocalD | OpCode::StoreLocalD |
            OpCode::LoadLocalQ | OpCode::StoreLocalQ | OpCode::GetFrameBase => {
                self.execute_local(op, operand, loc);
            }

            OpCode::LoadGlobal | OpCode::StoreGlobal | OpCode::LoadGlobalD | OpCode::StoreGlobalD |
            OpCode::LoadGlobalQ | OpCode::StoreGlobalQ => {
                self.execute_global(op, operand, loc);
            }

            OpCode::LoadMem | OpCode::StoreMem | OpCode::LoadMemD | OpCode::StoreMemD |
            OpCode::LoadMemByte | OpCode::StoreMemByte | OpCode::LoadMemQ | OpCode::StoreMemQ |
            OpCode::SplitD | OpCode::SplitQ => {
                self.execute_memory(op, operand, loc);
            }

            OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div | OpCode::Mod | OpCode::Neg => {
                self.execute_arithmetic(op, operand, loc);
            }

            OpCode::Eq | OpCode::Ne | OpCode::Lt | OpCode::Le | OpCode::Gt | OpCode::Ge |
            OpCode::And | OpCode::Or | OpCode::Not => {
                self.execute_comparison(op, operand, loc);
            }

            OpCode::BitAnd | OpCode::BitOr | OpCode::BitXor | OpCode::BitNot |
            OpCode::Shl | OpCode::Shr => {
                self.execute_bitwise(op, operand, loc);
            }

            OpCode::PushConstF | OpCode::AddF | OpCode::SubF | OpCode::MulF | OpCode::DivF |
            OpCode::NegF | OpCode::EqF | OpCode::NeF | OpCode::LtF | OpCode::LeF |
            OpCode::GtF | OpCode::GeF | OpCode::CastI2F | OpCode::CastF2I => {
                self.execute_float(op, operand, loc);
            }

            OpCode::PushConstD | OpCode::AddD | OpCode::SubD | OpCode::MulD | OpCode::DivD |
            OpCode::NegD | OpCode::CastI2D | OpCode::CastF2D | OpCode::CastD2I | OpCode::CastD2F |
            OpCode::EqD | OpCode::NeD | OpCode::LtD | OpCode::LeD | OpCode::GtD | OpCode::GeD => {
                self.execute_double(op, operand, loc);
            }

            OpCode::PushConstQ | OpCode::AddQ | OpCode::SubQ | OpCode::MulQ | OpCode::DivQ |
            OpCode::ModQ | OpCode::NegQ | OpCode::CastI2Q | OpCode::CastQ2I | OpCode::CastQ2D |
            OpCode::CastD2Q | OpCode::EqQ | OpCode::NeQ | OpCode::LtQ | OpCode::LeQ |
            OpCode::GtQ | OpCode::GeQ => {
                self.execute_longlong(op, operand, loc);
            }

            OpCode::Jump | OpCode::JumpIfZero | OpCode::JumpIfNotZero |
            OpCode::Call | OpCode::CallPtr | OpCode::CallHost | OpCode::Ret | OpCode::RetVoid => {
                return self.execute_control_flow(op, operand, loc, session);
            }

            OpCode::StepEvent | OpCode::TrapBounds => {
                return self.execute_debug(op, operand, loc);
            }
        }

        if !self.error.is_empty() {
            Some(StepResult::Trap)
        } else {
            None
        }
    }

    // --- Step (execute one instruction) ---

    pub fn step(&mut self, session: &mut Session) -> StepResult {
        if self.finished {
            return StepResult::Finished;
        }
        if !self.error.is_empty() {
            return StepResult::Trap;
        }
        if self.ip >= self.code.len() {
            return StepResult::Finished;
        }

        self.step_count = self.step_count.saturating_add(1);
        if self.step_count % SNAPSHOT_INTERVAL == 0 {
            self.snapshot_vars.clear();
            for sym in &self.symbols {
                if matches!(sym.ty.kind(), crate::compiler::ast::TypeKind::Array) {
                    continue;
                }
                self.snapshot_vars.insert(sym.name.clone(), self.read_variable(sym) as u64);
            }
            self.last_snapshot_step = self.step_count;
        }
        if self.step_count >= self.max_steps {
            let msg = self.format_infinite_loop_error();
            self.trap(&msg, &SourceLoc::default());
            return StepResult::Trap;
        }
        if self.cancelled {
            self.trap("执行已取消。", &SourceLoc::default());
            return StepResult::Trap;
        }

        let inst = self.code[self.ip];
        let ip_before = self.ip;
        self.ip += 1;

        // 记录执行热力图
        if inst.loc.line > 0 {
            session.runtime.heatmap.record(inst.loc.line);
        }

        // 清空上一步的变量访问记录
        self.last_accessed_vars.clear();

        // --- JIT: 热点检测（backward jump 目标计数） ---
        if matches!(inst.op, OpCode::Jump | OpCode::JumpIfZero | OpCode::JumpIfNotZero) {
            let target = inst.operand as usize;
            if target < self.ip {
                *self.ip_hits.entry(target).or_insert(0) += 1;
            }
        }

        // --- JIT: trace 录制触发 ---
        if !self.trace_recorder.is_recording()
            && !self.jit_traces.contains_key(&ip_before)
        {
            if let Some(&hits) = self.ip_hits.get(&ip_before) {
                if hits >= JIT_THRESHOLD {
                    self.trace_recorder.start(ip_before);
                }
            }
        }

        // 执行指令
        if let Some(r) = self.dispatch_single_instruction(inst.op, inst.operand, &inst.loc, session) {
            // trace 录制：遇到中断指令时取消
            if self.trace_recorder.is_recording() {
                self.trace_recorder.reset();
            }
            return r;
        }

        // --- JIT: trace 录制 ---
        if self.trace_recorder.is_recording() {
            match self.trace_recorder.record(inst, ip_before, self.ip) {
                crate::vm::jit_trace::RecordResult::Continue => {}
                crate::vm::jit_trace::RecordResult::Finish | crate::vm::jit_trace::RecordResult::Abort => {
                    if let Some(trace) = self.trace_recorder.finish() {
                        let compiled = crate::vm::jit_templates::compile_trace(&trace);
                        self.jit_traces.insert(trace.start_ip, Arc::new(compiled));
                        self.jit_stats.traces_compiled += 1;
                    }
                }
            }
        }

        if !self.error.is_empty() {
            StepResult::Trap
        } else {
            StepResult::Ok
        }
    }
}
