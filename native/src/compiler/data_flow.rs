//! Simplified data-flow analyses (P3).
//!
//! Currently provides:
//! - Live-variable analysis (to detect dead stores).
//! - Constant propagation (to detect always-true/always-false conditions).

use crate::compiler::ast::{Expr, Stmt};
use crate::compiler::cfg::{BasicBlock, BlockId, ControlFlowGraph};
use std::collections::{HashMap, HashSet};

/// A single variable definition or use.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VarRef {
    pub name: String,
    pub block: BlockId,
    pub stmt_index: usize,
}

/// Result of live-variable analysis.
/// Maps each block to the set of variable names that are live at its entry.
pub type LiveVarResult = HashMap<BlockId, HashSet<String>>;

/// Perform a simplified live-variable analysis on the CFG.
///
/// A variable is *live* at a program point if its value may be read later
/// before being overwritten.
pub fn analyze_live_variables(cfg: &ControlFlowGraph) -> LiveVarResult {
    let mut live_in: HashMap<BlockId, HashSet<String>> = HashMap::new();
    let mut live_out: HashMap<BlockId, HashSet<String>> = HashMap::new();

    // Initialize.
    for block in &cfg.blocks {
        live_in.insert(block.id, HashSet::new());
        live_out.insert(block.id, HashSet::new());
    }

    // Iterative fixed-point (backwards).
    let mut changed = true;
    while changed {
        changed = false;
        for block in &cfg.blocks {
            let id = block.id;
            let mut new_out = HashSet::new();
            for &(from, to) in &cfg.edges {
                if from == id {
                    new_out.extend(live_in.get(&to).cloned().unwrap_or_default());
                }
            }
            let new_in = block_live_in(block, &new_out);

            if let Some(old_in) = live_in.get(&id) {
                if *old_in != new_in {
                    changed = true;
                    live_in.insert(id, new_in);
                }
            }
            live_out.insert(id, new_out);
        }
    }

    live_in
}

fn block_live_in(block: &BasicBlock, out_set: &HashSet<String>) -> HashSet<String> {
    let mut result = out_set.clone();
    // Walk statements backwards.
    for stmt in block.stmts.iter().rev() {
        // Kill: variable is written (defined).
        for var in defined_vars(stmt) {
            result.remove(&var);
        }
        // Gen: variable is read (used).
        for var in used_vars(stmt) {
            result.insert(var);
        }
    }
    result
}

/// Extract variable names that are used (read) by a statement.
fn used_vars(stmt: &Stmt) -> Vec<String> {
    let mut vars = Vec::new();
    match stmt {
        Stmt::VarDecl { init, extra_vars, .. } => {
            if let Some(e) = init {
                collect_expr_vars(e, &mut vars, true);
            }
            for (_, _, init2) in extra_vars {
                if let Some(e) = init2 {
                    collect_expr_vars(e, &mut vars, true);
                }
            }
        }
        Stmt::Expr { expr, .. } => {
            collect_expr_vars(expr, &mut vars, true);
        }
        Stmt::If { cond, .. } => {
            collect_expr_vars(cond, &mut vars, true);
        }
        Stmt::While { cond, .. } => {
            collect_expr_vars(cond, &mut vars, true);
        }
        Stmt::DoWhile { cond, .. } => {
            collect_expr_vars(cond, &mut vars, true);
        }
        Stmt::For { cond, step, .. } => {
            if let Some(e) = cond {
                collect_expr_vars(e, &mut vars, true);
            }
            if let Some(e) = step {
                collect_expr_vars(e, &mut vars, true);
            }
        }
        Stmt::Return { value, .. } => {
            if let Some(e) = value {
                collect_expr_vars(e, &mut vars, true);
            }
        }
        Stmt::Switch { cond, .. } => {
            collect_expr_vars(cond, &mut vars, true);
        }
        _ => {}
    }
    vars
}

/// Extract variable names that are defined (written) by a statement.
fn defined_vars(stmt: &Stmt) -> Vec<String> {
    let mut vars = Vec::new();
    match stmt {
        Stmt::VarDecl { name, extra_vars, .. } => {
            vars.push(name.clone());
            for (_, n, _) in extra_vars {
                vars.push(n.clone());
            }
        }
        Stmt::Expr { expr, .. } => {
            collect_expr_vars(expr, &mut vars, false);
        }
        Stmt::For { init, .. } => {
            if let Some(s) = init {
                vars.extend(defined_vars(s));
            }
        }
        _ => {}
    }
    vars
}

