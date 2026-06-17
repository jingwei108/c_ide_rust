//! 数据结构相关算法检测

use super::features::{build_match, FuncFeatures};
use crate::session::AlgorithmMatch;

pub(crate) fn detect(name_lower: &str, features: &FuncFeatures, func_name: &str, line: i32) -> Vec<AlgorithmMatch> {
    let mut matches = Vec::new();

    // 链表操作（删除/插入等）
    if name_lower.contains("deletenode") || name_lower.contains("delete_node") {
        matches.push(build_match(
            "linked_list_delete",
            "链表删除",
            func_name,
            line,
            &features.compare_lines,
        ));
    }

    // 顺序表 / 数组操作
    if name_lower.contains("seqlist") || name_lower.contains("list_insert") || name_lower.contains("listdelete") {
        matches.push(build_match("seq_list", "顺序表", func_name, line, &features.compare_lines));
    }

    // 链表尾插 / 双向链表
    if name_lower.contains("append") && (name_lower.contains("list") || name_lower.contains("node")) {
        matches.push(build_match(
            "linked_list_append",
            "链表尾插",
            func_name,
            line,
            &features.compare_lines,
        ));
    }

    // 循环队列
    if name_lower.contains("circular") && name_lower.contains("queue") {
        matches.push(build_match(
            "circular_queue",
            "循环队列",
            func_name,
            line,
            &features.compare_lines,
        ));
    }

    // 链栈
    if name_lower.contains("linked") && name_lower.contains("stack") {
        matches.push(build_match("linked_stack", "链栈", func_name, line, &features.compare_lines));
    }

    // 链队列
    if name_lower.contains("linked") && name_lower.contains("queue") {
        matches.push(build_match("linked_queue", "链队列", func_name, line, &features.compare_lines));
    }

    // 哈希表
    if name_lower.contains("hash") && !name_lower.contains("cash") {
        matches.push(build_match("hash_table", "哈希表", func_name, line, &features.compare_lines));
    }

    // 约瑟夫环
    if name_lower.contains("josephus") {
        matches.push(build_match("josephus", "约瑟夫环", func_name, line, &features.compare_lines));
    }

    // 循环链表
    if name_lower.contains("circular") && name_lower.contains("list") {
        matches.push(build_match(
            "circular_linked_list",
            "循环链表",
            func_name,
            line,
            &features.compare_lines,
        ));
    }

    // 静态链表
    if name_lower.contains("static") && name_lower.contains("list") {
        matches.push(build_match(
            "static_linked_list",
            "静态链表",
            func_name,
            line,
            &features.compare_lines,
        ));
    }

    // 并查集
    if name_lower.contains("unionfind") || (name_lower.contains("union") && name_lower.contains("find")) {
        matches.push(build_match("union_find", "并查集", func_name, line, &features.compare_lines));
    }

    matches
}
