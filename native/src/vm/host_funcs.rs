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
/// 解析一个 printf/scanf 格式说明符（% 之后的内容）。
/// 返回 (格式字母, 是否 ll, 精度)。
#[allow(clippy::type_complexity)]
fn parse_format_spec(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<(char, bool, Option<usize>, Option<usize>, String)> {
    // 收集 flags
    let mut flags = String::new();
    while let Some(&c) = chars.peek() {
        if c == '-' || c == '+' || c == ' ' || c == '#' || c == '0' {
            if !flags.contains(c) {
                flags.push(c);
            }
            chars.next();
        } else {
            break;
        }
    }
    // 解析 width
    let mut width: Option<usize> = None;
    let mut width_str = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            width_str.push(c);
            chars.next();
        } else if c == '*' {
            chars.next();
            break;
        } else {
            break;
        }
    }
    if !width_str.is_empty() {
        width = width_str.parse().ok();
    }
    // 解析 precision
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
    // 格式字母
    chars.peek().copied().map(|spec| {
        chars.next();
        (spec, is_ll, precision, width, flags)
    })
}

fn parse_format_specs(fmt: &str) -> Vec<char> {
    let mut specs = Vec::new();
    let mut chars = fmt.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            if let Some(&next) = chars.peek() {
                if next == '%' {
                    chars.next(); // 跳过 %%
                } else if let Some((spec, _, _, _, _)) = parse_format_spec(&mut chars) {
                    specs.push(spec);
                }
            }
        }
    }
    specs
}

fn apply_width(s: &str, width: Option<usize>, flags: &str) -> String {
    let w = match width {
        Some(w) => w,
        None => return s.to_string(),
    };
    if s.len() >= w {
        return s.to_string();
    }
    let pad_len = w - s.len();
    let left_align = flags.contains('-');
    let zero_pad = flags.contains('0') && !left_align;
    let pad_char = if zero_pad { '0' } else { ' ' };
    if left_align {
        format!("{}{}", s, pad_char.to_string().repeat(pad_len))
    } else {
        format!("{}{}", pad_char.to_string().repeat(pad_len), s)
    }
}

fn trim_trailing_zeros_and_dot(s: &str) -> String {
    if !s.contains('.') {
        return s.to_string();
    }
    let mut result = s.trim_end_matches('0').to_string();
    if result.ends_with('.') {
        result.pop();
    }
    result
}

fn format_g(val: f64, prec: usize, upper: bool) -> String {
    if val.is_nan() {
        return if upper { "NAN" } else { "nan" }.to_string();
    }
    if val.is_infinite() {
        return if val.is_sign_positive() {
            if upper { "INF" } else { "inf" }.to_string()
        } else {
            if upper { "-INF" } else { "-inf" }.to_string()
        };
    }
    if val == 0.0 {
        return "0".to_string();
    }

    let prec = prec.max(1);
    let abs_val = val.abs();
    let exp = abs_val.log10().floor() as i32;

    let mut result;
    let e_char = if upper { 'E' } else { 'e' };

    if exp < -4 || exp >= prec as i32 {
        let mantissa = abs_val / 10f64.powi(exp);
        let s = format!("{:.*}", prec - 1, mantissa);
        let s = trim_trailing_zeros_and_dot(&s);
        result = format!("{}{}{:+#03}", s, e_char, exp);
    } else {
        let frac_digits = (prec as i32 - 1 - exp).max(0) as usize;
        let s = format!("{:.*}", frac_digits, abs_val);
        result = trim_trailing_zeros_and_dot(&s);
    }

    if val < 0.0 && !result.starts_with('-') {
        result = format!("-{}", result);
    }
    result
}

