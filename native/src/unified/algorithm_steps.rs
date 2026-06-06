//! 算法步骤语义标注
//!
//! 为已检测到的算法（冒泡排序、选择排序、插入排序、快速排序、归并排序、二分查找）
//! 生成教学友好的步骤描述，结合当前源码行特征和运行时变量值。

use crate::session::{AlgorithmMatch, Session};
use crate::unified::types::{AlgorithmStepSnapshot, ApiVariableSnapshot};

/// 推断当前执行步骤对应的算法语义描述。
pub fn infer_algorithm_step(
    code_line: i32,
    local_vars: &[ApiVariableSnapshot],
    func_name: &str,
    session: &Session,
) -> Option<AlgorithmStepSnapshot> {
    if code_line <= 0 || func_name.is_empty() {
        return None;
    }

    let source_line = get_source_line(session, code_line).unwrap_or_default();
    let algorithm = find_algorithm_for_func(session, func_name)?;
    let vars = VarMap::new(local_vars);

    match algorithm.name.as_str() {
        "bubble_sort" => infer_bubble_sort(&source_line, &vars, algorithm),
        "selection_sort" => infer_selection_sort(&source_line, &vars, algorithm),
        "insertion_sort" => infer_insertion_sort(&source_line, &vars, algorithm),
        "quick_sort" => infer_quick_sort(&source_line, &vars, algorithm, func_name),
        "merge_sort" => infer_merge_sort(&source_line, &vars, algorithm, func_name),
        "binary_search" => infer_binary_search(&source_line, &vars, algorithm),
        "heap_sort" => infer_heap_sort(&source_line, &vars, algorithm, func_name),
        "bfs" => infer_bfs(&source_line, &vars, algorithm),
        "dfs" => infer_dfs(&source_line, &vars, algorithm, func_name),
        "dp" => infer_dp(&source_line, &vars, algorithm),
        "shell_sort" => infer_shell_sort(&source_line, &vars, algorithm),
        "counting_sort" => infer_counting_sort(&source_line, &vars, algorithm),
        "linked_list_delete" => infer_linked_list_delete(&source_line, &vars, algorithm),
        "bst_insert" => infer_bst_insert(&source_line, &vars, algorithm, func_name),
        "string_reverse" => infer_string_reverse(&source_line, &vars, algorithm),
        "gcd" => infer_gcd(&source_line, &vars, algorithm),
        "is_prime" => infer_is_prime(&source_line, &vars, algorithm),
        "hanoi" => infer_hanoi(&source_line, &vars, algorithm, func_name),
        "seq_list" => infer_seq_list(&source_line, &vars, algorithm),
        "linked_list_append" => infer_linked_list_append(&source_line, &vars, algorithm),
        "circular_queue" => infer_circular_queue(&source_line, &vars, algorithm),
        "linked_stack" => infer_linked_stack(&source_line, &vars, algorithm),
        "linked_queue" => infer_linked_queue(&source_line, &vars, algorithm),
        "level_order" => infer_level_order(&source_line, &vars, algorithm),
        "bst_search" => infer_bst_search(&source_line, &vars, algorithm, func_name),
        "hash_table" => infer_hash_table(&source_line, &vars, algorithm),
        "josephus" => infer_josephus(&source_line, &vars, algorithm),
        _ => None,
    }
}

// ============================================================================
// 工具函数
// ============================================================================

fn get_source_line(session: &Session, code_line: i32) -> Option<String> {
    let unit = session.compile.compile_units.first()?;
    let line = unit.source.lines().nth((code_line - 1) as usize)?;
    Some(line.trim().to_string())
}

fn find_algorithm_for_func<'a>(session: &'a Session, func_name: &str) -> Option<&'a AlgorithmMatch> {
    session
        .compile
        .algorithm_matches
        .iter()
        .find(|m| m.func_name == func_name)
}

struct VarMap<'a> {
    vars: &'a [ApiVariableSnapshot],
}

impl<'a> VarMap<'a> {
    fn new(vars: &'a [ApiVariableSnapshot]) -> Self {
        Self { vars }
    }

    fn get_int(&self, name: &str) -> Option<i32> {
        self.vars
            .iter()
            .find(|v| v.name == name)
            .and_then(|v| v.value.parse::<i32>().ok())
    }

