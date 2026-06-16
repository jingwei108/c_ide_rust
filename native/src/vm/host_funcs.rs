use super::core::CideVM;
use super::host_func_id;
use crate::session::{MemoryRegion, Session};

pub(crate) use super::core::{ArrayConstructionGuard, FreedRegionInfo, NULL_TRAP_SIZE};
pub(crate) use super::instruction::SourceLoc;
pub(crate) use crate::session::FreeBlock;

#[path = "host/utils.rs"]
mod utils;
#[path = "host/memory.rs"]
mod memory;
#[path = "host/string.rs"]
mod string;
#[path = "host/io.rs"]
mod io;
pub(crate) use utils::*;
pub use memory::*;
pub use string::*;
pub use io::*;

pub fn host_rand(vm: &mut CideVM, session: &mut Session) {
    let seed = session.runtime.rand_seed;
    let next = seed.wrapping_mul(1103515245).wrapping_add(12345);
    session.runtime.rand_seed = next;
    vm.push((next & 0x7fff) as u64);
}

pub fn host_srand(vm: &mut CideVM, session: &mut Session) {
    let seed = vm.pop();
    session.runtime.rand_seed = seed as u32;
}

pub fn host_exit(vm: &mut CideVM, _session: &mut Session) {
    let code = vm.pop() as i32;
    vm.set_finished(code);
}

pub fn host_abs(vm: &mut CideVM, _session: &mut Session) {
    let n = vm.pop() as i32;
    vm.push(if n < 0 { n.wrapping_neg() as u64 } else { n as u64 });
}

// ========== math.h Host Functions ==========

pub fn host_sin(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::sin(x).to_bits());
}

pub fn host_cos(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::cos(x).to_bits());
}

pub fn host_sqrt(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::sqrt(x).to_bits());
}

pub fn host_pow(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    let y = f64::from_bits(vm.pop());
    vm.push(libm::pow(x, y).to_bits());
}

pub fn host_atan(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::atan(x).to_bits());
}

pub fn host_log(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::log(x).to_bits());
}

pub fn host_exp(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::exp(x).to_bits());
}

pub fn host_isdigit(vm: &mut CideVM, _session: &mut Session) {
    let c = vm.pop() as i32;
    vm.push(if c >= '0' as i32 && c <= '9' as i32 { 1 } else { 0 });
}

pub fn host_isalpha(vm: &mut CideVM, _session: &mut Session) {
    let c = vm.pop() as i32;
    vm.push(
        if (c >= 'a' as i32 && c <= 'z' as i32) || (c >= 'A' as i32 && c <= 'Z' as i32) {
            1
        } else {
            0
        },
    );
}

pub fn host_islower(vm: &mut CideVM, _session: &mut Session) {
    let c = vm.pop() as i32;
    vm.push(if c >= 'a' as i32 && c <= 'z' as i32 { 1 } else { 0 });
}

pub fn host_isupper(vm: &mut CideVM, _session: &mut Session) {
    let c = vm.pop() as i32;
    vm.push(if c >= 'A' as i32 && c <= 'Z' as i32 { 1 } else { 0 });
}

pub fn host_tolower(vm: &mut CideVM, _session: &mut Session) {
    let c = vm.pop() as i32;
    vm.push(if c >= 'A' as i32 && c <= 'Z' as i32 {
        c + ('a' as i32 - 'A' as i32)
    } else {
        c
    } as u64);
}

pub fn host_toupper(vm: &mut CideVM, _session: &mut Session) {
    let c = vm.pop() as i32;
    vm.push(if c >= 'a' as i32 && c <= 'z' as i32 {
        c + ('A' as i32 - 'a' as i32)
    } else {
        c
    } as u64);
}

