use cide_native::compiler::ast::{AccessSpec, ClassMember, Stmt};
use cide_native::compiler::lexer::Lexer;
use cide_native::compiler::parser::Parser;

fn parse_cpp(
    src: &str,
) -> (
    Option<cide_native::compiler::ast::ProgramNode>,
    Vec<cide_native::compiler::parser::ParseError>,
) {
    let (tokens, _) = Lexer::with_mode(src, true).tokenize();
    Parser::with_mode(tokens, true).parse()
}

#[test]
fn test_parser_cpp_class_basic() {
    let src = "class Foo { public: int x; }; int main() { return 0; }";
    let (program, errors) = parse_cpp(src);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    assert_eq!(program.classes.len(), 1);
    assert_eq!(program.classes[0].name, "Foo");
    assert!(program.classes[0].base.is_none());
    assert_eq!(program.classes[0].members.len(), 1);
    match &program.classes[0].members[0] {
        ClassMember::Field { name, access, .. } => {
            assert_eq!(name, "x");
            assert_eq!(*access, AccessSpec::Public);
        }
        _ => panic!("Expected Field"),
    }
}

#[test]
fn test_parser_cpp_class_with_private() {
    let src = r#"
class Point {
public:
    int x;
private:
    int y;
};
int main() { return 0; }
"#;
    let (program, errors) = parse_cpp(src);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    assert_eq!(program.classes.len(), 1);
    let class = &program.classes[0];
    assert_eq!(class.members.len(), 2);
    match &class.members[0] {
        ClassMember::Field { name, access, .. } => {
            assert_eq!(name, "x");
            assert_eq!(*access, AccessSpec::Public);
        }
        _ => panic!("Expected Field"),
    }
    match &class.members[1] {
        ClassMember::Field { name, access, .. } => {
            assert_eq!(name, "y");
            assert_eq!(*access, AccessSpec::Private);
        }
        _ => panic!("Expected Field"),
    }
}

#[test]
fn test_parser_cpp_class_method() {
    let src = r#"
class Rect {
public:
    int width;
    int area() { return width * 2; }
};
int main() { return 0; }
"#;
    let (program, errors) = parse_cpp(src);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    let class = &program.classes[0];
    assert_eq!(class.members.len(), 2);
    match &class.members[1] {
        ClassMember::Method { name, access, body, .. } => {
            assert_eq!(name, "area");
            assert_eq!(*access, AccessSpec::Public);
            assert!(body.is_some());
        }
        _ => panic!("Expected Method, got {:?}", class.members[1]),
    }
}

#[test]
fn test_parser_cpp_class_inheritance() {
    let src = r#"
class Base { public: int x; };
class Derived : public Base { public: int y; };
int main() { return 0; }
"#;
    let (program, errors) = parse_cpp(src);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    assert_eq!(program.classes.len(), 2);
    assert_eq!(program.classes[1].name, "Derived");
    assert_eq!(program.classes[1].base.as_ref().unwrap(), "Base");
}

#[test]
fn test_parser_cpp_class_constructor() {
    let src = r#"
class Point {
public:
    int x;
    Point() { x = 0; }
};
int main() { return 0; }
"#;
    let (program, errors) = parse_cpp(src);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    let class = &program.classes[0];
    assert_eq!(class.members.len(), 2);
    match &class.members[1] {
        ClassMember::Constructor { access, body, .. } => {
            assert_eq!(*access, AccessSpec::Public);
            assert!(body.is_some());
        }
        _ => panic!("Expected Constructor, got {:?}", class.members[1]),
    }
}

#[test]
fn test_parser_cpp_template_function() {
    let src = r#"
template <typename T> T max(T a, T b) { if (a > b) return a; return b; }
int main() { return 0; }
"#;
    let (program, errors) = parse_cpp(src);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    assert_eq!(program.templates.len(), 1);
    assert_eq!(program.templates[0].params.len(), 1);
    assert_eq!(program.templates[0].params[0].name, "T");
    match &program.templates[0].decl {
        cide_native::compiler::ast::Templateable::Func(ref f) => {
            assert_eq!(f.name, "max");
            assert_eq!(f.params.len(), 2);
        }
        _ => panic!("Expected Func template"),
    }
}

#[test]
fn test_parser_cpp_template_class() {
    let src = r#"
template <typename T> class Box { public: T value; };
int main() { return 0; }
"#;
    let (program, errors) = parse_cpp(src);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    assert_eq!(program.templates.len(), 1);
    match &program.templates[0].decl {
        cide_native::compiler::ast::Templateable::Class(ref c) => {
            assert_eq!(c.name, "Box");
            assert_eq!(c.members.len(), 1);
        }
        _ => panic!("Expected Class template"),
    }
}

