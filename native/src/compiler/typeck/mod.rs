use crate::compiler::ast::*;
use crate::diagnostics::error_codes::ErrorCode;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub line: i32,
    pub column: i32,
    pub code: i32,
}

#[derive(Debug, Clone)]
pub(crate) struct VarSymbol {
    ty: Type,
    #[allow(dead_code)]
    is_global: bool,
    is_extern: bool,
    is_static: bool,
}

#[derive(Debug, Clone)]
struct FuncSymbol {
    return_type: Type,
    param_types: Vec<Type>,
}

#[derive(Debug, Clone)]
struct StructSymbol {
    fields: Vec<(Type, String)>,
}

#[derive(Debug, Clone)]
pub(crate) struct MethodSig {
    pub ret: Type,
    pub param_types: Vec<Type>,
    pub is_virtual: bool,
    #[allow(dead_code)]
    pub is_static: bool,
    pub access: AccessSpec,
}

#[derive(Debug, Clone)]
pub(crate) struct ClassSymbol {
    pub fields: Vec<(Type, String, AccessSpec)>,
    pub methods: HashMap<String, MethodSig>,
    #[allow(dead_code)]
    pub base: Option<String>,
    pub vtable: Option<VTable>,
    pub size: i32,
}

#[derive(Debug, Clone)]
pub(crate) struct TemplateSymbol {
    pub params: Vec<TemplateParam>,
    pub decl: Templateable,
}

#[derive(Default)]
pub struct TypeChecker {
    errors: Vec<TypeError>,
    warnings: Vec<TypeError>,
    hints: Vec<TypeError>,
    funcs: HashMap<String, FuncSymbol>,
    static_func_sigs: HashMap<String, FuncSymbol>,
    static_func_files: HashMap<String, Vec<String>>,
    static_global_files: HashMap<String, Vec<String>>,
    structs: HashMap<String, StructSymbol>,
    unions: HashMap<String, StructSymbol>,
    classes: HashMap<String, ClassSymbol>,
    templates: HashMap<String, TemplateSymbol>,
    scopes: Vec<HashMap<String, VarSymbol>>,
    current_func_return: Type,
    current_file: String,
    loop_depth: i32,
    switch_depth: i32,
    current_func_params: HashSet<String>,
    func_labels: HashMap<String, SourceLoc>,
    pending_gotos: Vec<(String, SourceLoc)>,
    current_class: Option<String>,
    /// Template instantiations discovered during type checking; appended to program.funcs at the end.
    pending_instantiations: Vec<(String, FuncDecl)>,
    /// Class template instantiations discovered during type checking; appended to program.classes at the end.
    pending_class_instantiations: Vec<(String, ClassDecl)>,
    /// Lambdas discovered during type checking; lifted to ClassDecl + FuncDecl at the end.
    pending_lambdas: Vec<LambdaInfo>,
}

#[derive(Debug, Clone)]
struct LambdaInfo {
    id: u64,
    captures: Vec<(String, Type, bool)>, // (name, type, is_by_reference)
    params: Vec<Param>,
    body: Stmt,
    loc: SourceLoc,
}

/// 根据 (from, to) 类型对判断是否允许隐式转换，并返回转换后的目标类型。
fn implicit_cast_target(from: &Type, to: &Type) -> Option<Type> {
    use TypeKind::*;
    // Reference types do not participate in implicit scalar conversions
    if matches!(from.kind(), Reference | RValueRef) || matches!(to.kind(), Reference | RValueRef) {
        return None;
    }
    match (from.kind(), to.kind()) {
        (Int | Char | Float | LongLong, Double) => Some(Type::double()),
        (Double, Int) => Some(Type::Int {
            is_unsigned: to.is_unsigned(),
            is_const: false,
        }),
        (Double, Char) => Some(Type::char()),
        (Double, Float) => Some(Type::float()),
        (Double, LongLong) => Some(Type::LongLong {
            is_unsigned: to.is_unsigned(),
            is_const: false,
        }),
        (Int | Char | LongLong, Float) => Some(Type::float()),
        (Float, Int) => Some(Type::Int {
            is_unsigned: to.is_unsigned(),
            is_const: false,
        }),
        (Float, Char) => Some(Type::char()),
        (Float, LongLong) => Some(Type::LongLong {
            is_unsigned: to.is_unsigned(),
            is_const: false,
        }),
        (Int | Char, LongLong) => Some(Type::LongLong {
            is_unsigned: to.is_unsigned(),
            is_const: false,
        }),
        (LongLong, Int) => Some(Type::Int {
            is_unsigned: to.is_unsigned(),
            is_const: false,
        }),
        (LongLong, Char) => Some(Type::char()),
        _ => None,
    }
}

fn insert_implicit_cast(expr: &mut Expr, target: &Type) {
    let current_ty = expr.ty().clone();
    if target.kind() == TypeKind::Double && matches!(expr, Expr::FloatLiteral { .. }) {
        expr.set_ty(Type::double());
        return;
    }
    if let Some(target_ty) = implicit_cast_target(&current_ty, target) {
        let loc = *expr.loc();
        let old = std::mem::take(expr);
        *expr = Expr::Cast {
            expr: Box::new(old),
            target_type: target_ty.clone(),
            loc,
            ty: target_ty,
        };
    }
}

impl TypeChecker {
    /// Compute the byte size of a type using the registered struct/union definitions.
    pub fn compute_type_size(&self, ty: &Type) -> i32 {
        let struct_defs: HashMap<String, Vec<StructField>> = self
            .structs
            .iter()
            .map(|(name, sym)| {
                let fields: Vec<StructField> = sym
                    .fields
                    .iter()
                    .map(|(ty, name)| StructField {
                        ty: ty.clone(),
                        name: name.clone(),
                    })
                    .collect();
                (name.clone(), fields)
            })
            .collect();
        let union_defs: HashMap<String, Vec<StructField>> = self
            .unions
            .iter()
            .map(|(name, sym)| {
                let fields: Vec<StructField> = sym
                    .fields
                    .iter()
                    .map(|(ty, name)| StructField {
                        ty: ty.clone(),
                        name: name.clone(),
                    })
                    .collect();
                (name.clone(), fields)
            })
            .collect();
        let class_size_map: HashMap<String, i32> =
            self.classes.iter().map(|(name, sym)| (name.clone(), sym.size)).collect();
        crate::compiler::ast::compute_type_size(ty, &struct_defs, &union_defs, &class_size_map)
    }

