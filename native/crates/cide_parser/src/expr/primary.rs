use super::*;

impl Parser {
    pub(crate) fn parse_primary(&mut self) -> Expr {
        if self.match_token(TokenType::Number) {
            let prev = self.previous().clone();
            let value: i32 = prev.text.parse().unwrap_or_else(|_| {
                self.errors.push(ParseError {
                    message: format!("整数常量 '{}' 超出 int 表示范围", prev.text),
                    line: prev.line,
                    column: prev.column,
                    code: ErrorCode::E1006_UnsupportedFeature as i32,
                });
                0
            });
            let loc = SourceLoc {
                line: prev.line,
                column: prev.column,
                file_id: 0,
            };
            return Expr::Literal { value, loc, ty: Type::int() };
        }
        if self.match_token(TokenType::UnsignedLiteral) {
            let prev = self.previous().clone();
            let value: i32 = prev.text.parse::<u32>().unwrap_or_else(|_| {
                self.errors.push(ParseError {
                    message: format!("unsigned 常量 '{}' 超出范围", prev.text),
                    line: prev.line,
                    column: prev.column,
                    code: ErrorCode::E1006_UnsupportedFeature as i32,
                });
                0
            }) as i32;
            let loc = SourceLoc {
                line: prev.line,
                column: prev.column,
                file_id: 0,
            };
            return Expr::Literal {
                value,
                loc,
                ty: Type::unsigned_int(),
            };
        }
        if self.match_token(TokenType::LongLiteral) {
            let prev = self.previous().clone();
            let value: i64 = prev.text.parse().unwrap_or_else(|_| {
                self.errors.push(ParseError {
                    message: format!("long long 常量 '{}' 超出范围", prev.text),
                    line: prev.line,
                    column: prev.column,
                    code: ErrorCode::E1006_UnsupportedFeature as i32,
                });
                0
            });
            let loc = SourceLoc {
                line: prev.line,
                column: prev.column,
                file_id: 0,
            };
            return Expr::LongLiteral {
                value,
                loc,
                ty: Type::long_long(),
            };
        }
        if self.match_token(TokenType::FloatLiteral) {
            let prev = self.previous().clone();
            let text = prev.text.trim_end_matches('f').trim_end_matches('F');
            let value: f64 = text.parse().unwrap_or_else(|_| {
                self.errors.push(ParseError {
                    message: format!("浮点常量 '{}' 格式无效", prev.text),
                    line: prev.line,
                    column: prev.column,
                    code: ErrorCode::E1006_UnsupportedFeature as i32,
                });
                0.0
            });
            let loc = SourceLoc {
                line: prev.line,
                column: prev.column,
                file_id: 0,
            };
            return Expr::FloatLiteral { value, loc, ty: Type::float() };
        }
        if self.match_token(TokenType::CharLiteral) {
            let prev = self.previous().clone();
            let value: i32 = prev.text.parse().unwrap_or_else(|_| {
                self.errors.push(ParseError {
                    message: format!("字符常量 '{}' 解析失败", prev.text),
                    line: prev.line,
                    column: prev.column,
                    code: ErrorCode::E1006_UnsupportedFeature as i32,
                });
                0
            });
            let loc = SourceLoc {
                line: prev.line,
                column: prev.column,
                file_id: 0,
            };
            return Expr::Literal { value, loc, ty: Type::char() };
        }
        if self.match_token(TokenType::String) {
            let value = self.previous().text.clone();
            let loc = SourceLoc {
                line: self.previous().line,
                column: self.previous().column,
                file_id: 0,
            };
            let array_size = value.len() as i32 + 1; // including null terminator
            return Expr::StringLiteral {
                value,
                loc,
                ty: Type::Array {
                    element: Box::new(Type::char()),
                    array_size,
                    dims: vec![array_size],
                    is_const: false,
                    is_vla: false,
                    vla_dims: vec![],
                },
            };
        }
        if self.match_token(TokenType::Null) {
            let loc = SourceLoc {
                line: self.previous().line,
                column: self.previous().column,
                file_id: 0,
            };
            return Expr::Literal {
                value: 0,
                loc,
                ty: Type::pointer_to(Type::void()),
            };
        }
        if self.is_cpp_mode && self.match_token(TokenType::This) {
            let loc = SourceLoc {
                line: self.previous().line,
                column: self.previous().column,
                file_id: 0,
            };
            return Expr::This { loc, ty: Type::default() };
        }
        if self.is_cpp_mode && self.check(TokenType::LBracket) {
            return self.parse_lambda_expr();
        }
        if self.check(TokenType::Identifier) && self.peek(0).text == "__asm__" {
            // 支持 GCC 风格内联汇编占位：__asm__("...")
            // 教学子集不执行汇编指令，仅消费语法并返回 void 字面量。
            let name_tok = self.advance().clone();
            self.consume(TokenType::LParen, "__asm__ 后预期 '('");
            self.consume(TokenType::String, "__asm__ 预期汇编字符串");
            self.consume(TokenType::RParen, "__asm__ 预期 ')'");
            let loc = SourceLoc {
                line: name_tok.line,
                column: name_tok.column,
                file_id: 0,
            };
            return Expr::Literal {
                value: 0,
                loc,
                ty: Type::void(),
            };
        }
        if self.check(TokenType::Identifier) {
            let name_tok = self.advance().clone();
            let mut name = name_tok.text.clone();
            if self.is_cpp_mode && self.check(TokenType::ColonColon) {
                self.advance(); // ::
                let inner = self.consume(TokenType::Identifier, ":: 后预期标识符").clone();
                name = format!("{}__{}", name, inner.text);
            }
            let loc = SourceLoc {
                line: name_tok.line,
                column: name_tok.column,
                file_id: 0,
            };
            return Expr::Identifier { name, loc, ty: Type::default() };
        }
        if self.match_token(TokenType::LParen) {
            let expr = self.parse_expression();
            self.consume(TokenType::RParen, "预期 ')'");
            return expr;
        }
        self.errors.push(ParseError {
            message: "预期表达式".to_string(),
            line: self.current().line,
            column: self.current().column,
            code: ErrorCode::E2003_ExpectedExpr as i32,
        });
        let loc = SourceLoc {
            line: self.current().line,
            column: self.current().column,
            file_id: 0,
        };
        // 消费当前 token，防止外层 parse_statement 在相同位置无限循环。
        if !self.is_at_end() {
            self.advance();
        }
        Expr::Literal { value: 0, loc, ty: Type::int() }
    }
    // =========================================================================
    // Expressions (precedence climbing)
    // =========================================================================
}
