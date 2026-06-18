use crate::session::*;
use std::ffi::{c_char, c_int, CStr, CString};
use std::ptr;
use std::slice;

use crate::engine::compile_pipeline::run_multi_file_pipeline;
use crate::engine::session_ops::execute_run;

/// 将 C 字符串指针安全转换为 Rust &str。
///
/// 内部完成 null 检查，可作为 safe 函数调用。
fn cstr_to_str(s: *const c_char) -> Option<String> {
    if s.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(s).to_str().ok().map(|s| s.to_owned()) }
}

#[no_mangle]
pub extern "C" fn cide_session_create() -> *mut Session {
    Box::into_raw(Box::new(Session::default()))
}

#[no_mangle]
/// cide_session_destroy 的 C API 封装。
///
/// # Safety
/// - `s` 必须是由 `cide_session_create` 返回的有效 `Session` 指针，且未被 `cide_session_destroy` 销毁。
pub unsafe extern "C" fn cide_session_destroy(s: *mut Session) {
    if !s.is_null() {
        drop(Box::from_raw(s));
    }
}

#[no_mangle]
/// cide_compile 的 C API 封装。
///
/// # Safety
/// - `s` 必须是由 `cide_session_create` 返回的有效 `Session` 指针，且未被 `cide_session_destroy` 销毁。
/// - `source` 若非空，必须指向足够大的有效内存区域，供函数写入结果。
/// - `source` 若非空，必须指向以 null 结尾的有效 UTF-8 字符串。
pub unsafe extern "C" fn cide_compile(s: *mut Session, source: *const c_char) -> c_int {
    if s.is_null() || source.is_null() {
        return -1;
    }
    let session = &mut *s;
    let src = match cstr_to_str(source) {
        Some(v) => v,
        None => return -1,
    };
    session.compile.compile_units.clear();
    session.compile.compile_units.push(CompileUnit {
        filename: "main.c".to_string(),
        source: src.to_string(),
    });
    cide_compile_all(s)
}

#[no_mangle]
/// cide_compile_unit 的 C API 封装。
///
/// # Safety
/// - `s` 必须是由 `cide_session_create` 返回的有效 `Session` 指针，且未被 `cide_session_destroy` 销毁。
/// - `filename`, `source` 若非空，必须指向足够大的有效内存区域，供函数写入结果。
/// - `filename`, `source` 若非空，必须指向以 null 结尾的有效 UTF-8 字符串。
pub unsafe extern "C" fn cide_compile_unit(s: *mut Session, filename: *const c_char, source: *const c_char) -> c_int {
    if s.is_null() || filename.is_null() || source.is_null() {
        return -1;
    }
    let session = &mut *s;
    let fname = match cstr_to_str(filename) {
        Some(v) => v,
        None => return -1,
    };
    let src = match cstr_to_str(source) {
        Some(v) => v,
        None => return -1,
    };
    session.compile.compile_units.push(CompileUnit {
        filename: fname.to_string(),
        source: src.to_string(),
    });
    0
}

#[no_mangle]
/// cide_compile_all 的 C API 封装。
///
/// # Safety
/// - `s` 必须是由 `cide_session_create` 返回的有效 `Session` 指针，且未被 `cide_session_destroy` 销毁。
pub unsafe extern "C" fn cide_compile_all(s: *mut Session) -> c_int {
    if s.is_null() {
        return -1;
    }
    let session = &mut *s;

    let units = session.compile.compile_units.clone();
    if run_multi_file_pipeline(session, units, false).is_err() {
        return -1;
    }
    0
}

#[no_mangle]
/// cide_get_compile_errors 的 C API 封装。
///
/// # Safety
/// - `s` 必须是由 `cide_session_create` 返回的有效 `Session` 指针，且未被 `cide_session_destroy` 销毁。
pub unsafe extern "C" fn cide_get_compile_errors(s: *mut Session) -> *const c_char {
    if s.is_null() {
        return ptr::null();
    }
    let session = &mut *s;
    if session.compile.errors.is_empty() {
        return ptr::null();
    }
    match CString::new(session.compile.errors.clone()) {
        Ok(cstring) => {
            let ptr = cstring.as_ptr();
            session.compile.last_errors_cstring = Some(cstring);
            ptr
        }
        Err(_) => ptr::null(),
    }
}

