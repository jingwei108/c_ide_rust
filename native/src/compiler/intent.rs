//! Code intent inference (P3).
//!
//! Infers the high-level intent of a function based on:
//! - Function name heuristics
//! - Variable naming patterns
//! - Control flow structure (CFG features)
//! - Data flow patterns

use flutter_rust_bridge::frb;
use crate::compiler::ast::{FuncDecl, Stmt};
use crate::compiler::cfg::ControlFlowGraph;

/// High-level code intent categories.
#[frb]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeIntent {
    Sort,
    Search,
    Traverse,
    Compute,
    Transform,
    Unknown,
}

impl CodeIntent {
    pub fn as_str(&self) -> &'static str {
        match self {
            CodeIntent::Sort => "sort",
            CodeIntent::Search => "search",
            CodeIntent::Traverse => "traverse",
            CodeIntent::Compute => "compute",
            CodeIntent::Transform => "transform",
            CodeIntent::Unknown => "unknown",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            CodeIntent::Sort => "排序",
            CodeIntent::Search => "查找",
            CodeIntent::Traverse => "遍历",
            CodeIntent::Compute => "计算",
            CodeIntent::Transform => "转换",
            CodeIntent::Unknown => "未知",
        }
    }
}

/// A scored intent guess.
#[frb]
#[derive(Debug, Clone)]
pub struct IntentScore {
    pub intent: CodeIntent,
    pub score: i32,
    pub reasons: Vec<String>,
}

