//! Double-Free trap analysis.

use crate::unified::root_cause::RootCauseHint;
use crate::unified::trace_analyzer::utils::parse_alloc_freed_lines;
use crate::unified::types::StepPayload;

/// Analyze a Double-Free trap and return a structured root-cause hint.
pub fn analyze_double_free(trap_message: &str, steps: &[StepPayload], trap_step: usize) -> Option<RootCauseHint> {
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
