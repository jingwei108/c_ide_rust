//! 崩溃止血回归测试（2026-09-06 代码审查报告第一批修复）。
//!
//! 覆盖审查报告 P0 修复项，全部用例来自实测复现（release 版 cide_cli 验证）：
//! - F-P0-1：深嵌套/长注释/多星号曾导致编译器栈溢出（SIGSEGV，无法被 catch_unwind 捕获）
//! - T-P0-8：自含 struct 曾导致 TypeChecker/compute_type_size 无限递归栈溢出
//! - V-P0-1/2：INT_MIN % -1、LLONG_MIN / -1 等曾直接 panic
//! - V-P0-5：qsort/bsearch 恶意 nmemb*size 曾 wrap 绕过边界检查导致 OOM
//! - E-P1-6：apply_fix 对含中文的行按字节切片曾 panic
//!
//! 防线哲学：测试不是为了标榜通过率，而是诚实地发现自己可能存在的问题。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::ffi::{c_char, CString};

/// 与 cide_e2e.rs 相同的 C API 驱动方式（独立复制以避免跨测试文件依赖）。
fn compile_and_run(source: &str) -> Result<(i32, Vec<String>), String> {
    unsafe {
        let session = cide_native::capi::cide_session_create();
        if session.is_null() {
            return Err("Failed to create session".to_string());
        }

        let fname = CString::new("main.c").map_err(|e| e.to_string())?;
        let src = CString::new(source).map_err(|e| e.to_string())?;
        cide_native::capi::cide_compile_unit(session, fname.as_ptr() as *const c_char, src.as_ptr() as *const c_char);
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
            let out_str = String::from_utf8_lossy(&buf[..out_len as usize]);
            for line in out_str.lines() {
                outputs.push(line.to_string());
            }
        }

        let err_ptr = cide_native::capi::cide_get_runtime_error(session);
        let runtime_err = if err_ptr.is_null() {
            None
        } else {
            Some(std::ffi::CStr::from_ptr(err_ptr).to_string_lossy().to_string())
        };

        let _ = run_ret;
        cide_native::capi::cide_session_destroy(session);

        if let Some(e) = runtime_err {
            if !e.is_empty() {
                return Err(format!("Runtime error: {}", e));
            }
        }
        Ok((run_ret, outputs))
    }
}

// ===================== V-P0-1 / V-P0-2：算术溢出转教学 trap =====================

#[test]
fn test_int_min_mod_minus_one_traps_not_panics() {
    // Rust 的 % 在 i32::MIN % -1 时溢出 panic（debug 与 release 一致），必须先拦截。
    // 修复前：release 下 `attempt to calculate the remainder with overflow` panic。
    let src = "#include <stdio.h>\n#include <limits.h>\nint main(){ int a = INT_MIN; printf(\"%d\\n\", a % -1); return 0; }\n";
    let result = compile_and_run(src);
    match result {
        Err(msg) => assert!(msg.contains("取模溢出"), "应报告取模溢出教学诊断，实际: {}", msg),
        Ok(_) => panic!("INT_MIN % -1 应产生运行错误而非成功"),
    }
}

#[test]
fn test_llong_min_div_mod_neg_one_traps_not_panics() {
    for (name, expr) in [("div", "/ -1"), ("mod", "% -1"), ("neg", "-a")] {
        let src = format!(
            "#include <stdio.h>\nint main(){{ long long a = -9223372036854775807LL; a = a - 1; printf(\"%lld\\n\", {}); return 0; }}\n",
            if expr == "-a" { "-a".to_string() } else { format!("a {}", expr) }
        );
        let _ = name;
        match compile_and_run(&src) {
            Err(msg) => assert!(msg.contains("溢出"), "long long {} 应报告溢出教学诊断，实际: {}", name, msg),
            Ok(_) => panic!("LLONG_MIN {} 应产生运行错误而非成功", name),
        }
    }
}

// ===================== F-P0-1：递归深度防护 =====================

#[test]
fn test_massive_line_comments_no_stack_overflow() {
    // 修复前：2 万行 `//c` 注释使 lexer next_token 递归 2 万层，栈溢出 SIGSEGV。
    let mut src = String::from("int main(){ return 0; }\n");
    for _ in 0..20000 {
        src.push_str("//c\n");
    }
    let result = compile_and_run(&src);
    assert!(result.is_ok(), "大量行注释不应影响编译: {:?}", result.err());
}

#[test]
fn test_deep_brace_nesting_reports_error_no_crash() {
    // 修复前：6 万层 `{{{{` 使 parse_block 递归栈溢出。
    let mut src = String::from("int main()");
    src.push_str(&"{".repeat(60000));
    src.push_str(&"}".repeat(60000));
    src.push_str(" return 0; }\n");
    let result = compile_and_run(&src);
    assert!(result.is_err(), "过深花括号嵌套应编译失败");
}

#[test]
fn test_deep_paren_nesting_reports_error_no_crash() {
    // 修复前：5 万层 `((((` 使 parse_primary 递归栈溢出。
    let mut src = String::from("int main(){ int x = ");
    src.push_str(&"(".repeat(50000));
    src.push('1');
    src.push_str(&")".repeat(50000));
    src.push_str("; return 0; }\n");
    let result = compile_and_run(&src);
    assert!(result.is_err(), "过深括号嵌套应编译失败");
}