pub fn host_isspace(vm: &mut CideVM, _session: &mut Session) {
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

pub fn host_isalnum(vm: &mut CideVM, _session: &mut Session) {
    let c = vm.pop() as i32;
    let alpha = (c >= 'a' as i32 && c <= 'z' as i32) || (c >= 'A' as i32 && c <= 'Z' as i32);
    let digit = c >= '0' as i32 && c <= '9' as i32;
    vm.push(if alpha || digit { 1 } else { 0 });
}

pub fn host_isprint(vm: &mut CideVM, _session: &mut Session) {
    let c = vm.pop() as i32;
    vm.push(if c >= ' ' as i32 && c <= '~' as i32 { 1 } else { 0 });
}

pub fn host_iscntrl(vm: &mut CideVM, _session: &mut Session) {
    let c = vm.pop() as i32;
    vm.push(if (0..=31).contains(&c) || c == 127 { 1 } else { 0 });
}

pub fn host_isxdigit(vm: &mut CideVM, _session: &mut Session) {
    let c = vm.pop() as i32;
    let digit = c >= '0' as i32 && c <= '9' as i32;
    let lower = c >= 'a' as i32 && c <= 'f' as i32;
    let upper = c >= 'A' as i32 && c <= 'F' as i32;
    vm.push(if digit || lower || upper { 1 } else { 0 });
}

pub fn host_isgraph(vm: &mut CideVM, _session: &mut Session) {
    let c = vm.pop() as i32;
    vm.push(if c > ' ' as i32 && c <= '~' as i32 { 1 } else { 0 });
}

pub fn host_ispunct(vm: &mut CideVM, _session: &mut Session) {
    let c = vm.pop() as i32;
    let is_print = c >= ' ' as i32 && c <= '~' as i32;
    let is_alnum = (c >= 'a' as i32 && c <= 'z' as i32)
        || (c >= 'A' as i32 && c <= 'Z' as i32)
        || (c >= '0' as i32 && c <= '9' as i32);
    vm.push(if is_print && !is_alnum && c != ' ' as i32 { 1 } else { 0 });
}

pub fn host_isblank(vm: &mut CideVM, _session: &mut Session) {
    let c = vm.pop() as i32;
    vm.push(if c == ' ' as i32 || c == '\t' as i32 { 1 } else { 0 });
}

pub fn host_atoi(vm: &mut CideVM, _session: &mut Session) {
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

pub fn host_qsort(vm: &mut CideVM, session: &mut Session) {
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
            vm.trap("qsort: write back out of bounds", &super::instruction::SourceLoc::default());
            vm.set_qsort_depth(vm.qsort_depth() - 1);
            return;
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
                    alloc_line: vm.get_current_line(),
                    alloc_by: "fopen".to_string(),
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

fn host_fgets(vm: &mut CideVM, session: &mut Session) {
    // 参数从右到左压栈：buf, n, stream → 栈顶是 buf
    let buf = vm.pop() as u32;
    let n = vm.pop() as usize;
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let ret = session.vfs.fgets(fd, buf, n, vm);
    vm.push(ret as u64);
}

fn host_fputs(vm: &mut CideVM, session: &mut Session) {
    // 参数从右到左压栈：s, stream → 栈顶是 s
    let s_addr = vm.pop() as u32;
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let mut vfs = std::mem::take(&mut session.vfs);
    let ret = vfs.fputs(fd, s_addr, vm, &mut session.memory);
    session.vfs = vfs;
    vm.push(ret as u64);
}

/// 从 FILE* 指针（VM Heap 地址）读取 fd 索引
const MAX_BSEARCH_DEPTH: i32 = 8;

pub fn host_bsearch(vm: &mut CideVM, session: &mut Session) {
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

pub fn host_fgetc(vm: &mut CideVM, session: &mut Session) {
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let mut vfs = std::mem::take(&mut session.vfs);
    let ch = vfs.fgetc(fd, vm);
    session.vfs = vfs;
    vm.push(ch as i64 as u64);
}

pub fn host_fputc(vm: &mut CideVM, session: &mut Session) {
    let c = vm.pop() as i32;
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let mut vfs = std::mem::take(&mut session.vfs);
    let ret = vfs.fputc(fd, c, vm, &mut session.memory);
    session.vfs = vfs;
    vm.push(ret as i64 as u64);
}

pub fn host_fseek(vm: &mut CideVM, session: &mut Session) {
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let offset = vm.pop() as i32;
    let whence = vm.pop() as i32;
    let mut vfs = std::mem::take(&mut session.vfs);
    let ret = vfs.fseek(fd, offset, whence, vm);
    session.vfs = vfs;
    vm.push(ret as u64);
}

pub fn host_ftell(vm: &mut CideVM, session: &mut Session) {
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let vfs = std::mem::take(&mut session.vfs);
    let ret = vfs.ftell(fd, vm);
    session.vfs = vfs;
    vm.push(ret as i64 as u64);
}

pub fn host_rewind(vm: &mut CideVM, session: &mut Session) {
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let mut vfs = std::mem::take(&mut session.vfs);
    vfs.rewind(fd);
    session.vfs = vfs;
}

// ========== String / memory extensions ==========

pub fn host_atof(vm: &mut CideVM, _session: &mut Session) {
    let addr = vm.pop() as u32;
    let s = read_cstring(vm, addr);
    let val: f64 = s.trim().parse().unwrap_or(0.0);
    vm.push(val.to_bits());
}

pub fn host_atol(vm: &mut CideVM, _session: &mut Session) {
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

pub fn host_tan(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::tan(x).to_bits());
}

pub fn host_log10(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::log10(x).to_bits());
}

pub fn host_fabs(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::fabs(x).to_bits());
}

pub fn host_ceil(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::ceil(x).to_bits());
}

pub fn host_floor(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::floor(x).to_bits());
}

