use crate::compiler::ast::*;
use crate::compiler::lexer::{Token, TokenType};
use crate::diagnostics::error_codes::ErrorCode;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: i32,
    pub column: i32,
    pub code: i32,
}

#[derive(Debug, Clone)]
enum DeclaratorNode {
    Base,
    Pointer(Box<DeclaratorNode>),
    Array(Box<DeclaratorNode>, i32),
    Function(Box<DeclaratorNode>, Vec<Param>),
}

#[derive(Debug, Clone)]
enum DeclaratorSuffix {
    Array(i32),
    Function(Vec<Param>),
}

#[derive(Debug, Clone, Default)]
struct DeclaratorGuard {
    paren_depth: i32,
    ptr_count: i32,
    suffix_count: i32,
    cross_count: i32,
}

fn node_cross_count(node: &DeclaratorNode) -> i32 {
    match node {
        DeclaratorNode::Base => 0,
        DeclaratorNode::Pointer(inner) => {
            let add = match inner.as_ref() {
                DeclaratorNode::Array(_, _) | DeclaratorNode::Function(_, _) => 1,
                _ => 0,
            };
            node_cross_count(inner) + add
        }
        DeclaratorNode::Array(inner, _) | DeclaratorNode::Function(inner, _) => {
            node_cross_count(inner)
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    typedef_names: HashMap<String, Type>,
    pos: usize,
    anonymous_structs: Vec<StructDecl>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut typedef_names = HashMap::new();
        // 预定义 FILE 类型（stdio.h 中的不透明结构体指针）
        typedef_names.insert("FILE".to_string(), Type::void());
        Self {
            tokens,
            errors: Vec::new(),
            typedef_names,
            pos: 0,
            anonymous_structs: Vec::new(),
        }
    }

    pub fn parse(mut self) -> (Option<ProgramNode>, Vec<ParseError>) {
        let prog = match self.parse_program() {
            Some(mut p) => {
                p.structs.append(&mut self.anonymous_structs);
                p
            }
            None => return (None, self.errors),
        };
        (Some(prog), self.errors)
    }

    // =========================================================================
    // Token helpers
    // =========================================================================

    fn peek(&self, offset: usize) -> &Token {
        if self.pos + offset >= self.tokens.len() {
            static EOF: Token = Token { ty: TokenType::Eof, text: String::new(), line: -1, column: -1 };
            return &EOF;
        }
        &self.tokens[self.pos + offset]
    }

    fn current(&self) -> &Token { self.peek(0) }
    fn previous(&self) -> &Token {
        if self.pos == 0 { return self.peek(0); }
        &self.tokens[self.pos - 1]
    }

    fn check(&self, ty: TokenType) -> bool {
        if self.is_at_end() { return false; }
        self.current().ty == ty
    }

    fn is_at_end(&self) -> bool {
        self.current().ty == TokenType::Eof
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() { self.pos += 1; }
        if self.pos == 0 {
            return self.peek(0);
        }
        &self.tokens[self.pos - 1]
    }

    fn match_token(&mut self, ty: TokenType) -> bool {
        if self.check(ty) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, ty: TokenType, msg: &str) -> &Token {
        if self.check(ty) {
            return self.advance();
        }
        let code = match ty {
            TokenType::RBrace => ErrorCode::E2006_ExpectedClosingBrace,
            TokenType::RParen => ErrorCode::E2007_ExpectedClosingParen,
            TokenType::RBracket => ErrorCode::E2008_ExpectedClosingBracket,
            _ => ErrorCode::E2005_ExpectedSemicolon,
        };
        // For missing semicolon, report at the previous token's position
        // (where the semicolon should have been) rather than the next token.
        let (err_line, err_column) = if ty == TokenType::Semicolon && self.pos > 0 {
            (self.previous().line, self.previous().column)
        } else {
            (self.current().line, self.current().column)
        };
        self.errors.push(ParseError {
            message: msg.to_string(),
            line: err_line,
            column: err_column,
            code: code as i32,
        });
        // 自动恢复：根据缺失的闭合符号推断恢复集合，减少级联错误
        let recovery = match ty {
            TokenType::RParen => &[TokenType::RParen, TokenType::Semicolon][..],
            TokenType::RBrace => &[TokenType::RBrace, TokenType::Semicolon][..],
            TokenType::RBracket => &[TokenType::RBracket, TokenType::Semicolon][..],
            _ => &[TokenType::Semicolon][..],
        };
        self.synchronize(recovery);
        self.peek(0)
    }

    /// 通用同步：跳过 token 直到遇到恢复集合中的 token 或语句边界。
    fn synchronize(&mut self, recovery_set: &[TokenType]) {
        while !self.is_at_end() {
            let current = self.current().ty;
            if recovery_set.contains(&current) {
                return;
            }
            if self.previous().ty == TokenType::Semicolon { return; }
            match current {
                TokenType::Int | TokenType::Void | TokenType::Char | TokenType::Float | TokenType::Double |
                TokenType::If | TokenType::While | TokenType::Do | TokenType::For |
                TokenType::Return | TokenType::Break | TokenType::Continue |
                TokenType::Struct | TokenType::Switch | TokenType::Case |
                TokenType::Default | TokenType::Typedef | TokenType::Enum |
                TokenType::Unsigned | TokenType::Long | TokenType::Short |
                TokenType::Signed | TokenType::Const | TokenType::RBrace => return,
                _ => { self.advance(); }
            }
        }
    }

    fn is_type_token(&self) -> bool {
        if self.check(TokenType::Int) || self.check(TokenType::Void) ||
           self.check(TokenType::Char) || self.check(TokenType::Float) || self.check(TokenType::Double) || self.check(TokenType::Struct) ||
           self.check(TokenType::Enum) || self.check(TokenType::Unsigned) ||
           self.check(TokenType::Long) || self.check(TokenType::Short) ||
           self.check(TokenType::Signed) || self.check(TokenType::Const) ||
           self.check(TokenType::Union) {
            return true;
        }
        if self.check(TokenType::Identifier) {
            return self.typedef_names.contains_key(&self.current().text);
        }
        false
    }

    fn is_static_token(&self) -> bool {
        self.check(TokenType::Identifier) && self.current().text == "static"
    }

    // =========================================================================
    // Program
    // =========================================================================

    fn parse_program(&mut self) -> Option<ProgramNode> {
        let mut program = ProgramNode::default();

        while !self.is_at_end() {
            if self.check(TokenType::Typedef) {
                let checkpoint = self.pos;
                let errors_checkpoint = self.errors.len();
                self.advance();
                if self.check(TokenType::Struct) {
                    let s_checkpoint = self.pos;
                    let s_errors_checkpoint = self.errors.len();
                    self.advance();
                    if self.check(TokenType::Identifier) {
                        self.advance();
                    }
                    if self.check(TokenType::LBrace) {
                        self.pos = checkpoint;
                        self.errors.truncate(errors_checkpoint);
                        self.parse_typedef_struct_decl(&mut program);
                        continue;
                    }
                    self.pos = s_checkpoint;
                    self.errors.truncate(s_errors_checkpoint);
                }
                self.pos = checkpoint;
                self.errors.truncate(errors_checkpoint);
                self.parse_typedef();
            } else if self.check(TokenType::Enum) {
                let checkpoint = self.pos;
                let errors_checkpoint = self.errors.len();
                self.advance();
                self.consume(TokenType::Identifier, "预期 enum 名称");
                let is_enum_decl = self.check(TokenType::LBrace);
                self.pos = checkpoint;
                self.errors.truncate(errors_checkpoint);
                if is_enum_decl {
                    self.parse_enum_decl(&mut program);
                } else {
                    self.parse_global_var_or_func(&mut program, false);
                }
            } else if self.check(TokenType::Struct) {
                let checkpoint = self.pos;
                let errors_checkpoint = self.errors.len();
                self.advance();
                if self.check(TokenType::Identifier) {
                    self.advance();
                }
                if self.check(TokenType::LBrace) {
                    // 向前跳过结构体体，判断后面是变量声明还是纯类型定义
                    let mut brace_depth = 0;
                    while !self.is_at_end() {
                        if self.check(TokenType::LBrace) {
                            brace_depth += 1;
                        } else if self.check(TokenType::RBrace) {
                            brace_depth -= 1;
                            self.advance();
                            if brace_depth == 0 { break; }
                            continue;
                        }
                        self.advance();
                    }
                    let is_var_decl = self.check(TokenType::Identifier)
                        || self.check(TokenType::Star)
                        || self.check(TokenType::Semicolon);
                    // 纯 struct { ... };（无变量名）仍视为结构体定义
                    let is_pure_decl = self.check(TokenType::Semicolon);
                    self.pos = checkpoint;
                    self.errors.truncate(errors_checkpoint);
                    if is_var_decl && !is_pure_decl {
                        self.parse_global_var_or_func(&mut program, false);
                    } else {
                        program.structs.push(self.parse_struct_decl());
                    }
                } else {
                    self.pos = checkpoint;
                    self.errors.truncate(errors_checkpoint);
                    self.parse_global_var_or_func(&mut program, false);
                }
            } else if self.check(TokenType::Union) {
                program.unions.push(self.parse_union_decl());
            } else if self.is_type_token() || self.is_static_token() {
                let is_static = self.is_static_token();
                if is_static {
                    self.advance(); // consume 'static'
                }
                self.parse_global_var_or_func(&mut program, is_static);
            } else {
                self.errors.push(ParseError {
                    message: format!("预期 struct、函数或全局变量声明，找到: {}", self.current().text),
                    line: self.current().line,
                    column: self.current().column,
                    code: ErrorCode::E2005_ExpectedSemicolon as i32,
                });
                self.advance();
            }
        }

        Some(program)
    }

    /// 从当前位置前瞻，跳过连续的 `*` 指针前缀，返回跳过后的 token 索引。
    fn look_ahead_skip_stars(&self) -> usize {
        let mut lookahead = self.pos;
        while lookahead < self.tokens.len() && self.tokens[lookahead].ty == TokenType::Star {
            lookahead += 1;
        }
        lookahead
    }

    fn parse_global_var_or_func(&mut self, program: &mut ProgramNode, is_static: bool) {
        let checkpoint = self.pos;
        let base_type = self.parse_base_type();
        // 前瞻：跳过前导 *，检查是否是 identifier (
        let lookahead = self.look_ahead_skip_stars();
        let is_func_decl = lookahead < self.tokens.len()
            && self.tokens[lookahead].ty == TokenType::Identifier
            && lookahead + 1 < self.tokens.len()
            && self.tokens[lookahead + 1].ty == TokenType::LParen;
        if is_func_decl {
            self.pos = checkpoint;
            program.funcs.push(self.parse_func_decl(is_static));
        } else {
            let (ty, name) = self.parse_declarator(&base_type);
            let init = if self.match_token(TokenType::Assign) {
                if self.check(TokenType::LBrace) {
                    Some(self.parse_init_list())
                } else {
                    Some(self.parse_expression())
                }
            } else { None };
            program.globals.push(GlobalDecl {
                loc: SourceLoc { line: self.previous().line, column: self.previous().column },
                ty: ty.clone(), name, init,
                is_static,
                source_file: String::new(),
            });
            while self.match_token(TokenType::Comma) {
                let (extra_ty, extra_name) = self.parse_declarator(&base_type);
                let extra_init = if self.match_token(TokenType::Assign) {
                    if self.check(TokenType::LBrace) {
                        Some(self.parse_init_list())
                    } else {
                        Some(self.parse_expression())
                    }
                } else { None };
                program.globals.push(GlobalDecl {
                    loc: SourceLoc { line: self.previous().line, column: self.previous().column },
                    ty: extra_ty, name: extra_name, init: extra_init,
                    is_static,
                    source_file: String::new(),
                });
            }
            self.consume(TokenType::Semicolon, "全局变量声明后预期 ';'");
        }
    }

    fn parse_struct_body(&mut self, name: String, loc: SourceLoc) -> StructDecl {
        self.consume(TokenType::LBrace, "预期 '{'");
        let mut fields = Vec::new();
        while !self.check(TokenType::RBrace) && !self.is_at_end() {
            let field_checkpoint = self.pos;
            let (fty, fname) = self.parse_type_and_name();
            if self.pos == field_checkpoint {
                self.advance();
                break;
            }
            self.consume(TokenType::Semicolon, "预期 ';'");
            fields.push(StructField { ty: fty, name: fname });
        }
        self.consume(TokenType::RBrace, "预期 '}'");
        StructDecl { loc, name, fields }
    }

    fn parse_struct_decl(&mut self) -> StructDecl {
        self.consume(TokenType::Struct, "预期 'struct'");
        let name_tok = if self.check(TokenType::Identifier) {
            self.advance().clone()
        } else {
            Token {
                ty: TokenType::Identifier,
                text: format!("__anon_struct_{}", self.pos),
                line: self.current().line,
                column: self.current().column,
            }
        };
        let decl = self.parse_struct_body(name_tok.text.clone(), SourceLoc { line: name_tok.line, column: name_tok.column });
        self.consume(TokenType::Semicolon, "结构体声明后预期 ';'");
        decl
    }

    fn parse_union_decl(&mut self) -> StructDecl {
        self.consume(TokenType::Union, "预期 'union'");
        let name_tok = if self.check(TokenType::Identifier) {
            self.advance().clone()
        } else {
            Token {
                ty: TokenType::Identifier,
                text: format!("__anon_union_{}", self.pos),
                line: self.current().line,
                column: self.current().column,
            }
        };
        let decl = self.parse_struct_body(name_tok.text.clone(), SourceLoc { line: name_tok.line, column: name_tok.column });
        self.consume(TokenType::Semicolon, "联合体声明后预期 ';'");
        decl
    }

    fn parse_typedef_struct_decl(&mut self, program: &mut ProgramNode) {
        let loc = self.current().clone();
        self.advance(); // typedef
        self.advance(); // struct
        let name = if self.check(TokenType::Identifier) {
            let t = self.advance().clone();
            t.text
        } else {
            format!("__anon_struct_{}", self.pos)
        };
        let decl = self.parse_struct_body(name.clone(), SourceLoc { line: loc.line, column: loc.column });
        let alias_tok = self.consume(TokenType::Identifier, "typedef 后预期标识符名称").clone();
        self.consume(TokenType::Semicolon, "typedef 后预期 ';'");
        program.structs.push(decl);
        self.typedef_names.insert(alias_tok.text, Type::struct_type(name));
    }

    fn parse_func_decl(&mut self, is_static: bool) -> FuncDecl {
        let base_type = self.parse_base_type();

        let mut ret_type = base_type.clone();
        if self.match_token(TokenType::Star) {
            ret_type = Type::pointer_to(base_type.clone());
        }

        let name_tok = self.consume(TokenType::Identifier, "预期函数名称").clone();
        self.consume(TokenType::LParen, "预期 '('");

        let params = self.parse_param_list();
        self.consume(TokenType::RParen, "预期 ')'");
        let body = if self.check(TokenType::LBrace) {
            Some(self.parse_block())
        } else {
            self.consume(TokenType::Semicolon, "函数声明后预期 ';' 或 '{'");
            None
        };

        FuncDecl {
            loc: SourceLoc { line: name_tok.line, column: name_tok.column },
            return_type: ret_type,
            name: name_tok.text,
            params,
            body,
            is_static,
            source_file: String::new(),
        }
    }

    // =========================================================================
    // Type parsing
    // =========================================================================

    fn parse_base_type(&mut self) -> Type {
        // Collect type qualifiers/modifiers (const, signed, unsigned, long, short)
        let mut is_unsigned = false;
        let mut is_const = false;
        loop {
            if self.match_token(TokenType::Const) { is_const = true; continue; }
            if self.match_token(TokenType::Signed) { continue; }
            if self.match_token(TokenType::Unsigned) { is_unsigned = true; continue; }
            if self.match_token(TokenType::Long) {
                // Check for 'long long'
                if self.check(TokenType::Long) {
                    self.advance();
                    return Type::long_long();
                }
                continue;
            }
            if self.match_token(TokenType::Short) { continue; }
            break;
        }
        let mut t = if self.match_token(TokenType::Int) {
            if is_unsigned { Type::unsigned_int() } else { Type::int() }
        } else if self.match_token(TokenType::Void) {
            Type::void()
        } else if self.match_token(TokenType::Float) {
            Type::float()
        } else if self.match_token(TokenType::Double) {
            Type::double()
        } else if self.match_token(TokenType::Union) {
            if self.check(TokenType::Identifier) {
                let name_tok = self.advance().clone();
                Type::union_type(name_tok.text)
            } else {
                Type::union_type("")
            }
        } else if self.check(TokenType::Long) {
            self.advance();
            if self.check(TokenType::Long) {
                self.advance();
            }
            Type::long_long()
        } else if self.match_token(TokenType::Char) {
            if is_unsigned { Type::Char { is_unsigned: true, is_const: false } } else { Type::char() }
        } else if self.match_token(TokenType::Struct) {
            if self.check(TokenType::Identifier) {
                let name_tok = self.advance().clone();
                Type::struct_type(name_tok.text)
            } else if self.check(TokenType::LBrace) {
                let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
                let name = format!("__anon_struct_{}", self.pos);
                let decl = self.parse_struct_body(name.clone(), loc);
                self.anonymous_structs.push(decl);
                Type::struct_type(name)
            } else {
                self.errors.push(ParseError {
                    message: "struct 后预期标识符或 '{'".to_string(),
                    line: self.current().line,
                    column: self.current().column,
                    code: ErrorCode::E1006_UnsupportedFeature as i32,
                });
                Type::int()
            }
        } else if self.match_token(TokenType::Enum) {
            if self.check(TokenType::Identifier) {
                let name_tok = self.advance().clone();
                self.typedef_names.insert(name_tok.text.clone(), Type::int());
            }
            Type::int()
        } else if self.check(TokenType::Identifier) {
            if let Some(ty) = self.typedef_names.get(&self.current().text).cloned() {
                self.advance();
                ty
            } else {
                if is_unsigned { Type::unsigned_int() } else { Type::int() }
            }
        } else {
            if is_unsigned { Type::unsigned_int() } else { Type::int() }
        };
        if is_unsigned && !matches!(t.kind(), TypeKind::Int | TypeKind::Char) {
            self.errors.push(ParseError {
                message: format!("'unsigned' 不能修饰 '{}' 类型", match t.kind() {
                    TypeKind::Float => "float",
                    TypeKind::Double => "double",
                    TypeKind::Struct => "struct",
                    TypeKind::Void => "void",
                    TypeKind::Pointer => "指针",
                    TypeKind::Array => "数组",
                    _ => "此",
                }),
                line: self.current().line,
                column: self.current().column,
                code: ErrorCode::E1006_UnsupportedFeature as i32,
            });
            return Type::int();
        }
        t.set_const(is_const);
        t
    }

    fn parse_declarator(&mut self, base_type: &Type) -> (Type, String) {
        let mut guard = DeclaratorGuard::default();
        let (node, name) = self.parse_declarator_node(&mut guard, false);
        let name = name.unwrap_or_default();
        let ty = Self::interpret_declarator_node(&node, base_type);
        (ty, name)
    }

    // 声明符节点树：按 C 螺旋规则从内到外解释
    /// 解析声明符节点树（C 螺旋规则）。
    /// `is_abstract = true` 时用于 `sizeof(type)` 等抽象声明符场景，不读取标识符且不检查复杂度。
    fn parse_declarator_node(&mut self, guard: &mut DeclaratorGuard, is_abstract: bool) -> (DeclaratorNode, Option<String>) {
        let mut ptr_prefixes = 0;
        while self.match_token(TokenType::Star) {
            ptr_prefixes += 1;
            if !is_abstract {
                guard.ptr_count += 1;
            }
        }

        let (name, mut node) = if self.match_token(TokenType::LParen) {
            if !is_abstract {
                guard.paren_depth += 1;
                if guard.paren_depth > 2 {
                    self.errors.push(ParseError {
                        message: "声明符括号嵌套过深".to_string(),
                        line: self.current().line,
                        column: self.current().column,
                        code: ErrorCode::E1007_ComplexDeclarator as i32,
                    });
                    while !self.check(TokenType::RParen) && !self.is_at_end() {
                        self.advance();
                    }
                    self.match_token(TokenType::RParen);
                    guard.paren_depth -= 1;
                    return (DeclaratorNode::Base, Some(String::new()));
                }
            }
            let (inner_node, inner_name) = self.parse_declarator_node(guard, is_abstract);
            self.consume(TokenType::RParen, "预期 ')'");
            if !is_abstract {
                guard.paren_depth -= 1;
            }
            (inner_name, inner_node)
        } else if is_abstract {
            (None, DeclaratorNode::Base)
        } else {
            let name_tok = self.consume(TokenType::Identifier, "预期标识符名称").clone();
            (Some(name_tok.text), DeclaratorNode::Base)
        };

        // 收集后缀（它们绑定更紧）
        let mut suffixes = Vec::new();
        loop {
            if self.match_token(TokenType::LBracket) {
                if !is_abstract {
                    guard.suffix_count += 1;
                }
                let size = if self.check(TokenType::Number) {
                    let size_tok = self.advance().clone();
                    match size_tok.text.parse::<i32>() {
                        Ok(s) => s,
                        Err(_) => {
                            self.errors.push(ParseError {
                                message: format!("数组维度 '{}' 不是有效的编译期常量整数", size_tok.text),
                                line: size_tok.line,
                                column: size_tok.column,
                                code: ErrorCode::E2002_ExpectedArraySize as i32,
                            });
                            0
                        }
                    }
                } else if self.check(TokenType::RBracket) {
                    -1
                } else {
                    self.errors.push(ParseError {
                        message: "预期数组大小或 ']'".to_string(),
                        line: self.current().line,
                        column: self.current().column,
                        code: ErrorCode::E2002_ExpectedArraySize as i32,
                    });
                    0
                };
                self.consume(TokenType::RBracket, "预期 ']'");
                suffixes.push(DeclaratorSuffix::Array(size));
            } else if self.match_token(TokenType::LParen) {
                if !is_abstract {
                    guard.suffix_count += 1;
                }
                let params = self.parse_param_list();
                self.consume(TokenType::RParen, "预期 ')'");
                suffixes.push(DeclaratorSuffix::Function(params));
            } else {
                break;
            }
        }

        // 先应用后缀（它们绑定到标识符更紧）
        for suffix in suffixes {
            match suffix {
                DeclaratorSuffix::Array(size) => {
                    node = DeclaratorNode::Array(Box::new(node), size);
                }
                DeclaratorSuffix::Function(params) => {
                    node = DeclaratorNode::Function(Box::new(node), params);
                }
            }
        }

        // 再应用前缀指针（从外到内，但解释时从内到外）
        for _ in 0..ptr_prefixes {
            node = DeclaratorNode::Pointer(Box::new(node));
        }

        if !is_abstract {
            guard.cross_count = node_cross_count(&node);
            if guard.cross_count > 4 {
                self.errors.push(ParseError {
                    message: "声明符过于复杂".to_string(),
                    line: self.current().line,
                    column: self.current().column,
                    code: 1007,
                });
                return (DeclaratorNode::Base, name);
            }
        }

        (node, name)
    }

    fn interpret_declarator_node(node: &DeclaratorNode, base_type: &Type) -> Type {
        match node {
            DeclaratorNode::Base => base_type.clone(),
            DeclaratorNode::Pointer(inner) => {
                let inner_ty = Self::interpret_declarator_node(inner, base_type);
                match inner.as_ref() {
                    DeclaratorNode::Array(array_inner, size) => {
                        let elem_ty = Self::interpret_declarator_node(array_inner, base_type);
                        Type::Array {
                            element: Box::new(Type::pointer_to(elem_ty)),
                            array_size: *size,
                            dims: vec![*size],
                            is_const: false,
                        }
                    }
                    _ => Type::pointer_to(inner_ty),
                }
            }
            DeclaratorNode::Array(inner, size) => {
                let inner_ty = Self::interpret_declarator_node(inner, base_type);
                match inner.as_ref() {
                    DeclaratorNode::Pointer(ptr_inner) => {
                        let elem_ty = Self::interpret_declarator_node(ptr_inner, base_type);
                        Type::Pointer {
                            pointee: Box::new(Type::Array {
                                element: Box::new(elem_ty),
                                array_size: *size,
                                dims: vec![*size],
                                is_const: false,
                            }),
                            is_const: false,
                        }
                    }
                    _ => {
                        let (element, mut inner_dims, inner_array_size) = if let Type::Array { element, dims, array_size, .. } = &inner_ty {
                            (element.clone(), dims.clone(), *array_size)
                        } else {
                            (Box::new(inner_ty.clone()), Vec::new(), 1)
                        };
                        inner_dims.push(*size);
                        let array_size = if *size > 0 { *size * inner_array_size } else { *size };
                        Type::Array {
                            element,
                            array_size,
                            dims: inner_dims,
                            is_const: false,
                        }
                    }
                }
            }
            DeclaratorNode::Function(inner, params) => {
                match inner.as_ref() {
                    DeclaratorNode::Pointer(ptr_inner) => {
                        match ptr_inner.as_ref() {
                            DeclaratorNode::Array(array_inner, size) => {
                                // (*fp[N])(params) → function pointer array
                                let elem_ty = Self::interpret_declarator_node(array_inner, base_type);
                                Type::Array {
                                    element: Box::new(Type::Pointer {
                                        pointee: Box::new(Type::Function {
                                            return_type: Box::new(elem_ty),
                                            param_types: params.iter().map(|p| p.ty.clone()).collect(),
                                            is_const: false,
                                        }),
                                        is_const: false,
                                    }),
                                    array_size: *size,
                                    dims: vec![*size],
                                    is_const: false,
                                }
                            }
                            _ => {
                                let func_ptr_type = Type::Pointer {
                                    pointee: Box::new(Type::Function {
                                        return_type: Box::new(base_type.clone()),
                                        param_types: params.iter().map(|p| p.ty.clone()).collect(),
                                        is_const: false,
                                    }),
                                    is_const: false,
                                };
                                Self::interpret_declarator_node(ptr_inner, &func_ptr_type)
                            }
                        }
                    }
                    _ => {
                        let inner_ty = Self::interpret_declarator_node(inner, base_type);
                        Type::Pointer {
                            pointee: Box::new(Type::Function {
                                return_type: Box::new(inner_ty),
                                param_types: params.iter().map(|p| p.ty.clone()).collect(),
                                is_const: false,
                            }),
                            is_const: false,
                        }
                    }
                }
            }
        }
    }

    fn parse_type_and_name(&mut self) -> (Type, String) {
        let base_type = self.parse_base_type();
        self.parse_declarator(&base_type)
    }

    fn parse_param_list(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        if self.check(TokenType::RParen) { return params; }
        if self.check(TokenType::Void) && self.peek(1).ty == TokenType::RParen {
            self.advance();
            return params;
        }
        loop {
            let base_type = self.parse_base_type();
            let (pty, pname) = if self.check(TokenType::Comma) || self.check(TokenType::RParen) {
                // 无名参数（函数原型声明）：int foo(int);
                (base_type, String::new())
            } else {
                self.parse_declarator(&base_type)
            };
            params.push(Param { ty: pty, name: pname, loc: SourceLoc { line: self.current().line, column: self.current().column } });
            if !self.match_token(TokenType::Comma) { break; }
        }
        params
    }

    // =========================================================================
    // Statements
    // =========================================================================

    fn parse_statement(&mut self) -> Stmt {
        match self.current().ty {
            TokenType::LBrace => self.parse_block(),
            TokenType::If => self.parse_if_stmt(),
            TokenType::While => self.parse_while_stmt(),
            TokenType::Do => self.parse_do_while_stmt(),
            TokenType::For => self.parse_for_stmt(),
            TokenType::Return => self.parse_return_stmt(),
            TokenType::Break => self.parse_break_stmt(),
            TokenType::Continue => self.parse_continue_stmt(),
            TokenType::Switch => self.parse_switch_stmt(),
            TokenType::Case | TokenType::Default => self.parse_case_stmt(),
            _ if self.is_type_token() || self.is_static_token() => {
                if self.is_static_token() {
                    self.advance(); // consume 'static'
                }
                self.parse_var_decl_stmt()
            }
            _ => {
                let checkpoint = self.pos;
                let stmt = self.parse_expr_stmt();
                if self.pos == checkpoint {
                    self.synchronize(&[
                        TokenType::Semicolon, TokenType::RBrace,
                        TokenType::Int, TokenType::Void, TokenType::Char, TokenType::Float, TokenType::Double, TokenType::Long,
                        TokenType::If, TokenType::While, TokenType::Do, TokenType::For,
                        TokenType::Return, TokenType::Break, TokenType::Continue,
                        TokenType::Struct, TokenType::Switch, TokenType::Typedef,
                    ]);
                }
                stmt
            }
        }
    }

    fn parse_block(&mut self) -> Stmt {
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
        Stmt::Block { stmts, loc: SourceLoc { line: loc.line, column: loc.column } }
    }

    fn parse_var_decl_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        let base_type = self.parse_base_type();
        let (var_type, name) = self.parse_declarator(&base_type);
        let init = if self.match_token(TokenType::Assign) {
            if self.check(TokenType::LBrace) {
                Some(self.parse_init_list())
            } else {
                Some(self.parse_expression())
            }
        } else { None };

        let mut extra_vars = Vec::new();
        while self.match_token(TokenType::Comma) {
            let (extra_ty, extra_name) = self.parse_declarator(&base_type);
            let extra_init = if self.match_token(TokenType::Assign) {
                if self.check(TokenType::LBrace) {
                    Some(self.parse_init_list())
                } else {
                    Some(self.parse_expression())
                }
            } else { None };
            extra_vars.push((extra_ty, extra_name, extra_init));
        }

        self.consume(TokenType::Semicolon, "变量声明后预期 ';'");
        Stmt::VarDecl { var_type, name, init, extra_vars, loc: SourceLoc { line: loc.line, column: loc.column } }
    }

    fn parse_if_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.consume(TokenType::If, "预期 'if'");
        self.consume(TokenType::LParen, "预期 '('");
        let cond = self.parse_expression();
        self.consume(TokenType::RParen, "预期 ')'");
        let then_stmt = Box::new(self.parse_statement());
        let else_stmt = if self.match_token(TokenType::Else) {
            Some(Box::new(self.parse_statement()))
        } else { None };
        Stmt::If { cond, then_stmt, else_stmt, loc: SourceLoc { line: loc.line, column: loc.column } }
    }

    fn parse_while_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.consume(TokenType::While, "预期 'while'");
        self.consume(TokenType::LParen, "预期 '('");
        let cond = self.parse_expression();
        self.consume(TokenType::RParen, "预期 ')'");
        let body = Box::new(self.parse_statement());
        Stmt::While { cond, body, loc: SourceLoc { line: loc.line, column: loc.column } }
    }

    fn parse_do_while_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.consume(TokenType::Do, "预期 'do'");
        let body = Box::new(self.parse_statement());
        self.consume(TokenType::While, "预期 'while'");
        self.consume(TokenType::LParen, "预期 '('");
        let cond = self.parse_expression();
        self.consume(TokenType::RParen, "预期 ')'");
        self.consume(TokenType::Semicolon, "do...while 后预期 ';'");
        Stmt::DoWhile { body, cond, loc: SourceLoc { line: loc.line, column: loc.column } }
    }

    fn parse_break_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.consume(TokenType::Break, "预期 'break'");
        self.consume(TokenType::Semicolon, "break 后预期 ';'");
        Stmt::Break { loc: SourceLoc { line: loc.line, column: loc.column } }
    }

    fn parse_continue_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.consume(TokenType::Continue, "预期 'continue'");
        self.consume(TokenType::Semicolon, "continue 后预期 ';'");
        Stmt::Continue { loc: SourceLoc { line: loc.line, column: loc.column } }
    }

    fn parse_for_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.consume(TokenType::For, "预期 'for'");
        self.consume(TokenType::LParen, "预期 '('");

        let init: Option<Box<Stmt>> = if self.is_type_token() {
            let var_loc = self.current().clone();
            let base_type = self.parse_base_type();
            let (var_type, name) = self.parse_declarator(&base_type);
            let init_expr = if self.match_token(TokenType::Assign) {
                if self.check(TokenType::LBrace) {
                    Some(self.parse_init_list())
                } else {
                    Some(self.parse_expression())
                }
            } else { None };
            let mut extra_vars = Vec::new();
            while self.match_token(TokenType::Comma) {
                let (extra_ty, extra_name) = self.parse_declarator(&base_type);
                let extra_init = if self.match_token(TokenType::Assign) {
                    if self.check(TokenType::LBrace) {
                        Some(self.parse_init_list())
                    } else {
                        Some(self.parse_expression())
                    }
                } else { None };
                extra_vars.push((extra_ty, extra_name, extra_init));
            }
            Some(Box::new(Stmt::VarDecl {
                var_type, name, init: init_expr, extra_vars,
                loc: SourceLoc { line: var_loc.line, column: var_loc.column },
            }))
        } else if !self.check(TokenType::Semicolon) {
            let es_loc = self.current().clone();
            let expr = self.parse_expression();
            Some(Box::new(Stmt::Expr { expr, loc: SourceLoc { line: es_loc.line, column: es_loc.column } }))
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "预期 ';'");

        let cond = if !self.check(TokenType::Semicolon) {
            Some(self.parse_expression())
        } else { None };
        self.consume(TokenType::Semicolon, "预期 ';'");

        let step = if !self.check(TokenType::RParen) {
            Some(self.parse_expression())
        } else { None };
        self.consume(TokenType::RParen, "预期 ')'");

        let body = Box::new(self.parse_statement());
        Stmt::For { init, cond, step, body, loc: SourceLoc { line: loc.line, column: loc.column } }
    }

    fn parse_return_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.consume(TokenType::Return, "预期 'return'");
        let value = if !self.check(TokenType::Semicolon) {
            Some(self.parse_expression())
        } else { None };
        self.consume(TokenType::Semicolon, "return 后预期 ';'");
        Stmt::Return { value, loc: SourceLoc { line: loc.line, column: loc.column } }
    }

    fn parse_expr_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        let expr = self.parse_expression();
        self.consume(TokenType::Semicolon, "预期 ';'");
        Stmt::Expr { expr, loc: SourceLoc { line: loc.line, column: loc.column } }
    }

    // =========================================================================
    // Expressions (precedence climbing)
    // =========================================================================

    fn parse_expression(&mut self) -> Expr {
        self.parse_assign()
    }

    fn parse_assign(&mut self) -> Expr {
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

    fn parse_ternary(&mut self) -> Expr {
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

    fn parse_or(&mut self) -> Expr {
        let mut left = self.parse_and();
        while self.match_token(TokenType::OrOr) {
            let right = self.parse_and();
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            left = Expr::Binary { op: BinaryOp::Or, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
        }
        left
    }

    fn parse_and(&mut self) -> Expr {
        let mut left = self.parse_bit_or();
        while self.match_token(TokenType::AndAnd) {
            let right = self.parse_bit_or();
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            left = Expr::Binary { op: BinaryOp::And, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
        }
        left
    }

    fn parse_bit_or(&mut self) -> Expr {
        let mut left = self.parse_bit_xor();
        while self.match_token(TokenType::BitOr) {
            let right = self.parse_bit_xor();
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            left = Expr::Binary { op: BinaryOp::BitOr, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
        }
        left
    }

    fn parse_bit_xor(&mut self) -> Expr {
        let mut left = self.parse_bit_and();
        while self.match_token(TokenType::BitXor) {
            let right = self.parse_bit_and();
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            left = Expr::Binary { op: BinaryOp::BitXor, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
        }
        left
    }

    fn parse_bit_and(&mut self) -> Expr {
        let mut left = self.parse_equality();
        while self.match_token(TokenType::Ampersand) {
            let right = self.parse_equality();
            let loc = SourceLoc { line: self.previous().line, column: self.previous().column };
            left = Expr::Binary { op: BinaryOp::BitAnd, left: Box::new(left), right: Box::new(right), loc, ty: Type::default() };
        }
        left
    }

    fn parse_equality(&mut self) -> Expr {
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

    fn parse_relational(&mut self) -> Expr {
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

    fn parse_shift(&mut self) -> Expr {
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

    fn parse_additive(&mut self) -> Expr {
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

    fn parse_multiplicative(&mut self) -> Expr {
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

    fn parse_unary(&mut self) -> Expr {
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

    fn parse_abstract_declarator(&mut self) -> Option<DeclaratorNode> {
        let mut guard = DeclaratorGuard::default();
        let (node, _) = self.parse_declarator_node(&mut guard, true);
        if matches!(node, DeclaratorNode::Base) {
            None
        } else {
            Some(node)
        }
    }

    fn parse_sizeof(&mut self) -> Expr {
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

    fn parse_type_only(&mut self) -> Type {
        let base = self.parse_base_type();
        if self.match_token(TokenType::Star) {
            return Type::pointer_to(base.clone());
        }
        base
    }

    fn parse_postfix(&mut self) -> Expr {
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

    fn parse_init_list(&mut self) -> Expr {
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

    fn parse_primary(&mut self) -> Expr {
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
            return Expr::StringLiteral { value, loc, ty: Type::Array { element: Box::new(Type::char()), array_size, dims: vec![array_size], is_const: false } };
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

    fn parse_arg_list(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        if self.check(TokenType::RParen) { return args; }
        loop {
            args.push(self.parse_expression());
            if !self.match_token(TokenType::Comma) { break; }
        }
        args
    }

    fn parse_switch_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.advance();
        self.consume(TokenType::LParen, "switch 后预期 '('");
        let cond = self.parse_expression();
        self.consume(TokenType::RParen, "switch 条件后预期 ')'");
        let body = Box::new(self.parse_statement());
        Stmt::Switch { cond, body, loc: SourceLoc { line: loc.line, column: loc.column } }
    }

    fn parse_case_stmt(&mut self) -> Stmt {
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
            return Stmt::Block { stmts: Vec::new(), loc: SourceLoc { line: loc.line, column: loc.column } };
        };
        self.consume(TokenType::Colon, "case/default 后预期 ':'");
        let mut stmts = Vec::new();
        while !self.check(TokenType::Case) && !self.check(TokenType::Default) &&
              !self.check(TokenType::RBrace) && !self.is_at_end() {
            let stmt_checkpoint = self.pos;
            stmts.push(self.parse_statement());
            if self.pos == stmt_checkpoint {
                self.advance();
            }
        }
        let stmt = if stmts.is_empty() {
            Stmt::Block { stmts: Vec::new(), loc: SourceLoc { line: loc.line, column: loc.column } }
        } else if stmts.len() == 1 {
            stmts.pop().unwrap()
        } else {
            Stmt::Block { stmts, loc: SourceLoc { line: loc.line, column: loc.column } }
        };
        Stmt::Case { label, stmt: Box::new(stmt), loc: SourceLoc { line: loc.line, column: loc.column } }
    }

    fn parse_typedef(&mut self) {
        self.advance();
        let base_type = self.parse_base_type();
        let (ty, name) = self.parse_declarator(&base_type);
        self.consume(TokenType::Semicolon, "typedef 后预期 ';'");
        self.typedef_names.insert(name, ty);
    }

    fn parse_enum_decl(&mut self, program: &mut ProgramNode) {
        let loc = self.current().clone();
        self.advance();
        let mut enum_name = String::new();
        if self.check(TokenType::Identifier) {
            enum_name = self.current().text.clone();
            self.advance();
        }
        self.consume(TokenType::LBrace, "enum 后预期 '{'");
        let mut next_value = 0;
        while !self.check(TokenType::RBrace) && !self.is_at_end() {
            let member_tok = self.consume(TokenType::Identifier, "enum 成员预期标识符").clone();
            if self.match_token(TokenType::Assign) {
                let val_expr = self.parse_expression();
                if let Expr::Literal { value, .. } = val_expr {
                    next_value = value;
                }
            }
            program.globals.push(GlobalDecl {
                loc: SourceLoc { line: loc.line, column: loc.column },
                ty: Type::int(),
                name: member_tok.text,
                init: Some(Expr::Literal { value: next_value, loc: SourceLoc { line: member_tok.line, column: member_tok.column }, ty: Type::int() }),
                is_static: false,
                source_file: String::new(),
            });
            next_value += 1;
            if !self.match_token(TokenType::Comma) { break; }
        }
        self.consume(TokenType::RBrace, "enum 成员后预期 '}'");
        self.consume(TokenType::Semicolon, "enum 声明后预期 ';'");
        if !enum_name.is_empty() {
            self.typedef_names.insert(enum_name, Type::int());
        }
    }
}