    pub fn check(mut self, program: &mut ProgramNode) -> (Vec<TypeError>, Vec<TypeError>, Vec<TypeError>) {
        // Pass 1: Register structs and unions
        for s in &program.structs {
            if self.structs.contains_key(&s.name) {
                self.report_error(
                    &format!("结构体 '{}' 重复定义", s.name),
                    &s.loc,
                    ErrorCode::E3002_StructRedeclared,
                );
                continue;
            }
            let sym = StructSymbol {
                fields: s.fields.iter().map(|f| (f.ty.clone(), f.name.clone())).collect(),
            };
            self.structs.insert(s.name.clone(), sym);
        }
        for u in &program.unions {
            if self.unions.contains_key(&u.name) {
                self.report_error(
                    &format!("联合体 '{}' 重复定义", u.name),
                    &u.loc,
                    ErrorCode::E3002_StructRedeclared,
                );
                continue;
            }
            let sym = StructSymbol {
                fields: u.fields.iter().map(|f| (f.ty.clone(), f.name.clone())).collect(),
            };
            self.unions.insert(u.name.clone(), sym);
        }

        // Pass 1.5: Register classes and compute layouts
        self.register_class_layouts(program);

        // Pass 2: Register function signatures (including class methods as mangled funcs)
        for f in &program.funcs {
            let new_sym = FuncSymbol {
                return_type: f.return_type.clone(),
                param_types: f.params.iter().map(|p| p.ty.clone()).collect(),
            };
            if f.is_static {
                if let Some(existing) = self.static_func_sigs.get(&f.name) {
                    if existing.return_type != new_sym.return_type || existing.param_types != new_sym.param_types {
                        self.report_error(
                            &format!("函数 '{}' 的声明与之前定义签名不一致", f.name),
                            &f.loc,
                            ErrorCode::E3003_FuncRedeclared,
                        );
                    }
                } else {
                    self.static_func_sigs.insert(f.name.clone(), new_sym);
                }
                self.static_func_files
                    .entry(f.name.clone())
                    .or_default()
                    .push(f.source_file.clone());
            } else {
                if let Some(existing) = self.funcs.get(&f.name) {
                    if existing.return_type != new_sym.return_type || existing.param_types != new_sym.param_types {
                        self.report_error(
                            &format!("函数 '{}' 的声明与之前定义签名不一致", f.name),
                            &f.loc,
                            ErrorCode::E3003_FuncRedeclared,
                        );
                    }
                    continue;
                }
                self.funcs.insert(f.name.clone(), new_sym);
            }
        }

        // Pass 2.3: Register class methods as mangled global functions
        for c in &program.classes {
            for member in &c.members {
                if let ClassMember::Method { name, ret, params, .. } = member {
                    let mangled = format!("{}__{}", c.name, name);
                    let param_types: Vec<Type> = std::iter::once(Type::Pointer {
                        pointee: Box::new(Type::Class {
                            name: c.name.clone(),
                            is_const: false,
                        }),
                        is_const: false,
                    })
                    .chain(params.iter().map(|p| p.ty.clone()))
                    .collect();
                    let new_sym = FuncSymbol {
                        return_type: ret.clone(),
                        param_types,
                    };
                    if let Some(existing) = self.funcs.get(&mangled) {
                        if existing.return_type != new_sym.return_type || existing.param_types != new_sym.param_types {
                            self.report_error(
                                &format!("方法 '{}' 的声明与之前定义签名不一致", mangled),
                                &c.loc,
                                ErrorCode::E3003_FuncRedeclared,
                            );
                        }
                        continue;
                    }
                    self.funcs.insert(mangled, new_sym);
                }
                if let ClassMember::Constructor { params, .. } = member {
                    let mangled = format!("__ctor__{}", c.name);
                    let param_types: Vec<Type> = std::iter::once(Type::Pointer {
                        pointee: Box::new(Type::Class {
                            name: c.name.clone(),
                            is_const: false,
                        }),
                        is_const: false,
                    })
                    .chain(params.iter().map(|p| p.ty.clone()))
                    .collect();
                    let new_sym = FuncSymbol {
                        return_type: Type::void(),
                        param_types,
                    };
                    if self.funcs.contains_key(&mangled) {
                        continue;
                    }
                    self.funcs.insert(mangled, new_sym);
                }
                if let ClassMember::Destructor { .. } = member {
                    let mangled = format!("__dtor__{}", c.name);
                    let param_types = vec![Type::Pointer {
                        pointee: Box::new(Type::Class {
                            name: c.name.clone(),
                            is_const: false,
                        }),
                        is_const: false,
                    }];
                    let new_sym = FuncSymbol {
                        return_type: Type::void(),
                        param_types,
                    };
                    if self.funcs.contains_key(&mangled) {
                        continue;
                    }
                    self.funcs.insert(mangled, new_sym);
                }
            }
        }

        // Pass 2.6: Register templates
        for t in &program.templates {
            let name = match &t.decl {
                Templateable::Func(f) => f.name.clone(),
                Templateable::Class(c) => c.name.clone(),
            };
            if self.templates.contains_key(&name) {
                self.report_error(&format!("模板 '{}' 重复定义", name), &t.loc, ErrorCode::E3003_FuncRedeclared);
                continue;
            }
            let sym = TemplateSymbol {
                params: t.params.clone(),
                decl: t.decl.clone(),
            };
            self.templates.insert(name, sym);
        }

        // Pass 2.5: Register globals and check initializers
        self.enter_scope();
        for g in &mut program.globals {
            self.declare_var(&g.name, &g.ty, true, g.is_extern, g.is_static);
            if g.is_static {
                self.static_global_files
                    .entry(g.name.clone())
                    .or_default()
                    .push(g.source_file.clone());
            }
        }
        for g in &mut program.globals {
            if let Some(ref mut init) = g.init {
                if g.ty.is_array() {
                    self.check_array_initializer(&mut g.ty, init, &g.loc);
                } else if g.ty.is_struct() && matches!(init, Expr::InitList { .. }) {
                    self.check_struct_initializer(&g.ty, init, &g.loc);
                } else {
                    let init_type = self.resolve_expr_type(init);
                    if !self.check_assignable(&g.ty, &init_type, &g.loc) {
                        self.report_error(
                            &format!("类型不匹配：无法将 '{}' 赋值给 '{}'", init_type, g.ty),
                            &g.loc,
                            ErrorCode::E3004_TypeMismatch,
                        );
                    }
                }
            }
        }

        // Pass 3: Check function bodies
        for f in &mut program.funcs {
            if f.body.is_some() {
                self.visit_func_decl(f);
            }
        }

        // Drain class template instantiations discovered during Pass 3
        // so Pass 3.5 can check their methods.
        let pending_classes: Vec<_> = self.pending_class_instantiations.drain(..).collect();
        for (_name, c) in pending_classes {
            program.classes.push(c);
        }

        // Pass 3.5: Check class method / constructor / destructor bodies
        self.check_class_methods(program);

        self.exit_scope();

        // Append template instantiations discovered during type checking
        for (_, f) in self.pending_instantiations.drain(..) {
            program.funcs.push(f);
        }

        // Pass 4: Lift lambdas to ClassDecl + FuncDecl
        let lambdas: Vec<_> = self.pending_lambdas.drain(..).collect();
        for info in lambdas {
            let lambda_name = format!("__lambda_{}", info.id);
            let call_name = format!("{}__call", lambda_name);

            // Create ClassDecl with capture fields
            let class_members: Vec<ClassMember> = info
                .captures
                .iter()
                .map(|(name, ty, _)| ClassMember::Field {
                    name: name.clone(),
                    ty: ty.clone(),
                    access: AccessSpec::Public,
                })
                .collect();
            program.classes.push(ClassDecl {
                loc: info.loc,
                name: lambda_name.clone(),
                base: None,
                members: class_members,
                vtable: None,
            });

            // Create FuncDecl for __call
            let mut call_params = vec![Param {
                name: "this".to_string(),
                ty: Type::Pointer {
                    pointee: Box::new(Type::Class {
                        name: lambda_name.clone(),
                        is_const: false,
                    }),
                    is_const: false,
                },
                loc: info.loc,
            }];
            call_params.extend(info.params.iter().cloned());

            let mut func_decl = FuncDecl {
                loc: info.loc,
                return_type: Type::int(),
                name: call_name,
                params: call_params,
                body: Some(info.body),
                is_static: false,
                is_extern: false,
                source_file: self.current_file.clone(),
            };

            // Rewrite capture variable accesses to this->field
            if let Some(ref mut body) = func_decl.body {
                Self::rewrite_lambda_captures(body, &info.captures, &lambda_name);
            }

            // Type-check the generated function body
            self.current_class = Some(lambda_name.clone());
            self.visit_func_decl(&mut func_decl);
            self.current_class = None;
            program.funcs.push(func_decl);
        }

        (self.errors, self.warnings, self.hints)
    }

