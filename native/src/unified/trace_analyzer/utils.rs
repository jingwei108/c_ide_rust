//! Shared helpers for trace analysis.

use crate::session::Session;
use crate::unified::types::{ApiVariableSnapshot, StepPayload};

/// Information about a loop construct gathered from source scanning.
#[derive(Default)]
pub struct LoopInfo {
    /// Source lines that belong to the loop header / condition.
    pub lines: Vec<i32>,
    /// True if a `<=` operator is present in the condition.
    pub has_le: bool,
    /// True if a `>=` operator is present in the condition.
    pub has_ge: bool,
    /// Detected start value literal (e.g. `int i = 1` → Some(1)).
    pub start_val: Option<i32>,
    /// Detected increment literal (e.g. `i += 2` → Some(2), `i++` → Some(1)).
    pub increment: Option<i32>,
}

/// Extract variable snapshots for `var_name` from the `lookback` steps
/// immediately preceding (and including) `trap_step`.
pub fn slice_variable_history<'a>(
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
pub fn parse_bounds_message(msg: &str) -> Option<(String, i32)> {
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
pub fn extract_array_size(msg: &str) -> Option<i32> {
    // Message contains "只有 {} 个元素"
    let start = msg.find("只有 ")? + "只有 ".len();
    let rest = &msg[start..];
    let end = rest.find(" 个元素")?;
    rest[..end].parse::<i32>().ok()
}

/// Parse alloc_line and freed_line from UAF / Double-Free messages.
/// Both formats contain "第 {} 行被 free" and "由第 {} 行的 malloc".
pub fn parse_alloc_freed_lines(msg: &str) -> Option<(i32, i32)> {
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

/// Scan source lines around `trap_line` looking for loop constructs that involve
/// `var_name` and/or `array_name`.
pub fn scan_loop_info(session: &Session, trap_line: i32, var_name: &str, array_name: &str) -> LoopInfo {
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
pub fn extract_assignment_rhs(line: &str, var: &str) -> Option<i32> {
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
pub fn extract_increment(line: &str, var: &str) -> Option<i32> {
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

/// Retrieve a source line from the first compile unit (matches collector.rs logic).
pub fn get_source_line(session: &Session, line: i32) -> String {
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

/// Format a short variable-history string from raw values.
pub fn format_history_vals(vals: &[i32]) -> String {
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

/// Names commonly used as loop / index variables.
pub fn is_likely_loop_var(name: &str) -> bool {
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
