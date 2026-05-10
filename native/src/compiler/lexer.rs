use crate::diagnostics::error_codes::ErrorCode;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Int, Void, Char, If, Else, While, Do, For, Return, Break, Continue,
    Struct, Sizeof, Switch, Case, Default, Typedef, Enum, Unsigned, Long, Short, Signed, Const,
    Identifier, Number, CharLiteral, String,
    Plus, Minus, Star, Slash, Percent,
    Eq, Ne, Lt, Le, Gt, Ge,
    AndAnd, OrOr, Not,
    Assign, PlusAssign, MinusAssign, StarAssign, SlashAssign, PercentAssign,
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

pub struct Lexer {
    source: String,
    errors: Vec<LexerError>,
    pos: usize,
    line: i32,
    column: i32,
    macros: HashMap<String, Vec<Token>>,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Self {
            source,
            errors: Vec::new(),
            pos: 0,
            line: 1,
            column: 1,
            macros: HashMap::new(),
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

        if self.pos >= self.source.len() {
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
                self.advance();
                self.make_token(TokenType::Ampersand, "&")
            }
            '|' => {
                if self.match_char('|') { return self.make_token(TokenType::OrOr, "||"); }
                self.advance();
                self.make_token(TokenType::BitOr, "|")
            }
            '^' => {
                self.advance();
                self.make_token(TokenType::BitXor, "^")
            }
            '~' => {
                self.advance();
                self.make_token(TokenType::BitNot, "~")
            }
            '<' => {
                if self.match_char('<') { return self.make_token(TokenType::Shl, "<<"); }
                if self.match_char('=') { return self.make_token(TokenType::Le, "<="); }
                self.advance();
                self.make_token(TokenType::Lt, "<")
            }
            '>' => {
                if self.match_char('>') { return self.make_token(TokenType::Shr, ">>"); }
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
        while self.pos < self.source.len() {
            let c = self.peek(0);
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let text = &self.source[start..self.pos];
        let ty = keyword_type(text).unwrap_or(TokenType::Identifier);
        self.make_token(ty, text)
    }

    fn number(&mut self) -> Token {
        let start = self.pos;
        if self.peek(0) == '0' && (self.peek(1) == 'x' || self.peek(1) == 'X') {
            self.advance(); // '0'
            self.advance(); // 'x' or 'X'
            let hex_start = self.pos;
            while self.pos < self.source.len() && self.peek(0).is_ascii_hexdigit() {
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
            let text = &self.source[start..self.pos];
            // Convert hex to decimal string so parser can parse it
            let hex_str = &text[2..];
            if let Ok(val) = u64::from_str_radix(hex_str, 16) {
                return self.make_token(TokenType::Number, &val.to_string());
            }
            return self.make_token(TokenType::Number, text);
        }
        while self.pos < self.source.len() && self.peek(0).is_ascii_digit() {
            self.advance();
        }
        let text = &self.source[start..self.pos];
        self.make_token(TokenType::Number, text)
    }

    fn string_literal(&mut self) -> Token {
        let start = self.pos;
        self.advance(); // consume opening "
        let mut value = String::new();
        while self.pos < self.source.len() && self.peek(0) != '"' {
            if self.peek(0) == '\n' {
                self.errors.push(LexerError {
                    message: "字符串不能跨行".to_string(),
                    line: self.line,
                    column: self.column,
                    code: ErrorCode::E1003_StringCrossLine as i32,
                });
                break;
            }
            if self.peek(0) == '\\' && self.pos + 1 < self.source.len() {
                let next = self.source.as_bytes()[self.pos + 1] as char;
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
                            let hex = &self.source[self.pos + 2..self.pos + 4];
                            if let Ok(byte) = u8::from_str_radix(hex, 16) {
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
                value.push(self.peek(0));
                self.advance();
            }
        }
        if self.pos >= self.source.len() || self.peek(0) != '"' {
            self.errors.push(LexerError {
                message: "字符串未闭合".to_string(),
                line: self.line,
                column: self.column,
                code: ErrorCode::E1002_UnterminatedString as i32,
            });
        } else {
            self.advance(); // consume closing "
        }
        let mut tok = self.make_token(TokenType::String, &self.source[start..self.pos]);
        tok.text = value;
        tok
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.source.len() {
            let c = self.peek(0);
            if c.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        while self.pos < self.source.len() && self.peek(0) != '\n' {
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) {
        self.advance(); // '/'
        self.advance(); // '*'
        while self.pos < self.source.len() {
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
            code: ErrorCode::E1002_UnterminatedString as i32,
        });
    }

    fn char_literal(&mut self) -> Token {
        let start = self.pos;
        self.advance(); // consume opening '
        let mut value = 0i32;
        let mut valid = true;
        if self.pos < self.source.len() && self.peek(0) == '\'' {
            self.errors.push(LexerError {
                message: "空字符字面量".to_string(),
                line: self.line,
                column: self.column,
                code: ErrorCode::E1001_UnknownChar as i32,
            });
            valid = false;
        } else if self.pos < self.source.len() && self.peek(0) == '\\' && self.pos + 1 < self.source.len() {
            let next = self.source.as_bytes()[self.pos + 1] as char;
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
                        let hex = &self.source[self.pos + 2..self.pos + 4];
                        u8::from_str_radix(hex, 16).unwrap_or(0) as i32
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
        } else if self.pos < self.source.len() {
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
        if self.pos < self.source.len() && self.peek(0) == '\'' {
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
        let mut tok = self.make_token(TokenType::CharLiteral, &self.source[start..self.pos]);
        if valid {
            tok.text = value.to_string();
        }
        tok
    }

    fn skip_preprocessor_directive(&mut self) {
        self.advance(); // consume '#'
        self.skip_whitespace();

        if self.source[self.pos..].starts_with("define") {
            self.pos += 6;
            self.column += 6;
            self.parse_define_directive();
            return;
        }

        while self.pos < self.source.len() && self.peek(0) != '\n' {
            self.advance();
        }
    }

    fn parse_define_directive(&mut self) {
        self.skip_whitespace();
        if self.pos >= self.source.len() || !(self.peek(0).is_ascii_alphabetic() || self.peek(0) == '_') {
            self.errors.push(LexerError {
                message: "#define 后预期宏名称".to_string(),
                line: self.line,
                column: self.column,
                code: ErrorCode::E1005_InvalidDefine as i32,
            });
            while self.pos < self.source.len() && self.peek(0) != '\n' {
                self.advance();
            }
            return;
        }

        let name_start = self.pos;
        while self.pos < self.source.len() {
            let c = self.peek(0);
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let name = self.source[name_start..self.pos].to_string();

        self.skip_whitespace();

        let body_start = self.pos;
        while self.pos < self.source.len() && self.peek(0) != '\n' {
            self.advance();
        }
        let body = self.source[body_start..self.pos].to_string();

        let (body_tokens, _) = Lexer::new(body).tokenize();
        let mut macros = HashMap::new();
        macros.insert(name, body_tokens.into_iter().filter(|t| t.ty != TokenType::Eof).collect());
        self.macros.extend(macros);
    }

    fn expand_macros(&self, tokens: Vec<Token>) -> Vec<Token> {
        let mut result = Vec::new();
        for tok in tokens {
            if tok.ty == TokenType::Identifier {
                if let Some(macro_tokens) = self.macros.get(&tok.text) {
                    for mt in macro_tokens {
                        let mut expanded = mt.clone();
                        expanded.line = tok.line;
                        expanded.column = tok.column;
                        result.push(expanded);
                    }
                    continue;
                }
            }
            result.push(tok);
        }
        result
    }

    fn peek(&self, offset: usize) -> char {
        if self.pos + offset >= self.source.len() {
            '\0'
        } else {
            self.source.as_bytes()[self.pos + offset] as char
        }
    }

    fn advance(&mut self) -> char {
        if self.pos >= self.source.len() {
            return '\0';
        }
        let c = self.source[self.pos..].chars().next().unwrap_or('\0');
        self.pos += c.len_utf8();
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
        _          => None,
    }
}
