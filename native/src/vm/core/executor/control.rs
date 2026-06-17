use super::*;

impl CideVM {
    pub(crate) fn do_call(&mut self, func_idx: u32, loc: &SourceLoc, session: &mut Session, op_name: &str) {
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

    pub(crate) fn execute_control_flow(
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
                // SAFETY: 上面已检查 call_stack 非空。
                #[allow(clippy::unwrap_used)]
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
                // SAFETY: 上面已检查 call_stack 非空。
                #[allow(clippy::unwrap_used)]
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
}
