//! 排序算法检测

use super::features::{build_match, FuncFeatures};
use crate::session::AlgorithmMatch;

pub(crate) fn detect(name_lower: &str, features: &FuncFeatures, func_name: &str, line: i32) -> Vec<AlgorithmMatch> {
    let mut matches = Vec::new();

    // 冒泡排序
    if name_lower.contains("bubble")
        || (features.has_nested_loops
            && features.has_array_compare
            && features.has_swap
            && features.loop_depth >= 2
            && features.has_adjacent_index_compare)
    {
        matches.push(build_match("bubble_sort", "冒泡排序", func_name, line, &features.compare_lines));
    }

    // 选择排序
    if name_lower.contains("select")
        || (features.has_nested_loops
            && features.has_array_compare
            && features.has_min_max_track
            && features.loop_depth >= 2
            && !features.has_swap_in_inner_loop)
    {
        matches.push(build_match(
            "selection_sort",
            "选择排序",
            func_name,
            line,
            &features.compare_lines,
        ));
    }

    // 插入排序
    if name_lower.contains("insert")
        || (features.has_nested_loops && features.has_shift_pattern && features.loop_depth >= 2 && !features.has_swap)
    {
        matches.push(build_match(
            "insertion_sort",
            "插入排序",
            func_name,
            line,
            &features.compare_lines,
        ));
    }

    // 快速排序
    if name_lower.contains("quick")
        || (features.is_recursive && features.has_partition_pattern && features.has_nested_loops)
    {
        matches.push(build_match("quick_sort", "快速排序", func_name, line, &features.compare_lines));
    }

    // 归并排序
    if name_lower.contains("merge")
        || (features.is_recursive && features.has_merge_pattern && !features.has_partition_pattern)
    {
        matches.push(build_match("merge_sort", "归并排序", func_name, line, &features.compare_lines));
    }

    // 堆排序
    if name_lower.contains("heap")
        || (features.is_recursive && features.has_array_compare && features.has_swap && name_lower.contains("sort"))
    {
        matches.push(build_match("heap_sort", "堆排序", func_name, line, &features.compare_lines));
    }

    // 希尔排序
    if name_lower.contains("shell") {
        matches.push(build_match("shell_sort", "希尔排序", func_name, line, &features.compare_lines));
    }

    // 计数排序
    if name_lower.contains("counting") {
        matches.push(build_match(
            "counting_sort",
            "计数排序",
            func_name,
            line,
            &features.compare_lines,
        ));
    }

    // 基数排序
    if name_lower.contains("radix") {
        matches.push(build_match("radix_sort", "基数排序", func_name, line, &features.compare_lines));
    }

    matches
}