/// Collect variable names from an expression.
/// `read=true` collects used variables, `read=false` collects assigned variables.
fn collect_expr_vars(expr: &Expr, out: &mut Vec<String>, read: bool) {
    match expr {
        Expr::Identifier { name, .. } => {
            if read {
                out.push(name.clone());
            }
        }
        Expr::Binary { left, right, .. } => {
            collect_expr_vars(left, out, read);
            collect_expr_vars(right, out, read);
        }
        Expr::Unary { operand, op, .. } => {
            let is_deref = matches!(op, crate::compiler::ast::UnaryOp::Deref);
            collect_expr_vars(operand, out, read || is_deref);
        }
        Expr::Assign { left, right, .. } => {
            if read {
                collect_expr_vars(right, out, true);
            }
            collect_expr_vars(left, out, false);
        }
        Expr::Call { args, .. } => {
            for arg in args {
                collect_expr_vars(arg, out, read);
            }
        }
        Expr::CallPtr { callee, args, .. } => {
            collect_expr_vars(callee, out, read);
            for arg in args {
                collect_expr_vars(arg, out, read);
            }
        }
        Expr::Index { array, index, .. } => {
            collect_expr_vars(array, out, read);
            collect_expr_vars(index, out, read);
        }
        Expr::Member { object, .. } => {
            collect_expr_vars(object, out, read);
        }
        Expr::Ternary { cond, then_branch, else_branch, .. } => {
            collect_expr_vars(cond, out, read);
            collect_expr_vars(then_branch, out, read);
            collect_expr_vars(else_branch, out, read);
        }
        Expr::Cast { expr: inner, .. } => {
            collect_expr_vars(inner, out, read);
        }
        Expr::Sizeof { operand, .. } => {
            if let Some(e) = operand {
                collect_expr_vars(e, out, read);
            }
        }
        Expr::InitList { elements, .. } => {
            for e in elements {
                collect_expr_vars(e, out, read);
            }
        }
        _ => {}
    }
}

// ===================================================================
// Constant propagation (simplified)
// ===================================================================

/// Check whether a condition expression is always true or always false.
pub fn evaluate_constant_condition(expr: &Expr) -> Option<bool> {
    match expr {
        Expr::Literal { value, .. } => Some(*value != 0),
        Expr::Binary { op, left, right, .. } => {
            use crate::compiler::ast::BinaryOp;
            let l = match left.as_ref() {
                Expr::Literal { value, .. } => *value,
                _ => return None,
            };
            let r = match right.as_ref() {
                Expr::Literal { value, .. } => *value,
                _ => return None,
            };
            Some(match op {
                BinaryOp::Eq => l == r,
                BinaryOp::Ne => l != r,
                BinaryOp::Lt => l < r,
                BinaryOp::Le => l <= r,
                BinaryOp::Gt => l > r,
                BinaryOp::Ge => l >= r,
                BinaryOp::And => l != 0 && r != 0,
                BinaryOp::Or => l != 0 || r != 0,
                _ => return None,
            })
        }
        _ => None,
    }
}

// ===================================================================
// Unit tests
// ===================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::cfg::ControlFlowGraph;

    fn parse_func(source: &str) -> crate::compiler::ast::FuncDecl {
        let (tokens, _) = crate::compiler::lexer::Lexer::new(source).tokenize();
        let (program, _) = crate::compiler::parser::Parser::new(tokens).parse();
        program.unwrap().funcs.into_iter().next().unwrap()
    }

    #[test]
    fn test_live_variables_simple() {
        let func = parse_func("int main() { int x = 1; int y = x + 2; return y; }");
        let cfg = ControlFlowGraph::from_func(&func).unwrap();
        let live = analyze_live_variables(&cfg);
        assert!(!live.is_empty());
    }

    #[test]
    fn test_constant_condition_true() {
        let e = Expr::Literal { value: 5, loc: crate::compiler::ast::SourceLoc::default(), ty: crate::compiler::ast::Type::int() };
        assert_eq!(evaluate_constant_condition(&e), Some(true));
    }

    #[test]
    fn test_constant_condition_compare() {
        use crate::compiler::ast::{BinaryOp, SourceLoc, Type};
        let e = Expr::Binary {
            op: BinaryOp::Lt,
            left: Box::new(Expr::Literal { value: 3, loc: SourceLoc::default(), ty: Type::int() }),
            right: Box::new(Expr::Literal { value: 5, loc: SourceLoc::default(), ty: Type::int() }),
            loc: SourceLoc::default(),
            ty: Type::int(),
        };
        assert_eq!(evaluate_constant_condition(&e), Some(true));
    }
}
