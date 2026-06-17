//! 数学算法检测

use super::features::{build_match, FuncFeatures};
use crate::session::AlgorithmMatch;

pub(crate) fn detect(name_lower: &str, _features: &FuncFeatures, func_name: &str, line: i32) -> Vec<AlgorithmMatch> {
    let mut matches = Vec::new();

    // 最大公约数
    if name_lower.contains("gcd") || name_lower.contains("greatest") {
        matches.push(build_match("gcd", "最大公约数", func_name, line, &[]));
    }

    // 素数判断
    if name_lower.contains("prime") || name_lower.contains("isprime") {
        matches.push(build_match("is_prime", "素数判断", func_name, line, &[]));
    }

    // 汉诺塔
    if name_lower.contains("hanoi") {
        matches.push(build_match("hanoi", "汉诺塔", func_name, line, &[]));
    }

    matches
}
