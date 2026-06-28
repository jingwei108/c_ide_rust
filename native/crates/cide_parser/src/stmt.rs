use super::*;

impl Parser {
    // =========================================================================
    // Statements
    // =========================================================================

    pub(crate) fn parse_static_assert(&mut self) {
        // _Static_assert(constant-expression, string-literal);
        // 教学子集目前仅消费语法；常量表达式求值可在 TypeChecker 阶段扩展。
        self.advance(); // _Static_assert
        self.consume(TokenType::LParen, "_Static_assert 后预期 '('");
        // 消费常量表达式直到顶层逗号（避免 parse_expression 将逗号后的字符串纳入逗号表达式）
        let mut paren_depth = 1;
        while !self.is_at_end() {
            if self.check(TokenType::LParen) {
                paren_depth += 1;
                self.advance();
            } else if self.check(TokenType::RParen) {
                paren_depth -= 1;
                if paren_depth == 0 {
                    break;
                }
                self.advance();
            } else if self.check(TokenType::Comma) && paren_depth == 1 {
                break;
            } else {
                self.advance();
            }
        }
        self.consume(TokenType::Comma, "_Static_assert 预期 ','");
        self.consume(TokenType::String, "_Static_assert 预期字符串消息");
        self.consume(TokenType::RParen, "_Static_assert 预期 ')'");
        self.consume(TokenType::Semicolon, "_Static_assert 预期 ';'");
    }
    pub(crate) fn parse_statement(&mut self) -> Stmt {
        match self.current().ty {
            TokenType::Semicolon => {
                let loc = SourceLoc {
                    line: self.current().line,
                    column: self.current().column,
                    file_id: 0,
                };
                self.advance();
                Stmt::Block { stmts: Vec::new(), loc }
            }
            TokenType::LBrace => self.parse_block(),
            TokenType::If => self.parse_if_stmt(),
            TokenType::While => self.parse_while_stmt(),
            TokenType::Do => self.parse_do_while_stmt(),
            TokenType::For => self.parse_for_stmt(),
            TokenType::Return => self.parse_return_stmt(),
            TokenType::Break => self.parse_break_stmt(),
            TokenType::Continue => self.parse_continue_stmt(),
            TokenType::Goto => self.parse_goto_stmt(),
            TokenType::Switch => self.parse_switch_stmt(),
            TokenType::Case | TokenType::Default => self.parse_case_stmt(),
            _ if self.check(TokenType::Identifier) && self.peek(0).text == "_Static_assert" => {
                self.parse_static_assert();
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                    file_id: 0,
                };
                Stmt::Block { stmts: Vec::new(), loc }
            }
            _ if self.is_type_token()
                || self.is_static_token()
                || self.check(TokenType::Register)
                || self.check(TokenType::Auto)
                || self.check(TokenType::Inline) =>
            {
                let is_static = self.is_static_token();
                if is_static {
                    self.advance(); // consume 'static'
                }
                // In C++ mode, 'auto' is a type keyword, not a storage class; don't skip it.
                while self.check(TokenType::Register)
                    || (!self.is_cpp_mode && self.check(TokenType::Auto))
                    || self.check(TokenType::Inline)
                {
                    self.advance();
                }
                self.parse_var_decl_stmt(is_static)
            }
            _ => {
                // Label statement: ident : stmt
                if self.check(TokenType::Identifier) && self.peek(1).ty == TokenType::Colon {
                    return self.parse_label_stmt();
                }
                let checkpoint = self.pos;
                let stmt = self.parse_expr_stmt();
                // NOTE: Parser 零进度保护：若解析未前进则主动 advance，避免死循环。
                if self.pos == checkpoint {
                    self.synchronize(&[
                        TokenType::Semicolon,
                        TokenType::RBrace,
                        TokenType::Int,
                        TokenType::Void,
                        TokenType::Char,
                        TokenType::Float,
                        TokenType::Double,
                        TokenType::Long,
                        TokenType::Bool,
                        TokenType::If,
                        TokenType::While,
                        TokenType::Do,
                        TokenType::For,
                        TokenType::Return,
                        TokenType::Break,
                        TokenType::Continue,
                        TokenType::Struct,
                        TokenType::Switch,
                        TokenType::Typedef,
                    ]);
                }
                stmt
            }
        }
    }
    pub(crate) fn parse_block(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.consume(TokenType::LBrace, "预期 '{'");
        let mut stmts = Vec::new();
        while !self.check(TokenType::RBrace) && !self.is_at_end() {
            let stmt_checkpoint = self.pos;
            // 局部 typedef 声明不产生运行时语句
            if self.check(TokenType::Typedef) {
                self.parse_typedef();
                continue;
            }
            stmts.push(self.parse_statement());
            if self.pos == stmt_checkpoint {
                self.advance();
            }
        }
        self.consume(TokenType::RBrace, "预期 '}'");
        Stmt::Block {
            stmts,
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
                file_id: 0,
            },
        }
    }
    pub(crate) fn parse_var_decl_stmt(&mut self, is_static: bool) -> Stmt {
        let loc = self.current().clone();
        let base_type = self.parse_base_type();
        // Detect C++ constructor initialization syntax: `Type name(args);`
        // We must only treat it as a constructor call when the declarator is a
        // plain identifier (no pointer/reference/function-pointer suffix).
        // Function-pointer declarations such as `int (*fp)(int);` are parsed
        // normally by parse_var_declarator.
        let is_simple_ctor_init = (base_type.kind() == TypeKind::Class || base_type.kind() == TypeKind::TemplateId)
            && self.check(TokenType::Identifier)
            && self.peek(1).ty == TokenType::LParen;

        let (var_type, name, init) = if is_simple_ctor_init {
            let name_tok = self.advance().clone();
            self.advance(); // consume '('
            let args = self.parse_arg_list();
            self.consume(TokenType::RParen, "构造函数初始化预期 ')'");
            let ctor_name = if args.is_empty() {
                format!("__ctor__{}", base_type.name())
            } else {
                format!("__ctor__{}__{}", base_type.name(), args.len())
            };
            (
                base_type.clone(),
                name_tok.text,
                Some(Expr::Call {
                    name: ctor_name,
                    args,
                    loc: SourceLoc {
                        line: loc.line,
                        column: loc.column,
                        file_id: 0,
                    },
                    ty: Type::void(),
                }),
            )
        } else {
            let (var_type, name) = self.parse_var_declarator(&base_type);
            let init = if self.match_token(TokenType::Assign) {
                if self.check(TokenType::LBrace) {
                    Some(self.parse_init_list())
                } else {
                    Some(self.parse_assign())
                }
            } else {
                None
            };
            (var_type, name, init)
        };

        let mut extra_vars = Vec::new();
        while self.match_token(TokenType::Comma) {
            let is_extra_ctor_init = (base_type.kind() == TypeKind::Class || base_type.kind() == TypeKind::TemplateId)
                && self.check(TokenType::Identifier)
                && self.peek(1).ty == TokenType::LParen;
            let (extra_ty, extra_name, extra_init) = if is_extra_ctor_init {
                let name_tok = self.advance().clone();
                self.advance(); // consume '('
                let args = self.parse_arg_list();
                self.consume(TokenType::RParen, "构造函数初始化预期 ')'");
                let ctor_name = if args.is_empty() {
                    format!("__ctor__{}", base_type.name())
                } else {
                    format!("__ctor__{}__{}", base_type.name(), args.len())
                };
                (
                    base_type.clone(),
                    name_tok.text,
                    Some(Expr::Call {
                        name: ctor_name,
                        args,
                        loc: SourceLoc {
                            line: loc.line,
                            column: loc.column,
                            file_id: 0,
                        },
                        ty: Type::void(),
                    }),
                )
            } else {
                let (extra_ty, extra_name) = self.parse_var_declarator(&base_type);
                let extra_init = if self.match_token(TokenType::Assign) {
                    if self.check(TokenType::LBrace) {
                        Some(self.parse_init_list())
                    } else {
                        Some(self.parse_assign())
                    }
                } else {
                    None
                };
                (extra_ty, extra_name, extra_init)
            };
            extra_vars.push((extra_ty, extra_name, extra_init));
        }

        self.consume(TokenType::Semicolon, "变量声明后预期 ';'");
        Stmt::VarDecl {
            var_type,
            name,
            init,
            extra_vars,
            is_static,
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
                file_id: 0,
            },
        }
    }
    pub(crate) fn parse_if_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.consume(TokenType::If, "预期 'if'");
        self.consume(TokenType::LParen, "预期 '('");
        let cond = self.parse_expression();
        self.consume(TokenType::RParen, "预期 ')'");
        let then_stmt = Box::new(self.parse_statement());
        let else_stmt = if self.match_token(TokenType::Else) {
            Some(Box::new(self.parse_statement()))
        } else {
            None
        };
        Stmt::If {
            cond,
            then_stmt,
            else_stmt,
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
                file_id: 0,
            },
        }
    }
    pub(crate) fn parse_while_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.consume(TokenType::While, "预期 'while'");
        self.consume(TokenType::LParen, "预期 '('");
        let cond = self.parse_expression();
        self.consume(TokenType::RParen, "预期 ')'");
        let body = Box::new(self.parse_statement());
        Stmt::While {
            cond,
            body,
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
                file_id: 0,
            },
        }
    }
    pub(crate) fn parse_do_while_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.consume(TokenType::Do, "预期 'do'");
        let body = Box::new(self.parse_statement());
        self.consume(TokenType::While, "预期 'while'");
        self.consume(TokenType::LParen, "预期 '('");
        let cond = self.parse_expression();
        self.consume(TokenType::RParen, "预期 ')'");
        self.consume(TokenType::Semicolon, "do...while 后预期 ';'");
        Stmt::DoWhile {
            body,
            cond,
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
                file_id: 0,
            },
        }
    }
    pub(crate) fn parse_break_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.consume(TokenType::Break, "预期 'break'");
        self.consume(TokenType::Semicolon, "break 后预期 ';'");
        Stmt::Break {
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
                file_id: 0,
            },
        }
    }
    pub(crate) fn parse_continue_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.consume(TokenType::Continue, "预期 'continue'");
        self.consume(TokenType::Semicolon, "continue 后预期 ';'");
        Stmt::Continue {
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
                file_id: 0,
            },
        }
    }
    pub(crate) fn parse_goto_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.consume(TokenType::Goto, "预期 'goto'");
        let label = self.current().text.clone();
        self.consume(TokenType::Identifier, "goto 后预期标签名");
        self.consume(TokenType::Semicolon, "goto 后预期 ';'");
        Stmt::Goto {
            label,
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
                file_id: 0,
            },
        }
    }
    pub(crate) fn parse_label_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        let label = self.current().text.clone();
        self.advance(); // consume identifier
        self.consume(TokenType::Colon, "标签名后预期 ':'");
        let stmt = Box::new(self.parse_statement());
        Stmt::Label {
            label,
            stmt,
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
                file_id: 0,
            },
        }
    }
    pub(crate) fn parse_for_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.consume(TokenType::For, "预期 'for'");
        self.consume(TokenType::LParen, "预期 '('");

        // C++ 模式下检测 range for: for (auto x : expr) 或 for (Type x : expr)
        if self.is_cpp_mode {
            let checkpoint = self.pos;
            let is_range_for = if self.is_type_token() || self.check(TokenType::Auto) {
                let _ = self.parse_base_type();
                while self.match_token(TokenType::Star) {}
                if self.is_cpp_mode && (self.check(TokenType::Ampersand) || self.check(TokenType::AndAnd)) {
                    self.advance();
                    if self.check(TokenType::Const) {
                        self.advance();
                    }
                }
                if self.check(TokenType::Identifier) {
                    self.advance();
                    self.check(TokenType::Colon)
                } else {
                    false
                }
            } else {
                false
            };
            self.pos = checkpoint;

            if is_range_for {
                let var_type = self.parse_base_type();
                let mut final_type = var_type;
                while self.match_token(TokenType::Star) {
                    final_type = Type::pointer_to(final_type);
                }
                if self.is_cpp_mode {
                    if self.match_token(TokenType::Ampersand) {
                        let is_const = self.match_token(TokenType::Const);
                        final_type = Type::Reference {
                            base: Box::new(final_type),
                            is_const,
                        };
                    } else if self.match_token(TokenType::AndAnd) {
                        final_type = Type::RValueRef { base: Box::new(final_type) };
                    }
                }
                let var_name = self.advance().text.clone();
                self.consume(TokenType::Colon, "range for 预期 ':'");
                let iter = Box::new(self.parse_expression());
                self.consume(TokenType::RParen, "range for 预期 ')'");
                let body = Box::new(self.parse_statement());
                return Stmt::RangeFor {
                    var: var_name,
                    var_type: final_type,
                    iter,
                    body,
                    loc: SourceLoc {
                        line: loc.line,
                        column: loc.column,
                        file_id: 0,
                    },
                };
            }
        }

        let init: Option<Box<Stmt>> = if self.is_type_token() {
            let var_loc = self.current().clone();
            let base_type = self.parse_base_type();
            let (var_type, name) = self.parse_var_declarator(&base_type);
            let init_expr = if self.match_token(TokenType::Assign) {
                if self.check(TokenType::LBrace) {
                    Some(self.parse_init_list())
                } else {
                    Some(self.parse_assign())
                }
            } else {
                None
            };
            let mut extra_vars = Vec::new();
            while self.match_token(TokenType::Comma) {
                let (extra_ty, extra_name) = self.parse_var_declarator(&base_type);
                let extra_init = if self.match_token(TokenType::Assign) {
                    if self.check(TokenType::LBrace) {
                        Some(self.parse_init_list())
                    } else {
                        Some(self.parse_assign())
                    }
                } else {
                    None
                };
                extra_vars.push((extra_ty, extra_name, extra_init));
            }
            Some(Box::new(Stmt::VarDecl {
                var_type,
                name,
                init: init_expr,
                extra_vars,
                is_static: false,
                loc: SourceLoc {
                    line: var_loc.line,
                    column: var_loc.column,
                    file_id: 0,
                },
            }))
        } else if !self.check(TokenType::Semicolon) {
            let es_loc = self.current().clone();
            let mut exprs = vec![self.parse_expression()];
            while self.match_token(TokenType::Comma) {
                exprs.push(self.parse_expression());
            }
            let stmt = if exprs.len() == 1 {
                Stmt::Expr {
                    expr: exprs.remove(0),
                    loc: SourceLoc {
                        line: es_loc.line,
                        column: es_loc.column,
                        file_id: 0,
                    },
                }
            } else {
                Stmt::Block {
                    stmts: exprs
                        .into_iter()
                        .map(|e| Stmt::Expr {
                            expr: e,
                            loc: SourceLoc {
                                line: es_loc.line,
                                column: es_loc.column,
                                file_id: 0,
                            },
                        })
                        .collect(),
                    loc: SourceLoc {
                        line: es_loc.line,
                        column: es_loc.column,
                        file_id: 0,
                    },
                }
            };
            Some(Box::new(stmt))
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "预期 ';'");

        let cond = if !self.check(TokenType::Semicolon) {
            Some(self.parse_expression())
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "预期 ';'");

        let mut step = Vec::new();
        if !self.check(TokenType::RParen) {
            step.push(self.parse_expression());
            while self.match_token(TokenType::Comma) {
                step.push(self.parse_expression());
            }
        }
        self.consume(TokenType::RParen, "预期 ')'");

        let body = Box::new(self.parse_statement());
        Stmt::For {
            init,
            cond,
            step,
            body,
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
                file_id: 0,
            },
        }
    }
    pub(crate) fn parse_return_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.consume(TokenType::Return, "预期 'return'");
        let value = if !self.check(TokenType::Semicolon) {
            Some(self.parse_expression())
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "return 后预期 ';'");
        Stmt::Return {
            value,
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
                file_id: 0,
            },
        }
    }
    pub(crate) fn parse_expr_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        let expr = self.parse_expression();
        self.consume(TokenType::Semicolon, "预期 ';'");
        Stmt::Expr {
            expr,
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
                file_id: 0,
            },
        }
    }
    pub(crate) fn parse_switch_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.advance();
        self.consume(TokenType::LParen, "switch 后预期 '('");
        let cond = self.parse_expression();
        self.consume(TokenType::RParen, "switch 条件后预期 ')'");
        let body = Box::new(self.parse_statement());
        Stmt::Switch {
            cond,
            body,
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
                file_id: 0,
            },
        }
    }
    pub(crate) fn parse_case_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        let label = if self.match_token(TokenType::Case) {
            Some(self.parse_expression())
        } else if self.match_token(TokenType::Default) {
            None
        } else {
            self.errors.push(ParseError {
                message: "预期 'case' 或 'default'".to_string(),
                line: self.current().line,
                column: self.current().column,
                code: ErrorCode::E2004_ExpectedCaseOrDefault as i32,
            });
            return Stmt::Block {
                stmts: Vec::new(),
                loc: SourceLoc {
                    line: loc.line,
                    column: loc.column,
                    file_id: 0,
                },
            };
        };
        self.consume(TokenType::Colon, "case/default 后预期 ':'");
        let mut stmts = Vec::new();
        while !self.check(TokenType::Case)
            && !self.check(TokenType::Default)
            && !self.check(TokenType::RBrace)
            && !self.is_at_end()
        {
            let stmt_checkpoint = self.pos;
            stmts.push(self.parse_statement());
            if self.pos == stmt_checkpoint {
                self.advance();
            }
        }
        let stmt = if stmts.is_empty() {
            Stmt::Block {
                stmts: Vec::new(),
                loc: SourceLoc {
                    line: loc.line,
                    column: loc.column,
                    file_id: 0,
                },
            }
        } else if stmts.len() == 1 {
            stmts.remove(0)
        } else {
            Stmt::Block {
                stmts,
                loc: SourceLoc {
                    line: loc.line,
                    column: loc.column,
                    file_id: 0,
                },
            }
        };
        Stmt::Case {
            label,
            stmt: Box::new(stmt),
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
                file_id: 0,
            },
        }
    }
}
