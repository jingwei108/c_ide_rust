use std::ffi::{c_char, CString};

fn compile_and_run(source: &str) -> Result<(i32, Vec<String>), String> {
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

        // Collect output
        let mut outputs = Vec::new();
        let out_len = cide_native::capi::cide_get_output_length(session);
        if out_len > 0 {
            let mut buf = vec![0u8; out_len as usize + 1];
            cide_native::capi::cide_get_output(session, buf.as_mut_ptr() as *mut c_char, buf.len() as i32);
            let out_str = String::from_utf8_lossy(&buf);
            for line in out_str.lines() {
                if !line.is_empty() {
                    outputs.push(line.to_string());
                }
            }
        }

        // Check runtime error
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
fn test_e2e_hello_world() {
    let src = r#"
#include <stdio.h>
int main() {
    printf("Hello, World!");
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("Hello, World!")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_arithmetic_return() {
    let src = r#"
int main() {
    int a = 10;
    int b = 20;
    int c = a + b * 2;
    return c;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("50")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_if_else() {
    let src = r#"
#include <stdio.h>
int main() {
    int x = 5;
    if (x > 3) {
        printf("big");
    } else {
        printf("small");
    }
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("big")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_for_loop_sum() {
    let src = r#"
#include <stdio.h>
int main() {
    int sum = 0;
    for (int i = 0; i < 5; i = i + 1) {
        sum = sum + i;
    }
    printf("%d", sum);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("10")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_array_index() {
    let src = r#"
int main() {
    int arr[3];
    arr[0] = 1;
    arr[1] = 2;
    arr[2] = 3;
    return arr[1];
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("2")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_multidim_array() {
    let src = r#"
#include <stdio.h>
int main() {
    int arr[2][3];
    arr[1][2] = 42;
    printf("%d", arr[1][2]);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("42")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_struct_member() {
    let src = r#"
#include <stdio.h>
struct Point {
    int x;
    int y;
};
int main() {
    struct Point p;
    p.x = 3;
    p.y = 4;
    printf("%d", p.x + p.y);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("7")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_scanf_printf() {
    let src = r#"
#include <stdio.h>
int main() {
    int a;
    scanf("%d", &a);
    printf("got:%d", a);
    return 0;
}
"#;
    unsafe {
        let session = cide_native::capi::cide_session_create();
        let c_src = CString::new(src).unwrap();
        let compile_ret = cide_native::capi::cide_compile(session, c_src.as_ptr() as *const c_char);
        assert_eq!(compile_ret, 0);

        let input = CString::new("42").unwrap();
        cide_native::capi::cide_set_input(session, input.as_ptr() as *const c_char);

        let run_ret = cide_native::capi::cide_run(session);
        assert_eq!(run_ret, 0);

        let out_len = cide_native::capi::cide_get_output_length(session);
        let mut buf = vec![0u8; out_len as usize + 1];
        cide_native::capi::cide_get_output(session, buf.as_mut_ptr() as *mut c_char, buf.len() as i32);
        let out_str = String::from_utf8_lossy(&buf);
        assert!(out_str.contains("got:42"), "Output: {}", out_str);

        cide_native::capi::cide_session_destroy(session);
    }
}

#[test]
fn test_e2e_malloc_free() {
    let src = r#"
#include <stdio.h>
#include <stdlib.h>
int main() {
    int *p = malloc(sizeof(int));
    *p = 123;
    printf("%d", *p);
    free(p);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("123")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_switch_case() {
    let src = r#"
#include <stdio.h>
int main() {
    int x = 2;
    switch (x) {
        case 1: printf("one"); break;
        case 2: printf("two"); break;
        default: printf("other"); break;
    }
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("two")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_define_macro() {
    let src = r#"
#include <stdio.h>
#define N 100
int main() {
    printf("%d", N);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("100")), "Outputs: {:?}", outputs);
}


#[test]
fn test_e2e_pointer_deref() {
    let src = r#"
#include <stdio.h>
int main() {
    int x = 42;
    int *p = &x;
    printf("%d", *p);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("42")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_compound_assign() {
    let src = r#"
#include <stdio.h>
int main() {
    int a = 10;
    a += 5;
    a -= 3;
    a *= 2;
    a /= 4;
    printf("%d", a);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    // (10 + 5 - 3) * 2 / 4 = 12 * 2 / 4 = 24 / 4 = 6
    assert!(outputs.iter().any(|l| l.contains('6')), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_array_bounds_trap() {
    let src = r#"
int main() {
    int arr[3];
    arr[0] = 1;
    arr[5] = 99;
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_err(), "Expected runtime bounds error, got: {:?}", result);
    let err = result.unwrap_err();
    assert!(err.contains("数组越界") || err.contains("bounds"), "Error: {}", err);
}

#[test]
fn test_e2e_while_loop() {
    let src = r#"
#include <stdio.h>
int main() {
    int i = 0;
    int sum = 0;
    while (i < 5) {
        sum = sum + i;
        i = i + 1;
    }
    printf("%d", sum);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("10")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_function_call() {
    let src = r#"
#include <stdio.h>
int add(int a, int b) {
    return a + b;
}
int main() {
    int r = add(3, 4);
    printf("%d", r);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("7")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_recursive_factorial() {
    let src = r#"
#include <stdio.h>
int fact(int n) {
    if (n <= 1) return 1;
    return n * fact(n - 1);
}
int main() {
    printf("%d", fact(5));
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("120")), "Outputs: {:?}", outputs);
}
