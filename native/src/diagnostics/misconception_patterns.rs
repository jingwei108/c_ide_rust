//! Misconception pattern detection (P1).
//!
//! Analyzes a sliding window of recent compile/run records and identifies
//! stable misconception patterns (e.g. boundary confusion, pointer lifetime
//! misunderstanding, assignment-vs-comparison mix-up).

use flutter_rust_bridge::frb;

/// A single compile/run record from the student’s history.
#[frb]
#[derive(Debug, Clone)]
pub struct CompileRecord {
    /// Unix timestamp in milliseconds.
    pub timestamp_ms: i64,
    /// Whether compilation succeeded.
    pub success: bool,
    /// Error / warning / hint codes produced in this compile.
    pub error_codes: Vec<i32>,
    /// Runtime trap message, if any.
    pub trap_message: Option<String>,
}

/// A predefined misconception pattern.
#[frb]
#[derive(Debug, Clone)]
pub struct MisconceptionPattern {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Error codes that trigger this pattern.
    pub error_codes: Vec<i32>,
    /// Minimum number of matching records required to flag the pattern.
    pub min_occurrences: i32,
    /// Look-back window size (number of recent records to inspect).
    pub time_window: i32,
}

/// A detected misconception with confidence score.
#[frb]
#[derive(Debug, Clone)]
pub struct DetectedMisconception {
    pub pattern_id: String,
    pub pattern_name: String,
    pub description: String,
    /// How many records in the window matched.
    pub occurrence_count: i32,
    /// 0.0 ~ 1.0  (occurrences / min(window, min_occurrences) 的变体).
    pub confidence: f32,
}

/// Default built-in misconception patterns.
pub fn default_patterns() -> Vec<MisconceptionPattern> {
    vec![
        MisconceptionPattern {
            id: String::from("M01"),
            name: String::from("边界混淆"),
            description: String::from(
                "不理解数组“大小为 N”与“索引 0~N-1”的区别，循环条件常写成 <= 导致越界。"
            ),
            error_codes: vec![3021, 3051], // Bounds + OffByOne warning
            min_occurrences: 3,
            time_window: 10,
        },
        MisconceptionPattern {
            id: String::from("M02"),
            name: String::from("指针生命周期混淆"),
            description: String::from(
                "认为指针存的是“变量名”而非地址，导致 free 后继续访问或重复释放。"
            ),
            error_codes: vec![3035, 3060, 3061], // Null deref (TypeChecker) + UAF + Double-Free
            min_occurrences: 2,
            time_window: 10,
        },
        MisconceptionPattern {
            id: String::from("M03"),
            name: String::from("赋值与比较混淆"),
            description: String::from(
                "在 if/while 条件中误用 = 代替 ==，不理解两者的语义差异。"
            ),
            error_codes: vec![3050], // AssignInCondition warning
            min_occurrences: 3,
            time_window: 10,
        },
        MisconceptionPattern {
            id: String::from("M04"),
            name: String::from("数组指针退化误解"),
            description: String::from(
                "不知道数组在表达式中会退化为指针，导致 sizeof 或指针算术结果与预期不符。"
            ),
            error_codes: vec![3045, 3052], // PtrArithTypeError + ArrayDecay warning
            min_occurrences: 2,
            time_window: 10,
        },
        MisconceptionPattern {
            id: String::from("M05"),
            name: String::from("递归边界遗漏"),
            description: String::from(
                "不理解递归必须有终止条件，导致无限递归或栈溢出。"
            ),
            error_codes: vec![], // runtime trap keyword matching only
            min_occurrences: 2,
            time_window: 10,
        },
        MisconceptionPattern {
            id: String::from("M06"),
            name: String::from("格式化字符串误用"),
            description: String::from(
                "不理解 %d/%f/%s 与变量类型的对应关系，printf/scanf 格式与参数不匹配。"
            ),
            error_codes: vec![3030, 3031, 3032, 3033, 3034, 3035], // printf/scanf family
            min_occurrences: 3,
            time_window: 10,
        },
    ]
}

/// Analyze recent compile history and return detected misconceptions.
#[frb]
pub fn detect_misconceptions(history: Vec<CompileRecord>) -> Vec<DetectedMisconception> {
    let patterns = default_patterns();
    let mut results = Vec::new();

    for pat in &patterns {
        let window = pat.time_window.max(1) as usize;
        let recent = if history.len() > window {
            &history[history.len() - window..]
        } else {
            &history[..]
        };

        let mut count = 0i32;
        for rec in recent {
            if record_matches(rec, pat) {
                count += 1;
            }
        }

        if count >= pat.min_occurrences {
            let denominator = recent.len().min(pat.time_window as usize).max(1) as f32;
            let confidence = (count as f32 / denominator).min(1.0);
            results.push(DetectedMisconception {
                pattern_id: pat.id.clone(),
                pattern_name: pat.name.clone(),
                description: pat.description.clone(),
                occurrence_count: count,
                confidence,
            });
        }
    }

    // Sort by confidence descending.
    results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
    results
}

fn record_matches(rec: &CompileRecord, pat: &MisconceptionPattern) -> bool {
    // Match by error code.
    if !pat.error_codes.is_empty() {
        for code in &pat.error_codes {
            if rec.error_codes.contains(code) {
                return true;
            }
        }
    }

    // M05 special handling: match by trap keywords (stack overflow / infinite recursion).
    if pat.id == "M05" {
        if let Some(ref msg) = rec.trap_message {
            let lower = msg.to_lowercase();
            if lower.contains("stack overflow")
                || lower.contains("call stack")
                || lower.contains("栈溢出")
            {
                return true;
            }
        }
    }

    false
}

// ===================================================================
// Unit tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(codes: Vec<i32>, trap: Option<&str>) -> CompileRecord {
        CompileRecord {
            timestamp_ms: 0,
            success: codes.is_empty(),
            error_codes: codes,
            trap_message: trap.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_detect_m01_boundary_confusion() {
        let history = vec![
            make_record(vec![3051], None),
            make_record(vec![3021], None),
            make_record(vec![3051], None),
            make_record(vec![], None),
        ];
        let detected = detect_misconceptions(history);
        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].pattern_id, "M01");
        assert_eq!(detected[0].occurrence_count, 3);
    }

    #[test]
    fn test_detect_m03_assignment_in_condition() {
        let history = vec![
            make_record(vec![3050], None),
            make_record(vec![3050], None),
            make_record(vec![3050], None),
        ];
        let detected = detect_misconceptions(history);
        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].pattern_id, "M03");
    }

    #[test]
    fn test_detect_m05_stack_overflow_by_trap() {
        let history = vec![
            make_record(vec![], Some("栈溢出：调用栈深度超过限制")),
            make_record(vec![], Some("Stack overflow detected")),
        ];
        let detected = detect_misconceptions(history);
        let m05 = detected.iter().find(|d| d.pattern_id == "M05");
        assert!(m05.is_some(), "M05 should be detected by trap keyword");
    }

    #[test]
    fn test_no_detection_below_threshold() {
        let history = vec![
            make_record(vec![3051], None),
            make_record(vec![], None),
        ];
        let detected = detect_misconceptions(history);
        assert!(detected.is_empty());
    }
}
