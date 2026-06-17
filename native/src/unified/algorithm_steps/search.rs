use super::*;

// ============================================================================
// 二分查找
// ============================================================================

pub(crate) fn infer_binary_search(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let left = vars.get_int_any(&["left", "low", "l"]).unwrap_or(-1);
    let right = vars.get_int_any(&["right", "high", "r"]).unwrap_or(-1);
    let mid = vars.get_int_any(&["mid", "m"]).unwrap_or(-1);
    let target = vars
        .get_str("target")
        .or_else(|| vars.get_str("key"))
        .unwrap_or_else(|| "?".to_string());

    if line_lower.starts_with("while ") {
        let l = if left >= 0 { left.to_string() } else { "?".to_string() };
        let r = if right >= 0 { right.to_string() } else { "?".to_string() };
        return Some(build_step(algorithm, "loop", &format!("搜索范围 [{}, {}]", l, r)));
    }

    if line_lower.contains("mid") && (line_lower.contains('=') || line_lower.contains('/')) {
        let m = if mid >= 0 { mid.to_string() } else { "?".to_string() };
        return Some(build_step(algorithm, "mid_calc", &format!("计算中点 mid={}", m)));
    }

    if line_lower.starts_with("if ")
        && is_comparison_line(&line_lower)
        && (line_lower.contains("arr[") || line_lower.contains("a["))
    {
        let m = if mid >= 0 { mid.to_string() } else { "?".to_string() };
        return Some(build_step(
            algorithm,
            "compare",
            &format!("arr[{}] 与目标值 {} 比较", m, target),
        ));
    }

    if line_lower.contains("right") && line_lower.contains('=') && line_lower.contains("mid") {
        let m = if mid >= 0 { mid.to_string() } else { "?".to_string() };
        return Some(build_step(
            algorithm,
            "narrow_left",
            &format!("目标值在左半区，调整右边界 right={}", m),
        ));
    }

    if line_lower.contains("left") && line_lower.contains('=') && line_lower.contains("mid") {
        let m = if mid >= 0 {
            (mid + 1).to_string()
        } else {
            "?".to_string()
        };
        return Some(build_step(
            algorithm,
            "narrow_right",
            &format!("目标值在右半区，调整左边界 left={}", m),
        ));
    }

    if line_lower.starts_with("return ") && line_lower.contains("mid") {
        let m = if mid >= 0 { mid.to_string() } else { "?".to_string() };
        return Some(build_step(algorithm, "found", &format!("找到目标值，返回索引 {}", m)));
    }

    if line_lower.starts_with("return ") && (line_lower.contains("-1") || line_lower.contains("0")) {
        return Some(build_step(algorithm, "not_found", "搜索结束，未找到目标值"));
    }

    None
}
// ============================================================================
// 字符串反转
// ============================================================================

pub(crate) fn infer_string_reverse(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let len = vars.get_int_any(&["len", "length"]).unwrap_or(-1);
    let i = vars.get_int("i").unwrap_or(-1);

    if line_lower.starts_with("while ") && line_lower.contains("\\0") {
        return Some(build_step(algorithm, "measure", "扫描字符串，计算长度"));
    }

    if line_lower.starts_with("for ") && line_lower.contains("len") {
        return Some(build_step(
            algorithm,
            "swap",
            &format!("交换位置 {} 和 {}", i, if len >= 0 { len - i - 1 } else { -1 }),
        ));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "字符串反转完成"));
    }

    None
}
// ============================================================================
// 朴素模式匹配
// ============================================================================

pub(crate) fn infer_string_match_bf(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let i = vars.get_int_any(&["i"]).unwrap_or(-1);
    let j = vars.get_int_any(&["j"]).unwrap_or(-1);

    if line_lower.contains("s[") && line_lower.contains("t[") && line_lower.contains("==") {
        return Some(build_step(algorithm, "compare", &format!("比较 S[{}] 与 T[{}]", i, j)));
    }

    if line_lower.contains("i - j + 1") {
        return Some(build_step(algorithm, "backtrack", "字符不匹配，主串回溯"));
    }

    if line_lower.starts_with("return") && line_lower.contains("-1") {
        return Some(build_step(algorithm, "not_found", "模式匹配失败"));
    }

    None
}
// ============================================================================
// KMP 模式匹配
// ============================================================================

pub(crate) fn infer_string_match_kmp(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let j = vars.get_int_any(&["j"]).unwrap_or(-1);
    let k = vars.get_int_any(&["k"]).unwrap_or(-1);

    if line_lower.contains("getnext") || (line_lower.contains("next[") && line_lower.contains('=')) {
        return Some(build_step(
            algorithm,
            "build_next",
            &format!("构建 next 数组，next[{}]={}", j, k),
        ));
    }

    if line_lower.contains("s[") && line_lower.contains("t[") && line_lower.contains("==") {
        return Some(build_step(algorithm, "compare", "比较主串与模式串字符"));
    }

    if line_lower.contains("next[j]") {
        return Some(build_step(algorithm, "skip", &format!("j 回溯到 next[{}]={}", j, k)));
    }

    None
}