/// Infer the likely intent(s) of a function.
/// Returns a sorted list (highest score first).
pub fn infer_intent(func: &FuncDecl) -> Vec<IntentScore> {
    let mut scores = vec![
        IntentScore { intent: CodeIntent::Sort, score: 0, reasons: vec![] },
        IntentScore { intent: CodeIntent::Search, score: 0, reasons: vec![] },
        IntentScore { intent: CodeIntent::Traverse, score: 0, reasons: vec![] },
        IntentScore { intent: CodeIntent::Compute, score: 0, reasons: vec![] },
        IntentScore { intent: CodeIntent::Transform, score: 0, reasons: vec![] },
    ];

    let name_lower = func.name.to_lowercase();

    // --- Name heuristics ---
    for s in &mut scores {
        match &s.intent {
            CodeIntent::Sort
                if name_lower.contains("sort")
                    || name_lower.contains("bubble")
                    || name_lower.contains("order") =>
            {
                s.score += 50;
                s.reasons.push("函数名包含排序关键词".to_string());
            }
            CodeIntent::Search
                if name_lower.contains("search")
                    || name_lower.contains("find")
                    || name_lower.contains("lookup")
                    || name_lower.contains("binary") =>
            {
                s.score += 50;
                s.reasons.push("函数名包含查找关键词".to_string());
            }
            CodeIntent::Traverse
                if name_lower.contains("travers")
                    || name_lower.contains("walk")
                    || name_lower.contains("visit")
                    || name_lower.contains("print") =>
            {
                s.score += 40;
                s.reasons.push("函数名包含遍历关键词".to_string());
            }
            CodeIntent::Compute
                if name_lower.contains("calc")
                    || name_lower.contains("sum")
                    || name_lower.contains("count")
                    || name_lower.contains("max")
                    || name_lower.contains("min") =>
            {
                s.score += 40;
                s.reasons.push("函数名包含计算关键词".to_string());
            }
            CodeIntent::Transform
                if name_lower.contains("convert")
                    || name_lower.contains("revers")
                    || name_lower.contains("rotat") =>
            {
                s.score += 40;
                s.reasons.push("函数名包含转换关键词".to_string());
            }
            _ => {}
        }
    }

    // --- CFG structure heuristics ---
    if let (Some(body), Some(cfg)) = (func.body.as_ref(), ControlFlowGraph::from_func(func)) {
        let has_loop = !cfg.find_loops().is_empty();
        let has_back_edge = cfg.edges.iter().any(|(a, b)| *a >= *b);
        let unreachable = cfg.find_unreachable_blocks();

        for s in &mut scores {
            match &s.intent {
                CodeIntent::Sort if has_loop && has_back_edge => {
                    s.score += 20;
                    s.reasons.push("存在循环结构（排序通常需要循环）".to_string());
                }
                CodeIntent::Search => {
                    if has_loop && !has_back_edge {
                        s.score += 15;
                        s.reasons.push("存在前向循环（线性查找特征）".to_string());
                    }
                    if !unreachable.is_empty() {
                        s.score += 10;
                        s.reasons.push("存在提前返回路径（查找常提前退出）".to_string());
                    }
                }
                CodeIntent::Traverse if has_loop => {
                    s.score += 20;
                    s.reasons.push("存在循环结构".to_string());
                }
                CodeIntent::Compute => {
                    if has_loop {
                        s.score += 10;
                        s.reasons.push("存在循环结构".to_string());
                    } else {
                        s.score += 15;
                        s.reasons.push("无循环（纯计算特征）".to_string());
                    }
                }
                CodeIntent::Transform if has_loop && has_back_edge => {
                    s.score += 15;
                    s.reasons.push("存在循环结构".to_string());
                }
                _ => {}
            }
        }

        // --- Variable naming heuristics (from AST) ---
        let var_names = collect_var_names(body);
        for s in &mut scores {
            match &s.intent {
                CodeIntent::Sort
                    if var_names
                        .iter()
                        .any(|n| n.contains("swap") || n.contains("temp") || n.contains("tmp")) =>
                {
                    s.score += 15;
                    s.reasons.push("变量名包含 swap/temp（交换操作特征）".to_string());
                }
                CodeIntent::Search
                    if var_names.iter().any(|n| {
                        n.contains("mid")
                            || n.contains("left")
                            || n.contains("right")
                            || n.contains("target")
                            || n.contains("found")
                    }) =>
                {
                    s.score += 20;
                    s.reasons.push("变量名包含 mid/left/right/target（查找特征）".to_string());
                }
                CodeIntent::Traverse
                    if var_names
                        .iter()
                        .any(|n| n.contains("next") || n.contains("curr") || n.contains("head")) =>
                {
                    s.score += 15;
                    s.reasons.push("变量名包含 next/curr/head（链表遍历特征）".to_string());
                }
                CodeIntent::Compute
                    if var_names.iter().any(|n| {
                        n.contains("sum")
                            || n.contains("total")
                            || n.contains("count")
                            || n.contains("result")
                            || n.contains("res")
                    }) =>
                {
                    s.score += 15;
                    s.reasons.push("变量名包含 sum/result（累加特征）".to_string());
                }
                CodeIntent::Transform
                    if var_names
                        .iter()
                        .any(|n| n.contains("new") || n.contains("out") || n.contains("buf")) =>
                {
                    s.score += 10;
                    s.reasons.push("变量名包含 new/out/buf（转换特征）".to_string());
                }
                _ => {}
            }
        }
    }

    // --- Recursive heuristics ---
    if let Some(body) = func.body.as_ref() {
        if has_recursive_call(body, &func.name) {
            for s in &mut scores {
                match &s.intent {
                    CodeIntent::Sort => {
                        s.score += 15;
                        s.reasons.push("递归调用（快速/归并排序特征）".to_string());
                    }
                    CodeIntent::Search => {
                        s.score += 20;
                        s.reasons.push("递归调用（二分查找特征）".to_string());
                    }
                    CodeIntent::Traverse => {
                        s.score += 15;
                        s.reasons.push("递归调用（树遍历特征）".to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    scores.sort_by_key(|s| std::cmp::Reverse(s.score));
    scores.retain(|s| s.score > 0);
    scores
}

fn collect_var_names(stmt: &Stmt) -> Vec<String> {
    let mut names = Vec::new();
    match stmt {
        Stmt::Block { stmts, .. } => {
            for s in stmts {
                names.extend(collect_var_names(s));
            }
        }
        Stmt::If { then_stmt, else_stmt, .. } => {
            names.extend(collect_var_names(then_stmt));
            if let Some(e) = else_stmt {
                names.extend(collect_var_names(e));
            }
        }
        Stmt::While { body, .. } => {
            names.extend(collect_var_names(body));
        }
        Stmt::DoWhile { body, .. } => {
            names.extend(collect_var_names(body));
        }
        Stmt::For { init, body, .. } => {
            if let Some(i) = init {
                names.extend(collect_var_names(i));
            }
            names.extend(collect_var_names(body));
        }
        Stmt::VarDecl { name, extra_vars, .. } => {
            names.push(name.clone());
            for (_, n, _) in extra_vars {
                names.push(n.clone());
            }
        }
        Stmt::Switch { body, .. } => {
            names.extend(collect_var_names(body));
        }
        Stmt::Case { stmt: s, .. } => {
            names.extend(collect_var_names(s));
        }
        _ => {}
    }
    names
}

fn has_recursive_call(stmt: &Stmt, func_name: &str) -> bool {
    match stmt {
        Stmt::Block { stmts, .. } => stmts.iter().any(|s| has_recursive_call(s, func_name)),
        Stmt::If { then_stmt, else_stmt, .. } => {
            has_recursive_call(then_stmt, func_name)
                || else_stmt.as_ref().is_some_and(|e| has_recursive_call(e, func_name))
        }
        Stmt::While { body, .. } => has_recursive_call(body, func_name),
        Stmt::DoWhile { body, .. } => has_recursive_call(body, func_name),
        Stmt::For { body, .. } => has_recursive_call(body, func_name),
        Stmt::Expr { expr, .. } => expr_has_call(expr, func_name),
        Stmt::VarDecl { init, extra_vars, .. } => {
            init.as_ref().is_some_and(|e| expr_has_call(e, func_name))
                || extra_vars
                    .iter()
                    .any(|(_, _, e)| e.as_ref().is_some_and(|x| expr_has_call(x, func_name)))
        }
        Stmt::Return { value, .. } => {
            value.as_ref().is_some_and(|e| expr_has_call(e, func_name))
        }
        Stmt::Switch { body, .. } => has_recursive_call(body, func_name),
        Stmt::Case { stmt: s, .. } => has_recursive_call(s, func_name),
        _ => false,
    }
}

fn expr_has_call(expr: &crate::compiler::ast::Expr, func_name: &str) -> bool {
    use crate::compiler::ast::Expr;
    match expr {
        Expr::Call { name, .. } => name == func_name,
        Expr::Binary { left, right, .. } => {
            expr_has_call(left, func_name) || expr_has_call(right, func_name)
        }
        Expr::Unary { operand, .. } => expr_has_call(operand, func_name),
        Expr::Assign { left, right, .. } => {
            expr_has_call(left, func_name) || expr_has_call(right, func_name)
        }
        Expr::CallPtr { callee, args, .. } => {
            expr_has_call(callee, func_name) || args.iter().any(|a| expr_has_call(a, func_name))
        }
        Expr::Index { array, index, .. } => {
            expr_has_call(array, func_name) || expr_has_call(index, func_name)
        }
        Expr::Member { object, .. } => expr_has_call(object, func_name),
        Expr::Ternary { cond, then_branch, else_branch, .. } => {
            expr_has_call(cond, func_name)
                || expr_has_call(then_branch, func_name)
                || expr_has_call(else_branch, func_name)
        }
        Expr::Cast { expr: e, .. } => expr_has_call(e, func_name),
        Expr::Sizeof { operand: Some(e), .. } => expr_has_call(e, func_name),
        Expr::InitList { elements, .. } => elements.iter().any(|e| expr_has_call(e, func_name)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_func(source: &str) -> FuncDecl {
        let (tokens, _) = crate::compiler::lexer::Lexer::new(source).tokenize();
        let (program, _) = crate::compiler::parser::Parser::new(tokens).parse();
        program.unwrap().funcs.into_iter().next().unwrap()
    }

    #[test]
    fn test_intent_sort() {
        let func = parse_func("void bubble_sort(int a[], int n) { int i, j; for(i=0;i<n;i++) for(j=0;j<n-1;j++) if(a[j]>a[j+1]) { int t=a[j]; a[j]=a[j+1]; a[j+1]=t; } }");
        let intents = infer_intent(&func);
        assert!(!intents.is_empty());
        assert_eq!(intents[0].intent, CodeIntent::Sort);
    }

    #[test]
    fn test_intent_search() {
        let func = parse_func("int binary_search(int a[], int n, int target) { int left=0, right=n-1; while(left<=right) { int mid=(left+right)/2; if(a[mid]==target) return mid; if(a[mid]<target) left=mid+1; else right=mid-1; } return -1; }");
        let intents = infer_intent(&func);
        assert!(!intents.is_empty());
        assert_eq!(intents[0].intent, CodeIntent::Search);
    }

    #[test]
    fn test_intent_compute() {
        let func = parse_func("int sum(int a[], int n) { int total=0; for(int i=0;i<n;i++) total+=a[i]; return total; }");
        let intents = infer_intent(&func);
        assert!(!intents.is_empty());
        assert_eq!(intents[0].intent, CodeIntent::Compute);
    }
}
