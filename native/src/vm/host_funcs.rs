use super::host_func_id;
use super::vm::CideVM;
use crate::session::{MemoryRegion, Session};

fn read_cbytes(vm: &CideVM, addr: u32) -> Vec<u8> {
    let mem = vm.get_memory_slice();
    let start = addr as usize;
    if start >= mem.len() {
        return Vec::new();
    }
    mem[start..].iter().take_while(|&&b| b != 0).copied().collect()
}

fn read_cstring(vm: &CideVM, addr: u32) -> String {
    let bytes = read_cbytes(vm, addr);
    String::from_utf8_lossy(&bytes).into_owned()
}

/// 跳过 printf 格式字符串中的修饰符（宽度、精度、长度等），返回真正的格式字母列表。
/// 例如 "%6d" 返回 ['d']，"%.2f" 返回 ['f']，"%%" 不返回任何内容。
fn parse_format_specs(fmt: &str) -> Vec<char> {
    let mut specs = Vec::new();
    let mut chars = fmt.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            if let Some(&next) = chars.peek() {
                if next == '%' {
                    chars.next(); // 跳过 %%
                } else {
                    // 跳过 flags
                    while let Some(&c) = chars.peek() {
                        if c == '-' || c == '+' || c == ' ' || c == '#' || c == '0' {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    // 跳过 width（数字或 *）
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_digit() || c == '*' {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    // 跳过 precision（. 后跟数字或 *）
                    if let Some(&'.') = chars.peek() {
                        chars.next();
                        while let Some(&c) = chars.peek() {
                            if c.is_ascii_digit() || c == '*' {
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                    // 跳过长度修饰符（l, ll, h, hh, L, z, j, t）
                    if let Some(&c) = chars.peek() {
                        if c == 'l' || c == 'h' || c == 'L' || c == 'z' || c == 'j' || c == 't' {
                            chars.next();
                            if let Some(&c2) = chars.peek() {
                                if (c == 'l' && c2 == 'l') || (c == 'h' && c2 == 'h') {
                                    chars.next();
                                }
                            }
                        }
                    }
                    // 真正的格式字母
                    if let Some(&spec) = chars.peek() {
                        specs.push(spec);
                        chars.next();
                    }
                }
            }
        }
    }
    specs
}

/// 根据格式字符串和参数列表生成 printf 输出。
/// 会正确跳过宽度/精度/长度修饰符，只根据格式字母消费参数。
fn format_printf_string(vm: &CideVM, fmt: &str, args: &[i32]) -> String {
    let mut out = String::new();
    let mut used = 0;
    let mut chars = fmt.chars().peekable();
    while let Some(ch) = chars.next() {
        if used < args.len() && ch == '%' {
            if let Some(&next) = chars.peek() {
                if next == '%' {
                    out.push('%');
                    chars.next();
                } else {
                    // 跳过 flags
                    while let Some(&c) = chars.peek() {
                        if c == '-' || c == '+' || c == ' ' || c == '#' || c == '0' {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    // 提取 width（忽略，不用于格式化）
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_digit() || c == '*' {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    // 提取 precision
                    let mut precision: Option<usize> = None;
                    if let Some(&'.') = chars.peek() {
                        chars.next();
                        let mut prec_str = String::new();
                        while let Some(&c) = chars.peek() {
                            if c.is_ascii_digit() {
                                prec_str.push(c);
                                chars.next();
                            } else if c == '*' {
                                chars.next();
                                break;
                            } else {
                                break;
                            }
                        }
                        if !prec_str.is_empty() {
                            precision = prec_str.parse().ok();
                        }
                    }
                    // 跳过长度修饰符
                    if let Some(&c) = chars.peek() {
                        if c == 'l' || c == 'h' || c == 'L' || c == 'z' || c == 'j' || c == 't' {
                            chars.next();
                            if let Some(&c2) = chars.peek() {
                                if (c == 'l' && c2 == 'l') || (c == 'h' && c2 == 'h') {
                                    chars.next();
                                }
                            }
                        }
                    }
                    // 真正的格式字母
                    if let Some(&spec) = chars.peek() {
                        let arg = args[used];
                        match spec {
                            'd' | 'i' => out.push_str(&arg.to_string()),
                            'f' => {
                                let f = f32::from_bits(arg as u32);
                                let prec = precision.unwrap_or(6);
                                out.push_str(&format!("{:.*}", prec, f));
                            }
                            's' => {
                                let s = read_cstring(vm, arg as u32);
                                out.push_str(&s);
                            }
                            'c' => out.push(arg as u8 as char),
                            _ => { out.push(ch); out.push(spec); }
                        }
                        chars.next();
                        used += 1;
                    }
                }
            } else {
                out.push(ch);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

pub fn execute_host_func(vm: &mut CideVM, session: &mut Session, id: u32) {
    match id {
        host_func_id::OUTPUT => host_output(vm, session),
        host_func_id::STEP => host_step(vm, session),
        host_func_id::MALLOC => host_malloc(vm, session),
        host_func_id::FREE => host_free(vm, session),
        host_func_id::PRINTF_0 => host_printf_0(vm, session),
        host_func_id::PRINTF_1 => host_printf_1(vm, session),
        host_func_id::PRINTF_2 => host_printf_2(vm, session),
        host_func_id::PRINTF_N => host_printf_n(vm, session),
        host_func_id::SCANF_N => host_scanf_n(vm, session),
        host_func_id::STRLEN => host_strlen(vm, session),
        host_func_id::STRCPY => host_strcpy(vm, session),
        host_func_id::STRCMP => host_strcmp(vm, session),
        host_func_id::GETCHAR => host_getchar(vm, session),
        host_func_id::PUTCHAR => host_putchar(vm, session),
        host_func_id::RAND => host_rand(vm, session),
        host_func_id::SRAND => host_srand(vm, session),
        host_func_id::MEMSET => host_memset(vm, session),
        host_func_id::EXIT => host_exit(vm, session),
        host_func_id::STRCAT => host_strcat(vm, session),
        host_func_id::ATOI => host_atoi(vm, session),
        host_func_id::FPRINTF => host_fprintf_n(vm, session),
        host_func_id::REALLOC => host_realloc(vm, session),
        host_func_id::QSORT => host_qsort(vm, session),
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
    if size == 0 {
        session.runtime.output_lines.push("[warning] malloc(0) 返回 NULL。在 C 标准中，malloc(0) 的行为是实现定义的，可能返回 NULL 也可能返回一个不可解引用的非空指针。".to_string());
        vm.push(0);
        return;
    }
    if size < 0 {
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
        let new_offset = addr as u64 + aligned_size as u64;
        if new_offset > vm.get_memory_size() as u64 {
            vm.push(0);
            return;
        }
        if new_offset > u32::MAX as u64 {
            vm.push(0);
            return;
        }
        session.memory.heap_offset = new_offset as u32;
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

fn merge_free_list(free_list: &mut Vec<crate::session::FreeBlock>) {
    free_list.sort_by_key(|b| b.addr);
    let mut merged: Vec<crate::session::FreeBlock> = Vec::new();
    for block in free_list.drain(..) {
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
    *free_list = merged;
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
            merge_free_list(&mut session.memory.free_list);
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
                    'f' => { let f = f32::from_bits(arg as u32); out.push_str(&format!("{:.6}", f)); chars.next(); used = true; }
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
                    'f' => { let f = f32::from_bits(arg as u32); out.push_str(&format!("{:.6}", f)); chars.next(); used += 1; }
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
    let specs = parse_format_specs(&fmt);
    let mut args = Vec::with_capacity(specs.len());
    for _ in 0..specs.len() {
        args.push(vm.pop());
    }
    let out = format_printf_string(vm, &fmt, &args);
    session.runtime.output_lines.push(out);
}

fn host_scanf_n(vm: &mut CideVM, session: &mut Session) {
    let fmt_addr = vm.pop() as u32;
    let fmt = read_cstring(vm, fmt_addr);
    // 扫描格式字符串，记录每个 % 格式符的类型（跳过 %% 和修饰符）
    let spec_types = parse_format_specs(&fmt);
    // 按数量 pop 指针参数
    let mut ptrs = Vec::with_capacity(spec_types.len());
    for _ in 0..spec_types.len() {
        ptrs.push(vm.pop() as u32);
    }
    // 读取输入行
    if session.runtime.input_index >= session.runtime.input_lines.len() {
        // 输入不足：将已 pop 的参数重新 push 回栈，等待前端提供输入
        for &p in ptrs.iter().rev() {
            vm.push(p as i32);
        }
        vm.push(fmt_addr as i32);
        session.runtime.waiting_input = true;
        return;
    }
    let line = session.runtime.input_lines[session.runtime.input_index].clone();
    session.runtime.input_index += 1;
    let chars: Vec<char> = line.chars().collect();
    let mut pos = 0usize;
    // 依次解析并写入各指针地址
    for (i, spec) in spec_types.iter().enumerate() {
        let ptr = ptrs[i];
        match spec {
            'd' => {
                // 跳过前导空白
                while pos < chars.len() && chars[pos].is_whitespace() {
                    pos += 1;
                }
                if pos >= chars.len() { break; }
                let start = pos;
                if chars[pos] == '+' || chars[pos] == '-' {
                    pos += 1;
                }
                while pos < chars.len() && chars[pos].is_ascii_digit() {
                    pos += 1;
                }
                let token: String = chars[start..pos].iter().collect();
                let value: i32 = token.parse().unwrap_or(0);
                vm.store_i32(ptr, value, &super::instruction::SourceLoc::default());
            }
            'f' => {
                while pos < chars.len() && chars[pos].is_whitespace() {
                    pos += 1;
                }
                if pos >= chars.len() { break; }
                let start = pos;
                if chars[pos] == '+' || chars[pos] == '-' {
                    pos += 1;
                }
                while pos < chars.len() && (chars[pos].is_ascii_digit() || chars[pos] == '.') {
                    pos += 1;
                }
                let token: String = chars[start..pos].iter().collect();
                let value: f32 = token.parse().unwrap_or(0.0);
                vm.store_i32(ptr, value.to_bits() as i32, &super::instruction::SourceLoc::default());
            }
            'c' => {
                // 标准 C: %c 不跳过空白
                if pos >= chars.len() { break; }
                let ch = chars[pos];
                vm.store_i8(ptr, ch as i32, &super::instruction::SourceLoc::default());
                pos += 1;
            }
            _ => {}
        }
    }
}

fn host_strlen(vm: &mut CideVM, _session: &mut Session) {
    let addr = vm.pop() as u32;
    let bytes = read_cbytes(vm, addr);
    vm.push(bytes.len() as i32);
}

fn host_strcpy(vm: &mut CideVM, _session: &mut Session) {
    let dest = vm.pop() as u32;
    let src = vm.pop() as u32;
    let src_bytes = read_cbytes(vm, src);
    let mem_size = vm.get_memory_slice().len();
    if dest as usize >= mem_size {
        vm.push(0);
        return;
    }
    let max_copy = mem_size - dest as usize;
    let copy_len = src_bytes.len().min(max_copy.saturating_sub(1));
    for (i, &b) in src_bytes.iter().enumerate().take(copy_len) {
        vm.store_i8(dest + i as u32, b as i32, &super::instruction::SourceLoc::default());
    }
    let end = dest as usize + copy_len;
    vm.store_i8(end as u32, 0, &super::instruction::SourceLoc::default());
}

fn host_strcmp(vm: &mut CideVM, _session: &mut Session) {
    let addr1 = vm.pop() as u32;
    let addr2 = vm.pop() as u32;
    let mem = vm.get_memory_slice();
    let mut i = 0usize;
    let result = loop {
        let a = if addr1 as usize + i < mem.len() { mem[addr1 as usize + i] } else { 0 };
        let b = if addr2 as usize + i < mem.len() { mem[addr2 as usize + i] } else { 0 };
        if a != b {
            break (a as i8).wrapping_sub(b as i8) as i32;
        }
        if a == 0 {
            break 0;
        }
        i += 1;
    };
    vm.push(result);
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


fn host_getchar(vm: &mut CideVM, session: &mut Session) {
    // 先检查是否有可用输入
    let has_input = {
        let mut idx = session.runtime.input_index;
        let mut offset = session.runtime.input_char_offset;
        let mut found = false;
        while idx < session.runtime.input_lines.len() {
            if offset < session.runtime.input_lines[idx].len() {
                found = true;
                break;
            }
            idx += 1;
            offset = 0;
        }
        found
    };
    if !has_input {
        session.runtime.waiting_input = true;
        return;
    }
    let mut result = -1i32;
    while session.runtime.input_index < session.runtime.input_lines.len() {
        let line = &session.runtime.input_lines[session.runtime.input_index];
        if session.runtime.input_char_offset < line.len() {
            let ch = line.as_bytes()[session.runtime.input_char_offset];
            session.runtime.input_char_offset += 1;
            result = ch as i32;
            break;
        } else {
            session.runtime.input_index += 1;
            session.runtime.input_char_offset = 0;
        }
    }
    vm.push(result);
}

fn host_putchar(vm: &mut CideVM, session: &mut Session) {
    let val = vm.pop();
    session.runtime.output_lines.push((val as u8 as char).to_string());
}

fn host_rand(vm: &mut CideVM, session: &mut Session) {
    let seed = session.runtime.rand_seed;
    let next = seed.wrapping_mul(1103515245).wrapping_add(12345);
    session.runtime.rand_seed = next;
    vm.push((next & 0x7fff) as i32);
}

fn host_srand(vm: &mut CideVM, session: &mut Session) {
    let seed = vm.pop();
    session.runtime.rand_seed = seed as u32;
}

fn host_memset(vm: &mut CideVM, _session: &mut Session) {
    let ptr = vm.pop() as u32;
    let value = vm.pop();
    let size = vm.pop();
    let mem_size = vm.get_memory_slice().len();
    if ptr as usize >= mem_size {
        vm.push(ptr as i32);
        return;
    }
    let max_write = mem_size - ptr as usize;
    let write_len = (size as usize).min(max_write);
    let byte_val = (value & 0xFF) as u8;
    let mem = vm.memory_ref_mut();
    mem[ptr as usize..ptr as usize + write_len].fill(byte_val);
    vm.push(ptr as i32);
}

fn host_exit(vm: &mut CideVM, _session: &mut Session) {
    let code = vm.pop();
    vm.set_finished(code);
}

fn host_strcat(vm: &mut CideVM, _session: &mut Session) {
    let dest = vm.pop() as u32;
    let src = vm.pop() as u32;
    let mem_size = vm.get_memory_slice().len();
    if dest as usize >= mem_size {
        vm.push(dest as i32);
        return;
    }
    let mut end = dest as usize;
    while end < mem_size && vm.get_memory_slice()[end] != 0 {
        end += 1;
    }
    let max_copy = mem_size.saturating_sub(end).saturating_sub(1);
    let src_bytes = read_cbytes(vm, src);
    let copy_len = src_bytes.len().min(max_copy);
    for (i, &b) in src_bytes.iter().enumerate().take(copy_len) {
        vm.store_i8((end + i) as u32, b as i32, &super::instruction::SourceLoc::default());
    }
    vm.store_i8((end + copy_len) as u32, 0, &super::instruction::SourceLoc::default());
    vm.push(dest as i32);
}

fn host_atoi(vm: &mut CideVM, _session: &mut Session) {
    let addr = vm.pop() as u32;
    let s = read_cstring(vm, addr);
    let val = s.trim_start().parse::<i32>().unwrap_or(0);
    vm.push(val);
}

fn host_fprintf_n(vm: &mut CideVM, session: &mut Session) {
    let _stream = vm.pop();
    let fmt_addr = vm.pop() as u32;
    let fmt = read_cstring(vm, fmt_addr);
    let specs = parse_format_specs(&fmt);
    let mut args = Vec::with_capacity(specs.len());
    for _ in 0..specs.len() {
        args.push(vm.pop());
    }
    let out = format_printf_string(vm, &fmt, &args);
    session.runtime.output_lines.push(out);
}

fn host_realloc(vm: &mut CideVM, session: &mut Session) {
    let ptr = vm.pop() as u32;
    let new_size = vm.pop();

    if new_size <= 0 {
        if ptr != 0 {
            // Equivalent to free
            for r in &mut session.memory.regions {
                if r.addr == ptr && !r.is_freed {
                    r.is_freed = true;
                    let aligned_size = ((r.size as u32) + 3) & !3;
                    session.memory.free_list.push(crate::session::FreeBlock {
                        addr: r.addr,
                        size: aligned_size as i32,
                    });
                    merge_free_list(&mut session.memory.free_list);
                    break;
                }
            }
        }
        vm.push(0);
        return;
    }

    if ptr == 0 {
        // Equivalent to malloc
        host_malloc(vm, session);
        return;
    }

    // Find existing region
    let mut old_region = None;
    for r in &session.memory.regions {
        if r.addr == ptr && !r.is_freed {
            old_region = Some((r.addr, r.size));
            break;
        }
    }

    if old_region.is_none() {
        vm.push(0);
        return;
    }

    let (old_addr, old_size) = old_region.unwrap();
    let aligned_new_size = ((new_size as u32) + 3) & !3;
    let aligned_old_size = ((old_size as u32) + 3) & !3;

    // In-place shrink: old block is at the end of heap
    if aligned_new_size <= aligned_old_size && old_addr + aligned_old_size == session.memory.heap_offset {
        for r in &mut session.memory.regions {
            if r.addr == old_addr && !r.is_freed {
                r.size = new_size;
                break;
            }
        }
        let shrink_by = aligned_old_size - aligned_new_size;
        if shrink_by > 0 {
            session.memory.heap_offset -= shrink_by;
            session.memory.free_list.push(crate::session::FreeBlock {
                addr: session.memory.heap_offset,
                size: shrink_by as i32,
            });
            merge_free_list(&mut session.memory.free_list);
        }
        vm.push(old_addr as i32);
        return;
    }

    // Allocate new memory
    let mut new_addr = 0u32;
    let mut found_idx = None;
    for (i, block) in session.memory.free_list.iter().enumerate() {
        if (block.size as u32) >= aligned_new_size {
            new_addr = block.addr;
            found_idx = Some(i);
            break;
        }
    }

    if let Some(idx) = found_idx {
        let block = &mut session.memory.free_list[idx];
        if (block.size as u32) > aligned_new_size {
            block.addr += aligned_new_size;
            block.size -= aligned_new_size as i32;
        } else {
            session.memory.free_list.remove(idx);
        }
    } else {
        let offset = session.memory.heap_offset;
        let new_offset = (offset as u64) + (aligned_new_size as u64);
        if new_offset > vm.get_memory_size() as u64 {
            vm.push(0);
            return;
        }
        if new_offset > u32::MAX as u64 {
            vm.push(0);
            return;
        }
        new_addr = offset;
        session.memory.heap_offset = new_offset as u32;
    }

    // Copy old data
    let copy_size = (old_size as u32).min(aligned_new_size);
    let copy_buf = {
        let mem = vm.get_memory_slice();
        mem[old_addr as usize..(old_addr + copy_size) as usize].to_vec()
    };
    for i in 0..copy_size {
        vm.store_i8(new_addr + i, copy_buf[i as usize] as i32, &super::instruction::SourceLoc::default());
    }

    // Zero remaining bytes
    for i in copy_size..aligned_new_size {
        vm.store_i8(new_addr + i, 0, &super::instruction::SourceLoc::default());
    }

    // Free old region
    for r in &mut session.memory.regions {
        if r.addr == old_addr && !r.is_freed {
            r.is_freed = true;
            session.memory.free_list.push(crate::session::FreeBlock {
                addr: r.addr,
                size: aligned_old_size as i32,
            });
            merge_free_list(&mut session.memory.free_list);
            break;
        }
    }

    // Track new region
    session.memory.regions.push(MemoryRegion {
        addr: new_addr,
        size: new_size,
        name: String::new(),
        ty: String::new(),
        is_heap: true,
        is_freed: false,
    });

    vm.push(new_addr as i32);
}

const MAX_QSORT_DEPTH: i32 = 8;

fn host_qsort(vm: &mut CideVM, session: &mut Session) {
    let base = vm.pop() as u32;
    let nmemb = vm.pop() as usize;
    let size = vm.pop() as usize;
    let compar = vm.pop() as u32;

    if vm.qsort_depth() >= MAX_QSORT_DEPTH {
        session.runtime.output_lines.push("[qsort] 嵌套深度超过限制，防止栈溢出".to_string());
        return;
    }

    if nmemb <= 1 || size == 0 || base == 0 {
        return;
    }

    let mem_size = vm.get_memory_slice().len();
    if base as usize + nmemb * size > mem_size {
        return;
    }

    vm.set_qsort_depth(vm.qsort_depth() + 1);

    let mut indices: Vec<usize> = (0..nmemb).collect();
    const MAX_COMPARE_STEPS: i32 = 1000;

    if compar == 0 {
        // No comparison function, use default byte comparison
        let mem = vm.get_memory_slice();
        indices.sort_by(|&i, &j| {
            let a_start = (base as usize) + i * size;
            let b_start = (base as usize) + j * size;
            let a = &mem[a_start..a_start + size];
            let b = &mem[b_start..b_start + size];
            a.cmp(b)
        });
    } else {
        indices.sort_by(|&i, &j| {
            let addr_a = (base as i32) + (i as i32) * (size as i32);
            let addr_b = (base as i32) + (j as i32) * (size as i32);
            let result = vm.call_user_function(session, compar, &[addr_a, addr_b], MAX_COMPARE_STEPS);
            match result {
                Some(v) => v.cmp(&0),
                None => std::cmp::Ordering::Equal,
            }
        });
    }

    // Reorder memory according to sorted indices
    let mut temp: Vec<u8> = vec![0; nmemb * size];
    {
        let mem = vm.get_memory_slice();
        for (new_pos, &old_idx) in indices.iter().enumerate() {
            let src_start = (base as usize) + old_idx * size;
            let dst_start = new_pos * size;
            temp[dst_start..dst_start + size].copy_from_slice(&mem[src_start..src_start + size]);
        }
    }
    for i in 0..nmemb {
        let src_start = i * size;
        let dst_start = (base as usize) + i * size;
        for j in 0..size {
            vm.store_i8((dst_start + j) as u32, temp[src_start + j] as i32, &super::instruction::SourceLoc::default());
        }
    }
    vm.set_qsort_depth(vm.qsort_depth() - 1);
}
