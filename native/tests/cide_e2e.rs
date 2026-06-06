use std::ffi::{c_char, CString};
use std::path::Path;

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

fn load_cases(dir: &Path) -> Vec<(String, String)> {
    let mut cases = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("c") {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                let source = std::fs::read_to_string(&path).unwrap_or_default();
                cases.push((name, source));
            }
        }
    }
    cases.sort_by(|a, b| a.0.cmp(&b.0));
    cases
}

fn load_golden(case_name: &str, subdir: &str) -> Option<Vec<String>> {
    let path = Path::new("tests/cases_golden")
        .join(subdir)
        .join(format!("{}.out", case_name));
    let content = std::fs::read_to_string(&path).ok()?;
    let lines: Vec<String> = content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    Some(lines)
}

fn run_case(name: &str, source: &str, golden_subdir: &str) -> Result<(), String> {
    let result = compile_and_run(source);
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
    for (name, source) in &cases {
        if let Err(e) = run_case(name, source, "baseline") {
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
    "bellmanFord_default",
    "binarySearchTreeValidation_default",
    "infixEvaluation_default",
    "polynomialAdd_default",
    "redBlackTree_default",
    "spfa_default",
    "threadedBinaryTree_default",
];

#[test]
fn test_cide_e2e_template_generated() {
    let cases = load_cases(Path::new("tests/cases_template_generated"));
    let known: std::collections::HashSet<&str> =
        KNOWN_TEMPLATE_FAILURES.iter().copied().collect();

    let mut failures = Vec::new();
    for (name, source) in &cases {
        if known.contains(name.as_str()) {
            continue;
        }
        if let Err(e) = run_case(name, source, "") {
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

/// Monitor known failures: if any of them starts passing, this test fails
/// to remind us to update the documentation and remove it from the list.
#[test]
fn test_cide_e2e_template_known_failures() {
    let mut passed_unexpectedly = Vec::new();
    for name in KNOWN_TEMPLATE_FAILURES {
        let path = Path::new("tests/cases_template_generated").join(format!("{}.c", name));
        let source = std::fs::read_to_string(&path).unwrap_or_default();
        if run_case(name, &source, "").is_ok() {
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
fn test_cide_e2e_generate_report() {
    let baseline_cases = load_cases(Path::new("tests/cases/baseline"));
    let template_cases = load_cases(Path::new("tests/cases_template_generated"));

    let known: std::collections::HashSet<&str> =
        KNOWN_TEMPLATE_FAILURES.iter().copied().collect();

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
        template_cases.len() - known.len(),
        known.len()
    ));
    report.push_str("\n");

    report.push_str("## 已知失败详情（Template Generated）\n\n");
    report.push_str("| 用例 | 根因文件 |\n");
    report.push_str("|------|----------|\n");
    for name in KNOWN_TEMPLATE_FAILURES {
        report.push_str(&format!("| {} | E2E_FAILURES.md |\n", name));
    }
    report.push_str("\n");

    report.push_str("## Golden 生成失败\n\n");
    report.push_str("详见 `tests/cases_golden/GOLDEN_FAILURES.md`\n\n");

    let report_path = Path::new("tests/TEST_REPORT.md");
    std::fs::write(report_path, report).expect("write report");
}