#[test]
fn test_parser_cpp_mode_class_as_identifier_in_c() {
    // In C mode, 'class' should be treated as a normal identifier
    let src = "int class = 10; int main() { return class; }";
    let (tokens, _) = Lexer::new(src).tokenize();
    let (program, errors) = Parser::new(tokens).parse();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    assert!(program.classes.is_empty());
    assert_eq!(program.globals.len(), 1);
    assert_eq!(program.globals[0].name, "class");
}

// ============================================================================
// C++ Expression / Statement Tests (Step 4)
// ============================================================================

use cide_native::compiler::ast::Expr;

#[test]
fn test_parser_cpp_this_expr() {
    let src = r#"
class Foo {
public:
    int x;
    void set(int v) { this->x = v; }
};
int main() { return 0; }
"#;
    let (program, errors) = parse_cpp(src);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    let class = &program.classes[0];
    match &class.members[1] {
        ClassMember::Method { body, .. } => {
            if let Some(Stmt::Block { stmts, .. }) = body {
                if let Stmt::Expr { expr, .. } = &stmts[0] {
                    if let Expr::Assign { left, .. } = expr {
                        if let Expr::Member { object, .. } = left.as_ref() {
                            assert!(matches!(object.as_ref(), Expr::This { .. }), "Expected this->x");
                        } else {
                            panic!("Expected Member assign");
                        }
                    } else {
                        panic!("Expected Assign expr");
                    }
                } else {
                    panic!("Expected Expr stmt");
                }
            } else {
                panic!("Expected Block body");
            }
        }
        _ => panic!("Expected Method"),
    }
}

#[test]
fn test_parser_cpp_member_call() {
    let src = r#"
class Calc {
public:
    int add(int a, int b) { return a + b; }
};
int main() { return 0; }
"#;
    let (program, errors) = parse_cpp(src);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    let class = &program.classes[0];
    match &class.members[0] {
        ClassMember::Method { name, .. } => {
            assert_eq!(name, "add");
        }
        _ => panic!("Expected Method"),
    }
}

#[test]
fn test_parser_cpp_new_delete() {
    let src = r#"
int main() {
    int* p = new int(42);
    delete p;
    int* arr = new int[10];
    delete[] arr;
    return 0;
}
"#;
    let (program, errors) = parse_cpp(src);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    let body = program.funcs[0].body.as_ref().unwrap();
    if let Stmt::Block { stmts, .. } = body {
        // var decl: int* p = new int(42);
        if let Stmt::VarDecl { init: Some(expr), .. } = &stmts[0] {
            assert!(matches!(expr, Expr::New { .. }), "Expected new expr");
        } else {
            panic!("Expected VarDecl with new init");
        }
        // delete p;
        if let Stmt::Expr { expr, .. } = &stmts[1] {
            assert!(matches!(expr, Expr::Delete { is_array: false, .. }), "Expected delete expr");
        } else {
            panic!("Expected delete stmt");
        }
        // delete[] arr;
        if let Stmt::Expr { expr, .. } = &stmts[3] {
            assert!(matches!(expr, Expr::Delete { is_array: true, .. }), "Expected delete[] expr");
        } else {
            panic!("Expected delete[] stmt");
        }
    } else {
        panic!("Expected Block");
    }
}

#[test]
fn test_parser_cpp_range_for() {
    let src = r#"
int main() {
    int v[3] = {1, 2, 3};
    int sum = 0;
    for (auto x : v) { sum = sum + x; }
    return sum;
}
"#;
    let (program, errors) = parse_cpp(src);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    let body = program.funcs[0].body.as_ref().unwrap();
    if let Stmt::Block { stmts, .. } = body {
        if let Stmt::RangeFor { var, var_type, .. } = &stmts[2] {
            assert_eq!(var, "x");
            assert!(matches!(var_type, cide_native::compiler::ast::Type::Auto));
        } else {
            panic!("Expected RangeFor, got {:?}", stmts[2]);
        }
    } else {
        panic!("Expected Block");
    }
}

#[test]
fn test_parser_cpp_lambda() {
    let src = r#"
int main() {
    auto f = [](int x) { return x * 2; };
    return f(5);
}
"#;
    let (program, errors) = parse_cpp(src);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    let program = program.unwrap();
    let body = program.funcs[0].body.as_ref().unwrap();
    if let Stmt::Block { stmts, .. } = body {
        if let Stmt::VarDecl { init: Some(expr), .. } = &stmts[0] {
            assert!(matches!(expr, Expr::Lambda { .. }), "Expected Lambda expr");
        } else {
            panic!("Expected VarDecl with lambda init");
        }
    } else {
        panic!("Expected Block");
    }
}
