use cide_native::compiler::lexer::Lexer;
use cide_native::compiler::parser::Parser;
use cide_native::compiler::type_checker::TypeChecker;
use cide_native::compiler::ast::{Expr, Stmt};

fn find_expr<'a>(stmt: &'a Stmt, pred: &dyn Fn(&Expr) -> bool) -> Option<&'a Expr> {
    match stmt {
        Stmt::Expr { expr, .. } => find_expr_in_expr(expr, pred),
        Stmt::VarDecl { init, extra_vars, .. } => {
            if let Some(e) = init.as_ref().and_then(|e| find_expr_in_expr(e, pred)) { return Some(e); }
            for (_, e) in extra_vars {
                if let Some(e) = e.as_ref().and_then(|e| find_expr_in_expr(e, pred)) { return Some(e); }
            }
            None
        }
        Stmt::If { cond, then_stmt, else_stmt, .. } => {
            if let Some(e) = find_expr_in_expr(cond, pred) { return Some(e); }
            if let Some(e) = find_expr(then_stmt, pred) { return Some(e); }
            if let Some(e) = else_stmt.as_ref().and_then(|s| find_expr(s, pred)) { return Some(e); }
            None
        }
        Stmt::While { cond, body, .. } => {
            if let Some(e) = find_expr_in_expr(cond, pred) { return Some(e); }
            if let Some(e) = find_expr(body, pred) { return Some(e); }
            None
        }
        Stmt::DoWhile { cond, body, .. } => {
            if let Some(e) = find_expr_in_expr(cond, pred) { return Some(e); }
            if let Some(e) = find_expr(body, pred) { return Some(e); }
            None
        }
        Stmt::For { init, cond, step, body, .. } => {
            if let Some(e) = init.as_ref().and_then(|s| find_expr(s, pred)) { return Some(e); }
            if let Some(e) = cond.as_ref().and_then(|e| find_expr_in_expr(e, pred)) { return Some(e); }
            if let Some(e) = step.as_ref().and_then(|e| find_expr_in_expr(e, pred)) { return Some(e); }
            if let Some(e) = find_expr(body, pred) { return Some(e); }
            None
        }
        Stmt::Block { stmts, .. } => {
            for s in stmts {
                if let Some(e) = find_expr(s, pred) { return Some(e); }
            }
            None
        }
        Stmt::Return { value, .. } => {
            value.as_ref().and_then(|e| find_expr_in_expr(e, pred))
        }
        _ => None,
    }
}

fn find_expr_in_expr<'a>(expr: &'a Expr, pred: &dyn Fn(&Expr) -> bool) -> Option<&'a Expr> {
    if pred(expr) { return Some(expr); }
    match expr {
        Expr::Binary { left, right, .. } => {
            find_expr_in_expr(left, pred).or_else(|| find_expr_in_expr(right, pred))
        }
        Expr::Unary { operand, .. } => find_expr_in_expr(operand, pred),
        Expr::Call { args, .. } => {
            for a in args {
                if let Some(e) = find_expr_in_expr(a, pred) { return Some(e); }
            }
            None
        }
        Expr::Index { array, index, .. } => {
            find_expr_in_expr(array, pred).or_else(|| find_expr_in_expr(index, pred))
        }
        Expr::Member { object, .. } => find_expr_in_expr(object, pred),
        Expr::Cast { expr, .. } => find_expr_in_expr(expr, pred),
        Expr::Ternary { cond, then_branch, else_branch, .. } => {
            find_expr_in_expr(cond, pred)
                .or_else(|| find_expr_in_expr(then_branch, pred))
                .or_else(|| find_expr_in_expr(else_branch, pred))
        }
        Expr::Assign { left, right, .. } => {
            find_expr_in_expr(left, pred).or_else(|| find_expr_in_expr(right, pred))
        }
        _ => None,
    }
}

#[test]
fn temp_test_ptr_array() {
    let src = r#"
#include <stdio.h>
int main() {
    int a = 10, b = 20, c = 30;
    int *arr[3];
    arr[0] = &a;
    arr[1] = &b;
    arr[2] = &c;
    for (int i = 0; i < 3; i++) {
        printf("%d ", *arr[i]);
    }
    printf("\n");
    return 0;
}
"#;
    let (tokens, _lex_errors) = Lexer::new(src.to_string()).tokenize();
    let (maybe_program, _parse_errors) = Parser::new(tokens).parse();
    let mut program = maybe_program.unwrap();
    let (type_errors, _warnings, _hints) = TypeChecker::new().check(&mut program);
    for e in &type_errors {
        eprintln!("TypeError: {:?}", e);
    }
    
    // Find the printf call and inspect arg types
    for f in &program.funcs {
        if let Some(ref body) = f.body {
            if let Some(Expr::Call { args, .. }) = find_expr(body, &|e| matches!(e, Expr::Call { name, .. } if name == "printf")) {
                for (i, arg) in args.iter().enumerate() {
                    eprintln!("printf arg {}: ty = {:?}, kind = {:?}", i, arg.ty(), arg.ty().kind);
                }
            }
        }
    }
}