/// 根据格式字符串和参数列表生成 printf 输出。
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
                } else if let Some((spec, is_ll, precision, width, flags)) = parse_format_spec(&mut chars) {
                    let arg = args[used];
                    let mut piece = String::new();
                    match spec {
                        'd' | 'i' => {
                            let val = if is_ll { (arg as i64).to_string() } else { (arg as i32).to_string() };
                            piece = apply_width(&val, width, &flags);
                        }
                        'u' => {
                            let val = if is_ll { arg.to_string() } else { (arg as u32).to_string() };
                            piece = apply_width(&val, width, &flags);
                        }
                        'x' => {
                            let val = if is_ll { format!("{:x}", arg) } else { format!("{:x}", arg as u32) };
                            piece = apply_width(&val, width, &flags);
                        }
                        'X' => {
                            let val = if is_ll { format!("{:X}", arg) } else { format!("{:X}", arg as u32) };
                            piece = apply_width(&val, width, &flags);
                        }
                        'o' => {
                            let val = if is_ll { format!("{:o}", arg) } else { format!("{:o}", arg as u32) };
                            piece = apply_width(&val, width, &flags);
                        }
                        'p' => {
                            let val = format!("{:p}", arg as u32 as *const ());
                            piece = apply_width(&val, width, &flags);
                        }
                        'f' => {
                            let f = f64::from_bits(arg);
                            let prec = precision.unwrap_or(6);
                            let val = format!("{:.*}", prec, f);
                            piece = apply_width(&val, width, &flags);
                        }
                        'g' => {
                            let f = f64::from_bits(arg);
                            let prec = precision.unwrap_or(6);
                            let val = format_g(f, prec, false);
                            piece = apply_width(&val, width, &flags);
                        }
                        'G' => {
                            let f = f64::from_bits(arg);
                            let prec = precision.unwrap_or(6);
                            let val = format_g(f, prec, true);
                            piece = apply_width(&val, width, &flags);
                        }
                        's' => {
                            let val = read_cstring(vm, arg as u32);
                            piece = apply_width(&val, width, &flags);
                        }
                        'c' => {
                            let val = (arg as u8 as char).to_string();
                            piece = apply_width(&val, width, &flags);
                        }
                        _ => { piece.push(ch); piece.push(spec); }
                    }
                    out.push_str(&piece);
                    used += 1;
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

pub fn host_malloc(vm: &mut CideVM, session: &mut Session) {
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
    // 清理被新分配重用的 freed_logs
    let new_end = addr.saturating_add(aligned_size);
    vm.freed_logs.retain(|log| {
        let log_end = log.addr.saturating_add(log.size);
        log_end <= addr || log.addr >= new_end
    });
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
            alloc_line: vm.get_current_line(),
            alloc_by: "malloc".to_string(),
        });
    } else {
        // 复用已释放的 region 时更新分配信息
        for r in &mut session.memory.regions {
            if r.addr == addr && !r.is_freed {
                r.alloc_line = vm.get_current_line();
                r.alloc_by = "malloc".to_string();
                break;
            }
        }
    }
    vm.push(addr as u64);
}