    fn get_int_any(&self, names: &[&str]) -> Option<i32> {
        for name in names {
            if let Some(v) = self.get_int(name) {
                return Some(v);
            }
        }
        None
    }

    fn get_str(&self, name: &str) -> Option<String> {
        self.vars
            .iter()
            .find(|v| v.name == name)
            .map(|v| v.value.clone())
    }
}

fn is_comparison_line(line: &str) -> bool {
    line.contains('>') || line.contains('<') || line.contains("==") || line.contains("!=")
}

fn build_step(algorithm: &AlgorithmMatch, phase: &str, description: &str) -> AlgorithmStepSnapshot {
    AlgorithmStepSnapshot {
        algorithm_name: algorithm.name.clone(),
        display_name: algorithm.display_name.clone(),
        phase: phase.to_string(),
        description: description.to_string(),
    }
}

// ============================================================================
// 冒泡排序
// ============================================================================

fn infer_bubble_sort(
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
            let pass = if i >= 0 {
                (i + 1).to_string()
            } else {
                "?".to_string()
            };
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
            let j_str = if j >= 0 {
                j.to_string()
            } else {
                "?".to_string()
            };
            return Some(build_step(
                algorithm,
                "inner_loop",
                &format!("内层循环 j={}，比较相邻元素", j_str),
            ));
        }
    }

    if line_lower.starts_with("if ") && is_comparison_line(&line_lower) {
        let j_str = if j >= 0 {
            j.to_string()
        } else {
            "?".to_string()
        };
        let j1 = if j >= 0 {
            (j + 1).to_string()
        } else {
            "?".to_string()
        };
        return Some(build_step(
            algorithm,
            "compare",
            &format!("比较 arr[{}] 与 arr[{}]", j_str, j1),
        ));
    }

    if source_line.contains("temp") && source_line.contains('=') {
        let j_str = if j >= 0 {
            j.to_string()
        } else {
            "?".to_string()
        };
        let j1 = if j >= 0 {
            (j + 1).to_string()
        } else {
            "?".to_string()
        };
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

fn infer_selection_sort(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let i = vars.get_int("i").unwrap_or(-1);
    let j = vars.get_int("j").unwrap_or(-1);
    let min_idx = vars
        .get_int_any(&["min_idx", "minIndex", "min", "minindex"])
        .unwrap_or(-1);

    if line_lower.starts_with("for ") || line_lower.starts_with("while ") {
        if line_lower.contains('i') && !line_lower.contains('j') {
            let pass = if i >= 0 {
                i.to_string()
            } else {
                "?".to_string()
            };
            return Some(build_step(
                algorithm,
                "outer_loop",
                &format!("第 {} 趟：从第 {} 个位置开始找最小值", pass, pass),
            ));
        }
        if line_lower.contains('j') {
            let j_str = if j >= 0 {
                j.to_string()
            } else {
                "?".to_string()
            };
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
        let j_str = if j >= 0 {
            j.to_string()
        } else {
            "?".to_string()
        };
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
        let i_str = if i >= 0 {
            i.to_string()
        } else {
            "?".to_string()
        };
        return Some(build_step(
            algorithm,
            "swap",
            &format!("将最小元素交换到位置 {}", i_str),
        ));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "排序完成"));
    }

    None
}

// ============================================================================
// 插入排序
// ============================================================================

