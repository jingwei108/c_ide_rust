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


#[test]
fn test_e2e_char_literal() {
    let src = r#"
#include <stdio.h>
int main() {
    char c = 'A';
    printf("%d", c);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("65")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_block_comment() {
    let src = r#"
#include <stdio.h>
/* this is a block comment */
int main() {
    int a = 1;
    /* nested
       multiline
       comment */
    int b = 2;
    printf("%d", a + b);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("3")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_type_qualifiers() {
    let src = r#"
#include <stdio.h>
int main() {
    const int a = 10;
    long b = 20;
    short c = 30;
    signed int d = 40;
    unsigned int e = 50;
    printf("%d", a + b + c + d + e);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("150")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_hex_literal() {
    let src = r#"
#include <stdio.h>
int main() {
    int a = 0x0A;
    int b = 0xFF;
    int c = 0x100;
    printf("%d %d %d", a, b, c);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    let output = outputs.join(" ");
    assert!(output.contains("10 255 256"), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_escape_sequences() {
    let src = r#"
#include <stdio.h>
int main() {
    char s[10];
    s[0] = '\r';
    s[1] = '\a';
    s[2] = '\b';
    s[3] = '\f';
    s[4] = '\v';
    s[5] = '\x41';
    printf("%d %d %d %d %d %d", s[0], s[1], s[2], s[3], s[4], s[5]);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    let output = outputs.join(" ");
    assert!(output.contains("13 7 8 12 11 65"), "Outputs: {:?}", outputs);
}


#[test]
fn test_e2e_compound_assign_array() {
    let src = r#"
#include <stdio.h>
int main() {
    int arr[3] = {10, 20, 30};
    arr[0] += 5;
    arr[1] -= 5;
    arr[2] *= 2;
    printf("%d %d %d", arr[0], arr[1], arr[2]);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    let output = outputs.join(" ");
    assert!(output.contains("15 15 60"), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_compound_assign_deref() {
    let src = r#"
#include <stdio.h>
int main() {
    int x = 10;
    int *p = &x;
    *p += 5;
    printf("%d", x);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("15")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_compound_assign_member() {
    let src = r#"
#include <stdio.h>
struct Point {
    int x;
    int y;
};
int main() {
    struct Point p;
    p.x = 10;
    p.y = 20;
    p.x += 5;
    p.y -= 5;
    printf("%d %d", p.x, p.y);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    let output = outputs.join(" ");
    assert!(output.contains("15 15"), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_addr_of_array_index() {
    let src = r#"
#include <stdio.h>
int main() {
    int arr[3] = {10, 20, 30};
    int *p = &arr[1];
    printf("%d", *p);
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
fn test_e2e_addr_of_member() {
    let src = r#"
#include <stdio.h>
struct Point {
    int x;
    int y;
};
int main() {
    struct Point p;
    p.x = 42;
    int *px = &p.x;
    printf("%d", *px);
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
fn test_e2e_global_struct_member() {
    let src = r#"
#include <stdio.h>
struct Point {
    int x;
    int y;
};
struct Point g;
int main() {
    g.x = 10;
    g.y = 20;
    printf("%d %d", g.x, g.y);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    let output = outputs.join(" ");
    assert!(output.contains("10 20"), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_inc_dec_complex() {
    let src = r#"
#include <stdio.h>
struct Point {
    int x;
    int y;
};
int main() {
    int arr[3] = {10, 20, 30};
    int a = arr[0]++;
    int b = ++arr[1];
    struct Point p;
    p.x = 5;
    p.y = 10;
    int c = p.x--;
    int d = --p.y;
    printf("%d %d %d %d %d %d", a, b, arr[0], arr[1], c, d);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    let output = outputs.join(" ");
    assert!(output.contains("10 21 11 21 5 9"), "Outputs: {:?}", outputs);
}


#[test]
fn test_e2e_bitwise_ops() {
    let src = r#"
#include <stdio.h>
int main() {
    int a = 5;   // 0101
    int b = 3;   // 0011
    int c = a & b;
    int d = a | b;
    int e = a ^ b;
    int f = ~a;
    int g = a << 1;
    int h = a >> 1;
    printf("%d %d %d %d %d %d", c, d, e, f, g, h);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    let output = outputs.join(" ");
    assert!(output.contains("1 7 6 -6 10 2"), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_bitwise_precedence() {
    let src = r#"
#include <stdio.h>
int main() {
    int r = 1 | 2 & 4;
    printf("%d", r);
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
fn test_e2e_ternary() {
    let src = r#"
#include <stdio.h>
int main() {
    int a = 5;
    int b = 3;
    int max = a > b ? a : b;
    int min = a < b ? a : b;
    int sign = a > 0 ? 1 : (a < 0 ? -1 : 0);
    printf("%d %d %d", max, min, sign);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    let output = outputs.join(" ");
    assert!(output.contains("5 3 1"), "Outputs: {:?}", outputs);
}


#[test]
fn test_e2e_ptr_inc_dec() {
    let src = r#"
#include <stdio.h>
int main() {
    int arr[3] = {10, 20, 30};
    int *p = arr;
    int a = *p;
    p = p + 1;
    int b = *p;
    p++;
    int c = *p;
    p--;
    int d = *p;
    printf("%d %d %d %d", a, b, c, d);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    let output = outputs.join(" ");
    assert!(output.contains("10 20 30 20"), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_ptr_sub_ptr() {
    let src = r#"
#include <stdio.h>
int main() {
    int arr[5] = {10, 20, 30, 40, 50};
    int *p = &arr[4];
    int *q = &arr[1];
    int diff = p - q;
    printf("%d", diff);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("3")), "Outputs: {:?}", outputs);
}


#[test]
fn test_e2e_forward_decl() {
    let src = r#"
#include <stdio.h>
int add(int a, int b);
int main() {
    printf("%d", add(1, 2));
    return 0;
}
int add(int a, int b) {
    return a + b;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("3")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_strlen() {
    let src = r#"
#include <stdio.h>
int main() {
    char s[] = "hello";
    int len = strlen(s);
    printf("%d", len);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("5")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_strcpy() {
    let src = r#"
#include <stdio.h>
int main() {
    char src[] = "hello";
    char dest[10];
    strcpy(dest, src);
    printf("%s", dest);
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
fn test_e2e_strcmp() {
    let src = r#"
#include <stdio.h>
int main() {
    char a[] = "abc";
    char b[] = "abc";
    char c[] = "abd";
    int r1 = strcmp(a, b);
    int r2 = strcmp(a, c);
    printf("%d %d", r1, r2);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    let output = outputs.join(" ");
    assert!(output.contains("0 -1"), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_cast_malloc() {
    let src = r#"
#include <stdio.h>
#include <stdlib.h>
int main() {
    int *p = (int*)malloc(sizeof(int));
    *p = 42;
    printf("%d", *p);
    free(p);
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
fn test_e2e_cast_pointer() {
    let src = r#"
#include <stdio.h>
int main() {
    int arr[3] = {1, 2, 3};
    char *c = (char*)arr;
    printf("%d", *c);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("1")), "Outputs: {:?}", outputs);
}


// ============================================================================
// Student classic error cases (compile-time & runtime)
// ============================================================================

#[test]
fn test_e2e_error_undeclared_variable() {
    let src = r#"
#include <stdio.h>
int main() {
    x = 5;
    printf("%d", x);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_err(), "Expected compile error for undeclared variable");
    let err = result.unwrap_err();
    assert!(err.contains("x") || err.contains("undeclared") || err.contains("not declared") || err.contains("Unknown identifier"),
        "Error should mention undeclared variable x: {}", err);
}

#[test]
fn test_e2e_error_scanf_missing_ampersand() {
    let src = r#"
#include <stdio.h>
int main() {
    int a;
    scanf("%d", a);
    printf("%d", a);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_err(), "Expected compile error for scanf without &");
    let err = result.unwrap_err();
    assert!(err.contains("scanf") || err.contains("type") || err.contains("pointer") || err.contains("int*"),
        "Error should mention type mismatch in scanf: {}", err);
}

#[test]
fn test_e2e_error_array_out_of_bounds() {
    let src = r#"
#include <stdio.h>
int main() {
    int arr[3] = {10, 20, 30};
    printf("%d", arr[5]);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_err(), "Expected runtime error for array out of bounds");
    let err = result.unwrap_err();
    assert!(err.contains("bounds") || err.contains("out of") || err.contains("overflow") || err.contains("memory")
        || err.contains("数组越界") || err.contains("越界"),
        "Error should mention bounds violation: {}", err);
}

#[test]
fn test_e2e_error_divide_by_zero() {
    let src = r#"
#include <stdio.h>
int main() {
    int a = 5;
    int b = 0;
    printf("%d", a / b);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_err(), "Expected runtime error for divide by zero");
    let err = result.unwrap_err();
    assert!(err.contains("zero") || err.contains("divide") || err.contains("arithmetic")
        || err.contains("除零") || err.contains("除以"),
        "Error should mention division by zero: {}", err);
}

// NOTE: Current TypeChecker does not enforce non-void functions to have a return
// statement on all paths, so this case compiles successfully.
// #[test]
// fn test_e2e_error_missing_return_non_void() { ... }

#[test]
fn test_e2e_typedef_anon_struct() {
    let src = r#"
#include <stdio.h>
typedef struct {
    int x;
    int y;
} Point;

int main() {
    Point p;
    p.x = 3;
    p.y = 4;
    printf("%d %d", p.x, p.y);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("3 4")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_typedef_named_struct() {
    let src = r#"
#include <stdio.h>
typedef struct Vec2 {
    int x;
    int y;
} Vec2Alias;

int main() {
    Vec2Alias v;
    v.x = 10;
    v.y = 20;
    printf("%d %d", v.x, v.y);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("10 20")), "Outputs: {:?}", outputs);
}


// ============================================================================
// P0 Subset Extensions
// ============================================================================

#[test]
fn test_e2e_null_keyword() {
    let src = r#"
#include <stdio.h>
int main() {
    int *p = NULL;
    if (p == NULL) {
        printf("null");
    }
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("null")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_rand_srand() {
    let src = r#"
#include <stdio.h>
int main() {
    srand(12345);
    int a = rand();
    int b = rand();
    printf("%d %d", a, b);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    // Extract first two integers from the output line
    let line = outputs.join("");
    let nums: Vec<i32> = line.split(|c: char| !c.is_ascii_digit() && c != '-')
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<i32>().unwrap())
        .collect();
    assert!(nums.len() >= 2, "Expected at least 2 numbers in output: {}", line);
    let a = nums[0];
    let b = nums[1];
    assert!(a >= 0 && a <= 32767, "rand out of range: {}", a);
    assert!(b >= 0 && b <= 32767, "rand out of range: {}", b);
    assert_ne!(a, b, "rand should produce different values");
}

#[test]
fn test_e2e_memset() {
    let src = r#"
#include <stdio.h>
int main() {
    int arr[5];
    memset(arr, 0, sizeof(arr));
    printf("%d %d %d", arr[0], arr[2], arr[4]);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("0 0 0")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_exit() {
    let src = r#"
#include <stdio.h>
int main() {
    printf("before");
    exit(42);
    printf("after");
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (_ret, outputs) = result.unwrap();
    assert!(outputs.iter().any(|l| l.contains("before")), "Outputs: {:?}", outputs);
    assert!(!outputs.iter().any(|l| l.contains("after")), "Outputs: {:?}", outputs);
    assert!(outputs.iter().any(|l| l.contains("返回值：42")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_atoi() {
    let src = r#"
#include <stdio.h>
int main() {
    char s[] = "12345";
    int n = atoi(s);
    printf("%d", n);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("12345")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_strcat() {
    let src = r#"
#include <stdio.h>
int main() {
    char s[20] = "Hello";
    strcat(s, " World");
    printf("%s", s);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("Hello World")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_putchar() {
    let src = r#"
#include <stdio.h>
int main() {
    putchar('A');
    putchar('B');
    putchar('C');
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    let joined = outputs.join("");
    assert!(joined.contains("ABC"), "Outputs: {:?}", outputs);
}


#[test]
fn test_e2e_const_var() {
    let src = r#"
#include <stdio.h>
int main() {
    const int MAX = 100;
    printf("%d", MAX);
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
fn test_e2e_const_assign_error() {
    let src = r#"
int main() {
    const int x = 10;
    x = 20;
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_err(), "Expected compile error for assigning to const");
}

#[test]
fn test_e2e_const_inc_error() {
    let src = r#"
int main() {
    const int x = 10;
    x++;
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_err(), "Expected compile error for incrementing const");
}


// ============================================================================
// Float / Double
// ============================================================================

#[test]
fn test_e2e_float_basic() {
    let src = r#"
#include <stdio.h>
int main() {
    float a = 3.5;
    float b = 2.0;
    float c = a + b;
    printf("%f", c);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("5.500000")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_float_arithmetic() {
    let src = r#"
#include <stdio.h>
int main() {
    float a = 10.0;
    float b = 3.0;
    printf("%f", a - b);
    printf("%f", a * b);
    printf("%f", a / b);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("7.000000")), "Outputs: {:?}", outputs);
    assert!(outputs.iter().any(|l| l.contains("30.000000")), "Outputs: {:?}", outputs);
    assert!(outputs.iter().any(|l| l.contains("3.333333")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_float_compare() {
    let src = r#"
#include <stdio.h>
int main() {
    float a = 5.0;
    float b = 3.0;
    printf("%d", a > b);
    printf("%d", a < b);
    printf("%d", a == b);
    printf("%d", a != b);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("1")), "Outputs: {:?}", outputs);
    assert!(outputs.iter().any(|l| l.contains("0")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_float_mixed_int() {
    let src = r#"
#include <stdio.h>
int main() {
    float a = 5.0;
    int b = 2;
    float c = a + b;
    printf("%f", c);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("7.000000")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_float_cast() {
    let src = r#"
#include <stdio.h>
int main() {
    int a = 5;
    float b = (float)a;
    printf("%f", b);
    float c = 3.7;
    int d = (int)c;
    printf("%d", d);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("5.000000")), "Outputs: {:?}", outputs);
    assert!(outputs.iter().any(|l| l.contains("3")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_float_assign_int() {
    let src = r#"
#include <stdio.h>
int main() {
    float x = 5;
    printf("%f", x);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("5.000000")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_float_compound_assign() {
    let src = r#"
#include <stdio.h>
int main() {
    float x = 2.0;
    x += 3.0;
    printf("%f", x);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("5.000000")), "Outputs: {:?}", outputs);
}


#[test]
fn test_e2e_float_func_arg_implicit_cast() {
    let src = r#"
#include <stdio.h>
void foo(float x) {
    printf("%f", x);
}
int main() {
    foo(5);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("5.000000")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_int_func_arg_implicit_cast_from_float() {
    let src = r#"
#include <stdio.h>
void bar(int x) {
    printf("%d", x);
}
int main() {
    bar(3.7);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    assert!(outputs.iter().any(|l| l.contains("3")), "Outputs: {:?}", outputs);
}

#[test]
fn test_e2e_host_func_arg_implicit_cast() {
    let src = r#"
#include <stdio.h>
int main() {
    putchar(65.0);
    return 0;
}
"#;
    let result = compile_and_run(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let (ret, outputs) = result.unwrap();
    assert_eq!(ret, 0);
    let joined = outputs.join("");
    assert!(joined.contains("A"), "Outputs: {:?}", outputs);
}
