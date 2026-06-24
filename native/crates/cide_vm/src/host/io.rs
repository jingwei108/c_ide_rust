use super::*;
use crate::VmContext;

pub fn host_printf_n(vm: &mut CideVM, session: &mut VmContext<'_>) {
    let fmt_addr = vm.pop() as u32;
    let fmt = read_cstring(vm, fmt_addr);
    let specs = parse_format_specs(&fmt);
    if vm.get_stack().len() < specs.len() {
        vm.trap("printf: 格式字符串要求的参数多于实际提供的参数。", &SourceLoc::default());
        return;
    }
    let mut args = Vec::with_capacity(specs.len());
    for _ in 0..specs.len() {
        args.push(vm.pop());
    }
    let out = format_printf_string(vm, &fmt, &args);
    session.runtime.output_lines.push(out);
}

pub fn host_scanf_n(vm: &mut CideVM, session: &mut VmContext<'_>) {
    let fmt_addr = vm.pop() as u32;
    let fmt = read_cstring(vm, fmt_addr);
    // 扫描格式字符串，记录每个 % 格式符的类型及是否带 long 修饰符
    let spec_types = parse_scanf_specs(&fmt);
    if vm.get_stack().len() < spec_types.len() {
        vm.trap("scanf: 格式字符串要求的参数多于实际提供的参数。", &SourceLoc::default());
        return;
    }
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
                if pos >= chars.len() {
                    break;
                }
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
                    vm.store_i64(ptr, value as u64, &SourceLoc::default());
                } else {
                    let value: i32 = token.parse().unwrap_or(0);
                    vm.store_i32(ptr, value, &SourceLoc::default());
                }
            }
            'u' => {
                while pos < chars.len() && chars[pos].is_whitespace() {
                    pos += 1;
                }
                if pos >= chars.len() {
                    break;
                }
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
                    vm.store_i64(ptr, value, &SourceLoc::default());
                } else {
                    let value: u32 = token.parse().unwrap_or(0);
                    vm.store_i32(ptr, value as i32, &SourceLoc::default());
                }
            }
            'f' => {
                let (token, new_pos) = read_float_token(&chars, pos);
                pos = new_pos;
                if token.is_empty() {
                    break;
                }
                if *len_mod >= 1 {
                    // %lf → double (8 bytes)
                    let value: f64 = token.parse().unwrap_or(0.0);
                    vm.store_i64(ptr, value.to_bits(), &SourceLoc::default());
                } else {
                    // %f → float (4 bytes)
                    let value: f32 = token.parse().unwrap_or(0.0);
                    vm.store_i32(ptr, value.to_bits() as i32, &SourceLoc::default());
                }
            }
            'c' => {
                // 标准 C: %c 不跳过空白
                if pos >= chars.len() {
                    break;
                }
                let ch = chars[pos];
                vm.store_i8(ptr, ch as i32, &SourceLoc::default());
                pos += 1;
            }
            _ => {}
        }
    }
}

pub fn host_ungetc(vm: &mut CideVM, session: &mut VmContext<'_>) {
    let ch = vm.pop() as i32;
    let _stream = vm.pop() as i32;
    session.runtime.ungetc_char = Some(ch);
    vm.push(ch as i64 as u64);
}

