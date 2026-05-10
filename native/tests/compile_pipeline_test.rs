use cide_native::compiler::lexer::Lexer;
use cide_native::compiler::parser::Parser;
use cide_native::compiler::type_checker::TypeChecker;
use cide_native::compiler::bytecode_gen::BytecodeGen;

fn compile_source(source: &str) -> Result<cide_native::compiler::bytecode_gen::CompileOutput, String> {
    let (tokens, lex_errors) = Lexer::new(source.to_string()).tokenize();
    if !lex_errors.is_empty() {
        return Err(format!("Lexer errors: {:?}", lex_errors));
    }

    let (maybe_program, parse_errors) = Parser::new(tokens).parse();
    if !parse_errors.is_empty() {
        return Err(format!("Parse errors: {:?}", parse_errors));
    }

    let mut program = maybe_program.ok_or("Parser returned None")?;

    let (type_errors, _warnings) = TypeChecker::new().check(&mut program);
    if !type_errors.is_empty() {
        return Err(format!("Type errors: {:?}", type_errors));
    }

    let gen = BytecodeGen::new();
    gen.generate(&mut program)
        .map_err(|e| format!("BytecodeGen errors: {:?}", e))
}

#[test]
fn test_compile_empty_main() {
    let src = r#"
int main() {
    return 0;
}
"#;
    let result = compile_source(src);
    assert!(result.is_ok(), "{:?}", result.err());
    let output = result.unwrap();
    assert!(!output.code.is_empty());
    assert!(output.func_index.contains_key("main"));
}

#[test]
fn test_compile_hello_world() {
    let src = r#"
#include <stdio.h>
int main() {
    printf("Hello, World!\n");
    return 0;
}
"#;
    let result = compile_source(src);
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn test_compile_with_vars_and_arithmetic() {
    let src = r#"
int main() {
    int a = 10;
    int b = 20;
    int c = a + b * 2;
    return c;
}
"#;
    let result = compile_source(src);
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn test_compile_multidim_array() {
    let src = r#"
int main() {
    int arr[3][3];
    arr[1][2] = 5;
    return arr[1][2];
}
"#;
    let result = compile_source(src);
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn test_compile_struct() {
    let src = r#"
struct Point {
    int x;
    int y;
};
int main() {
    struct Point p;
    p.x = 3;
    p.y = 4;
    return p.x + p.y;
}
"#;
    let result = compile_source(src);
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn test_compile_if_while_for() {
    let src = r#"
int main() {
    int sum = 0;
    for (int i = 0; i < 5; i = i + 1) {
        sum = sum + i;
    }
    int j = 0;
    while (j < 3) {
        sum = sum + j;
        j = j + 1;
    }
    if (sum > 0) {
        return sum;
    } else {
        return 0;
    }
}
"#;
    let result = compile_source(src);
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn test_compile_malloc_free() {
    let src = r#"
#include <stdlib.h>
int main() {
    int *p = malloc(sizeof(int));
    *p = 42;
    int val = *p;
    free(p);
    return val;
}
"#;
    let result = compile_source(src);
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn test_compile_switch() {
    let src = r#"
int main() {
    int x = 2;
    switch (x) {
        case 1: return 10;
        case 2: return 20;
        default: return 0;
    }
}
"#;
    let result = compile_source(src);
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn test_compile_scanf() {
    let src = r#"
#include <stdio.h>
int main() {
    int a;
    scanf("%d", &a);
    return a;
}
"#;
    let result = compile_source(src);
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn test_compile_define_macro() {
    let src = r#"
#define N 100
int main() {
    int arr[N];
    return N;
}
"#;
    let result = compile_source(src);
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn test_type_error_detected() {
    let src = r#"
int main() {
    int x = "hello";
    return 0;
}
"#;
    let (tokens, _lex_errors) = Lexer::new(src.to_string()).tokenize();
    let (maybe_program, _parse_errors) = Parser::new(tokens).parse();
    let mut program = maybe_program.unwrap();
    let (type_errors, _warnings) = TypeChecker::new().check(&mut program);
    assert!(!type_errors.is_empty(), "Expected type error for string assigned to int");
}

#[test]
fn test_undeclared_var_detected() {
    let src = r#"
int main() {
    return unknown_var;
}
"#;
    let (tokens, _lex_errors) = Lexer::new(src.to_string()).tokenize();
    let (maybe_program, _parse_errors) = Parser::new(tokens).parse();
    let mut program = maybe_program.unwrap();
    let (type_errors, _warnings) = TypeChecker::new().check(&mut program);
    assert!(!type_errors.is_empty(), "Expected type error for undeclared variable");
}
