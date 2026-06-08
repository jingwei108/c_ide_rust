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
        DeclaratorNode::Pointer(inner) => {
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
        // 预定义 FILE 类型（stdio.h 中的不透明结构体指针）
        typedef_names.insert("FILE".to_string(), Type::void());
        // 预注册 C++ 内置容器类型名
        if is_cpp_mode {
            for name in ["cide_vec_int", "cide_vec_float", "cide_vec_char", "cide_string", "cide_list_int"] {
                typedef_names.insert(name.to_string(), Type::Class { name: name.to_string(), is_const: false });
            }
        }
        Self {
            tokens,
            errors: Vec::new(),
            typedef_names,
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
            // 跳过函数前缀修饰符 inline（可能出现在 static/extern 之前或之后）
            while self.check(TokenType::Inline) {
                self.advance();
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
                    self.parse_global_var_or_func(&mut program, false, false);
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
                        program.structs.push(self.parse_struct_decl());
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
                program.templates.push(template_decl);
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
        while lookahead < self.tokens.len() && self.tokens[lookahead].ty == TokenType::Star {
            lookahead += 1;
        }
        lookahead
    }

    fn parse_global_var_or_func(&mut self, program: &mut ProgramNode, is_static: bool, is_extern: bool) {
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
            program.funcs.push(self.parse_func_decl(is_static, is_extern));
        } else {
            let (ty, name) = self.parse_declarator(&base_type);
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
                let (extra_ty, extra_name) = self.parse_declarator(&base_type);
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
        let decl = self.parse_struct_body(
            name_tok.text.clone(),
            SourceLoc {
                line: name_tok.line,
                column: name_tok.column,
            },
        );
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
        let decl = self.parse_struct_body(
            name_tok.text.clone(),
            SourceLoc {
                line: name_tok.line,
                column: name_tok.column,
            },
        );
        self.consume(TokenType::Semicolon, "联合体声明后预期 ';'");
        decl
    }

    // =========================================================================
    // C++ Class Declaration (Phase 31)
    // =========================================================================

    fn parse_class_decl(&mut self) -> ClassDecl {
        let loc = SourceLoc {
            line: self.current().line,
            column: self.current().column,
        };
        self.consume(TokenType::Class, "预期 'class'");
        let name = self.consume(TokenType::Identifier, "预期类名").text.clone();

        let mut base = None;
        if self.match_token(TokenType::Colon) {
            // 支持 ': public Base'（public 可选）
            if self.check(TokenType::Public) || self.check(TokenType::Private) || self.check(TokenType::Protected) {
                self.advance();
            }
            base = Some(self.consume(TokenType::Identifier, "预期基类名").text.clone());
        }

        self.consume(TokenType::LBrace, "预期 '{'");

        let mut members = Vec::new();
        let mut current_access = AccessSpec::Private; // class 默认 private

        while !self.check(TokenType::RBrace) && !self.is_at_end() {
            // 访问说明符
            if self.check(TokenType::Public) || self.check(TokenType::Private) || self.check(TokenType::Protected) {
                let access = match self.current().ty {
                    TokenType::Public => AccessSpec::Public,
                    TokenType::Private => AccessSpec::Private,
                    TokenType::Protected => AccessSpec::Protected,
                    _ => unreachable!(),
                };
                self.advance();
                self.consume(TokenType::Colon, "访问说明符后预期 ':'");
                current_access = access;
                continue;
            }

            // 跳过 inline / virtual 修饰符
            let mut is_virtual = false;
            while self.check(TokenType::Inline) || self.check(TokenType::Virtual) {
                if self.check(TokenType::Virtual) {
                    is_virtual = true;
                }
                self.advance();
            }

            // 析构函数 ~Name()
            if self.check(TokenType::BitNot)
                && self.peek(1).ty == TokenType::Identifier
                && self.peek(2).ty == TokenType::LParen
            {
                // ~Destructor()
                self.advance(); // ~
                let _dtor_name = self.advance().text.clone();
                self.consume(TokenType::LParen, "预期 '('");
                self.consume(TokenType::RParen, "预期 ')'");
                let body = if self.check(TokenType::LBrace) {
                    Some(self.parse_statement())
                } else {
                    self.consume(TokenType::Semicolon, "析构函数声明后预期 ';'");
                    None
                };
                members.push(ClassMember::Destructor {
                    body,
                    access: current_access,
                    is_virtual,
                });
                continue;
            }

            // 构造函数检查（无返回类型）
            if self.check(TokenType::Identifier)
                && self.current().text == name
                && self.peek(1).ty == TokenType::LParen
            {
                self.advance(); // class name
                self.consume(TokenType::LParen, "预期 '('");
                let params = self.parse_param_list();
                self.consume(TokenType::RParen, "预期 ')'");
                let body = if self.check(TokenType::LBrace) {
                    Some(self.parse_statement())
                } else {
                    self.consume(TokenType::Semicolon, "构造函数声明后预期 ';'");
                    None
                };
                members.push(ClassMember::Constructor {
                    params,
                    body,
                    is_default: false,
                    access: current_access,
                });
                continue;
            }

            // 构造函数 / 方法 / 字段 需要类型前瞻
            if !self.is_type_token() && !self.check(TokenType::Identifier) {
                // 未知内容，报错并跳过
                self.errors.push(ParseError {
                    message: format!("类成员声明中预期类型或方法名，找到: {}", self.current().text),
                    line: self.current().line,
                    column: self.current().column,
                    code: ErrorCode::E2005_ExpectedSemicolon as i32,
                });
                self.advance();
                continue;
            }

            let checkpoint = self.pos;
            let _member_base_type = self.parse_base_type();
            let lookahead = self.look_ahead_skip_stars();

            if lookahead < self.tokens.len()
                && self.tokens[lookahead].ty == TokenType::Identifier
                && lookahead + 1 < self.tokens.len()
                && self.tokens[lookahead + 1].ty == TokenType::LParen
            {
                // 方法或构造函数
                self.pos = checkpoint;
                let ret_type = self.parse_base_type();
                let mut ptr_depth = 0;
                while self.match_token(TokenType::Star) {
                    ptr_depth += 1;
                }
                let mut final_ret = ret_type;
                for _ in 0..ptr_depth {
                    final_ret = Type::pointer_to(final_ret);
                }
                let method_name = self.advance().text.clone();
                self.consume(TokenType::LParen, "预期 '('");
                let params = self.parse_param_list();
                self.consume(TokenType::RParen, "预期 ')'");

                // 检查 override
                if self.check(TokenType::Override) {
                    self.advance();
                }

                if method_name == name && ptr_depth == 0 && matches!(final_ret.kind(), TypeKind::Void | TypeKind::Int) {
                    // 构造函数（简化：与类同名且无返回类型修饰）
                    let body = if self.check(TokenType::LBrace) {
                        Some(self.parse_statement())
                    } else {
                        self.consume(TokenType::Semicolon, "构造函数声明后预期 ';'");
                        None
                    };
                    members.push(ClassMember::Constructor {
                        params,
                        body,
                        is_default: false,
                        access: current_access,
                    });
                } else {
                    let body = if self.check(TokenType::LBrace) {
                        Some(self.parse_statement())
                    } else {
                        self.consume(TokenType::Semicolon, "方法声明后预期 ';'");
                        None
                    };
                    members.push(ClassMember::Method {
                        name: method_name,
                        ret: final_ret,
                        params,
                        body,
                        is_virtual,
                        access: current_access,
                        is_static: false,
                    });
                }
            } else {
                // 字段声明
                self.pos = checkpoint;
                let field_type = self.parse_base_type();
                let (ty, field_name) = self.parse_declarator(&field_type);
                self.consume(TokenType::Semicolon, "字段声明后预期 ';'");
                members.push(ClassMember::Field {
                    name: field_name,
                    ty,
                    access: current_access,
                });
            }
        }

        self.consume(TokenType::RBrace, "预期 '}'");
        self.consume(TokenType::Semicolon, "类声明后预期 ';'");

        ClassDecl {
            loc,
            name,
            base,
            members,
            vtable: None,
        }
    }

    // =========================================================================
    // C++ Template Declaration (Phase 31)
    // =========================================================================

    fn parse_template_decl(&mut self) -> TemplateDecl {
        let loc = SourceLoc {
            line: self.current().line,
            column: self.current().column,
        };
        self.consume(TokenType::Template, "预期 'template'");
        self.consume(TokenType::Lt, "预期 '<'");

        let mut params = Vec::new();
        while !self.check(TokenType::Gt) && !self.is_at_end() {
            // typename T 或 class T
            if self.check(TokenType::Typename) || self.check(TokenType::Class) {
                self.advance();
            }
            let param_name = self.consume(TokenType::Identifier, "预期模板参数名").text.clone();
            params.push(TemplateParam { name: param_name.clone(), loc });
            // 将模板参数注册为类型名，使其在函数/类体中可被识别
            // 使用 Class 类型作为占位符，以便 TypeChecker 在单态化时识别模板参数
            let tp_name = params.last().unwrap().name.clone();
            self.typedef_names
                .insert(param_name, Type::Class { name: tp_name, is_const: false });
            if !self.match_token(TokenType::Comma) {
                break;
            }
        }
        self.consume(TokenType::Gt, "预期 '>'");

        // 模板后面跟着函数或类声明
        let decl = if self.check(TokenType::Class) {
            Templateable::Class(Box::new(self.parse_class_decl()))
        } else {
            Templateable::Func(Box::new(self.parse_func_decl(false, false)))
        };
        TemplateDecl { loc, params, decl }
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
        let decl = self.parse_struct_body(
            name.clone(),
            SourceLoc {
                line: loc.line,
                column: loc.column,
            },
        );
        let alias_tok = self.consume(TokenType::Identifier, "typedef 后预期标识符名称").clone();
        self.consume(TokenType::Semicolon, "typedef 后预期 ';'");
        program.structs.push(decl);
        self.typedef_names.insert(alias_tok.text, Type::struct_type(name));
    }

    fn parse_func_decl(&mut self, is_static: bool, is_extern: bool) -> FuncDecl {
        let base_type = self.parse_base_type();

        let mut ret_type = base_type.clone();
        while self.match_token(TokenType::Star) {
            ret_type = Type::pointer_to(ret_type.clone());
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
            loc: SourceLoc {
                line: name_tok.line,
                column: name_tok.column,
            },
            return_type: ret_type,
            name: name_tok.text,
            params,
            body,
            is_static,
            is_extern,
            source_file: String::new(),
        }
    }

    // =========================================================================
    // Type parsing
    // =========================================================================

    pub(crate) fn parse_base_type(&mut self) -> Type {
        // Collect type qualifiers/modifiers (const, signed, unsigned, long, short)
        let mut is_unsigned = false;
        let mut is_const = false;
        loop {
            if self.match_token(TokenType::Const) {
                is_const = true;
                continue;
            }
            if self.match_token(TokenType::Volatile) {
                continue;
            }
            if self.match_token(TokenType::Restrict) {
                continue;
            }
            if self.match_token(TokenType::Register) {
                continue;
            }
            if self.match_token(TokenType::Auto) {
                if self.is_cpp_mode {
                    if is_const {
                        return Type::Auto; // auto is not const-qualifiable in our simplified model
                    }
                    return Type::Auto;
                }
                continue;
            }
            if self.match_token(TokenType::Inline) {
                continue;
            }
            if self.match_token(TokenType::Signed) {
                continue;
            }
            if self.match_token(TokenType::Unsigned) {
                is_unsigned = true;
                continue;
            }
            if self.match_token(TokenType::Long) {
                // Check for 'long long'
                if self.check(TokenType::Long) {
                    self.advance();
                    return Type::long_long();
                }
                continue;
            }
            if self.match_token(TokenType::Short) {
                continue;
            }
            break;
        }
        let mut t = if self.match_token(TokenType::Int) {
            if is_unsigned {
                Type::unsigned_int()
            } else {
                Type::int()
            }
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
            if is_unsigned {
                Type::Char {
                    is_unsigned: true,
                    is_const: false,
                }
            } else {
                Type::char()
            }
        } else if self.match_token(TokenType::Struct) {
            if self.check(TokenType::Identifier) {
                let name_tok = self.advance().clone();
                Type::struct_type(name_tok.text)
            } else if self.check(TokenType::LBrace) {
                let loc = SourceLoc {
                    line: self.previous().line,
                    column: self.previous().column,
                };
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
        } else if self.match_token(TokenType::Bool) {
            Type::int()
        } else if self.check(TokenType::Identifier) {
            if let Some(ty) = self.typedef_names.get(&self.current().text).cloned() {
                self.advance();
                ty
            } else if self.is_cpp_mode {
                let name = self.advance().text.clone();
                Type::Class {
                    name,
                    is_const: false,
                }
            } else {
                if is_unsigned {
                    Type::unsigned_int()
                } else {
                    Type::int()
                }
            }
        } else {
            if is_unsigned {
                Type::unsigned_int()
            } else {
                Type::int()
            }
        };
        if is_unsigned && !matches!(t.kind(), TypeKind::Int | TypeKind::Char) {
            self.errors.push(ParseError {
                message: format!(
                    "'unsigned' 不能修饰 '{}' 类型",
                    match t.kind() {
                        TypeKind::Float => "float",
                        TypeKind::Double => "double",
                        TypeKind::Struct => "struct",
                        TypeKind::Void => "void",
                        TypeKind::Pointer => "指针",
                        TypeKind::Array => "数组",
                        _ => "此",
                    }
                ),
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
    pub(crate) fn parse_declarator_node(
        &mut self,
        guard: &mut DeclaratorGuard,
        is_abstract: bool,
    ) -> (DeclaratorNode, Option<String>) {
        let mut ptr_prefixes = 0;
        while self.match_token(TokenType::Star) {
            ptr_prefixes += 1;
            if !is_abstract {
                guard.ptr_count += 1;
            }
            // 跳过指针限定符（const/volatile/restrict），教学 VM 中无特殊语义
            while self.match_token(TokenType::Const)
                || self.match_token(TokenType::Volatile)
                || self.match_token(TokenType::Restrict)
            {}
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
                let size_expr = if self.check(TokenType::RBracket) {
                    None
                } else {
                    let expr = self.parse_assign();
                    Some(Box::new(expr))
                };
                self.consume(TokenType::RBracket, "预期 ']'");
                suffixes.push(DeclaratorSuffix::Array(size_expr));
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
                DeclaratorSuffix::Array(size_expr) => {
                    node = DeclaratorNode::Array(Box::new(node), size_expr);
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

    fn array_dim_info(size_expr: &Option<Box<Expr>>) -> (i32, bool, Option<Box<Expr>>) {
        match size_expr {
            None => (-1, false, None),
            Some(expr) => {
                if let Expr::Literal { value, .. } = expr.as_ref() {
                    (*value, false, None)
                } else {
                    (0, true, Some(expr.clone()))
                }
            }
        }
    }

    fn interpret_declarator_node(node: &DeclaratorNode, base_type: &Type) -> Type {
        match node {
            DeclaratorNode::Base => base_type.clone(),
            DeclaratorNode::Pointer(inner) => {
                let inner_ty = Self::interpret_declarator_node(inner, base_type);
                match inner.as_ref() {
                    DeclaratorNode::Array(array_inner, size_expr) => {
                        let elem_ty = Self::interpret_declarator_node(array_inner, base_type);
                        let (size, is_vla, vla_dim) = Self::array_dim_info(size_expr);
                        Type::Array {
                            element: Box::new(Type::pointer_to(elem_ty)),
                            array_size: size,
                            dims: vec![size],
                            is_const: false,
                            is_vla,
                            vla_dims: if let Some(e) = vla_dim { vec![e] } else { vec![] },
                        }
                    }
                    _ => Type::pointer_to(inner_ty),
                }
            }
            DeclaratorNode::Array(inner, size_expr) => {
                let inner_ty = Self::interpret_declarator_node(inner, base_type);
                let (size, is_vla, vla_dim) = Self::array_dim_info(size_expr);
                match inner.as_ref() {
                    DeclaratorNode::Pointer(ptr_inner) => {
                        let elem_ty = Self::interpret_declarator_node(ptr_inner, base_type);
                        Type::Pointer {
                            pointee: Box::new(Type::Array {
                                element: Box::new(elem_ty),
                                array_size: size,
                                dims: vec![size],
                                is_const: false,
                                is_vla,
                                vla_dims: if let Some(e) = vla_dim { vec![e] } else { vec![] },
                            }),
                            is_const: false,
                        }
                    }
                    _ => {
                        let (element, mut inner_dims, inner_array_size, mut inner_is_vla, mut inner_vla_dims) =
                            if let Type::Array {
                                element,
                                dims,
                                array_size,
                                is_vla,
                                vla_dims,
                                ..
                            } = &inner_ty
                            {
                                (element.clone(), dims.clone(), *array_size, *is_vla, vla_dims.clone())
                            } else {
                                (Box::new(inner_ty.clone()), Vec::new(), 1, false, vec![])
                            };
                        inner_dims.push(size);
                        if is_vla {
                            inner_is_vla = true;
                            if let Some(e) = vla_dim {
                                inner_vla_dims.push(e);
                            }
                        }
                        let array_size = if size > 0 { size * inner_array_size } else { size };
                        Type::Array {
                            element,
                            array_size,
                            dims: inner_dims,
                            is_const: false,
                            is_vla: inner_is_vla,
                            vla_dims: inner_vla_dims,
                        }
                    }
                }
            }
            DeclaratorNode::Function(inner, params) => {
                match inner.as_ref() {
                    DeclaratorNode::Pointer(ptr_inner) => {
                        match ptr_inner.as_ref() {
                            DeclaratorNode::Array(array_inner, size_expr) => {
                                // (*fp[N])(params) → function pointer array
                                let elem_ty = Self::interpret_declarator_node(array_inner, base_type);
                                let (size, is_vla, vla_dim) = Self::array_dim_info(size_expr);
                                Type::Array {
                                    element: Box::new(Type::Pointer {
                                        pointee: Box::new(Type::Function {
                                            return_type: Box::new(elem_ty),
                                            param_types: params.iter().map(|p| p.ty.clone()).collect(),
                                            is_const: false,
                                        }),
                                        is_const: false,
                                    }),
                                    array_size: size,
                                    dims: vec![size],
                                    is_const: false,
                                    is_vla,
                                    vla_dims: if let Some(e) = vla_dim { vec![e] } else { vec![] },
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
        if self.check(TokenType::RParen) {
            return params;
        }
        if self.check(TokenType::Void) && self.peek(1).ty == TokenType::RParen {
            self.advance();
            return params;
        }
        loop {
            let base_type = self.parse_base_type();
            let (pty, pname) = if self.check(TokenType::Comma) || self.check(TokenType::RParen) {
                // 无名参数（函数原型声明）：int foo(int);
                (base_type, String::new())
            } else if self.check(TokenType::Star) {
                // 前瞻：跳过所有 * 后看是否是 Comma/RParen，以支持 int foo(char *);
                let lookahead = self.look_ahead_skip_stars();
                if lookahead < self.tokens.len()
                    && (self.tokens[lookahead].ty == TokenType::Comma || self.tokens[lookahead].ty == TokenType::RParen)
                {
                    let mut ty = base_type;
                    while self.match_token(TokenType::Star) {
                        ty = Type::pointer_to(ty);
                    }
                    (ty, String::new())
                } else {
                    self.parse_declarator(&base_type)
                }
            } else {
                self.parse_declarator(&base_type)
            };
            // In C, array parameters in function declarations decay to pointers.
            // For int a[5] -> int*; for int a[3][3] -> int (*a)[3].
            let pty = if pty.is_array() {
                let is_const = pty.is_const();
                Type::Pointer {
                    pointee: Box::new(pty.subscript_type()),
                    is_const,
                }
            } else {
                pty
            };
            params.push(Param {
                ty: pty,
                name: pname,
                loc: SourceLoc {
                    line: self.current().line,
                    column: self.current().column,
                },
            });
            if !self.match_token(TokenType::Comma) {
                break;
            }
        }
        params
    }

    // =========================================================================
    // Statements
    // =========================================================================

    fn parse_statement(&mut self) -> Stmt {
        match self.current().ty {
            TokenType::Semicolon => {
                let loc = SourceLoc {
                    line: self.current().line,
                    column: self.current().column,
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
        Stmt::Block {
            stmts,
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
            },
        }
    }

    fn parse_var_decl_stmt(&mut self, is_static: bool) -> Stmt {
        let loc = self.current().clone();
        let base_type = self.parse_base_type();
        let (var_type, name) = self.parse_declarator(&base_type);
        let init = if self.match_token(TokenType::Assign) {
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
            let (extra_ty, extra_name) = self.parse_declarator(&base_type);
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
            },
        }
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
            },
        }
    }

    fn parse_while_stmt(&mut self) -> Stmt {
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
            },
        }
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
        Stmt::DoWhile {
            body,
            cond,
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
            },
        }
    }

    fn parse_break_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.consume(TokenType::Break, "预期 'break'");
        self.consume(TokenType::Semicolon, "break 后预期 ';'");
        Stmt::Break {
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
            },
        }
    }

    fn parse_continue_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.consume(TokenType::Continue, "预期 'continue'");
        self.consume(TokenType::Semicolon, "continue 后预期 ';'");
        Stmt::Continue {
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
            },
        }
    }

    fn parse_goto_stmt(&mut self) -> Stmt {
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
            },
        }
    }

    fn parse_label_stmt(&mut self) -> Stmt {
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
            },
        }
    }

    fn parse_for_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        self.consume(TokenType::For, "预期 'for'");
        self.consume(TokenType::LParen, "预期 '('");

        // C++ 模式下检测 range for: for (auto x : expr) 或 for (Type x : expr)
        if self.is_cpp_mode {
            let checkpoint = self.pos;
            let is_range_for = if self.is_type_token() || self.check(TokenType::Auto) {
                let _ = self.parse_base_type();
                while self.match_token(TokenType::Star) {}
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
                    },
                };
            }
        }

        let init: Option<Box<Stmt>> = if self.is_type_token() {
            let var_loc = self.current().clone();
            let base_type = self.parse_base_type();
            let (var_type, name) = self.parse_declarator(&base_type);
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
                let (extra_ty, extra_name) = self.parse_declarator(&base_type);
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
                            },
                        })
                        .collect(),
                    loc: SourceLoc {
                        line: es_loc.line,
                        column: es_loc.column,
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
            },
        }
    }

    fn parse_return_stmt(&mut self) -> Stmt {
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
            },
        }
    }

    fn parse_expr_stmt(&mut self) -> Stmt {
        let loc = self.current().clone();
        let expr = self.parse_expression();
        self.consume(TokenType::Semicolon, "预期 ';'");
        Stmt::Expr {
            expr,
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
            },
        }
    }

    // =========================================================================
    // Expressions (precedence climbing)
    // =========================================================================

    fn parse_arg_list(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        if self.check(TokenType::RParen) {
            return args;
        }
        loop {
            args.push(self.parse_assign());
            if !self.match_token(TokenType::Comma) {
                break;
            }
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
        Stmt::Switch {
            cond,
            body,
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
            },
        }
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
            return Stmt::Block {
                stmts: Vec::new(),
                loc: SourceLoc {
                    line: loc.line,
                    column: loc.column,
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
                },
            }
        };
        Stmt::Case {
            label,
            stmt: Box::new(stmt),
            loc: SourceLoc {
                line: loc.line,
                column: loc.column,
            },
        }
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
                let val_expr = self.parse_assign();
                if let Expr::Literal { value, .. } = val_expr {
                    next_value = value;
                }
            }
            program.globals.push(GlobalDecl {
                loc: SourceLoc {
                    line: loc.line,
                    column: loc.column,
                },
                ty: Type::int(),
                name: member_tok.text,
                init: Some(Expr::Literal {
                    value: next_value,
                    loc: SourceLoc {
                        line: member_tok.line,
                        column: member_tok.column,
                    },
                    ty: Type::int(),
                }),
                is_static: false,
                is_extern: false,
                source_file: String::new(),
            });
            next_value += 1;
            if !self.match_token(TokenType::Comma) {
                break;
            }
        }
        self.consume(TokenType::RBrace, "enum 成员后预期 '}'");
        self.consume(TokenType::Semicolon, "enum 声明后预期 ';'");
        if !enum_name.is_empty() {
            self.typedef_names.insert(enum_name, Type::int());
        }
    }
}

mod expr;
