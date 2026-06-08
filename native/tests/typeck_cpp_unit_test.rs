use cide_native::compiler::ast::*;
use cide_native::compiler::lexer::Lexer;
use cide_native::compiler::parser::Parser;
use cide_native::compiler::typeck::TypeChecker;

fn parse_and_typecheck_cpp(src: &str) -> (ProgramNode, Vec<cide_native::compiler::typeck::TypeError>) {
    let (tokens, _) = Lexer::with_mode(src, true).tokenize();
    let (mut program, parse_errors) = Parser::with_mode(tokens, true).parse();
    if !parse_errors.is_empty() {
        panic!("Parse errors: {:?}", parse_errors);
    }
    let mut program = program.unwrap();
    let checker = TypeChecker::default();
    let (errors, _warnings, _hints) = checker.check(&mut program);
    (program, errors)
}

// ============================================================================
// Class Type Checking Tests
// ============================================================================

#[test]
fn test_class_basic_typecheck() {
    let src = r#"
class Point {
public:
    int x;
    int y;
};
int main() {
    Point p;
    p.x = 10;
    p.y = 20;
    return p.x + p.y;
}
"#;
    let (program, errors) = parse_and_typecheck_cpp(src);
    assert!(errors.is_empty(), "Type errors: {:?}", errors);
    assert_eq!(program.classes.len(), 1);
}

#[test]
fn test_class_private_access_denied() {
    let src = r#"
class Secret {
private:
    int code;
};
int main() {
    Secret s;
    s.code = 42;
    return 0;
}
"#;
    let (_program, errors) = parse_and_typecheck_cpp(src);
    let has_private_error = errors.iter().any(|e| e.message.contains("私有"));
    assert!(has_private_error, "Expected private access error, got: {:?}", errors);
}

#[test]
fn test_class_method_call() {
    let src = r#"
class Rect {
public:
    int width;
    int area() { return width * 2; }
};
int main() {
    Rect r;
    r.width = 5;
    int a = r.area();
    return a;
}
"#;
    let (_program, errors) = parse_and_typecheck_cpp(src);
    assert!(errors.is_empty(), "Type errors: {:?}", errors);
}

#[test]
fn test_this_type() {
    let src = r#"
class Counter {
public:
    int val;
    void inc() { this->val = this->val + 1; }
};
int main() { return 0; }
"#;
    let (_program, errors) = parse_and_typecheck_cpp(src);
    assert!(errors.is_empty(), "Type errors: {:?}", errors);
}

#[test]
fn test_this_outside_class() {
    let src = r#"
int main() {
    return this->x;
}
"#;
    let (_program, errors) = parse_and_typecheck_cpp(src);
    let has_this_error = errors.iter().any(|e| e.message.contains("this"));
    assert!(has_this_error, "Expected 'this' outside class error, got: {:?}", errors);
}

// ============================================================================
// Auto Type Deduction Tests
// ============================================================================

#[test]
fn test_auto_deduce_int() {
    let src = r#"
int main() {
    auto x = 5;
    return x;
}
"#;
    let (_program, errors) = parse_and_typecheck_cpp(src);
    assert!(errors.is_empty(), "Type errors: {:?}", errors);
}

#[test]
fn test_auto_deduce_float() {
    let src = r#"
int main() {
    auto f = 3.14;
    return 0;
}
"#;
    let (_program, errors) = parse_and_typecheck_cpp(src);
    assert!(errors.is_empty(), "Type errors: {:?}", errors);
}

#[test]
fn test_auto_no_init_error() {
    let src = r#"
int main() {
    auto x;
    return 0;
}
"#;
    let (_program, errors) = parse_and_typecheck_cpp(src);
    let has_auto_error = errors.iter().any(|e| e.message.contains("auto"));
    assert!(has_auto_error, "Expected auto requires init error, got: {:?}", errors);
}

// ============================================================================
// Reference Type Tests
// ============================================================================
// NOTE: Reference declarations (`int& r = x`) require Parser support for `&`
// in declarators, which is not yet implemented (Phase 31 only added AST/Lexer
// nodes). These tests are deferred until Parser handles reference syntax.

// ============================================================================
// Template Monomorphization Tests
// ============================================================================

#[test]
fn test_template_func_monomorph() {
    let src = r#"
template <typename T> T max(T a, T b) { if (a > b) return a; return b; }
int main() {
    int m = max(3, 5);
    return m;
}
"#;
    let (program, errors) = parse_and_typecheck_cpp(src);
    assert!(errors.is_empty(), "Type errors: {:?}", errors);
    // Check that the instantiated function was added
    let has_instantiated = program.funcs.iter().any(|f| f.name == "max__int");
    assert!(has_instantiated, "Expected instantiated function 'max__int' in program.funcs");
}

// ============================================================================
// New / Delete Type Check Tests
// ============================================================================

#[test]
fn test_new_delete_typecheck() {
    let src = r#"
int main() {
    int* p = new int(42);
    delete p;
    return 0;
}
"#;
    let (_program, errors) = parse_and_typecheck_cpp(src);
    assert!(errors.is_empty(), "Type errors: {:?}", errors);
}

// ============================================================================
// Range For Tests
// ============================================================================

#[test]
fn test_range_for_array() {
    let src = r#"
int main() {
    int v[3] = {1, 2, 3};
    int sum = 0;
    for (auto x : v) { sum = sum + x; }
    return sum;
}
"#;
    let (_program, errors) = parse_and_typecheck_cpp(src);
    assert!(errors.is_empty(), "Type errors: {:?}", errors);
}

// ============================================================================
// Try / Catch Error Test
// ============================================================================
// NOTE: Parser does not support try/catch syntax yet. TypeChecker handles
// Stmt::Try by reporting E4001, but we cannot test it end-to-end without
// Parser support.

#[test]
fn test_try_stmt_typecheck() {
    use cide_native::compiler::ast::{SourceLoc, Stmt};
    let mut try_stmt = Stmt::Try {
        body: Box::new(Stmt::Block {
            stmts: vec![],
            loc: SourceLoc { line: 1, column: 1 },
        }),
        catches: vec![],
        loc: SourceLoc { line: 1, column: 1 },
    };
    let checker = TypeChecker::default();
    let mut program = ProgramNode::default();
    program.funcs.push(FuncDecl {
        loc: SourceLoc { line: 1, column: 1 },
        return_type: Type::int(),
        name: "main".to_string(),
        params: vec![],
        body: Some(try_stmt.clone()),
        is_static: false,
        is_extern: false,
        source_file: String::new(),
    });
    let (errors, _warnings, _hints) = checker.check(&mut program);
    let has_try_error = errors.iter().any(|e| e.message.contains("异常"));
    assert!(has_try_error, "Expected exception not supported error, got: {:?}", errors);
}
