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
