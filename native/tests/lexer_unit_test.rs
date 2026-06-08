use cide_native::compiler::lexer::{Lexer, TokenType};

fn tokenize(src: &str) -> Vec<(TokenType, String)> {
    let (tokens, errors) = Lexer::new(src).tokenize();
    assert!(errors.is_empty(), "Lexer errors: {:?}", errors);
    tokens.into_iter().map(|t| (t.ty, t.text)).collect()
}

#[test]
fn test_lexer_basic_tokens() {
    let tokens = tokenize("int main() { return 0; }");
    assert_eq!(tokens[0], (TokenType::Int, "int".to_string()));
    assert_eq!(tokens[1], (TokenType::Identifier, "main".to_string()));
    assert_eq!(tokens[2], (TokenType::LParen, "(".to_string()));
    assert_eq!(tokens[3], (TokenType::RParen, ")".to_string()));
    assert_eq!(tokens[4], (TokenType::LBrace, "{".to_string()));
    assert_eq!(tokens[5], (TokenType::Return, "return".to_string()));
    assert_eq!(tokens[6], (TokenType::Number, "0".to_string()));
    assert_eq!(tokens[7], (TokenType::Semicolon, ";".to_string()));
    assert_eq!(tokens[8], (TokenType::RBrace, "}".to_string()));
    assert_eq!(tokens[9], (TokenType::Eof, "".to_string()));
}

#[test]
fn test_lexer_hex_literal() {
    let tokens = tokenize("0xFF 0x80000000 0x0");
    assert_eq!(tokens[0], (TokenType::Number, "255".to_string()));
    assert_eq!(tokens[1], (TokenType::UnsignedLiteral, "2147483648".to_string()));
    assert_eq!(tokens[2], (TokenType::Number, "0".to_string()));
}

#[test]
fn test_lexer_string_literal() {
    let tokens = tokenize("\"hello\" \"world\\n\"");
    assert_eq!(tokens[0], (TokenType::String, "hello".to_string()));
    assert_eq!(tokens[1], (TokenType::String, "world\n".to_string()));
}

#[test]
fn test_lexer_char_literal() {
    let tokens = tokenize("'a' '\\n' '0'");
    assert_eq!(tokens[0], (TokenType::CharLiteral, "97".to_string()));
    assert_eq!(tokens[1], (TokenType::CharLiteral, "10".to_string()));
    assert_eq!(tokens[2], (TokenType::CharLiteral, "48".to_string()));
}

#[test]
fn test_lexer_operators() {
    let tokens = tokenize("+ - * / % == != <= >= && || << >> & | ^ ~");
    let ops = vec![
        "+", "-", "*", "/", "%", "==", "!=", "<=", ">=", "&&", "||", "<<", ">>", "&", "|", "^", "~",
    ];
    for (i, op) in ops.iter().enumerate() {
        assert_eq!(tokens[i].1, op.to_string(), "operator mismatch at index {}", i);
    }
}

#[test]
fn test_lexer_comments() {
    let tokens = tokenize("// line comment\nint x;\n/* block\ncomment */ float y;");
    // comments are skipped, only real tokens remain
    assert_eq!(tokens[0], (TokenType::Int, "int".to_string()));
    assert_eq!(tokens[1], (TokenType::Identifier, "x".to_string()));
    assert_eq!(tokens[2], (TokenType::Semicolon, ";".to_string()));
    assert_eq!(tokens[3], (TokenType::Float, "float".to_string()));
    assert_eq!(tokens[4], (TokenType::Identifier, "y".to_string()));
    assert_eq!(tokens[5], (TokenType::Semicolon, ";".to_string()));
}

#[test]
fn test_lexer_define_macro() {
    let src = "#define MAX 100\nint x = MAX;";
    let tokens = tokenize(src);
    // After macro expansion, MAX should be tokenized as Number 100
    let nums: Vec<_> = tokens.iter().filter(|(t, _)| *t == TokenType::Number).collect();
    assert_eq!(nums.len(), 1);
    assert_eq!(nums[0].1, "100");
}

