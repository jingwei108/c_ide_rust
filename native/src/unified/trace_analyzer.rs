//! Execution-trace slicing and root-cause inference (P0).
//!
//! When a trap occurs in unified mode, `TraceAnalyzer` looks back through the
//! collected `StepPayload` history, slices variable timelines, and produces a
//! human-readable `RootCauseHint`.

use crate::session::Session;
use crate::unified::root_cause::RootCauseHint;
use crate::unified::types::{ApiVariableSnapshot, StepPayload};

/// Fine-grained sub-category for array-bounds traps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoundsCategory {
    /// `i <= n` should be `i < n`.
    OffByOne,
    /// Loop starts at wrong value (e.g. `i = 1` instead of `i = 0`).
    WrongInit,
    /// Step size causes skip-over (e.g. `i += 2`).
    WrongIncrement,
    /// Index variable likely uninitialized or from external input.
    UninitializedIndex,
    /// Generic fallback when pattern cannot be determined.
    Generic,
}

impl BoundsCategory {
    fn as_str(&self) -> &'static str {
        match self {
            BoundsCategory::OffByOne => "OffByOne",
            BoundsCategory::WrongInit => "WrongInit",
            BoundsCategory::WrongIncrement => "WrongIncrement",
            BoundsCategory::UninitializedIndex => "UninitializedIndex",
            BoundsCategory::Generic => "GenericBounds",
        }
    }
}

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
        let array_size = extract_array_size(trap_message).unwrap_or(0);

        // Try to locate the loop variable whose current value equals the bad index.
        let suspect_var = trap_payload
            .local_vars
            .iter()
            .find(|v| v.value.parse::<i32>().ok() == Some(accessed_index) && is_likely_loop_var(&v.name))
            .map(|v| v.name.clone());

        let mut related_lines = Vec::new();

        let (category, one_liner, fix_kind, fix_line, fix_desc) = if let Some(ref var_name) = suspect_var {
            // Slice a longer history (up to 20 steps) for pattern inference.
            let history = slice_variable_history(steps, var_name, trap_step, 20);
            let hist_vals: Vec<i32> = history.iter().filter_map(|v| v.value.parse::<i32>().ok()).collect();

            // Scan surrounding source for loop constructs.
            let loop_info = scan_loop_info(session, trap_payload.code_line, var_name, &array_name);
            related_lines.extend(&loop_info.lines);
            if !related_lines.contains(&trap_payload.code_line) {
                related_lines.push(trap_payload.code_line);
            }
            related_lines.sort();
            related_lines.dedup();

            // Infer fine-grained category from history + source patterns.
            let category = infer_bounds_category(&hist_vals, accessed_index, array_size, &loop_info);

            let (one_liner, fix_kind, fix_line, fix_desc) = build_bounds_hint(
                category,
                var_name,
                &array_name,
                accessed_index,
                array_size,
                &loop_info,
                &hist_vals,
            );
            (category, one_liner, fix_kind, fix_line, fix_desc)
        } else {
            related_lines.push(trap_payload.code_line);
            (
                BoundsCategory::Generic,
                format!(
                    "数组 '{}' 访问了越界索引 {}。请检查数组声明大小和所有使用该数组的循环条件。",
                    array_name, accessed_index
                ),
                String::from("None"),
                None,
                None,
            )
        };

        Some(RootCauseHint {
            category: category.as_str().to_string(),
            one_liner,
            related_lines,
            suggested_fix_kind: fix_kind,
            suggested_fix_line: fix_line,
            suggested_fix_desc: fix_desc,
        })
    }

    // ------------------------------------------------------------------
    // Use-After-Free
    // ------------------------------------------------------------------

    fn analyze_use_after_free(trap_message: &str, steps: &[StepPayload], trap_step: usize) -> Option<RootCauseHint> {
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
            one_liner.push_str(&format!(" 相关指针: {}。", suspect_pointers.join(", ")));
        }

        let mut related_lines = vec![alloc_line, freed_line, trap_payload.code_line];
        related_lines.sort();
        related_lines.dedup();

        Some(RootCauseHint {
            category: String::from("UseAfterFree"),
            one_liner,
            related_lines,
            suggested_fix_kind: String::from("SetNullAfterFree"),
            suggested_fix_line: Some(freed_line),
            suggested_fix_desc: Some(String::from("free 后执行 p = NULL")),
        })
    }

    // ------------------------------------------------------------------
    // Double-Free
    // ------------------------------------------------------------------

    fn analyze_double_free(trap_message: &str, steps: &[StepPayload], trap_step: usize) -> Option<RootCauseHint> {
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
            suggested_fix_line: Some(freed_line),
            suggested_fix_desc: Some(String::from("free 后执行 p = NULL")),
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
            one_liner.push_str(&format!(" 被除数是 {}，但除数变成了 0。请确认除法前检查了除数是否为零。", d));
        }

        Some(RootCauseHint {
            category: String::from("DivZero"),
            one_liner,
            related_lines: vec![trap_payload.code_line],
            suggested_fix_kind: String::from("AvoidDivZero"),
            suggested_fix_line: Some(trap_payload.code_line),
            suggested_fix_desc: Some(String::from("除法前检查除数是否为 0")),
        })
    }

    // ------------------------------------------------------------------
    // NULL pointer dereference
    // ------------------------------------------------------------------

    fn analyze_null_deref(_trap_message: &str, steps: &[StepPayload], trap_step: usize) -> Option<RootCauseHint> {
        let trap_payload = steps.get(trap_step)?;

        // Find null pointers at the trap step.
        let null_ptrs: Vec<String> = trap_payload
            .pointer_snapshots
            .iter()
            .filter(|p| p.status == crate::unified::types::PointerStatus::Null)
            .map(|p| p.name.clone())
            .collect();

        let mut one_liner = String::from("试图解引用一个 NULL 指针。NULL 指针不指向任何有效内存。");

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
                    one_liner
                        .push_str(&format!(" 注意：'{}' 之前指向过地址 {}，后来变成了 NULL。", first_ptr, h.value));
                    break;
                }
            }
        }

        let fix_line = related_lines.first().copied();
        Some(RootCauseHint {
            category: String::from("NullDeref"),
            one_liner,
            related_lines,
            suggested_fix_kind: String::from("AddNullCheck"),
            suggested_fix_line: fix_line,
            suggested_fix_desc: Some(String::from("使用指针前检查是否为 NULL")),
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

/// Information about a loop construct gathered from source scanning.
#[derive(Default)]
struct LoopInfo {
    /// Source lines that belong to the loop header / condition.
    lines: Vec<i32>,
    /// True if a `<=` operator is present in the condition.
    has_le: bool,
    /// True if a `>=` operator is present in the condition.
    has_ge: bool,
    /// Detected start value literal (e.g. `int i = 1` → Some(1)).
    start_val: Option<i32>,
    /// Detected increment literal (e.g. `i += 2` → Some(2), `i++` → Some(1)).
    increment: Option<i32>,
}

/// Scan source lines around `trap_line` looking for loop constructs that involve
/// `var_name` and/or `array_name`.
fn scan_loop_info(session: &Session, trap_line: i32, var_name: &str, array_name: &str) -> LoopInfo {
    let mut info = LoopInfo::default();
    // Scan a window of ±6 lines around the trap.
    for offset in -6_i32..=6_i32 {
        let line_no = trap_line + offset;
        if line_no <= 0 {
            continue;
        }
        let text = get_source_line(session, line_no);
        if text.is_empty() {
            continue;
        }
        // Heuristic: line contains the variable or array name and looks like a loop.
        let looks_like_loop = text.contains("for(") || text.contains("for (");
        let looks_like_while = text.contains("while(") || text.contains("while (");
        let refers_var = text.contains(var_name);
        let refers_array = text.contains(array_name);
        if (looks_like_loop || looks_like_while) && (refers_var || refers_array) {
            info.lines.push(line_no);
            if text.contains("<=") {
                info.has_le = true;
            }
            if text.contains(">=") {
                info.has_ge = true;
            }
            // Try to extract `int i = X` or `i = X`.
            if let Some(val) = extract_assignment_rhs(&text, var_name) {
                info.start_val = Some(val);
            }
        }
        // Also look for increment statements on their own line: `i++`, `i += 2`, etc.
        if refers_var && !looks_like_loop && !looks_like_while {
            if let Some(inc) = extract_increment(&text, var_name) {
                info.increment = Some(inc);
                if !info.lines.contains(&line_no) {
                    info.lines.push(line_no);
                }
            }
        }
    }
    info.lines.sort();
    info.lines.dedup();
    info
}

/// Try to extract the RHS of an assignment like `int i = 1` or `i = 1`.
fn extract_assignment_rhs(line: &str, var: &str) -> Option<i32> {
    // Pattern: var = digits
    let pat = format!("{} = ", var);
    if let Some(pos) = line.find(&pat) {
        let rest = &line[pos + pat.len()..];
        let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit() || *c == '-').collect();
        return num_str.parse::<i32>().ok();
    }
    // Pattern: var=digits (no spaces)
    let pat2 = format!("{}=", var);
    if let Some(pos) = line.find(&pat2) {
        let rest = &line[pos + pat2.len()..];
        let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit() || *c == '-').collect();
        return num_str.parse::<i32>().ok();
    }
    None
}

