use crate::*;

// 冒泡排序
// ============================================================================

pub(crate) fn infer_bubble_sort(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let i = vars.get_int("i").unwrap_or(-1);
    let j = vars.get_int("j").unwrap_or(-1);
    let n = vars.get_int_any(&["n", "len", "size", "length"]).unwrap_or(-1);

    if line_lower.starts_with("for ") || line_lower.starts_with("while ") {
        if line_lower.contains('i') && !line_lower.contains('j') {
            let pass = if i >= 0 { (i + 1).to_string() } else { "?".to_string() };
            let kth = if n > 0 && i >= 0 {
                (n - i).to_string()
            } else {
                "?".to_string()
            };
            return Some(build_step(
                algorithm,
                "outer_loop",
                &format!("第 {} 趟：将第 {} 大的元素放到正确位置", pass, kth),
            ));
        }
        if line_lower.contains('j') {
            let j_str = if j >= 0 { j.to_string() } else { "?".to_string() };
            return Some(build_step(
                algorithm,
                "inner_loop",
                &format!("内层循环 j={}，比较相邻元素", j_str),
            ));
        }
    }

    if line_lower.starts_with("if ") && is_comparison_line(&line_lower) {
        let j_str = if j >= 0 { j.to_string() } else { "?".to_string() };
        let j1 = if j >= 0 { (j + 1).to_string() } else { "?".to_string() };
        return Some(build_step(algorithm, "compare", &format!("比较 arr[{}] 与 arr[{}]", j_str, j1)));
    }

    if source_line.contains("temp") && source_line.contains('=') {
        let j_str = if j >= 0 { j.to_string() } else { "?".to_string() };
        let j1 = if j >= 0 { (j + 1).to_string() } else { "?".to_string() };
        return Some(build_step(
            algorithm,
            "swap",
            &format!("交换 arr[{}]↔arr[{}]，较大的元素向右移动", j_str, j1),
        ));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "排序完成"));
    }

    None
}

// ============================================================================
// 选择排序
// ============================================================================

pub(crate) fn infer_selection_sort(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let i = vars.get_int("i").unwrap_or(-1);
    let j = vars.get_int("j").unwrap_or(-1);
    let min_idx = vars.get_int_any(&["min_idx", "minIndex", "min", "minindex"]).unwrap_or(-1);

    if line_lower.starts_with("for ") || line_lower.starts_with("while ") {
        if line_lower.contains('i') && !line_lower.contains('j') {
            let pass = if i >= 0 { i.to_string() } else { "?".to_string() };
            return Some(build_step(
                algorithm,
                "outer_loop",
                &format!("第 {} 趟：从第 {} 个位置开始找最小值", pass, pass),
            ));
        }
        if line_lower.contains('j') {
            let j_str = if j >= 0 { j.to_string() } else { "?".to_string() };
            let min_str = if min_idx >= 0 {
                min_idx.to_string()
            } else {
                "?".to_string()
            };
            return Some(build_step(
                algorithm,
                "inner_loop",
                &format!("扫描 j={}，当前最小值在 min_idx={}", j_str, min_str),
            ));
        }
    }

    if line_lower.starts_with("if ") && is_comparison_line(&line_lower) {
        let j_str = if j >= 0 { j.to_string() } else { "?".to_string() };
        let min_str = if min_idx >= 0 {
            min_idx.to_string()
        } else {
            "?".to_string()
        };
        return Some(build_step(
            algorithm,
            "compare",
            &format!("比较 arr[{}] 与当前最小值 arr[{}]", j_str, min_str),
        ));
    }

    if source_line.contains("temp") && source_line.contains('=') {
        let i_str = if i >= 0 { i.to_string() } else { "?".to_string() };
        return Some(build_step(algorithm, "swap", &format!("将最小元素交换到位置 {}", i_str)));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "排序完成"));
    }

    None
}

// ============================================================================
// 插入排序
// ============================================================================

