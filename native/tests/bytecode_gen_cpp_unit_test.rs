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

#[test]
fn test_cpp_lambda_capture_by_value() {
    let src = r#"
#include <stdio.h>
int main() {
    int x = 5;
    auto f = [x](int y) { return x + y; };
    printf("%d\n", f(3));
    return 0;
}
"#;
    let (ret, outputs) = compile_and_run_cpp(src).expect("Compile/run failed");
    assert_eq!(ret, 0);
    assert_eq!(outputs, vec!["8"]);
}

#[test]
fn test_cpp_lambda_capture_by_reference() {
    let src = r#"
#include <stdio.h>
int main() {
    int x = 5;
    auto f = [&x](int y) { x = x + y; };
    f(3);
    printf("%d\n", x);
    return 0;
}
"#;
    let (ret, outputs) = compile_and_run_cpp(src).expect("Compile/run failed");
    assert_eq!(ret, 0);
    assert_eq!(outputs, vec!["8"]);
}

#[test]
fn test_cpp_delete_calls_dtor() {
    let src = r#"
#include <stdio.h>
class Flag {
public:
    int* p;
    Flag(int* ptr) { p = ptr; }
    ~Flag() { *p = 1; }
};
int main() {
    int flag = 0;
    Flag* f = new Flag(&flag);
    delete f;
    printf("%d\n", flag);
    return 0;
}
"#;
    let (ret, outputs) = compile_and_run_cpp(src).expect("Compile/run failed");
    assert_eq!(ret, 0);
    assert_eq!(outputs, vec!["1"]);
}

#[test]
fn test_cpp_range_for_vector() {
    let src = r#"
#include <stdio.h>
int main() {
    cide_vec_int v;
    cide_vec_init_int(&v);
    cide_vec_push_int(&v, 1);
    cide_vec_push_int(&v, 2);
    cide_vec_push_int(&v, 3);
    int sum = 0;
    for (auto x : v) {
        sum = sum + x;
    }
    printf("%d\n", sum);
    cide_vec_destroy_int(&v);
    return 0;
}
"#;
    let (ret, outputs) = compile_and_run_cpp(src).expect("Compile/run failed");
    assert_eq!(ret, 0);
    assert_eq!(outputs, vec!["6"]);
}

#[test]
fn test_cpp_range_for_string() {
    let src = r#"
#include <stdio.h>
int main() {
    cide_string s;
    cide_string_init(&s);
    cide_string_push_back(&s, 'a');
    cide_string_push_back(&s, 'b');
    cide_string_push_back(&s, 'c');
    for (auto c : s) {
        putchar(c);
    }
    putchar('\n');
    cide_string_destroy(&s);
    return 0;
}
"#;
    let (ret, outputs) = compile_and_run_cpp(src).expect("Compile/run failed");
    assert_eq!(ret, 0);
    assert_eq!(outputs, vec!["abc"]);
}

#[test]
fn test_cpp_container_vec_int() {
    let src = r#"
#include <stdio.h>
int main() {
    cide_vec_int v;
    cide_vec_init_int(&v);
    cide_vec_push_int(&v, 10);
    cide_vec_push_int(&v, 20);
    printf("%d\n", cide_vec_size_int(&v));
    printf("%d\n", cide_vec_get_int(&v, 0));
    printf("%d\n", cide_vec_get_int(&v, 1));
    cide_vec_pop_int(&v);
    printf("%d\n", cide_vec_size_int(&v));
    cide_vec_destroy_int(&v);
    return 0;
}
"#;
    let (ret, outputs) = compile_and_run_cpp(src).expect("Compile/run failed");
    assert_eq!(ret, 0);
    assert_eq!(outputs, vec!["2", "10", "20", "1"]);
}

#[test]
fn test_cpp_container_vec_float() {
    let src = r#"
#include <stdio.h>
int main() {
    cide_vec_float v;
    cide_vec_init_float(&v);
    cide_vec_push_float(&v, 15);
    cide_vec_push_float(&v, 25);
    printf("%.1f\n", cide_vec_get_float(&v, 0));
    printf("%.1f\n", cide_vec_get_float(&v, 1));
    cide_vec_destroy_float(&v);
    return 0;
}
"#;
    let (ret, outputs) = compile_and_run_cpp(src).expect("Compile/run failed");
    assert_eq!(ret, 0);
    assert_eq!(outputs, vec!["15.0", "25.0"]);
}

#[test]
fn test_cpp_container_string() {
    let src = r#"
#include <stdio.h>
int main() {
    cide_string s;
    cide_string_init(&s);
    cide_string_push_back(&s, 'h');
    cide_string_push_back(&s, 'i');
    printf("%d\n", cide_string_size(&s));
    printf("%c\n", cide_string_get(&s, 0));
    printf("%c\n", cide_string_get(&s, 1));
    cide_string_pop_back(&s);
    printf("%d\n", cide_string_size(&s));
    cide_string_destroy(&s);
    return 0;
}
"#;
    let (ret, outputs) = compile_and_run_cpp(src).expect("Compile/run failed");
    assert_eq!(ret, 0);
    assert_eq!(outputs, vec!["2", "h", "i", "1"]);
}

#[test]
fn test_cpp_builtin_layout_from_toml() {
    let layout = cide_native::compiler::cpp_frontend::builtin_layout::builtin_class_layout("cide_vec_int");
    assert!(layout.is_some(), "builtin_class_layout should return Some for cide_vec_int");
    let layout = layout.unwrap();
    assert_eq!(layout.size, 12, "cide_vec_int size should be 12");
    assert_eq!(layout.fields.len(), 3, "cide_vec_int should have 3 fields");
    let method_names: Vec<_> = layout.methods.iter().map(|m| m.name.as_str()).collect();
    assert!(method_names.contains(&"push_back"), "cide_vec_int should have push_back method");
    assert!(method_names.contains(&"size"), "cide_vec_int should have size method");
    assert!(method_names.contains(&"get"), "cide_vec_int should have get method");
}

#[test]
fn test_cpp_type_map_lookup() {
    assert_eq!(cide_native::compiler::cpp_frontend::type_map::cpp_type_to_cide("vector<int>"), Some("cide_vec_int"));
    assert_eq!(cide_native::compiler::cpp_frontend::type_map::cpp_type_to_cide("vector<float>"), Some("cide_vec_float"));
    assert_eq!(cide_native::compiler::cpp_frontend::type_map::cpp_type_to_cide("string"), Some("cide_string"));
    assert_eq!(
        cide_native::compiler::cpp_frontend::type_map::map_container_method("cide_vec_int", "push_back"),
        Some("cide_vec_push_int")
    );
    assert_eq!(
        cide_native::compiler::cpp_frontend::type_map::map_container_method("cide_string", "push_back"),
        Some("cide_string_push_back")
    );
}