/// Try to extract the increment step from `i++`, `i += 2`, `i = i + 2`, etc.
fn extract_increment(line: &str, var: &str) -> Option<i32> {
    let trimmed = line.trim();
    // i++ or ++i
    if trimmed == format!("{}++;", var) || trimmed == format!("++{};", var) {
        return Some(1);
    }
    // i += N
    let pat = format!("{} += ", var);
    if let Some(pos) = trimmed.find(&pat) {
        let rest = &trimmed[pos + pat.len()..];
        let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        return num_str.parse::<i32>().ok();
    }
    // i = i + N
    let pat2 = format!("{} = {} + ", var, var);
    if let Some(pos) = trimmed.find(&pat2) {
        let rest = &trimmed[pos + pat2.len()..];
        let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        return num_str.parse::<i32>().ok();
    }
    None
}

/// Infer the fine-grained bounds category from variable history and loop info.
fn infer_bounds_category(
    hist_vals: &[i32],
    accessed_index: i32,
    array_size: i32,
    loop_info: &LoopInfo,
) -> BoundsCategory {
    if hist_vals.len() < 2 {
        return BoundsCategory::UninitializedIndex;
    }

    let start_val = hist_vals.first().copied().unwrap_or(0);
    let is_monotonic_inc = hist_vals.windows(2).all(|w| w[1] >= w[0]);
    let diffs: Vec<i32> = hist_vals.windows(2).map(|w| w[1] - w[0]).collect();
    let max_step = diffs.iter().copied().max().unwrap_or(0);

    // Off-by-one: monotonic +1, ends exactly at array_size, and source has <=.
    if accessed_index == array_size && is_monotonic_inc && max_step == 1 && start_val == 0 && loop_info.has_le {
        return BoundsCategory::OffByOne;
    }

    // Wrong init: starts at >0, monotonic +1, and reaches or exceeds array size.
    if start_val > 0 && is_monotonic_inc && max_step == 1 && accessed_index >= array_size {
        return BoundsCategory::WrongInit;
    }

    // Wrong increment: step > 1 causes skip-over.
    if max_step > 1 && is_monotonic_inc {
        return BoundsCategory::WrongIncrement;
    }

    // Fallback: still OffByOne if classic pattern matches even without full history.
    if accessed_index == array_size && loop_info.has_le {
        return BoundsCategory::OffByOne;
    }

    BoundsCategory::Generic
}