    // =========================================================================
    // Scope management
    // =========================================================================

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare_var(&mut self, name: &str, ty: &Type, is_global: bool, is_extern: bool, is_static: bool) {
        if self.scopes.is_empty() {
            self.scopes.push(HashMap::new());
        }
        let scope = self.scopes.last_mut().expect("scopes 在上一步已确保非空");
        if let Some(existing) = scope.get(name) {
            if is_extern {
                // extern declaration of an existing symbol is allowed
                return;
            }
            // Non-extern definition can replace an extern declaration
            if existing.is_extern {
                scope.insert(
                    name.to_string(),
                    VarSymbol {
                        ty: ty.clone(),
                        is_global,
                        is_extern,
                        is_static,
                    },
                );
                return;
            }
            // Multiple static globals with the same name in different files are allowed
            // (internal linkage). We keep the latest one; access check is done at use site.
            if existing.is_static && is_static && is_global {
                scope.insert(
                    name.to_string(),
                    VarSymbol {
                        ty: ty.clone(),
                        is_global,
                        is_extern,
                        is_static,
                    },
                );
                return;
            }
            self.report_error(
                &format!("变量 '{}' 已在此作用域中声明", name),
                &SourceLoc { line: 0, column: 0 },
                ErrorCode::E3001_VarRedeclared,
            );
            return;
        }
        scope.insert(
            name.to_string(),
            VarSymbol {
                ty: ty.clone(),
                is_global,
                is_extern,
                is_static,
            },
        );
    }

