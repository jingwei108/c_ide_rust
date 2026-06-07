use crate::diagnostics::error_codes::ErrorCode;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Int, Void, Char, If, Else, While, Do, For, Return, Break, Continue,
    Struct, Union, Sizeof, Switch, Case, Default, Typedef, Enum, Unsigned, Long, Short, Signed, Const, Extern, Float, Double,
    Null,
    Identifier, Number, UnsignedLiteral, FloatLiteral, LongLiteral, CharLiteral, String,
    Plus, Minus, Star, Slash, Percent,
    Eq, Ne, Lt, Le, Gt, Ge,
    AndAnd, OrOr, Not,
    Assign, PlusAssign, MinusAssign, StarAssign, SlashAssign, PercentAssign,
    AndAssign, OrAssign, XorAssign, ShlAssign, ShrAssign,
    Ampersand, BitOr, BitXor, BitNot, Shl, Shr,
    Increment, Decrement,
    Semicolon, Comma,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Dot, Arrow,
    Colon, Question,
    Eof,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub ty: TokenType,
    pub text: String,
    pub line: i32,
    pub column: i32,
}

#[derive(Debug, Clone)]
pub struct LexerError {
    pub message: String,
    pub line: i32,
    pub column: i32,
    pub code: i32,
}

#[derive(Debug, Clone)]
struct MacroDef {
    params: Vec<String>,
    body: Vec<Token>,
}

pub struct Lexer {
    chars: Vec<char>,
    errors: Vec<LexerError>,
    pos: usize,
    line: i32,
    column: i32,
    macros: HashMap<String, MacroDef>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        let mut macros = HashMap::new();
        // Predefine common stdio macros for fprintf compatibility
        macros.insert("stdout".to_string(), MacroDef {
            params: vec![],
            body: vec![Token {
                ty: TokenType::Number,
                text: "1".to_string(),
                line: 0,
                column: 0,
            }],
        });
        macros.insert("stderr".to_string(), MacroDef {
            params: vec![],
            body: vec![Token {
                ty: TokenType::Number,
                text: "2".to_string(),
                line: 0,
                column: 0,
            }],
        });
        macros.insert("NULL".to_string(), MacroDef {
            params: vec![],
            body: vec![Token {
                ty: TokenType::Number,
                text: "0".to_string(),
                line: 0,
                column: 0,
            }],
        });
        macros.insert("EOF".to_string(), MacroDef {
            params: vec![],
            body: vec![Token {
                ty: TokenType::Number,
                text: "-1".to_string(),
                line: 0,
                column: 0,
            }],
        });
        macros.insert("stdin".to_string(), MacroDef {
            params: vec![],
            body: vec![Token {
                ty: TokenType::Number,
                text: "0".to_string(),
                line: 0,
                column: 0,
            }],
        });
        Self {
            chars: source.chars().collect(),
            errors: Vec::new(),
            pos: 0,
            line: 1,
            column: 1,
            macros,
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
        let expanded = self.expand_macros(tokens);
        (expanded, self.errors)
    }

    pub fn into_errors(self) -> Vec<LexerError> {
        self.errors
    }

    fn next_token(&mut self) -> Token {
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
                if self.match_char('+') { return self.make_token(TokenType::Increment, "++"); }
                if self.match_char('=') { return self.make_token(TokenType::PlusAssign, "+="); }
                self.advance();
                self.make_token(TokenType::Plus, "+")
            }
            '-' => {
                if self.match_char('>') { return self.make_token(TokenType::Arrow, "->"); }
                if self.match_char('-') { return self.make_token(TokenType::Decrement, "--"); }
                if self.match_char('=') { return self.make_token(TokenType::MinusAssign, "-="); }
                self.advance();
                self.make_token(TokenType::Minus, "-")
            }
            '*' => {
                if self.match_char('=') { return self.make_token(TokenType::StarAssign, "*="); }
                self.advance();
                self.make_token(TokenType::Star, "*")
            }
            '/' => {
                if self.match_char('=') { return self.make_token(TokenType::SlashAssign, "/="); }
                self.advance();
                self.make_token(TokenType::Slash, "/")
            }
            '%' => {
                if self.match_char('=') { return self.make_token(TokenType::PercentAssign, "%="); }
                self.advance();
                self.make_token(TokenType::Percent, "%")
            }
            '=' => {
                if self.match_char('=') { return self.make_token(TokenType::Eq, "=="); }
                self.advance();
                self.make_token(TokenType::Assign, "=")
            }
            '!' => {
                if self.match_char('=') { return self.make_token(TokenType::Ne, "!="); }
                self.advance();
                self.make_token(TokenType::Not, "!")
            }

