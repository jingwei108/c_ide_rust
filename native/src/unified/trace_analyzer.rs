//! Execution-trace slicing and root-cause inference (P0).
//!
//! When a trap occurs in unified mode, `TraceAnalyzer` looks back through the
//! collected `StepPayload` history, slices variable timelines, and produces a
//! human-readable `RootCauseHint`.

use crate::session::Session;
use crate::unified::root_cause::RootCauseHint;
use crate::unified::types::{ApiVariableSnapshot, StepPayload};

pub struct TraceAnalyzer;

impl TraceAnalyzer {
    /// Analyze a runtime trap and return a structured root-cause hint.
    ///
    /// * `steps`       – all previously collected steps (including the trap step).
    /// * `trap_step`   – index (in `steps`) of the step where the trap happened.
    /// * `trap_message`– VM error message (e.g. bounds, UAF, div-by-zero).
    /// * `session`     – current session (for source code inspection).
    pub fn analyze_trap(
        steps: &[StepPayload],
        trap_step: usize,
        trap_message: &str,
        session: &Session,
    ) -> Option<RootCauseHint> {
        if trap_message.contains("数组越界") {
            Self::analyze_bounds(trap_message, steps, trap_step, session)
        } else if trap_message.contains("Use-After-Free") || trap_message.contains("E3060") {
            Self::analyze_use_after_free(trap_message, steps, trap_step)
        } else if trap_message.contains("Double-Free") || trap_message.contains("E3061") {
            Self::analyze_double_free(trap_message, steps, trap_step)
        } else if trap_message.contains("除零") || trap_message.contains("除以零") {
            Self::analyze_div_zero(trap_message, steps, trap_step, session)
        } else if trap_message.contains("NULL") || trap_message.contains("null") {
            Self::analyze_null_deref(trap_message, steps, trap_step)
        } else {
            None
        }
    }

    // ------------------------------------------------------------------
    // Bounds (array index out of range)
    // ------------------------------------------------------------------

    fn analyze_bounds(
        trap_message: &str,
        steps: &[StepPayload],
        trap_step: usize,
        session: &Session,
    ) -> Option<RootCauseHint> {
        let (array_name, accessed_index) = parse_bounds_message(trap_message)?;
        let trap_payload = steps.get(trap_step)?;

        // Try to locate the loop variable whose current value equals the bad index.
        let suspect_var = trap_payload
            .local_vars
            .iter()
            .find(|v| {
                v.value.parse::<i32>().ok() == Some(accessed_index)
                    && is_likely_loop_var(&v.name)
            })
            .map(|v| v.name.clone());

        let mut related_lines = Vec::new();
        let (one_liner, fix_kind) = if let Some(ref var_name) = suspect_var {
            // Slice the variable's history over the last few steps.
            let history = slice_variable_history(steps, var_name, trap_step, 8);

            // Look at the source line where the trap occurred.
            let _trap_line_text = get_source_line(session, trap_payload.code_line);

            // Check surrounding lines for loop conditions containing '<='.
            let mut has_le_condition = false;
            for offset in -3..=3 {
                let line_no = trap_payload.code_line + offset;
                if line_no <= 0 {
                    continue;
                }
                let line_text = get_source_line(session, line_no);
                if line_text.contains("<=") && line_text.contains(&array_name) {
                    has_le_condition = true;
                    related_lines.push(line_no);
                }
            }
            // Also add the trap line itself.
            if !related_lines.contains(&trap_payload.code_line) {
                related_lines.push(trap_payload.code_line);
            }
            related_lines.sort();
            related_lines.dedup();

            // Infer root-cause category.
            let array_size = extract_array_size(trap_message).unwrap_or(0);
            if accessed_index == array_size && has_le_condition {
                (
                    format!(
                        "数组越界是因为循环条件使用了 <=，数组 '{}' 大小为 {}，最后一个有效索引是 {}。建议将 <= 改为 <。",
                        array_name, array_size, array_size.saturating_sub(1)
                    ),
                    String::from("ChangeLeToLt"),
                )
            } else if accessed_index == array_size {
                (
                    format!(
                        "数组 '{}' 的有效索引是 0~{}，但你访问了索引 {}。请检查循环边界或索引计算。",
                        array_name,
                        array_size.saturating_sub(1),
                        accessed_index
                    ),
                    String::from("ChangeLeToLt"),
                )
            } else if accessed_index > array_size {
                (
                    format!(
                        "索引 {} 远超数组 '{}' 的大小 {}。请检查循环起始值或索引表达式是否写错。",
                        accessed_index, array_name, array_size
                    ),
                    String::from("FixLoopStart"),
                )
            } else if history.len() < 2 {
                // Very little history → likely uninitialized or coming from scanf.
                (
                    format!(
                        "索引变量 '{}' 的值是 {}，但执行历史太短，可能是变量未初始化或从外部输入读取了非法值。",
                        var_name, accessed_index
                    ),
                    String::from("InitVariable"),
                )
            } else {
                (
                    format!(
                        "数组 '{}' 访问了越界索引 {}。请检查 '{}' 的最近变化：{}。",
                        array_name,
                        accessed_index,
                        var_name,
                        format_history(&history)
                    ),
                    String::from("None"),
                )
            }
        } else {
            // Could not identify a specific loop variable; give generic hint.
            related_lines.push(trap_payload.code_line);
            (
                format!(
                    "数组 '{}' 访问了越界索引 {}。请检查数组声明大小和所有使用该数组的循环条件。",
                    array_name, accessed_index
                ),
                String::from("None"),
            )
        };

        Some(RootCauseHint {
            category: String::from("OffByOne"),
            one_liner,
            related_lines,
            suggested_fix_kind: fix_kind,
        })
    }

