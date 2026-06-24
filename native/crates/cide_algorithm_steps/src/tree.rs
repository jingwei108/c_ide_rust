use crate::*;

// ============================================================================
// BST 插入
// ============================================================================

pub(crate) fn infer_bst_insert(
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
// 层序遍历
// ============================================================================

pub(crate) fn infer_level_order(
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

pub(crate) fn infer_bst_search(
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
// 线索二叉树
// ============================================================================

pub(crate) fn infer_threaded_binary_tree(
    source_line: &str,
    _vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();

    if line_lower.contains("ltag") && line_lower.contains("thread") {
        return Some(build_step(algorithm, "thread_left", "建立左线索指向前驱"));
    }

    if line_lower.contains("rtag") && line_lower.contains("thread") {
        return Some(build_step(algorithm, "thread_right", "建立右线索指向后继"));
    }

    if line_lower.contains("ltag") && line_lower.contains("link") {
        return Some(build_step(algorithm, "find_leftmost", "沿左孩子找到最左节点"));
    }

    if line_lower.contains("rtag") && line_lower.contains("thread") && line_lower.contains("while") {
        return Some(build_step(algorithm, "follow_thread", "沿后继线索访问节点"));
    }

    None
}
// ============================================================================
// 哈夫曼树
// ============================================================================

pub(crate) fn infer_huffman_tree(
    source_line: &str,
    _vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();

    if line_lower.contains("parent") && line_lower.contains("-1") && line_lower.contains("weight") {
        return Some(build_step(algorithm, "select", "在森林中选择两个最小权值节点"));
    }

    if line_lower.contains("parent") && line_lower.contains('=') && !line_lower.contains("-1") {
        return Some(build_step(algorithm, "merge", "合并两个节点为新树"));
    }

    None
}
// ============================================================================
// AVL 树
// ============================================================================

pub(crate) fn infer_avl_tree(
    source_line: &str,
    _vars: &VarMap,
    algorithm: &AlgorithmMatch,
) -> Option<AlgorithmStepSnapshot> {
    let line_lower = source_line.to_lowercase();

    if line_lower.contains("r_rotate") || line_lower.contains("l_rotate") {
        return Some(build_step(algorithm, "rotate", "旋转调整平衡"));
    }

    if line_lower.contains("leftbalance") || line_lower.contains("rightbalance") {
        return Some(build_step(algorithm, "balance", "平衡因子失衡，进行平衡调整"));
    }

    if line_lower.contains("bf") && line_lower.contains('=') && !line_lower.contains("null") {
        return Some(build_step(algorithm, "update_bf", "更新平衡因子"));
    }

    None
}
