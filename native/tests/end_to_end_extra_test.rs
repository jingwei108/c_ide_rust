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

// ============================================================================
// Control Flow
// ============================================================================

#[test]
fn test_e2e_do_while_loop() {
    let src = r#"
#include <stdio.h>
int main() {
    int i = 0;
    int sum = 0;
    do {
        sum = sum + i;
        i = i + 1;
    } while (i < 5);
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
fn test_e2e_break_continue() {
    let src = r#"
#include <stdio.h>
int main() {
    int sum = 0;
    for (int i = 0; i < 10; i = i + 1) {
        if (i == 3) continue;
        if (i == 7) break;
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
    // 0+1+2+4+5+6 = 18
    assert!(outputs.iter().any(|l| l.contains("18")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_nested_if() {
    let src = r#"
#include <stdio.h>
int main() {
    int x = 5;
    int y = 3;
    if (x > 3) {
        if (y > 2) {
            printf("both");
        } else {
            printf("x only");
        }
    } else {
        printf("none");
    }
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("both")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_logical_operators() {
    let src = r#"
#include <stdio.h>
int main() {
    int a = 1;
    int b = 0;
    if (a && !b) printf("ok1");
    if (a || b) printf("ok2");
    if (!(a && b)) printf("ok3");
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("ok1")), "Outputs: {:?}", outputs);
    assert!(outputs.iter().any(|l| l.contains("ok2")), "Outputs: {:?}", outputs);
    assert!(outputs.iter().any(|l| l.contains("ok3")), "Outputs: {:?}", outputs);
}

// ============================================================================
// Data & Types
// ============================================================================

#[test]
fn test_e2e_global_variables() {
    let src = r#"
#include <stdio.h>
int g = 42;
int main() {
    printf("%d", g);
    g = 100;
    printf("%d", g);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("42")), "Outputs: {:?}", outputs);
    assert!(outputs.iter().any(|l| l.contains("100")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_string_char_array() {
    let src = r#"
#include <stdio.h>
int main() {
    char s[6] = "hello";
    printf("%s", s);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("hello")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_multi_variable_decl() {
    let src = r#"
#include <stdio.h>
int main() {
    int a = 1, b = 2, c = 3;
    printf("%d", a + b + c);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("6")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_negative_numbers() {
    let src = r#"
#include <stdio.h>
int main() {
    int a = -5;
    int b = -a;
    printf("%d %d", a, b);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("-5")), "Outputs: {:?}", outputs);
    assert!(outputs.iter().any(|l| l.contains("5")), "Outputs: {:?}", outputs);
}

// ============================================================================
// Functions
// ============================================================================

#[test]
fn test_e2e_multi_arg_function() {
    let src = r#"
#include <stdio.h>
int add3(int a, int b, int c) {
    return a + b + c;
}
int main() {
    printf("%d", add3(1, 2, 3));
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("6")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_array_as_function_arg() {
    let src = r#"
#include <stdio.h>
int sum(int arr[], int n) {
    int s = 0;
    for (int i = 0; i < n; i = i + 1) {
        s = s + arr[i];
    }
    return s;
}
int main() {
    int a[3] = {1, 2, 3};
    printf("%d", sum(a, 3));
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("6")), "Outputs: {:?}", outputs);
}

// ============================================================================
// Pointers
// ============================================================================

#[test]
fn test_e2e_pointer_arithmetic() {
    let src = r#"
#include <stdio.h>
int main() {
    int arr[3] = {10, 20, 30};
    int *p = arr;
    printf("%d", *(p + 1));
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("20")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_null_pointer_trap() {
    let src = r#"
int main() {
    int *p = 0;
    return *p;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_err(), "Expected null pointer trap, got: {:?}", result);
    let err = result.unwrap_err();
    assert!(
        err.contains("NULL") || err.contains("空指针") || err.contains("null"),
        "Error: {}",
        err
    );
}

// ============================================================================
// Runtime Traps
// ============================================================================

#[test]
fn test_e2e_div_by_zero_trap() {
    let src = r#"
int main() {
    int a = 10;
    int b = 0;
    return a / b;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_err(), "Expected div-by-zero trap, got: {:?}", result);
    let err = result.unwrap_err();
    assert!(
        err.contains("除零") || err.contains("div") || err.contains("zero"),
        "Error: {}",
        err
    );
}

#[test]
fn test_e2e_infinite_loop_trap() {
    let src = r#"
int main() {
    while (1) {}
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_err(), "Expected infinite loop trap, got: {:?}", result);
    let err = result.unwrap_err();
    assert!(
        err.contains("无限循环") || err.contains("步数超过限制") || err.contains("infinite"),
        "Error: {}",
        err
    );
}

// ============================================================================
// Arrays & Structs
// ============================================================================

#[test]
fn test_e2e_multidim_array_init() {
    let src = r#"
#include <stdio.h>
int main() {
    int arr[2][3] = {{1, 2, 3}, {4, 5, 6}};
    printf("%d", arr[1][2]);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("6")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_struct_init_list() {
    let src = r#"
#include <stdio.h>
struct Point {
    int x;
    int y;
};
int main() {
    struct Point p = {3, 4};
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

// ============================================================================
// Enum / Typedef
// ============================================================================

#[test]
fn test_e2e_enum() {
    let src = r#"
#include <stdio.h>
enum Color { RED, GREEN, BLUE };
int main() {
    enum Color c = GREEN;
    printf("%d", c);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("1")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_typedef() {
    let src = r#"
#include <stdio.h>
typedef int Integer;
int main() {
    Integer a = 42;
    printf("%d", a);
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
fn test_e2e_sizeof() {
    let src = r#"
#include <stdio.h>
struct Point {
    int x;
    int y;
};
int main() {
    int arr[5];
    struct Point p;
    int *ptr;
    printf("%d ", sizeof(int));
    printf("%d ", sizeof(char));
    printf("%d ", sizeof(struct Point));
    printf("%d ", sizeof(arr));
    printf("%d ", sizeof(ptr));
    printf("%d", sizeof(p));
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    let output = outputs.join(" ");
    assert!(output.contains("4 1 8 20 4 8"), "Outputs: {:?}", outputs);
}
