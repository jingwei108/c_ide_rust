use super::*;
use crate::VmContext;

pub fn host_rand(vm: &mut CideVM, session: &mut VmContext<'_>) {
    let seed = session.runtime.rand_seed;
    let next = seed.wrapping_mul(1103515245).wrapping_add(12345);
    session.runtime.rand_seed = next;
    vm.push((next & 0x7fff) as u64);
}

pub fn host_srand(vm: &mut CideVM, session: &mut VmContext<'_>) {
    let seed = vm.pop();
    session.runtime.rand_seed = seed as u32;
}

pub fn host_exit(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let code = vm.pop() as i32;
    vm.set_finished(code);
}

pub fn host_isdigit(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let c = vm.pop() as i32;
    vm.push(if c >= '0' as i32 && c <= '9' as i32 { 1 } else { 0 });
}

pub fn host_isalpha(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let c = vm.pop() as i32;
    vm.push(
        if (c >= 'a' as i32 && c <= 'z' as i32) || (c >= 'A' as i32 && c <= 'Z' as i32) {
            1
        } else {
            0
        },
    );
}

pub fn host_islower(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let c = vm.pop() as i32;
    vm.push(if c >= 'a' as i32 && c <= 'z' as i32 { 1 } else { 0 });
}

pub fn host_isupper(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let c = vm.pop() as i32;
    vm.push(if c >= 'A' as i32 && c <= 'Z' as i32 { 1 } else { 0 });
}

pub fn host_tolower(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let c = vm.pop() as i32;
    vm.push(if c >= 'A' as i32 && c <= 'Z' as i32 {
        c + ('a' as i32 - 'A' as i32)
    } else {
        c
    } as u64);
}

pub fn host_toupper(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let c = vm.pop() as i32;
    vm.push(if c >= 'a' as i32 && c <= 'z' as i32 {
        c + ('A' as i32 - 'a' as i32)
    } else {
        c
    } as u64);
}

pub fn host_isspace(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let c = vm.pop() as i32;
    vm.push(
        if c == ' ' as i32
            || c == '\t' as i32
            || c == '\n' as i32
            || c == '\r' as i32
            || c == '\x0C' as i32
            || c == '\x0B' as i32
        {
            1
        } else {
            0
        },
    );
}

pub fn host_isalnum(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let c = vm.pop() as i32;
    let alpha = (c >= 'a' as i32 && c <= 'z' as i32) || (c >= 'A' as i32 && c <= 'Z' as i32);
    let digit = c >= '0' as i32 && c <= '9' as i32;
    vm.push(if alpha || digit { 1 } else { 0 });
}

pub fn host_isprint(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let c = vm.pop() as i32;
    vm.push(if c >= ' ' as i32 && c <= '~' as i32 { 1 } else { 0 });
}

pub fn host_iscntrl(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let c = vm.pop() as i32;
    vm.push(if (0..=31).contains(&c) || c == 127 { 1 } else { 0 });
}

pub fn host_isxdigit(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let c = vm.pop() as i32;
    let digit = c >= '0' as i32 && c <= '9' as i32;
    let lower = c >= 'a' as i32 && c <= 'f' as i32;
    let upper = c >= 'A' as i32 && c <= 'F' as i32;
    vm.push(if digit || lower || upper { 1 } else { 0 });
}

pub fn host_isgraph(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let c = vm.pop() as i32;
    vm.push(if c > ' ' as i32 && c <= '~' as i32 { 1 } else { 0 });
}

pub fn host_ispunct(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let c = vm.pop() as i32;
    let is_print = c >= ' ' as i32 && c <= '~' as i32;
    let is_alnum = (c >= 'a' as i32 && c <= 'z' as i32)
        || (c >= 'A' as i32 && c <= 'Z' as i32)
        || (c >= '0' as i32 && c <= '9' as i32);
    vm.push(if is_print && !is_alnum && c != ' ' as i32 { 1 } else { 0 });
}

pub fn host_isblank(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let c = vm.pop() as i32;
    vm.push(if c == ' ' as i32 || c == '\t' as i32 { 1 } else { 0 });
}

pub fn host_atoi(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let addr = vm.pop() as u32;
    let s = read_cstring(vm, addr);
    let trimmed = s.trim_start();
    let mut chars = trimmed.chars().peekable();
    let mut sign = 1i32;
    if let Some(&c) = chars.peek() {
        if c == '-' {
            sign = -1;
            chars.next();
        } else if c == '+' {
            chars.next();
        }
    }
    let mut val: i32 = 0;
    for c in chars {
        if c.is_ascii_digit() {
            val = val.wrapping_mul(10).wrapping_add(c as i32 - '0' as i32);
        } else {
            break;
        }
    }
    vm.push((sign.wrapping_mul(val)) as u64);
}

