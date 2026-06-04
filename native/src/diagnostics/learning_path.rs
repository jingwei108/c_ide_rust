//! Learning-path recommendation engine (P1).
//!
//! Given a list of detected misconceptions, assembles minimal effective
//! learning paths from the existing knowledge-card and template libraries.

use flutter_rust_bridge::frb;

use crate::diagnostics::misconception_patterns::DetectedMisconception;

/// A single step in a recommended learning path.
#[frb]
#[derive(Debug, Clone)]
pub struct PathStep {
    /// Step type identifier: "ReadKnowledgeCard" | "StudyTemplate" | "CompleteExercise" | "ReviewOwnCode"
    pub step_type: String,
    /// Human-readable title shown in the UI.
    pub title: String,
    /// Detailed description of what the student should do.
    pub detail: String,
    /// Target ID (card_id / template_id / exercise_id).
    pub target_id: String,
    /// Lines to highlight when studying a template (1-based).
    pub highlight_lines: Vec<i32>,
}

/// A complete learning path for a single misconception.
#[frb]
#[derive(Debug, Clone)]
pub struct LearningPath {
    pub target_misconception_id: String,
    pub target_misconception_name: String,
    /// Estimated time to complete, in minutes.
    pub estimated_time_minutes: i32,
    pub steps: Vec<PathStep>,
}

/// Recommend learning paths for the given detected misconceptions.
#[frb]
pub fn recommend_learning_paths(
    detected: Vec<DetectedMisconception>,
) -> Vec<LearningPath> {
    detected
        .into_iter()
        .filter_map(|d| build_path(&d))
        .collect()
}