#[no_mangle]
/// 设置命令行参数（供 `main(int argc, char *argv[])` 使用）。
///
/// # Safety
/// - `s` 必须是由 `cide_session_create` 返回的有效 `Session` 指针，且未被 `cide_session_destroy` 销毁。
/// - `argv` 必须是长度为 `argc` 的 C 字符串指针数组，每个指针指向有效的以 NUL 结尾的字符串。
/// - 字符串内容在调用期间必须保持有效；本函数会复制其内容到 Session 中。
pub unsafe extern "C" fn cide_set_argv(s: *mut Session, argc: c_int, argv: *const *const c_char) {
    if s.is_null() || argv.is_null() {
        return;
    }
    let session = &mut *s;
    let mut args = Vec::with_capacity(argc as usize);
    for i in 0..argc as isize {
        let ptr = *argv.offset(i);
        if ptr.is_null() {
            args.push(String::new());
        } else {
            args.push(std::ffi::CStr::from_ptr(ptr).to_string_lossy().to_string());
        }
    }
    session.runtime.argc = argc;
    session.runtime.argv = args;
}

#[no_mangle]
/// 设置输入模式：0 为交互式（默认），非 0 为批量模式。
/// 批量模式下 getchar 在输入耗尽后立即返回 EOF，不进入等待状态。
///
/// # Safety
/// - `s` 必须是由 `cide_session_create` 返回的有效 `Session` 指针，且未被 `cide_session_destroy` 销毁。
pub unsafe extern "C" fn cide_set_input_mode(s: *mut Session, is_batch: c_int) {
    if s.is_null() {
        return;
    }
    let session = &mut *s;
    session.runtime.input_mode = if is_batch != 0 {
        crate::session::InputMode::Batch
    } else {
        crate::session::InputMode::Interactive
    };
}