fn infer_insertion_sort(
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
            let pass = if i >= 0 {
                i.to_string()
            } else {
                "?".to_string()
            };
            return Some(build_step(
                algorithm,
                "outer_loop",
                &format!("第 {} 个元素：准备插入到已排序部分", pass),
            ));
        }
        if line_lower.contains('j') && line_lower.contains('[') {
            let j_str = if j >= 0 {
                j.to_string()
            } else {
                "?".to_string()
            };
            return Some(build_step(
                algorithm,
                "inner_loop",
                &format!("元素后移 j={}，为插入腾出位置", j_str),
            ));
        }
    }

    if line_lower.starts_with("if ") && is_comparison_line(&line_lower) {
        let j_str = if j >= 0 {
            j.to_string()
        } else {
            "?".to_string()
        };
        return Some(build_step(
            algorithm,
            "compare",
            &format!("比较 arr[{}] 与 key={}", j_str, key),
        ));
    }

    if (source_line.contains("arr[") || source_line.contains("a["))
        && source_line.contains('=')
        && !line_lower.starts_with("for ")
        && !line_lower.starts_with("while ")
    {
        let pos = if j >= 0 {
            (j + 1).to_string()
        } else {
            "?".to_string()
        };
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

fn infer_quick_sort(
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
            &format!(
                "递归调用 {}，处理{}子数组 [left={}, right={}]",
                func_name, side, left, right
            ),
        ));
    }

    if line_lower.contains("pivot") && line_lower.contains('=') && !line_lower.starts_with("for ") {
        let p_str = if pivot >= 0 {
            pivot.to_string()
        } else {
            "?".to_string()
        };
        return Some(build_step(
            algorithm,
            "partition_init",
            &format!("分区：选取枢轴 pivot={}", p_str),
        ));
    }

    if (line_lower.starts_with("while ") || line_lower.starts_with("for "))
        && (line_lower.contains('i') || line_lower.contains('j'))
    {
        return Some(build_step(
            algorithm,
            "partition_scan",
            "分区扫描：将小于枢轴的元素放到左侧",
        ));
    }

    if source_line.contains("temp") && source_line.contains('=') {
        return Some(build_step(
            algorithm,
            "partition_swap",
            "交换元素，调整分区位置",
        ));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "快速排序完成"));
    }

    None
}

// ============================================================================
// 归并排序
// ============================================================================

fn infer_merge_sort(
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
        && (line_lower.contains('i')
            || line_lower.contains('j')
            || line_lower.contains('k'))
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

fn infer_heap_sort(
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
// BFS
// ============================================================================

fn infer_bfs(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let u = vars.get_int_any(&["u", "front", "start"]).unwrap_or(-1);
    let front = vars.get_int("front").unwrap_or(-1);
    let rear = vars.get_int("rear").unwrap_or(-1);

    if line_lower.starts_with("while ") {
        return Some(build_step(
            algorithm,
            "loop",
            &format!("队列非空，继续广度优先搜索 [front={}, rear={}]", front, rear),
        ));
    }

    if line_lower.contains("queue[front++]") || (line_lower.contains("front") && line_lower.contains("++")) {
        return Some(build_step(algorithm, "dequeue", &format!("出队节点 u={}", u)));
    }

    if line_lower.contains("queue[rear++]") || (line_lower.contains("rear") && line_lower.contains("++")) {
        return Some(build_step(algorithm, "enqueue", "邻居节点入队"));
    }

    if line_lower.contains("visited") && line_lower.contains('=') {
        return Some(build_step(algorithm, "visit", &format!("标记节点 {} 为已访问", u)));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "BFS 遍历完成"));
    }

    None
}

// ============================================================================
// DFS
// ============================================================================

fn infer_dfs(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
    func_name: &str,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let u = vars.get_int_any(&["u", "v", "start"]).unwrap_or(-1);

    if source_line.contains(&format!("{}(", func_name)) {
        return Some(build_step(
            algorithm,
            "recursive",
            &format!("递归深入：从节点 {} 继续深度优先搜索", u),
        ));
    }

    if line_lower.contains("visited") && line_lower.contains('=') {
        return Some(build_step(algorithm, "visit", &format!("标记节点 {} 为已访问", u)));
    }

    if line_lower.starts_with("for ") || line_lower.starts_with("while ") {
        return Some(build_step(algorithm, "scan", &format!("扫描节点 {} 的邻居", u)));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "DFS 遍历完成"));
    }

    None
}

// ============================================================================
// 动态规划
// ============================================================================

fn infer_dp(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let i = vars.get_int_any(&["i", "n"]).unwrap_or(-1);
    let w = vars.get_int_any(&["w", "j", "capacity"]).unwrap_or(-1);

    if line_lower.starts_with("for ") || line_lower.starts_with("while ") {
        if line_lower.contains("i") && line_lower.contains("n") {
            return Some(build_step(algorithm, "outer_loop", &format!("遍历物品 i={}", i)));
        }
        if line_lower.contains("w") || line_lower.contains("j") {
            return Some(build_step(algorithm, "inner_loop", &format!("遍历容量 w={}", w)));
        }
    }

    if line_lower.contains("dp[") && line_lower.contains('=') {
        return Some(build_step(algorithm, "transition", "状态转移：计算当前子问题的最优解"));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "动态规划计算完成"));
    }

    None
}