pub fn host_getchar(vm: &mut CideVM, session: &mut VmContext<'_>) {
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
        if session.runtime.input_mode == cide_runtime::InputMode::Batch {
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

pub fn host_putchar(vm: &mut CideVM, session: &mut VmContext<'_>) {
    let val = vm.pop();
    session.runtime.output_lines.push((val as u8 as char).to_string());
}

pub fn host_fprintf_n(vm: &mut CideVM, session: &mut VmContext<'_>) {
    let _stream = vm.pop();
    let fmt_addr = vm.pop() as u32;
    let fmt = read_cstring(vm, fmt_addr);
    let specs = parse_format_specs(&fmt);
    if vm.get_stack().len() < specs.len() {
        vm.trap("fprintf: 格式字符串要求的参数多于实际提供的参数。", &SourceLoc::default());
        return;
    }
    let mut args = Vec::with_capacity(specs.len());
    for _ in 0..specs.len() {
        args.push(vm.pop());
    }
    let out = format_printf_string(vm, &fmt, &args);
    session.runtime.output_lines.push(out);
}

pub fn host_puts(vm: &mut CideVM, session: &mut VmContext<'_>) {
    let s_addr = vm.pop() as u32;
    let s = read_cstring(vm, s_addr);
    session.runtime.output_lines.push(s + "\n");
    vm.push(1); // puts returns non-negative on success
}

pub fn host_sprintf(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let buf_addr = vm.pop() as u32;
    let fmt_addr = vm.pop() as u32;
    let fmt = read_cstring(vm, fmt_addr);
    let specs = parse_format_specs(&fmt);
    let mut args = Vec::with_capacity(specs.len());
    for _ in 0..specs.len() {
        args.push(vm.pop());
    }
    let out = format_printf_string(vm, &fmt, &args);
    let bytes = out.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        vm.store_i8(buf_addr + i as u32, b as i32, &SourceLoc::default());
    }
    vm.store_i8(buf_addr + bytes.len() as u32, 0, &SourceLoc::default());
    vm.push(bytes.len() as u64);
}

pub fn host_snprintf(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let buf_addr = vm.pop() as u32;
    let size = vm.pop() as i32;
    let fmt_addr = vm.pop() as u32;
    let fmt = read_cstring(vm, fmt_addr);
    let specs = parse_format_specs(&fmt);
    let mut args = Vec::with_capacity(specs.len());
    for _ in 0..specs.len() {
        args.push(vm.pop());
    }
    let out = format_printf_string(vm, &fmt, &args);
    let bytes = out.as_bytes();
    if size > 0 {
        let n = std::cmp::min(bytes.len(), (size as usize).saturating_sub(1));
        for (i, &byte) in bytes.iter().enumerate().take(n) {
            vm.store_i8(buf_addr + i as u32, byte as i32, &SourceLoc::default());
        }
        vm.store_i8(buf_addr + n as u32, 0, &SourceLoc::default());
    }
    vm.push(bytes.len() as u64);
}

pub fn host_sscanf(vm: &mut CideVM, _session: &mut VmContext<'_>) {
    let str_addr = vm.pop() as u32;
    let fmt_addr = vm.pop() as u32;
    let fmt = read_cstring(vm, fmt_addr);
    let src = read_cstring(vm, str_addr);
    let spec_types = parse_scanf_specs(&fmt);
    let mut ptrs = Vec::with_capacity(spec_types.len());
    for _ in 0..spec_types.len() {
        ptrs.push(vm.pop() as u32);
    }
    let chars: Vec<char> = src.chars().collect();
    let mut pos = 0usize;
    let mut matched = 0usize;
    for (i, (spec, len_mod)) in spec_types.iter().enumerate() {
        let ptr = ptrs[i];
        match spec {
            'd' => {
                while pos < chars.len() && chars[pos].is_whitespace() {
                    pos += 1;
                }
                if pos >= chars.len() {
                    break;
                }
                let start = pos;
                if chars[pos] == '+' || chars[pos] == '-' {
                    pos += 1;
                }
                while pos < chars.len() && chars[pos].is_ascii_digit() {
                    pos += 1;
                }
                let token: String = chars[start..pos].iter().collect();
                if *len_mod >= 2 {
                    let value: i64 = token.parse().unwrap_or(0);
                    vm.store_i64(ptr, value as u64, &SourceLoc::default());
                } else {
                    let value: i32 = token.parse().unwrap_or(0);
                    vm.store_i32(ptr, value, &SourceLoc::default());
                }
                matched += 1;
            }
            'u' => {
                while pos < chars.len() && chars[pos].is_whitespace() {
                    pos += 1;
                }
                if pos >= chars.len() {
                    break;
                }
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
                    vm.store_i64(ptr, value, &SourceLoc::default());
                } else {
                    let value: u32 = token.parse().unwrap_or(0);
                    vm.store_i32(ptr, value as i32, &SourceLoc::default());
                }
                matched += 1;
            }
            'f' => {
                let (token, new_pos) = read_float_token(&chars, pos);
                pos = new_pos;
                if token.is_empty() {
                    break;
                }
                if *len_mod >= 1 {
                    let value: f64 = token.parse().unwrap_or(0.0);
                    vm.store_i64(ptr, value.to_bits(), &SourceLoc::default());
                } else {
                    let value: f32 = token.parse().unwrap_or(0.0);
                    vm.store_i32(ptr, value.to_bits() as i32, &SourceLoc::default());
                }
                matched += 1;
            }
            'c' => {
                if pos >= chars.len() {
                    break;
                }
                let ch = chars[pos];
                vm.store_i8(ptr, ch as i32, &SourceLoc::default());
                pos += 1;
                matched += 1;
            }
            's' => {
                while pos < chars.len() && chars[pos].is_whitespace() {
                    pos += 1;
                }
                if pos >= chars.len() {
                    break;
                }
                let start = pos;
                while pos < chars.len() && !chars[pos].is_whitespace() {
                    pos += 1;
                }
                let token: String = chars[start..pos].iter().collect();
                let bytes = token.as_bytes();
                for (j, &b) in bytes.iter().enumerate() {
                    vm.store_i8(ptr + j as u32, b as i32, &SourceLoc::default());
                }
                vm.store_i8(ptr + bytes.len() as u32, 0, &SourceLoc::default());
                matched += 1;
            }
            _ => {}
        }
    }
    vm.push(matched as u64);
}

// ========== VFS I/O extensions ==========
