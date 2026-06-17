//! 树算法检测

use super::features::{build_match, FuncFeatures};
use crate::session::AlgorithmMatch;

pub(crate) fn detect(name_lower: &str, features: &FuncFeatures, func_name: &str, line: i32) -> Vec<AlgorithmMatch> {
    let mut matches = Vec::new();

    // 二叉搜索树插入
    if name_lower.contains("bst")
        || name_lower.contains("insert") && features.is_recursive && features.cfg_has_back_edge
    {
        matches.push(build_match("bst_insert", "BST 插入", func_name, line, &features.compare_lines));
    }

    // BST 查找
    if name_lower.contains("search") && features.is_recursive && features.cfg_has_back_edge {
        matches.push(build_match("bst_search", "BST 查找", func_name, line, &features.compare_lines));
    }

    // 层序遍历
    if name_lower.contains("levelorder") || name_lower.contains("level_order") {
        matches.push(build_match("level_order", "层序遍历", func_name, line, &features.compare_lines));
    }

    // AVL 树
    if name_lower.contains("avl") {
        matches.push(build_match("avl_tree", "AVL 树", func_name, line, &features.compare_lines));
    }

    // 哈夫曼树
    if name_lower.contains("huffman") {
        matches.push(build_match(
            "huffman_tree",
            "哈夫曼树",
            func_name,
            line,
            &features.compare_lines,
        ));
    }

    // 线索二叉树
    if name_lower.contains("thread") && name_lower.contains("tree") {
        matches.push(build_match(
            "threaded_binary_tree",
            "线索二叉树",
            func_name,
            line,
            &features.compare_lines,
        ));
    }

    matches
}