/// Build the human-readable hint, fix-kind, fix-line and fix-desc from the inferred category.
fn build_bounds_hint(
    category: BoundsCategory,
    var_name: &str,
    array_name: &str,
    accessed_index: i32,
    array_size: i32,
    loop_info: &LoopInfo,
    hist_vals: &[i32],
) -> (String, String, Option<i32>, Option<String>) {
    match category {
        BoundsCategory::OffByOne => {
            let line = loop_info.lines.first().copied();
            let msg = if let Some(l) = line {
                format!(
                    "数组越界是因为第 {} 行的循环条件使用了 '<='。数组 '{}' 大小为 {}，最后一个有效索引是 {}。建议将 '<=' 改为 '<'。",
                    l, array_name, array_size, array_size.saturating_sub(1)
                )
            } else {
                format!(
                    "数组越界是因为循环条件使用了 '<='。数组 '{}' 大小为 {}，最后一个有效索引是 {}。建议将 '<=' 改为 '<'。",
                    array_name, array_size, array_size.saturating_sub(1)
                )
            };
            (msg, String::from("ChangeLeToLt"), line, Some(String::from("将 <= 改为 <")))
        }
        BoundsCategory::WrongInit => {
            let start_val = hist_vals.first().copied().unwrap_or(0);
            let line = loop_info.lines.first().copied();
            let msg = if let Some(l) = line {
                format!(
                    "数组 '{}' 访问了越界索引 {}。循环变量 '{}' 从 {} 开始，导致最终访问到索引 {}。建议将第 {} 行的初始值改为 0。",
                    array_name, accessed_index, var_name, start_val, accessed_index, l
                )
            } else {
                format!(
                    "数组 '{}' 访问了越界索引 {}。循环变量 '{}' 从 {} 开始，导致最终访问到索引 {}。建议将初始值改为 0。",
                    array_name, accessed_index, var_name, start_val, accessed_index
                )
            };
            (
                msg,
                String::from("FixLoopStart"),
                line,
                Some(format!("将 {} 的初始值 {} 改为 0", var_name, start_val)),
            )
        }
        BoundsCategory::WrongIncrement => {
            let step = loop_info.increment.unwrap_or(2);
            let line = loop_info.lines.first().copied();
            let msg = if let Some(l) = line {
                format!(
                    "数组 '{}' 访问了越界索引 {}。循环变量 '{}' 的步长是 {}，导致跳过了边界检查。建议将第 {} 行的步长改为 1，或调整循环条件。",
                    array_name, accessed_index, var_name, step, l
                )
            } else {
                format!(
                    "数组 '{}' 访问了越界索引 {}。循环变量 '{}' 的步长是 {}，导致跳过了边界检查。建议将步长改为 1，或调整循环条件。",
                    array_name, accessed_index, var_name, step
                )
            };
            (
                msg,
                String::from("FixLoopIncrement"),
                line,
                Some(format!("将步长 {} 改为 1", step)),
            )
        }
        BoundsCategory::UninitializedIndex => {
            let msg = format!(
                "索引变量 '{}' 的值是 {}，但执行历史太短，可能是变量未初始化或从外部输入读取了非法值。建议在使用前初始化 '{}'。",
                var_name, accessed_index, var_name
            );
            (msg, String::from("InitVariable"), None, Some(format!("初始化 '{}'", var_name)))
        }
        BoundsCategory::Generic => {
            let msg = if hist_vals.len() >= 2 {
                format!(
                    "数组 '{}' 访问了越界索引 {}。请检查 '{}' 的最近变化：{}。",
                    array_name,
                    accessed_index,
                    var_name,
                    format_history_vals(hist_vals)
                )
            } else {
                format!(
                    "数组 '{}' 访问了越界索引 {}。请检查循环边界或索引计算。",
                    array_name, accessed_index
                )
            };
            (msg, String::from("None"), None, None)
        }
    }
}