const MAX_QSORT_DEPTH: i32 = 8;

pub fn host_qsort(vm: &mut CideVM, session: &mut VmContext<'_>) {
    let base = vm.pop() as u32;
    let nmemb = vm.pop() as usize;
    let size = vm.pop() as usize;
    let compar = vm.pop() as u32;

    if vm.qsort_depth() >= MAX_QSORT_DEPTH {
        session
            .runtime
            .output_lines
            .push("[qsort] 嵌套深度超过限制，防止栈溢出".to_string());
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

    // Reorder memory according to sorted indices.
    // Use a temporary buffer to avoid overwriting data during in-place reordering,
    // then write back in chunks via `write_memory` instead of byte-by-byte store_i8.
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
        if !vm.write_memory(dst_start as u32, &temp[src_start..src_start + size]) {
            vm.trap("qsort: write back out of bounds", &SourceLoc::default());
            vm.set_qsort_depth(vm.qsort_depth() - 1);
            return;
        }
    }
    vm.set_qsort_depth(vm.qsort_depth() - 1);
}

// ========== VFS File I/O Host Functions ==========

const MAX_BSEARCH_DEPTH: i32 = 8;

pub fn host_bsearch(vm: &mut CideVM, session: &mut VmContext<'_>) {
    let key = vm.pop() as u32;
    let base = vm.pop() as u32;
    let nmemb = vm.pop() as usize;
    let size = vm.pop() as usize;
    let compar = vm.pop() as u32;

    if vm.qsort_depth() >= MAX_BSEARCH_DEPTH {
        session
            .runtime
            .output_lines
            .push("[bsearch] 嵌套深度超过限制，防止栈溢出".to_string());
        vm.push(0);
        return;
    }

    if nmemb == 0 || size == 0 || base == 0 || key == 0 {
        vm.push(0);
        return;
    }

    let mem_size = vm.get_memory_slice().len();
    if base as usize + nmemb * size > mem_size {
        vm.push(0);
        return;
    }

    vm.set_qsort_depth(vm.qsort_depth() + 1);

    const MAX_COMPARE_STEPS: i32 = 1000;
    let mut lo: usize = 0;
    let mut hi: usize = nmemb;
    let mut result_ptr: u32 = 0;

    if compar == 0 {
        let mem = vm.get_memory_slice();
        while lo < hi {
            let mid = (lo + hi) / 2;
            let a_start = key as usize;
            let b_start = (base as usize) + mid * size;
            let a = &mem[a_start..a_start + size];
            let b = &mem[b_start..b_start + size];
            match a.cmp(b) {
                std::cmp::Ordering::Less => {
                    lo = mid + 1;
                }
                std::cmp::Ordering::Greater => {
                    hi = mid;
                }
                std::cmp::Ordering::Equal => {
                    result_ptr = b_start as u32;
                    break;
                }
            }
        }
    } else {
        while lo < hi {
            let mid = (lo + hi) / 2;
            let addr_key = key as i32;
            let addr_mid = (base as i32) + (mid as i32) * (size as i32);
            let cmp_result = vm.call_user_function(session, compar, &[addr_key, addr_mid], MAX_COMPARE_STEPS);
            match cmp_result {
                Some(v) if v < 0 => {
                    hi = mid;
                }
                Some(v) if v > 0 => {
                    lo = mid + 1;
                }
                Some(_) => {
                    result_ptr = addr_mid as u32;
                    break;
                }
                None => {
                    break;
                }
            }
        }
    }

    vm.set_qsort_depth(vm.qsort_depth() - 1);
    vm.push(result_ptr as u64);
}

pub fn host_atof(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let addr = vm.pop() as u32;
    let s = read_cstring(vm, addr);
    let val: f64 = s.trim().parse().unwrap_or(0.0);
    vm.push(val.to_bits());
}

pub fn host_atol(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let addr = vm.pop() as u32;
    let s = read_cstring(vm, addr);
    let trimmed = s.trim_start();
    let mut chars = trimmed.chars().peekable();
    let mut sign = 1i64;
    if let Some(&c) = chars.peek() {
        if c == '-' {
            sign = -1;
            chars.next();
        } else if c == '+' {
            chars.next();
        }
    }
    let mut val: i64 = 0;
    for c in chars {
        if c.is_ascii_digit() {
            val = val.wrapping_mul(10).wrapping_add(c as i64 - '0' as i64);
        } else {
            break;
        }
    }
    vm.push((sign.wrapping_mul(val)) as u64);
}

// ========== Math extensions ==========

