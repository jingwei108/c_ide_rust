//! `semantic_label` 受控词汇表 —— **词汇即契约**（schema 附录 B，词汇只增不改）。
//!
//! 背景（下游需求清单 B2-3 / SharpTutor S4 §6、S5 §3）：
//! `semantic_label` 长期是"自由文本启发"——`collector.rs::infer_semantic_label` 直接
//! 拼字符串，消费方只能对 label 做模式匹配，词汇一旦漂移（`循环边界` → `循环`）
//! 下游知识卡片静默失效。本模块把词汇升级为**受控词汇表**：
//!
//! - 词汇表在此**单源定义**（Rust 常量），schema 附录 B 与出口 `semantic_labels`
//!   均从它派生，禁止三处各写一份；
//! - `classify()` 是"产出的 label → 词汇表条目"的映射，冻结测试
//!   （`step_payload_schema_v0_1_test::test_semantic_label_vocabulary_closed`）断言
//!   引擎实际产出的每个 label 都能归类——新增标签而不登记词汇表 = 测试失败；
//! - **只增不改**：既有 `id` 与 `template` 不得改语义；扩充只能追加条目。
//!   这与 CSHARP_EXTENSION_PLAN §6-B 的"消费端 UI 直读不做推断"是同一份契约。
//!
//! 状态位 `status`：
//! - `active`：当前引擎会产出（C 域，`collector.rs`）；
//! - `reserved`：随 v0.1 冻结预先登记的词汇（异常域，CS3b 激活时开始产出）——
//!   登记在案 = 下游可提前把 UI 分支写好，激活不是"新增契约"而是"契约兑现"。

/// 词汇表条目：一个 `semantic_label` 的受控形态。
#[derive(Debug, Clone, serde::Serialize)]
pub struct SemanticLabelKind {
    /// 稳定标识（消费方按此分支，**不解析 `template` 文本**）
    pub id: &'static str,
    /// 语言域：`c` / `csharp`（同一词汇表按域登记，字段下发按语言域）
    pub domain: &'static str,
    /// 人类可读模板（`{...}` 为值槽）
    pub template: &'static str,
    /// 实际产出样例（给文档与联调对照）
    pub example: &'static str,
    /// `active`（已产出）/ `reserved`（已登记，激活前不产出）
    pub status: &'static str,
    /// 激活批次（`-` = 已激活）
    pub since: &'static str,
}

/// `semantic_label` 受控词汇表（schema 附录 B 的机器可读单源）。
///
/// 顺序即 `classify()` 的判定顺序（前缀更专的条目在前）。
pub const SEMANTIC_LABEL_VOCABULARY: &[SemanticLabelKind] = &[
    SemanticLabelKind {
        id: "swap",
        domain: "c",
        template: "交换 arr[{i}]↔arr[{j}]",
        example: "交换 arr[3]↔arr[4]",
        status: "active",
        since: "-",
    },
    SemanticLabelKind {
        id: "recursive_call",
        domain: "c",
        template: "递归调用 {func}",
        example: "递归调用 fib",
        status: "active",
        since: "-",
    },
    SemanticLabelKind {
        id: "call",
        domain: "c",
        template: "调用 {func}",
        example: "调用 printf / 调用 qsort / 调用 swap",
        status: "active",
        since: "-",
    },
    SemanticLabelKind {
        id: "generic_call",
        domain: "c",
        template: "函数调用",
        example: "函数调用",
        status: "active",
        since: "-",
    },
    SemanticLabelKind {
        id: "loop",
        domain: "c",
        template: "循环 {iter=v, …}",
        example: "循环 i=0, j=1（无值形态为裸 `循环`）",
        status: "active",
        since: "-",
    },
    SemanticLabelKind {
        id: "return",
        domain: "c",
        template: "返回",
        example: "返回",
        status: "active",
        since: "-",
    },
    SemanticLabelKind {
        id: "heap_alloc",
        domain: "c",
        template: "内存分配",
        example: "内存分配",
        status: "active",
        since: "-",
    },
    SemanticLabelKind {
        id: "heap_free",
        domain: "c",
        template: "释放内存",
        example: "释放内存",
        status: "active",
        since: "-",
    },
    SemanticLabelKind {
        id: "io",
        domain: "c",
        template: "IO 操作",
        example: "IO 操作",
        status: "active",
        since: "-",
    },
    SemanticLabelKind {
        id: "line_fallback",
        domain: "c",
        template: "第 {line} 行",
        example: "第 12 行",
        status: "active",
        since: "-",
    },
    // ── 异常域（SharpTutor S4 §6 首批；CS3b 激活）──────────────────────────
    SemanticLabelKind {
        id: "throw",
        domain: "csharp",
        template: "抛出异常",
        example: "抛出异常",
        status: "reserved",
        since: "CS3b",
    },
    SemanticLabelKind {
        id: "unwind",
        domain: "csharp",
        template: "栈展开",
        example: "栈展开",
        status: "reserved",
        since: "CS3b",
    },
    SemanticLabelKind {
        id: "catch_enter",
        domain: "csharp",
        template: "进入 catch",
        example: "进入 catch",
        status: "reserved",
        since: "CS3b",
    },
    SemanticLabelKind {
        id: "finally",
        domain: "csharp",
        template: "finally 执行",
        example: "finally 执行",
        status: "reserved",
        since: "CS3b",
    },
];

