use super::*;

impl Parser {
    pub(crate) fn parse_expression(&mut self) -> Expr {
        self.parse_comma()
    }

    pub(crate) fn parse_comma(&mut self) -> Expr {
        let mut left = self.parse_assign();
        while self.match_token(TokenType::Comma) {
            let right = self.parse_assign();
            let loc = SourceLoc {
                line: self.previous().line,
                column: self.previous().column,
            };
            left = Expr::Binary {
                op: BinaryOp::Comma,
                left: Box::new(left),
                right: Box::new(right),
                loc,
                ty: Type::default(),
            };
        }
        left
    }

    pub(crate) fn parse_assign(&mut self) -> Expr {
        let left = self.parse_ternary();
        let loc = SourceLoc {
            line: self.previous().line,
            column: self.previous().column,
        };

        if self.match_token(TokenType::Assign) {
            let right = self.parse_assign();
            return Expr::Assign {
                op: AssignOp::Assign,
                left: Box::new(left),
                right: Box::new(right),
                loc,
                ty: Type::default(),
            };
        }
        if self.match_token(TokenType::PlusAssign) {
            let right = self.parse_assign();
            return Expr::Assign {
                op: AssignOp::AddAssign,
                left: Box::new(left),
                right: Box::new(right),
                loc,
                ty: Type::default(),
            };
        }
        if self.match_token(TokenType::MinusAssign) {
            let right = self.parse_assign();
            return Expr::Assign {
                op: AssignOp::SubAssign,
                left: Box::new(left),
                right: Box::new(right),
                loc,
                ty: Type::default(),
            };
        }
        if self.match_token(TokenType::StarAssign) {
            let right = self.parse_assign();
            return Expr::Assign {
                op: AssignOp::MulAssign,
                left: Box::new(left),
                right: Box::new(right),
                loc,
                ty: Type::default(),
            };
        }
        if self.match_token(TokenType::SlashAssign) {
            let right = self.parse_assign();
            return Expr::Assign {
                op: AssignOp::DivAssign,
                left: Box::new(left),
                right: Box::new(right),
                loc,
                ty: Type::default(),
            };
        }
        if self.match_token(TokenType::PercentAssign) {
            let right = self.parse_assign();
            return Expr::Assign {
                op: AssignOp::ModAssign,
                left: Box::new(left),
                right: Box::new(right),
                loc,
                ty: Type::default(),
            };
        }
        if self.match_token(TokenType::AndAssign) {
            let right = self.parse_assign();
            return Expr::Assign {
                op: AssignOp::AndAssign,
                left: Box::new(left),
                right: Box::new(right),
                loc,
                ty: Type::default(),
            };
        }
        if self.match_token(TokenType::OrAssign) {
            let right = self.parse_assign();
            return Expr::Assign {
                op: AssignOp::OrAssign,
                left: Box::new(left),
                right: Box::new(right),
                loc,
                ty: Type::default(),
            };
        }
        if self.match_token(TokenType::XorAssign) {
            let right = self.parse_assign();
            return Expr::Assign {
                op: AssignOp::XorAssign,
                left: Box::new(left),
                right: Box::new(right),
                loc,
                ty: Type::default(),
            };
        }
        if self.match_token(TokenType::ShlAssign) {
            let right = self.parse_assign();
            return Expr::Assign {
                op: AssignOp::ShlAssign,
                left: Box::new(left),
                right: Box::new(right),
                loc,
                ty: Type::default(),
            };
        }
        if self.match_token(TokenType::ShrAssign) {
            let right = self.parse_assign();
            return Expr::Assign {
                op: AssignOp::ShrAssign,
                left: Box::new(left),
                right: Box::new(right),
                loc,
                ty: Type::default(),
            };
        }

        left
    }

    pub(crate) fn parse_ternary(&mut self) -> Expr {
        let cond = self.parse_or();
        if self.match_token(TokenType::Question) {
            let then_branch = self.parse_ternary();
            self.consume(TokenType::Colon, "预期 ':'");
            let else_branch = self.parse_ternary();
            let loc = SourceLoc {
                line: self.previous().line,
                column: self.previous().column,
            };
            return Expr::Ternary {
                cond: Box::new(cond),
                then_branch: Box::new(then_branch),
                else_branch: Box::new(else_branch),
                loc,
                ty: Type::default(),
            };
        }
        cond
    }

