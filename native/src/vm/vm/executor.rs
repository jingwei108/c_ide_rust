use super::*;

impl CideVM {
    pub fn run(&mut self, session: &mut Session) -> i32 {
        loop {
            // --- JIT fast path ---
            if let Some(trace) = self.jit_traces.get(&self.ip).cloned() {
                let (result, steps) = execute_trace_bulk(self, session, &trace);
                self.jit_stats.steps_accelerated += steps;
                if let Some(r) = result {
                    match r {
                        StepResult::Finished => {
                            return if self.finished {
                                self.exit_code
                            } else {
                                self.stack.last().copied().unwrap_or(0) as i32
                            };
                        }
                        StepResult::Trap => {
                    self.rollback_pending_array_construction(session);
                    return 0;
                }
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
                    return if self.finished {
                        self.exit_code
                    } else {
                        self.stack.last().copied().unwrap_or(0) as i32
                    };
                }
                StepResult::Trap => {
                    self.rollback_pending_array_construction(session);
                    return 0;
                }
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
            OpCode::PushArgc => {
                self.push(self.argc as u64);
            }
            OpCode::PushArgv => {
                self.push(self.argv_addr as u64);
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
                let var_name = self
                    .local_sym_map
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
                let var_name = self
                    .local_sym_map
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
                let var_name = self
                    .global_sym_map
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
                let var_name = self
                    .global_sym_map
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
            OpCode::StackAlloc => {
                let size = self.pop() as i32;
                if size < 0 {
                    self.trap("StackAlloc: 分配的栈空间大小为负数", loc);
                    return;
                }
                let aligned_size = ((size + 3) & !3) as u32;
                if aligned_size > self.mem_stack_top {
                    self.trap("StackAlloc: 栈溢出", loc);
                    return;
                }
                if self.mem_stack_top - aligned_size < NULL_TRAP_SIZE {
                    self.trap("StackAlloc: 栈溢出（触及保留区）", loc);
                    return;
                }
                self.mem_stack_top -= aligned_size;
                self.push(self.mem_stack_top as u64);
            }
            OpCode::Memcpy => {
                let dest = self.pop() as u32;
                let src = self.pop() as u32;
                let n = self.pop();
                let n_u32 = n as u32;
                // 统一边界检查：NULL 区、上界、UAF。
                if !self.check_mem_access(dest, n_u32, loc, true)
                    || !self.check_mem_access(src, n_u32, loc, false)
                {
                    self.push(dest as u64);
                    return;
                }
                if let Some(log) = self.check_uaf(dest, n_u32) {
                    let msg = self.format_uaf_message(log, true);
                    self.trap(&msg, loc);
                    self.push(dest as u64);
                    return;
                }
                if let Some(log) = self.check_uaf(src, n_u32) {
                    let msg = self.format_uaf_message(log, false);
                    self.trap(&msg, loc);
                    self.push(dest as u64);
                    return;
                }
                let mem_size = self.memory_ref().len();
                let copy_len = (n as usize).min(mem_size - dest as usize).min(mem_size - src as usize);
                if copy_len > 0 {
                    let buf = {
                        let mem = self.memory_ref();
                        mem[src as usize..src as usize + copy_len].to_vec()
                    };
                    let mem = self.memory_ref_mut();
                    for i in 0..copy_len {
                        mem[dest as usize + i] = buf[i];
                    }
                }
                self.push(dest as u64);
            }
            OpCode::Memset => {
                let ptr = self.pop() as u32;
                let value = self.pop();
                let size = self.pop();
                let size_u32 = size as u32;
                // 统一边界检查：NULL 区、上界、UAF；越界时 trap 而非静默截断。
                if !self.check_mem_access(ptr, size_u32, loc, true) {
                    self.push(ptr as u64);
                    return;
                }
                if let Some(log) = self.check_uaf(ptr, size_u32) {
                    let msg = self.format_uaf_message(log, true);
                    self.trap(&msg, loc);
                    self.push(ptr as u64);
                    return;
                }
                let mem_size = self.memory_ref().len();
                let max_write = mem_size - ptr as usize;
                let write_len = (size as usize).min(max_write);
                let byte_val = (value & 0xFF) as u8;
                let mem = self.memory_ref_mut();
                mem[ptr as usize..ptr as usize + write_len].fill(byte_val);
                self.push(ptr as u64);
            }
            OpCode::Strlen => {
                let addr = self.pop() as u32;
                // 统一 NULL 区检查；上界在扫描时自然处理。
                if addr < NULL_TRAP_SIZE {
                    self.trap(
                        &format!("访问了 NULL 指针区域（地址 0x{:04X}）。NULL 指针不能解引用。请确认指针已被正确初始化。", addr),
                        loc,
                    );
                    self.push(0);
                } else {
                    let mem = self.memory_ref();
                    let start = addr as usize;
                    if start >= mem.len() {
                        self.push(0);
                    } else {
                        let len = mem[start..].iter().take_while(|&&b| b != 0).count();
                        self.push(len as u64);
                    }
                }
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
            OpCode::UAdd => {
                let b = self.pop() as i32;
                let a = self.pop() as i32;
                self.push(a.wrapping_add(b) as u64);
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
            OpCode::USub => {
                let b = self.pop() as i32;
                let a = self.pop() as i32;
                self.push(a.wrapping_sub(b) as u64);
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
            OpCode::UMul => {
                let b = self.pop() as i32;
                let a = self.pop() as i32;
                self.push(a.wrapping_mul(b) as u64);
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
            OpCode::UDiv => {
                let b = self.pop() as u32;
                let a = self.pop() as u32;
                if let Some(res) = a.checked_div(b) {
                    self.push(res as u64);
                } else {
                    let msg = self.format_div_zero_error(a as i32, b as i32);
                    self.trap(&msg, loc);
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
            OpCode::UMod => {
                let b = self.pop() as u32;
                let a = self.pop() as u32;
                if b == 0 {
                    let msg = self.format_div_zero_error(a as i32, b as i32);
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
            OpCode::UNeg => {
                let a = self.pop() as u32;
                self.push(a.wrapping_neg() as u64);
            }
            _ => {}
        }
    }
    fn execute_comparison(&mut self, op: OpCode, _operand: i32, _loc: &SourceLoc) {
        match op {
            OpCode::Eq => {
                let b = self.pop() as i32;
                let a = self.pop() as i32;
                self.push(if a == b { 1 } else { 0 });
            }
            OpCode::Ne => {
                let b = self.pop() as i32;
                let a = self.pop() as i32;
                self.push(if a != b { 1 } else { 0 });
            }
            OpCode::Lt => {
                let b = self.pop() as i32;
                let a = self.pop() as i32;
                self.push(if a < b { 1 } else { 0 });
            }
            OpCode::Le => {
                let b = self.pop() as i32;
                let a = self.pop() as i32;
                self.push(if a <= b { 1 } else { 0 });
            }
            OpCode::Gt => {
                let b = self.pop() as i32;
                let a = self.pop() as i32;
                self.push(if a > b { 1 } else { 0 });
            }
            OpCode::Ge => {
                let b = self.pop() as i32;
                let a = self.pop() as i32;
                self.push(if a >= b { 1 } else { 0 });
            }
            OpCode::ULt => {
                let b = self.pop() as u32;
                let a = self.pop() as u32;
                self.push(if a < b { 1 } else { 0 });
            }
            OpCode::ULe => {
                let b = self.pop() as u32;
                let a = self.pop() as u32;
                self.push(if a <= b { 1 } else { 0 });
            }
            OpCode::UGt => {
                let b = self.pop() as u32;
                let a = self.pop() as u32;
                self.push(if a > b { 1 } else { 0 });
            }
            OpCode::UGe => {
                let b = self.pop() as u32;
                let a = self.pop() as u32;
                self.push(if a >= b { 1 } else { 0 });
            }
            OpCode::And => {
                let b = self.pop() as i32;
                let a = self.pop() as i32;
                self.push(if a != 0 && b != 0 { 1 } else { 0 });
            }
            OpCode::Or => {
                let b = self.pop() as i32;
                let a = self.pop() as i32;
                self.push(if a != 0 || b != 0 { 1 } else { 0 });
            }
            OpCode::Not => {
                let a = self.pop() as i32;
                self.push(if a != 0 { 0 } else { 1 });
            }
            _ => {}
        }
    }
    fn execute_bitwise(&mut self, op: OpCode, _operand: i32, loc: &SourceLoc) {
        match op {
            OpCode::BitAnd => {
                let b = self.pop() as i32;
                let a = self.pop() as i32;
                self.push((a & b) as u64);
            }
            OpCode::BitOr => {
                let b = self.pop() as i32;
                let a = self.pop() as i32;
                self.push((a | b) as u64);
            }
            OpCode::BitXor => {
                let b = self.pop() as i32;
                let a = self.pop() as i32;
                self.push((a ^ b) as u64);
            }
            OpCode::BitNot => {
                let a = self.pop() as i32;
                self.push((!a) as u64);
            }
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
            OpCode::LShr => {
                let b = self.pop() as i32;
                let a = self.pop() as u32;
                if !(0..32).contains(&b) {
                    self.trap(&format!("LShr 移位量越界：{}（必须是 0~31）", b), loc);
                } else {
                    self.push((a >> b) as u64);
                }
            }
            _ => {}
        }
    }
    fn execute_float(&mut self, op: OpCode, operand: i32, loc: &SourceLoc) {
        match op {
            OpCode::PushConstF => {
                self.push(operand as u32 as u64);
            }
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
            OpCode::EqF => {
                let b = f32::from_bits(self.pop() as u32);
                let a = f32::from_bits(self.pop() as u32);
                self.push(if (a - b).abs() < EPS_F32 { 1 } else { 0 });
            }
            OpCode::NeF => {
                let b = f32::from_bits(self.pop() as u32);
                let a = f32::from_bits(self.pop() as u32);
                self.push(if (a - b).abs() >= EPS_F32 { 1 } else { 0 });
            }
            OpCode::LtF => {
                let b = f32::from_bits(self.pop() as u32);
                let a = f32::from_bits(self.pop() as u32);
                self.push(if a < b && (a - b).abs() >= EPS_F32 { 1 } else { 0 });
            }
            OpCode::LeF => {
                let b = f32::from_bits(self.pop() as u32);
                let a = f32::from_bits(self.pop() as u32);
                self.push(if a <= b || (a - b).abs() < EPS_F32 { 1 } else { 0 });
            }
            OpCode::GtF => {
                let b = f32::from_bits(self.pop() as u32);
                let a = f32::from_bits(self.pop() as u32);
                self.push(if a > b && (a - b).abs() >= EPS_F32 { 1 } else { 0 });
            }
            OpCode::GeF => {
                let b = f32::from_bits(self.pop() as u32);
                let a = f32::from_bits(self.pop() as u32);
                self.push(if a >= b || (a - b).abs() < EPS_F32 { 1 } else { 0 });
            }
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
            OpCode::EqD => {
                let b = f64::from_bits(self.pop());
                let a = f64::from_bits(self.pop());
                self.push(if (a - b).abs() < EPS_F64 { 1 } else { 0 });
            }
            OpCode::NeD => {
                let b = f64::from_bits(self.pop());
                let a = f64::from_bits(self.pop());
                self.push(if (a - b).abs() >= EPS_F64 { 1 } else { 0 });
            }
            OpCode::LtD => {
                let b = f64::from_bits(self.pop());
                let a = f64::from_bits(self.pop());
                self.push(if a < b && (a - b).abs() >= EPS_F64 { 1 } else { 0 });
            }
            OpCode::LeD => {
                let b = f64::from_bits(self.pop());
                let a = f64::from_bits(self.pop());
                self.push(if a <= b || (a - b).abs() < EPS_F64 { 1 } else { 0 });
            }
            OpCode::GtD => {
                let b = f64::from_bits(self.pop());
                let a = f64::from_bits(self.pop());
                self.push(if a > b && (a - b).abs() >= EPS_F64 { 1 } else { 0 });
            }
            OpCode::GeD => {
                let b = f64::from_bits(self.pop());
                let a = f64::from_bits(self.pop());
                self.push(if a >= b || (a - b).abs() < EPS_F64 { 1 } else { 0 });
            }
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
            OpCode::EqQ => {
                let b = self.pop() as i64;
                let a = self.pop() as i64;
                self.push(if a == b { 1 } else { 0 });
            }
            OpCode::NeQ => {
                let b = self.pop() as i64;
                let a = self.pop() as i64;
                self.push(if a != b { 1 } else { 0 });
            }
            OpCode::LtQ => {
                let b = self.pop() as i64;
                let a = self.pop() as i64;
                self.push(if a < b { 1 } else { 0 });
            }
            OpCode::LeQ => {
                let b = self.pop() as i64;
                let a = self.pop() as i64;
                self.push(if a <= b { 1 } else { 0 });
            }
            OpCode::GtQ => {
                let b = self.pop() as i64;
                let a = self.pop() as i64;
                self.push(if a > b { 1 } else { 0 });
            }
            OpCode::GeQ => {
                let b = self.pop() as i64;
                let a = self.pop() as i64;
                self.push(if a >= b { 1 } else { 0 });
            }
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
        let func_name = if idx < self.func_names.len() {
            self.func_names[idx].clone()
        } else {
            format!("func_{}", func_idx)
        };
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
            self.trap(
                &format!("{}: 栈溢出（栈与堆发生碰撞）。请减少递归深度或动态内存分配。", op_name),
                loc,
            );
            return;
        }
        let original_stack_top = self.mem_stack_top;
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
            self.store_i8(addr, 0, loc);
        }
        self.call_stack.push(CallFrame {
            return_ip: self.ip,
            locals_base,
            local_count: meta.local_count,
            func_name,
            original_stack_top,
            caller_line: self.current_line,
        });
        self.rebuild_local_sym_map();
        self.ip = meta.ip;
    }

    fn execute_control_flow(
        &mut self,
        op: OpCode,
        operand: i32,
        loc: &SourceLoc,
        session: &mut Session,
    ) -> Option<StepResult> {
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
                        self.trap(
                            &format!("JumpIfZero 目标越界：{}（代码长度：{}）", target, self.code.len()),
                            loc,
                        );
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
                        self.trap(
                            &format!("JumpIfNotZero 目标越界：{}（代码长度：{}）", target, self.code.len()),
                            loc,
                        );
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
                    self.mem_stack_top = frame.original_stack_top;
                    self.push(ret_val);
                    self.local_sym_map.clear();
                    return Some(StepResult::Finished);
                }
                self.ip = frame.return_ip;
                self.mem_stack_top = frame.original_stack_top;
                self.current_line = frame.caller_line;
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
                    self.mem_stack_top = frame.original_stack_top;
                    self.local_sym_map.clear();
                    return Some(StepResult::Finished);
                }
                self.ip = frame.return_ip;
                self.mem_stack_top = frame.original_stack_top;
                self.current_line = frame.caller_line;
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
                            index,
                            size.saturating_sub(1)
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

            OpCode::PushConst
            | OpCode::PushArgc
            | OpCode::PushArgv
            | OpCode::Pop
            | OpCode::Dup
            | OpCode::Swap => {
                self.execute_stack(op, operand, loc);
            }

            OpCode::LoadLocal
            | OpCode::StoreLocal
            | OpCode::LoadLocalD
            | OpCode::StoreLocalD
            | OpCode::LoadLocalQ
            | OpCode::StoreLocalQ
            | OpCode::GetFrameBase => {
                self.execute_local(op, operand, loc);
            }

            OpCode::LoadGlobal
            | OpCode::StoreGlobal
            | OpCode::LoadGlobalD
            | OpCode::StoreGlobalD
            | OpCode::LoadGlobalQ
            | OpCode::StoreGlobalQ => {
                self.execute_global(op, operand, loc);
            }

            OpCode::LoadMem
            | OpCode::StoreMem
            | OpCode::LoadMemD
            | OpCode::StoreMemD
            | OpCode::LoadMemByte
            | OpCode::StoreMemByte
            | OpCode::LoadMemQ
            | OpCode::StoreMemQ
            | OpCode::SplitD
            | OpCode::SplitQ
            | OpCode::StackAlloc
            | OpCode::Memcpy
            | OpCode::Memset
            | OpCode::Strlen => {
                self.execute_memory(op, operand, loc);
            }

            OpCode::Add
            | OpCode::Sub
            | OpCode::Mul
            | OpCode::Div
            | OpCode::Mod
            | OpCode::Neg
            | OpCode::UDiv
            | OpCode::UMod
            | OpCode::USub
            | OpCode::UNeg
            | OpCode::UAdd
            | OpCode::UMul => {
                self.execute_arithmetic(op, operand, loc);
            }

            OpCode::Eq
            | OpCode::Ne
            | OpCode::Lt
            | OpCode::Le
            | OpCode::Gt
            | OpCode::Ge
            | OpCode::And
            | OpCode::Or
            | OpCode::Not
            | OpCode::ULt
            | OpCode::ULe
            | OpCode::UGt
            | OpCode::UGe => {
                self.execute_comparison(op, operand, loc);
            }

            OpCode::BitAnd
            | OpCode::BitOr
            | OpCode::BitXor
            | OpCode::BitNot
            | OpCode::Shl
            | OpCode::Shr
            | OpCode::LShr => {
                self.execute_bitwise(op, operand, loc);
            }

            OpCode::PushConstF
            | OpCode::AddF
            | OpCode::SubF
            | OpCode::MulF
            | OpCode::DivF
            | OpCode::NegF
            | OpCode::EqF
            | OpCode::NeF
            | OpCode::LtF
            | OpCode::LeF
            | OpCode::GtF
            | OpCode::GeF
            | OpCode::CastI2F
            | OpCode::CastF2I => {
                self.execute_float(op, operand, loc);
            }

            OpCode::PushConstD
            | OpCode::AddD
            | OpCode::SubD
            | OpCode::MulD
            | OpCode::DivD
            | OpCode::NegD
            | OpCode::CastI2D
            | OpCode::CastF2D
            | OpCode::CastD2I
            | OpCode::CastD2F
            | OpCode::EqD
            | OpCode::NeD
            | OpCode::LtD
            | OpCode::LeD
            | OpCode::GtD
            | OpCode::GeD => {
                self.execute_double(op, operand, loc);
            }

            OpCode::PushConstQ
            | OpCode::AddQ
            | OpCode::SubQ
            | OpCode::MulQ
            | OpCode::DivQ
            | OpCode::ModQ
            | OpCode::NegQ
            | OpCode::CastI2Q
            | OpCode::CastQ2I
            | OpCode::CastQ2D
            | OpCode::CastD2Q
            | OpCode::EqQ
            | OpCode::NeQ
            | OpCode::LtQ
            | OpCode::LeQ
            | OpCode::GtQ
            | OpCode::GeQ => {
                self.execute_longlong(op, operand, loc);
            }

            OpCode::Jump
            | OpCode::JumpIfZero
            | OpCode::JumpIfNotZero
            | OpCode::Call
            | OpCode::CallPtr
            | OpCode::CallHost
            | OpCode::Ret
            | OpCode::RetVoid => {
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
            self.rollback_pending_array_construction(session);
            return StepResult::Trap;
        }
        if self.cancelled {
            self.trap("执行已取消。", &SourceLoc::default());
            self.rollback_pending_array_construction(session);
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
        if !self.trace_recorder.is_recording() && !self.jit_traces.contains_key(&ip_before) {
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
            if matches!(r, StepResult::Trap) {
                self.rollback_pending_array_construction(session);
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
            self.rollback_pending_array_construction(session);
            StepResult::Trap
        } else {
            StepResult::Ok
        }
    }
}

#[cfg(test)]
mod builtin_tests {
    use super::*;
    use crate::session::Session;
    use crate::vm::instruction::{Instruction, SourceLoc};
    use crate::vm::opcode::OpCode;

    fn exec_single(vm: &mut CideVM, session: &mut Session, op: OpCode) {
        vm.set_test_code(vec![Instruction::new(op, 0, SourceLoc { line: 1, column: 1 })]);
        let result = vm.step(session);
        assert!(
            !vm.has_error() || matches!(result, StepResult::Trap),
            "指令意外失败: {}",
            vm.get_error()
        );
    }

    #[test]
    fn test_builtin_strlen() {
        let mut vm = CideVM::new();
        let mut session = Session::default();
        let addr = 0x2000u32;
        let s = b"hello\0";
        vm.memory_ref_mut()[addr as usize..addr as usize + s.len()].copy_from_slice(s);
        vm.push(addr as u64);
        exec_single(&mut vm, &mut session, OpCode::Strlen);
        assert_eq!(vm.get_stack().last().copied().unwrap() as usize, 5);
    }

    #[test]
    fn test_builtin_strlen_empty() {
        let mut vm = CideVM::new();
        let mut session = Session::default();
        let addr = 0x2000u32;
        vm.memory_ref_mut()[addr as usize] = 0;
        vm.push(addr as u64);
        exec_single(&mut vm, &mut session, OpCode::Strlen);
        assert_eq!(vm.get_stack().last().copied().unwrap() as usize, 0);
    }

    #[test]
    fn test_builtin_strlen_out_of_bounds() {
        let mut vm = CideVM::new();
        let mut session = Session::default();
        vm.push((1024 * 1024) as u64); // MEM_SIZE
        exec_single(&mut vm, &mut session, OpCode::Strlen);
        assert_eq!(vm.get_stack().last().copied().unwrap() as usize, 0);
    }

    #[test]
    fn test_builtin_memset() {
        let mut vm = CideVM::new();
        let mut session = Session::default();
        let addr = 0x2000u32;
        // 参数入栈顺序与 codegen 一致（从右到左：size, value, ptr）
        vm.push(5); // size
        vm.push(0x41); // value ('A')
        vm.push(addr as u64); // ptr
        exec_single(&mut vm, &mut session, OpCode::Memset);
        assert_eq!(vm.get_stack().last().copied().unwrap() as u32, addr);
        let mem = vm.memory_ref();
        for i in 0..5 {
            assert_eq!(mem[addr as usize + i], b'A');
        }
    }

    #[test]
    fn test_builtin_memset_null_trap() {
        let mut vm = CideVM::new();
        let mut session = Session::default();
        let addr = 0x100u32; // NULL trap area
        vm.push(5);
        vm.push(0x41);
        vm.push(addr as u64);
        exec_single(&mut vm, &mut session, OpCode::Memset);
        assert!(vm.has_error(), "Memset 应对 NULL 指针区域触发 trap");
    }

    #[test]
    fn test_builtin_memcpy() {
        let mut vm = CideVM::new();
        let mut session = Session::default();
        let src = 0x2000u32;
        let dst = 0x3000u32;
        let data = b"world\0";
        vm.memory_ref_mut()[src as usize..src as usize + data.len()].copy_from_slice(data);
        // 参数入栈顺序与 codegen 一致（从右到左：n, src, dest）
        vm.push(5); // n
        vm.push(src as u64); // src
        vm.push(dst as u64); // dest
        exec_single(&mut vm, &mut session, OpCode::Memcpy);
        assert_eq!(vm.get_stack().last().copied().unwrap() as u32, dst);
        let mem = vm.memory_ref();
        assert_eq!(&mem[dst as usize..dst as usize + 5], b"world");
    }

    #[test]
    fn test_builtin_memcpy_overlap_safety() {
        // Builtin Memcpy 不做重叠处理（与 host_memcpy 一致，未定义行为）。
        // 此处仅验证它能正确复制无重叠区域。
        let mut vm = CideVM::new();
        let mut session = Session::default();
        let src = 0x2000u32;
        let dst = 0x2005u32;
        let data = b"abcde";
        vm.memory_ref_mut()[src as usize..src as usize + data.len()].copy_from_slice(data);
        vm.push(5);
        vm.push(src as u64);
        vm.push(dst as u64);
        exec_single(&mut vm, &mut session, OpCode::Memcpy);
        let mem = vm.memory_ref();
        assert_eq!(&mem[dst as usize..dst as usize + 5], b"abcde");
    }
}
