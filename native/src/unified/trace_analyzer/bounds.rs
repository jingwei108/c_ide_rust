//! Array-bounds trap analysis.

use crate::session::Session;
use crate::unified::root_cause::RootCauseHint;
use crate::unified::trace_analyzer::utils::{
    extract_array_size, is_likely_loop_var, parse_bounds_message, scan_loop_info, slice_variable_history, LoopInfo,
};

/// Fine-grained sub-category for array-bounds traps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundsCategory {
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
    pub fn as_str(&self) -> &'static str {
        match self {
            BoundsCategory::OffByOne => "OffByOne",
            BoundsCategory::WrongInit => "WrongInit",
            BoundsCategory::WrongIncrement => "WrongIncrement",
            BoundsCategory::UninitializedIndex => "UninitializedIndex",
            BoundsCategory::Generic => "GenericBounds",
        }
    }
}

/// Analyze a runtime array-bounds trap and return a structured root-cause hint.
pub fn analyze_bounds(
    trap_message: &str,
    steps: &[crate::unified::types::StepPayload],
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

/// Infer the fine-grained bounds category from variable history and loop info.
pub(crate) fn infer_bounds_category(
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
pub(crate) fn build_bounds_hint(
    category: BoundsCategory,
    var_name: &str,
    array_name: &str,
    accessed_index: i32,
    array_size: i32,
    loop_info: &LoopInfo,
    hist_vals: &[i32],
) -> (String, String, Option<i32>, Option<String>) {
    use crate::unified::trace_analyzer::utils::format_history_vals;
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
