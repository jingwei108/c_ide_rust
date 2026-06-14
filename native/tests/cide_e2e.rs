use std::ffi::{c_char, CString};
use std::path::Path;

use cide_native::session::InputMode;

fn compile_and_run(source: &str, input: Option<&str>, input_mode: InputMode) -> Result<(i32, Vec<String>), String> {
    compile_and_run_with_filename(source, input, input_mode, "main.c")
}

fn compile_and_run_cpp(source: &str, input: Option<&str>, input_mode: InputMode) -> Result<(i32, Vec<String>), String> {
    compile_and_run_with_filename(source, input, input_mode, "main.cpp")
}

fn compile_and_run_with_filename(
    source: &str,
    input: Option<&str>,
    input_mode: InputMode,
    filename: &str,
) -> Result<(i32, Vec<String>), String> {
    unsafe {
        let session = cide_native::capi::cide_session_create();
        if session.is_null() {
            return Err("Failed to create session".to_string());
        }

        (*session).runtime.input_mode = input_mode;

        // 设置输入数据（如果提供）
        if let Some(input_str) = input {
            // 保留换行符以模拟真实 stdin 字节流（getchar 需要读到 \n）
            let normalized = input_str.replace("\r\n", "\n");
            (*session).runtime.input_lines = normalized.split_inclusive('\n').map(|l| l.to_string()).collect();
        }

        let fname = CString::new(filename).map_err(|e| e.to_string())?;
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
            // Exclude the trailing null byte that cide_get_output writes
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

        cide_native::capi::cide_session_destroy(session);

        if let Some(e) = runtime_err {
            if !e.is_empty() {
                return Err(format!("Runtime error: {}", e));
            }
        }

        Ok((run_ret, outputs))
    }
}

/// Filter out Cide-specific diagnostic lines that are not part of the
/// program's own stdout (e.g. completion message, leak report).
fn filter_cide_diagnostics(lines: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    let mut in_leak_report = false;
    for line in lines {
        // Cide appends "程序运行完成" directly after the last program output
        // if there is no trailing newline. Strip it from the line tail.
        let mut cleaned = line.clone();
        if let Some(pos) = cleaned.find("程序运行完成，返回值：") {
            cleaned = cleaned[..pos].to_string();
        }
        if cleaned.starts_with("===== 内存泄漏检测报告 =====") {
            in_leak_report = true;
            continue;
        }
        if in_leak_report && cleaned.starts_with("==============================") {
            in_leak_report = false;
            continue;
        }
        if in_leak_report {
            continue;
        }
        let trimmed = cleaned.trim();
        if !trimmed.is_empty() {
            result.push(trimmed.to_string());
        }
    }
    result
}

fn load_cases(dir: &Path) -> Vec<(String, String, Option<String>)> {
    load_cases_with_ext(dir, "c")
}

fn load_cpp_cases(dir: &Path) -> Vec<(String, String, Option<String>)> {
    load_cases_with_ext(dir, "cpp")
}

fn load_cases_with_ext(dir: &Path, ext: &str) -> Vec<(String, String, Option<String>)> {
    let mut cases = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some(ext) {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                let source = std::fs::read_to_string(&path).unwrap_or_default();
                let input_path = path.with_extension("in");
                let input = if input_path.exists() {
                    Some(std::fs::read_to_string(&input_path).unwrap_or_default())
                } else {
                    None
                };
                cases.push((name, source, input));
            }
        }
    }
    cases.sort_by(|a, b| a.0.cmp(&b.0));
    cases
}

fn load_golden(case_name: &str, subdir: &str) -> Option<Vec<String>> {
    let path = Path::new("tests/cases_golden").join(subdir).join(format!("{}.out", case_name));
    let content = std::fs::read_to_string(&path).ok()?;
    let lines: Vec<String> = content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    Some(lines)
}

fn run_case(
    name: &str,
    source: &str,
    input: Option<&str>,
    golden_subdir: &str,
    input_mode: InputMode,
) -> Result<(), String> {
    run_case_with_compiler(name, source, input, golden_subdir, input_mode, false)
}

fn run_cpp_case(
    name: &str,
    source: &str,
    input: Option<&str>,
    golden_subdir: &str,
    input_mode: InputMode,
) -> Result<(), String> {
    run_case_with_compiler(name, source, input, golden_subdir, input_mode, true)
}

