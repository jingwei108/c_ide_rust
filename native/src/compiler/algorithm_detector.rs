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
    let features = extract_features(body);

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
        || (features.has_nested_loops
            && features.has_shift_pattern
            && features.loop_depth >= 2
            && !features.has_swap)
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
        || (features.is_recursive
            && features.has_partition_pattern
            && features.has_nested_loops)
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
        || (features.is_recursive
            && features.has_merge_pattern
            && !features.has_partition_pattern)
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
        || (features.has_single_loop
            && features.has_mid_calculation
            && features.has_left_right_update)
    {
        matches.push(build_match(
            "binary_search",
            "二分查找",
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
            if let Some(ref s) = step {
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
            if matches!(op, BinaryOp::Add | BinaryOp::Div) {
                let es = expr_to_string(expr);
                if es.contains("mid")
                    || (expr_to_string(left).contains("left") && expr_to_string(right).contains("right"))
                {
                    f.has_mid_calculation = true;
                }
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

            // 检测 left/right 更新
            let s = expr_to_string(expr);
            if (s.contains("left") || s.contains("right")) && s.contains("mid") {
                f.has_left_right_update = true;
            }

            // 检测 shift 模式：arr[j+1] = arr[j] 或类似的后移操作
            if matches!(op, AssignOp::Assign)
                && s.contains('[') && expr_to_string(right).contains('[') {
                f.has_shift_pattern = true;
            }
        }
        Expr::Ternary { cond, then_branch, else_branch, .. } => {
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
        Expr::Ternary { cond, then_branch, else_branch, .. } => {
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
    matches!(op, BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge | BinaryOp::Eq | BinaryOp::Ne)
}

fn is_index_access(expr: &Expr) -> bool {
    matches!(expr, Expr::Index { .. })
}

fn is_adjacent_compare(a: &Expr, b: &Expr) -> bool {
    // 检查是否是 arr[x] 和 arr[x+1] 的比较（基于 AST 结构，而非字符串格式）
    if let Expr::Index { array: arr_a, index: idx_a, .. } = a {
        if let Expr::Index { array: arr_b, index: idx_b, .. } = b {
            if expr_to_string(arr_a) == expr_to_string(arr_b) {
                // 检查 idx_b 是否是 idx_a + 1
                if let Expr::Binary { op: BinaryOp::Add, left, right, .. } = idx_b.as_ref() {
                    if expr_to_string(left) == expr_to_string(idx_a) {
                        if let Expr::Literal { value: 1, .. } = right.as_ref() {
                            return true;
                        }
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
            };
            format!("{} {} {}", expr_to_string(left), op_str, expr_to_string(right))
        }
        Expr::Ternary { cond, then_branch, else_branch, .. } => {
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
            let elems: Vec<String> = elements.iter().map(expr_to_string).collect();
            format!("{{{}}}", elems.join(", "))
        }
        Expr::CallPtr { .. } => "fp(...)".to_string(),
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
        "bubble_sort" => "冒泡排序：通过相邻元素两两比较并交换，将最大元素逐步「冒泡」到数组末尾。时间复杂度 O(n²)。".to_string(),
        "selection_sort" => "选择排序：每次从未排序部分选择最小元素，放到已排序部分末尾。时间复杂度 O(n²)。".to_string(),
        "insertion_sort" => "插入排序：将元素逐个插入到已排序部分的正确位置。时间复杂度 O(n²)，对近乎有序的数组效率很高。".to_string(),
        "quick_sort" => "快速排序：通过分治法，选取枢轴将数组分区，再递归排序子数组。平均时间复杂度 O(n log n)。".to_string(),
        "merge_sort" => "归并排序：将数组递归分成两半，排序后合并。时间复杂度稳定为 O(n log n)。".to_string(),
        "binary_search" => "二分查找：在有序数组中每次将搜索范围减半。时间复杂度 O(log n)。".to_string(),
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
