//! 图算法检测

use super::features::{build_match, FuncFeatures};
use crate::session::AlgorithmMatch;

pub(crate) fn detect(name_lower: &str, features: &FuncFeatures, func_name: &str, line: i32) -> Vec<AlgorithmMatch> {
    let mut matches = Vec::new();

    // BFS 广度优先搜索
    if name_lower.contains("bfs")
        || name_lower.contains("breadth")
        || (name_lower.contains("search") && features.has_single_loop && features.cfg_has_back_edge)
    {
        matches.push(build_match("bfs", "BFS 广度优先搜索", func_name, line, &features.compare_lines));
    }

    // DFS 深度优先搜索
    if name_lower.contains("dfs")
        || name_lower.contains("depth")
        || (features.is_recursive
            && !features.has_partition_pattern
            && !features.has_merge_pattern
            && name_lower.contains("search"))
    {
        matches.push(build_match("dfs", "DFS 深度优先搜索", func_name, line, &features.compare_lines));
    }

    // Prim 最小生成树
    if name_lower.contains("prim") {
        matches.push(build_match(
            "prim_mst",
            "Prim 最小生成树",
            func_name,
            line,
            &features.compare_lines,
        ));
    }

    // Kruskal 最小生成树
    if name_lower.contains("kruskal") {
        matches.push(build_match(
            "kruskal_mst",
            "Kruskal 最小生成树",
            func_name,
            line,
            &features.compare_lines,
        ));
    }

    // Dijkstra 最短路径
    if name_lower.contains("dijkstra") {
        matches.push(build_match(
            "dijkstra",
            "Dijkstra 最短路径",
            func_name,
            line,
            &features.compare_lines,
        ));
    }

    // Floyd 最短路径
    if name_lower.contains("floyd") {
        matches.push(build_match("floyd", "Floyd 最短路径", func_name, line, &features.compare_lines));
    }

    // 拓扑排序
    if name_lower.contains("topolog") {
        matches.push(build_match(
            "topological_sort",
            "拓扑排序",
            func_name,
            line,
            &features.compare_lines,
        ));
    }

    matches
}
