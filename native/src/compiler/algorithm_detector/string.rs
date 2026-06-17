//! 字符串算法检测

use super::features::{build_match, FuncFeatures};
use crate::session::AlgorithmMatch;

pub(crate) fn detect(name_lower: &str, _features: &FuncFeatures, func_name: &str, line: i32) -> Vec<AlgorithmMatch> {
    let mut matches = Vec::new();

    // 字符串反转
    if name_lower.contains("reverse") && name_lower.contains("str") {
        matches.push(build_match("string_reverse", "字符串反转", func_name, line, &[]));
    }

    // 朴素模式匹配
    if name_lower.contains("indexbf") || name_lower.contains("bfmatch") || name_lower.contains("brute") {
        matches.push(build_match("string_match_bf", "朴素模式匹配", func_name, line, &[]));
    }

    // KMP 模式匹配
    if name_lower.contains("kmp") || name_lower.contains("indexkmp") || name_lower.contains("getnext") {
        matches.push(build_match("string_match_kmp", "KMP 模式匹配", func_name, line, &[]));
    }

    matches
}