    // ------------------------------------------------------------------
    // Use-After-Free
    // ------------------------------------------------------------------

    fn analyze_use_after_free(
        trap_message: &str,
        steps: &[StepPayload],
        trap_step: usize,
    ) -> Option<RootCauseHint> {
        let (alloc_line, freed_line) = parse_alloc_freed_lines(trap_message)?;
        let trap_payload = steps.get(trap_step)?;

        // Find pointer variables that are currently dangling / freed.
        let suspect_pointers: Vec<String> = trap_payload
            .pointer_snapshots
            .iter()
            .filter(|p| p.status == crate::unified::types::PointerStatus::Freed)
            .map(|p| p.name.clone())
            .collect();

        let mut one_liner = format!(
            "这块内存在第 {} 行被分配，在第 {} 行被释放，现在又被访问了。free 后请立即将指针置为 NULL，避免继续使用。",
            alloc_line, freed_line
        );

        if !suspect_pointers.is_empty() {
            one_liner.push_str(&format!(
                " 相关指针: {}。",
                suspect_pointers.join(", ")
            ));
        }

        let mut related_lines = vec![alloc_line, freed_line, trap_payload.code_line];
        related_lines.sort();
        related_lines.dedup();

        Some(RootCauseHint {
            category: String::from("UseAfterFree"),
            one_liner,
            related_lines,
            suggested_fix_kind: String::from("SetNullAfterFree"),
        })
    }

    // ------------------------------------------------------------------
    // Double-Free
    // ------------------------------------------------------------------

    fn analyze_double_free(
        trap_message: &str,
        steps: &[StepPayload],
        trap_step: usize,
    ) -> Option<RootCauseHint> {
        let (alloc_line, freed_line) = parse_alloc_freed_lines(trap_message)?;
        let trap_payload = steps.get(trap_step)?;

        let one_liner = format!(
            "同一块内存在第 {} 行被释放了两次。第一次释放发生在第 {} 行。每次 free(p) 后请立刻执行 p = NULL;，因为对 NULL 重复 free 是安全的。",
            freed_line, alloc_line
        );

        let mut related_lines = vec![alloc_line, freed_line, trap_payload.code_line];
        related_lines.sort();
        related_lines.dedup();

        Some(RootCauseHint {
            category: String::from("DoubleFree"),
            one_liner,
            related_lines,
            suggested_fix_kind: String::from("SetNullAfterFree"),
        })
    }

    // ------------------------------------------------------------------
    // Division by zero
    // ------------------------------------------------------------------