#[test]
fn test_lexer_keywords() {
    let src = "if else while for do return break continue switch case default struct typedef enum sizeof const void float int char unsigned signed long short";
    let tokens = tokenize(src);
    let expected = vec![
        TokenType::If,
        TokenType::Else,
        TokenType::While,
        TokenType::For,
        TokenType::Do,
        TokenType::Return,
        TokenType::Break,
        TokenType::Continue,
        TokenType::Switch,
        TokenType::Case,
        TokenType::Default,
        TokenType::Struct,
        TokenType::Typedef,
        TokenType::Enum,
        TokenType::Sizeof,
        TokenType::Const,
        TokenType::Void,
        TokenType::Float,
        TokenType::Int,
        TokenType::Char,
        TokenType::Unsigned,
        TokenType::Signed,
        TokenType::Long,
        TokenType::Short,
    ];
    for (i, exp) in expected.iter().enumerate() {
        assert_eq!(&tokens[i].0, exp, "keyword mismatch at index {}", i);
    }
}

#[test]
fn test_lexer_error_unknown_char() {
    let (_, errors) = Lexer::new("int @ x;").tokenize();
    assert!(!errors.is_empty(), "Expected lexer error for unknown char @");
    assert!(
        errors[0].message.contains("无法识别") || errors[0].message.contains("未知"),
        "Expected Chinese error message, got: {}",
        errors[0].message
    );
}

#[test]
fn test_lexer_multiline() {
    let src = "int a = 1;\nint b = 2;";
    let (tokens, _) = Lexer::new(src).tokenize();
    // check line numbers of some tokens
    assert_eq!(tokens[0].line, 1); // int
    assert_eq!(tokens[5].line, 2); // int on second line
}

// ============================================================================
// 条件编译（#ifdef / #ifndef / #else / #endif）单元测试
// ============================================================================

#[test]
fn test_lexer_ifdef_defined() {
    let src = "#define DEBUG 1\n#ifdef DEBUG\nint x = 1;\n#endif";
    let tokens = tokenize(src);
    // x 应该被编译
    let ids: Vec<_> = tokens.iter().filter(|(t, _)| *t == TokenType::Identifier).collect();
    assert!(
        ids.iter().any(|(_, text)| text == "x"),
        "x should be tokenized when DEBUG is defined"
    );
}

#[test]
fn test_lexer_ifdef_undefined() {
    let src = "#ifdef UNDEFINED\nint y = 2;\n#endif\nint z = 3;";
    let tokens = tokenize(src);
    // y 应该被跳过，z 应该被编译
    let ids: Vec<_> = tokens.iter().filter(|(t, _)| *t == TokenType::Identifier).collect();
    assert!(
        !ids.iter().any(|(_, text)| text == "y"),
        "y should be skipped when UNDEFINED is not defined"
    );
    assert!(ids.iter().any(|(_, text)| text == "z"), "z should be tokenized after #endif");
}

#[test]
fn test_lexer_ifndef() {
    let src = "#ifndef FLAG\nint a = 1;\n#endif";
    let tokens = tokenize(src);
    // FLAG 未定义，所以 a 应该被编译
    let ids: Vec<_> = tokens.iter().filter(|(t, _)| *t == TokenType::Identifier).collect();
    assert!(
        ids.iter().any(|(_, text)| text == "a"),
        "a should be tokenized when FLAG is not defined"
    );
}

#[test]
fn test_lexer_ifndef_defined() {
    let src = "#define FLAG 1\n#ifndef FLAG\nint b = 2;\n#endif\nint c = 3;";
    let tokens = tokenize(src);
    // FLAG 已定义，所以 b 应该被跳过，c 应该被编译
    let ids: Vec<_> = tokens.iter().filter(|(t, _)| *t == TokenType::Identifier).collect();
    assert!(
        !ids.iter().any(|(_, text)| text == "b"),
        "b should be skipped when FLAG is defined"
    );
    assert!(ids.iter().any(|(_, text)| text == "c"), "c should be tokenized after #endif");
}

