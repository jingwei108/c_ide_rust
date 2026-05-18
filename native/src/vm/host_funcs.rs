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
fn format_printf_string(vm: &CideVM, fmt: &str, args: &[u64]) -> String {
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
                    // 跳过长度修饰符，记录是否 ll
                    let mut is_ll = false;
                    if let Some(&c) = chars.peek() {
                        if c == 'l' || c == 'h' || c == 'L' || c == 'z' || c == 'j' || c == 't' {
                            chars.next();
                            if let Some(&c2) = chars.peek() {
                                if c == 'l' && c2 == 'l' {
                                    is_ll = true;
                                    chars.next();
                                } else if c == 'h' && c2 == 'h' {
                                    chars.next();
                                }
                            }
                        }
                    }
                    // 真正的格式字母
                    if let Some(&spec) = chars.peek() {
                        let arg = args[used];
                        match spec {
                            'd' | 'i' => {
                                if is_ll {
                                    out.push_str(&(arg as i64).to_string());
                                } else {
                                    out.push_str(&(arg as i32).to_string());
                                }
                            }
                            'f' => {
                                let f = f64::from_bits(arg);
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
        host_func_id::FOPEN => host_fopen(vm, session),
        host_func_id::FREAD => host_fread(vm, session),
        host_func_id::FWRITE => host_fwrite(vm, session),
        host_func_id::FCLOSE => host_fclose(vm, session),
        host_func_id::FEOF => host_feof(vm, session),
        _ => {}
    }
}

fn host_output(vm: &mut CideVM, session: &mut Session) {
    let val = vm.pop();
    session.runtime.output_lines.push(format!("{}\n", val));
}

fn host_step(vm: &mut CideVM, session: &mut Session) {
    let line = vm.pop() as i32;
    session.runtime.current_line = line;
    session.runtime.trace.push(crate::session::TraceEntry {
        line,
        operation: "step".to_string(),
    });
}

fn host_malloc(vm: &mut CideVM, session: &mut Session) {
    let size = vm.pop() as i32;
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
    let addr = match session.memory.allocate_raw(aligned_size, vm.get_memory_size()) {
        Some(a) => a,
        None => {
            vm.push(0);
            return;
        }
    };
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
    vm.push(addr as u64);
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
            session.memory.merge_free_list();
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
    let out = format_printf_string(vm, &fmt, &[arg]);
    session.runtime.output_lines.push(out);
}

fn host_printf_2(vm: &mut CideVM, session: &mut Session) {
    let fmt_addr = vm.pop() as u32;
    let arg1 = vm.pop();
    let arg2 = vm.pop();
    let fmt = read_cstring(vm, fmt_addr);
    let out = format_printf_string(vm, &fmt, &[arg1, arg2]);
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

/// 解析 scanf 格式字符串，返回每个格式符的类型及长度修饰符级别（0=无, 1=l/h, 2=ll）。
fn parse_scanf_specs(fmt: &str) -> Vec<(char, i32)> {
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
                    // 跳过 width
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_digit() || c == '*' {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    // 跳过 precision
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
                    // 读取长度修饰符
                    let mut len_mod = 0i32;
                    if let Some(&c) = chars.peek() {
                        if c == 'l' {
                            len_mod = 1;
                            chars.next();
                            if let Some(&c2) = chars.peek() {
                                if c2 == 'l' { len_mod = 2; chars.next(); }
                            }
                        } else if c == 'h' {
                            len_mod = 1;
                            chars.next();
                            if let Some(&c2) = chars.peek() {
                                if c2 == 'h' { chars.next(); }
                            }
                        } else if c == 'L' {
                            chars.next();
                        }
                    }
                    // 真正的格式字母
                    if let Some(&spec) = chars.peek() {
                        specs.push((spec, len_mod));
                        chars.next();
                    }
                }
            }
        }
    }
    specs
}

