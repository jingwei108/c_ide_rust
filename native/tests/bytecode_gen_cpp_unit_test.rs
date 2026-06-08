use std::ffi::{c_char, CString};

fn compile_and_run_cpp(source: &str) -> Result<(i32, Vec<String>), String> {
    unsafe {
        let session = cide_native::capi::cide_session_create();
        if session.is_null() {
            return Err("Failed to create session".to_string());
        }

        let src = CString::new(source).map_err(|e| e.to_string())?;
        let fname = CString::new("main.cpp").map_err(|e| e.to_string())?;
        cide_native::capi::cide_compile_unit(
            session,
            fname.as_ptr() as *const c_char,
            src.as_ptr() as *const c_char,
        );

        let compile_ret = cide_native::capi::cide_compile_all(session);
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
            let out_str = String::from_utf8_lossy(&buf);
            for line in out_str.lines() {
                let trimmed = line.trim_matches('\0');
                if !trimmed.is_empty() && !trimmed.starts_with("程序运行完成") {
                    outputs.push(trimmed.to_string());
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

        if let Some(e) = runtime_err {
            if !e.is_empty() {
                return Err(format!("Runtime error: {}", e));
            }
        }

        Ok((run_ret, outputs))
    }
}

#[test]
fn test_cpp_class_field_and_method() {
    let src = r#"
#include <stdio.h>
class Point {
public:
    int x;
    int getX() { return this->x; }
};
int main() {
    Point p;
    p.x = 42;
    printf("%d\n", p.getX());
    return 0;
}
"#;
    let (ret, outputs) = compile_and_run_cpp(src).expect("Compile/run failed");
    assert_eq!(ret, 0);
    assert_eq!(outputs, vec!["42"]);
}

#[test]
fn test_cpp_new_delete_with_ctor() {
    let src = r#"
#include <stdio.h>
class Point {
public:
    int x;
    Point(int v) { this->x = v; }
};
int main() {
    Point* p = new Point(10);
    printf("%d\n", p->x);
    delete p;
    return 0;
}
"#;
    let (ret, outputs) = compile_and_run_cpp(src).expect("Compile/run failed");
    assert_eq!(ret, 0);
    assert_eq!(outputs, vec!["10"]);
}

#[test]
fn test_cpp_range_for_array() {
    let src = r#"
#include <stdio.h>
int main() {
    int arr[] = {1, 2, 3};
    int sum = 0;
    for (int x : arr) {
        sum = sum + x;
    }
    printf("%d\n", sum);
    return 0;
}
"#;
    let (ret, outputs) = compile_and_run_cpp(src).expect("Compile/run failed");
    assert_eq!(ret, 0);
    assert_eq!(outputs, vec!["6"]);
}

#[test]
fn test_cpp_virtual_call() {
    let src = r#"
#include <stdio.h>
class Base {
public:
    virtual int foo() { return 1; }
};
class Derived : public Base {
public:
    int foo() { return 2; }
};
int main() {
    Base* b = new Derived();
    printf("%d\n", b->foo());
    delete b;
    return 0;
}
"#;
    let (ret, outputs) = compile_and_run_cpp(src).expect("Compile/run failed");
    assert_eq!(ret, 0);
    assert_eq!(outputs, vec!["2"]);
}