pub fn host_free(vm: &mut CideVM, session: &mut Session) {
    let addr = vm.pop() as u32;
    if addr == 0 {
        return;
    }
    // Double-Free 检测
    if let Some(log) = vm.freed_logs.iter().find(|log| log.addr == addr) {
        let msg = format!("🔁 Double-Free (E3061)：你正在 free 一块已经在第 {} 行被释放过的内存（由第 {} 行的 malloc/realloc 分配）。\n\n💡 原因：同一块内存被释放了两次，这通常是因为 free(p) 后没有将 p 置为 NULL，或者两个指针指向同一块内存且都被释放了。\n✅ 解决方法：每次 free(p) 后立刻写 p = NULL;。对 NULL 指针重复 free 是安全的。", log.freed_line, log.alloc_line);
        vm.trap(&msg, &super::instruction::SourceLoc::default());
        return;
    }
    for r in &mut session.memory.regions {
        if r.addr == addr && !r.is_freed {
            r.is_freed = true;
            let aligned_size = ((r.size as u32) + 3) & !3;
            vm.freed_logs.push(super::vm::FreedRegionInfo {
                addr: r.addr,
                size: aligned_size,
                alloc_line: r.alloc_line,
                freed_line: vm.get_current_line(),
                alloc_step: 0,
                freed_step: vm.get_executed_steps(),
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

pub fn host_printf_n(vm: &mut CideVM, session: &mut Session) {
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

pub fn host_scanf_n(vm: &mut CideVM, session: &mut Session) {
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
            'u' => {
                while pos < chars.len() && chars[pos].is_whitespace() {
                    pos += 1;
                }
                if pos >= chars.len() { break; }
                let start = pos;
                if chars[pos] == '+' {
                    pos += 1;
                }
                while pos < chars.len() && chars[pos].is_ascii_digit() {
                    pos += 1;
                }
                let token: String = chars[start..pos].iter().collect();
                if *len_mod >= 2 {
                    let value: u64 = token.parse().unwrap_or(0);
                    vm.store_i64(ptr, value, &super::instruction::SourceLoc::default());
                } else {
                    let value: u32 = token.parse().unwrap_or(0);
                    vm.store_i32(ptr, value as i32, &super::instruction::SourceLoc::default());
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

pub fn host_strlen(vm: &mut CideVM, _session: &mut Session) {
    let addr = vm.pop() as u32;
    let bytes = read_cbytes(vm, addr);
    vm.push(bytes.len() as u64);
}

pub fn host_strdup(vm: &mut CideVM, session: &mut Session) {
    let src = vm.pop() as u32;
    let src_bytes = read_cbytes(vm, src);
    let len = src_bytes.len();
    let size = (len + 1) as i32;
    if size <= 0 {
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
    // 清理被新分配重用的 freed_logs
    let new_end = addr.saturating_add(aligned_size);
    vm.freed_logs.retain(|log| {
        let log_end = log.addr.saturating_add(log.size);
        log_end <= addr || log.addr >= new_end
    });
    // 复制字符串内容（含终止符）
    let mem_size = vm.get_memory_slice().len();
    if (addr as usize) < mem_size {
        let max_write = mem_size - addr as usize;
        let write_len = len.min(max_write.saturating_sub(1));
        for (i, &b) in src_bytes.iter().enumerate().take(write_len) {
            vm.store_i8(addr + i as u32, b as i32, &super::instruction::SourceLoc::default());
        }
        let end = addr as usize + write_len;
        if end < mem_size {
            vm.store_i8(end as u32, 0, &super::instruction::SourceLoc::default());
        }
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
            alloc_line: vm.get_current_line(),
            alloc_by: "strdup".to_string(),
        });
    } else {
        for r in &mut session.memory.regions {
            if r.addr == addr && !r.is_freed {
                r.alloc_line = vm.get_current_line();
                r.alloc_by = "strdup".to_string();
                break;
            }
        }
    }
    vm.push(addr as u64);
}

pub fn host_strcpy(vm: &mut CideVM, session: &mut Session) {
    let dest = vm.pop() as u32;
    let src = vm.pop() as u32;
    let src_bytes = read_cbytes(vm, src);
    let src_len = src_bytes.len();
    let mem_size = vm.get_memory_slice().len();
    if dest as usize >= mem_size {
        vm.push(0);
        return;
    }

    // 注入边界检查：如果 dest 落在已知的堆分配区域内，验证容量
    for r in &session.memory.regions {
        if !r.is_freed && dest >= r.addr && dest < r.addr + r.size as u32 {
            let offset = (dest - r.addr) as usize;
            let available = (r.size as usize).saturating_sub(offset);
            if src_len + 1 > available {
                vm.trap(
                    &format!(
                        "💥 Buffer Overflow (E3070)：strcpy 目标缓冲区溢出。源字符串长度 {} 字节（含终止符 {} 字节），但目标区域 '{}' 从偏移 {} 开始仅剩 {} 字节。\n\n💡 原因：strcpy 不会检查目标缓冲区大小。\n✅ 解决方法：使用 strncpy 或确保目标缓冲区足够大（至少 {} 字节）。",
                        src_len, src_len + 1, r.name, offset, available, src_len + 1
                    ),
                    &super::instruction::SourceLoc::default(),
                );
                return;
            }
            break;
        }
    }

    let max_copy = mem_size - dest as usize;
    let copy_len = src_len.min(max_copy.saturating_sub(1));
    for (i, &b) in src_bytes.iter().enumerate().take(copy_len) {
        vm.store_i8(dest + i as u32, b as i32, &super::instruction::SourceLoc::default());
    }
    let end = dest as usize + copy_len;
    vm.store_i8(end as u32, 0, &super::instruction::SourceLoc::default());
    vm.push(dest as u64);
}

pub fn host_strcmp(vm: &mut CideVM, _session: &mut Session) {
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


pub fn host_ungetc(vm: &mut CideVM, session: &mut Session) {
    let ch = vm.pop() as i32;
    let _stream = vm.pop() as i32;
    session.runtime.ungetc_char = Some(ch);
    vm.push(ch as i64 as u64);
}

pub fn host_getchar(vm: &mut CideVM, session: &mut Session) {
    // 先检查 ungetc 缓存
    if let Some(ch) = session.runtime.ungetc_char.take() {
        vm.push(ch as i64 as u64);
        return;
    }
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
        if session.runtime.input_mode == crate::session::InputMode::Batch {
            // Batch 模式：输入耗尽后返回 EOF (-1)
            vm.push((-1i32) as u64);
        } else {
            session.runtime.waiting_input = true;
        }
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

pub fn host_putchar(vm: &mut CideVM, session: &mut Session) {
    let val = vm.pop();
    session.runtime.output_lines.push((val as u8 as char).to_string());
}

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

pub fn host_memset(vm: &mut CideVM, _session: &mut Session) {
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
    // 注入 NULL 指针安全检查（与 VM store_i8 保持一致）
    if ptr < super::vm::NULL_TRAP_SIZE && write_len > 0 {
        let msg = format!("向 NULL 指针区域写入（地址 0x{:04X}）。请确认指针已被正确初始化。", ptr);
        vm.trap(&msg, &super::instruction::SourceLoc::default());
        vm.push(ptr as u64);
        return;
    }
    let byte_val = (value & 0xFF) as u8;
    let mem = vm.memory_ref_mut();
    mem[ptr as usize..ptr as usize + write_len].fill(byte_val);
    vm.push(ptr as u64);
}

pub fn host_exit(vm: &mut CideVM, _session: &mut Session) {
    let code = vm.pop() as i32;
    vm.set_finished(code);
}

pub fn host_strcat(vm: &mut CideVM, session: &mut Session) {
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
    let dest_len = end - dest as usize;
    let src_bytes = read_cbytes(vm, src);
    let src_len = src_bytes.len();

    // 注入边界检查
    for r in &session.memory.regions {
        if !r.is_freed && dest >= r.addr && dest < r.addr + r.size as u32 {
            let offset = (dest - r.addr) as usize;
            let available = (r.size as usize).saturating_sub(offset);
            if dest_len + src_len + 1 > available {
                vm.trap(
                    &format!(
                        "💥 Buffer Overflow (E3070)：strcat 目标缓冲区溢出。已有 {} 字节 + 源字符串 {} 字节 + 终止符 1 字节 = {} 字节，但目标区域 '{}' 从偏移 {} 开始仅剩 {} 字节。\n\n💡 原因：strcat 不会检查目标缓冲区剩余容量。\n✅ 解决方法：使用 strncat 或确保目标缓冲区足够大。",
                        dest_len, src_len, dest_len + src_len + 1, r.name, offset, available
                    ),
                    &super::instruction::SourceLoc::default(),
                );
                return;
            }
            break;
        }
    }

    let max_copy = mem_size.saturating_sub(end).saturating_sub(1);
    let copy_len = src_len.min(max_copy);
    for (i, &b) in src_bytes.iter().enumerate().take(copy_len) {
        vm.store_i8((end + i) as u32, b as i32, &super::instruction::SourceLoc::default());
    }
    vm.store_i8((end + copy_len) as u32, 0, &super::instruction::SourceLoc::default());
    vm.push(dest as u64);
}

pub fn host_strncpy(vm: &mut CideVM, _session: &mut Session) {
    let dest = vm.pop() as u32;
    let src = vm.pop() as u32;
    let n = vm.pop() as i32;
    let src_bytes = read_cbytes(vm, src);
    let mem_size = vm.get_memory_slice().len();
    if dest as usize >= mem_size {
        vm.push(dest as u64);
        return;
    }
    let max_write = mem_size - dest as usize;
    let write_len = (n as usize).min(max_write);
    let copy_len = src_bytes.len().min(write_len);
    for (i, &byte) in src_bytes.iter().enumerate().take(copy_len) {
        vm.store_i8(dest + i as u32, byte as i32, &super::instruction::SourceLoc::default());
    }
    for i in copy_len..write_len {
        vm.store_i8(dest + i as u32, 0, &super::instruction::SourceLoc::default());
    }
    vm.push(dest as u64);
}

pub fn host_memcpy(vm: &mut CideVM, _session: &mut Session) {
    let dest = vm.pop() as u32;
    let src = vm.pop() as u32;
    let n = vm.pop() as i32;
    let mem_size = vm.get_memory_slice().len();
    if dest as usize >= mem_size || src as usize >= mem_size {
        vm.push(dest as u64);
        return;
    }
    let copy_len = (n as usize).min(mem_size - dest as usize).min(mem_size - src as usize);
    if copy_len > 0 {
        let buf = {
            let mem = vm.memory_ref();
            mem[src as usize..src as usize + copy_len].to_vec()
        };
        let mem = vm.memory_ref_mut();
        for i in 0..copy_len {
            mem[dest as usize + i] = buf[i];
        }
    }
    vm.push(dest as u64);
}

pub fn host_memmove(vm: &mut CideVM, _session: &mut Session) {
    let dest = vm.pop() as u32;
    let src = vm.pop() as u32;
    let n = vm.pop() as i32;
    let mem_size = vm.get_memory_slice().len();
    if dest as usize >= mem_size || src as usize >= mem_size {
        vm.push(dest as u64);
        return;
    }
    let copy_len = (n as usize).min(mem_size - dest as usize).min(mem_size - src as usize);
    if copy_len > 0 {
        let buf = {
            let mem = vm.memory_ref();
            mem[src as usize..src as usize + copy_len].to_vec()
        };
        let mem = vm.memory_ref_mut();
        for i in 0..copy_len {
            mem[dest as usize + i] = buf[i];
        }
    }
    vm.push(dest as u64);
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
    vm.push(if (c >= 'a' as i32 && c <= 'z' as i32) || (c >= 'A' as i32 && c <= 'Z' as i32) { 1 } else { 0 });
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
    vm.push(if c >= 'A' as i32 && c <= 'Z' as i32 { c + ('a' as i32 - 'A' as i32) } else { c } as u64);
}

pub fn host_toupper(vm: &mut CideVM, _session: &mut Session) {
    let c = vm.pop() as i32;
    vm.push(if c >= 'a' as i32 && c <= 'z' as i32 { c + ('A' as i32 - 'a' as i32) } else { c } as u64);
}

pub fn host_isspace(vm: &mut CideVM, _session: &mut Session) {
    let c = vm.pop() as i32;
    vm.push(if c == ' ' as i32 || c == '\t' as i32 || c == '\n' as i32 || c == '\r' as i32 || c == '\x0C' as i32 || c == '\x0B' as i32 { 1 } else { 0 });
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

pub fn host_fprintf_n(vm: &mut CideVM, session: &mut Session) {
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

pub fn host_realloc(vm: &mut CideVM, session: &mut Session) {
    let ptr = vm.pop() as u32;
    let new_size = vm.pop() as i32;

    if new_size <= 0 {
        if ptr != 0 {
            // Equivalent to free
            for r in &mut session.memory.regions {
                if r.addr == ptr && !r.is_freed {
                    r.is_freed = true;
                    let aligned_size = ((r.size as u32) + 3) & !3;
                    vm.freed_logs.push(super::vm::FreedRegionInfo {
                        addr: r.addr,
                        size: aligned_size,
                        alloc_line: r.alloc_line,
                        freed_line: vm.get_current_line(),
                        alloc_step: 0,
                        freed_step: vm.get_executed_steps(),
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
        vm.push(0);
        return;
    }

    if ptr == 0 {
        // Equivalent to malloc
        vm.push(new_size as u64);
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

    let Some((old_addr, old_size)) = old_region else {
        vm.push(0);
        return;
    };
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

    // 清理被新分配重用的 freed_logs（必须在写入新内存之前执行，
    // 否则 store_i8 会触发 Use-After-Free 误报）
    let new_end = new_addr.saturating_add(aligned_new_size);
    vm.freed_logs.retain(|log| {
        let log_end = log.addr.saturating_add(log.size);
        log_end <= new_addr || log.addr >= new_end
    });

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
            vm.freed_logs.push(super::vm::FreedRegionInfo {
                addr: r.addr,
                size: aligned_old_size,
                alloc_line: r.alloc_line,
                freed_line: vm.get_current_line(),
                alloc_step: 0,
                freed_step: vm.get_executed_steps(),
            });
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
        alloc_line: vm.get_current_line(),
        alloc_by: "realloc".to_string(),
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
