//! 搜索算法检测

use super::features::{build_match, FuncFeatures};
use crate::session::AlgorithmMatch;

pub(crate) fn detect(name_lower: &str, features: &FuncFeatures, func_name: &str, line: i32) -> Vec<AlgorithmMatch> {
    let mut matches = Vec::new();

    // 二分查找
    if name_lower.contains("binary")
        || name_lower.contains("bsearch")
        || (features.has_single_loop && features.has_mid_calculation && features.has_left_right_update)
    {
        matches.push(build_match(
            "binary_search",
            "二分查找",
            func_name,
            line,
            &features.compare_lines,
        ));
    }

    matches
}
