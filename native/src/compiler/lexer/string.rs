//! 字符串与字符字面量解析。

use crate::diagnostics::error_codes::ErrorCode;

use super::token::{LexerError, Token, TokenType};
use super::Lexer;

impl Lexer {
    pub(crate) fn string_literal(&mut self) -> Token {
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

    pub(crate) fn char_literal(&mut self) -> Token {
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
}