    fn analyze_div_zero(
        trap_message: &str,
        steps: &[StepPayload],
        trap_step: usize,
        _session: &Session,
    ) -> Option<RootCauseHint> {
        let trap_payload = steps.get(trap_step)?;

        // Try to extract the dividend from the Chinese message:
        // "😵 除零错误：你试图用 42 除以 0。"
        let dividend = trap_message
            .split("你试图用 ")
            .nth(1)
            .and_then(|s| s.split(" 除以").next())
            .and_then(|s| s.parse::<i32>().ok());

        let mut one_liner = String::from("发生了除零错误。除数变成了 0，这在数学上没有定义。");

        // Look for a variable whose current value is 0 and that is likely the divisor.
        let zero_vars: Vec<String> = trap_payload
            .local_vars
            .iter()
            .filter(|v| v.value == "0" && !is_likely_loop_var(&v.name))
            .map(|v| v.name.clone())
            .collect();

        if !zero_vars.is_empty() {
            one_liner.push_str(&format!(
                " 当前值为 0 的变量: {}。请检查这些变量是否在除法前被正确初始化或保护。",
                zero_vars.join(", ")
            ));
        } else if let Some(d) = dividend {
            one_liner.push_str(&format!(
                " 被除数是 {}，但除数变成了 0。请确认除法前检查了除数是否为零。",
                d
            ));
        }

        Some(RootCauseHint {
            category: String::from("DivZero"),
            one_liner,
            related_lines: vec![trap_payload.code_line],
            suggested_fix_kind: String::from("AvoidDivZero"),
        })
    }

    // ------------------------------------------------------------------
    // NULL pointer dereference
    // ------------------------------------------------------------------

    fn analyze_null_deref(
        _trap_message: &str,
        steps: &[StepPayload],
        trap_step: usize,
    ) -> Option<RootCauseHint> {
        let trap_payload = steps.get(trap_step)?;

        // Find null pointers at the trap step.
        let null_ptrs: Vec<String> = trap_payload
            .pointer_snapshots
            .iter()
            .filter(|p| p.status == crate::unified::types::PointerStatus::Null)
            .map(|p| p.name.clone())
            .collect();

        let mut one_liner =
            String::from("试图解引用一个 NULL 指针。NULL 指针不指向任何有效内存。");

        if !null_ptrs.is_empty() {
            one_liner.push_str(&format!(
                " 当前为 NULL 的指针: {}。请在使用前确保指针已被正确初始化（如通过 malloc 或赋值）。",
                null_ptrs.join(", ")
            ));
        }

        // Slice history of the first null pointer to see if it was ever non-null.
        let related_lines = vec![trap_payload.code_line];
        if let Some(first_ptr) = null_ptrs.first() {
            let history = slice_variable_history(steps, first_ptr, trap_step, 5);
            for h in &history {
                if h.value != "0" && h.value != "0x0" && h.value != "NULL" {
                    // The pointer was non-null earlier → look for a free or assignment to 0.
                    one_liner.push_str(&format!(
                        " 注意：'{}' 之前指向过地址 {}，后来变成了 NULL。",
                        first_ptr, h.value
                    ));
                    break;
                }
            }
        }

        Some(RootCauseHint {
            category: String::from("NullDeref"),
            one_liner,
            related_lines,
            suggested_fix_kind: String::from("AddNullCheck"),
        })
    }
}

// ===================================================================
// Helpers
// ===================================================================

/// Extract variable snapshots for `var_name` from the `lookback` steps
/// immediately preceding (and including) `trap_step`.
fn slice_variable_history<'a>(
    steps: &'a [StepPayload],
    var_name: &str,
    trap_step: usize,
    lookback: usize,
) -> Vec<&'a ApiVariableSnapshot> {
    let start = trap_step.saturating_sub(lookback);
    let mut result = Vec::new();
    for step in &steps[start..=trap_step] {
        if let Some(v) = step.local_vars.iter().find(|v| v.name == var_name) {
            result.push(v);
        }
    }
    result
}

/// Parse the Chinese bounds message to extract (array_name, accessed_index).
/// Example: "🚫 数组越界：你访问了 arr[5]，但数组 'arr' 只有 5 个元素..."
fn parse_bounds_message(msg: &str) -> Option<(String, i32)> {
    let start = msg.find("你访问了 ")? + "你访问了 ".len();
    let rest = &msg[start..];
    let bracket_open = rest.find('[')?;
    let bracket_close = rest.find(']')?;
    let array_name = rest[..bracket_open].trim().to_string();
    let index_str = &rest[bracket_open + 1..bracket_close];
    let index = index_str.parse::<i32>().ok()?;
    Some((array_name, index))
}

/// Parse the array size from the bounds message.
fn extract_array_size(msg: &str) -> Option<i32> {
    // Message contains "只有 {} 个元素"
    let start = msg.find("只有 ")? + "只有 ".len();
    let rest = &msg[start..];
    let end = rest.find(" 个元素")?;
    rest[..end].parse::<i32>().ok()
}

