use super::*;

// ============================================================================
// 动态规划
// ============================================================================

pub(crate) fn infer_dp(source_line: &str, vars: &VarMap, algorithm: &AlgorithmMatch) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let i = vars.get_int_any(&["i", "n"]).unwrap_or(-1);
    let w = vars.get_int_any(&["w", "j", "capacity"]).unwrap_or(-1);

    if line_lower.starts_with("for ") || line_lower.starts_with("while ") {
        if line_lower.contains("i") && line_lower.contains("n") {
            return Some(build_step(algorithm, "outer_loop", &format!("遍历物品 i={}", i)));
        }
        if line_lower.contains("w") || line_lower.contains("j") {
            return Some(build_step(algorithm, "inner_loop", &format!("遍历容量 w={}", w)));
        }
    }

    if line_lower.contains("dp[") && line_lower.contains('=') {
        return Some(build_step(algorithm, "transition", "状态转移：计算当前子问题的最优解"));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "动态规划计算完成"));
    }

    None
}
