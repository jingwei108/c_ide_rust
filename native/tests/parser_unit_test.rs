use cide_native::compiler::ast::{BinaryOp, Expr, Stmt};
use cide_native::compiler::lexer::Lexer;
use cide_native::compiler::parser::Parser;

fn parse(
    src: &str,
) -> (
    Option<cide_native::compiler::ast::ProgramNode>,
    Vec<cide_native::compiler::parser::ParseError>,
) {
    let (tokens, _) = Lexer::new(src).tokenize();
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
fn test_parser_struct_multi_field_decl() {
    // 修复：struct 体应支持逗号分隔的多字段声明（如 int u, v, w;）
    let (program, errors) = parse("struct Edge { int u, v, w; }; int main() { return 0; }");
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    assert_eq!(program.structs.len(), 1);
    assert_eq!(program.structs[0].name, "Edge");
    assert_eq!(program.structs[0].fields.len(), 3);
    assert_eq!(program.structs[0].fields[0].name, "u");
    assert_eq!(program.structs[0].fields[1].name, "v");
    assert_eq!(program.structs[0].fields[2].name, "w");
}

#[test]
fn test_parser_typedef_struct() {
    let (program, errors) = parse("typedef struct { int x; } Point; int main() { Point p; return 0; }");
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    assert_eq!(program.structs.len(), 1);
}

#[test]
fn test_parser_typedef_enum_anon() {
    // 修复：typedef enum { A, B } Tag; 应被正确解析
    let (program, errors) = parse("typedef enum { Link, Thread } PointerTag; int main() { return 0; }");
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    assert_eq!(program.globals.len(), 2);
    assert_eq!(program.globals[0].name, "Link");
    assert_eq!(program.globals[1].name, "Thread");
}

#[test]
fn test_parser_enum_anon() {
    // 修复：enum { NAME, PARENS }; 匿名枚举声明应被支持
    let (program, errors) = parse("enum { NAME, PARENS, BRACKETS }; int main() { return 0; }");
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    assert_eq!(program.globals.len(), 3);
    assert_eq!(program.globals[0].name, "NAME");
    assert_eq!(program.globals[1].name, "PARENS");
    assert_eq!(program.globals[2].name, "BRACKETS");
}

#[test]
fn test_parser_if_else() {
    let (program, errors) = parse("int main() { if (1) { return 1; } else { return 0; } }");
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    let body = program.funcs[0].body.as_ref().unwrap();
    if let Stmt::Block { stmts, .. } = body {
        if let Stmt::If {
            cond, then_stmt: _, else_stmt, ..
        } = &stmts[0]
        {
            assert!(matches!(cond, Expr::Literal { value: 1, .. }));
            assert!(else_stmt.is_some());
        } else {
            panic!("Expected If statement");
        }
    } else {
        panic!("Expected Block");
    }
}

#[test]
fn test_parser_binary_expr() {
    let (program, errors) = parse("int main() { return 1 + 2 * 3; }");
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    let body = program.funcs[0].body.as_ref().unwrap();
    if let Stmt::Block { stmts, .. } = body {
        if let Stmt::Return { value: Some(ret), .. } = &stmts[0] {
            if let Expr::Binary { op, .. } = ret {
                assert_eq!(*op, BinaryOp::Add);
            } else {
                panic!("Expected binary expression");
            }
        } else {
            panic!("Expected return statement");
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
    assert!(!errors.is_empty()); // should have parse errors but not panic
}
