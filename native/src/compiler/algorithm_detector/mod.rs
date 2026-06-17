//! 算法模式检测器
//!
//! 基于 AST 进行启发式算法识别，支持排序、图、树、搜索、字符串、数学及常见数据结构。

use crate::compiler::ast::ProgramNode;
use crate::session::AlgorithmMatch;

pub(crate) mod features;
pub(crate) mod graph;
pub(crate) mod math;
pub(crate) mod search;
pub(crate) mod sorting;
pub(crate) mod string;
pub(crate) mod structures;
pub(crate) mod tree;

/// 检测程序中的所有算法模式
pub fn detect_algorithms(program: &ProgramNode) -> Vec<AlgorithmMatch> {
    let mut matches = Vec::new();
    for func in &program.funcs {
        matches.extend(detect_in_func(func));
    }
    matches
}

fn detect_in_func(func: &crate::compiler::ast::FuncDecl) -> Vec<AlgorithmMatch> {
    let body = match func.body.as_ref() {
        Some(b) => b,
        None => return Vec::new(),
    };
    let features = features::extract_features(func, body);

    let name_lower = func.name.to_lowercase();
    let mut matches = Vec::new();
    matches.extend(sorting::detect(&name_lower, &features, &func.name, func.loc.line));
    matches.extend(search::detect(&name_lower, &features, &func.name, func.loc.line));
    matches.extend(graph::detect(&name_lower, &features, &func.name, func.loc.line));
    matches.extend(tree::detect(&name_lower, &features, &func.name, func.loc.line));
    matches.extend(structures::detect(&name_lower, &features, &func.name, func.loc.line));
    matches.extend(string::detect(&name_lower, &features, &func.name, func.loc.line));
    matches.extend(math::detect(&name_lower, &features, &func.name, func.loc.line));

    matches
}
