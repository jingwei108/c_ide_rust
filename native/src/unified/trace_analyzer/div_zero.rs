//! Division-by-zero trap analysis.

use crate::session::Session;
use crate::unified::root_cause::RootCauseHint;
use crate::unified::trace_analyzer::utils::is_likely_loop_var;
use crate::unified::types::StepPayload;

/// Analyze a division-by-zero trap and return a structured root-cause hint.
pub fn analyze_div_zero(
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