    pub(crate) fn parse_or(&mut self) -> Expr {
        let mut left = self.parse_and();
        while self.match_token(TokenType::OrOr) {
            let right = self.parse_and();
            let loc = SourceLoc {
                line: self.previous().line,
                column: self.previous().column,
            };
            left = Expr::Binary {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
                loc,
                ty: Type::default(),
            };
        }
        left
    }

    pub(crate) fn parse_and(&mut self) -> Expr {
        let mut left = self.parse_bit_or();
        while self.match_token(TokenType::AndAnd) {
            let right = self.parse_bit_or();
            let loc = SourceLoc {
                line: self.previous().line,
                column: self.previous().column,
            };
            left = Expr::Binary {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
                loc,
                ty: Type::default(),
            };
        }
        left
    }

    pub(crate) fn parse_bit_or(&mut self) -> Expr {
        let mut left = self.parse_bit_xor();
        while self.match_token(TokenType::BitOr) {
            let right = self.parse_bit_xor();
            let loc = SourceLoc {
                line: self.previous().line,
                column: self.previous().column,
            };
            left = Expr::Binary {
                op: BinaryOp::BitOr,
                left: Box::new(left),
                right: Box::new(right),
                loc,
                ty: Type::default(),
            };
        }
        left
    }

    pub(crate) fn parse_bit_xor(&mut self) -> Expr {
        let mut left = self.parse_bit_and();
        while self.match_token(TokenType::BitXor) {
            let right = self.parse_bit_and();
            let loc = SourceLoc {
                line: self.previous().line,
                column: self.previous().column,
            };
            left = Expr::Binary {
                op: BinaryOp::BitXor,
                left: Box::new(left),
                right: Box::new(right),
                loc,
                ty: Type::default(),
            };
        }
        left
    }

    pub(crate) fn parse_bit_and(&mut self) -> Expr {
        let mut left = self.parse_equality();
        while self.match_token(TokenType::Ampersand) {
            let right = self.parse_equality();
            let loc = SourceLoc {
                line: self.previous().line,
                column: self.previous().column,
            };
            left = Expr::Binary {
                op: BinaryOp::BitAnd,
                left: Box::new(left),
                right: Box::new(right),
                loc,
                ty: Type::default(),
            };
        }
        left
    }

