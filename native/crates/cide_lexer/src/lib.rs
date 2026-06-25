//! Cide 词法分析器。
//!
//! 从 `cide_native::compiler::lexer` 拆分而来，负责 C/C++ 教学子集的 token 化、宏预处理与条件编译。

// TODO(#D08): Lexer 已承载 C/C++ 混合词法，未来应将 C++ 专属词法拆分到 lexer/cpp.rs。
use cide_shared::ErrorCode;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub mod comment;
pub mod keyword;
pub mod number;
pub mod preprocessor;
pub mod string;
pub mod token;

pub use preprocessor::{ConditionalState, MacroDef};
pub use token::{LexerError, Token, TokenType};

pub struct Lexer {
    pub(crate) chars: Vec<char>,
    pub(crate) errors: Vec<LexerError>,
    pub(crate) pos: usize,
    pub(crate) line: i32,
    pub(crate) column: i32,
    pub(crate) macros: HashMap<String, MacroDef>,
    pub(crate) conditional_stack: Vec<ConditionalState>,
    pub(crate) is_cpp_mode: bool,
    /// 源文件所在目录，用于解析 `#include "..."` / `#include <...>` 非标准库路径。
    pub(crate) base_path: Option<PathBuf>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self::with_mode_and_path(source, false, None)
    }

    pub fn with_mode(source: &str, is_cpp_mode: bool) -> Self {
        Self::with_mode_and_path(source, is_cpp_mode, None)
    }

    pub fn with_base_path(source: &str, base_path: Option<PathBuf>) -> Self {
        Self::with_mode_and_path(source, false, base_path)
    }

    pub fn with_mode_and_path(source: &str, is_cpp_mode: bool, base_path: Option<PathBuf>) -> Self {
        let mut macros = HashMap::new();
        // Predefine common stdio macros for fprintf compatibility
        macros.insert(
            "stdout".to_string(),
            MacroDef {
                params: vec![],
                body: vec![Token {
                    ty: TokenType::Number,
                    text: "1".to_string(),
                    line: 0,
                    column: 0,
                }],
            },
        );
        macros.insert(
            "stderr".to_string(),
            MacroDef {
                params: vec![],
                body: vec![Token {
                    ty: TokenType::Number,
                    text: "2".to_string(),
                    line: 0,
                    column: 0,
                }],
            },
        );
        macros.insert(
            "EOF".to_string(),
            MacroDef {
                params: vec![],
                body: vec![Token {
                    ty: TokenType::Number,
                    text: "-1".to_string(),
                    line: 0,
                    column: 0,
                }],
            },
        );
        macros.insert(
            "stdin".to_string(),
            MacroDef {
                params: vec![],
                body: vec![Token {
                    ty: TokenType::Number,
                    text: "0".to_string(),
                    line: 0,
                    column: 0,
                }],
            },
        );
        // stdlib.h macros
        macros.insert(
            "EXIT_SUCCESS".to_string(),
            MacroDef {
                params: vec![],
                body: vec![Token {
                    ty: TokenType::Number,
                    text: "0".to_string(),
                    line: 0,
                    column: 0,
                }],
            },
        );
        macros.insert(
            "EXIT_FAILURE".to_string(),
            MacroDef {
                params: vec![],
                body: vec![Token {
                    ty: TokenType::Number,
                    text: "1".to_string(),
                    line: 0,
                    column: 0,
                }],
            },
        );
        macros.insert(
            "RAND_MAX".to_string(),
            MacroDef {
                params: vec![],
                body: vec![Token {
                    ty: TokenType::Number,
                    text: "32767".to_string(),
                    line: 0,
                    column: 0,
                }],
            },
        );
        // stdio.h macros
        macros.insert(
            "SEEK_SET".to_string(),
            MacroDef {
                params: vec![],
                body: vec![Token {
                    ty: TokenType::Number,
                    text: "0".to_string(),
                    line: 0,
                    column: 0,
                }],
            },
        );
        macros.insert(
            "SEEK_CUR".to_string(),
            MacroDef {
                params: vec![],
                body: vec![Token {
                    ty: TokenType::Number,
                    text: "1".to_string(),
                    line: 0,
                    column: 0,
                }],
            },
        );
        macros.insert(
            "SEEK_END".to_string(),
            MacroDef {
                params: vec![],
                body: vec![Token {
                    ty: TokenType::Number,
                    text: "2".to_string(),
                    line: 0,
                    column: 0,
                }],
            },
        );
        // limits.h macros
        macros.insert(
            "INT_MAX".to_string(),
            MacroDef {
                params: vec![],
                body: vec![Token {
                    ty: TokenType::Number,
                    text: "2147483647".to_string(),
                    line: 0,
                    column: 0,
                }],
            },
        );
        macros.insert(
            "INT_MIN".to_string(),
            MacroDef {
                params: vec![],
                body: vec![Token {
                    ty: TokenType::Number,
                    text: "-2147483648".to_string(),
                    line: 0,
                    column: 0,
                }],
            },
        );
        macros.insert(
            "LONG_MAX".to_string(),
            MacroDef {
                params: vec![],
                body: vec![Token {
                    ty: TokenType::Number,
                    text: "2147483647".to_string(),
                    line: 0,
                    column: 0,
                }],
            },
        );
        macros.insert(
            "LONG_MIN".to_string(),
            MacroDef {
                params: vec![],
                body: vec![Token {
                    ty: TokenType::Number,
                    text: "-2147483648".to_string(),
                    line: 0,
                    column: 0,
                }],
            },
        );
        macros.insert(
            "CHAR_BIT".to_string(),
            MacroDef {
                params: vec![],
                body: vec![Token {
                    ty: TokenType::Number,
                    text: "8".to_string(),
                    line: 0,
                    column: 0,
                }],
            },
        );
        // stdbool.h macros
        macros.insert(
            "true".to_string(),
            MacroDef {
                params: vec![],
                body: vec![Token {
                    ty: TokenType::Number,
                    text: "1".to_string(),
                    line: 0,
                    column: 0,
                }],
            },
        );
        macros.insert(
            "false".to_string(),
            MacroDef {
                params: vec![],
                body: vec![Token {
                    ty: TokenType::Number,
                    text: "0".to_string(),
                    line: 0,
                    column: 0,
                }],
            },
        );
        // stdarg.h 宏：将标准写法映射到 Cide 内部 host func 调用
        macros.insert(
            "va_start".to_string(),
            MacroDef {
                params: vec!["ap".to_string(), "last".to_string()],
                body: vec![
                    Token {
                        ty: TokenType::Identifier,
                        text: "__cide_va_start".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::LParen,
                        text: "(".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::Ampersand,
                        text: "&".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::LParen,
                        text: "(".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::Identifier,
                        text: "ap".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::RParen,
                        text: ")".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::Comma,
                        text: ",".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::Ampersand,
                        text: "&".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::LParen,
                        text: "(".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::Identifier,
                        text: "last".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::RParen,
                        text: ")".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::Comma,
                        text: ",".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::Sizeof,
                        text: "sizeof".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::LParen,
                        text: "(".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::Identifier,
                        text: "last".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::RParen,
                        text: ")".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::RParen,
                        text: ")".to_string(),
                        line: 0,
                        column: 0,
                    },
                ],
            },
        );
        macros.insert(
            "va_arg".to_string(),
            MacroDef {
                params: vec!["ap".to_string(), "type".to_string()],
                body: vec![
                    Token {
                        ty: TokenType::Star,
                        text: "*".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::LParen,
                        text: "(".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::Identifier,
                        text: "type".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::Star,
                        text: "*".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::RParen,
                        text: ")".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::Identifier,
                        text: "__cide_va_arg".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::LParen,
                        text: "(".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::Ampersand,
                        text: "&".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::LParen,
                        text: "(".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::Identifier,
                        text: "ap".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::RParen,
                        text: ")".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::Comma,
                        text: ",".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::Sizeof,
                        text: "sizeof".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::LParen,
                        text: "(".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::Identifier,
                        text: "type".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::RParen,
                        text: ")".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::RParen,
                        text: ")".to_string(),
                        line: 0,
                        column: 0,
                    },
                ],
            },
        );
        macros.insert(
            "va_end".to_string(),
            MacroDef {
                params: vec!["ap".to_string()],
                body: vec![
                    Token {
                        ty: TokenType::Identifier,
                        text: "__cide_va_end".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::LParen,
                        text: "(".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::Ampersand,
                        text: "&".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::LParen,
                        text: "(".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::Identifier,
                        text: "ap".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::RParen,
                        text: ")".to_string(),
                        line: 0,
                        column: 0,
                    },
                    Token {
                        ty: TokenType::RParen,
                        text: ")".to_string(),
                        line: 0,
                        column: 0,
                    },
                ],
            },
        );
        Self {
            chars: source.chars().collect(),
            errors: Vec::new(),
            pos: 0,
            line: 1,
            column: 1,
            macros,
            conditional_stack: Vec::new(),
            is_cpp_mode,
            base_path,
        }
    }

    pub fn tokenize(mut self) -> (Vec<Token>, Vec<LexerError>) {
        let mut tokens = Vec::new();
        loop {
            let t = self.next_token();
            if t.ty == TokenType::Eof {
                tokens.push(t);
                break;
            }
            tokens.push(t);
        }
        if !self.conditional_stack.is_empty() {
            self.errors.push(LexerError {
                message: "未闭合的 #ifdef / #ifndef 块".to_string(),
                line: self.line,
                column: self.column,
                code: ErrorCode::E1013_UnclosedConditional as i32,
            });
        }
        let expanded = self.expand_macros(tokens);
        (expanded, self.errors)
    }

    pub fn into_errors(self) -> Vec<LexerError> {
        self.errors
    }

    fn next_token(&mut self) -> Token {
        // 条件编译跳过模式：跳过所有不活跃的代码块
        while self.is_skipping() {
            match self.skip_inactive_line() {
                None => return self.make_token(TokenType::Eof, ""),
                Some(false) => break,
                Some(true) => continue,
            }
        }

        self.skip_whitespace();

        if self.pos >= self.chars.len() {
            return self.make_token(TokenType::Eof, "");
        }

        let c = self.peek(0);

        if c.is_ascii_alphabetic() || c == '_' {
            return self.identifier_or_keyword();
        }

        if c.is_ascii_digit() {
            return self.number();
        }

        if c == '"' {
            return self.string_literal();
        }

        if c == '\'' {
            return self.char_literal();
        }

        if c == '/' && self.peek(1) == '/' {
            self.skip_comment();
            return self.next_token();
        }

        if c == '/' && self.peek(1) == '*' {
            self.skip_block_comment();
            return self.next_token();
        }

        if c == '#' {
            self.skip_preprocessor_directive();
            return self.next_token();
        }

        match c {
            '+' => {
                if self.match_char('+') {
                    return self.make_token(TokenType::Increment, "++");
                }
                if self.match_char('=') {
                    return self.make_token(TokenType::PlusAssign, "+=");
                }
                self.advance();
                self.make_token(TokenType::Plus, "+")
            }
            '-' => {
                if self.match_char('>') {
                    if self.peek(0) == '*' {
                        self.advance();
                        return self.make_token(TokenType::ArrowStar, "->*");
                    }
                    return self.make_token(TokenType::Arrow, "->");
                }
                if self.match_char('-') {
                    return self.make_token(TokenType::Decrement, "--");
                }
                if self.match_char('=') {
                    return self.make_token(TokenType::MinusAssign, "-=");
                }
                self.advance();
                self.make_token(TokenType::Minus, "-")
            }
            '*' => {
                if self.match_char('=') {
                    return self.make_token(TokenType::StarAssign, "*=");
                }
                self.advance();
                self.make_token(TokenType::Star, "*")
            }
            '/' => {
                if self.match_char('=') {
                    return self.make_token(TokenType::SlashAssign, "/=");
                }
                self.advance();
                self.make_token(TokenType::Slash, "/")
            }
            '%' => {
                if self.match_char('=') {
                    return self.make_token(TokenType::PercentAssign, "%=");
                }
                self.advance();
                self.make_token(TokenType::Percent, "%")
            }
            '=' => {
                if self.match_char('=') {
                    return self.make_token(TokenType::Eq, "==");
                }
                self.advance();
                self.make_token(TokenType::Assign, "=")
            }
            '!' => {
                if self.match_char('=') {
                    return self.make_token(TokenType::Ne, "!=");
                }
                self.advance();
                self.make_token(TokenType::Not, "!")
            }

            '&' => {
                if self.match_char('&') {
                    return self.make_token(TokenType::AndAnd, "&&");
                }
                if self.match_char('=') {
                    return self.make_token(TokenType::AndAssign, "&=");
                }
                self.advance();
                self.make_token(TokenType::Ampersand, "&")
            }
            '|' => {
                if self.match_char('|') {
                    return self.make_token(TokenType::OrOr, "||");
                }
                if self.match_char('=') {
                    return self.make_token(TokenType::OrAssign, "|=");
                }
                self.advance();
                self.make_token(TokenType::BitOr, "|")
            }
            '^' => {
                if self.match_char('=') {
                    return self.make_token(TokenType::XorAssign, "^=");
                }
                self.advance();
                self.make_token(TokenType::BitXor, "^")
            }
            '~' => {
                self.advance();
                self.make_token(TokenType::BitNot, "~")
            }
            '<' => {
                if self.match_char('<') {
                    if self.peek(0) == '=' {
                        self.advance();
                        return self.make_token(TokenType::ShlAssign, "<<=");
                    }
                    return self.make_token(TokenType::Shl, "<<");
                }
                if self.match_char('=') {
                    return self.make_token(TokenType::Le, "<=");
                }
                self.advance();
                self.make_token(TokenType::Lt, "<")
            }
            '>' => {
                if self.match_char('>') {
                    if self.peek(0) == '=' {
                        self.advance();
                        return self.make_token(TokenType::ShrAssign, ">>=");
                    }
                    return self.make_token(TokenType::Shr, ">>");
                }
                if self.match_char('=') {
                    return self.make_token(TokenType::Ge, ">=");
                }
                self.advance();
                self.make_token(TokenType::Gt, ">")
            }
            ';' => {
                self.advance();
                self.make_token(TokenType::Semicolon, ";")
            }
            ',' => {
                self.advance();
                self.make_token(TokenType::Comma, ",")
            }
            '(' => {
                self.advance();
                self.make_token(TokenType::LParen, "(")
            }
            ')' => {
                self.advance();
                self.make_token(TokenType::RParen, ")")
            }
            '{' => {
                self.advance();
                self.make_token(TokenType::LBrace, "{")
            }
            '}' => {
                self.advance();
                self.make_token(TokenType::RBrace, "}")
            }
            '[' => {
                self.advance();
                self.make_token(TokenType::LBracket, "[")
            }
            ']' => {
                self.advance();
                self.make_token(TokenType::RBracket, "]")
            }
            '.' => {
                if self.match_char('*') {
                    return self.make_token(TokenType::DotStar, ".*");
                }
                if self.peek(1) == '.' && self.peek(2) == '.' {
                    self.advance();
                    self.advance();
                    self.advance();
                    return self.make_token(TokenType::Ellipsis, "...");
                }
                self.advance();
                self.make_token(TokenType::Dot, ".")
            }
            ':' => {
                if self.match_char(':') {
                    return self.make_token(TokenType::ColonColon, "::");
                }
                self.advance();
                self.make_token(TokenType::Colon, ":")
            }
            '?' => {
                self.advance();
                self.make_token(TokenType::Question, "?")
            }
            _ => {
                self.advance();
                self.errors.push(LexerError {
                    message: format!("无法识别的字符: '{}'", c),
                    line: self.line,
                    column: self.column,
                    code: ErrorCode::E1001_UnknownChar as i32,
                });
                self.make_token(TokenType::Unknown, &c.to_string())
            }
        }
    }

    fn identifier_or_keyword(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.chars.len() {
            let c = self.peek(0);
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        let ty = keyword::keyword_type(&text)
            .or_else(|| {
                if self.is_cpp_mode {
                    keyword::cpp_keyword_type(&text)
                } else {
                    None
                }
            })
            .unwrap_or(TokenType::Identifier);
        self.make_token(ty, &text)
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() {
            let c = self.peek(0);
            if c.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn expand_macros(&self, tokens: Vec<Token>) -> Vec<Token> {
        self.expand_macros_inner(&tokens, &mut HashSet::new())
    }

    fn expand_macros_inner(&self, tokens: &[Token], expanding: &mut HashSet<String>) -> Vec<Token> {
        let mut result = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            let tok = &tokens[i];
            if tok.ty == TokenType::Identifier {
                if let Some(mdef) = self.macros.get(&tok.text) {
                    if expanding.contains(&tok.text) {
                        result.push(tok.clone());
                        i += 1;
                        continue;
                    }
                    if mdef.params.is_empty() {
                        // 对象式宏
                        expanding.insert(tok.text.clone());
                        let expanded = self.expand_macros_inner(&mdef.body, expanding);
                        expanding.remove(&tok.text);
                        for mut mt in expanded {
                            mt.line = tok.line;
                            mt.column = tok.column;
                            result.push(mt);
                        }
                        i += 1;
                        continue;
                    } else {
                        // 参数化宏：检查下一个 token 是否是 (
                        if i + 1 < tokens.len() && tokens[i + 1].ty == TokenType::LParen {
                            let mut args: Vec<Vec<Token>> = Vec::new();
                            let mut current_arg: Vec<Token> = Vec::new();
                            let mut depth = 1;
                            let mut j = i + 2; // skip LParen
                            while j < tokens.len() && depth > 0 {
                                match tokens[j].ty {
                                    TokenType::LParen => {
                                        depth += 1;
                                        current_arg.push(tokens[j].clone());
                                    }
                                    TokenType::RParen => {
                                        depth -= 1;
                                        if depth == 0 {
                                            break;
                                        }
                                        current_arg.push(tokens[j].clone());
                                    }
                                    TokenType::Comma if depth == 1 => {
                                        args.push(current_arg);
                                        current_arg = Vec::new();
                                    }
                                    _ => current_arg.push(tokens[j].clone()),
                                }
                                j += 1;
                            }
                            args.push(current_arg);

                            if args.len() != mdef.params.len() {
                                // 参数数量不匹配，不展开，保留原 token
                                result.push(tok.clone());
                                i += 1;
                                continue;
                            }

                            // 替换 body 中的参数
                            expanding.insert(tok.text.clone());
                            let mut substituted = Vec::new();
                            for bt in &mdef.body {
                                if bt.ty == TokenType::Identifier {
                                    if let Some(param_idx) = mdef.params.iter().position(|p| p == &bt.text) {
                                        substituted.extend(args[param_idx].iter().cloned());
                                        continue;
                                    }
                                }
                                substituted.push(bt.clone());
                            }

                            // H01: 参数化宏体为大括号块且调用后紧跟分号时，
                            // 动态包装为 do { ... } while(0)，使宏调用在 if/else 等语句中可正确解析。
                            let body_is_brace_block =
                                mdef.body.first().map(|t| t.ty == TokenType::LBrace).unwrap_or(false)
                                    && mdef.body.last().map(|t| t.ty == TokenType::RBrace).unwrap_or(false);
                            let followed_by_semicolon =
                                j + 1 < tokens.len() && tokens[j + 1].ty == TokenType::Semicolon;
                            if body_is_brace_block && followed_by_semicolon {
                                substituted.insert(
                                    0,
                                    Token {
                                        ty: TokenType::Do,
                                        text: "do".to_string(),
                                        line: tok.line,
                                        column: tok.column,
                                    },
                                );
                                substituted.push(Token {
                                    ty: TokenType::While,
                                    text: "while".to_string(),
                                    line: tok.line,
                                    column: tok.column,
                                });
                                substituted.push(Token {
                                    ty: TokenType::LParen,
                                    text: "(".to_string(),
                                    line: tok.line,
                                    column: tok.column,
                                });
                                substituted.push(Token {
                                    ty: TokenType::Number,
                                    text: "0".to_string(),
                                    line: tok.line,
                                    column: tok.column,
                                });
                                substituted.push(Token {
                                    ty: TokenType::RParen,
                                    text: ")".to_string(),
                                    line: tok.line,
                                    column: tok.column,
                                });
                            }

                            let expanded = self.expand_macros_inner(&substituted, expanding);
                            expanding.remove(&tok.text);
                            for mut mt in expanded {
                                mt.line = tok.line;
                                mt.column = tok.column;
                                result.push(mt);
                            }
                            i = j + 1;
                            continue;
                        }
                    }
                }
            }
            result.push(tok.clone());
            i += 1;
        }
        result
    }

    pub(crate) fn peek(&self, offset: usize) -> char {
        if self.pos >= self.chars.len() {
            '\0'
        } else {
            self.chars.get(self.pos + offset).copied().unwrap_or('\0')
        }
    }

    pub(crate) fn advance(&mut self) -> char {
        if self.pos >= self.chars.len() {
            return '\0';
        }
        let c = self.chars.get(self.pos).copied().unwrap_or('\0');
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        c
    }

    pub(crate) fn match_char(&mut self, expected: char) -> bool {
        if self.peek(1) != expected {
            return false;
        }
        self.advance();
        self.advance();
        true
    }

    pub(crate) fn make_token(&self, ty: TokenType, text: &str) -> Token {
        Token {
            ty,
            text: text.to_string(),
            line: self.line,
            column: self.column - text.len() as i32,
        }
    }
}