#[test]
fn test_excessive_pointer_stars_report_error_no_crash() {
    // 修复前：10 万个 `*` 构造同深度嵌套 DeclaratorNode，后续遍历栈溢出。
    let mut src = String::from("int main(){ int ");
    src.push_str(&"*".repeat(100000));
    src.push_str(" p; return 0; }\n");
    let result = compile_and_run(&src);
    assert!(result.is_err(), "过深指针层级应编译失败");
}

#[test]
fn test_legal_deep_expression_still_compiles() {
    // 深度防护回归：40 层合法括号嵌套必须照常编译（上限 64 层远高于教学需求）。
    let mut src = String::from("int main(){ int x = ");
    src.push_str(&"(".repeat(40));
    src.push('1');
    src.push_str(&")".repeat(40));
    src.push_str("; return x; }\n");
    let result = compile_and_run(&src);
    assert!(result.is_ok(), "40 层合法嵌套不应被深度防护误伤: {:?}", result.err());
}

// ===================== T-P0-8：自含 struct 环检测 =====================

#[test]
fn test_self_contained_struct_reports_e3072() {
    // 修复前：`struct S { struct S inner; }` 使 compute_type_size 无限递归栈溢出。
    // 学生写链表节点漏 `*` 即触发，是教学场景高频错误。
    let src = "struct S { int a; struct S inner; };\nint main(){ struct S s; s.a = 1; return 0; }\n";
    let result = compile_and_run(src);
    match result {
        Err(msg) => assert!(
            msg.contains("循环包含") || msg.contains("E3072"),
            "自含 struct 应报告 E3072 循环包含，实际: {}",
            msg
        ),
        Ok(_) => panic!("自含 struct 应编译失败"),
    }
}

#[test]
fn test_mutual_struct_containment_reports_e3072() {
    let src = "struct A { int x; struct B b; };\nstruct B { int y; struct A a; };\nint main(){ return 0; }\n";
    let result = compile_and_run(src);
    match result {
        Err(msg) => assert!(
            msg.contains("循环包含") || msg.contains("E3072"),
            "互相包含 struct 应报告 E3072 循环包含，实际: {}",
            msg
        ),
        Ok(_) => panic!("互相包含 struct 应编译失败"),
    }
}

#[test]
fn test_legal_linked_list_pointer_still_compiles() {
    // 环检测回归：指针成员不构成环，合法链表节点必须照常编译运行。
    let src = "struct Node { int val; struct Node* next; };\nint main(){ struct Node n; n.val = 1; n.next = 0; return n.val; }\n";
    let result = compile_and_run(src);
    assert!(result.is_ok(), "合法链表节点不应被环检测误伤: {:?}", result.err());
}

// ===================== V-P0-5：乘法溢出防护 =====================

#[test]
fn test_qsort_huge_params_rejected_no_oom() {
    // 修复前：qsort(base, 2^32, 2^32, cmp) 的 nmemb*size usize 乘法 wrap 为 0，
    // 绕过边界检查后 (0..2^32).collect() 分配 ~32GB 导致 OOM abort。
    let src = "#include <stdlib.h>\nint cmp(const void* a, const void* b){ return 0; }\nint main(){ qsort((void*)100, 4294967295u, 4294967295u, cmp); return 0; }\n";
    let result = compile_and_run(src);
    assert!(result.is_ok(), "恶意 qsort 参数应被静默拒绝而非 OOM: {:?}", result.err());
}

// ===================== E-P1-6：apply_fix 中文行安全 =====================

#[test]
fn test_apply_fix_chinese_line_no_panic() {
    use cide_native::session::Diagnostic;

    let make_diag = |start_col: i32, end_col: i32, text: &str| Diagnostic {
        line: 1,
        column: 1,
        error_code: 3001,
        severity: 1,
        message: "test".to_string(),
        fix_suggestion: String::new(),
        fix_kind: 1, // ReplaceText
        replace_start_line: 1,
        replace_start_column: start_col,
        replace_end_line: 1,
        replace_end_column: end_col,
        replacement_text: text.to_string(),
        filename: "main.c".to_string(),
    };

    // 含中文的行：字节与字符语义混用时，旧实现按字节切片会 panic（非字符边界）。
    // "中文" 占 6 字节；列 3 在字节语义下落在 '文' 中间。
    let source = "int 中文变量 = 1;".to_string();

    // 字节列恰好合法（列 6：'文' 之后）
    let r = cide_native::api::cide::apply_fix(source.clone(), make_diag(6, 9, "X"));
    assert!(r.is_some(), "合法字节边界列应正常替换");

    // 字节列落在多字节字符中间（列 3）：旧实现 panic，新实现按字符语义回退
    let r = cide_native::api::cide::apply_fix(source.clone(), make_diag(3, 9, "X"));
    assert!(r.is_some(), "非字符边界列不应 panic，应按字符语义回退");

    // 字符语义列（列 4 = 第 5 个字符 '文' 之后的位置）也应工作
    let r = cide_native::api::cide::apply_fix(source.clone(), make_diag(4, 10, "X"));
    assert!(r.is_some(), "字符语义列应正常替换");
}