/// 从 chars 中读取一个浮点数字符串，返回 (token_string, new_pos)。
fn read_float_token(chars: &[char], mut pos: usize) -> (String, usize) {
    while pos < chars.len() && chars[pos].is_whitespace() {
        pos += 1;
    }
    if pos >= chars.len() {
        return (String::new(), pos);
    }
    let start = pos;
    if chars[pos] == '+' || chars[pos] == '-' {
        pos += 1;
    }
    while pos < chars.len() && (chars[pos].is_ascii_digit() || chars[pos] == '.') {
        pos += 1;
    }
    let token: String = chars[start..pos].iter().collect();
    (token, pos)
}

fn host_scanf_n(vm: &mut CideVM, session: &mut Session) {
    let fmt_addr = vm.pop() as u32;
    let fmt = read_cstring(vm, fmt_addr);
    // 扫描格式字符串，记录每个 % 格式符的类型及是否带 long 修饰符
    let spec_types = parse_scanf_specs(&fmt);
    // 按数量 pop 指针参数
    let mut ptrs = Vec::with_capacity(spec_types.len());
    for _ in 0..spec_types.len() {
        ptrs.push(vm.pop() as u32);
    }
    // 读取输入行
    if session.runtime.input_index >= session.runtime.input_lines.len() {
        // 输入不足：将已 pop 的参数重新 push 回栈，等待前端提供输入
        for &p in ptrs.iter().rev() {
            vm.push(p as u64);
        }
        vm.push(fmt_addr as u64);
        session.runtime.waiting_input = true;
        return;
    }
    let line = session.runtime.input_lines[session.runtime.input_index].clone();
    session.runtime.input_index += 1;
    let chars: Vec<char> = line.chars().collect();
    let mut pos = 0usize;
    // 依次解析并写入各指针地址
    for (i, (spec, len_mod)) in spec_types.iter().enumerate() {
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
                if *len_mod >= 2 {
                    // %lld → long long (8 bytes)
                    let value: i64 = token.parse().unwrap_or(0);
                    vm.store_i64(ptr, value as u64, &super::instruction::SourceLoc::default());
                } else {
                    let value: i32 = token.parse().unwrap_or(0);
                    vm.store_i32(ptr, value, &super::instruction::SourceLoc::default());
                }
            }
            'f' => {
                let (token, new_pos) = read_float_token(&chars, pos);
                pos = new_pos;
                if token.is_empty() { break; }
                if *len_mod >= 1 {
                    // %lf → double (8 bytes)
                    let value: f64 = token.parse().unwrap_or(0.0);
                    vm.store_i64(ptr, value.to_bits(), &super::instruction::SourceLoc::default());
                } else {
                    // %f → float (4 bytes)
                    let value: f32 = token.parse().unwrap_or(0.0);
                    vm.store_i32(ptr, value.to_bits() as i32, &super::instruction::SourceLoc::default());
                }
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
    vm.push(bytes.len() as u64);
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
    vm.push(result as u64);
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
    vm.push(result as u64);
}

fn host_putchar(vm: &mut CideVM, session: &mut Session) {
    let val = vm.pop();
    session.runtime.output_lines.push((val as u8 as char).to_string());
}

fn host_rand(vm: &mut CideVM, session: &mut Session) {
    let seed = session.runtime.rand_seed;
    let next = seed.wrapping_mul(1103515245).wrapping_add(12345);
    session.runtime.rand_seed = next;
    vm.push((next & 0x7fff) as u64);
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
        vm.push(ptr as u64);
        return;
    }
    let max_write = mem_size - ptr as usize;
    let write_len = (size as usize).min(max_write);
    let byte_val = (value & 0xFF) as u8;
    let mem = vm.memory_ref_mut();
    mem[ptr as usize..ptr as usize + write_len].fill(byte_val);
    vm.push(ptr as u64);
}

fn host_exit(vm: &mut CideVM, _session: &mut Session) {
    let code = vm.pop() as i32;
    vm.set_finished(code);
}

