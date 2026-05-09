use super::vm::CideVM;
use crate::session::{MemoryRegion, Session};

fn read_cstring(vm: &CideVM, addr: u32) -> String {
    let mem = &vm.get_memory_slice();
    let mut s = String::new();
    let start = addr as usize;
    if start >= mem.len() {
        return s;
    }
    for i in start..mem.len() {
        if mem[i] == 0 {
            break;
        }
        s.push(mem[i] as char);
    }
    s
}

pub fn execute_host_func(vm: &mut CideVM, session: &mut Session, id: u32) {
    match id {
        0 => host_output(vm, session),
        1 => host_step(vm, session),
        2 => host_malloc(vm, session),
        3 => host_free(vm, session),
        10 => host_printf_0(vm, session),
        11 => host_printf_1(vm, session),
        12 => host_printf_2(vm, session),
        15 => host_printf_n(vm, session),
        20 => host_scanf_1(vm, session),
        _ => {}
    }
}

fn host_output(vm: &mut CideVM, session: &mut Session) {
    let val = vm.pop();
    session.runtime.output_lines.push(format!("{}\n", val));
}

fn host_step(vm: &mut CideVM, session: &mut Session) {
    let line = vm.pop();
    session.runtime.current_line = line;
    session.runtime.trace.push(crate::session::TraceEntry {
        line,
        operation: "step".to_string(),
    });
}

fn host_malloc(vm: &mut CideVM, session: &mut Session) {
    let size = vm.pop();
    if size <= 0 {
        vm.push(0);
        return;
    }
    let aligned_size = ((size as u32) + 3) & !3;
    let mut addr = 0u32;
    // first-fit from free list
    let mut found_idx = None;
    for (i, block) in session.memory.free_list.iter().enumerate() {
        if (block.size as u32) >= aligned_size {
            addr = block.addr;
            found_idx = Some(i);
            break;
        }
    }
    if let Some(idx) = found_idx {
        let block = &mut session.memory.free_list[idx];
        if (block.size as u32) > aligned_size {
            block.addr += aligned_size;
            block.size -= aligned_size as i32;
        } else {
            session.memory.free_list.remove(idx);
        }
    } else {
        addr = session.memory.heap_offset;
        let new_offset = addr + aligned_size;
        if new_offset > vm.get_memory_size() {
            vm.push(0);
            return;
        }
        session.memory.heap_offset = new_offset;
    }
    // reuse or add region
    let mut reused = false;
    for r in &mut session.memory.regions {
        if r.addr == addr && r.is_freed {
            r.is_freed = false;
            r.size = size;
            reused = true;
            break;
        }
    }
    if !reused {
        session.memory.alloc_counter += 1;
        session.memory.regions.push(MemoryRegion {
            addr,
            size,
            name: format!("heap_{}", session.memory.alloc_counter),
            ty: "int".to_string(),
            is_heap: true,
            is_freed: false,
        });
    }
    vm.push(addr as i32);
}

fn host_free(vm: &mut CideVM, session: &mut Session) {
    let addr = vm.pop() as u32;
    for r in &mut session.memory.regions {
        if r.addr == addr && !r.is_freed {
            r.is_freed = true;
            let aligned_size = ((r.size as u32) + 3) & !3;
            session.memory.free_list.push(crate::session::FreeBlock {
                addr: r.addr,
                size: aligned_size as i32,
            });
            session.memory.free_list.sort_by_key(|b| b.addr);
            let mut merged: Vec<crate::session::FreeBlock> = Vec::new();
            for block in session.memory.free_list.drain(..) {
                if let Some(last) = merged.last_mut() {
                    if (last.addr as u64) + (last.size as u64) == (block.addr as u64) {
                        last.size += block.size;
                    } else {
                        merged.push(block);
                    }
                } else {
                    merged.push(block);
                }
            }
            session.memory.free_list = merged;
            break;
        }
    }
}

fn host_printf_0(vm: &mut CideVM, session: &mut Session) {
    let fmt_addr = vm.pop() as u32;
    let out = read_cstring(vm, fmt_addr);
    session.runtime.output_lines.push(out);
}