pub fn host_abort(vm: &mut CideVM, session: &mut VmContext<'_>) {
    session.runtime.output_lines.push("[abort] 程序异常终止 (SIGABRT)".to_string());
    vm.set_finished(134);
}

pub fn host_strtol(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let str_addr = vm.pop() as u32;
    let endptr_addr = vm.pop() as u32;
    let base = vm.pop() as i32;
    let s = read_cstring(vm, str_addr);
    let bytes = s.as_bytes();
    let mut pos = 0usize;
    while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }
    let mut sign = 1i64;
    if pos < bytes.len() {
        if bytes[pos] == b'-' {
            sign = -1;
            pos += 1;
        } else if bytes[pos] == b'+' {
            pos += 1;
        }
    }
    let start_pos = pos;
    let mut val: i64 = 0;
    let effective_base = if base == 0 { 10 } else { base };
    while pos < bytes.len() {
        let c = bytes[pos];
        let digit = if c.is_ascii_digit() {
            (c - b'0') as i64
        } else if c.is_ascii_lowercase() {
            (c - b'a' + 10) as i64
        } else if c.is_ascii_uppercase() {
            (c - b'A' + 10) as i64
        } else {
            break;
        };
        if digit >= effective_base as i64 {
            break;
        }
        val = val.wrapping_mul(effective_base as i64).wrapping_add(digit);
        pos += 1;
    }
    if pos == start_pos {
        set_errno(vm, 1); // EINVAL
    }
    if endptr_addr != 0 {
        let stop_addr = str_addr + pos as u32;
        vm.write_memory(endptr_addr, &stop_addr.to_le_bytes());
    }
    vm.push((sign.wrapping_mul(val)) as u64);
}

pub fn host_strtod(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let str_addr = vm.pop() as u32;
    let endptr_addr = vm.pop() as u32;
    let s = read_cstring(vm, str_addr);
    let bytes = s.as_bytes();
    let mut pos = 0usize;
    while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }
    let start = pos;
    if pos < bytes.len() && (bytes[pos] == b'+' || bytes[pos] == b'-') {
        pos += 1;
    }
    let mut has_dot = false;
    let mut has_exp = false;
    while pos < bytes.len() {
        let c = bytes[pos];
        if c.is_ascii_digit() {
            pos += 1;
        } else if c == b'.' && !has_dot {
            has_dot = true;
            pos += 1;
        } else if (c == b'e' || c == b'E') && !has_exp {
            has_exp = true;
            pos += 1;
            if pos < bytes.len() && (bytes[pos] == b'+' || bytes[pos] == b'-') {
                pos += 1;
            }
        } else {
            break;
        }
    }
    let num_str = std::str::from_utf8(&bytes[start..pos]).unwrap_or("");
    let val: f64 = num_str.parse().unwrap_or(0.0);
    if pos == start {
        set_errno(vm, 1); // EINVAL
    }
    if endptr_addr != 0 {
        let stop_addr = str_addr + pos as u32;
        vm.write_memory(endptr_addr, &stop_addr.to_le_bytes());
    }
    vm.push(val.to_bits());
}

pub fn host_strerror(vm: &mut CideVM, session: &mut VmContext<'_>) {
    let errnum = vm.pop() as i32;
    let msg = match errnum {
        1 => "Invalid argument\0",
        2 => "Numerical result out of range\0",
        3 => "Domain error\0",
        4 => "No such file or directory\0",
        5 => "Permission denied\0",
        _ => "Unknown error\0",
    };
    let addr = match session.memory.allocate_raw(msg.len() as u32, vm.get_memory_size()) {
        Some(a) => a,
        None => {
            vm.push(0);
            return;
        }
    };
    vm.write_memory(addr, msg.as_bytes());
    vm.push(addr as u64);
}

pub fn host_time(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let _tloc = vm.pop() as u32;
    let secs = (current_time_millis() / 1000) as i64;
    vm.push(secs as u64);
}

pub fn host_clock(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    // 返回一个近似值：使用当前时间戳的微秒数作为单调时钟
    // CLOCKS_PER_SEC 通常定义为 1_000_000，返回微秒级精度
    let clocks = (current_time_millis() * 1000) as i64;
    vm.push(clocks as u64);
}

pub fn host_cide_assert_fail(vm: &mut CideVM, session: &mut VmContext<'_>) {
    session.runtime.output_lines.push("🚫 断言失败 (assertion failed)".to_string());
    vm.set_finished(1);
}

pub fn host_set_array_guard(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let base_addr = vm.pop() as u32;
    vm.pending_array_construction = Some(ArrayConstructionGuard {
        base_addr,
        frame_depth: vm.call_stack_len(),
    });
}

pub fn host_clear_array_guard(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    vm.pending_array_construction = None;
}