fn host_strcat(vm: &mut CideVM, _session: &mut Session) {
    let dest = vm.pop() as u32;
    let src = vm.pop() as u32;
    let mem_size = vm.get_memory_slice().len();
    if dest as usize >= mem_size {
        vm.push(dest as u64);
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
    vm.push(dest as u64);
}

fn host_atoi(vm: &mut CideVM, _session: &mut Session) {
    let addr = vm.pop() as u32;
    let s = read_cstring(vm, addr);
    let val = s.trim_start().parse::<i32>().unwrap_or(0);
    vm.push(val as u64);
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
    let new_size = vm.pop() as i32;

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
                    session.memory.merge_free_list();
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
            session.memory.merge_free_list();
        }
        vm.push(old_addr as u64);
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
            session.memory.merge_free_list();
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

    vm.push(new_addr as u64);
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

// ========== VFS File I/O Host Functions ==========

fn host_fopen(vm: &mut CideVM, session: &mut Session) {
    // 参数从右到左压栈：path, mode → 栈顶是 path
    let path_addr = vm.pop() as u32;
    let mode_addr = vm.pop() as u32;
    let path = read_cstring(vm, path_addr);
    let mode = read_cstring(vm, mode_addr);
    let mut vfs = std::mem::take(&mut session.vfs);
    let fd = vfs.fopen(&path, &mode, vm, &mut session.memory);
    session.vfs = vfs;
    // 在 VM Heap 中分配 4 字节存储 fd，返回 FILE*
    if fd != 0 {
        let mut file_ptr = 0u32;
        let aligned = 4u32;
        let addr = session.memory.heap_offset;
        let new_offset = addr as u64 + aligned as u64;
        if new_offset <= vm.get_memory_size() as u64 && new_offset <= u32::MAX as u64 {
            session.memory.heap_offset = new_offset as u32;
            let mut reused = false;
            for r in &mut session.memory.regions {
                if r.addr == addr && r.is_freed {
                    r.is_freed = false;
                    r.size = 4;
                    r.name = format!("FILE:{}", path);
                    reused = true;
                    break;
                }
            }
            if !reused {
                session.memory.alloc_counter += 1;
                session.memory.regions.push(MemoryRegion {
                    addr,
                    size: 4,
                    name: format!("FILE:{}", path),
                    ty: "int".to_string(),
                    is_heap: true,
                    is_freed: false,
                });
            }
            // 写入 fd 到 FILE* 结构体
            let mem = vm.memory_ref_mut();
            let a = addr as usize;
            mem[a..a + 4].copy_from_slice(&(fd as i32).to_le_bytes());
            file_ptr = addr;
        }
        vm.push(file_ptr as u64);
    } else {
        vm.push(0);
    }
}

fn host_fread(vm: &mut CideVM, session: &mut Session) {
    // 参数从右到左压栈：buf, size, nmemb, stream → 栈顶是 buf
    let buf = vm.pop() as u32;
    let size = vm.pop() as usize;
    let nmemb = vm.pop() as usize;
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let mut vfs = std::mem::take(&mut session.vfs);
    let n = vfs.fread(fd, buf, size, nmemb, vm);
    session.vfs = vfs;
    vm.push(n as u64);
}

fn host_fwrite(vm: &mut CideVM, session: &mut Session) {
    // 参数从右到左压栈：buf, size, nmemb, stream → 栈顶是 buf
    let buf = vm.pop() as u32;
    let size = vm.pop() as usize;
    let nmemb = vm.pop() as usize;
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let mut vfs = std::mem::take(&mut session.vfs);
    let n = vfs.fwrite(fd, buf, size, nmemb, vm, &mut session.memory);
    session.vfs = vfs;
    vm.push(n as u64);
}

fn host_fclose(vm: &mut CideVM, session: &mut Session) {
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let mut vfs = std::mem::take(&mut session.vfs);
    let ret = vfs.fclose(fd, &mut session.memory);
    session.vfs = vfs;
    vm.push(ret as u64);
}

fn host_feof(vm: &mut CideVM, session: &mut Session) {
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let ret = session.vfs.feof(fd);
    vm.push(ret as u64);
}

/// 从 FILE* 指针（VM Heap 地址）读取 fd 索引
fn read_fd_from_stream(vm: &CideVM, stream: u32) -> u32 {
    if stream == 0 {
        return 0;
    }
    let mem = vm.get_memory_slice();
    let start = stream as usize;
    if start + 4 > mem.len() {
        return 0;
    }
    i32::from_le_bytes([mem[start], mem[start + 1], mem[start + 2], mem[start + 3]]) as u32
}
