//! 算法模式检测器
//!
//! 基于 AST 进行启发式算法识别，支持：
//! - 冒泡排序 (bubble_sort)
//! - 选择排序 (selection_sort)
//! - 插入排序 (insertion_sort)
//! - 快速排序 (quick_sort)
//! - 归并排序 (merge_sort)
//! - 二分查找 (binary_search)

use crate::compiler::ast::*;
use crate::compiler::cfg::ControlFlowGraph;
use crate::session::{AlgorithmMatch, VisEvent};

/// 检测程序中的所有算法模式
pub fn detect_algorithms(program: &ProgramNode) -> Vec<AlgorithmMatch> {
    let mut matches = Vec::new();
    for func in &program.funcs {
        matches.extend(detect_in_func(func));
    }
    matches
}

fn detect_in_func(func: &FuncDecl) -> Vec<AlgorithmMatch> {
    let body = match func.body.as_ref() {
        Some(b) => b,
        None => return Vec::new(),
    };
    let mut features = extract_features(body);

    // P3: augment with CFG features
    if let Some(cfg) = ControlFlowGraph::from_func(func) {
        features.cfg_has_back_edge = cfg.edges.iter().any(|(a, b)| *a >= *b);
        features.cfg_num_blocks = cfg.blocks.len();
        features.cfg_has_unreachable = !cfg.find_unreachable_blocks().is_empty();
        features.cfg_has_early_return = cfg
            .blocks
            .iter()
            .filter(|b| matches!(b.terminator, crate::compiler::cfg::Terminator::Return))
            .count()
            > 1;
    }

    // 基于函数名和结构特征进行匹配（收集所有匹配）
    let name_lower = func.name.to_lowercase();
    let mut matches = Vec::new();

    // 冒泡排序
    if name_lower.contains("bubble")
        || (features.has_nested_loops
            && features.has_array_compare
            && features.has_swap
            && features.loop_depth >= 2
            && features.has_adjacent_index_compare)
    {
        matches.push(build_match(
            "bubble_sort",
            "冒泡排序",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
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
            &func.name,
            func.loc.line,
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
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // 快速排序
    if name_lower.contains("quick")
        || (features.is_recursive && features.has_partition_pattern && features.has_nested_loops)
    {
        matches.push(build_match(
            "quick_sort",
            "快速排序",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // 归并排序
    if name_lower.contains("merge")
        || (features.is_recursive && features.has_merge_pattern && !features.has_partition_pattern)
    {
        matches.push(build_match(
            "merge_sort",
            "归并排序",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // 二分查找
    if name_lower.contains("binary")
        || name_lower.contains("bsearch")
        || (features.has_single_loop && features.has_mid_calculation && features.has_left_right_update)
    {
        matches.push(build_match(
            "binary_search",
            "二分查找",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // 堆排序
    if name_lower.contains("heap")
        || (features.is_recursive && features.has_array_compare && features.has_swap && name_lower.contains("sort"))
    {
        matches.push(build_match(
            "heap_sort",
            "堆排序",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // BFS 广度优先搜索
    if name_lower.contains("bfs")
        || name_lower.contains("breadth")
        || (name_lower.contains("search") && features.has_single_loop && features.cfg_has_back_edge)
    {
        matches.push(build_match(
            "bfs",
            "BFS 广度优先搜索",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // DFS 深度优先搜索
    if name_lower.contains("dfs")
        || name_lower.contains("depth")
        || (features.is_recursive
            && !features.has_partition_pattern
            && !features.has_merge_pattern
            && name_lower.contains("search"))
    {
        matches.push(build_match(
            "dfs",
            "DFS 深度优先搜索",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // DP 动态规划（斐波那契 / 背包 等）
    if name_lower.contains("dp") || name_lower.contains("knapsack") || name_lower.contains("dynamic") {
        matches.push(build_match(
            "dp",
            "动态规划",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // 希尔排序
    if name_lower.contains("shell") {
        matches.push(build_match(
            "shell_sort",
            "希尔排序",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // 计数排序
    if name_lower.contains("counting") {
        matches.push(build_match(
            "counting_sort",
            "计数排序",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // 链表操作（删除/插入等）
    if name_lower.contains("deletenode") || name_lower.contains("delete_node") {
        matches.push(build_match(
            "linked_list_delete",
            "链表删除",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // 二叉搜索树
    if name_lower.contains("bst")
        || name_lower.contains("insert") && features.is_recursive && features.cfg_has_back_edge
    {
        matches.push(build_match(
            "bst_insert",
            "BST 插入",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // 字符串操作
    if name_lower.contains("reverse") && name_lower.contains("str") {
        matches.push(build_match(
            "string_reverse",
            "字符串反转",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // 数学算法
    if name_lower.contains("gcd") || name_lower.contains("greatest") {
        matches.push(build_match(
            "gcd",
            "最大公约数",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    if name_lower.contains("prime") || name_lower.contains("isprime") {
        matches.push(build_match(
            "is_prime",
            "素数判断",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // 汉诺塔
    if name_lower.contains("hanoi") {
        matches.push(build_match(
            "hanoi",
            "汉诺塔",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // 顺序表 / 数组操作
    if name_lower.contains("seqlist") || name_lower.contains("list_insert") || name_lower.contains("listdelete") {
        matches.push(build_match(
            "seq_list",
            "顺序表",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // 链表尾插 / 双向链表
    if name_lower.contains("append") && (name_lower.contains("list") || name_lower.contains("node")) {
        matches.push(build_match(
            "linked_list_append",
            "链表尾插",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // 循环队列
    if name_lower.contains("circular") && name_lower.contains("queue") {
        matches.push(build_match(
            "circular_queue",
            "循环队列",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // 链栈
    if name_lower.contains("linked") && name_lower.contains("stack") {
        matches.push(build_match(
            "linked_stack",
            "链栈",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // 链队列
    if name_lower.contains("linked") && name_lower.contains("queue") {
        matches.push(build_match(
            "linked_queue",
            "链队列",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // 层序遍历
    if name_lower.contains("levelorder") || name_lower.contains("level_order") {
        matches.push(build_match(
            "level_order",
            "层序遍历",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // BST 查找
    if name_lower.contains("search") && features.is_recursive && features.cfg_has_back_edge {
        matches.push(build_match(
            "bst_search",
            "BST 查找",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // 哈希表
    if name_lower.contains("hash") && !name_lower.contains("cash") {
        matches.push(build_match(
            "hash_table",
            "哈希表",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // 约瑟夫环
    if name_lower.contains("josephus") {
        matches.push(build_match(
            "josephus",
            "约瑟夫环",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // P0: 循环链表
    if name_lower.contains("circular") && name_lower.contains("list") {
        matches.push(build_match(
            "circular_linked_list",
            "循环链表",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // P0: 静态链表
    if name_lower.contains("static") && name_lower.contains("list") {
        matches.push(build_match(
            "static_linked_list",
            "静态链表",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // P0: 朴素模式匹配
    if name_lower.contains("indexbf") || name_lower.contains("bfmatch") || name_lower.contains("brute") {
        matches.push(build_match(
            "string_match_bf",
            "朴素模式匹配",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // P0: KMP 模式匹配
    if name_lower.contains("kmp") || name_lower.contains("indexkmp") || name_lower.contains("getnext") {
        matches.push(build_match(
            "string_match_kmp",
            "KMP 模式匹配",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // P0: 线索二叉树
    if name_lower.contains("thread") && name_lower.contains("tree") {
        matches.push(build_match(
            "threaded_binary_tree",
            "线索二叉树",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // P0: 哈夫曼树
    if name_lower.contains("huffman") {
        matches.push(build_match(
            "huffman_tree",
            "哈夫曼树",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // P0: 并查集
    if name_lower.contains("unionfind") || (name_lower.contains("union") && name_lower.contains("find")) {
        matches.push(build_match(
            "union_find",
            "并查集",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // P0: AVL 树
    if name_lower.contains("avl") {
        matches.push(build_match(
            "avl_tree",
            "AVL 树",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // P0: Prim 最小生成树
    if name_lower.contains("prim") {
        matches.push(build_match(
            "prim_mst",
            "Prim 最小生成树",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // P0: Kruskal 最小生成树
    if name_lower.contains("kruskal") {
        matches.push(build_match(
            "kruskal_mst",
            "Kruskal 最小生成树",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // P0: Dijkstra 最短路径
    if name_lower.contains("dijkstra") {
        matches.push(build_match(
            "dijkstra",
            "Dijkstra 最短路径",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // P0: Floyd 最短路径
    if name_lower.contains("floyd") {
        matches.push(build_match(
            "floyd",
            "Floyd 最短路径",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // P0: 拓扑排序
    if name_lower.contains("topolog") {
        matches.push(build_match(
            "topological_sort",
            "拓扑排序",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    // P0: 基数排序
    if name_lower.contains("radix") {
        matches.push(build_match(
            "radix_sort",
            "基数排序",
            &func.name,
            func.loc.line,
            &features.compare_lines,
        ));
    }

    matches
}

// ============================================================================
// 特征提取
// ============================================================================

#[derive(Default)]
struct FuncFeatures {
    loop_depth: i32,
    max_loop_depth: i32,
    has_nested_loops: bool,
    has_single_loop: bool,
    has_array_compare: bool,
    has_swap: bool,
    has_swap_in_inner_loop: bool,
    has_min_max_track: bool,
    has_shift_pattern: bool,
    is_recursive: bool,
    has_partition_pattern: bool,
    has_merge_pattern: bool,
    has_mid_calculation: bool,
    has_left_right_update: bool,
    has_adjacent_index_compare: bool,
    // CFG-derived features (P3)
    cfg_has_early_return: bool,
    cfg_has_back_edge: bool,
    cfg_num_blocks: usize,
    cfg_has_unreachable: bool,
    compare_lines: Vec<(i32, i32, String)>, // (line, type, context)
}

const MAX_WALK_DEPTH: i32 = 512;

fn extract_features(stmt: &Stmt) -> FuncFeatures {
    let mut f = FuncFeatures::default();
    walk_stmt(stmt, &mut f, 0, "", 0);
    f.has_nested_loops = f.max_loop_depth >= 2;
    f.has_single_loop = f.max_loop_depth >= 1;
    f
}

fn walk_stmt(stmt: &Stmt, f: &mut FuncFeatures, loop_depth: i32, func_name: &str, depth: i32) {
    if depth > MAX_WALK_DEPTH {
        return;
    }
    match stmt {
        Stmt::Block { stmts, .. } => {
            for s in stmts {
                walk_stmt(s, f, loop_depth, func_name, depth + 1);
            }
        }
        Stmt::If { cond, then_stmt, else_stmt, .. } => {
            check_compare_expr(cond, f, loop_depth);
            // P3: detect min/max tracking pattern inside loops
            if loop_depth >= 1
                && is_comparison_expr(cond)
                && (stmt_has_min_max_assign(then_stmt)
                    || else_stmt.as_ref().is_some_and(|s| stmt_has_min_max_assign(s)))
            {
                f.has_min_max_track = true;
            }
            walk_stmt(then_stmt, f, loop_depth, func_name, depth + 1);
            if let Some(e) = else_stmt {
                walk_stmt(e, f, loop_depth, func_name, depth + 1);
            }
        }
        Stmt::While { cond, body, .. } => {
            check_compare_expr(cond, f, loop_depth);
            let new_depth = loop_depth + 1;
            f.max_loop_depth = f.max_loop_depth.max(new_depth);
            walk_stmt(body, f, new_depth, func_name, depth + 1);
        }
        Stmt::DoWhile { body, cond, .. } => {
            let new_depth = loop_depth + 1;
            f.max_loop_depth = f.max_loop_depth.max(new_depth);
            walk_stmt(body, f, new_depth, func_name, depth + 1);
            check_compare_expr(cond, f, loop_depth);
        }
        Stmt::For { init, cond, step, body, .. } => {
            let new_depth = loop_depth + 1;
            f.max_loop_depth = f.max_loop_depth.max(new_depth);
            if let Some(ref i) = init {
                walk_stmt(i, f, loop_depth, func_name, depth + 1);
            }
            if let Some(c) = cond {
                check_compare_expr(c, f, loop_depth);
            }
            for s in step {
                walk_expr(s, f, loop_depth, func_name, depth + 1);
            }
            walk_stmt(body, f, new_depth, func_name, depth + 1);
        }
        Stmt::VarDecl { init, extra_vars, .. } => {
            if let Some(e) = init {
                walk_expr(e, f, loop_depth, func_name, depth + 1);
            }
            for (_, _, e2) in extra_vars {
                if let Some(e) = e2 {
                    walk_expr(e, f, loop_depth, func_name, depth + 1);
                }
            }
        }
        Stmt::Expr { expr, .. } => {
            walk_expr(expr, f, loop_depth, func_name, depth + 1);
        }
        Stmt::Return { value: Some(v), .. } => {
            walk_expr(v, f, loop_depth, func_name, depth + 1);
        }
        Stmt::Switch { cond, body, .. } => {
            walk_expr(cond, f, loop_depth, func_name, depth + 1);
            walk_stmt(body, f, loop_depth, func_name, depth + 1);
        }
        Stmt::Case { stmt: s, .. } => {
            walk_stmt(s, f, loop_depth, func_name, depth + 1);
        }
        _ => {}
    }
}

fn walk_expr(expr: &Expr, f: &mut FuncFeatures, loop_depth: i32, func_name: &str, depth: i32) {
    if depth > MAX_WALK_DEPTH {
        return;
    }
    match expr {
        Expr::Binary { op, left, right, .. } => {
            walk_expr(left, f, loop_depth, func_name, depth + 1);
            walk_expr(right, f, loop_depth, func_name, depth + 1);

            // 检测 mid 计算：left + (right - left) / 2 或 (left + right) / 2
            if matches!(op, BinaryOp::Add | BinaryOp::Div) && is_mid_calculation(expr) {
                f.has_mid_calculation = true;
            }
        }
        Expr::Unary { operand, .. } => {
            walk_expr(operand, f, loop_depth, func_name, depth + 1);
        }
        Expr::Call { name, args, .. } => {
            for a in args {
                walk_expr(a, f, loop_depth, func_name, depth + 1);
            }
            // 递归调用
            if name == func_name {
                f.is_recursive = true;
            }
            // partition / merge 模式
            let n = name.to_lowercase();
            if n.contains("partition") {
                f.has_partition_pattern = true;
            }
            if n.contains("merge") {
                f.has_merge_pattern = true;
            }
        }
        Expr::Index { array, index, .. } => {
            walk_expr(array, f, loop_depth, func_name, depth + 1);
            walk_expr(index, f, loop_depth, func_name, depth + 1);
        }
        Expr::Member { object, .. } => {
            walk_expr(object, f, loop_depth, func_name, depth + 1);
        }
        Expr::Assign { op, left, right, .. } => {
            walk_expr(left, f, loop_depth, func_name, depth + 1);
            walk_expr(right, f, loop_depth, func_name, depth + 1);

            // 检测交换模式
            if is_index_access(left) && is_index_access(right) {
                if loop_depth >= 2 {
                    f.has_swap_in_inner_loop = true;
                }
                f.has_swap = true;
            }

            // 检测 left/right 更新（基于 AST 结构：涉及索引访问的赋值）
            if matches!(op, AssignOp::Assign) {
                if let Expr::Index { index: idx, .. } = left.as_ref() {
                    if expr_to_string(idx).contains("mid") {
                        f.has_left_right_update = true;
                    }
                }
            }

            // 检测 shift 模式：arr[j+1] = arr[j] 或类似的后移操作
            if matches!(op, AssignOp::Assign) && is_shift_pattern(left, right) {
                f.has_shift_pattern = true;
            }
        }
        Expr::Ternary {
            cond, then_branch, else_branch, ..
        } => {
            walk_expr(cond, f, loop_depth, func_name, depth + 1);
            walk_expr(then_branch, f, loop_depth, func_name, depth + 1);
            walk_expr(else_branch, f, loop_depth, func_name, depth + 1);
        }
        Expr::Cast { expr: e, .. } => {
            walk_expr(e, f, loop_depth, func_name, depth + 1);
        }
        Expr::Sizeof { operand: Some(e), .. } => {
            walk_expr(e, f, loop_depth, func_name, depth + 1);
        }
        Expr::InitList { elements, .. } => {
            for e in elements {
                walk_expr(e, f, loop_depth, func_name, depth + 1);
            }
        }
        _ => {}
    }
}

fn check_compare_expr(expr: &Expr, f: &mut FuncFeatures, _loop_depth: i32) {
    match expr {
        Expr::Binary { op, left, right, loc, .. } => {
            if is_comparison_op(op) {
                if is_index_access(left) || is_index_access(right) {
                    f.has_array_compare = true;
                    let ctx = format_compare_context(left, right);
                    f.compare_lines.push((loc.line, 1, ctx));

                    // 检测相邻索引比较：arr[i] vs arr[i+1] 或 arr[j] vs arr[j+1]
                    if is_adjacent_compare(left, right) || is_adjacent_compare(right, left) {
                        f.has_adjacent_index_compare = true;
                    }
                } else {
                    let ctx = format_compare_context(left, right);
                    f.compare_lines.push((loc.line, 1, ctx));
                }
            }
            check_compare_expr(left, f, _loop_depth);
            check_compare_expr(right, f, _loop_depth);
        }
        Expr::Unary { operand, .. } => {
            check_compare_expr(operand, f, _loop_depth);
        }
        Expr::Call { args, .. } => {
            for a in args {
                check_compare_expr(a, f, _loop_depth);
            }
        }
        Expr::Index { array, index, .. } => {
            check_compare_expr(array, f, _loop_depth);
            check_compare_expr(index, f, _loop_depth);
        }
        Expr::Member { object, .. } => {
            check_compare_expr(object, f, _loop_depth);
        }
        Expr::Ternary {
            cond, then_branch, else_branch, ..
        } => {
            check_compare_expr(cond, f, _loop_depth);
            check_compare_expr(then_branch, f, _loop_depth);
            check_compare_expr(else_branch, f, _loop_depth);
        }
        Expr::Cast { expr: e, .. } => {
            check_compare_expr(e, f, _loop_depth);
        }
        Expr::Sizeof { operand: Some(e), .. } => {
            check_compare_expr(e, f, _loop_depth);
        }
        _ => {}
    }
}

fn is_comparison_op(op: &BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge | BinaryOp::Eq | BinaryOp::Ne
    )
}

fn is_comparison_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Binary { op, .. } => is_comparison_op(op),
        Expr::Unary { op: UnaryOp::Not, operand, .. } => is_comparison_expr(operand),
        _ => false,
    }
}

fn stmt_has_min_max_assign(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Block { stmts, .. } => stmts.iter().any(stmt_has_min_max_assign),
        Stmt::Expr { expr, .. } => expr_has_min_max_assign(expr),
        Stmt::VarDecl { init, extra_vars, .. } => {
            init.as_ref().is_some_and(expr_has_min_max_assign)
                || extra_vars
                    .iter()
                    .any(|(_, _, e)| e.as_ref().is_some_and(expr_has_min_max_assign))
        }
        _ => false,
    }
}

fn expr_has_min_max_assign(expr: &Expr) -> bool {
    match expr {
        Expr::Assign { left, .. } => {
            if let Expr::Identifier { name, .. } = left.as_ref() {
                let n = name.to_lowercase();
                return n.contains("min") || n.contains("max");
            }
            false
        }
        Expr::Binary { left, right, .. } => expr_has_min_max_assign(left) || expr_has_min_max_assign(right),
        Expr::Unary { operand, .. } => expr_has_min_max_assign(operand),
        Expr::Ternary {
            cond, then_branch, else_branch, ..
        } => {
            expr_has_min_max_assign(cond)
                || expr_has_min_max_assign(then_branch)
                || expr_has_min_max_assign(else_branch)
        }
        Expr::Call { args, .. } => args.iter().any(expr_has_min_max_assign),
        Expr::CallPtr { callee, args, .. } => {
            expr_has_min_max_assign(callee) || args.iter().any(expr_has_min_max_assign)
        }
        Expr::Index { array, index, .. } => expr_has_min_max_assign(array) || expr_has_min_max_assign(index),
        Expr::Member { object, .. } => expr_has_min_max_assign(object),
        Expr::Cast { expr: e, .. } => expr_has_min_max_assign(e),
        Expr::Sizeof { operand: Some(e), .. } => expr_has_min_max_assign(e),
        Expr::InitList { elements, .. } => elements.iter().any(|e| expr_has_min_max_assign(&e.value)),
        _ => false,
    }
}

fn is_index_access(expr: &Expr) -> bool {
    matches!(expr, Expr::Index { .. })
}

fn is_literal_int(expr: &Expr, val: i32) -> bool {
    matches!(expr, Expr::Literal { value: v, .. } if *v == val)
}

fn is_adjacent_compare(a: &Expr, b: &Expr) -> bool {
    // 检查是否是 arr[x] 和 arr[x+1] 的比较（基于 AST 结构，而非字符串格式）
    if let Expr::Index { array: arr_a, index: idx_a, .. } = a {
        if let Expr::Index { array: arr_b, index: idx_b, .. } = b {
            if expr_to_string(arr_a) == expr_to_string(arr_b) {
                // 检查 idx_b 是否是 idx_a + 1
                if let Expr::Binary {
                    op: BinaryOp::Add, left, right, ..
                } = idx_b.as_ref()
                {
                    if expr_to_string(left) == expr_to_string(idx_a) && is_literal_int(right, 1) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn is_mid_calculation(expr: &Expr) -> bool {
    // 模式1: (a + b) / 2
    if let Expr::Binary {
        op: BinaryOp::Div, left, right, ..
    } = expr
    {
        if is_literal_int(right, 2) {
            if let Expr::Binary { op: BinaryOp::Add, .. } = left.as_ref() {
                return true;
            }
        }
    }
    // 模式2: a + (b - a) / 2
    if let Expr::Binary {
        op: BinaryOp::Add, left, right, ..
    } = expr
    {
        if let Expr::Binary {
            op: BinaryOp::Div,
            left: div_left,
            right: div_right,
            ..
        } = right.as_ref()
        {
            if is_literal_int(div_right, 2) {
                if let Expr::Binary {
                    op: BinaryOp::Sub,
                    left: _,
                    right: sub_right,
                    ..
                } = div_left.as_ref()
                {
                    if expr_to_string(sub_right) == expr_to_string(left) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn is_shift_pattern(left: &Expr, right: &Expr) -> bool {
    // 检测 arr[x] = arr[y] 的后移模式，要求 x == y + 1
    if let Expr::Index { array: arr_l, index: idx_l, .. } = left {
        if let Expr::Index { array: arr_r, index: idx_r, .. } = right {
            if expr_to_string(arr_l) == expr_to_string(arr_r) {
                // 检查 idx_l 是否为 idx_r + 1
                if let Expr::Binary {
                    op: BinaryOp::Add,
                    left: add_left,
                    right: add_right,
                    ..
                } = idx_l.as_ref()
                {
                    if expr_to_string(add_left) == expr_to_string(idx_r) && is_literal_int(add_right, 1) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn format_compare_context(left: &Expr, right: &Expr) -> String {
    format!("{}:{}", expr_to_string(left), expr_to_string(right))
}

fn expr_to_string(expr: &Expr) -> String {
    match expr {
        Expr::Binary { op, left, right, .. } => {
            let op_str = match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
                BinaryOp::Mod => "%",
                BinaryOp::Eq => "==",
                BinaryOp::Ne => "!=",
                BinaryOp::Lt => "<",
                BinaryOp::Le => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::Ge => ">=",
                BinaryOp::And => "&&",
                BinaryOp::Or => "||",
                BinaryOp::BitAnd => "&",
                BinaryOp::BitOr => "|",
                BinaryOp::BitXor => "^",
                BinaryOp::Shl => "<<",
                BinaryOp::Shr => ">>",
                BinaryOp::Comma => ",",
            };
            format!("{} {} {}", expr_to_string(left), op_str, expr_to_string(right))
        }
        Expr::Unary { op, operand, .. } => {
            let op_str = match op {
                UnaryOp::Neg => "-",
                UnaryOp::Not => "!",
                UnaryOp::BitNot => "~",
                UnaryOp::Addr => "&",
                UnaryOp::Deref => "*",
                UnaryOp::PreInc => "++",
                UnaryOp::PreDec => "--",
                UnaryOp::PostInc => "++",
                UnaryOp::PostDec => "--",
            };
            format!("{}{}", op_str, expr_to_string(operand))
        }
        Expr::Literal { value, .. } => value.to_string(),
        Expr::LongLiteral { value, .. } => value.to_string(),
        Expr::FloatLiteral { value, .. } => value.to_string(),
        Expr::StringLiteral { value, .. } => format!("\"{}\"", value),
        Expr::Identifier { name, .. } => name.clone(),
        Expr::Call { name, args, .. } => {
            let args_str: Vec<String> = args.iter().map(expr_to_string).collect();
            format!("{}({})", name, args_str.join(", "))
        }
        Expr::Index { array, index, .. } => {
            format!("{}[{}]", expr_to_string(array), expr_to_string(index))
        }
        Expr::Member { object, member, .. } => {
            format!("{}.{}", expr_to_string(object), member)
        }
        Expr::Assign { op, left, right, .. } => {
            let op_str = match op {
                AssignOp::Assign => "=",
                AssignOp::AddAssign => "+=",
                AssignOp::SubAssign => "-=",
                AssignOp::MulAssign => "*=",
                AssignOp::DivAssign => "/=",
                AssignOp::ModAssign => "%=",
                AssignOp::AndAssign => "&=",
                AssignOp::OrAssign => "|=",
                AssignOp::XorAssign => "^=",
                AssignOp::ShlAssign => "<<=",
                AssignOp::ShrAssign => ">>=",
            };
            format!("{} {} {}", expr_to_string(left), op_str, expr_to_string(right))
        }
        Expr::Ternary {
            cond, then_branch, else_branch, ..
        } => {
            format!(
                "{} ? {} : {}",
                expr_to_string(cond),
                expr_to_string(then_branch),
                expr_to_string(else_branch)
            )
        }
        Expr::Sizeof { target_type, operand, .. } => {
            if let Some(t) = target_type {
                format!("sizeof({})", t)
            } else if let Some(e) = operand {
                format!("sizeof({})", expr_to_string(e))
            } else {
                "sizeof".to_string()
            }
        }
        Expr::Cast { expr: e, target_type, .. } => {
            format!("({}){}", target_type, expr_to_string(e))
        }
        Expr::InitList { elements, .. } => {
            let elems: Vec<String> = elements.iter().map(|e| expr_to_string(&e.value)).collect();
            format!("{{{}}}", elems.join(", "))
        }
        Expr::CallPtr { .. } => "fp(...)".to_string(),
        Expr::Offsetof { target_type, field, .. } => {
            format!("offsetof({}, {})", target_type, field)
        }
        // === C++ 新增 (Phase 31 占位) ===
        _ => "<cpp-expr>".to_string(),
    }
}

fn build_match(
    name: &str,
    display_name: &str,
    func_name: &str,
    line: i32,
    compare_lines: &[(i32, i32, String)],
) -> AlgorithmMatch {
    let suggestion = match name {
        "bubble_sort" => {
            "冒泡排序：通过相邻元素两两比较并交换，将最大元素逐步「冒泡」到数组末尾。时间复杂度 O(n²)。".to_string()
        }
        "selection_sort" => {
            "选择排序：每次从未排序部分选择最小元素，放到已排序部分末尾。时间复杂度 O(n²)。".to_string()
        }
        "insertion_sort" => {
            "插入排序：将元素逐个插入到已排序部分的正确位置。时间复杂度 O(n²)，对近乎有序的数组效率很高。".to_string()
        }
        "quick_sort" => {
            "快速排序：通过分治法，选取枢轴将数组分区，再递归排序子数组。平均时间复杂度 O(n log n)。".to_string()
        }
        "merge_sort" => "归并排序：将数组递归分成两半，排序后合并。时间复杂度稳定为 O(n log n)。".to_string(),
        "binary_search" => "二分查找：在有序数组中每次将搜索范围减半。时间复杂度 O(log n)。".to_string(),
        "heap_sort" => "堆排序：利用堆数据结构进行排序。先建堆再反复取出堆顶元素。时间复杂度 O(n log n)。".to_string(),
        "bfs" => "BFS 广度优先搜索：从起点出发，逐层扩展访问邻居。适合求最短路径。".to_string(),
        "dfs" => "DFS 深度优先搜索：从起点出发，沿着一条路径走到尽头再回溯。适合连通性判断。".to_string(),
        "dp" => "动态规划：将复杂问题分解为子问题，保存子问题答案避免重复计算。".to_string(),
        "shell_sort" => {
            "希尔排序：通过增量分组进行插入排序，逐步缩小增量至 1。时间复杂度介于 O(n log n) 和 O(n²) 之间。"
                .to_string()
        }
        "counting_sort" => {
            "计数排序：用统计数组记录元素出现次数，适合数据范围小的场景。时间复杂度 O(n+k)。".to_string()
        }
        "linked_list_delete" => "链表删除：遍历链表找到目标节点，调整指针并释放内存。".to_string(),
        "bst_insert" => "BST 插入：利用二叉搜索树性质，递归找到正确位置插入新节点。".to_string(),
        "string_reverse" => "字符串反转：利用双指针从两端向中间交换字符。".to_string(),
        "gcd" => "辗转相除法：gcd(a,b) = gcd(b, a mod b)，直到余数为 0。".to_string(),
        "is_prime" => "素数判断：试除法，只需检查 2 到 sqrt(n) 是否能整除。".to_string(),
        "hanoi" => {
            "汉诺塔：经典递归问题，将 n 个盘子分解为移动 n-1 个盘子 + 移动最底下盘子 + 再移动 n-1 个盘子。".to_string()
        }
        "seq_list" => "顺序表：用连续数组存储数据，支持按位置插入、删除和查找。".to_string(),
        "linked_list_append" => "链表尾插法：将新节点追加到链表末尾，保持插入顺序。".to_string(),
        "circular_queue" => "循环队列：用数组实现队列，front/rear 指针循环移动，牺牲一个单元区分空和满。".to_string(),
        "linked_stack" => "链栈：用单链表实现栈，top 指针指向栈顶，没有固定容量限制。".to_string(),
        "linked_queue" => "链队列：用链表实现队列，front 指向队头，rear 指向队尾。".to_string(),
        "level_order" => "层序遍历：利用队列按从上到下、从左到右的顺序访问二叉树节点。".to_string(),
        "bst_search" => "BST 查找：利用二叉搜索树性质，每次比较可排除一半子树，平均时间复杂度 O(log n)。".to_string(),
        "hash_table" => "哈希表：通过哈希函数直接定位存储位置，理想情况下查找时间复杂度 O(1)。".to_string(),
        "josephus" => "约瑟夫环：经典的循环报数淘汰问题，可用数组模拟圆圈解决。".to_string(),
        "circular_linked_list" => "循环链表：尾节点 next 回指头节点，遍历时需用 do-while 判断终止。".to_string(),
        "static_linked_list" => "静态链表：用数组游标模拟指针，下标 0 作为备用链表头。".to_string(),
        "string_match_bf" => {
            "朴素模式匹配：双指针逐位比较，失配后主串回溯、模式串归零。时间复杂度 O(m·n)。".to_string()
        }
        "string_match_kmp" => "KMP 模式匹配：利用 next 数组避免主串回溯，时间复杂度 O(m+n)。".to_string(),
        "threaded_binary_tree" => "线索二叉树：利用空指针域存储中序前驱和后继，实现无栈遍历。".to_string(),
        "huffman_tree" => "哈夫曼树：每次选两个最小权值节点合并，构造带权路径长度最小的二叉树。".to_string(),
        "union_find" => "并查集：用数组表示森林，支持路径压缩和按秩合并，近乎 O(1) 的查询与合并。".to_string(),
        "avl_tree" => "AVL 树：自平衡二叉搜索树，通过旋转保持左右子树高度差不超过 1。".to_string(),
        "prim_mst" => "Prim 最小生成树：从顶点出发，每次选连接两集合的最小边，贪心扩展生成树。".to_string(),
        "kruskal_mst" => "Kruskal 最小生成树：按边权排序后用并查集判环，贪心选取不形成环的边。".to_string(),
        "dijkstra" => "Dijkstra 最短路径：单源最短路径，每次选距离最近的未确定顶点进行松弛。".to_string(),
        "floyd" => "Floyd 最短路径：三重循环枚举中间点，动态规划求所有顶点对最短路径。".to_string(),
        "topological_sort" => "拓扑排序：Kahn 算法，利用入度和队列，输出有向无环图的一个线性序列。".to_string(),
        "radix_sort" => "基数排序：按位进行稳定计数排序，从低位到高位依次分配收集。时间复杂度 O(d·(n+k))。".to_string(),
        _ => String::new(),
    };

    AlgorithmMatch {
        name: name.to_string(),
        display_name: display_name.to_string(),
        func_name: func_name.to_string(),
        confidence: 85,
        suggestion,
        line,
        vis_events: compare_lines
            .iter()
            .map(|&(line, ty, ref ctx)| VisEvent {
                line,
                ty,
                extra0: 0,
                extra1: 0,
                extra2: 0,
                context: ctx.clone(),
            })
            .collect(),
    }
}