#[no_mangle]
/// cide_run 的 C API 封装。
///
/// # Safety
/// - `s` 必须是由 `cide_session_create` 返回的有效 `Session` 指针，且未被 `cide_session_destroy` 销毁。
pub unsafe extern "C" fn cide_run(s: *mut Session) -> c_int {
    if s.is_null() || !(*s).compile.compiled {
        if !s.is_null() {
            (*s).runtime.error = "程序尚未编译。请先编译代码。".to_string();
        }
        return -1;
    }
    let session = &mut *s;
    match execute_run(session) {
        Ok((_, true)) => 2,
        Ok((_, false)) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
/// cide_get_runtime_error 的 C API 封装。
///
/// # Safety
/// - `s` 必须是由 `cide_session_create` 返回的有效 `Session` 指针，且未被 `cide_session_destroy` 销毁。
pub unsafe extern "C" fn cide_get_runtime_error(s: *mut Session) -> *const c_char {
    if s.is_null() {
        return ptr::null();
    }
    let session = &mut *s;
    if session.runtime.error.is_empty() {
        return ptr::null();
    }
    match CString::new(session.runtime.error.clone()) {
        Ok(cstring) => {
            let ptr = cstring.as_ptr();
            session.runtime.last_error_cstring = Some(cstring);
            ptr
        }
        Err(_) => ptr::null(),
    }
}

#[no_mangle]
/// cide_set_input 的 C API 封装。
///
/// # Safety
/// - `s` 必须是由 `cide_session_create` 返回的有效 `Session` 指针，且未被 `cide_session_destroy` 销毁。
/// - `input` 若非空，必须指向足够大的有效内存区域，供函数写入结果。
/// - `input` 若非空，必须指向以 null 结尾的有效 UTF-8 字符串。
pub unsafe extern "C" fn cide_set_input(s: *mut Session, input: *const c_char) {
    if s.is_null() {
        return;
    }
    let session = &mut *s;
    session.runtime.input_lines.clear();
    session.runtime.input_index = 0;
    session.runtime.input_char_offset = 0;
    let input_str = match cstr_to_str(input) {
        Some(v) => v,
        None => return,
    };
    for line in input_str.lines() {
        session.runtime.input_lines.push(line.trim_end_matches('\r').to_string());
    }
}

#[no_mangle]
/// cide_is_waiting_input 的 C API 封装。
///
/// # Safety
/// - `s` 必须是由 `cide_session_create` 返回的有效 `Session` 指针，且未被 `cide_session_destroy` 销毁。
pub unsafe extern "C" fn cide_is_waiting_input(s: *mut Session) -> c_int {
    if s.is_null() {
        return 0;
    }
    if (*s).runtime.waiting_input {
        1
    } else {
        0
    }
}

#[no_mangle]
/// cide_provide_input_line 的 C API 封装。
///
/// # Safety
/// - `s` 必须是由 `cide_session_create` 返回的有效 `Session` 指针，且未被 `cide_session_destroy` 销毁。
/// - `line` 若非空，必须指向足够大的有效内存区域，供函数写入结果。
/// - `line` 若非空，必须指向以 null 结尾的有效 UTF-8 字符串。
pub unsafe extern "C" fn cide_provide_input_line(s: *mut Session, line: *const c_char) -> c_int {
    if s.is_null() {
        return -1;
    }
    let session = &mut *s;
    let line_str = match cstr_to_str(line) {
        Some(v) => v,
        None => return -1,
    };
    session.runtime.input_lines.push(line_str.to_string());
    session.runtime.waiting_input = false;
    if let Some(ref mut vm) = session.vm {
        vm.resume();
    }
    0
}

#[no_mangle]
/// cide_get_output_length 的 C API 封装。
///
/// # Safety
/// - `s` 必须是由 `cide_session_create` 返回的有效 `Session` 指针，且未被 `cide_session_destroy` 销毁。
pub unsafe extern "C" fn cide_get_output_length(s: *mut Session) -> c_int {
    if s.is_null() {
        return 0;
    }
    (*s).runtime.output_lines.iter().map(|l| l.len()).sum::<usize>() as c_int
}

#[no_mangle]
/// cide_get_output 的 C API 封装。
///
/// # Safety
/// - `s` 必须是由 `cide_session_create` 返回的有效 `Session` 指针，且未被 `cide_session_destroy` 销毁。
/// - `buf` 若非空，必须指向足够大的有效内存区域，供函数写入结果。
pub unsafe extern "C" fn cide_get_output(s: *mut Session, buf: *mut c_char, max_len: c_int) {
    if s.is_null() || buf.is_null() || max_len <= 0 {
        return;
    }
    let session = &*s;
    let all: String = session.runtime.output_lines.concat();
    let copy_len = all.len().min((max_len - 1) as usize);
    let slice = slice::from_raw_parts_mut(buf as *mut u8, copy_len);
    slice.copy_from_slice(&all.as_bytes()[..copy_len]);
    *buf.add(copy_len) = 0;
}

#[no_mangle]
/// 获取最近一次运行后的 JIT 统计信息。
///
/// # Safety
/// - `s` 必须是由 `cide_session_create` 返回的有效 `Session` 指针，且未被 `cide_session_destroy` 销毁。
/// - `traces_compiled` 与 `steps_accelerated` 若非空，必须指向有效的 `c_int` 内存。
pub unsafe extern "C" fn cide_get_jit_stats(
    s: *mut Session,
    traces_compiled: *mut c_int,
    steps_accelerated: *mut c_int,
) {
    if s.is_null() {
        return;
    }
    let session = &*s;
    let stats = session.vm.as_ref().map(|vm| vm.jit_stats()).cloned().unwrap_or_default();
    if !traces_compiled.is_null() {
        *traces_compiled = stats.traces_compiled as c_int;
    }
    if !steps_accelerated.is_null() {
        *steps_accelerated = stats.steps_accelerated as c_int;
    }
}
