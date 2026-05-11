use cide_native::compiler::lexer::Lexer;
use cide_native::compiler::parser::Parser;
use cide_native::compiler::type_checker::TypeChecker;
use cide_native::compiler::bytecode_gen::BytecodeGen;

#[test]
fn temp_test_nested_struct() {
    let src = r#"
#include <stdio.h>
typedef struct {
    int x;
    int y;
} Point;

typedef struct {
    Point top_left;
    Point bottom_right;
} Rect;

int main() {
    Rect r;
    r.top_left.x = 0;
    r.top_left.y = 0;
    r.bottom_right.x = 10;
    r.bottom_right.y = 20;
    printf("%d %d %d %d\n", r.top_left.x, r.top_left.y, r.bottom_right.x, r.bottom_right.y);
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
    assert!(type_errors.is_empty(), "Expected no type errors");
    
    for s in &program.structs {
        eprintln!("struct {}: {} fields", s.name, s.fields.len());
        for f in &s.fields {
            eprintln!("  {}: {:?} {}", f.name, f.ty.kind, f.ty.name);
        }
    }
    
    let gen = BytecodeGen::new();
    let output = gen.generate(&mut program).unwrap();
    for (name, meta) in &output.func_table {
        eprintln!("func {}: local_count = {}", name, meta.local_count);
    }
}