#[test]
fn test_lexer_conditional_else() {
    let src = "#define MODE 1\n#ifdef MODE\nint a = 1;\n#else\nint a = 2;\n#endif";
    let tokens = tokenize(src);
    // MODE 定义了，所以 a=1 应该被编译，a=2 应该被跳过
    let nums: Vec<_> = tokens.iter().filter(|(t, _)| *t == TokenType::Number).collect();
    assert!(nums.iter().any(|(_, text)| text == "1"), "1 should appear when MODE is defined");
    assert!(!nums.iter().any(|(_, text)| text == "2"), "2 should be skipped in #else block");
}

#[test]
fn test_lexer_conditional_else_undefined() {
    let src = "#ifdef MODE\nint a = 1;\n#else\nint a = 2;\n#endif";
    let tokens = tokenize(src);
    // MODE 未定义，所以 a=1 应该被跳过，a=2 应该被编译
    let nums: Vec<_> = tokens.iter().filter(|(t, _)| *t == TokenType::Number).collect();
    assert!(
        !nums.iter().any(|(_, text)| text == "1"),
        "1 should be skipped when MODE is not defined"
    );
    assert!(nums.iter().any(|(_, text)| text == "2"), "2 should appear in #else block");
}

#[test]
fn test_lexer_nested_conditional() {
    let src = r#"
#define OUTER
#ifdef OUTER
  #define INNER
  #ifdef INNER
    int x = 1;
  #else
    int x = 2;
  #endif
#else
  int x = 3;
#endif
"#;
    let tokens = tokenize(src);
    // OUTER 和 INNER 都定义了，所以 x=1 应该被编译，x=2 和 x=3 应该被跳过
    let nums: Vec<_> = tokens.iter().filter(|(t, _)| *t == TokenType::Number).collect();
    assert!(nums.iter().any(|(_, text)| text == "1"), "1 should appear");
    assert!(!nums.iter().any(|(_, text)| text == "2"), "2 should be skipped");
    assert!(!nums.iter().any(|(_, text)| text == "3"), "3 should be skipped");
}

#[test]
fn test_lexer_nested_skip_inner() {
    let src = r#"
#ifdef OUTER
  #ifdef INNER
    int x = 1;
  #endif
#endif
int y = 2;
"#;
    let tokens = tokenize(src);
    // OUTER 未定义，所以所有内部代码都被跳过，只有 y=2 被编译
    let ids: Vec<_> = tokens.iter().filter(|(t, _)| *t == TokenType::Identifier).collect();
    assert!(!ids.iter().any(|(_, text)| text == "x"), "x should be skipped");
    assert!(ids.iter().any(|(_, text)| text == "y"), "y should be tokenized");
}

#[test]
fn test_lexer_header_guard_pattern() {
    let src = r#"
#ifndef MYHEADER_H
#define MYHEADER_H
int global = 42;
#endif
"#;
    let tokens = tokenize(src);
    // 第一次遇到 MYHEADER_H 未定义，所以内容被编译；然后 MYHEADER_H 被定义
    let ids: Vec<_> = tokens.iter().filter(|(t, _)| *t == TokenType::Identifier).collect();
    assert!(ids.iter().any(|(_, text)| text == "global"), "global should be tokenized");
}

#[test]
fn test_lexer_conditional_with_comments() {
    let src = r#"
#ifdef FLAG
/* block comment */
int a = 1;
#else
// line comment
int a = 2;
#endif
"#;
    let tokens = tokenize(src);
    // FLAG 未定义，所以 #else 块生效，a=2 被编译
    let nums: Vec<_> = tokens.iter().filter(|(t, _)| *t == TokenType::Number).collect();
    assert!(!nums.iter().any(|(_, text)| text == "1"), "1 should be skipped");
    assert!(nums.iter().any(|(_, text)| text == "2"), "2 should appear");
}