/// 产出的 `semantic_label` → 词汇表条目 `id`。
///
/// `None` 有两种语义，消费方与冻结测试都需区分：
/// - **空串**（`code_line <= 0`，无源码行）不是词汇缺失，是"本步无标签"——
///   调用方应先判空；
/// - 非空却归类失败 = **词汇漂移**（引擎产出了未登记形态），属防线失败。
pub fn classify(label: &str) -> Option<&'static str> {
    if label.is_empty() {
        return None;
    }
    if label.starts_with("交换 arr[") && label.contains('↔') {
        return Some("swap");
    }
    if label.starts_with("递归调用 ") {
        return Some("recursive_call");
    }
    if label == "函数调用" {
        return Some("generic_call");
    }
    if let Some(rest) = label.strip_prefix("调用 ") {
        if !rest.is_empty() {
            return Some("call");
        }
    }
    if label == "循环" || label.starts_with("循环 ") {
        return Some("loop");
    }
    if label == "返回" {
        return Some("return");
    }
    if label == "内存分配" {
        return Some("heap_alloc");
    }
    if label == "释放内存" {
        return Some("heap_free");
    }
    if label == "IO 操作" {
        return Some("io");
    }
    if label.starts_with("第 ") && label.ends_with(" 行") {
        return Some("line_fallback");
    }
    // ── 异常域（预留）──────────────────────────────────────────────────────
    if label == "抛出异常" {
        return Some("throw");
    }
    if label == "栈展开" {
        return Some("unwind");
    }
    if label == "进入 catch" {
        return Some("catch_enter");
    }
    if label == "finally 执行" {
        return Some("finally");
    }
    None
}

/// 按 `id` 取条目（`classify` 的逆查，供出口与文档校验）。
pub fn kind_by_id(id: &str) -> Option<&'static SemanticLabelKind> {
    SEMANTIC_LABEL_VOCABULARY.iter().find(|k| k.id == id)
}

/// 词汇表 JSON（serve `semantic_labels` / 文档自检共用）。
pub fn vocabulary_json() -> serde_json::Value {
    serde_json::json!({
        "schema": crate::unified::contracts::SCHEMA_VERSION,
        "discipline": "词汇只增不改：id 与 template 一经发布不得改语义，扩充只能追加条目",
        "labels": SEMANTIC_LABEL_VOCABULARY,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_covers_every_active_kind() {
        for k in SEMANTIC_LABEL_VOCABULARY {
            // example 取首个逗号/斜杠前的片段，保证是单一样本
            let sample = k.example.split(&['/', '（'][..]).next().unwrap_or("").trim();
            assert_eq!(
                classify(sample),
                Some(k.id),
                "词汇表条目 {} 的 example `{}` 无法归类——example 与 classify 口径不一致",
                k.id,
                sample
            );
        }
    }

    #[test]
    fn classify_rejects_unknown_and_empty() {
        assert_eq!(classify(""), None, "空串 = 无标签，不是词汇条目");
        assert_eq!(classify("循环边界"), None, "未登记形态必须归类失败（防线信号）");
        assert_eq!(classify("调用 "), None, "空函数名的调用是退化形态，不予放行");
    }

    #[test]
    fn ids_are_unique_and_templates_frozen() {
        let mut ids: Vec<&str> = SEMANTIC_LABEL_VOCABULARY.iter().map(|k| k.id).collect();
        let n = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), n, "词汇表 id 重复");
    }
}
