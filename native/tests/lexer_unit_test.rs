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
    assert_eq!(tokens[1], (TokenType::Number, "2147483648".to_string()));
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
    let ops = vec!["+", "-", "*", "/", "%", "==", "!=", "<=", ">=", "&&", "||", "<<", ">>", "&", "|", "^", "~"];
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
        TokenType::If, TokenType::Else, TokenType::While, TokenType::For,
        TokenType::Do, TokenType::Return, TokenType::Break, TokenType::Continue,
        TokenType::Switch, TokenType::Case, TokenType::Default, TokenType::Struct,
        TokenType::Typedef, TokenType::Enum, TokenType::Sizeof, TokenType::Const,
        TokenType::Void, TokenType::Float, TokenType::Int, TokenType::Char,
        TokenType::Unsigned, TokenType::Signed, TokenType::Long, TokenType::Short,
    ];
    for (i, exp) in expected.iter().enumerate() {
        assert_eq!(&tokens[i].0, exp, "keyword mismatch at index {}", i);
    }
}

#[test]
fn test_lexer_error_unknown_char() {
    let (_, errors) = Lexer::new("int @ x;").tokenize();
    assert!(!errors.is_empty(), "Expected lexer error for unknown char @");
    assert!(errors[0].message.contains("无法识别") || errors[0].message.contains("未知"), "Expected Chinese error message, got: {}", errors[0].message);
}

#[test]
fn test_lexer_multiline() {
    let src = "int a = 1;\nint b = 2;";
    let (tokens, _) = Lexer::new(src).tokenize();
    // check line numbers of some tokens
    assert_eq!(tokens[0].line, 1); // int
    assert_eq!(tokens[5].line, 2); // int on second line
}