/// Format a short variable-history string from raw values.
fn format_history_vals(vals: &[i32]) -> String {
    if vals.len() <= 4 {
        vals.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" → ")
    } else {
        format!(
            "{} → ... → {}",
            vals.first().copied().unwrap_or(0),
            vals.last().copied().unwrap_or(0)
        )
    }
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
        .and_then(|u| u.source.lines().nth((line - 1) as usize).map(|s| s.trim().to_string()))
        .unwrap_or_default()
}

/// Names commonly used as loop / index variables.
fn is_likely_loop_var(name: &str) -> bool {
    matches!(
        name,
        "i" | "j"
            | "k"
            | "idx"
            | "index"
            | "m"
            | "n"
            | "left"
            | "right"
            | "mid"
            | "low"
            | "high"
            | "pivot"
            | "gap"
            | "l"
            | "r"
    )
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

    #[test]
    fn test_infer_off_by_one() {
        // History: i goes 0→1→2→3→4→5, array_size=5, accessed=5, source has <=
        let hist = vec![0, 1, 2, 3, 4, 5];
        let loop_info = LoopInfo {
            lines: vec![3],
            has_le: true,
            has_ge: false,
            start_val: Some(0),
            increment: Some(1),
        };
        let cat = infer_bounds_category(&hist, 5, 5, &loop_info);
        assert_eq!(cat, BoundsCategory::OffByOne);

        let (msg, fix, fix_line, fix_desc) = build_bounds_hint(cat, "i", "arr", 5, 5, &loop_info, &hist);
        assert!(msg.contains("<="));
        assert!(msg.contains("第 3 行"));
        assert_eq!(fix, "ChangeLeToLt");
        assert_eq!(fix_line, Some(3));
        assert_eq!(fix_desc, Some("将 <= 改为 <".to_string()));
    }

    #[test]
    fn test_infer_wrong_init() {
        // History: i goes 1→2→3→4→5, array_size=5, accessed=5 (should have been 0..4)
        let hist = vec![1, 2, 3, 4, 5];
        let loop_info = LoopInfo {
            lines: vec![3],
            has_le: false,
            has_ge: false,
            start_val: Some(1),
            increment: Some(1),
        };
        let cat = infer_bounds_category(&hist, 5, 5, &loop_info);
        assert_eq!(cat, BoundsCategory::WrongInit);

        let (msg, fix, fix_line, fix_desc) = build_bounds_hint(cat, "i", "arr", 5, 5, &loop_info, &hist);
        assert!(msg.contains("从 1 开始"));
        assert!(msg.contains("改为 0"));
        assert_eq!(fix, "FixLoopStart");
        assert_eq!(fix_line, Some(3));
        assert_eq!(fix_desc, Some("将 i 的初始值 1 改为 0".to_string()));
    }

    #[test]
    fn test_infer_wrong_increment() {
        // History: i goes 0→2→4→6, array_size=5, accessed=6 (step=2 skips over)
        let hist = vec![0, 2, 4, 6];
        let loop_info = LoopInfo {
            lines: vec![3, 5],
            has_le: false,
            has_ge: false,
            start_val: Some(0),
            increment: Some(2),
        };
        let cat = infer_bounds_category(&hist, 6, 5, &loop_info);
        assert_eq!(cat, BoundsCategory::WrongIncrement);

        let (msg, fix, fix_line, fix_desc) = build_bounds_hint(cat, "i", "arr", 6, 5, &loop_info, &hist);
        assert!(msg.contains("步长是 2"));
        assert_eq!(fix, "FixLoopIncrement");
        assert_eq!(fix_line, Some(3));
        assert_eq!(fix_desc, Some("将步长 2 改为 1".to_string()));
    }

    #[test]
    fn test_infer_uninitialized_index() {
        let hist: Vec<i32> = vec![];
        let loop_info = LoopInfo::default();
        let cat = infer_bounds_category(&hist, 99, 5, &loop_info);
        assert_eq!(cat, BoundsCategory::UninitializedIndex);

        let (msg, fix, fix_line, fix_desc) = build_bounds_hint(cat, "idx", "arr", 99, 5, &loop_info, &hist);
        assert!(msg.contains("未初始化"));
        assert_eq!(fix, "InitVariable");
        assert_eq!(fix_line, None);
        assert_eq!(fix_desc, Some("初始化 'idx'".to_string()));
    }

    #[test]
    fn test_extract_assignment_rhs() {
        assert_eq!(extract_assignment_rhs("int i = 0;", "i"), Some(0));
        assert_eq!(extract_assignment_rhs("int i = 5;", "i"), Some(5));
        assert_eq!(extract_assignment_rhs("i=3", "i"), Some(3));
        assert_eq!(extract_assignment_rhs("j = 10", "i"), None);
    }

    #[test]
    fn test_extract_increment() {
        assert_eq!(extract_increment("i++;", "i"), Some(1));
        assert_eq!(extract_increment("++i;", "i"), Some(1));
        assert_eq!(extract_increment("i += 2;", "i"), Some(2));
        assert_eq!(extract_increment("i = i + 3;", "i"), Some(3));
        assert_eq!(extract_increment("j += 2;", "i"), None);
    }
}