    pub(crate) fn parse_equality(&mut self) -> Expr {
        let mut left = self.parse_relational();
        loop {
            if self.match_token(TokenType::Eq) {
                let right = self.parse_relational();
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                };
                left = Expr::Binary {
                    op: BinaryOp::Eq,
                    left: Box::new(left),
                    right: Box::new(right),
                    loc,
                    ty: Type::default(),
                };
            } else if self.match_token(TokenType::Ne) {
                let right = self.parse_relational();
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                };
                left = Expr::Binary {
                    op: BinaryOp::Ne,
                    left: Box::new(left),
                    right: Box::new(right),
                    loc,
                    ty: Type::default(),
                };
            } else {
                break;
            }
        }
        left
    }

    pub(crate) fn parse_relational(&mut self) -> Expr {
        let mut left = self.parse_shift();
        loop {
            if self.match_token(TokenType::Lt) {
                let right = self.parse_shift();
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                };
                left = Expr::Binary {
                    op: BinaryOp::Lt,
                    left: Box::new(left),
                    right: Box::new(right),
                    loc,
                    ty: Type::default(),
                };
            } else if self.match_token(TokenType::Le) {
                let right = self.parse_shift();
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                };
                left = Expr::Binary {
                    op: BinaryOp::Le,
                    left: Box::new(left),
                    right: Box::new(right),
                    loc,
                    ty: Type::default(),
                };
            } else if self.match_token(TokenType::Gt) {
                let right = self.parse_shift();
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                };
                left = Expr::Binary {
                    op: BinaryOp::Gt,
                    left: Box::new(left),
                    right: Box::new(right),
                    loc,
                    ty: Type::default(),
                };
            } else if self.match_token(TokenType::Ge) {
                let right = self.parse_shift();
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                };
                left = Expr::Binary {
                    op: BinaryOp::Ge,
                    left: Box::new(left),
                    right: Box::new(right),
                    loc,
                    ty: Type::default(),
                };
            } else {
                break;
            }
        }
        left
    }

    pub(crate) fn parse_shift(&mut self) -> Expr {
        let mut left = self.parse_additive();
        loop {
            if self.match_token(TokenType::Shl) {
                let right = self.parse_additive();
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                };
                left = Expr::Binary {
                    op: BinaryOp::Shl,
                    left: Box::new(left),
                    right: Box::new(right),
                    loc,
                    ty: Type::default(),
                };
            } else if self.match_token(TokenType::Shr) {
                let right = self.parse_additive();
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                };
                left = Expr::Binary {
                    op: BinaryOp::Shr,
                    left: Box::new(left),
                    right: Box::new(right),
                    loc,
                    ty: Type::default(),
                };
            } else {
                break;
            }
        }
        left
    }

    pub(crate) fn parse_additive(&mut self) -> Expr {
        let mut left = self.parse_multiplicative();
        loop {
            if self.match_token(TokenType::Plus) {
                let right = self.parse_multiplicative();
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                };
                left = Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(left),
                    right: Box::new(right),
                    loc,
                    ty: Type::default(),
                };
            } else if self.match_token(TokenType::Minus) {
                let right = self.parse_multiplicative();
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                };
                left = Expr::Binary {
                    op: BinaryOp::Sub,
                    left: Box::new(left),
                    right: Box::new(right),
                    loc,
                    ty: Type::default(),
                };
            } else {
                break;
            }
        }
        left
    }

    pub(crate) fn parse_multiplicative(&mut self) -> Expr {
        let mut left = self.parse_unary();
        loop {
            if self.match_token(TokenType::Star) {
                let right = self.parse_unary();
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                };
                left = Expr::Binary {
                    op: BinaryOp::Mul,
                    left: Box::new(left),
                    right: Box::new(right),
                    loc,
                    ty: Type::default(),
                };
            } else if self.match_token(TokenType::Slash) {
                let right = self.parse_unary();
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                };
                left = Expr::Binary {
                    op: BinaryOp::Div,
                    left: Box::new(left),
                    right: Box::new(right),
                    loc,
                    ty: Type::default(),
                };
            } else if self.match_token(TokenType::Percent) {
                let right = self.parse_unary();
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                };
                left = Expr::Binary {
                    op: BinaryOp::Mod,
                    left: Box::new(left),
                    right: Box::new(right),
                    loc,
                    ty: Type::default(),
                };
            } else {
                break;
            }
        }
        left
    }

    pub(crate) fn parse_unary(&mut self) -> Expr {
        if self.match_token(TokenType::Sizeof) {
            return self.parse_sizeof();
        }
        if self.match_token(TokenType::Offsetof) {
            return self.parse_offsetof();
        }
        if self.is_cpp_mode && self.match_token(TokenType::New) {
            return self.parse_new_expr();
        }
        if self.is_cpp_mode && self.match_token(TokenType::Delete) {
            return self.parse_delete_expr();
        }
        if self.check(TokenType::LParen) {
            let checkpoint = self.pos;
            let typedef_snapshot = self.typedef_names.clone();
            self.advance(); // consume '('
            if self.is_type_token() {
                let t = self.parse_type_only();
                if self.match_token(TokenType::RParen) {
                    let operand = self.parse_unary();
                    let loc = SourceLoc {
                        line: self.previous().line,
                        column: self.previous().column,
                    };
                    return Expr::Cast {
                        expr: Box::new(operand),
                        target_type: t.clone(),
                        loc,
                        ty: t,
                    };
                }
            }
            self.pos = checkpoint;
            self.typedef_names = typedef_snapshot;
        }
        if self.match_token(TokenType::Minus) {
            let operand = self.parse_unary();
            let loc = SourceLoc {
                line: self.previous().line,
                column: self.previous().column,
            };
            return Expr::Unary {
                op: UnaryOp::Neg,
                operand: Box::new(operand),
                loc,
                ty: Type::default(),
            };
        }
        if self.match_token(TokenType::Not) {
            let operand = self.parse_unary();
            let loc = SourceLoc {
                line: self.previous().line,
                column: self.previous().column,
            };
            return Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(operand),
                loc,
                ty: Type::default(),
            };
        }
        if self.match_token(TokenType::BitNot) {
            let operand = self.parse_unary();
            let loc = SourceLoc {
                line: self.previous().line,
                column: self.previous().column,
            };
            return Expr::Unary {
                op: UnaryOp::BitNot,
                operand: Box::new(operand),
                loc,
                ty: Type::default(),
            };
        }
        if self.match_token(TokenType::Ampersand) {
            let operand = self.parse_unary();
            let loc = SourceLoc {
                line: self.previous().line,
                column: self.previous().column,
            };
            return Expr::Unary {
                op: UnaryOp::Addr,
                operand: Box::new(operand),
                loc,
                ty: Type::default(),
            };
        }
        if self.match_token(TokenType::Star) {
            let operand = self.parse_unary();
            let loc = SourceLoc {
                line: self.previous().line,
                column: self.previous().column,
            };
            return Expr::Unary {
                op: UnaryOp::Deref,
                operand: Box::new(operand),
                loc,
                ty: Type::default(),
            };
        }
        if self.match_token(TokenType::Increment) {
            let operand = self.parse_unary();
            let loc = SourceLoc {
                line: self.previous().line,
                column: self.previous().column,
            };
            return Expr::Unary {
                op: UnaryOp::PreInc,
                operand: Box::new(operand),
                loc,
                ty: Type::default(),
            };
        }
        if self.match_token(TokenType::Decrement) {
            let operand = self.parse_unary();
            let loc = SourceLoc {
                line: self.previous().line,
                column: self.previous().column,
            };
            return Expr::Unary {
                op: UnaryOp::PreDec,
                operand: Box::new(operand),
                loc,
                ty: Type::default(),
            };
        }
        self.parse_postfix()
    }

    pub(crate) fn parse_abstract_declarator(&mut self) -> Option<DeclaratorNode> {
        let mut guard = DeclaratorGuard::default();
        let (node, _) = self.parse_declarator_node(&mut guard, true, true);
        if matches!(node, DeclaratorNode::Base) {
            None
        } else {
            Some(node)
        }
    }

    pub(crate) fn parse_sizeof(&mut self) -> Expr {
        let loc = SourceLoc {
            line: self.previous().line,
            column: self.previous().column,
        };
        if self.match_token(TokenType::LParen) {
            let checkpoint = self.pos;
            let mut is_type = false;
            let mut t = Type::default();
            if self.is_type_token() {
                t = self.parse_base_type();
                if let Some(node) = self.parse_abstract_declarator() {
                    t = Self::interpret_declarator_node(&node, &t);
                }
                if self.check(TokenType::RParen) {
                    is_type = true;
                }
            }
            if is_type {
                self.consume(TokenType::RParen, "sizeof(type) 后预期 ')'");
                return Expr::Sizeof {
                    target_type: Some(t),
                    operand: None,
                    loc,
                    ty: Type::int(),
                };
            }
            self.pos = checkpoint;
            let expr = self.parse_expression();
            self.consume(TokenType::RParen, "sizeof(expr) 后预期 ')'");
            return Expr::Sizeof {
                target_type: None,
                operand: Some(Box::new(expr)),
                loc,
                ty: Type::int(),
            };
        }
        let expr = self.parse_unary();
        Expr::Sizeof {
            target_type: None,
            operand: Some(Box::new(expr)),
            loc,
            ty: Type::int(),
        }
    }

    pub(crate) fn parse_offsetof(&mut self) -> Expr {
        let loc = SourceLoc {
            line: self.previous().line,
            column: self.previous().column,
        };
        self.consume(TokenType::LParen, "offsetof 后预期 '('");
        let target_type = self.parse_base_type();
        self.consume(TokenType::Comma, "offsetof 参数之间预期 ','");
        let field_tok = self.consume(TokenType::Identifier, "offsetof 预期字段名").clone();
        self.consume(TokenType::RParen, "offsetof 后预期 ')'");
        Expr::Offsetof {
            target_type,
            field: field_tok.text,
            loc,
            ty: Type::int(),
        }
    }

    // =========================================================================
    // C++ Expression Parsers (Phase 31)
    // =========================================================================

    fn parse_new_expr(&mut self) -> Expr {
        let loc = SourceLoc {
            line: self.previous().line,
            column: self.previous().column,
        };
        let elem_type = self.parse_base_type();
        let mut final_type = elem_type.clone();
        while self.match_token(TokenType::Star) {
            final_type = Type::pointer_to(final_type);
        }

        let mut size_expr = None;
        let mut init = None;

        if self.match_token(TokenType::LBracket) {
            size_expr = Some(Box::new(self.parse_expression()));
            self.consume(TokenType::RBracket, "new[] 预期 ']'");
        }

        if self.match_token(TokenType::LParen) {
            if self.check(TokenType::RParen) {
                // Empty parens: treat as default constructor call
                init = Some(Box::new(Expr::Call {
                    name: format!("__ctor__{}", elem_type.name()),
                    args: Vec::new(),
                    loc,
                    ty: Type::void(),
                }));
            } else {
                let is_class = matches!(elem_type.kind(), TypeKind::Class | TypeKind::TemplateId);
                if is_class {
                    let ctor_args = self.parse_arg_list();
                    init = Some(Box::new(Expr::Call {
                        name: format!("__ctor__{}__{}", elem_type.name(), ctor_args.len()),
                        args: ctor_args,
                        loc,
                        ty: Type::void(),
                    }));
                } else {
                    init = Some(Box::new(self.parse_expression()));
                }
            }
            self.consume(TokenType::RParen, "new 初始化预期 ')'");
        }

        Expr::New {
            elem_type: final_type,
            size_expr,
            init,
            loc,
            ty: Type::pointer_to(elem_type),
        }
    }

    fn parse_delete_expr(&mut self) -> Expr {
        let loc = SourceLoc {
            line: self.previous().line,
            column: self.previous().column,
        };
        let mut is_array = false;
        if self.check(TokenType::LBracket) {
            self.advance(); // [
            if self.check(TokenType::RBracket) {
                self.advance(); // ]
                is_array = true;
            } else {
                // 回退，当作普通表达式中的 [ 处理
                // 但这里不太可能，因为 delete [ 后面应该总是 ]
            }
        }
        let expr = Box::new(self.parse_unary());
        Expr::Delete {
            expr,
            is_array,
            loc,
            ty: Type::void(),
        }
    }

    fn parse_lambda_expr(&mut self) -> Expr {
        let loc = SourceLoc {
            line: self.current().line,
            column: self.current().column,
        };
        self.consume(TokenType::LBracket, "Lambda 预期 '['");
        let mut capture = Vec::new();
        // 支持 []、[x]、[&x]、[=]、[&]、[a, &b]、[this]、[=, &a]、[&, a]
        if !self.check(TokenType::RBracket) {
            loop {
                if self.check(TokenType::Assign) {
                    self.advance(); // =
                    capture.push(CaptureMode::Implicit);
                } else if self.check(TokenType::Ampersand) {
                    self.advance(); // &
                    if self.check(TokenType::Identifier) {
                        let name = self.current().text.clone();
                        capture.push(CaptureMode::ByReference(name));
                        self.advance();
                    } else {
                        capture.push(CaptureMode::Implicit);
                    }
                } else if self.check(TokenType::Identifier) {
                    let name = self.current().text.clone();
                    capture.push(CaptureMode::ByValue(name));
                    self.advance();
                } else {
                    break;
                }
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RBracket, "Lambda 预期 ']'");
        self.consume(TokenType::LParen, "Lambda 预期 '('");
        let params = self.parse_param_list();
        self.consume(TokenType::RParen, "Lambda 预期 ')'");

        // 可选的返回类型 -> Type
        if self.check(TokenType::Minus) && self.peek(1).ty == TokenType::Gt {
            self.advance(); // -
            self.advance(); // >
            let _ret_type = self.parse_base_type();
            // 简化：忽略显式返回类型，由 TypeChecker 推断
        }

        let body = Box::new(self.parse_statement());
        let id = self.next_lambda_id;
        self.next_lambda_id += 1;
        Expr::Lambda {
            capture,
            params,
            body,
            unique_id: id,
            loc,
            ty: Type::default(),
        }
    }

    pub(crate) fn parse_type_only(&mut self) -> Type {
        let mut base = self.parse_base_type();
        if let Some(node) = self.parse_abstract_declarator() {
            base = Self::interpret_declarator_node(&node, &base);
        }
        base
    }

    pub(crate) fn parse_postfix(&mut self) -> Expr {
        let mut expr = self.parse_primary();
        loop {
            if self.match_token(TokenType::LBracket) {
                let index = self.parse_expression();
                self.consume(TokenType::RBracket, "预期 ']'");
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                };
                expr = Expr::Index {
                    array: Box::new(expr),
                    index: Box::new(index),
                    loc,
                    ty: Type::default(),
                };
            } else if self.match_token(TokenType::LParen) {
                // Function call: direct named call or function pointer call
                let args = self.parse_arg_list();
                self.consume(TokenType::RParen, "预期 ')'");
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                };
                expr = Expr::CallPtr {
                    callee: Box::new(expr),
                    args,
                    loc,
                    ty: Type::default(),
                };
            } else if self.match_token(TokenType::Dot) || self.match_token(TokenType::Arrow) {
                let member_tok = self.consume(TokenType::Identifier, "预期成员名称").clone();
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                };
                // C++ 模式下检查是否是方法调用：obj.method(args)
                if self.is_cpp_mode && self.check(TokenType::LParen) {
                    self.advance(); // consume '('
                    let args = self.parse_arg_list();
                    self.consume(TokenType::RParen, "预期 ')'");
                    expr = Expr::MemberCall {
                        object: Box::new(expr),
                        method: member_tok.text,
                        args,
                        is_virtual: false,
                        resolved_mangled: None,
                        loc,
                        ty: Type::default(),
                    };
                } else {
                    expr = Expr::Member {
                        object: Box::new(expr),
                        member: member_tok.text,
                        loc,
                        ty: Type::default(),
                    };
                }
            } else if self.match_token(TokenType::Increment) {
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                };
                expr = Expr::Unary {
                    op: UnaryOp::PostInc,
                    operand: Box::new(expr),
                    loc,
                    ty: Type::default(),
                };
            } else if self.match_token(TokenType::Decrement) {
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                };
                expr = Expr::Unary {
                    op: UnaryOp::PostDec,
                    operand: Box::new(expr),
                    loc,
                    ty: Type::default(),
                };
            } else {
                break;
            }
        }
        expr
    }

    pub(crate) fn parse_init_list(&mut self) -> Expr {
        let loc = self.current().clone();
        self.consume(TokenType::LBrace, "初始化列表预期 '{'");
        let mut elements = Vec::new();
        if !self.check(TokenType::RBrace) {
            loop {
                let mut designators = Vec::new();
                // Designated initializer: .field = value or [index] = value
                if self.match_token(TokenType::Dot) {
                    let field_tok = self.consume(TokenType::Identifier, "Designator 预期字段名").clone();
                    designators.push(Designator::Field(field_tok.text));
                    self.consume(TokenType::Assign, "Designated initializer 预期 '='");
                } else if self.match_token(TokenType::LBracket) {
                    let idx_expr = self.parse_assign();
                    self.consume(TokenType::RBracket, "Designator 预期 ']'");
                    designators.push(Designator::Index(Box::new(idx_expr)));
                    self.consume(TokenType::Assign, "Designated initializer 预期 '='");
                }
                let value = if self.check(TokenType::LBrace) {
                    self.parse_init_list()
                } else {
                    self.parse_assign()
                };
                elements.push(InitElement { designators, value });
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RBrace, "初始化列表预期 '}'");
        Expr::InitList {
            elements,
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
            },
            ty: Type::default(),
        }
    }

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
            };
            return Expr::Literal { value, loc, ty: Type::char() };
        }
        if self.match_token(TokenType::String) {
            let value = self.previous().text.clone();
            let loc = SourceLoc {
                line: self.previous().line,
                column: self.previous().column,
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
            };
            return Expr::This { loc, ty: Type::default() };
        }
        if self.is_cpp_mode && self.check(TokenType::LBracket) {
            return self.parse_lambda_expr();
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
        };
        // 消费当前 token，防止外层 parse_statement 在相同位置无限循环。
        if !self.is_at_end() {
            self.advance();
        }
        Expr::Literal { value: 0, loc, ty: Type::int() }
    }
}
