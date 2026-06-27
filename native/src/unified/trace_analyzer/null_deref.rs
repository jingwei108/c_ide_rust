//! NULL pointer dereference trap analysis.

use crate::unified::root_cause::RootCauseHint;
use crate::unified::trace_analyzer::utils::slice_variable_history;
use crate::unified::types::{PointerStatus, StepPayload};

/// Analyze a NULL pointer dereference trap and return a structured root-cause hint.
pub fn analyze_null_deref(_trap_message: &str, steps: &[StepPayload], trap_step: usize) -> Option<RootCauseHint> {
    let trap_payload = steps.get(trap_step)?;

    // Find null pointers at the trap step.
    let null_ptrs: Vec<String> = trap_payload
        .pointer_snapshots
        .iter()
        .filter(|p| p.status == PointerStatus::Null)
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
                one_liner.push_str(&format!(" 注意：'{}' 之前指向过地址 {}，后来变成了 NULL。", first_ptr, h.value));
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