// ============================================================================
// 二分查找
// ============================================================================

fn infer_binary_search(
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
        let l = if left >= 0 {
            left.to_string()
        } else {
            "?".to_string()
        };
        let r = if right >= 0 {
            right.to_string()
        } else {
            "?".to_string()
        };
        return Some(build_step(
            algorithm,
            "loop",
            &format!("搜索范围 [{}, {}]", l, r),
        ));
    }

    if line_lower.contains("mid") && (line_lower.contains('=') || line_lower.contains('/')) {
        let m = if mid >= 0 {
            mid.to_string()
        } else {
            "?".to_string()
        };
        return Some(build_step(
            algorithm,
            "mid_calc",
            &format!("计算中点 mid={}", m),
        ));
    }

    if line_lower.starts_with("if ")
        && is_comparison_line(&line_lower)
        && (line_lower.contains("arr[") || line_lower.contains("a["))
    {
        let m = if mid >= 0 {
            mid.to_string()
        } else {
            "?".to_string()
        };
        return Some(build_step(
            algorithm,
            "compare",
            &format!("arr[{}] 与目标值 {} 比较", m, target),
        ));
    }

    if line_lower.contains("right") && line_lower.contains('=') && line_lower.contains("mid") {
        let m = if mid >= 0 {
            mid.to_string()
        } else {
            "?".to_string()
        };
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
        let m = if mid >= 0 {
            mid.to_string()
        } else {
            "?".to_string()
        };
        return Some(build_step(
            algorithm,
            "found",
            &format!("找到目标值，返回索引 {}", m),
        ));
    }

    if line_lower.starts_with("return ") && (line_lower.contains("-1") || line_lower.contains("0"))
    {
        return Some(build_step(
            algorithm,
            "not_found",
            "搜索结束，未找到目标值",
        ));
    }

    None
}

// ============================================================================
// 希尔排序
// ============================================================================

fn infer_shell_sort(
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

fn infer_counting_sort(
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

fn infer_linked_list_delete(
    source_line: &str,
    _vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();

    if line_lower.contains("head") && line_lower.contains("next") && line_lower.contains("free") {
        return Some(build_step(algorithm, "delete_head", "删除头节点并释放内存"));
    }

    if line_lower.starts_with("while ") && line_lower.contains("data") && line_lower.contains("key") {
        return Some(build_step(algorithm, "search", "遍历链表查找目标节点"));
    }

    if line_lower.contains("prev") && line_lower.contains("next") && line_lower.contains('=') {
        return Some(build_step(algorithm, "unlink", "调整前驱指针，跳过待删除节点"));
    }

    if line_lower.contains("free") && line_lower.contains("temp") {
        return Some(build_step(algorithm, "free", "释放被删除节点的堆内存"));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "链表删除完成"));
    }

    None
}

// ============================================================================
// BST 插入
// ============================================================================

fn infer_bst_insert(
    source_line: &str,
    _vars: &VarMap,
    algorithm: &AlgorithmMatch,
    func_name: &str,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();

    if source_line.contains(&format!("{}(", func_name)) {
        return Some(build_step(algorithm, "recursive", "递归查找插入位置"));
    }

    if line_lower.contains("null") && line_lower.contains("return") {
        return Some(build_step(algorithm, "create", "找到空位，创建新节点"));
    }

    if line_lower.contains("val") && line_lower.contains("root->val") && line_lower.contains('<') {
        return Some(build_step(algorithm, "compare", "比较插入值与当前节点值，决定向左或向右"));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "BST 插入完成"));
    }

    None
}

// ============================================================================
// 字符串反转
// ============================================================================