#[test]
fn test_lexer_conditional_error_unclosed() {
    let src = "#ifdef FLAG\nint x = 1;";
    let (_, errors) = Lexer::new(src).tokenize();
    assert!(!errors.is_empty(), "Expected error for unclosed #ifdef");
    assert!(errors.iter().any(|e| e.code == 1013), "Expected E1013_UnclosedConditional");
}

#[test]
fn test_lexer_conditional_error_unmatched_endif() {
    let src = "int x = 1;\n#endif";
    let (_, errors) = Lexer::new(src).tokenize();
    assert!(!errors.is_empty(), "Expected error for unmatched #endif");
    assert!(errors.iter().any(|e| e.code == 1011), "Expected E1011_UnmatchedConditional");
}

#[test]
fn test_lexer_conditional_error_duplicate_else() {
    let src = "#ifdef FLAG\nint a = 1;\n#else\nint a = 2;\n#else\nint a = 3;\n#endif";
    let (_, errors) = Lexer::new(src).tokenize();
    assert!(!errors.is_empty(), "Expected error for duplicate #else");
    assert!(errors.iter().any(|e| e.code == 1012), "Expected E1012_DuplicateElse");
}

#[test]
fn test_lexer_define_inside_skipped_block() {
    let src = r#"
#ifdef UNDEFINED
#define SECRET 999
#endif
int x = SECRET;
"#;
    let (_, errors) = Lexer::new(src).tokenize();
    // SECRET 在跳过块内定义，不应生效；之后使用 SECRET 应该报错（未定义标识符）
    // 注意：lexer 层只会保留 SECRET 作为 Identifier，不会展开宏
    // 这里主要验证 SECRET 没有被注册为宏
    // 由于 SECRET 未定义，编译时会在 parser/typeck 报错
    // 本测试只验证 lexer 没有产生错误（条件编译本身正确处理）
    assert!(
        errors.is_empty() || !errors.iter().any(|e| e.code == 1011 || e.code == 1012 || e.code == 1013),
        "Should not produce conditional compilation errors"
    );
}

#[test]
fn test_lexer_cpp_keywords() {
    let src = "class public private protected this using namespace virtual override friend template typename static_cast const_cast reinterpret_cast new delete nullptr";
    let (tokens, errors) = Lexer::with_mode(src, true).tokenize();
    assert!(errors.is_empty(), "Lexer errors: {:?}", errors);
    let tokens: Vec<_> = tokens.into_iter().map(|t| (t.ty, t.text)).collect();
    let expected = vec![
        TokenType::Class,
        TokenType::Public,
        TokenType::Private,
        TokenType::Protected,
        TokenType::This,
        TokenType::Using,
        TokenType::Namespace,
        TokenType::Virtual,
        TokenType::Override,
        TokenType::Friend,
        TokenType::Template,
        TokenType::Typename,
        TokenType::StaticCast,
        TokenType::ConstCast,
        TokenType::ReinterpretCast,
        TokenType::New,
        TokenType::Delete,
        TokenType::Null,
    ];
    for (i, exp) in expected.iter().enumerate() {
        assert_eq!(&tokens[i].0, exp, "C++ keyword mismatch at index {}: got {:?}", i, tokens[i].1);
    }
}

#[test]
fn test_lexer_cpp_operators() {
    let tokens = tokenize(":: ->* .*");
    assert_eq!(tokens[0], (TokenType::ColonColon, "::".to_string()));
    assert_eq!(tokens[1], (TokenType::ArrowStar, "->*".to_string()));
    assert_eq!(tokens[2], (TokenType::DotStar, ".*".to_string()));
}

#[test]
fn test_lexer_cpp_line_comment() {
    // C++ // comment should be skipped like C-style block comments
    let tokens = tokenize("// C++ line comment\nint x;");
    assert_eq!(tokens[0], (TokenType::Int, "int".to_string()));
    assert_eq!(tokens[1], (TokenType::Identifier, "x".to_string()));
    assert_eq!(tokens[2], (TokenType::Semicolon, ";".to_string()));
}
