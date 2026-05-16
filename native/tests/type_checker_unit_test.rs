use cide_native::compiler::lexer::Lexer;
use cide_native::compiler::parser::Parser;
use cide_native::compiler::type_checker::TypeChecker;

fn type_check(src: &str) -> (Vec<cide_native::compiler::type_checker::TypeError>, Vec<cide_native::compiler::type_checker::TypeError>, Vec<cide_native::compiler::type_checker::TypeError>) {
    let (tokens, _) = Lexer::new(src).tokenize();
    let (maybe_program, parse_errors) = Parser::new(tokens).parse();
    assert!(parse_errors.is_empty(), "Parse errors: {:?}", parse_errors);
    let mut program = maybe_program.unwrap();
    TypeChecker::default().check(&mut program)
}

#[test]
fn test_type_checker_no_errors() {
    let (errors, _warnings, _hints) = type_check("int main() { int x = 10; return 0; }");
    assert!(errors.is_empty(), "Type errors: {:?}", errors);
}

#[test]
fn test_type_checker_undeclared_var() {
    let (errors, _, _) = type_check("int main() { return x; }");
    assert!(!errors.is_empty(), "Expected error for undeclared variable");
    assert!(errors[0].message.contains("未声明"), "Expected 'undeclared' message");
}

#[test]
fn test_type_checker_type_mismatch() {
    let (errors, _, _) = type_check("int main() { int x = \"hello\"; return 0; }");
    assert!(!errors.is_empty(), "Expected error for type mismatch");
}

#[test]
fn test_type_checker_void_return() {
    let (errors, _, _) = type_check("void foo() { } int main() { foo(); return 0; }");
    assert!(errors.is_empty(), "Void function should not require return");
}

#[test]
fn test_type_checker_function_arg_count() {
    let (errors, _, _) = type_check("int foo(int a) { return a; } int main() { foo(); return 0; }");
    assert!(!errors.is_empty(), "Expected error for wrong arg count");
}

#[test]
fn test_type_checker_struct_member_access() {
    let src = r#"
        struct Point { int x; int y; };
        int main() {
            struct Point p;
            p.x = 10;
            return p.x;
        }
    "#;
    let (errors, _, _) = type_check(src);
    assert!(errors.is_empty(), "Type errors: {:?}", errors);
}

#[test]
fn test_type_checker_pointer_arithmetic() {
    let (errors, _, _) = type_check("int main() { int *p; p = p + 1; return 0; }");
    assert!(errors.is_empty(), "Pointer arithmetic should be valid");
}

#[test]
fn test_type_checker_array_index() {
    let (errors, _, _) = type_check("int main() { int arr[5]; arr[0] = 1; return arr[0]; }");
    assert!(errors.is_empty(), "Array index should be valid");
}

#[test]
fn test_type_checker_float_implicit_cast() {
    let (errors, _warnings, _) = type_check("int main() { float f = 10; return 0; }");
    assert!(errors.is_empty(), "int to float implicit cast should be allowed");
    // Should have a hint but not an error
}

#[test]
fn test_type_checker_recursive_call() {
    let src = r#"
        int fact(int n) {
            if (n <= 1) return 1;
            return n * fact(n - 1);
        }
        int main() { return fact(5); }
    "#;
    let (errors, _, _) = type_check(src);
    assert!(errors.is_empty(), "Recursive call should be valid: {:?}", errors);
}

#[test]
fn test_type_checker_forward_decl() {
    let src = r#"
        int foo(int x);
        int main() { return foo(5); }
        int foo(int x) { return x * 2; }
    "#;
    let (errors, _, _) = type_check(src);
    assert!(errors.is_empty(), "Forward declaration should be valid: {:?}", errors);
}

#[test]
fn test_type_checker_duplicate_var() {
    let (errors, _, _) = type_check("int main() { int x; int x; return 0; }");
    assert!(!errors.is_empty(), "Expected error for duplicate variable");
}