pub fn host_round(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::round(x).to_bits());
}

pub fn host_fmod(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    let y = f64::from_bits(vm.pop());
    vm.push(libm::fmod(x, y).to_bits());
}

pub fn host_asin(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::asin(x).to_bits());
}

pub fn host_acos(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::acos(x).to_bits());
}

pub fn host_atan2(vm: &mut CideVM, _session: &mut Session) {
    let y = f64::from_bits(vm.pop());
    let x = f64::from_bits(vm.pop());
    vm.push(libm::atan2(y, x).to_bits());
}

pub fn host_sinh(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::sinh(x).to_bits());
}

pub fn host_cosh(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::cosh(x).to_bits());
}

pub fn host_tanh(vm: &mut CideVM, _session: &mut Session) {
    let x = f64::from_bits(vm.pop());
    vm.push(libm::tanh(x).to_bits());
}

pub fn host_llabs(vm: &mut CideVM, _session: &mut Session) {
    let n = vm.pop() as i64;
    vm.push(if n < 0 { n.wrapping_neg() as u64 } else { n as u64 });
}

pub fn host_abort(vm: &mut CideVM, session: &mut Session) {
    session.runtime.output_lines.push("[abort] 程序异常终止 (SIGABRT)".to_string());
    vm.set_finished(134);
}