            '&' => {
                if self.match_char('&') { return self.make_token(TokenType::AndAnd, "&&"); }
                if self.match_char('=') { return self.make_token(TokenType::AndAssign, "&="); }
                self.advance();
                self.make_token(TokenType::Ampersand, "&")
            }
            '|' => {
                if self.match_char('|') { return self.make_token(TokenType::OrOr, "||"); }
                if self.match_char('=') { return self.make_token(TokenType::OrAssign, "|="); }
                self.advance();
                self.make_token(TokenType::BitOr, "|")
            }
            '^' => {
                if self.match_char('=') { return self.make_token(TokenType::XorAssign, "^="); }
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
                if self.match_char('=') { return self.make_token(TokenType::Le, "<="); }
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
                if self.match_char('=') { return self.make_token(TokenType::Ge, ">="); }
                self.advance();
                self.make_token(TokenType::Gt, ">")
            }
            ';' => { self.advance(); self.make_token(TokenType::Semicolon, ";") }
            ',' => { self.advance(); self.make_token(TokenType::Comma, ",") }
            '(' => { self.advance(); self.make_token(TokenType::LParen, "(") }
            ')' => { self.advance(); self.make_token(TokenType::RParen, ")") }
            '{' => { self.advance(); self.make_token(TokenType::LBrace, "{") }
            '}' => { self.advance(); self.make_token(TokenType::RBrace, "}") }
            '[' => { self.advance(); self.make_token(TokenType::LBracket, "[") }
            ']' => { self.advance(); self.make_token(TokenType::RBracket, "]") }
            '.' => { self.advance(); self.make_token(TokenType::Dot, ".") }
            ':' => { self.advance(); self.make_token(TokenType::Colon, ":") }
            '?' => { self.advance(); self.make_token(TokenType::Question, "?") }
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
        let ty = keyword_type(&text).unwrap_or(TokenType::Identifier);
        self.make_token(ty, &text)
    }

    fn number(&mut self) -> Token {
        let start = self.pos;
        let mut is_hex_or_octal = false;
        let mut val: u64 = 0;

        // Hexadecimal literal
        if self.peek(0) == '0' && (self.peek(1) == 'x' || self.peek(1) == 'X') {
            is_hex_or_octal = true;
            self.advance(); // '0'
            self.advance(); // 'x' or 'X'
            let hex_start = self.pos;
            while self.pos < self.chars.len() && self.peek(0).is_ascii_hexdigit() {
                self.advance();
            }
            if self.pos == hex_start {
                self.errors.push(LexerError {
                    message: "十六进制数字格式错误".to_string(),
                    line: self.line,
                    column: self.column,
                    code: ErrorCode::E1001_UnknownChar as i32,
                });
                return self.make_token(TokenType::Unknown, "0x");
            }
            let hex_str: String = self.chars[hex_start..self.pos].iter().collect();
            match u64::from_str_radix(&hex_str, 16) {
                Ok(v) => val = v,
                Err(_) => {
                    self.errors.push(LexerError {
                        message: format!("十六进制数值 0x{} 超出可表示范围", hex_str),
                        line: self.line,
                        column: self.column,
                        code: ErrorCode::E1006_UnsupportedFeature as i32,
                    });
                    return self.make_token(TokenType::Number, "0");
                }
            }
        }
        // Octal literal: 0[0-7]+
        else if self.peek(0) == '0' && self.peek(1).is_ascii_digit() {
            is_hex_or_octal = true;
            self.advance(); // '0'
            let oct_start = self.pos;
            while self.pos < self.chars.len() && self.peek(0) >= '0' && self.peek(0) <= '7' {
                self.advance();
            }
            if self.pos > oct_start {
                let oct_str: String = self.chars[oct_start..self.pos].iter().collect();
                val = u64::from_str_radix(&oct_str, 8).unwrap_or(0);
            }
        }
        // Decimal literal
        else {
            while self.pos < self.chars.len() && self.peek(0).is_ascii_digit() {
                self.advance();
            }
        }

        // check for float literal (e.g. 3.14)
        if !is_hex_or_octal && self.peek(0) == '.' && self.peek(1).is_ascii_digit() {
            self.advance(); // '.'
            while self.pos < self.chars.len() && self.peek(0).is_ascii_digit() {
                self.advance();
            }
            let text: String = self.chars[start..self.pos].iter().collect();
            return self.make_token(TokenType::FloatLiteral, &text);
        }

        // For decimal, compute value now (before suffix parsing)
        if !is_hex_or_octal {
            let text: String = self.chars[start..self.pos].iter().collect();
            val = text.parse::<u64>().unwrap_or(0);
        }

        // Parse suffix: [Uu][Ll]? or [Ll][Uu]?
        let mut has_u = false;
        let mut has_l = false;

        if self.peek(0) == 'U' || self.peek(0) == 'u' {
            has_u = true;
            self.advance();
            if self.peek(0) == 'L' || self.peek(0) == 'l' {
                has_l = true;
                self.advance();
                if self.peek(0) == 'L' || self.peek(0) == 'l' {
                    self.advance();
                }
            }
        } else if self.peek(0) == 'L' || self.peek(0) == 'l' {
            has_l = true;
            self.advance();
            if self.peek(0) == 'L' || self.peek(0) == 'l' {
                self.advance();
            }
            if self.peek(0) == 'U' || self.peek(0) == 'u' {
                has_u = true;
                self.advance();
            }
        }

        // Determine token type according to C standard rules
        if has_u && has_l {
            // unsigned long / unsigned long long — map to unsigned int if fits
            if val > u32::MAX as u64 {
                self.errors.push(LexerError {
                    message: format!("unsigned long long 常量 {} 超出支持范围", val),
                    line: self.line,
                    column: self.column,
                    code: ErrorCode::E1006_UnsupportedFeature as i32,
                });
                return self.make_token(TokenType::UnsignedLiteral, "0");
            }
            return self.make_token(TokenType::UnsignedLiteral, &val.to_string());
        }

        if has_u {
            if val > u32::MAX as u64 {
                self.errors.push(LexerError {
                    message: format!("unsigned 常量 {} 超出 unsigned int 范围", val),
                    line: self.line,
                    column: self.column,
                    code: ErrorCode::E1006_UnsupportedFeature as i32,
                });
                return self.make_token(TokenType::UnsignedLiteral, "0");
            }
            return self.make_token(TokenType::UnsignedLiteral, &val.to_string());
        }

        if has_l {
            if val > i64::MAX as u64 {
                self.errors.push(LexerError {
                    message: format!("long long 常量 {} 超出范围", val),
                    line: self.line,
                    column: self.column,
                    code: ErrorCode::E1006_UnsupportedFeature as i32,
                });
                return self.make_token(TokenType::LongLiteral, "0");
            }
            return self.make_token(TokenType::LongLiteral, &val.to_string());
        }

        // No suffix: apply C standard type promotion rules
        if is_hex_or_octal {
            if val <= i32::MAX as u64 {
                self.make_token(TokenType::Number, &val.to_string())
            } else if val <= u32::MAX as u64 {
                self.make_token(TokenType::UnsignedLiteral, &val.to_string())
            } else if val <= i64::MAX as u64 {
                self.make_token(TokenType::LongLiteral, &val.to_string())
            } else {
                self.errors.push(LexerError {
                    message: format!("整数常量 {} 超出可表示范围", val),
                    line: self.line,
                    column: self.column,
                    code: ErrorCode::E1006_UnsupportedFeature as i32,
                });
                self.make_token(TokenType::Number, "0")
            }
        } else {
            // Decimal: no automatic unsigned promotion
            if val <= i32::MAX as u64 {
                self.make_token(TokenType::Number, &val.to_string())
            } else if val <= i64::MAX as u64 {
                self.make_token(TokenType::LongLiteral, &val.to_string())
            } else {
                self.errors.push(LexerError {
                    message: format!("整数常量 {} 超出可表示范围", val),
                    line: self.line,
                    column: self.column,
                    code: ErrorCode::E1006_UnsupportedFeature as i32,
                });
                self.make_token(TokenType::Number, "0")
            }
        }
    }

    fn string_literal(&mut self) -> Token {
        let start = self.pos;
        self.advance(); // consume opening "
        let mut value = String::new();
        while self.pos < self.chars.len() && self.peek(0) != '"' {
            if self.peek(0) == '\n' {
                self.errors.push(LexerError {
                    message: "字符串不能跨行".to_string(),
                    line: self.line,
                    column: self.column,
                    code: ErrorCode::E1003_StringCrossLine as i32,
                });
                break;
            }
            if self.peek(0) == '\\' && self.pos + 1 < self.chars.len() {
                let next = self.chars[self.pos + 1];
                match next {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    'a' => value.push('\x07'),
                    'b' => value.push('\x08'),
                    'f' => value.push('\x0C'),
                    'v' => value.push('\x0B'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    '0' => value.push('\0'),
                    'x' => {
                        // \xHH hex escape
                        let h1 = self.peek(2);
                        let h2 = self.peek(3);
                        if h1.is_ascii_hexdigit() && h2.is_ascii_hexdigit() {
                            let hex: String = self.chars[self.pos + 2..self.pos + 4].iter().collect();
                            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                                value.push(byte as char);
                            }
                            self.advance();
                            self.advance();
                            self.advance();
                            self.advance();
                            continue;
                        }
                        value.push(next);
                    }
                    _ => value.push(next),
                }
                self.advance();
                self.advance();
            } else {
                let c = self.advance();
                value.push(c);
            }
        }
        if self.pos >= self.chars.len() || self.peek(0) != '"' {
            self.errors.push(LexerError {
                message: "字符串未闭合".to_string(),
                line: self.line,
                column: self.column,
                code: ErrorCode::E1002_UnterminatedString as i32,
            });
        } else {
            self.advance(); // consume closing "
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        let mut tok = self.make_token(TokenType::String, &text);
        tok.text = value;
        tok
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

    fn skip_comment(&mut self) {
        while self.pos < self.chars.len() && self.peek(0) != '\n' {
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) {
        self.advance(); // '/'
        self.advance(); // '*'
        while self.pos < self.chars.len() {
            if self.peek(0) == '*' && self.peek(1) == '/' {
                self.advance();
                self.advance();
                return;
            }
            self.advance();
        }
        self.errors.push(LexerError {
            message: "块注释未闭合".to_string(),
            line: self.line,
            column: self.column,
            code: ErrorCode::E1010_UnterminatedComment as i32,
        });
    }

    fn char_literal(&mut self) -> Token {
        let start = self.pos;
        self.advance(); // consume opening '
        let mut value = 0i32;
        let mut valid = true;
        if self.pos < self.chars.len() && self.peek(0) == '\'' {
            self.errors.push(LexerError {
                message: "空字符字面量".to_string(),
                line: self.line,
                column: self.column,
                code: ErrorCode::E1001_UnknownChar as i32,
            });
            valid = false;
        } else if self.pos < self.chars.len() && self.peek(0) == '\\' && self.pos + 1 < self.chars.len() {
            let next = self.chars[self.pos + 1];
            value = match next {
                'n' => '\n' as i32,
                't' => '\t' as i32,
                'r' => '\r' as i32,
                'a' => 0x07,
                'b' => 0x08,
                'f' => 0x0C,
                'v' => 0x0B,
                '\\' => '\\' as i32,
                '\'' => '\'' as i32,
                '0' => 0,
                'x' => {
                    let h1 = self.peek(2);
                    let h2 = self.peek(3);
                    if h1.is_ascii_hexdigit() && h2.is_ascii_hexdigit() {
                        let hex: String = self.chars[self.pos + 2..self.pos + 4].iter().collect();
                        u8::from_str_radix(&hex, 16).unwrap_or(0) as i32
                    } else {
                        self.errors.push(LexerError {
                            message: "字符字面量十六进制转义格式错误".to_string(),
                            line: self.line,
                            column: self.column,
                            code: ErrorCode::E1001_UnknownChar as i32,
                        });
                        valid = false;
                        0
                    }
                }
                _ => {
                    self.errors.push(LexerError {
                        message: format!("未知字符转义: '\\{}'", next),
                        line: self.line,
                        column: self.column,
                        code: ErrorCode::E1001_UnknownChar as i32,
                    });
                    valid = false;
                    0
                }
            };
            self.advance();
            self.advance();
            if next == 'x' && valid {
                self.advance();
                self.advance();
            }
        } else if self.pos < self.chars.len() {
            value = self.peek(0) as i32;
            self.advance();
        } else {
            self.errors.push(LexerError {
                message: "字符字面量未闭合".to_string(),
                line: self.line,
                column: self.column,
                code: ErrorCode::E1002_UnterminatedString as i32,
            });
            valid = false;
        }
        if self.pos < self.chars.len() && self.peek(0) == '\'' {
            self.advance();
        } else {
            self.errors.push(LexerError {
                message: "字符字面量未闭合".to_string(),
                line: self.line,
                column: self.column,
                code: ErrorCode::E1002_UnterminatedString as i32,
            });
            valid = false;
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        let mut tok = self.make_token(TokenType::CharLiteral, &text);
        if valid {
            tok.text = value.to_string();
        }
        tok
    }

    fn skip_preprocessor_directive(&mut self) {
        self.advance(); // consume '#'
        self.skip_whitespace();

        if self.chars[self.pos..].starts_with(&['d', 'e', 'f', 'i', 'n', 'e']) {
            self.pos += 6;
            self.column += 6;
            self.parse_define_directive();
            return;
        }

        if self.chars[self.pos..].starts_with(&['i', 'n', 'c', 'l', 'u', 'd', 'e']) {
            self.pos += 7;
            self.column += 7;
            self.skip_whitespace();
            if let Some(path) = self.parse_include_path() {
                if let Some(stub) = Self::load_stub(&path) {
                    let mut line_end = self.pos;
                    while line_end < self.chars.len() && self.chars[line_end] != '\n' {
                        line_end += 1;
                    }
                    if line_end < self.chars.len() && self.chars[line_end] == '\n' {
                        line_end += 1;
                    }
                    let stub_chars: Vec<char> = stub.replace('\n', " ").chars().collect();
                    self.chars.splice(line_end..line_end, stub_chars);
                }
            }
            while self.pos < self.chars.len() && self.peek(0) != '\n' {
                self.advance();
            }
            return;
        }

        while self.pos < self.chars.len() && self.peek(0) != '\n' {
            self.advance();
        }
    }

    fn parse_include_path(&mut self) -> Option<String> {
        let delimiter = self.peek(0);
        if delimiter != '<' && delimiter != '"' {
            return None;
        }
        self.advance(); // consume opening delimiter
        let start = self.pos;
        let end_delim = if delimiter == '<' { '>' } else { '"' };
        while self.pos < self.chars.len() && self.peek(0) != end_delim {
            self.advance();
        }
        let path: String = self.chars[start..self.pos].iter().collect();
        if self.pos < self.chars.len() && self.peek(0) == end_delim {
            self.advance(); // consume closing delimiter
        }
        Some(path)
    }

    fn load_stub(path: &str) -> Option<&'static str> {
        match path {
            "stdio.h" => Some(include_str!("../../runtime_libc/include/stdio.h")),
            "stdlib.h" => Some(include_str!("../../runtime_libc/include/stdlib.h")),
            "ctype.h" => Some(include_str!("../../runtime_libc/include/ctype.h")),
            "math.h" => Some(include_str!("../../runtime_libc/include/math.h")),
            "string.h" => Some(include_str!("../../runtime_libc/include/string.h")),
            _ => None,
        }
    }

    fn parse_define_directive(&mut self) {
        self.skip_whitespace();
        if self.pos >= self.chars.len() || !(self.peek(0).is_ascii_alphabetic() || self.peek(0) == '_') {
            self.errors.push(LexerError {
                message: "#define 后预期宏名称".to_string(),
                line: self.line,
                column: self.column,
                code: ErrorCode::E1005_InvalidDefine as i32,
            });
            while self.pos < self.chars.len() && self.peek(0) != '\n' {
                self.advance();
            }
            return;
        }

        let name_start = self.pos;
        while self.pos < self.chars.len() {
            let c = self.peek(0);
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let name: String = self.chars[name_start..self.pos].iter().collect();

        let mut params: Vec<String> = Vec::new();
        // 检查是否有参数列表：宏名后紧跟 '('（中间无空白）
        if self.peek(0) == '(' {
            self.advance(); // consume '('
            loop {
                self.skip_whitespace();
                if self.peek(0) == ')' {
                    self.advance();
                    break;
                }
                if !(self.peek(0).is_ascii_alphabetic() || self.peek(0) == '_') {
                    self.errors.push(LexerError {
                        message: "宏参数必须是标识符".to_string(),
                        line: self.line,
                        column: self.column,
                        code: ErrorCode::E1005_InvalidDefine as i32,
                    });
                    while self.pos < self.chars.len() && self.peek(0) != '\n' {
                        self.advance();
                    }
                    return;
                }
                let p_start = self.pos;
                while self.pos < self.chars.len() {
                    let c = self.peek(0);
                    if c.is_ascii_alphanumeric() || c == '_' {
                        self.advance();
                    } else {
                        break;
                    }
                }
                params.push(self.chars[p_start..self.pos].iter().collect());
                self.skip_whitespace();
                if self.peek(0) == ',' {
                    self.advance();
                } else if self.peek(0) == ')' {
                    self.advance();
                    break;
                } else {
                    self.errors.push(LexerError {
                        message: "宏参数列表格式错误".to_string(),
                        line: self.line,
                        column: self.column,
                        code: ErrorCode::E1005_InvalidDefine as i32,
                    });
                    while self.pos < self.chars.len() && self.peek(0) != '\n' {
                        self.advance();
                    }
                    return;
                }
            }
        }

        self.skip_whitespace();

        let body_start = self.pos;
        while self.pos < self.chars.len() && self.peek(0) != '\n' {
            self.advance();
        }
        let body: String = self.chars[body_start..self.pos].iter().collect();

        let (body_tokens, _) = Lexer::new(&body).tokenize();
        let mdef = MacroDef {
            params,
            body: body_tokens.into_iter().filter(|t| t.ty != TokenType::Eof).collect(),
        };
        self.macros.insert(name, mdef);
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

    fn peek(&self, offset: usize) -> char {
        if self.pos >= self.chars.len() {
            '\0'
        } else {
            self.chars.get(self.pos + offset).copied().unwrap_or('\0')
        }
    }

    fn advance(&mut self) -> char {
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

    fn match_char(&mut self, expected: char) -> bool {
        if self.peek(1) != expected {
            return false;
        }
        self.advance();
        self.advance();
        true
    }

    fn make_token(&self, ty: TokenType, text: &str) -> Token {
        Token {
            ty,
            text: text.to_string(),
            line: self.line,
            column: self.column - text.len() as i32,
        }
    }
}

fn keyword_type(text: &str) -> Option<TokenType> {
    match text {
        "int"      => Some(TokenType::Int),
        "void"     => Some(TokenType::Void),
        "char"     => Some(TokenType::Char),
        "if"       => Some(TokenType::If),
        "else"     => Some(TokenType::Else),
        "while"    => Some(TokenType::While),
        "do"       => Some(TokenType::Do),
        "for"      => Some(TokenType::For),
        "return"   => Some(TokenType::Return),
        "break"    => Some(TokenType::Break),
        "continue" => Some(TokenType::Continue),
        "struct"   => Some(TokenType::Struct),
        "union"    => Some(TokenType::Union),
        "sizeof"   => Some(TokenType::Sizeof),
        "switch"   => Some(TokenType::Switch),
        "case"     => Some(TokenType::Case),
        "default"  => Some(TokenType::Default),
        "typedef"  => Some(TokenType::Typedef),
        "enum"     => Some(TokenType::Enum),
        "unsigned" => Some(TokenType::Unsigned),
        "long"     => Some(TokenType::Long),
        "short"    => Some(TokenType::Short),
        "signed"   => Some(TokenType::Signed),
        "const"    => Some(TokenType::Const),
        "extern"   => Some(TokenType::Extern),
        "float"    => Some(TokenType::Float),
        "double"   => Some(TokenType::Double),
        "NULL"     => Some(TokenType::Null),
        "null"     => Some(TokenType::Null),
        _          => None,
    }
}

