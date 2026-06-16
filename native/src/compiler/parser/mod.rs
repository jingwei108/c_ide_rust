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
pub(crate) enum DeclaratorNode {
    Base,
    Pointer(Box<DeclaratorNode>),
    Reference(Box<DeclaratorNode>, bool /* is_const */),
    RValueRef(Box<DeclaratorNode>),
    Array(Box<DeclaratorNode>, Option<Box<Expr>>),
    Function(Box<DeclaratorNode>, Vec<Param>),
}

#[derive(Debug, Clone)]
enum DeclaratorSuffix {
    Array(Option<Box<Expr>>),
    Function(Vec<Param>),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct DeclaratorGuard {
    paren_depth: i32,
    ptr_count: i32,
    suffix_count: i32,
    cross_count: i32,
}

fn node_cross_count(node: &DeclaratorNode) -> i32 {
    match node {
        DeclaratorNode::Base => 0,
        DeclaratorNode::Pointer(inner) | DeclaratorNode::Reference(inner, _) | DeclaratorNode::RValueRef(inner) => {
            let add = match inner.as_ref() {
                DeclaratorNode::Array(_, _) | DeclaratorNode::Function(_, _) => 1,
                _ => 0,
            };
            node_cross_count(inner) + add
        }
        DeclaratorNode::Array(inner, _) | DeclaratorNode::Function(inner, _) => node_cross_count(inner),
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    typedef_names: HashMap<String, Type>,
    template_names: std::collections::HashSet<String>,
    pos: usize,
    anonymous_structs: Vec<StructDecl>,
    is_cpp_mode: bool,
    next_lambda_id: u64,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self::with_mode(tokens, false)
    }

    pub fn with_mode(tokens: Vec<Token>, is_cpp_mode: bool) -> Self {
        let mut typedef_names = HashMap::new();
        let mut template_names = std::collections::HashSet::new();
        // 预定义 FILE 类型（stdio.h 中的不透明结构体指针）
        typedef_names.insert("FILE".to_string(), Type::void());
        // 预注册 C++ 内置容器类型名
        if is_cpp_mode {
            for name in [
                "cide_vec_int",
                "cide_vec_float",
                "cide_vec_char",
                "cide_string",
                "cide_list_int",
            ] {
                typedef_names.insert(
                    name.to_string(),
                    Type::Class {
                        name: name.to_string(),
                        is_const: false,
                    },
                );
            }
            // 预注册内置容器模板名，支持显式实例化写法。
            // 注意：cide_string 本身是最终类名，不在这里作为模板基名注册，
            // 避免与用户代码中的 typedef struct cide_string { ... } 冲突。
            for name in ["cide_vec", "cide_list"] {
                template_names.insert(name.to_string());
            }
        }
        Self {
            tokens,
            errors: Vec::new(),
            typedef_names,
            template_names,
            pos: 0,
            anonymous_structs: Vec::new(),
            is_cpp_mode,
            next_lambda_id: 0,
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

    pub(crate) fn peek(&self, offset: usize) -> &Token {
        if self.pos + offset >= self.tokens.len() {
            static EOF: Token = Token {
                ty: TokenType::Eof,
                text: String::new(),
                line: -1,
                column: -1,
            };
            return &EOF;
        }
        &self.tokens[self.pos + offset]
    }

    pub(crate) fn current(&self) -> &Token {
        self.peek(0)
    }
    pub(crate) fn previous(&self) -> &Token {
        if self.pos == 0 {
            return self.peek(0);
        }
        &self.tokens[self.pos - 1]
    }

    pub(crate) fn check(&self, ty: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.current().ty == ty
    }

    pub(crate) fn is_at_end(&self) -> bool {
        self.current().ty == TokenType::Eof
    }

    pub(crate) fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        if self.pos == 0 {
            return self.peek(0);
        }
        &self.tokens[self.pos - 1]
    }

    pub(crate) fn match_token(&mut self, ty: TokenType) -> bool {
        if self.check(ty) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub(crate) fn consume(&mut self, ty: TokenType, msg: &str) -> &Token {
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
    pub(crate) fn synchronize(&mut self, recovery_set: &[TokenType]) {
        while !self.is_at_end() {
            let current = self.current().ty;
            if recovery_set.contains(&current) {
                return;
            }
            if self.previous().ty == TokenType::Semicolon {
                return;
            }
            match current {
                TokenType::Int
                | TokenType::Void
                | TokenType::Char
                | TokenType::Float
                | TokenType::Double
                | TokenType::If
                | TokenType::While
                | TokenType::Do
                | TokenType::For
                | TokenType::Return
                | TokenType::Break
                | TokenType::Continue
                | TokenType::Struct
                | TokenType::Switch
                | TokenType::Case
                | TokenType::Default
                | TokenType::Typedef
                | TokenType::Enum
                | TokenType::Unsigned
                | TokenType::Long
                | TokenType::Short
                | TokenType::Signed
                | TokenType::Const
                | TokenType::Bool
                | TokenType::RBrace => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    pub(crate) fn is_type_token(&self) -> bool {
        if self.check(TokenType::Int)
            || self.check(TokenType::Void)
            || self.check(TokenType::Char)
            || self.check(TokenType::Float)
            || self.check(TokenType::Double)
            || self.check(TokenType::Struct)
            || self.check(TokenType::Enum)
            || self.check(TokenType::Unsigned)
            || self.check(TokenType::Long)
            || self.check(TokenType::Short)
            || self.check(TokenType::Signed)
            || self.check(TokenType::Const)
            || self.check(TokenType::Volatile)
            || self.check(TokenType::Bool)
            || self.check(TokenType::Union)
        {
            return true;
        }
        // C++ 模式下 class 和 auto 也是类型 token
        if self.is_cpp_mode && (self.check(TokenType::Class) || self.check(TokenType::Auto)) {
            return true;
        }
        if self.check(TokenType::Identifier) {
            let name = &self.current().text;
            if name == "typeof" || name == "__typeof__" || name == "__typeof" {
                return true;
            }
            return self.typedef_names.contains_key(name);
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
            // 跳过函数前缀修饰符 inline（可能出现在 static/extern 之前或之后）
            while self.check(TokenType::Inline) {
                self.advance();
            }
            if self.check(TokenType::Identifier) && self.peek(0).text == "_Static_assert" {
                self.parse_static_assert();
                continue;
            }
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
                if self.check(TokenType::Enum) {
                    let e_checkpoint = self.pos;
                    let e_errors_checkpoint = self.errors.len();
                    self.advance();
                    if self.check(TokenType::Identifier) {
                        self.advance();
                    }
                    if self.check(TokenType::LBrace) {
                        self.pos = checkpoint;
                        self.errors.truncate(errors_checkpoint);
                        self.parse_typedef_enum_decl(&mut program);
                        continue;
                    }
                    self.pos = e_checkpoint;
                    self.errors.truncate(e_errors_checkpoint);
                }
                self.pos = checkpoint;
                self.errors.truncate(errors_checkpoint);
                self.parse_typedef();
            } else if self.check(TokenType::Enum) {
                let checkpoint = self.pos;
                let errors_checkpoint = self.errors.len();
                self.advance();
                if self.check(TokenType::Identifier) {
                    self.advance();
                }
                let is_enum_decl = self.check(TokenType::LBrace);
                self.pos = checkpoint;
                self.errors.truncate(errors_checkpoint);
                if is_enum_decl {
                    self.parse_enum_decl(&mut program);
                } else {
                    self.parse_global_var_or_func(&mut program, false, false);
                }
            } else if self.check(TokenType::Struct) {
                let checkpoint = self.pos;
                let errors_checkpoint = self.errors.len();
                self.advance();
                let has_name = self.check(TokenType::Identifier);
                if has_name {
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
                            if brace_depth == 0 {
                                break;
                            }
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
                        self.parse_global_var_or_func(&mut program, false, false);
                    } else {
                        // C++ 模式下，有名字的 struct 声明解析为 class-like（支持构造函数、方法等）
                        if self.is_cpp_mode && has_name {
                            let class_decl = self.parse_class_decl_inner(true);
                            self.typedef_names.insert(
                                class_decl.name.clone(),
                                Type::Class {
                                    name: class_decl.name.clone(),
                                    is_const: false,
                                },
                            );
                            program.classes.push(class_decl);
                        } else {
                            let struct_decl = self.parse_struct_decl();
                            if self.is_cpp_mode {
                                self.typedef_names
                                    .insert(struct_decl.name.clone(), Type::struct_type(struct_decl.name.clone()));
                            }
                            program.structs.push(struct_decl);
                        }
                    }
                } else {
                    self.pos = checkpoint;
                    self.errors.truncate(errors_checkpoint);
                    self.parse_global_var_or_func(&mut program, false, false);
                }
            } else if self.check(TokenType::Union) {
                program.unions.push(self.parse_union_decl());
            } else if self.is_cpp_mode && self.check(TokenType::Class) {
                let class_decl = self.parse_class_decl();
                self.typedef_names.insert(
                    class_decl.name.clone(),
                    Type::Class {
                        name: class_decl.name.clone(),
                        is_const: false,
                    },
                );
                program.classes.push(class_decl);
            } else if self.is_cpp_mode && self.check(TokenType::Template) {
                if self.is_template_explicit_instantiation() {
                    let inst = self.parse_template_instantiation();
                    program.template_instantiations.push(inst);
                } else {
                    let template_decl = self.parse_template_decl();
                    // 将类模板名加入 typedef_names
                    if let crate::compiler::ast::Templateable::Class(ref c) = template_decl.decl {
                        self.typedef_names.insert(
                            c.name.clone(),
                            Type::Class {
                                name: c.name.clone(),
                                is_const: false,
                            },
                        );
                    }
                    // 类模板方法类外定义：如 template<class T> void Box<T>::set(T x) { ... }
                    // 已把 body 合并到对应类模板的 method 声明中，避免生成独立函数模板。
                    let mut merged = false;
                    if let crate::compiler::ast::Templateable::Func(ref f) = template_decl.decl {
                        if f.body.is_some() {
                            if let Some((class_name, method_name)) = f.name.split_once("__") {
                                for t in program.templates.iter_mut() {
                                    if let crate::compiler::ast::Templateable::Class(ref mut c) = t.decl {
                                        if c.name == class_name {
                                            for member in &mut c.members {
                                                if let crate::compiler::ast::ClassMember::Method {
                                                    name, body, ..
                                                } = member
                                                {
                                                    if name == method_name && body.is_none() {
                                                        *body = f.body.clone();
                                                        merged = true;
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !merged {
                        program.templates.push(template_decl);
                    }
                }
            } else if self.check(TokenType::Extern) {
                self.advance(); // consume 'extern'
                self.parse_global_var_or_func(&mut program, false, true);
            } else if self.is_type_token() || self.is_static_token() {
                let is_static = self.is_static_token();
                if is_static {
                    self.advance(); // consume 'static'
                }
                self.parse_global_var_or_func(&mut program, is_static, false);
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
        while lookahead < self.tokens.len() {
            match self.tokens[lookahead].ty {
                TokenType::Star => lookahead += 1,
                TokenType::Ampersand | TokenType::AndAnd if self.is_cpp_mode => lookahead += 1,
                _ => break,
            }
        }
        lookahead
    }

    fn parse_global_var_or_func(&mut self, program: &mut ProgramNode, is_static: bool, is_extern: bool) {
        let checkpoint = self.pos;
        let base_type = self.parse_base_type();
        // C++ 构造函数类外定义：Counter::Counter() { ... }
        // parse_base_type 已经把类名 'Counter' 作为返回类型读入，当前 token 是 '::'。
        if self.is_cpp_mode
            && matches!(&base_type, Type::Class { name, .. } if !name.is_empty())
            && self.check(TokenType::ColonColon)
            && self.peek(1).ty == TokenType::Identifier
            && self.peek(2).ty == TokenType::LParen
            && self.peek(1).text == base_type.name()
        {
            self.consume(TokenType::ColonColon, "预期 '::'");
            let name_tok = self.consume(TokenType::Identifier, "预期构造函数名").clone();
            self.consume(TokenType::LParen, "预期 '('");
            let params = self.parse_param_list();
            self.consume(TokenType::RParen, "预期 ')'");
            let body = if self.check(TokenType::LBrace) {
                Some(self.parse_block())
            } else {
                self.consume(TokenType::Semicolon, "构造函数声明后预期 ';' 或 '{'");
                None
            };
            program.funcs.push(FuncDecl {
                loc: SourceLoc {
                    line: name_tok.line,
                    column: name_tok.column,
                },
                return_type: Type::void(),
                name: format!("__ctor__{}", name_tok.text),
                params,
                body,
                is_static,
                is_extern,
                source_file: String::new(),
            });
            return;
        }
        // 前瞻：跳过前导 *，检查是否是 identifier (
        let lookahead = self.look_ahead_skip_stars();
        let is_func_decl = if lookahead < self.tokens.len() && self.tokens[lookahead].ty == TokenType::Identifier {
            if lookahead + 1 < self.tokens.len() && self.tokens[lookahead + 1].ty == TokenType::LParen {
                true
            } else if self.is_cpp_mode
                && lookahead + 3 < self.tokens.len()
                && self.tokens[lookahead + 1].ty == TokenType::ColonColon
                && self.tokens[lookahead + 2].ty == TokenType::Identifier
                && self.tokens[lookahead + 3].ty == TokenType::LParen
            {
                // C++ qualified name: Bar::set(...)
                true
            } else {
                false
            }
        } else {
            false
        };
        if is_func_decl {
            self.pos = checkpoint;
            program.funcs.push(self.parse_func_decl(is_static, is_extern));
        } else if self.is_cpp_mode
            && lookahead < self.tokens.len()
            && self.tokens[lookahead].ty == TokenType::Identifier
            && lookahead + 2 < self.tokens.len()
            && self.tokens[lookahead + 1].ty == TokenType::ColonColon
            && self.tokens[lookahead + 2].ty == TokenType::Identifier
        {
            // C++ qualified static field definition: int A::count = 0;
            let ty = base_type.clone();
            let class_name = self.consume(TokenType::Identifier, "预期类名").text.clone();
            self.consume(TokenType::ColonColon, "预期 '::'");
            let field_tok = self.consume(TokenType::Identifier, "预期字段名").clone();
            let name = format!("{}__{}", class_name, field_tok.text);
            let init = if is_extern {
                None
            } else if self.match_token(TokenType::Assign) {
                if self.check(TokenType::LBrace) {
                    Some(self.parse_init_list())
                } else {
                    Some(self.parse_assign())
                }
            } else {
                None
            };
            program.globals.push(GlobalDecl {
                loc: SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                },
                ty: ty.clone(),
                name,
                init,
                is_static,
                is_extern,
                source_file: String::new(),
            });
            self.consume(TokenType::Semicolon, "静态成员定义后预期 ';'");
        } else {
            let (ty, name) = self.parse_var_declarator(&base_type);
            let init = if is_extern {
                None
            } else if self.match_token(TokenType::Assign) {
                if self.check(TokenType::LBrace) {
                    Some(self.parse_init_list())
                } else {
                    Some(self.parse_assign())
                }
            } else {
                None
            };
            program.globals.push(GlobalDecl {
                loc: SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                },
                ty: ty.clone(),
                name,
                init,
                is_static,
                is_extern,
                source_file: String::new(),
            });
            while self.match_token(TokenType::Comma) {
                let (extra_ty, extra_name) = self.parse_var_declarator(&base_type);
                let extra_init = if is_extern {
                    None
                } else if self.match_token(TokenType::Assign) {
                    if self.check(TokenType::LBrace) {
                        Some(self.parse_init_list())
                    } else {
                        Some(self.parse_assign())
                    }
                } else {
                    None
                };
                program.globals.push(GlobalDecl {
                    loc: SourceLoc {
                        line: self.previous().line,
                        column: self.previous().column,
                    },
                    ty: extra_ty,
                    name: extra_name,
                    init: extra_init,
                    is_static,
                    is_extern,
                    source_file: String::new(),
                });
            }
            self.consume(TokenType::Semicolon, "全局变量声明后预期 ';'");
        }
    }







































}

mod type_;
mod decl;
mod stmt;
mod expr;
mod cpp;
