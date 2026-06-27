//! Use-After-Free trap analysis.

use crate::unified::root_cause::RootCauseHint;
use crate::unified::trace_analyzer::utils::parse_alloc_freed_lines;
use crate::unified::types::{PointerStatus, StepPayload};

/// Analyze a Use-After-Free trap and return a structured root-cause hint.
pub fn analyze_use_after_free(trap_message: &str, steps: &[StepPayload], trap_step: usize) -> Option<RootCauseHint> {
    let (alloc_line, freed_line) = parse_alloc_freed_lines(trap_message)?;
    let trap_payload = steps.get(trap_step)?;

    // Find pointer variables that are currently dangling / freed.
    let suspect_pointers: Vec<String> = trap_payload
        .pointer_snapshots
        .iter()
        .filter(|p| p.status == PointerStatus::Freed)
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
