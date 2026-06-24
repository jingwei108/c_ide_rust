//! C 预处理器指令支持（宏定义、条件编译、#include 存根）。

use cide_shared::ErrorCode;

use super::token::{LexerError, Token, TokenType};
use super::Lexer;

/// 宏定义。
#[derive(Debug, Clone)]
pub struct MacroDef {
    pub params: Vec<String>,
    pub body: Vec<Token>,
}

/// 条件编译状态。
#[derive(Debug, Clone, Copy)]
pub struct ConditionalState {
    pub active: bool,
    pub has_else: bool,
}

impl Lexer {
    pub(crate) fn is_skipping(&self) -> bool {
        self.conditional_stack.iter().any(|s| !s.active)
    }

    /// 在条件编译跳过模式下，跳过一个不活跃的逻辑行（或注释块、预处理指令）。
    /// 返回 None 表示已到 EOF；Some(false) 表示预处理指令导致 skipping 结束；
    /// Some(true) 表示仍在 skipping 中，应继续循环。
    pub(crate) fn skip_inactive_line(&mut self) -> Option<bool> {
        // 跳过空白
        while self.pos < self.chars.len() {
            let c = self.peek(0);
            if c.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
        if self.pos >= self.chars.len() {
            return None;
        }
        // 跳过块注释
        if self.peek(0) == '/' && self.peek(1) == '*' {
            self.advance();
            self.advance();
            while self.pos < self.chars.len() {
                if self.peek(0) == '*' && self.peek(1) == '/' {
                    self.advance();
                    self.advance();
                    break;
                }
                self.advance();
            }
            return Some(true);
        }
        // 跳过行注释
        if self.peek(0) == '/' && self.peek(1) == '/' {
            while self.pos < self.chars.len() && self.peek(0) != '\n' {
                self.advance();
            }
            if self.pos < self.chars.len() {
                self.advance();
            }
            return Some(true);
        }
        if self.peek(0) == '#' {
            self.skip_preprocessor_directive();
            return Some(self.is_skipping());
        }
        // 不是预处理指令，跳过整行
        while self.pos < self.chars.len() && self.peek(0) != '\n' {
            self.advance();
        }
        if self.pos < self.chars.len() {
            self.advance();
        }
        Some(true)
    }

    pub(crate) fn skip_preprocessor_directive(&mut self) {
        self.advance(); // consume '#'
        self.skip_whitespace();

        // 读取指令名
        let dir_start = self.pos;
        while self.pos < self.chars.len() {
            let c = self.peek(0);
            if c.is_ascii_alphabetic() {
                self.advance();
            } else {
                break;
            }
        }
        let directive: String = self.chars[dir_start..self.pos].iter().collect();

        match directive.as_str() {
            "define" => {
                if !self.is_skipping() {
                    self.parse_define_directive();
                } else {
                    while self.pos < self.chars.len() && self.peek(0) != '\n' {
                        self.advance();
                    }
                }
            }
            "include" => {
                if !self.is_skipping() {
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
                }
                while self.pos < self.chars.len() && self.peek(0) != '\n' {
                    self.advance();
                }
            }
            "ifdef" => {
                self.handle_conditional_directive(true);
            }
            "ifndef" => {
                self.handle_conditional_directive(false);
            }
            "else" => {
                self.handle_else_directive();
            }
            "endif" => {
                self.handle_endif_directive();
            }
            _ => {
                // 未知指令，跳过整行
                while self.pos < self.chars.len() && self.peek(0) != '\n' {
                    self.advance();
                }
            }
        }
    }

    pub(crate) fn handle_conditional_directive(&mut self, is_ifdef: bool) {
        self.skip_whitespace();
        let id_start = self.pos;
        while self.pos < self.chars.len() {
            let c = self.peek(0);
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let _ident: String = self.chars[id_start..self.pos].iter().collect();

        // 跳过行尾
        while self.pos < self.chars.len() && self.peek(0) != '\n' {
            self.advance();
        }

        let active = if self.is_skipping() {
            false
        } else {
            let defined = self.macros.contains_key(&_ident);
            if is_ifdef {
                defined
            } else {
                !defined
            }
        };

        self.conditional_stack.push(ConditionalState { active, has_else: false });
    }

    pub(crate) fn handle_else_directive(&mut self) {
        while self.pos < self.chars.len() && self.peek(0) != '\n' {
            self.advance();
        }

        if let Some(state) = self.conditional_stack.last_mut() {
            if state.has_else {
                self.errors.push(LexerError {
                    message: "重复的 #else".to_string(),
                    line: self.line,
                    column: self.column,
                    code: ErrorCode::E1012_DuplicateElse as i32,
                });
            } else {
                state.has_else = true;
                state.active = !state.active;
            }
        } else {
            self.errors.push(LexerError {
                message: "没有匹配的 #ifdef / #ifndef 就出现 #else".to_string(),
                line: self.line,
                column: self.column,
                code: ErrorCode::E1011_UnmatchedConditional as i32,
            });
        }
    }

    pub(crate) fn handle_endif_directive(&mut self) {
        while self.pos < self.chars.len() && self.peek(0) != '\n' {
            self.advance();
        }

        if self.conditional_stack.pop().is_none() {
            self.errors.push(LexerError {
                message: "没有匹配的 #ifdef / #ifndef 就出现 #endif".to_string(),
                line: self.line,
                column: self.column,
                code: ErrorCode::E1011_UnmatchedConditional as i32,
            });
        }
    }

    pub(crate) fn parse_include_path(&mut self) -> Option<String> {
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

    pub(crate) fn load_stub(path: &str) -> Option<&'static str> {
        match path {
            "stdio.h" => Some(include_str!("../../../runtime_libc/include/stdio.h")),
            "stdlib.h" => Some(include_str!("../../../runtime_libc/include/stdlib.h")),
            "ctype.h" => Some(include_str!("../../../runtime_libc/include/ctype.h")),
            "math.h" => Some(include_str!("../../../runtime_libc/include/math.h")),
            "string.h" => Some(include_str!("../../../runtime_libc/include/string.h")),
            "limits.h" => Some(include_str!("../../../runtime_libc/include/limits.h")),
            "stdbool.h" => Some(include_str!("../../../runtime_libc/include/stdbool.h")),
            "stddef.h" => Some(include_str!("../../../runtime_libc/include/stddef.h")),
            "stdint.h" => Some(include_str!("../../../runtime_libc/include/stdint.h")),
            "time.h" => Some(include_str!("../../../runtime_libc/include/time.h")),
            "assert.h" => Some(include_str!("../../../runtime_libc/include/assert.h")),
            "errno.h" => Some(include_str!("../../../runtime_libc/include/errno.h")),
            "float.h" => Some(include_str!("../../../runtime_libc/include/float.h")),
            _ => None,
        }
    }

    pub(crate) fn parse_define_directive(&mut self) {
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

        // 跳过宏名与 body 之间的空白，但不要把下一行也吞进来
        while self.pos < self.chars.len() {
            let c = self.peek(0);
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }

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
}
