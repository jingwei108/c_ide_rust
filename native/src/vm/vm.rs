use super::host_funcs::execute_host_func;
use super::instruction::{Instruction, SourceLoc};
use super::opcode::OpCode;
use crate::compiler::ast::Type;
use crate::session::{Session, VisEvent};
use std::collections::{HashMap, HashSet};

pub const MEM_SIZE: u32 = 256 * 1024;
pub const NULL_TRAP_SIZE: u32 = 0x1000;
pub const GLOBAL_START: u32 = 0x1000;
pub const HEAP_START: u32 = 0x5000;
pub const STACK_START: u32 = 0x10000;
pub const SNAPSHOT_INTERVAL: i32 = 100_000;
pub const MAX_STACK_DEPTH: usize = 10_000;

#[derive(Debug, Clone, Default)]
pub struct FuncMeta {
    pub ip: usize,
    pub arg_count: i32,
    pub local_count: i32,
}

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum StepResult {
    Ok,
    Paused,
    Finished,
    Trap,
}

pub struct CideVM {
    code: Vec<Instruction>,
    ip: usize,
    memory: Vec<u8>,
    stack: Vec<i32>,
    mem_stack_top: u32,
    global_count: usize,
    call_stack: Vec<CallFrame>,
    func_table: Vec<FuncMeta>,
    func_names: Vec<String>,
    symbols: Vec<VMSymbol>,
    vis_event_lines: Vec<(i32, i32)>,
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
    snapshot_vars: HashMap<String, i32>,
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
        self.memory.fill(0);
        self.mem_stack_top = STACK_START;
    }

    pub fn load_program(&mut self, code: Vec<Instruction>) {
        self.code = code;
        self.ip = 0;
    }

    pub fn set_globals(&mut self, globals: &[i32]) {
        self.global_count = globals.len();
        for (i, &v) in globals.iter().enumerate() {
            let addr = GLOBAL_START + (i as u32) * 4;
            if addr + 4 <= MEM_SIZE {
                self.write_i32(addr, v);
            }
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
        self.symbols = symbols;
    }

    pub fn set_vis_event_lines(&mut self, lines: Vec<(i32, i32)>) {
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

    pub fn set_max_steps(&mut self, max: i32) {
        self.max_steps = max;
    }

    pub fn has_error(&self) -> bool {
        !self.error.is_empty()
    }

    pub fn get_error(&self) -> &str {
        &self.error
    }

    pub fn get_current_line(&self) -> i32 {
        self.current_line
    }

    pub fn get_executed_steps(&self) -> i32 {
        self.step_count
    }

    pub fn get_memory(&mut self) -> *mut u8 {
        self.memory.as_mut_ptr()
    }

    pub fn memory_ref(&self) -> &[u8] {
        &self.memory
    }

    pub fn get_memory_size(&self) -> u32 {
        MEM_SIZE
    }

    pub fn get_stack(&self) -> &[i32] {
        &self.stack
    }

    pub fn get_symbols(&self) -> &[VMSymbol] {
        &self.symbols
    }

    pub fn get_call_stack(&self) -> &[CallFrame] {
        &self.call_stack
    }

    pub fn was_step_event_hit(&self) -> bool {
        self.step_event_hit
    }

    // --- Stack helpers ---

    pub fn pop(&mut self) -> i32 {
        match self.stack.pop() {
            Some(v) => v,
            None => {
                self.trap("运行时错误：栈下溢", &SourceLoc::default());
                0
            }
        }
    }

    pub fn push(&mut self, val: i32) {
        if self.stack.len() >= MAX_STACK_DEPTH {
            self.trap("值栈溢出：栈深度超过限制。请检查是否有无限递归或过多嵌套表达式。", &SourceLoc::default());
            return;
        }
        self.stack.push(val);
    }

    // --- Memory helpers ---

    pub fn load_i32(&mut self, addr: u32, loc: &SourceLoc) -> i32 {
        if addr < NULL_TRAP_SIZE {
            self.trap(&format!("访问了 NULL 指针区域（地址 0x{:04X}）。NULL 指针不能解引用。请确认指针已被正确初始化。", addr), loc);
            return 0;
        }
        if addr as u64 + 4 > MEM_SIZE as u64 {
            self.trap(&self.format_bounds_error(addr), loc);
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
        if addr < NULL_TRAP_SIZE {
            self.trap(&format!("向 NULL 指针区域写入（地址 0x{:04X}）。请确认指针已被正确初始化。", addr), loc);
            return;
        }
        if addr as u64 + 4 > MEM_SIZE as u64 {
            self.trap(&self.format_bounds_error(addr), loc);
            return;
        }
        let bytes = val.to_le_bytes();
        self.memory[addr as usize..addr as usize + 4].copy_from_slice(&bytes);
    }

    pub fn load_i8(&mut self, addr: u32, loc: &SourceLoc) -> i32 {
        if addr < NULL_TRAP_SIZE {
            self.trap(&format!("访问了 NULL 指针区域（地址 0x{:04X}）", addr), loc);
            return 0;
        }
        if addr as u64 >= MEM_SIZE as u64 {
            self.trap(&self.format_bounds_error(addr), loc);
            return 0;
        }
        self.memory[addr as usize] as i8 as i32
    }

    pub fn store_i8(&mut self, addr: u32, val: i32, loc: &SourceLoc) {
        if addr < NULL_TRAP_SIZE {
            self.trap(&format!("向 NULL 指针区域写入（地址 0x{:04X}）。请确认指针已被正确初始化。", addr), loc);
            return;
        }
        if addr as u64 >= MEM_SIZE as u64 {
            self.trap(&self.format_bounds_error(addr), loc);
            return;
        }
        self.memory[addr as usize] = val as u8;
    }

    fn write_i32(&mut self, addr: u32, val: i32) {
        if addr as u64 + 4 > MEM_SIZE as u64 {
            return;
        }
        let bytes = val.to_le_bytes();
        self.memory[addr as usize..addr as usize + 4].copy_from_slice(&bytes);
    }

    // --- Error formatting ---

    fn format_bounds_error(&self, addr: u32) -> String {
        let mut best_sym: Option<(&VMSymbol, u32, i32)> = None;
        let mut best_dist = i32::MAX;

        for sym in &self.symbols {
            if !matches!(sym.ty.kind, crate::compiler::ast::TypeKind::Array) || sym.ty.array_size <= 0 {
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
            let size = (sym.ty.array_size as u32) * 4;
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
            let index = ((addr as i64 - base as i64) / 4) as i32;
            format!(
                "🚫 数组越界：你访问了 {}[{}]，但数组 '{}' 只有 {} 个元素，有效索引是 0~{}。\n\n📍 发生在第 {} 行\n💡 原因：数组索引超出了合法范围。\n✅ 检查方法：确认索引变量值在 0 到 {} 之间。",
                sym.name, index, sym.name, sym.ty.array_size, sym.ty.array_size - 1,
                self.current_line, sym.ty.array_size - 1
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
            if matches!(sym.ty.kind, crate::compiler::ast::TypeKind::Array) {
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
            if matches!(sym.ty.kind, crate::compiler::ast::TypeKind::Array) {
                continue;
            }
            let cur_val = self.read_variable(sym);
            if let Some(&old_val) = self.snapshot_vars.get(&sym.name) {
                if old_val == cur_val {
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

    fn read_variable(&self, sym: &VMSymbol) -> i32 {
        let mut vaddr = sym.addr;
        if sym.is_local {
            if let Some(frame) = self.call_stack.last() {
                vaddr = frame.locals_base + sym.addr;
            } else {
                return 0;
            }
        }
        if vaddr + 4 > MEM_SIZE || vaddr < NULL_TRAP_SIZE {
            return 0;
        }
        i32::from_le_bytes([
            self.memory[vaddr as usize],
            self.memory[vaddr as usize + 1],
            self.memory[vaddr as usize + 2],
            self.memory[vaddr as usize + 3],
        ])
    }

    pub fn get_variable_snapshot(&self) -> Vec<crate::session::VariableSnapshot> {
        self.symbols.iter().filter_map(|sym| {
            let mut vaddr = sym.addr;
            if sym.is_local {
                if let Some(frame) = self.call_stack.last() {
                    vaddr = frame.locals_base + sym.addr;
                } else {
                    return None;
                }
            }
            if vaddr + 4 > MEM_SIZE || vaddr < NULL_TRAP_SIZE {
                return None;
            }
            let val = i32::from_le_bytes([
                self.memory[vaddr as usize],
                self.memory[vaddr as usize + 1],
                self.memory[vaddr as usize + 2],
                self.memory[vaddr as usize + 3],
            ]);
            Some(crate::session::VariableSnapshot {
                name: sym.name.clone(),
                addr: vaddr,
                is_local: sym.is_local,
                ty: sym.ty.clone(),
                value: val,
            })
        }).collect()
    }

    // --- Run ---

    pub fn run(&mut self, session: &mut Session) -> i32 {
        loop {
            let result = self.step(session);
            match result {
                StepResult::Finished => {
                    return self.stack.last().copied().unwrap_or(0);
                }
                StepResult::Trap => return 0,
                StepResult::Paused => {
                    self.paused = false;
                }
                StepResult::Ok => {}
            }
        }
    }


    // --- Step (execute one instruction) ---

    pub fn step(&mut self, session: &mut Session) -> StepResult {
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
                if matches!(sym.ty.kind, crate::compiler::ast::TypeKind::Array) {
                    continue;
                }
                self.snapshot_vars.insert(sym.name.clone(), self.read_variable(sym));
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
        self.ip += 1;

        match inst.op {
            OpCode::Nop => {}

            OpCode::PushConst => {
                self.push(inst.operand);
            }

            OpCode::LoadLocal => {
                if let Some(frame) = self.call_stack.last() {
                    let addr = (frame.locals_base as u64) + (inst.operand as u64) * 4;
                    if addr + 4 > MEM_SIZE as u64 || addr < NULL_TRAP_SIZE as u64 {
                        self.trap("LoadLocal: 地址越界", &inst.loc);
                    } else {
                        let val = self.load_i32(addr as u32, &inst.loc);
                        self.push(val);
                    }
                } else {
                    self.trap("LoadLocal: 无调用帧", &inst.loc);
                }
            }

            OpCode::StoreLocal => {
                if let Some(frame) = self.call_stack.last() {
                    let addr = (frame.locals_base as u64) + (inst.operand as u64) * 4;
                    if addr + 4 > MEM_SIZE as u64 || addr < NULL_TRAP_SIZE as u64 {
                        self.trap("StoreLocal: 地址越界", &inst.loc);
                    } else {
                        let val = self.pop();
                        self.store_i32(addr as u32, val, &inst.loc);
                    }
                } else {
                    self.trap("StoreLocal: 无调用帧", &inst.loc);
                }
            }

            OpCode::GetFrameBase => {
                if let Some(frame) = self.call_stack.last() {
                    self.push(frame.locals_base as i32);
                } else {
                    self.trap("GetFrameBase: 无调用帧", &inst.loc);
                }
            }

            OpCode::LoadGlobal => {
                let idx = inst.operand as usize;
                if idx >= self.global_count {
                    self.trap("LoadGlobal: 索引越界", &inst.loc);
                } else {
                    let addr = GLOBAL_START + (idx as u32) * 4;
                    let val = self.load_i32(addr, &inst.loc);
                    self.push(val);
                }
            }

            OpCode::StoreGlobal => {
                let idx = inst.operand as usize;
                if idx >= self.global_count {
                    self.trap("StoreGlobal: 索引越界", &inst.loc);
                } else {
                    let addr = GLOBAL_START + (idx as u32) * 4;
                    let val = self.pop();
                    self.store_i32(addr, val, &inst.loc);
                }
            }

            OpCode::Pop => {
                self.pop();
            }

            OpCode::Dup => {
                if let Some(&v) = self.stack.last() {
                    self.push(v);
                } else {
                    self.trap("Dup: 栈空", &inst.loc);
                }
            }

            OpCode::Swap => {
                let len = self.stack.len();
                if len >= 2 {
                    self.stack.swap(len - 1, len - 2);
                } else {
                    self.trap("Swap: 栈不足", &inst.loc);
                }
            }

            OpCode::LoadMem => {
                let addr = self.pop() as u32;
                let val = self.load_i32(addr, &inst.loc);
                self.push(val);
            }

            OpCode::StoreMem => {
                let val = self.pop();
                let addr = self.pop() as u32;
                self.store_i32(addr, val, &inst.loc);
            }

            OpCode::LoadMemByte => {
                let addr = self.pop() as u32;
                let val = self.load_i8(addr, &inst.loc);
                self.push(val);
            }

            OpCode::StoreMemByte => {
                let val = self.pop();
                let addr = self.pop() as u32;
                self.store_i8(addr, val, &inst.loc);
            }

            OpCode::Add => {
                let b = self.pop() as i64;
                let a = self.pop() as i64;
                let r = a + b;
                if r > i32::MAX as i64 || r < i32::MIN as i64 {
                    self.trap("整数加法溢出。两个很大的正数（或很小的负数）相加超出了 int 能表示的范围。", &inst.loc);
                } else {
                    self.push(r as i32);
                }
            }

            OpCode::Sub => {
                let b = self.pop() as i64;
                let a = self.pop() as i64;
                let r = a - b;
                if r > i32::MAX as i64 || r < i32::MIN as i64 {
                    self.trap("整数减法溢出。被减数太小而减数太大，结果超出了 int 能表示的范围。", &inst.loc);
                } else {
                    self.push(r as i32);
                }
            }

            OpCode::Mul => {
                let b = self.pop() as i64;
                let a = self.pop() as i64;
                let r = a * b;
                if r > i32::MAX as i64 || r < i32::MIN as i64 {
                    self.trap("整数乘法溢出。乘积太大，超出了 int 能表示的范围。", &inst.loc);
                } else {
                    self.push(r as i32);
                }
            }

            OpCode::Div => {
                let b = self.pop();
                let a = self.pop();
                if b == 0 {
                    let msg = self.format_div_zero_error(a, b);
                    self.trap(&msg, &inst.loc);
                } else if a == i32::MIN && b == -1 {
                    self.trap("整数除法溢出。INT_MIN / -1 的结果超出了 int 能表示的范围。", &inst.loc);
                } else {
                    self.push(a / b);
                }
            }

            OpCode::Mod => {
                let b = self.pop();
                let a = self.pop();
                if b == 0 {
                    let msg = self.format_div_zero_error(a, b);
                    self.trap(&msg, &inst.loc);
                } else {
                    self.push(a % b);
                }
            }

            OpCode::Neg => {
                let a = self.pop();
                if a == i32::MIN {
                    self.trap("整数取反溢出。-INT_MIN 的结果超出了 int 能表示的范围。", &inst.loc);
                } else {
                    self.push(-a);
                }
            }

            OpCode::Eq => { let b = self.pop(); let a = self.pop(); self.push(if a == b { 1 } else { 0 }); }
            OpCode::Ne => { let b = self.pop(); let a = self.pop(); self.push(if a != b { 1 } else { 0 }); }
            OpCode::Lt => { let b = self.pop(); let a = self.pop(); self.push(if a < b { 1 } else { 0 }); }
            OpCode::Le => { let b = self.pop(); let a = self.pop(); self.push(if a <= b { 1 } else { 0 }); }
            OpCode::Gt => { let b = self.pop(); let a = self.pop(); self.push(if a > b { 1 } else { 0 }); }
            OpCode::Ge => { let b = self.pop(); let a = self.pop(); self.push(if a >= b { 1 } else { 0 }); }

            OpCode::And => { let b = self.pop(); let a = self.pop(); self.push(if a != 0 && b != 0 { 1 } else { 0 }); }
            OpCode::Or  => { let b = self.pop(); let a = self.pop(); self.push(if a != 0 || b != 0 { 1 } else { 0 }); }
            OpCode::Not => { let a = self.pop(); self.push(if a != 0 { 0 } else { 1 }); }
            OpCode::BitAnd => { let b = self.pop(); let a = self.pop(); self.push(a & b); }
            OpCode::BitOr  => { let b = self.pop(); let a = self.pop(); self.push(a | b); }
            OpCode::BitXor => { let b = self.pop(); let a = self.pop(); self.push(a ^ b); }
            OpCode::BitNot => { let a = self.pop(); self.push(!a); }
            OpCode::Shl => {
                let b = self.pop();
                let a = self.pop();
                if !(0..32).contains(&b) {
                    self.trap(&format!("Shl 移位量越界：{}（必须是 0~31）", b), &inst.loc);
                } else {
                    self.push(a << b);
                }
            }
            OpCode::Shr => {
                let b = self.pop();
                let a = self.pop();
                if !(0..32).contains(&b) {
                    self.trap(&format!("Shr 移位量越界：{}（必须是 0~31）", b), &inst.loc);
                } else {
                    self.push(a >> b);
                }
            }

            OpCode::Jump => {
                let target = inst.operand as usize;
                if target >= self.code.len() {
                    self.trap(&format!("Jump 目标越界：{}（代码长度：{}）", target, self.code.len()), &inst.loc);
                } else {
                    self.ip = target;
                }
            }

            OpCode::JumpIfZero => {
                let val = self.pop();
                if val == 0 {
                    let target = inst.operand as usize;
                    if target >= self.code.len() {
                        self.trap(&format!("JumpIfZero 目标越界：{}（代码长度：{}）", target, self.code.len()), &inst.loc);
                    } else {
                        self.ip = target;
                    }
                }
            }

            OpCode::JumpIfNotZero => {
                let val = self.pop();
                if val != 0 {
                    let target = inst.operand as usize;
                    if target >= self.code.len() {
                        self.trap(&format!("JumpIfNotZero 目标越界：{}（代码长度：{}）", target, self.code.len()), &inst.loc);
                    } else {
                        self.ip = target;
                    }
                }
            }

            OpCode::Call => {
                let func_idx = inst.operand as u32;
                let idx = func_idx as usize;
                if idx >= self.func_table.len() || self.func_table[idx].ip == 0 {
                    self.trap(&format!("Call: 未知函数索引 {}", func_idx), &inst.loc);
                } else {
                    let meta = self.func_table[idx].clone();
                    let frame_size = (meta.local_count as u64) * 4;
                    if frame_size > MEM_SIZE as u64 || frame_size > self.mem_stack_top as u64 {
                        self.trap("Call: 栈溢出", &inst.loc);
                    } else {
                        let frame_size_u32 = frame_size as u32;
                        if self.mem_stack_top < NULL_TRAP_SIZE + frame_size_u32 {
                            self.trap("Call: 栈溢出", &inst.loc);
                        } else {
                            let heap_limit = session.memory.heap_offset;
                            if self.mem_stack_top - frame_size_u32 < heap_limit {
                                self.trap("Call: 栈溢出（栈与堆发生碰撞）。请减少递归深度或动态内存分配。", &inst.loc);
                            } else {
                                self.mem_stack_top -= frame_size_u32;
                                let locals_base = self.mem_stack_top;
                                for i in (0..meta.arg_count).rev() {
                                    let arg = self.pop();
                                    let arg_addr = (locals_base as u64) + ((meta.arg_count - 1 - i) as u64) * 4;
                                    self.write_i32(arg_addr as u32, arg);
                                }
                                for i in meta.arg_count..meta.local_count {
                                    let local_addr = (locals_base as u64) + (i as u64) * 4;
                                    self.write_i32(local_addr as u32, 0);
                                }
                                let func_name = if (func_idx as usize) < self.func_names.len() {
                                    self.func_names[func_idx as usize].clone()
                                } else {
                                    format!("func_{}", func_idx)
                                };
                                self.call_stack.push(CallFrame {
                                    return_ip: self.ip,
                                    locals_base,
                                    local_count: meta.local_count,
                                    func_name,
                                });
                                self.ip = meta.ip;
                            }
                        }
                    }
                }
            }

            OpCode::CallHost => {
                execute_host_func(self, session, inst.operand as u32);
            }

            OpCode::Ret => {
                if self.call_stack.is_empty() {
                    return StepResult::Finished;
                }
                let ret_val = self.pop();
                let frame = self.call_stack.pop().unwrap();
                self.ip = frame.return_ip;
                self.mem_stack_top = frame.locals_base;
                self.push(ret_val);
            }

            OpCode::RetVoid => {
                if self.call_stack.is_empty() {
                    return StepResult::Finished;
                }
                let frame = self.call_stack.pop().unwrap();
                self.ip = frame.return_ip;
                self.mem_stack_top = frame.locals_base;
            }

            OpCode::StepEvent => {
                self.current_line = inst.operand;
                if self.breakpoints.contains(&self.current_line) {
                    self.paused = true;
                }
                self.step_event_hit = true;
                for &(line, ty) in &self.vis_event_lines {
                    if line == inst.operand {
                        self.vis_event_queue.push(VisEvent {
                            ty,
                            line: inst.operand,
                            extra: [0, 0, 0],
                        });
                    }
                }
                if self.paused {
                    return StepResult::Paused;
                }
            }

            OpCode::TrapBounds => {
                let sym_idx = inst.operand as usize;
                let mut name = "数组".to_string();
                let mut size = 0;
                let mut index = 0;
                if sym_idx < self.symbols.len() {
                    let sym = &self.symbols[sym_idx];
                    name = sym.name.clone();
                    size = sym.ty.array_size;
                }
                if !self.stack.is_empty() {
                    index = self.pop();
                }
                let diag = format!(
                    "🚫 数组越界：你访问了 {}[{}]，但数组 '{}' 只有 {} 个元素，有效索引是 0~{}。\n\n💡 原因：数组索引超出了合法范围。\n✅ 检查方法：确认索引变量值在 0 到 {} 之间。",
                    name, index, name, size, size.saturating_sub(1), size.saturating_sub(1)
                );
                self.trap(&diag, &inst.loc);
            }
        }

        if !self.error.is_empty() {
            StepResult::Trap
        } else {
            StepResult::Ok
        }
    }
}