fn host_printf_1(vm: &mut CideVM, session: &mut Session) {
    let fmt_addr = vm.pop() as u32;
    let arg = vm.pop();
    let fmt = read_cstring(vm, fmt_addr);
    let mut out = String::new();
    let mut used = false;
    let mut chars = fmt.chars().peekable();
    while let Some(ch) = chars.next() {
        if !used && ch == '%' {
            if let Some(&next) = chars.peek() {
                match next {
                    'd' => { out.push_str(&arg.to_string()); chars.next(); used = true; }
                    's' => {
                        let s = read_cstring(vm, arg as u32);
                        out.push_str(&s);
                        chars.next(); used = true;
                    }
                    'c' => { out.push(arg as u8 as char); chars.next(); used = true; }
                    '%' => { out.push('%'); chars.next(); }
                    _ => { out.push(ch); }
                }
            } else {
                out.push(ch);
            }
        } else {
            out.push(ch);
        }
    }
    session.runtime.output_lines.push(out);
}

fn host_printf_2(vm: &mut CideVM, session: &mut Session) {
    let fmt_addr = vm.pop() as u32;
    let arg1 = vm.pop();
    let arg2 = vm.pop();
    let fmt = read_cstring(vm, fmt_addr);
    let mut out = String::new();
    let mut used = 0;
    let mut chars = fmt.chars().peekable();
    while let Some(ch) = chars.next() {
        if used < 2 && ch == '%' {
            if let Some(&next) = chars.peek() {
                let arg = if used == 0 { arg1 } else { arg2 };
                match next {
                    'd' => { out.push_str(&arg.to_string()); chars.next(); used += 1; }
                    's' => {
                        let s = read_cstring(vm, arg as u32);
                        out.push_str(&s);
                        chars.next(); used += 1;
                    }
                    'c' => { out.push(arg as u8 as char); chars.next(); used += 1; }
                    '%' => { out.push('%'); chars.next(); }
                    _ => { out.push(ch); }
                }
            } else {
                out.push(ch);
            }
        } else {
            out.push(ch);
        }
    }
    session.runtime.output_lines.push(out);
}

fn host_printf_n(vm: &mut CideVM, session: &mut Session) {
    let fmt_addr = vm.pop() as u32;
    let fmt = read_cstring(vm, fmt_addr);
    // count specifiers excluding %%
    let mut spec_count = 0;
    let mut chars = fmt.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            if let Some(&next) = chars.peek() {
                if next == '%' {
                    chars.next();
                } else {
                    spec_count += 1;
                    chars.next();
                }
            }
        }
    }
    let mut args = Vec::with_capacity(spec_count);
    for _ in 0..spec_count {
        args.push(vm.pop());
    }
    let mut out = String::new();
    let mut used = 0;
    let mut chars = fmt.chars().peekable();
    while let Some(ch) = chars.next() {
        if used < spec_count && ch == '%' {
            if let Some(&next) = chars.peek() {
                if next == '%' {
                    out.push('%');
                    chars.next();
                } else {
                    let arg = args[used];
                    match next {
                        'd' => out.push_str(&arg.to_string()),
                        's' => {
                            let s = read_cstring(vm, arg as u32);
                            out.push_str(&s);
                        }
                        'c' => out.push(arg as u8 as char),
                        _ => { out.push(ch); out.push(next); }
                    }
                    chars.next();
                    used += 1;
                }
            } else {
                out.push(ch);
            }
        } else {
            out.push(ch);
        }
    }
    session.runtime.output_lines.push(out);
}

fn host_scanf_1(vm: &mut CideVM, session: &mut Session) {
    let fmt_addr = vm.pop() as u32;
    let p1 = vm.pop() as u32;
    let mut fmt_spec = 'd';
    let mem = vm.get_memory_slice();
    let mut i = fmt_addr as usize;
    while i < mem.len() {
        if mem[i] == 0 { break; }
        if mem[i] as char == '%' && i + 1 < mem.len() && mem[i + 1] != 0 {
            fmt_spec = mem[i + 1] as char;
            break;
        }
        i += 1;
    }
    if session.runtime.input_index >= session.runtime.input_lines.len() {
        return;
    }
    let line = session.runtime.input_lines[session.runtime.input_index].clone();
    session.runtime.input_index += 1;
    if fmt_spec == 'c' {
        let ch = line.chars().next().unwrap_or('\0');
        vm.store_i8(p1, ch as i32, &super::instruction::SourceLoc::default());
    } else {
        let value: i32 = line.trim().parse().unwrap_or(0);
        vm.store_i32(p1, value, &super::instruction::SourceLoc::default());
    }
}

// Helper trait to get memory slice safely
pub trait MemorySlice {
    fn get_memory_slice(&self) -> &[u8];
}

impl MemorySlice for CideVM {
    fn get_memory_slice(&self) -> &[u8] {
        self.memory_ref()
    }
}