fn infer_string_reverse(
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
// 最大公约数
// ============================================================================

fn infer_gcd(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let a = vars.get_int_any(&["a"]).unwrap_or(-1);
    let b = vars.get_int_any(&["b"]).unwrap_or(-1);

    if line_lower.starts_with("while ") {
        return Some(build_step(algorithm, "loop", &format!("辗转相除：a={}, b={}", a, b)));
    }

    if line_lower.contains("a % b") || line_lower.contains("mod") {
        return Some(build_step(algorithm, "mod", &format!("计算 {} % {} = {}", a, b, if b != 0 { a % b } else { 0 })));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", &format!("最大公约数为 {}", a)));
    }

    None
}

// ============================================================================
// 素数判断
// ============================================================================

fn infer_is_prime(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let n = vars.get_int_any(&["n"]).unwrap_or(-1);
    let i = vars.get_int("i").unwrap_or(-1);

    if line_lower.starts_with("if ") && line_lower.contains("<=") && line_lower.contains("1") {
        return Some(build_step(algorithm, "check_small", "排除小于等于 1 的数"));
    }

    if line_lower.starts_with("for ") {
        return Some(build_step(algorithm, "test_divisor", &format!("试除 i={}，检查 {} % {} == 0", i, n, i)));
    }

    if line_lower.contains("n % i") && line_lower.contains("== 0") {
        return Some(build_step(algorithm, "found_factor", &format!("发现因子 {}，{} 不是素数", i, n)));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", &format!("{} 是素数", n)));
    }

    None
}

// ============================================================================
// 汉诺塔
// ============================================================================

fn infer_hanoi(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
    func_name: &str,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let n = vars.get_int_any(&["n"]).unwrap_or(-1);

    if line_lower.contains("n == 1") || line_lower.contains("n==1") {
        return Some(build_step(algorithm, "base", "基准情况：直接把盘子从起始柱移到目标柱"));
    }

    if source_line.contains(&format!("{}(", func_name)) {
        return Some(build_step(algorithm, "recursive", &format!("递归移动 {} 个盘子", n - 1)));
    }

    if line_lower.contains("move") && line_lower.contains("disk") {
        return Some(build_step(algorithm, "move", &format!("移动第 {} 个盘子", n)));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "汉诺塔移动完成"));
    }

    None
}

// ============================================================================
// 顺序表
// ============================================================================

fn infer_seq_list(
    source_line: &str,
    _vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();

    if line_lower.contains("length") && line_lower.contains("maxsize") && line_lower.contains("||") {
        return Some(build_step(algorithm, "check", "检查插入/删除位置的合法性"));
    }

    if line_lower.contains("data[i]") && line_lower.contains("data[i - 1]") && line_lower.contains('=') {
        return Some(build_step(algorithm, "shift", "移动元素，腾出或填补位置"));
    }

    if line_lower.contains("data[pos]") && line_lower.contains('=') && !line_lower.contains("||") {
        return Some(build_step(algorithm, "place", "在目标位置放入元素"));
    }

    if line_lower.contains("length") && line_lower.contains("++") {
        return Some(build_step(algorithm, "update_len", "更新表长度"));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "顺序表操作完成"));
    }

    None
}

// ============================================================================
// 链表尾插
// ============================================================================

fn infer_linked_list_append(
    source_line: &str,
    _vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();

    if line_lower.contains("next") && line_lower.contains("null") && line_lower.contains("while") {
        return Some(build_step(algorithm, "find_tail", "遍历链表寻找尾节点"));
    }

    if line_lower.contains("next") && line_lower.contains('=') && !line_lower.contains("null") {
        return Some(build_step(algorithm, "link", "将新节点链接到链表尾部"));
    }

    None
}

// ============================================================================
// 循环队列
// ============================================================================

fn infer_circular_queue(
    source_line: &str,
    _vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();

    if line_lower.contains("rear") && line_lower.contains("front") && line_lower.contains('%') {
        return Some(build_step(algorithm, "check", "利用取模判断队列空或满"));
    }

    if line_lower.contains("data") && line_lower.contains("rear") && line_lower.contains('=') {
        return Some(build_step(algorithm, "enqueue", "元素入队"));
    }

    if line_lower.contains("data") && line_lower.contains("front") && !line_lower.contains("==") {
        return Some(build_step(algorithm, "dequeue", "元素出队"));
    }

    if line_lower.contains("rear") && line_lower.contains('%') {
        return Some(build_step(algorithm, "wrap", "指针循环绕回数组开头"));
    }

    None
}

// ============================================================================
// 链栈
// ============================================================================