pub(crate) fn infer_insertion_sort(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let i = vars.get_int("i").unwrap_or(-1);
    let j = vars.get_int("j").unwrap_or(-1);
    let key = vars.get_str("key").unwrap_or_else(|| "?".to_string());

    if line_lower.starts_with("for ") || line_lower.starts_with("while ") {
        if line_lower.contains('i') && !line_lower.contains('j') {
            let pass = if i >= 0 { i.to_string() } else { "?".to_string() };
            return Some(build_step(
                algorithm,
                "outer_loop",
                &format!("第 {} 个元素：准备插入到已排序部分", pass),
            ));
        }
        if line_lower.contains('j') && line_lower.contains('[') {
            let j_str = if j >= 0 { j.to_string() } else { "?".to_string() };
            return Some(build_step(
                algorithm,
                "inner_loop",
                &format!("元素后移 j={}，为插入腾出位置", j_str),
            ));
        }
    }

    if line_lower.starts_with("if ") && is_comparison_line(&line_lower) {
        let j_str = if j >= 0 { j.to_string() } else { "?".to_string() };
        return Some(build_step(algorithm, "compare", &format!("比较 arr[{}] 与 key={}", j_str, key)));
    }

    if (source_line.contains("arr[") || source_line.contains("a["))
        && source_line.contains('=')
        && !line_lower.starts_with("for ")
        && !line_lower.starts_with("while ")
    {
        let pos = if j >= 0 { (j + 1).to_string() } else { "?".to_string() };
        return Some(build_step(
            algorithm,
            "insert",
            &format!("将 key={} 插入到正确位置 {}", key, pos),
        ));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "排序完成"));
    }

    None
}

// ============================================================================
// 快速排序
// ============================================================================

pub(crate) fn infer_quick_sort(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
    func_name: &str,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let left = vars.get_int_any(&["left", "low", "l"]).unwrap_or(-1);
    let right = vars.get_int_any(&["right", "high", "r"]).unwrap_or(-1);
    let pivot = vars.get_int_any(&["pivot", "p"]).unwrap_or(-1);
    let i = vars.get_int("i").unwrap_or(-1);

    // 递归调用自身
    if source_line.contains(&format!("{}(", func_name)) {
        let side = if i >= 0 && pivot >= 0 {
            if i < pivot {
                "左"
            } else {
                "右"
            }
        } else {
            "子"
        };
        return Some(build_step(
            algorithm,
            "recursive",
            &format!("递归调用 {}，处理{}子数组 [left={}, right={}]", func_name, side, left, right),
        ));
    }

    if line_lower.contains("pivot") && line_lower.contains('=') && !line_lower.starts_with("for ") {
        let p_str = if pivot >= 0 { pivot.to_string() } else { "?".to_string() };
        return Some(build_step(
            algorithm,
            "partition_init",
            &format!("分区：选取枢轴 pivot={}", p_str),
        ));
    }

    if (line_lower.starts_with("while ") || line_lower.starts_with("for "))
        && (line_lower.contains('i') || line_lower.contains('j'))
    {
        return Some(build_step(algorithm, "partition_scan", "分区扫描：将小于枢轴的元素放到左侧"));
    }

    if source_line.contains("temp") && source_line.contains('=') {
        return Some(build_step(algorithm, "partition_swap", "交换元素，调整分区位置"));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "快速排序完成"));
    }

    None
}

// ============================================================================
// 归并排序
// ============================================================================

pub(crate) fn infer_merge_sort(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
    func_name: &str,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let left = vars.get_int_any(&["left", "low", "l", "start"]).unwrap_or(-1);
    let right = vars.get_int_any(&["right", "high", "r", "end"]).unwrap_or(-1);

    if source_line.contains(&format!("{}(", func_name)) {
        return Some(build_step(
            algorithm,
            "recursive_split",
            &format!("将数组区间 [{}, {}] 递归分成两半", left, right),
        ));
    }

    if line_lower.contains("merge") && line_lower.contains('(') {
        return Some(build_step(algorithm, "merge", "合并两个有序子数组"));
    }

    if (line_lower.starts_with("while ") || line_lower.starts_with("for "))
        && (line_lower.contains('i') || line_lower.contains('j') || line_lower.contains('k'))
    {
        return Some(build_step(algorithm, "merge", "合并两个有序子数组"));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "归并排序完成"));
    }

    None
}

