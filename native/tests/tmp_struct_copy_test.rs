use std::ffi::{c_char, CString};

fn compile_and_run(source: &str) -> Result<(i32, Vec<String>, Option<String>), String> {
    unsafe {
        let session = cide_native::capi::cide_session_create();
        if session.is_null() {
            return Err("Failed to create session".to_string());
        }
        let src = CString::new(source).map_err(|e| e.to_string())?;
        let compile_ret = cide_native::capi::cide_compile(session, src.as_ptr() as *const c_char);
        if compile_ret != 0 {
            let err_ptr = cide_native::capi::cide_get_compile_errors(session);
            let err_msg = if err_ptr.is_null() {
                "Unknown compile error".to_string()
            } else {
                std::ffi::CStr::from_ptr(err_ptr).to_string_lossy().to_string()
            };
            cide_native::capi::cide_session_destroy(session);
            return Err(err_msg);
        }
        let run_ret = cide_native::capi::cide_run(session);
        let mut outputs = Vec::new();
        let out_len = cide_native::capi::cide_get_output_length(session);
        if out_len > 0 {
            let mut buf = vec![0u8; out_len as usize + 1];
            cide_native::capi::cide_get_output(session, buf.as_mut_ptr() as *mut c_char, buf.len() as i32);
            let out_str = String::from_utf8_lossy(&buf[..out_len as usize]);
            for line in out_str.lines() {
                if !line.is_empty() {
                    outputs.push(line.to_string());
                }
            }
        }
        let err_ptr = cide_native::capi::cide_get_runtime_error(session);
        let runtime_err = if err_ptr.is_null() {
            None
        } else {
            Some(std::ffi::CStr::from_ptr(err_ptr).to_string_lossy().to_string())
        };
        cide_native::capi::cide_session_destroy(session);
        Ok((run_ret, outputs, runtime_err))
    }
}

fn filter_outputs(outputs: Vec<String>) -> Vec<String> {
    outputs.into_iter().filter(|s| !s.starts_with("程序运行完成")).collect()
}

#[test]
fn test_struct_array_copy() {
    let src = r#"
#include <stdio.h>
typedef struct { int id; int score; } Student;
int main() {
    Student s[2];
    s[0].id = 1; s[0].score = 10;
    s[1].id = 2; s[1].score = 20;
    s[0] = s[1];
    printf("%d %d\n", s[0].id, s[0].score);
    return 0;
}
"#;
    let result = compile_and_run(src);
    println!("Result: {:?}", result);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs, err) = result.unwrap();
    println!("ret={}, outputs={:?}, err={:?}", ret, outputs, err);
    assert_eq!(ret, 0);
    let out = filter_outputs(outputs);
    assert_eq!(out, vec!["2 20"]);
}

#[test]
fn test_struct_local_copy() {
    let src = r#"
#include <stdio.h>
typedef struct { int id; int score; } Student;
int main() {
    Student a;
    a.id = 1; a.score = 10;
    Student b;
    b = a;
    printf("%d %d\n", b.id, b.score);
    return 0;
}
"#;
    let result = compile_and_run(src);
    println!("Result: {:?}", result);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs, err) = result.unwrap();
    println!("ret={}, outputs={:?}, err={:?}", ret, outputs, err);
    assert_eq!(ret, 0);
    let out = filter_outputs(outputs);
    assert_eq!(out, vec!["1 10"]);
}
