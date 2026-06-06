use super::*;

impl Parser {
    pub(crate) fn parse_expression(&mut self) -> Expr {
        self.parse_assign()
    }

    pub(crate) fn parse_assign(&mut self) -> Expr {
        let left = self.parse_ternary();
        let loc = SourceLoc { line: self.previous().line, column: self.previous().column };

        if self.match_token(TokenType::Assign) {
            let right = self.parse_assign();
            return Expr::Assign { op: AssignOp::Assign, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
        }
        if self.match_token(TokenType::PlusAssign) {
            let right = self.parse_assign();
            return Expr::Assign { op: AssignOp::AddAssign, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
        }
        if self.match_token(TokenType::MinusAssign) {
            let right = self.parse_assign();
            return Expr::Assign { op: AssignOp::SubAssign, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
        }
        if self.match_token(TokenType::StarAssign) {
            let right = self.parse_assign();
            return Expr::Assign { op: AssignOp::MulAssign, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
        }
        if self.match_token(TokenType::SlashAssign) {
            let right = self.parse_assign();
            return Expr::Assign { op: AssignOp::DivAssign, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
        }
        if self.match_token(TokenType::PercentAssign) {
            let right = self.parse_assign();
            return Expr::Assign { op: AssignOp::ModAssign, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
        }

        left
    }

    pub(crate) fn parse_ternary(&mut self) -> Expr {
        let cond = self.parse_or();
        if self.match_token(TokenType::Question) {
            let then_branch = self.parse_ternary();
            self.consume(TokenType::Colon, "预期 ':'");
            let else_branch = self.parse_ternary();
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            return Expr::Ternary { cond: Box::new(cond), then_branch: Box::new(then_branch), else_branch: Box::new(else_branch), loc, ty: Type::default() };
        }
        cond
    }

    pub(crate) fn parse_or(&mut self) -> Expr {
        let mut left = self.parse_and();
        while self.match_token(TokenType::OrOr) {
            let right = self.parse_and();
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            left = Expr::Binary { op: BinaryOp::Or, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
        }
        left
    }

    pub(crate) fn parse_and(&mut self) -> Expr {
        let mut left = self.parse_bit_or();
        while self.match_token(TokenType::AndAnd) {
            let right = self.parse_bit_or();
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            left = Expr::Binary { op: BinaryOp::And, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
        }
        left
    }

    pub(crate) fn parse_bit_or(&mut self) -> Expr {
        let mut left = self.parse_bit_xor();
        while self.match_token(TokenType::BitOr) {
            let right = self.parse_bit_xor();
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            left = Expr::Binary { op: BinaryOp::BitOr, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
        }
        left
    }

    pub(crate) fn parse_bit_xor(&mut self) -> Expr {
        let mut left = self.parse_bit_and();
        while self.match_token(TokenType::BitXor) {
            let right = self.parse_bit_and();
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            left = Expr::Binary { op: BinaryOp::BitXor, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
        }
        left
    }

    pub(crate) fn parse_bit_and(&mut self) -> Expr {
        let mut left = self.parse_equality();
        while self.match_token(TokenType::Ampersand) {
            let right = self.parse_equality();
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            left = Expr::Binary { op: BinaryOp::BitAnd, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
        }
        left
    }

    pub(crate) fn parse_equality(&mut self) -> Expr {
        let mut left = self.parse_relational();
        loop {
            if self.match_token(TokenType::Eq) {
                let right = self.parse_relational();
                let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                left = Expr::Binary { op: BinaryOp::Eq, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
            } else if self.match_token(TokenType::Ne) {
                let right = self.parse_relational();
                let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                left = Expr::Binary { op: BinaryOp::Ne, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
            } else { break; }
        }
        left
    }

    pub(crate) fn parse_relational(&mut self) -> Expr {
        let mut left = self.parse_shift();
        loop {
            if self.match_token(TokenType::Lt) {
                let right = self.parse_shift();
                let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                left = Expr::Binary { op: BinaryOp::Lt, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
            } else if self.match_token(TokenType::Le) {
                let right = self.parse_shift();
                let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                left = Expr::Binary { op: BinaryOp::Le, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
            } else if self.match_token(TokenType::Gt) {
                let right = self.parse_shift();
                let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                left = Expr::Binary { op: BinaryOp::Gt, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
            } else if self.match_token(TokenType::Ge) {
                let right = self.parse_shift();
                let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                left = Expr::Binary { op: BinaryOp::Ge, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
            } else { break; }
        }
        left
    }

    pub(crate) fn parse_shift(&mut self) -> Expr {
        let mut left = self.parse_additive();
        loop {
            if self.match_token(TokenType::Shl) {
                let right = self.parse_additive();
                let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                left = Expr::Binary { op: BinaryOp::Shl, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
            } else if self.match_token(TokenType::Shr) {
                let right = self.parse_additive();
                let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                left = Expr::Binary { op: BinaryOp::Shr, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
            } else { break; }
        }
        left
    }

    pub(crate) fn parse_additive(&mut self) -> Expr {
        let mut left = self.parse_multiplicative();
        loop {
            if self.match_token(TokenType::Plus) {
                let right = self.parse_multiplicative();
                let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                left = Expr::Binary { op: BinaryOp::Add, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
            } else if self.match_token(TokenType::Minus) {
                let right = self.parse_multiplicative();
                let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                left = Expr::Binary { op: BinaryOp::Sub, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
            } else { break; }
        }
        left
    }

    pub(crate) fn parse_multiplicative(&mut self) -> Expr {
        let mut left = self.parse_unary();
        loop {
            if self.match_token(TokenType::Star) {
                let right = self.parse_unary();
                let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                left = Expr::Binary { op: BinaryOp::Mul, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
            } else if self.match_token(TokenType::Slash) {
                let right = self.parse_unary();
                let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                left = Expr::Binary { op: BinaryOp::Div, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
            } else if self.match_token(TokenType::Percent) {
                let right = self.parse_unary();
                let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                left = Expr::Binary { op: BinaryOp::Mod, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
            } else { break; }
        }
        left
    }

    pub(crate) fn parse_unary(&mut self) -> Expr {
        if self.match_token(TokenType::Sizeof) {
            return self.parse_sizeof();
        }
        if self.check(TokenType::LParen) {
            let checkpoint = self.pos;
            let typedef_snapshot = self.typedef_names.clone();
            self.advance(); // consume '('
            if self.is_type_token() {
                let t = self.parse_type_only();
                if self.match_token(TokenType::RParen) {
                    let operand = self.parse_unary();
                    let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                    return Expr::Cast { expr: Box::new(operand), target_type: t.clone(), loc, ty: t };
                }
            }
            self.pos = checkpoint;
            self.typedef_names = typedef_snapshot;
        }
        if self.match_token(TokenType::Minus) {
            let operand = self.parse_unary();
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            return Expr::Unary { op: UnaryOp::Neg, operand: Box::new(operand), loc, ty: Type::default() };
        }
        if self.match_token(TokenType::Not) {
            let operand = self.parse_unary();
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            return Expr::Unary { op: UnaryOp::Not, operand: Box::new(operand), loc, ty: Type::default() };
        }
        if self.match_token(TokenType::BitNot) {
            let operand = self.parse_unary();
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            return Expr::Unary { op: UnaryOp::BitNot, operand: Box::new(operand), loc, ty: Type::default() };
        }
        if self.match_token(TokenType::Ampersand) {
            let operand = self.parse_unary();
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            return Expr::Unary { op: UnaryOp::Addr, operand: Box::new(operand), loc, ty: Type::default() };
        }
        if self.match_token(TokenType::Star) {
            let operand = self.parse_unary();
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            return Expr::Unary { op: UnaryOp::Deref, operand: Box::new(operand), loc, ty: Type::default() };
        }
        if self.match_token(TokenType::Increment) {
            let operand = self.parse_unary();
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            return Expr::Unary { op: UnaryOp::PreInc, operand: Box::new(operand), loc, ty: Type::default() };
        }
        if self.match_token(TokenType::Decrement) {
            let operand = self.parse_unary();
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            return Expr::Unary { op: UnaryOp::PreDec, operand: Box::new(operand), loc, ty: Type::default() };
        }
        self.parse_postfix()
    }

    pub(crate) fn parse_abstract_declarator(&mut self) -> Option<DeclaratorNode> {
        let mut guard = DeclaratorGuard::default();
        let (node, _) = self.parse_declarator_node(&mut guard, true);
        if matches!(node, DeclaratorNode::Base) {
            None
        } else {
            Some(node)
        }
    }

    pub(crate) fn parse_sizeof(&mut self) -> Expr {
        let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
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
                return Expr::Sizeof { target_type: Some(t), operand: None, loc, ty: Type::int() };
            }
            self.pos = checkpoint;
            let expr = self.parse_expression();
            self.consume(TokenType::RParen, "sizeof(expr) 后预期 ')'");
            return Expr::Sizeof { target_type: None, operand: Some(Box::new(expr)), loc, ty: Type::int() };
        }
        let expr = self.parse_unary();
        Expr::Sizeof { target_type: None, operand: Some(Box::new(expr)), loc, ty: Type::int() }
    }

    pub(crate) fn parse_type_only(&mut self) -> Type {
        let mut base = self.parse_base_type();
        while self.match_token(TokenType::Star) {
            base = Type::pointer_to(base);
        }
        base
    }

    pub(crate) fn parse_postfix(&mut self) -> Expr {
        let mut expr = self.parse_primary();
        loop {
            if self.match_token(TokenType::LBracket) {
                let index = self.parse_expression();
                self.consume(TokenType::RBracket, "预期 ']'");
                let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                expr = Expr::Index { array: Box::new(expr), index: Box::new(index), loc, ty: Type::default() };
            } else if self.match_token(TokenType::LParen) {
                // Function call: direct named call or function pointer call
                let args = self.parse_arg_list();
                self.consume(TokenType::RParen, "预期 ')'");
                let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                expr = Expr::CallPtr { callee: Box::new(expr), args, loc, ty: Type::default() };
            } else if self.match_token(TokenType::Dot) || self.match_token(TokenType::Arrow) {
                let member_tok = self.consume(TokenType::Identifier, "预期成员名称").clone();
                let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                expr = Expr::Member { object: Box::new(expr), member: member_tok.text, loc, ty: Type::default() };
            } else if self.match_token(TokenType::Increment) {
                let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                expr = Expr::Unary { op: UnaryOp::PostInc, operand: Box::new(expr), loc, ty: Type::default() };
            } else if self.match_token(TokenType::Decrement) {
                let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                expr = Expr::Unary { op: UnaryOp::PostDec, operand: Box::new(expr), loc, ty: Type::default() };
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
                if self.check(TokenType::LBrace) {
                    elements.push(self.parse_init_list());
                } else {
                    elements.push(self.parse_expression());
                }
                if !self.match_token(TokenType::Comma) { break; }
            }
        }
        self.consume(TokenType::RBrace, "初始化列表预期 '}'");
        Expr::InitList { elements, loc: SourceLoc { line: loc.line, column: loc.column }, ty: Type::default() }
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
            let loc = SourceLoc { line: prev.line, column: prev.column };
            return Expr::Literal { value, loc, ty: Type::int() };
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
            let loc = SourceLoc { line: prev.line, column: prev.column };
            return Expr::LongLiteral { value, loc, ty: Type::long_long() };
        }
        if self.match_token(TokenType::FloatLiteral) {
            let prev = self.previous().clone();
            let value: f64 = prev.text.parse().unwrap_or_else(|_| {
                self.errors.push(ParseError {
                    message: format!("浮点常量 '{}' 格式无效", prev.text),
                    line: prev.line,
                    column: prev.column,
                    code: ErrorCode::E1006_UnsupportedFeature as i32,
                });
                0.0
            });
            let loc = SourceLoc { line: prev.line, column: prev.column };
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
            let loc = SourceLoc { line: prev.line, column: prev.column };
            return Expr::Literal { value, loc, ty: Type::char() };
        }
        if self.match_token(TokenType::String) {
            let value = self.previous().text.clone();
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            let array_size = value.len() as i32 + 1; // including null terminator
            return Expr::StringLiteral { value, loc, ty: Type::Array { element: Box::new(Type::char()), array_size, dims: vec![array_size], is_const: false, is_vla: false, vla_dims: vec![] } };
        }
        if self.match_token(TokenType::Null) {
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            return Expr::Literal { value: 0, loc, ty: Type::pointer_to(Type::void()) };
        }
        if self.check(TokenType::Identifier) {
            let name_tok = self.advance().clone();
            let loc = SourceLoc { line: name_tok.line, column: name_tok.column };
            return Expr::Identifier { name: name_tok.text, loc, ty: Type::default() };
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
        let loc = SourceLoc { line: self.current().line, column: self.current().column };
        Expr::Literal { value: 0, loc, ty: Type::int() }
    }

}
