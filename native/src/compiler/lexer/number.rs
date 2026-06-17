//! 数字字面量解析（整数、浮点数、后缀处理）。

use crate::diagnostics::error_codes::ErrorCode;

use super::token::{LexerError, Token, TokenType};
use super::Lexer;

impl Lexer {
    pub(crate) fn number(&mut self) -> Token {
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

        // check for float literal (e.g. 3.14, 3.14f, 3.f)
        let is_float_prefix = !is_hex_or_octal
            && self.peek(0) == '.'
            && (self.peek(1).is_ascii_digit() || self.peek(1) == 'f' || self.peek(1) == 'F');
        if is_float_prefix {
            self.advance(); // '.'
            while self.pos < self.chars.len() && self.peek(0).is_ascii_digit() {
                self.advance();
            }
            // Optional float suffix: f/F
            if self.peek(0) == 'f' || self.peek(0) == 'F' {
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
}
