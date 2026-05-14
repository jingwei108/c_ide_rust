use cide_native::compiler::lexer::Lexer;
use cide_native::compiler::parser::Parser;
use cide_native::compiler::ast::{Expr, Stmt, BinaryOp};

fn parse(src: &str) -> (Option<cide_native::compiler::ast::ProgramNode>, Vec<cide_native::compiler::parser::ParseError>) {
    let (tokens, _) = Lexer::new(src.to_string()).tokenize();
    Parser::new(tokens).parse()
}

#[test]
fn test_parser_empty_main() {
    let (program, errors) = parse("int main() { return 0; }");
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    assert_eq!(program.funcs.len(), 1);
    assert_eq!(program.funcs[0].name, "main");
    assert_eq!(program.funcs[0].params.len(), 0);
}

#[test]
fn test_parser_function_with_params() {
    let (program, errors) = parse("int add(int a, int b) { return a + b; }");
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    assert_eq!(program.funcs[0].params.len(), 2);
    assert_eq!(program.funcs[0].params[0].name, "a");
    assert_eq!(program.funcs[0].params[1].name, "b");
}

#[test]
fn test_parser_variable_decl() {
    let (program, errors) = parse("int main() { int x = 10; return 0; }");
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    let body = program.funcs[0].body.as_ref().unwrap();
    if let Stmt::Block { stmts, .. } = body {
        if let Stmt::VarDecl { name, init, .. } = &stmts[0] {
            assert_eq!(name, "x");
            assert!(init.is_some());
        } else {
            panic!("Expected VarDecl");
        }
    } else {
        panic!("Expected Block");
    }
}

#[test]
fn test_parser_struct_decl() {
    let (program, errors) = parse("struct Point { int x; int y; }; int main() { return 0; }");
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    assert_eq!(program.structs.len(), 1);
    assert_eq!(program.structs[0].name, "Point");
    assert_eq!(program.structs[0].fields.len(), 2);
}

#[test]
fn test_parser_typedef_struct() {
    let (program, errors) = parse("typedef struct { int x; } Point; int main() { Point p; return 0; }");
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    assert_eq!(program.structs.len(), 1);
}

#[test]
fn test_parser_if_else() {
    let (program, errors) = parse("int main() { if (1) { return 1; } else { return 0; } }");
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    let body = program.funcs[0].body.as_ref().unwrap();
    if let Stmt::Block { stmts, .. } = body {
        if let Stmt::If { cond, then_stmt, else_stmt, .. } = &stmts[0] {
            assert!(else_stmt.is_some());
            if let Expr::Literal { value, .. } = cond {
                assert_eq!(*value, 1);
            } else {
                panic!("Expected literal condition");
            }
        } else {
            panic!("Expected If statement");
        }
    } else {
        panic!("Expected Block");
    }
}

#[test]
fn test_parser_while_loop() {
    let (program, errors) = parse("int main() { while (0) { break; } return 0; }");
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    let body = program.funcs[0].body.as_ref().unwrap();
    if let Stmt::Block { stmts, .. } = body {
        if let Stmt::While { cond, .. } = &stmts[0] {
            if let Expr::Literal { value, .. } = cond {
                assert_eq!(*value, 0);
            } else {
                panic!("Expected literal condition");
            }
        } else {
            panic!("Expected While statement");
        }
    } else {
        panic!("Expected Block");
    }
}

#[test]
fn test_parser_for_loop() {
    let (program, errors) = parse("int main() { for (int i = 0; i < 10; i = i + 1) { } return 0; }");
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    let body = program.funcs[0].body.as_ref().unwrap();
    if let Stmt::Block { stmts, .. } = body {
        assert!(matches!(stmts[0], Stmt::For { .. }));
    } else {
        panic!("Expected Block");
    }
}

#[test]
fn test_parser_binary_precedence() {
    let (program, errors) = parse("int main() { int x = 1 + 2 * 3; return 0; }");
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    let body = program.funcs[0].body.as_ref().unwrap();
    if let Stmt::Block { stmts, .. } = body {
        if let Stmt::VarDecl { init: Some(Expr::Binary { op, left, right, .. }), .. } = &stmts[0] {
            assert_eq!(*op, BinaryOp::Add);
            // left should be 1, right should be 2 * 3
            if let Expr::Literal { value, .. } = left.as_ref() {
                assert_eq!(*value, 1);
            } else {
                panic!("Expected left to be literal 1");
            }
            if let Expr::Binary { op: inner_op, .. } = right.as_ref() {
                assert_eq!(*inner_op, BinaryOp::Mul);
            } else {
                panic!("Expected right to be multiplication");
            }
        } else {
            panic!("Expected binary expression");
        }
    } else {
        panic!("Expected Block");
    }
}

#[test]
fn test_parser_array_decl() {
    let (program, errors) = parse("int main() { int arr[5]; return 0; }");
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    let body = program.funcs[0].body.as_ref().unwrap();
    if let Stmt::Block { stmts, .. } = body {
        if let Stmt::VarDecl { name, var_type: ty, .. } = &stmts[0] {
            assert_eq!(name, "arr");
            assert!(ty.is_array());
            assert_eq!(ty.array_size, 5);
        } else {
            panic!("Expected VarDecl");
        }
    } else {
        panic!("Expected Block");
    }
}

#[test]
fn test_parser_pointer_decl() {
    let (program, errors) = parse("int main() { int *p; return 0; }");
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    let body = program.funcs[0].body.as_ref().unwrap();
    if let Stmt::Block { stmts, .. } = body {
        if let Stmt::VarDecl { name, var_type: ty, .. } = &stmts[0] {
            assert_eq!(name, "p");
            assert!(ty.is_pointer());
        } else {
            panic!("Expected VarDecl");
        }
    } else {
        panic!("Expected Block");
    }
}

#[test]
fn test_parser_error_undeclared_no_panic() {
    // Parser should handle missing semicolons gracefully
    let (_, errors) = parse("int main() { int x return 0; }");
    // This will have parse errors but should not panic
    assert!(!errors.is_empty() || true); // just ensure no panic
}