fn build_path(d: &DetectedMisconception) -> Option<LearningPath> {
    match d.pattern_id.as_str() {
        "M01" => Some(LearningPath {
            target_misconception_id: d.pattern_id.clone(),
            target_misconception_name: d.pattern_name.clone(),
            estimated_time_minutes: 10,
            steps: vec![
                PathStep {
                    step_type: String::from("ReadKnowledgeCard"),
                    title: String::from("阅读：数组越界访问"),
                    detail: String::from("了解为什么数组索引从 0 开始，以及大小为 N 的数组最后一个有效索引是 N-1。"),
                    target_id: String::from("array_out_of_bounds"),
                    highlight_lines: vec![],
                },
                PathStep {
                    step_type: String::from("StudyTemplate"),
                    title: String::from("学习模板：冒泡排序"),
                    detail: String::from("重点观察循环条件 `i < n - 1`，思考为什么用 < 而不是 <=。"),
                    target_id: String::from("bubble"),
                    highlight_lines: vec![4], // approximate loop line
                },
                PathStep {
                    step_type: String::from("CompleteExercise"),
                    title: String::from("练习：修复 3 个越界循环"),
                    detail: String::from("给定的代码中有 3 处数组越界，请找出并修正循环条件。"),
                    target_id: String::from("EX_BOUNDARY_FIX"),
                    highlight_lines: vec![],
                },
            ],
        }),
        "M02" => Some(LearningPath {
            target_misconception_id: d.pattern_id.clone(),
            target_misconception_name: d.pattern_name.clone(),
            estimated_time_minutes: 12,
            steps: vec![
                PathStep {
                    step_type: String::from("ReadKnowledgeCard"),
                    title: String::from("阅读：访问已释放内存"),
                    detail: String::from("理解 free(p) 后为什么必须将 p 置为 NULL，以及悬空指针的危害。"),
                    target_id: String::from("use_after_free"),
                    highlight_lines: vec![],
                },
                PathStep {
                    step_type: String::from("ReadKnowledgeCard"),
                    title: String::from("阅读：NULL 指针解引用"),
                    detail: String::from("了解 NULL 指针的含义，以及使用前为什么必须检查。"),
                    target_id: String::from("null_pointer"),
                    highlight_lines: vec![],
                },
                PathStep {
                    step_type: String::from("StudyTemplate"),
                    title: String::from("学习模板：链表头插法"),
                    detail: String::from("观察 malloc 创建节点和 free 释放节点的完整生命周期。"),
                    target_id: String::from("linkedInsert"),
                    highlight_lines: vec![],
                },
            ],
        }),
        "M03" => Some(LearningPath {
            target_misconception_id: d.pattern_id.clone(),
            target_misconception_name: d.pattern_name.clone(),
            estimated_time_minutes: 8,
            steps: vec![
                PathStep {
                    step_type: String::from("ReadKnowledgeCard"),
                    title: String::from("阅读：条件内使用 = 而非 =="),
                    detail: String::from("= 是赋值，== 是比较。在 if/while 中误用会导致条件永远为真并修改变量。"),
                    target_id: String::from("assignment_in_condition"),
                    highlight_lines: vec![],
                },
                PathStep {
                    step_type: String::from("StudyTemplate"),
                    title: String::from("学习模板：二分查找"),
                    detail: String::from("注意所有比较操作都使用 ==，观察条件判断的正确写法。"),
                    target_id: String::from("binary"),
                    highlight_lines: vec![],
                },
            ],
        }),
        "M04" => Some(LearningPath {
            target_misconception_id: d.pattern_id.clone(),
            target_misconception_name: d.pattern_name.clone(),
            estimated_time_minutes: 10,
            steps: vec![
                PathStep {
                    step_type: String::from("ReadKnowledgeCard"),
                    title: String::from("阅读：复杂声明的建议"),
                    detail: String::from("数组名在表达式中会退化为指针，sizeof(arr) 在函数参数中结果与预期不同。"),
                    target_id: String::from("complex_declarator"),
                    highlight_lines: vec![],
                },
                PathStep {
                    step_type: String::from("StudyTemplate"),
                    title: String::from("学习模板：指针基础"),
                    detail: String::from("观察数组名如何作为指针使用，以及指针算术的步长计算。"),
                    target_id: String::from("pointer"),
                    highlight_lines: vec![],
                },
            ],
        }),
        "M05" => Some(LearningPath {
            target_misconception_id: d.pattern_id.clone(),
            target_misconception_name: d.pattern_name.clone(),
            estimated_time_minutes: 10,
            steps: vec![
                PathStep {
                    step_type: String::from("ReadKnowledgeCard"),
                    title: String::from("阅读：栈溢出"),
                    detail: String::from("递归必须有终止条件，否则会导致无限递归和栈溢出。"),
                    target_id: String::from("stack_overflow"),
                    highlight_lines: vec![],
                },
                PathStep {
                    step_type: String::from("StudyTemplate"),
                    title: String::from("学习模板：递归阶乘"),
                    detail: String::from("重点观察 `if (n <= 1) return 1;` 这一行，它是递归的终止条件。"),
                    target_id: String::from("factorial"),
                    highlight_lines: vec![3],
                },
            ],
        }),
        "M06" => Some(LearningPath {
            target_misconception_id: d.pattern_id.clone(),
            target_misconception_name: d.pattern_name.clone(),
            estimated_time_minutes: 8,
            steps: vec![
                PathStep {
                    step_type: String::from("ReadKnowledgeCard"),
                    title: String::from("阅读：scanf 忘记取地址"),
                    detail: String::from("scanf 需要传入变量的地址，但 printf 不需要。注意两者的区别。"),
                    target_id: String::from("scanf_address"),
                    highlight_lines: vec![],
                },
                PathStep {
                    step_type: String::from("StudyTemplate"),
                    title: String::from("学习模板：数组基础"),
                    detail: String::from("观察 printf 中 %d 与数组元素的对应关系。"),
                    target_id: String::from("array"),
                    highlight_lines: vec![],
                },
            ],
        }),
        _ => None,
    }
}

// ===================================================================
// Unit tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::misconception_patterns::DetectedMisconception;

    #[test]
    fn test_recommend_m01() {
        let detected = vec![DetectedMisconception {
            pattern_id: String::from("M01"),
            pattern_name: String::from("边界混淆"),
            description: String::new(),
            occurrence_count: 3,
            confidence: 0.75,
        }];
        let paths = recommend_learning_paths(detected);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].target_misconception_id, "M01");
        assert_eq!(paths[0].steps.len(), 3);
        assert_eq!(paths[0].steps[0].step_type, "ReadKnowledgeCard");
        assert_eq!(paths[0].steps[1].step_type, "StudyTemplate");
    }

    #[test]
    fn test_recommend_empty() {
        let paths = recommend_learning_paths(vec![]);
        assert!(paths.is_empty());
    }
}