// ============================================================================
// 堆排序
// ============================================================================

pub(crate) fn infer_heap_sort(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
    func_name: &str,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let _i = vars.get_int_any(&["i", "largest"]).unwrap_or(-1);
    let _n = vars.get_int_any(&["n", "len", "size"]).unwrap_or(-1);

    if source_line.contains(&format!("{}(", func_name)) && func_name != "heapSort" {
        return Some(build_step(algorithm, "heapify", "递归堆化：确保子树满足堆性质"));
    }

    if line_lower.starts_with("for ") || line_lower.starts_with("while ") {
        if line_lower.contains("n / 2") || line_lower.contains("mid") {
            return Some(build_step(algorithm, "build_heap", "建堆：自底向上将数组调整为最大堆"));
        }
        if line_lower.contains("i") && line_lower.contains("n - 1") {
            return Some(build_step(algorithm, "extract", "取出堆顶元素并重新堆化"));
        }
    }

    if source_line.contains("temp") && source_line.contains('=') {
        return Some(build_step(algorithm, "swap", "交换元素，调整堆结构"));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "堆排序完成"));
    }

    None
}
// ============================================================================
// 希尔排序
// ============================================================================

pub(crate) fn infer_shell_sort(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let gap = vars.get_int_any(&["gap"]).unwrap_or(-1);

    if line_lower.starts_with("for ") && line_lower.contains("gap") {
        return Some(build_step(
            algorithm,
            "outer_loop",
            &format!("取增量 gap={}，分组进行插入排序", gap),
        ));
    }

    if line_lower.contains("temp") && line_lower.contains('=') && !line_lower.starts_with("for ") {
        return Some(build_step(algorithm, "insert", "保存当前元素，准备在同组内插入"));
    }

    if line_lower.contains("j") && line_lower.contains("gap") && line_lower.contains('[') {
        return Some(build_step(algorithm, "inner_loop", "同组内元素后移，腾出插入位置"));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "希尔排序完成"));
    }

    None
}

// ============================================================================
// 计数排序
// ============================================================================

pub(crate) fn infer_counting_sort(
    source_line: &str,
    _vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();

    if line_lower.starts_with("for ") && line_lower.contains("count") && line_lower.contains("++") {
        return Some(build_step(algorithm, "count", "统计每个数值的出现次数"));
    }

    if line_lower.starts_with("for ") && line_lower.contains("i") && line_lower.contains("10") {
        return Some(build_step(algorithm, "collect", "按数值从小到大收集元素"));
    }

    if line_lower.contains("index") && line_lower.contains('=') {
        return Some(build_step(algorithm, "place", "将数值放回原数组的正确位置"));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "计数排序完成"));
    }

    None
}

// ============================================================================
// 链表删除
// ============================================================================

// ============================================================================
// 基数排序
// ============================================================================

pub(crate) fn infer_radix_sort(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let exp = vars.get_int_any(&["exp"]).unwrap_or(-1);

    if line_lower.contains("exp") && line_lower.contains("max") && line_lower.contains("/") {
        return Some(build_step(algorithm, "digit_loop", &format!("按第 {} 位进行分配-收集", exp)));
    }

    if line_lower.contains("count[") && line_lower.contains("++") && !line_lower.contains("+=") {
        return Some(build_step(algorithm, "count", "统计当前位各数字出现次数"));
    }

    if line_lower.contains("count[i]") && line_lower.contains("count[i - 1]") {
        return Some(build_step(algorithm, "prefix", "计算前缀和，确定位置"));
    }

    if line_lower.contains("output[") && line_lower.contains("count[") && line_lower.contains('=') {
        return Some(build_step(algorithm, "place", "按前缀和放置元素"));
    }

    None
}