/// Parse alloc_line and freed_line from UAF / Double-Free messages.
/// Both formats contain "第 {} 行被 free" and "由第 {} 行的 malloc".
fn parse_alloc_freed_lines(msg: &str) -> Option<(i32, i32)> {
    // freed_line
    let freed_start = msg.find("第 ")? + "第 ".len();
    let freed_rest = &msg[freed_start..];
    let freed_end = freed_rest.find(" 行")?;
    let freed_line = freed_rest[..freed_end].parse::<i32>().ok()?;

    // alloc_line
    let alloc_marker = "由第 ";
    let alloc_start = msg.find(alloc_marker)? + alloc_marker.len();
    let alloc_rest = &msg[alloc_start..];
    let alloc_end = alloc_rest.find(" 行的")?;
    let alloc_line = alloc_rest[..alloc_end].parse::<i32>().ok()?;

    Some((alloc_line, freed_line))
}

/// Retrieve a source line from the first compile unit (matches collector.rs logic).
fn get_source_line(session: &Session, line: i32) -> String {
    if line <= 0 {
        return String::new();
    }
    session
        .compile
        .compile_units
        .first()
        .and_then(|u| {
            u.source
                .lines()
                .nth((line - 1) as usize)
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_default()
}

/// Names commonly used as loop / index variables.
fn is_likely_loop_var(name: &str) -> bool {
    matches!(
        name,
        "i" | "j" | "k" | "idx" | "index" | "m" | "n" | "left" | "right" | "mid" | "low"
            | "high" | "pivot" | "gap" | "l" | "r"
    )
}

/// Format a short variable-history string for the one-liner.
fn format_history(history: &[&ApiVariableSnapshot]) -> String {
    if history.len() <= 3 {
        history
            .iter()
            .map(|v| format!("{}={}", v.name, v.value))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        let first = history.first().unwrap();
        let last = history.last().unwrap();
        format!(
            "{}={} → ... → {}={}",
            first.name, first.value, last.name, last.value
        )
    }
}

// ===================================================================
// Unit tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bounds_message() {
        let msg = "🚫 数组越界：你访问了 arr[5]，但数组 'arr' 只有 5 个元素，有效索引是 0~4。";
        let (name, idx) = parse_bounds_message(msg).unwrap();
        assert_eq!(name, "arr");
        assert_eq!(idx, 5);
        assert_eq!(extract_array_size(msg), Some(5));
    }

    #[test]
    fn test_parse_alloc_freed_lines() {
        let msg = "💥 Use-After-Free (E3060)：你正在读取一块已经在第 10 行被 free 的内存（由第 3 行的 malloc/realloc 分配）。";
        let (alloc, freed) = parse_alloc_freed_lines(msg).unwrap();
        assert_eq!(alloc, 3);
        assert_eq!(freed, 10);
    }

    #[test]
    fn test_is_likely_loop_var() {
        assert!(is_likely_loop_var("i"));
        assert!(is_likely_loop_var("idx"));
        assert!(!is_likely_loop_var("total"));
    }

    #[test]
    fn test_slice_variable_history() {
        let steps = vec![
            StepPayload {
                step_index: 0,
                code_line: 1,
                func_name: "main".into(),
                semantic_label: "".into(),
                algorithm_step: None,
                local_vars: vec![ApiVariableSnapshot {
                    name: "i".into(),
                    addr: 0,
                    is_local: true,
                    ty_name: "int".into(),
                    value: "0".into(),
                }],
                call_stack: vec![],
                vis_events: vec![],
                heatmap_line: 1,
                heatmap_count: 0,
                accessed_vars: vec![],
                array_snapshots: vec![],
                pointer_snapshots: vec![],
                root_cause_hint: None,
            },
            StepPayload {
                step_index: 1,
                code_line: 1,
                func_name: "main".into(),
                semantic_label: "".into(),
                algorithm_step: None,
                local_vars: vec![ApiVariableSnapshot {
                    name: "i".into(),
                    addr: 0,
                    is_local: true,
                    ty_name: "int".into(),
                    value: "1".into(),
                }],
                call_stack: vec![],
                vis_events: vec![],
                heatmap_line: 1,
                heatmap_count: 0,
                accessed_vars: vec![],
                array_snapshots: vec![],
                pointer_snapshots: vec![],
                root_cause_hint: None,
            },
        ];
        let hist = slice_variable_history(&steps, "i", 1, 3);
        assert_eq!(hist.len(), 2);
        assert_eq!(hist[0].value, "0");
        assert_eq!(hist[1].value, "1");
    }
}