fn run_case_with_compiler(
    name: &str,
    source: &str,
    input: Option<&str>,
    golden_subdir: &str,
    input_mode: InputMode,
    is_cpp: bool,
) -> Result<(), String> {
    let result = if is_cpp {
        compile_and_run_cpp(source, input, input_mode)
    } else {
        compile_and_run(source, input, input_mode)
    };
    match result {
        Ok((ret, outputs)) => {
            if ret != 0 {
                return Err(format!("Exit code {} != 0", ret));
            }
            let filtered = filter_cide_diagnostics(&outputs);
            if let Some(golden) = load_golden(name, golden_subdir) {
                if filtered != golden {
                    return Err(format!(
                        "Output mismatch.\nExpected ({} lines): {:?}\nActual ({} lines): {:?}",
                        golden.len(),
                        golden,
                        filtered.len(),
                        filtered
                    ));
                }
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

#[test]
fn test_cide_e2e_baseline() {
    let cases = load_cases(Path::new("tests/cases/baseline"));
    let mut failures = Vec::new();
    for (name, source, input) in &cases {
        if let Err(e) = run_case(name, source, input.as_deref(), "baseline", InputMode::Interactive) {
            failures.push(format!("{}: {}", name, e));
        }
    }
    if !failures.is_empty() {
        panic!(
            "Baseline e2e failures ({} of {}):\n{}",
            failures.len(),
            cases.len(),
            failures.join("\n")
        );
    }
}

/// Known template failures documented in E2E_FAILURES.md.
/// These are NOT silently ignored — they are monitored by
/// `test_cide_e2e_template_known_failures` below.
const KNOWN_TEMPLATE_FAILURES: &[&str] = &[
    "bTree_default",
    "infixEvaluation_default",
    "spfa_default",
    "threadedBinaryTree_default",
];

/// Known K&R failures documented in KR_FAILURES.md.
/// Monitored by `test_cide_e2e_knr_known_failures` below.
const KNOWN_KR_FAILURES: &[&str] = &[
    // 阶段 2: K&R 第 3-4 章（残留）
    // kr_4_3/kr_4_4/kr_4_5/kr_4_6 已修复 (作用域隔离 + printf %g 格式支持 + math.h)
    // kr_4_9 已修复 (用户函数 qsort 可遮蔽内置函数)
    // 阶段 3: K&R 第 5-6 章
    // kr_5_1 已修复 (添加 ungetc Host Func)
    // kr_5_2 已修复 (添加 ungetc Host Func)
    // kr_5_8 已修复 (Parser 支持函数指针类型转换；Golden 按 Clang+stdlib.h 重新生成)
    // kr_5_9 已修复 (TypeChecker 支持 (*fp)(args) + BytecodeGen 函数指针解引用不加载内存)
    // kr_5_10 已修复 (VM 支持 main(int argc, char *argv[]) + CLI 传递参数)
    // kr_5_11 已修复 (char*[] 指针数组 elem_type_size / 初始化路径)
    // kr_5_13 已修复 (getchar Batch 模式多行输入)
    // kr_5_14 已修复 (Parser 支持函数指针类型转换/抽象声明符)
    "kr_6_1",
    // kr_6_2 已修复 (Parser 指针无名参数 + ungetc)
    // kr_6_3 已修复 (Parser 指针无名参数 + ungetc)
    // kr_6_4 已修复 (Parser 指针无名参数 + ungetc)
    // kr_6_5 已修复 (添加 strdup Host Func)
    // kr_6_6 已修复 (添加 strdup Host Func)
];

/// Known LeetCode failures documented in LEETCODE_FAILURES.md.
/// Monitored by `test_cide_e2e_leetcode_known_failures` below.
const KNOWN_LEETCODE_FAILURES: &[&str] = &[
    // 阶段 4~5 逐步填充
];

/// Known C++ failures documented in CPP_FAILURES.md.
/// Monitored by `test_cide_e2e_cpp_known_failures` below.
const KNOWN_CPP_FAILURES: &[&str] = &[
    // M6 推进过程中逐步填充
];

#[test]
fn test_cide_e2e_template_generated() {
    let cases = load_cases(Path::new("tests/cases_template_generated"));
    let known: std::collections::HashSet<&str> = KNOWN_TEMPLATE_FAILURES.iter().copied().collect();

    let mut failures = Vec::new();
    for (name, source, input) in &cases {
        if known.contains(name.as_str()) {
            continue;
        }
        if let Err(e) = run_case(name, source, input.as_deref(), "", InputMode::Interactive) {
            failures.push(format!("{}: {}", name, e));
        }
    }
    if !failures.is_empty() {
        panic!(
            "Template generated e2e failures ({} of {} non-known):\n{}\n\n\
             These are NEW failures not yet in KNOWN_TEMPLATE_FAILURES. \
             Please investigate, record in E2E_FAILURES.md, and update the list.",
            failures.len(),
            cases.len() - known.len(),
            failures.join("\n")
        );
    }
}

#[test]
fn test_cide_e2e_knr() {
    let cases = load_cases(Path::new("tests/cases/knr"));
    let known: std::collections::HashSet<&str> = KNOWN_KR_FAILURES.iter().copied().collect();

    let mut failures = Vec::new();
    for (name, source, input) in &cases {
        if known.contains(name.as_str()) {
            continue;
        }
        // K&R getchar 用例在 Batch 模式下运行（输入耗尽返回 EOF）
        let mode = if source.contains("getchar()") {
            InputMode::Batch
        } else {
            InputMode::Interactive
        };
        if let Err(e) = run_case(name, source, input.as_deref(), "knr", mode) {
            failures.push(format!("{}: {}", name, e));
        }
    }
    if !failures.is_empty() {
        panic!(
            "K&R e2e failures ({} of {} non-known):\n{}\n\n\
             These are NEW failures not yet in KNOWN_KR_FAILURES. \
             Please investigate, record in KR_FAILURES.md, and update the list.",
            failures.len(),
            cases.len() - known.len(),
            failures.join("\n")
        );
    }
}

#[test]
fn test_cide_e2e_leetcode() {
    let cases = load_cases(Path::new("tests/cases/leetcode"));
    let known: std::collections::HashSet<&str> = KNOWN_LEETCODE_FAILURES.iter().copied().collect();

    let mut failures = Vec::new();
    for (name, source, input) in &cases {
        if known.contains(name.as_str()) {
            continue;
        }
        if let Err(e) = run_case(name, source, input.as_deref(), "leetcode", InputMode::Interactive) {
            failures.push(format!("{}: {}", name, e));
        }
    }
    if !failures.is_empty() {
        panic!(
            "LeetCode e2e failures ({} of {} non-known):\n{}\n\n\
             These are NEW failures not yet in KNOWN_LEETCODE_FAILURES. \
             Please investigate, record in LEETCODE_FAILURES.md, and update the list.",
            failures.len(),
            cases.len() - known.len(),
            failures.join("\n")
        );
    }
}

/// Monitor known failures: if any of them starts passing, this test fails
/// to remind us to update the documentation and remove it from the list.
#[test]
fn test_cide_e2e_template_known_failures() {
    let mut passed_unexpectedly = Vec::new();
    for name in KNOWN_TEMPLATE_FAILURES {
        let path = Path::new("tests/cases_template_generated").join(format!("{}.c", name));
        let source = std::fs::read_to_string(&path).unwrap_or_default();
        let input_path = path.with_extension("in");
        let input = if input_path.exists() {
            Some(std::fs::read_to_string(&input_path).unwrap_or_default())
        } else {
            None
        };
        if run_case(name, &source, input.as_deref(), "", InputMode::Interactive).is_ok() {
            passed_unexpectedly.push(name.to_string());
        }
    }
    if !passed_unexpectedly.is_empty() {
        panic!(
            "Known failures unexpectedly PASSED ({}). \
             Please update E2E_FAILURES.md and KNOWN_TEMPLATE_FAILURES in cide_e2e.rs:\n{}",
            passed_unexpectedly.len(),
            passed_unexpectedly.join("\n")
        );
    }
}

#[test]
fn test_cide_e2e_knr_known_failures() {
    let mut passed_unexpectedly = Vec::new();
    for name in KNOWN_KR_FAILURES {
        let path = Path::new("tests/cases/knr").join(format!("{}.c", name));
        let source = std::fs::read_to_string(&path).unwrap_or_default();
        let input_path = path.with_extension("in");
        let input = if input_path.exists() {
            Some(std::fs::read_to_string(&input_path).unwrap_or_default())
        } else {
            None
        };
        let mode = if source.contains("getchar()") {
            InputMode::Batch
        } else {
            InputMode::Interactive
        };
        if run_case(name, &source, input.as_deref(), "knr", mode).is_ok() {
            passed_unexpectedly.push(name.to_string());
        }
    }
    if !passed_unexpectedly.is_empty() {
        panic!(
            "Known K&R failures unexpectedly PASSED ({}). \
             Please update KR_FAILURES.md and KNOWN_KR_FAILURES in cide_e2e.rs:\n{}",
            passed_unexpectedly.len(),
            passed_unexpectedly.join("\n")
        );
    }
}

#[test]
fn test_cide_e2e_leetcode_known_failures() {
    let mut passed_unexpectedly = Vec::new();
    for name in KNOWN_LEETCODE_FAILURES {
        let path = Path::new("tests/cases/leetcode").join(format!("{}.c", name));
        let source = std::fs::read_to_string(&path).unwrap_or_default();
        let input_path = path.with_extension("in");
        let input = if input_path.exists() {
            Some(std::fs::read_to_string(&input_path).unwrap_or_default())
        } else {
            None
        };
        if run_case(name, &source, input.as_deref(), "leetcode", InputMode::Interactive).is_ok() {
            passed_unexpectedly.push(name.to_string());
        }
    }
    if !passed_unexpectedly.is_empty() {
        panic!(
            "Known LeetCode failures unexpectedly PASSED ({}). \
             Please update LEETCODE_FAILURES.md and KNOWN_LEETCODE_FAILURES in cide_e2e.rs:\n{}",
            passed_unexpectedly.len(),
            passed_unexpectedly.join("\n")
        );
    }
}

#[test]
fn test_cide_e2e_cpp() {
    let cases = load_cpp_cases(Path::new("tests/cases/cpp"));
    let known: std::collections::HashSet<&str> = KNOWN_CPP_FAILURES.iter().copied().collect();

    let mut failures = Vec::new();
    for (name, source, input) in &cases {
        if known.contains(name.as_str()) {
            continue;
        }
        let mode = if source.contains("getchar()") {
            InputMode::Batch
        } else {
            InputMode::Interactive
        };
        if let Err(e) = run_cpp_case(name, source, input.as_deref(), "cpp", mode) {
            failures.push(format!("{}: {}", name, e));
        }
    }
    if !failures.is_empty() {
        panic!(
            "C++ e2e failures ({} of {} non-known):\n{}\n\n\
             These are NEW failures not yet in KNOWN_CPP_FAILURES. \
             Please investigate, record in CPP_FAILURES.md, and update the list.",
            failures.len(),
            cases.len() - known.len(),
            failures.join("\n")
        );
    }
}

#[test]
fn test_cide_e2e_cpp_known_failures() {
    let mut passed_unexpectedly = Vec::new();
    for name in KNOWN_CPP_FAILURES {
        let path = Path::new("tests/cases/cpp").join(format!("{}.cpp", name));
        let source = std::fs::read_to_string(&path).unwrap_or_default();
        let input_path = path.with_extension("in");
        let input = if input_path.exists() {
            Some(std::fs::read_to_string(&input_path).unwrap_or_default())
        } else {
            None
        };
        let mode = if source.contains("getchar()") {
            InputMode::Batch
        } else {
            InputMode::Interactive
        };
        if run_cpp_case(name, &source, input.as_deref(), "cpp", mode).is_ok() {
            passed_unexpectedly.push(name.to_string());
        }
    }
    if !passed_unexpectedly.is_empty() {
        panic!(
            "Known C++ failures unexpectedly PASSED ({}). \
             Please update CPP_FAILURES.md and KNOWN_CPP_FAILURES in cide_e2e.rs:\n{}",
            passed_unexpectedly.len(),
            passed_unexpectedly.join("\n")
        );
    }
}

#[test]
fn test_cide_e2e_generate_report() {
    let baseline_cases = load_cases(Path::new("tests/cases/baseline"));
    let template_cases = load_cases(Path::new("tests/cases_template_generated"));
    let knr_cases = load_cases(Path::new("tests/cases/knr"));
    let leetcode_cases = load_cases(Path::new("tests/cases/leetcode"));
    let cpp_cases = load_cpp_cases(Path::new("tests/cases/cpp"));

    let known_template: std::collections::HashSet<&str> = KNOWN_TEMPLATE_FAILURES.iter().copied().collect();
    let known_kr: std::collections::HashSet<&str> = KNOWN_KR_FAILURES.iter().copied().collect();
    let known_leetcode: std::collections::HashSet<&str> = KNOWN_LEETCODE_FAILURES.iter().copied().collect();
    let known_cpp: std::collections::HashSet<&str> = KNOWN_CPP_FAILURES.iter().copied().collect();

    let mut report = String::new();
    report.push_str("# Cide E2E 测试报告\n\n");
    let now = std::time::SystemTime::now();
    let dt = now.duration_since(std::time::UNIX_EPOCH).unwrap();
    let ts = format!("{}.{:03}Z", dt.as_secs(), dt.subsec_millis());
    report.push_str(&format!("生成时间 (Unix): {}\n\n", ts));

    report.push_str("## 摘要\n\n");
    report.push_str("| 类别 | 总数 | 通过 | 已知失败 |\n");
    report.push_str("|------|------|------|----------|\n");
    report.push_str(&format!(
        "| Baseline | {} | {} | 0 |\n",
        baseline_cases.len(),
        baseline_cases.len()
    ));
    report.push_str(&format!(
        "| Template Generated | {} | {} | {} |\n",
        template_cases.len(),
        template_cases.len() - known_template.len(),
        known_template.len()
    ));
    report.push_str(&format!(
        "| K&R | {} | {} | {} |\n",
        knr_cases.len(),
        knr_cases.len().saturating_sub(known_kr.len()),
        known_kr.len()
    ));
    report.push_str(&format!(
        "| LeetCode | {} | {} | {} |\n",
        leetcode_cases.len(),
        leetcode_cases.len().saturating_sub(known_leetcode.len()),
        known_leetcode.len()
    ));
    report.push_str(&format!(
        "| C++ | {} | {} | {} |\n",
        cpp_cases.len(),
        cpp_cases.len().saturating_sub(known_cpp.len()),
        known_cpp.len()
    ));
    report.push('\n');

    if !KNOWN_TEMPLATE_FAILURES.is_empty() {
        report.push_str("## 已知失败详情（Template Generated）\n\n");
        report.push_str("| 用例 | 根因文件 |\n");
        report.push_str("|------|----------|\n");
        for name in KNOWN_TEMPLATE_FAILURES {
            report.push_str(&format!("| {} | E2E_FAILURES.md |\n", name));
        }
        report.push('\n');
    }

    if !KNOWN_KR_FAILURES.is_empty() {
        report.push_str("## 已知失败详情（K&R）\n\n");
        report.push_str("| 用例 | 根因文件 |\n");
        report.push_str("|------|----------|\n");
        for name in KNOWN_KR_FAILURES {
            report.push_str(&format!("| {} | KR_FAILURES.md |\n", name));
        }
        report.push('\n');
    }

    if !KNOWN_LEETCODE_FAILURES.is_empty() {
        report.push_str("## 已知失败详情（LeetCode）\n\n");
        report.push_str("| 用例 | 根因文件 |\n");
        report.push_str("|------|----------|\n");
        for name in KNOWN_LEETCODE_FAILURES {
            report.push_str(&format!("| {} | LEETCODE_FAILURES.md |\n", name));
        }
        report.push('\n');
    }

    if !KNOWN_CPP_FAILURES.is_empty() {
        report.push_str("## 已知失败详情（C++）\n\n");
        report.push_str("| 用例 | 根因文件 |\n");
        report.push_str("|------|----------|\n");
        for name in KNOWN_CPP_FAILURES {
            report.push_str(&format!("| {} | CPP_FAILURES.md |\n", name));
        }
        report.push('\n');
    }

    report.push_str("## Golden 生成失败\n\n");
    report.push_str("详见 `tests/cases_golden/GOLDEN_FAILURES.md`\n\n");

    let report_path = Path::new("tests/TEST_REPORT.md");
    std::fs::write(report_path, report).expect("write report");
}