    pub(crate) fn lookup_var(&self, name: &str) -> Option<VarSymbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Some(sym.clone());
            }
        }
        None
    }

    // =========================================================================
    // Type operations
    // =========================================================================

    pub(crate) fn report_error(&mut self, msg: &str, loc: &SourceLoc, code: ErrorCode) {
        self.errors.push(TypeError {
            message: msg.to_string(),
            line: loc.line,
            column: loc.column,
            code: code as i32,
        });
    }

    pub(crate) fn report_warning(&mut self, msg: &str, loc: &SourceLoc, code: ErrorCode) {
        self.warnings.push(TypeError {
            message: msg.to_string(),
            line: loc.line,
            column: loc.column,
            code: code as i32,
        });
    }

    fn report_hint(&mut self, msg: &str, loc: &SourceLoc, code: ErrorCode) {
        self.hints.push(TypeError {
            message: msg.to_string(),
            line: loc.line,
            column: loc.column,
            code: code as i32,
        });
    }

    pub(crate) fn is_int(&self, t: &Type) -> bool {
        matches!(t.kind(), TypeKind::Int | TypeKind::Char)
    }
    pub(crate) fn is_scalar(&self, t: &Type) -> bool {
        matches!(
            t.kind(),
            TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double | TypeKind::LongLong
        )
    }

    pub(crate) fn is_comparable(&self, a: &Type, b: &Type) -> bool {
        if matches!(a.kind(), TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double)
            && matches!(b.kind(), TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double)
        {
            return true;
        }
        if matches!(a.kind(), TypeKind::Pointer) && matches!(b.kind(), TypeKind::Pointer) {
            return true;
        }
        if matches!(a.kind(), TypeKind::Pointer) && matches!(b.kind(), TypeKind::Array) {
            return true;
        }
        if matches!(a.kind(), TypeKind::Array) && matches!(b.kind(), TypeKind::Pointer) {
            return true;
        }
        if matches!(a.kind(), TypeKind::Pointer) && matches!(b.kind(), TypeKind::Int) {
            return true;
        }
        if matches!(a.kind(), TypeKind::Int) && matches!(b.kind(), TypeKind::Pointer) {
            return true;
        }
        false
    }

    fn check_array_pointer_assignable(&mut self, target: &Type, value: &Type, loc: &SourceLoc) -> bool {
        if matches!(target.kind(), TypeKind::Pointer) && matches!(value.kind(), TypeKind::Array) {
            if let (Type::Pointer { pointee: t_pointee, .. }, Type::Array { element: v_element, .. }) = (target, value)
            {
                if t_pointee.as_ref() == v_element.as_ref() {
                    self.report_warning(
                        "数组隐式转换为指针。数组名在表达式中会自动退化为指向首元素的指针。",
                        loc,
                        ErrorCode::W3052_ArrayToPointerDecay,
                    );
                    return true;
                }
                // Multidimensional array decay: int arr[3][3] -> int (*)[3]
                if t_pointee.as_ref() == &value.subscript_type() {
                    self.report_warning(
                        "数组隐式转换为指针。数组名在表达式中会自动退化为指向首元素的指针。",
                        loc,
                        ErrorCode::W3052_ArrayToPointerDecay,
                    );
                    return true;
                }
            }
        }
        if matches!(target.kind(), TypeKind::Array) && matches!(value.kind(), TypeKind::Pointer) {
            if let (Type::Array { element: t_element, .. }, Type::Pointer { pointee: v_pointee, .. }) = (target, value)
            {
                if t_element == v_pointee {
                    return true;
                }
            }
        }
        if matches!(target.kind(), TypeKind::Array) && matches!(value.kind(), TypeKind::Array) {
            if let (Type::Array { element: t_element, .. }, Type::Array { element: v_element, .. }) = (target, value) {
                if t_element == v_element {
                    let check_count = target.dims().len().min(value.dims().len());
                    let mut dims_compatible = true;
                    for i in 0..check_count {
                        if target.dims()[i] > 0 && target.dims()[i] != value.dims()[i] {
                            dims_compatible = false;
                            break;
                        }
                    }
                    if dims_compatible {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn check_function_pointer_assignable(&mut self, target: &Type, value: &Type, loc: &SourceLoc) -> bool {
        if target.is_function_pointer() && value.is_function_pointer() {
            if let (Type::Pointer { pointee: t_pointee, .. }, Type::Pointer { pointee: v_pointee, .. }) =
                (target, value)
            {
                if let (
                    Type::Function {
                        return_type: t_ret,
                        param_types: t_params,
                        ..
                    },
                    Type::Function {
                        return_type: v_ret,
                        param_types: v_params,
                        ..
                    },
                ) = (t_pointee.as_ref(), v_pointee.as_ref())
                {
                    if t_params.len() == v_params.len() {
                        let params_compatible = t_params.iter().zip(v_params.iter()).all(|(a, b)| a == b);
                        if params_compatible && t_ret == v_ret {
                            return true;
                        }
                    }
                }
            }
            self.report_warning(
                "函数指针类型不完全匹配，赋值可能存在风险",
                loc,
                ErrorCode::W3053_ImplicitScalarConversion,
            );
            return true;
        }
        if target.is_pointer() && value.is_function_pointer() {
            return true;
        }
        if target.is_function_pointer() && value.is_pointer() {
            self.report_warning("将通用指针赋值给函数指针，建议显式转换", loc, ErrorCode::W3055_VoidPointerCast);
            return true;
        }
        false
    }

    fn check_scalar_assignable(&mut self, target: &Type, value: &Type, loc: &SourceLoc) -> bool {
        if !matches!(
            target.kind(),
            TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double | TypeKind::LongLong
        ) {
            return false;
        }
        if !matches!(
            value.kind(),
            TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double | TypeKind::LongLong
        ) {
            return false;
        }
        // 警告可能丢失精度的情况
        if matches!(target.kind(), TypeKind::Char)
            && matches!(
                value.kind(),
                TypeKind::Int | TypeKind::Float | TypeKind::Double | TypeKind::LongLong
            )
        {
            self.report_warning(
                "被隐式转换为 char，可能会丢失精度。",
                loc,
                ErrorCode::W3053_ImplicitScalarConversion,
            );
        }
        if matches!(target.kind(), TypeKind::Int)
            && matches!(value.kind(), TypeKind::Float | TypeKind::Double | TypeKind::LongLong)
        {
            self.report_warning(
                &format!("{} 被隐式转换为 int，可能会丢失精度。", value),
                loc,
                ErrorCode::W3053_ImplicitScalarConversion,
            );
        }
        if matches!(target.kind(), TypeKind::Float) && matches!(value.kind(), TypeKind::Double | TypeKind::LongLong) {
            self.report_warning(
                "double 被隐式转换为 float，可能会丢失精度。",
                loc,
                ErrorCode::W3053_ImplicitScalarConversion,
            );
        }
        // 提示安全的隐式提升
        if matches!(target.kind(), TypeKind::Int) && matches!(value.kind(), TypeKind::Char) {
            self.report_hint("char 被隐式提升为 int。", loc, ErrorCode::H3057_ImplicitConversionHint);
        }
        if matches!(target.kind(), TypeKind::Float) && matches!(value.kind(), TypeKind::Int | TypeKind::Char) {
            let src = if matches!(value.kind(), TypeKind::Char) {
                "char"
            } else {
                "int"
            };
            self.report_hint(
                &format!("{} 被隐式提升为 float。", src),
                loc,
                ErrorCode::H3057_ImplicitConversionHint,
            );
        }
        if matches!(target.kind(), TypeKind::Double)
            && matches!(
                value.kind(),
                TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::LongLong
            )
        {
            let src = match value.kind() {
                TypeKind::Char => "char",
                TypeKind::Float => "float",
                TypeKind::LongLong => "long long",
                _ => "int",
            };
            self.report_hint(
                &format!("{} 被隐式提升为 double。", src),
                loc,
                ErrorCode::H3057_ImplicitConversionHint,
            );
        }
        true
    }

    fn check_pointer_assignable(&mut self, target: &Type, value: &Type, loc: &SourceLoc) -> bool {
        if matches!(target.kind(), TypeKind::Pointer) && matches!(value.kind(), TypeKind::Int) {
            self.report_warning(
                "整数被隐式转换为指针。建议确保这是有意义的地址值（如 NULL = 0）。",
                loc,
                ErrorCode::W3054_IntToPointerCast,
            );
            return true;
        }
        // C 标准：任意指针或数组都可以隐式转换为 void*（Phase D 补齐 Host 函数所需）
        if matches!(target.kind(), TypeKind::Pointer) {
            if let Type::Pointer { pointee, .. } = target {
                if matches!(pointee.as_ref(), Type::Void { .. })
                    && matches!(value.kind(), TypeKind::Pointer | TypeKind::Array)
                {
                    self.report_hint("具体指针类型被隐式转换为 void*。", loc, ErrorCode::H3057_ImplicitConversionHint);
                    return true;
                }
            }
        }
        if matches!(target.kind(), TypeKind::Pointer) && matches!(value.kind(), TypeKind::Pointer) {
            if let (
                Type::Pointer { is_const: t_const, .. },
                Type::Pointer {
                    pointee: v_pointee,
                    is_const: v_const,
                },
            ) = (target, value)
            {
                if matches!(v_pointee.as_ref(), Type::Void { .. }) {
                    self.report_hint("void* 被隐式转换为具体指针类型。", loc, ErrorCode::H3057_ImplicitConversionHint);
                }
                if *v_const && !*t_const {
                    self.report_warning(
                        "将 const 指针赋值给非 const 指针，可能通过后者修改 const 数据。",
                        loc,
                        ErrorCode::W3053_ImplicitScalarConversion,
                    );
                }
            }
            return true;
        }
        false
    }

    pub(crate) fn check_assignable(&mut self, target: &Type, value: &Type, loc: &SourceLoc) -> bool {
        if target == value {
            return true;
        }
        // Reference type compatibility (basic type check, lvalue/rvalue checked at use site)
        if let Type::Reference {
            base: t_base,
            is_const: t_const,
        } = target
        {
            if let Type::Reference {
                base: v_base,
                is_const: v_const,
            } = value
            {
                // Reference to reference: only const& can bind to const&
                if t_base == v_base && (*t_const || !*v_const) {
                    return true;
                }
                return false;
            }
            // Non-reference value binding to reference: check base type compatibility
            if t_base.as_ref() == value {
                return true;
            }
            if t_base.kind() == value.kind() && matches!(t_base.kind(), TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double | TypeKind::LongLong) {
                return true;
            }
            if let Type::Pointer { pointee: t_pt, .. } = t_base.as_ref() {
                if let Type::Pointer { pointee: v_pt, .. } = value {
                    if t_pt == v_pt || matches!(t_pt.as_ref(), Type::Void { .. }) {
                        return true;
                    }
                }
            }
            return false;
        }
        if let Type::RValueRef { base: t_base } = target {
            if t_base.as_ref() == value {
                return true;
            }
            if self.check_scalar_assignable(t_base, value, loc) {
                return true;
            }
            return false;
        }
        if self.check_array_pointer_assignable(target, value, loc) {
            return true;
        }
        if self.check_function_pointer_assignable(target, value, loc) {
            return true;
        }
        if self.check_scalar_assignable(target, value, loc) {
            return true;
        }
        if self.check_pointer_assignable(target, value, loc) {
            return true;
        }
        false
    }

    /// 判断表达式是否为左值（可被取地址的表达式）。
    /// 尝试隐式实例化函数模板，返回 (mangled_name, FuncDecl) 但不注册到 program。
    pub(crate) fn try_instantiate_template(&mut self, name: &str, arg_types: &[Type]) -> Option<(String, FuncDecl)> {
        self.try_monomorphize_func(name, arg_types)
    }

    pub(crate) fn is_lvalue(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Identifier { .. } => true,
            Expr::Member { .. } => true,
            Expr::Index { .. } => true,
            Expr::Unary { op: UnaryOp::Deref, .. } => true,
            Expr::Call { .. } | Expr::CallPtr { .. } => expr.ty().is_reference() || expr.ty().is_rvalue_ref(),
            _ => false,
        }
    }

    pub(crate) fn get_struct_field_type(&self, struct_name: &str, field_name: &str) -> Option<Type> {
        let sym = self.structs.get(struct_name)?;
        for (fty, fname) in &sym.fields {
            if fname == field_name {
                return Some(fty.clone());
            }
        }
        None
    }
    pub(crate) fn get_union_field_type(&self, union_name: &str, field_name: &str) -> Option<Type> {
        let sym = self.unions.get(union_name)?;
        for (fty, fname) in &sym.fields {
            if fname == field_name {
                return Some(fty.clone());
            }
        }
        None
    }
    #[allow(dead_code)]
    pub(crate) fn get_class_field_type(&self, class_name: &str, field_name: &str) -> Option<Type> {
        let sym = self.classes.get(class_name)?;
        for (fty, fname, _) in &sym.fields {
            if fname == field_name {
                return Some(fty.clone());
            }
        }
        None
    }
    pub(crate) fn get_class_field_type_with_access(
        &self,
        class_name: &str,
        field_name: &str,
    ) -> (Option<Type>, Option<AccessSpec>) {
        let sym = match self.classes.get(class_name) {
            Some(s) => s,
            None => return (None, None),
        };
        for (fty, fname, faccess) in &sym.fields {
            if fname == field_name {
                return (Some(fty.clone()), Some(*faccess));
            }
        }
        (None, None)
    }
    pub(crate) fn find_class_method(&self, class_name: &str, method_name: &str) -> Option<MethodSig> {
        let sym = self.classes.get(class_name)?;
        sym.methods.get(method_name).cloned()
    }

    fn expr_involves_array_or_pointer(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Index { .. } => true,
            Expr::Identifier { name, .. } => self
                .lookup_var(name)
                .map(|s| s.ty.is_array() || s.ty.is_pointer())
                .unwrap_or(false),
            Expr::Binary { left, right, .. } => {
                self.expr_involves_array_or_pointer(left) || self.expr_involves_array_or_pointer(right)
            }
            Expr::Unary { operand, .. } => self.expr_involves_array_or_pointer(operand),
            Expr::Assign { left, right, .. } => {
                self.expr_involves_array_or_pointer(left) || self.expr_involves_array_or_pointer(right)
            }
            Expr::Ternary {
                cond, then_branch, else_branch, ..
            } => {
                self.expr_involves_array_or_pointer(cond)
                    || self.expr_involves_array_or_pointer(then_branch)
                    || self.expr_involves_array_or_pointer(else_branch)
            }
            _ => false,
        }
    }

    // =========================================================================
    // Initializer checks
    // =========================================================================

    fn check_struct_initializer(&mut self, struct_type: &Type, init: &mut Expr, loc: &SourceLoc) {
        if !matches!(init, Expr::InitList { .. }) {
            let init_type = self.resolve_expr_type(init);
            if !self.check_assignable(struct_type, &init_type, loc) {
                self.report_error(
                    &format!("类型不匹配：无法将 '{}' 赋值给 '{}'", init_type, struct_type),
                    loc,
                    ErrorCode::E3004_TypeMismatch,
                );
            }
            return;
        }
        let elements = match init {
            Expr::InitList { elements, .. } => elements.as_mut_slice(),
            _ => return,
        };
        let fields = match self.structs.get(struct_type.name()) {
            Some(s) => s.fields.clone(),
            None => {
                self.report_error(
                    &format!("未知的结构体类型 '{}'", struct_type.name()),
                    loc,
                    ErrorCode::E3004_TypeMismatch,
                );
                return;
            }
        };
        let has_designators = elements.iter().any(|e| !e.designators.is_empty());
        if has_designators {
            for elem in elements.iter_mut() {
                if elem.designators.is_empty() {
                    self.report_error(
                        "初始化列表中不能混合使用指定初始化和非指定初始化",
                        loc,
                        ErrorCode::E3005_ArrayInitTooMany,
                    );
                    continue;
                }
                if elem.designators.len() != 1 {
                    self.report_error("暂不支持多级 designated initializer", loc, ErrorCode::E3005_ArrayInitTooMany);
                    continue;
                }
                match &elem.designators[0] {
                    Designator::Field(field_name) => {
                        if let Some(field_idx) = fields.iter().position(|f| &f.1 == field_name) {
                            let field_ty = &fields[field_idx].0;
                            if field_ty.is_struct() && matches!(&elem.value, Expr::InitList { .. }) {
                                self.check_struct_initializer(field_ty, &mut elem.value, loc);
                            } else if field_ty.is_array() && matches!(&elem.value, Expr::InitList { .. }) {
                                let mut sub_ty = field_ty.clone();
                                self.check_array_initializer(&mut sub_ty, &mut elem.value, loc);
                            } else {
                                let e_type = self.resolve_expr_type(&mut elem.value);
                                if !self.check_assignable(field_ty, &e_type, loc) {
                                    self.report_error(
                                        &format!(
                                            "结构体初始化类型不匹配：字段 '{}' 期望 '{}'，实际 '{}'",
                                            field_name, field_ty, e_type
                                        ),
                                        loc,
                                        ErrorCode::E3006_ArrayInitTypeMismatch,
                                    );
                                } else {
                                    insert_implicit_cast(&mut elem.value, field_ty);
                                }
                            }
                        } else {
                            self.report_error(
                                &format!("结构体 '{}' 没有字段 '{}'", struct_type.name(), field_name),
                                loc,
                                ErrorCode::E3042_UnknownMember,
                            );
                        }
                    }
                    _ => {
                        self.report_error(
                            "结构体初始化只能使用 .field 形式的 designator",
                            loc,
                            ErrorCode::E3005_ArrayInitTooMany,
                        );
                    }
                }
            }
            return;
        }
        if elements.len() > fields.len() {
            self.report_error("初始化列表元素数量超过结构体字段数", loc, ErrorCode::E3005_ArrayInitTooMany);
        }
        for (i, elem) in elements.iter_mut().enumerate() {
            if i >= fields.len() {
                break;
            }
            if fields[i].0.is_struct() && matches!(&elem.value, Expr::InitList { .. }) {
                self.check_struct_initializer(&fields[i].0, elem, loc);
            } else if fields[i].0.is_array() && matches!(&elem.value, Expr::InitList { .. }) {
                let mut sub_ty = fields[i].0.clone();
                self.check_array_initializer(&mut sub_ty, elem, loc);
            } else {
                let e_type = self.resolve_expr_type(elem);
                if !self.check_assignable(&fields[i].0, &e_type, loc) {
                    self.report_error(
                        &format!(
                            "结构体初始化类型不匹配：字段 '{}' 期望 '{}'，实际 '{}'",
                            fields[i].1, fields[i].0, e_type
                        ),
                        loc,
                        ErrorCode::E3006_ArrayInitTypeMismatch,
                    );
                }
            }
        }
    }

    fn validate_nested_init_list(
        &mut self,
        dims: &[i32],
        init: &mut Expr,
        loc: &SourceLoc,
        element_type: &Type,
    ) -> bool {
        if dims.is_empty() {
            if element_type.is_struct() && matches!(init, Expr::InitList { .. }) {
                self.check_struct_initializer(element_type, init, loc);
                return true;
            }
            if element_type.is_array() && matches!(init, Expr::InitList { .. }) {
                let mut sub_ty = element_type.clone();
                self.check_array_initializer(&mut sub_ty, init, loc);
                return true;
            }
            let e_type = self.resolve_expr_type(init);
            if !self.check_assignable(element_type, &e_type, loc) {
                self.report_error(
                    &format!("数组初始化元素类型不匹配：期望 '{}'，实际 '{}'", element_type, e_type),
                    loc,
                    ErrorCode::E3006_ArrayInitTypeMismatch,
                );
                return false;
            }
            insert_implicit_cast(init, element_type);
            return true;
        }
        if !matches!(init, Expr::InitList { .. }) {
            self.report_error("多维数组初始化需要嵌套初始化列表", loc, ErrorCode::E3009_InvalidArrayInit);
            return false;
        }
        let elements = match init {
            Expr::InitList { elements, .. } => elements.as_mut_slice(),
            _ => return false,
        };
        let expected_count = if dims[0] > 0 { dims[0] as usize } else { elements.len() };
        if elements.len() > expected_count {
            self.report_error("初始化列表元素数量超过数组维度大小", loc, ErrorCode::E3005_ArrayInitTooMany);
        }
        for elem in elements {
            if !self.validate_nested_init_list(&dims[1..], elem, loc, element_type) {
                return false;
            }
        }
        true
    }

    fn check_array_initializer(&mut self, arr_type: &mut Type, init: &mut Expr, loc: &SourceLoc) {
        let elem_type = arr_type.innermost_element_type();

        if !arr_type.dims().is_empty() && arr_type.dims().len() > 1 {
            if let Expr::InitList { elements, .. } = init {
                let has_designators = elements.iter().any(|e| !e.designators.is_empty());
                if has_designators {
                    self.report_error(
                        "多维数组暂不支持 designated initializer",
                        loc,
                        ErrorCode::E3009_InvalidArrayInit,
                    );
                    return;
                }
                let total_elems = arr_type.total_elements();
                if let Type::Array { dims, array_size, .. } = arr_type {
                    if dims[0] <= 0 {
                        dims[0] = elements.len() as i32;
                        *array_size = total_elems;
                    }
                    let dims_copy = dims.clone();
                    self.validate_nested_init_list(&dims_copy, init, loc, &elem_type);
                }
            } else {
                let init_type = self.resolve_expr_type(init);
                self.report_error(
                    &format!("多维数组初始化必须使用嵌套初始化列表，不能是 '{}'", init_type),
                    loc,
                    ErrorCode::E3009_InvalidArrayInit,
                );
            }
            return;
        }

        if let Expr::InitList { elements, .. } = init {
            let has_designators = elements.iter().any(|e| !e.designators.is_empty());
            if has_designators {
                for elem in elements.iter_mut() {
                    if elem.designators.is_empty() {
                        self.report_error(
                            "初始化列表中不能混合使用指定初始化和非指定初始化",
                            loc,
                            ErrorCode::E3005_ArrayInitTooMany,
                        );
                        continue;
                    }
                    if elem.designators.len() != 1 {
                        self.report_error(
                            "暂不支持多级 designated initializer",
                            loc,
                            ErrorCode::E3005_ArrayInitTooMany,
                        );
                        continue;
                    }
                    match &mut elem.designators[0] {
                        Designator::Index(idx_expr) => {
                            let idx_ty = self.resolve_expr_type(idx_expr);
                            if !self.is_int(&idx_ty) {
                                self.report_error("数组索引必须是 int 类型", loc, ErrorCode::E3039_ArrayIndexType);
                            }
                            let e_type = self.resolve_expr_type(&mut elem.value);
                            if !self.check_assignable(&elem_type, &e_type, loc) {
                                self.report_error(
                                    &format!("数组初始化元素类型不匹配：期望 '{}'，实际 '{}'", elem_type, e_type),
                                    loc,
                                    ErrorCode::E3006_ArrayInitTypeMismatch,
                                );
                            } else {
                                insert_implicit_cast(&mut elem.value, &elem_type);
                            }
                        }
                        _ => {
                            self.report_error(
                                "数组初始化只能使用 [index] 形式的 designator",
                                loc,
                                ErrorCode::E3005_ArrayInitTooMany,
                            );
                        }
                    }
                }
                return;
            }
            let mut expected_size = arr_type.array_size();
            if expected_size <= 0 {
                expected_size = elements.len() as i32;
                if let Type::Array { array_size, .. } = arr_type {
                    *array_size = expected_size;
                }
            }
            if elements.len() > expected_size as usize {
                self.report_error("初始化列表元素数量超过数组大小", loc, ErrorCode::E3005_ArrayInitTooMany);
            }
            for elem in elements.iter_mut() {
                if elem_type.is_struct() && matches!(&elem.value, Expr::InitList { .. }) {
                    self.check_struct_initializer(&elem_type, elem, loc);
                } else if elem_type.is_array() && matches!(&elem.value, Expr::InitList { .. }) {
                    let mut sub_ty = elem_type.clone();
                    self.check_array_initializer(&mut sub_ty, elem, loc);
                } else {
                    let e_type = self.resolve_expr_type(elem);
                    if !self.check_assignable(&elem_type, &e_type, loc) {
                        self.report_error(
                            &format!("数组初始化元素类型不匹配：期望 '{}'，实际 '{}'", elem_type, e_type),
                            loc,
                            ErrorCode::E3006_ArrayInitTypeMismatch,
                        );
                    } else {
                        insert_implicit_cast(elem, &elem_type);
                    }
                }
            }
        } else if let Expr::StringLiteral { value, .. } = init {
            if elem_type.kind() != TypeKind::Char {
                self.report_error(
                    "字符串字面量只能用于初始化 char 数组",
                    loc,
                    ErrorCode::E3007_StringInitNonCharArray,
                );
                return;
            }
            let str_len = value.len() as i32;
            if arr_type.array_size() <= 0 {
                if let Type::Array { array_size, .. } = arr_type {
                    *array_size = str_len + 1;
                }
            } else if str_len + 1 > arr_type.array_size() {
                self.report_error("字符串字面量长度超过数组大小", loc, ErrorCode::E3008_StringTooLong);
            }
        } else {
            let init_type = self.resolve_expr_type(init);
            self.report_error(
                &format!("数组初始化必须使用初始化列表或字符串字面量，不能是 '{}'", init_type),
                loc,
                ErrorCode::E3009_InvalidArrayInit,
            );
        }
    }

    // =========================================================================
    // Lambda capture rewriting: replace captured identifiers with this->field
    // =========================================================================

    fn rewrite_lambda_captures(stmt: &mut Stmt, captures: &[(String, Type, bool)], lambda_name: &str) {
        match stmt {
            Stmt::Block { stmts, .. } => {
                for s in stmts {
                    Self::rewrite_lambda_captures(s, captures, lambda_name);
                }
            }
            Stmt::VarDecl { init, .. } => {
                if let Some(e) = init {
                    Self::rewrite_lambda_captures_in_expr(e, captures, lambda_name);
                }
            }
            Stmt::Expr { expr, .. } => {
                Self::rewrite_lambda_captures_in_expr(expr, captures, lambda_name);
            }
            Stmt::If { cond, then_stmt, else_stmt, .. } => {
                Self::rewrite_lambda_captures_in_expr(cond, captures, lambda_name);
                Self::rewrite_lambda_captures(then_stmt, captures, lambda_name);
                if let Some(s) = else_stmt {
                    Self::rewrite_lambda_captures(s, captures, lambda_name);
                }
            }
            Stmt::While { cond, body, .. } => {
                Self::rewrite_lambda_captures_in_expr(cond, captures, lambda_name);
                Self::rewrite_lambda_captures(body, captures, lambda_name);
            }
            Stmt::DoWhile { body, cond, .. } => {
                Self::rewrite_lambda_captures(body, captures, lambda_name);
                Self::rewrite_lambda_captures_in_expr(cond, captures, lambda_name);
            }
            Stmt::For { init, cond, step, body, .. } => {
                if let Some(i) = init {
                    Self::rewrite_lambda_captures(i, captures, lambda_name);
                }
                if let Some(c) = cond {
                    Self::rewrite_lambda_captures_in_expr(c, captures, lambda_name);
                }
                for s in step.iter_mut() {
                    Self::rewrite_lambda_captures_in_expr(s, captures, lambda_name);
                }
                Self::rewrite_lambda_captures(body, captures, lambda_name);
            }
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    Self::rewrite_lambda_captures_in_expr(v, captures, lambda_name);
                }
            }
            Stmt::Switch { cond, body, .. } => {
                Self::rewrite_lambda_captures_in_expr(cond, captures, lambda_name);
                Self::rewrite_lambda_captures(body, captures, lambda_name);
            }
            Stmt::RangeFor { iter, body, .. } => {
                Self::rewrite_lambda_captures_in_expr(iter, captures, lambda_name);
                Self::rewrite_lambda_captures(body, captures, lambda_name);
            }
            Stmt::Try { body, .. } => {
                Self::rewrite_lambda_captures(body, captures, lambda_name);
            }
            _ => {}
        }
    }

    fn rewrite_lambda_captures_in_expr(expr: &mut Expr, captures: &[(String, Type, bool)], lambda_name: &str) {
        match expr {
            Expr::Identifier { name, loc, ty: _ } => {
                for (cap_name, cap_ty, _) in captures.iter() {
                    if cap_name == name {
                        let this_ty = Type::Pointer {
                            pointee: Box::new(Type::Class {
                                name: lambda_name.to_string(),
                                is_const: false,
                            }),
                            is_const: false,
                        };
                        *expr = Expr::Member {
                            object: Box::new(Expr::This {
                                loc: *loc,
                                ty: this_ty.clone(),
                            }),
                            member: name.clone(),
                            loc: *loc,
                            ty: cap_ty.clone(),
                        };
                        break;
                    }
                }
            }
            Expr::Binary { left, right, .. } => {
                Self::rewrite_lambda_captures_in_expr(left, captures, lambda_name);
                Self::rewrite_lambda_captures_in_expr(right, captures, lambda_name);
            }
            Expr::Unary { operand, .. } => {
                Self::rewrite_lambda_captures_in_expr(operand, captures, lambda_name);
            }
            Expr::Call { name: _, args, .. } => {
                // name is a String, not an Expr to rewrite
                for a in args.iter_mut() {
                    Self::rewrite_lambda_captures_in_expr(a, captures, lambda_name);
                }
            }
            Expr::MemberCall { object, args, .. } => {
                Self::rewrite_lambda_captures_in_expr(object, captures, lambda_name);
                for a in args.iter_mut() {
                    Self::rewrite_lambda_captures_in_expr(a, captures, lambda_name);
                }
            }
            Expr::Index { array, index, .. } => {
                Self::rewrite_lambda_captures_in_expr(array, captures, lambda_name);
                Self::rewrite_lambda_captures_in_expr(index, captures, lambda_name);
            }
            Expr::Member { object, .. } => {
                Self::rewrite_lambda_captures_in_expr(object, captures, lambda_name);
            }
            Expr::Assign { left, right, .. } => {
                Self::rewrite_lambda_captures_in_expr(left, captures, lambda_name);
                Self::rewrite_lambda_captures_in_expr(right, captures, lambda_name);
            }
            Expr::Ternary { cond, then_branch, else_branch, .. } => {
                Self::rewrite_lambda_captures_in_expr(cond, captures, lambda_name);
                Self::rewrite_lambda_captures_in_expr(then_branch, captures, lambda_name);
                Self::rewrite_lambda_captures_in_expr(else_branch, captures, lambda_name);
            }
            Expr::Cast { expr, .. } => {
                Self::rewrite_lambda_captures_in_expr(expr, captures, lambda_name);
            }
            Expr::Sizeof { operand, .. } => {
                if let Some(e) = operand {
                    Self::rewrite_lambda_captures_in_expr(e, captures, lambda_name);
                }
            }
            Expr::InitList { elements, .. } => {
                for e in elements.iter_mut() {
                    Self::rewrite_lambda_captures_in_expr(&mut e.value, captures, lambda_name);
                }
            }
            Expr::Offsetof { .. } => {}
            Expr::Lambda { body, .. } => {
                Self::rewrite_lambda_captures(body, captures, lambda_name);
            }
            _ => {}
        }
    }
}

mod builtin;
mod cpp_auto;
mod cpp_class_layout;
mod cpp_container;
mod cpp_monomorph;
mod cpp_overload;
mod decl;
mod expr;