pub fn host_strtol(vm: &mut CideVM, _session: &mut Session) {
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

pub fn host_strtod(vm: &mut CideVM, _session: &mut Session) {
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

pub fn host_strerror(vm: &mut CideVM, session: &mut Session) {
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

pub fn host_fflush(vm: &mut CideVM, session: &mut Session) {
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let mut vfs = std::mem::take(&mut session.vfs);
    let ret = if fd == 0 {
        // fflush(NULL) — 刷新所有流，VFS 内存模式无缓冲，视为成功
        0
    } else {
        vfs.fflush(fd)
    };
    session.vfs = vfs;
    vm.push(ret as u64);
}

pub fn host_perror(vm: &mut CideVM, session: &mut Session) {
    let s_addr = vm.pop() as u32;
    let prefix = read_cstring(vm, s_addr);
    let msg = if prefix.is_empty() {
        "Error\n".to_string()
    } else {
        format!("{}: Error\n", prefix)
    };
    session.runtime.output_lines.push(msg);
}

pub fn host_clearerr(vm: &mut CideVM, session: &mut Session) {
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let mut vfs = std::mem::take(&mut session.vfs);
    vfs.clearerr(fd);
    session.vfs = vfs;
}

pub fn host_time(vm: &mut CideVM, _session: &mut Session) {
    let _tloc = vm.pop() as u32;
    let secs = (current_time_millis() / 1000) as i64;
    vm.push(secs as u64);
}

pub fn host_clock(vm: &mut CideVM, _session: &mut Session) {
    // 返回一个近似值：使用当前时间戳的微秒数作为单调时钟
    // CLOCKS_PER_SEC 通常定义为 1_000_000，返回微秒级精度
    let clocks = (current_time_millis() * 1000) as i64;
    vm.push(clocks as u64);
}

pub fn host_cide_assert_fail(vm: &mut CideVM, session: &mut Session) {
    session.runtime.output_lines.push("🚫 断言失败 (assertion failed)".to_string());
    vm.set_finished(1);
}

pub fn host_set_array_guard(vm: &mut CideVM, _session: &mut Session) {
    let base_addr = vm.pop() as u32;
    vm.pending_array_construction = Some(crate::vm::core::ArrayConstructionGuard {
        base_addr,
        frame_depth: vm.call_stack_len(),
    });
}

pub fn host_clear_array_guard(vm: &mut CideVM, _session: &mut Session) {
    vm.pending_array_construction = None;
}

pub fn host_remove(vm: &mut CideVM, session: &mut Session) {
    let path_addr = vm.pop() as u32;
    let path = read_cstring(vm, path_addr);
    let mut vfs = std::mem::take(&mut session.vfs);
    let ret = vfs.remove(&path, &mut session.memory);
    session.vfs = vfs;
    vm.push(ret as u64);
}

pub fn host_rename(vm: &mut CideVM, session: &mut Session) {
    let old_path_addr = vm.pop() as u32;
    let new_path_addr = vm.pop() as u32;
    let old_path = read_cstring(vm, old_path_addr);
    let new_path = read_cstring(vm, new_path_addr);
    let mut vfs = std::mem::take(&mut session.vfs);
    let ret = vfs.rename(&old_path, &new_path);
    session.vfs = vfs;
    vm.push(ret as u64);
}

pub fn execute_host_func(vm: &mut CideVM, session: &mut Session, id: u32) {
    match id {
        host_func_id::OUTPUT => host_output(vm, session),
        host_func_id::STEP => host_step(vm, session),
        host_func_id::MALLOC => host_malloc(vm, session),
        host_func_id::FREE => host_free(vm, session),
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
        host_func_id::ABS => host_abs(vm, session),
        host_func_id::ISDIGIT => host_isdigit(vm, session),
        host_func_id::ISALPHA => host_isalpha(vm, session),
        host_func_id::ISLOWER => host_islower(vm, session),
        host_func_id::ISUPPER => host_isupper(vm, session),
        host_func_id::TOLOWER => host_tolower(vm, session),
        host_func_id::TOUPPER => host_toupper(vm, session),
        host_func_id::ISSPACE => host_isspace(vm, session),
        host_func_id::ISALNUM => host_isalnum(vm, session),
        host_func_id::ISPRINT => host_isprint(vm, session),
        host_func_id::ISCNTRL => host_iscntrl(vm, session),
        host_func_id::ISXDIGIT => host_isxdigit(vm, session),
        host_func_id::STRNCPY => host_strncpy(vm, session),
        host_func_id::MEMCPY => host_memcpy(vm, session),
        host_func_id::MEMMOVE => host_memmove(vm, session),
        host_func_id::FPRINTF => host_fprintf_n(vm, session),
        host_func_id::REALLOC => host_realloc(vm, session),
        host_func_id::QSORT => host_qsort(vm, session),
        host_func_id::FOPEN => host_fopen(vm, session),
        host_func_id::FREAD => host_fread(vm, session),
        host_func_id::FWRITE => host_fwrite(vm, session),
        host_func_id::FCLOSE => host_fclose(vm, session),
        host_func_id::FEOF => host_feof(vm, session),
        host_func_id::FGETS => host_fgets(vm, session),
        host_func_id::FPUTS => host_fputs(vm, session),
        host_func_id::SIN => host_sin(vm, session),
        host_func_id::COS => host_cos(vm, session),
        host_func_id::SQRT => host_sqrt(vm, session),
        host_func_id::POW => host_pow(vm, session),
        host_func_id::ATAN => host_atan(vm, session),
        host_func_id::LOG => host_log(vm, session),
        host_func_id::EXP => host_exp(vm, session),
        host_func_id::STRDUP => host_strdup(vm, session),
        host_func_id::UNGETC => host_ungetc(vm, session),
        host_func_id::PUTS => host_puts(vm, session),
        host_func_id::CALLOC => host_calloc(vm, session),
        host_func_id::BSEARCH => host_bsearch(vm, session),
        host_func_id::SPRINTF => host_sprintf(vm, session),
        host_func_id::SNPRINTF => host_snprintf(vm, session),
        host_func_id::SSCANF => host_sscanf(vm, session),
        host_func_id::FGETC => host_fgetc(vm, session),
        host_func_id::FPUTC => host_fputc(vm, session),
        host_func_id::FSEEK => host_fseek(vm, session),
        host_func_id::FTELL => host_ftell(vm, session),
        host_func_id::REWIND => host_rewind(vm, session),
        host_func_id::STRNCAT => host_strncat(vm, session),
        host_func_id::STRNCMP => host_strncmp(vm, session),
        host_func_id::MEMCMP => host_memcmp(vm, session),
        host_func_id::STRCHR => host_strchr(vm, session),
        host_func_id::STRRCHR => host_strrchr(vm, session),
        host_func_id::STRSTR => host_strstr(vm, session),
        host_func_id::MEMCHR => host_memchr(vm, session),
        host_func_id::ATOF => host_atof(vm, session),
        host_func_id::ATOL => host_atol(vm, session),
        host_func_id::TAN => host_tan(vm, session),
        host_func_id::LOG10 => host_log10(vm, session),
        host_func_id::FABS => host_fabs(vm, session),
        host_func_id::CEIL => host_ceil(vm, session),
        host_func_id::FLOOR => host_floor(vm, session),
        host_func_id::ROUND => host_round(vm, session),
        host_func_id::FMOD => host_fmod(vm, session),
        host_func_id::ISGRAPH => host_isgraph(vm, session),
        host_func_id::ISPUNCT => host_ispunct(vm, session),
        host_func_id::ISBLANK => host_isblank(vm, session),
        host_func_id::ASIN => host_asin(vm, session),
        host_func_id::ACOS => host_acos(vm, session),
        host_func_id::ATAN2 => host_atan2(vm, session),
        host_func_id::SINH => host_sinh(vm, session),
        host_func_id::COSH => host_cosh(vm, session),
        host_func_id::TANH => host_tanh(vm, session),
        host_func_id::LLABS => host_llabs(vm, session),
        host_func_id::ABORT => host_abort(vm, session),
        host_func_id::STRTOL => host_strtol(vm, session),
        host_func_id::STRTOD => host_strtod(vm, session),
        host_func_id::STRERROR => host_strerror(vm, session),
        host_func_id::FFLUSH => host_fflush(vm, session),
        host_func_id::PERROR => host_perror(vm, session),
        host_func_id::CLEARERR => host_clearerr(vm, session),
        host_func_id::TIME => host_time(vm, session),
        host_func_id::CLOCK => host_clock(vm, session),
        host_func_id::ASSERT_FAIL => host_cide_assert_fail(vm, session),
        host_func_id::SET_ARRAY_GUARD => host_set_array_guard(vm, session),
        host_func_id::CLEAR_ARRAY_GUARD => host_clear_array_guard(vm, session),
        host_func_id::REMOVE => host_remove(vm, session),
        host_func_id::RENAME => host_rename(vm, session),
        host_func_id::STRPBRK => host_strpbrk(vm, session),
        host_func_id::STRSPN => host_strspn(vm, session),
        host_func_id::STRCSPN => host_strcspn(vm, session),
        _ => {}
    }
}

fn host_step(vm: &mut CideVM, session: &mut Session) {
    let line = vm.pop() as i32;
    session.runtime.current_line = line;
    session.runtime.trace.push(crate::session::TraceEntry {
        line,
        operation: "step".to_string(),
    });
}

fn host_output(vm: &mut CideVM, session: &mut Session) {
    let val = vm.pop();
    session.runtime.output_lines.push(format!("{}\n", val));
}