fn infer_linked_stack(
    source_line: &str,
    _vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();

    if line_lower.contains("next") && line_lower.contains("top") && line_lower.contains('=') && line_lower.contains("malloc") {
        return Some(build_step(algorithm, "push", "新节点入栈"));
    }

    if line_lower.contains("free") && line_lower.contains("temp") {
        return Some(build_step(algorithm, "pop", "释放栈顶节点"));
    }

    None
}

// ============================================================================
// 链队列
// ============================================================================

fn infer_linked_queue(
    source_line: &str,
    _vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();

    if line_lower.contains("rear") && line_lower.contains("next") && line_lower.contains('=') {
        return Some(build_step(algorithm, "enqueue", "新节点入队并更新尾指针"));
    }

    if line_lower.contains("front") && line_lower.contains("next") && !line_lower.contains("rear") {
        return Some(build_step(algorithm, "dequeue", "队头出队"));
    }

    if line_lower.contains("free") {
        return Some(build_step(algorithm, "free", "释放被删除节点"));
    }

    None
}

// ============================================================================
// 层序遍历
// ============================================================================

fn infer_level_order(
    source_line: &str,
    _vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();

    if line_lower.contains("queue") && line_lower.contains("[") && line_lower.contains("root") {
        return Some(build_step(algorithm, "enqueue", "根节点入队"));
    }

    if line_lower.contains("node") && line_lower.contains("queue") && line_lower.contains("front") {
        return Some(build_step(algorithm, "dequeue", "取出队头节点访问"));
    }

    if line_lower.contains("left") && line_lower.contains("queue") && line_lower.contains("[") {
        return Some(build_step(algorithm, "enqueue_left", "左子节点入队"));
    }

    if line_lower.contains("right") && line_lower.contains("queue") && line_lower.contains("[") {
        return Some(build_step(algorithm, "enqueue_right", "右子节点入队"));
    }

    None
}

// ============================================================================
// BST 查找
// ============================================================================

fn infer_bst_search(
    source_line: &str,
    _vars: &VarMap,
    algorithm: &AlgorithmMatch,
    func_name: &str,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();

    if source_line.contains(&format!("{}(", func_name)) {
        return Some(build_step(algorithm, "recursive", "递归进入子树查找"));
    }

    if line_lower.contains("val") && line_lower.contains("key") && line_lower.contains("==") {
        return Some(build_step(algorithm, "hit", "找到目标节点"));
    }

    if line_lower.contains("null") && line_lower.contains("return") {
        return Some(build_step(algorithm, "miss", "到达空节点，查找失败"));
    }

    if line_lower.contains("key") && line_lower.contains("val") && line_lower.contains('<') {
        return Some(build_step(algorithm, "compare", "比较关键字与当前节点值"));
    }

    None
}

// ============================================================================
// 哈希表
// ============================================================================

fn infer_hash_table(
    source_line: &str,
    _vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();

    if line_lower.contains("key") && line_lower.contains('%') {
        return Some(build_step(algorithm, "hash", "计算哈希值"));
    }

    if line_lower.contains("occupied") && line_lower.contains("while") {
        return Some(build_step(algorithm, "probe", "线性探测寻找空位或目标"));
    }

    if line_lower.contains("key") && line_lower.contains("==") && line_lower.contains("return") {
        return Some(build_step(algorithm, "hit", "找到目标关键字"));
    }

    None
}

// ============================================================================
// 约瑟夫环
// ============================================================================

fn infer_josephus(
    source_line: &str,
    vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();
    let m = vars.get_int_any(&["m"]).unwrap_or(-1);
    let remain = vars.get_int_any(&["remain"]).unwrap_or(-1);

    if line_lower.contains("alive") && line_lower.contains("1") && line_lower.contains('=') {
        return Some(build_step(algorithm, "init", "初始化所有人为存活状态"));
    }

    if line_lower.contains("alive") && line_lower.contains("0") && line_lower.contains('=') {
        return Some(build_step(algorithm, "eliminate", &format!("报到 {} 的人被淘汰，剩余 {} 人", m, remain)));
    }

    if line_lower.contains('%') && line_lower.contains("+ 1") {
        return Some(build_step(algorithm, "rotate", "下标循环绕回，模拟圆圈"));
    }

    if line_lower.starts_with("return") {
        return Some(build_step(algorithm, "finish", "约瑟夫环淘汰完成"));
    }

    None
}
