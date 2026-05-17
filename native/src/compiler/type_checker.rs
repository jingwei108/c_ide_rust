use crate::compiler::ast::*;
use crate::diagnostics::error_codes::ErrorCode;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub line: i32,
    pub column: i32,
    pub code: i32,
}

#[derive(Debug, Clone)]
struct VarSymbol {
    ty: Type,
    #[allow(dead_code)]
    is_global: bool,
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

#[derive(Default)]
pub struct TypeChecker {
    errors: Vec<TypeError>,
    warnings: Vec<TypeError>,
    hints: Vec<TypeError>,
    funcs: HashMap<String, FuncSymbol>,
    structs: HashMap<String, StructSymbol>,
    unions: HashMap<String, StructSymbol>,
    scopes: Vec<HashMap<String, VarSymbol>>,
    current_func_return: Type,
    loop_depth: i32,
    switch_depth: i32,
}

fn insert_implicit_cast(expr: &mut Expr, target: &Type) {
    let current_ty = expr.ty().clone();
    if target.kind == TypeKind::Double && current_ty.kind != TypeKind::Double && matches!(current_ty.kind, TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::LongLong) {
        if matches!(expr, Expr::FloatLiteral { .. }) {
            expr.set_ty(Type::double());
        } else {
            let loc = *expr.loc();
            let old = std::mem::replace(expr, Expr::Literal { value: 0, loc, ty: Type::int() });
            *expr = Expr::Cast {
                expr: Box::new(old),
                target_type: Type::double(),
                loc,
                ty: Type::double(),
            };
        }
    } else if target.kind != TypeKind::Double && current_ty.kind == TypeKind::Double && matches!(target.kind, TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::LongLong) {
        let loc = *expr.loc();
        let target_ty = match target.kind {
            TypeKind::Char => Type::char(),
            TypeKind::Float => Type::float(),
            TypeKind::LongLong => Type::long_long(),
            _ => Type::int(),
        };
        let old = std::mem::replace(expr, Expr::Literal { value: 0, loc, ty: Type::int() });
        *expr = Expr::Cast {
            expr: Box::new(old),
            target_type: target_ty.clone(),
            loc,
            ty: target_ty,
        };
    } else if target.kind == TypeKind::Float && current_ty.kind != TypeKind::Float && matches!(current_ty.kind, TypeKind::Int | TypeKind::Char | TypeKind::LongLong) {
        let loc = *expr.loc();
        let old = std::mem::replace(expr, Expr::Literal { value: 0, loc, ty: Type::int() });
        *expr = Expr::Cast {
            expr: Box::new(old),
            target_type: Type::float(),
            loc,
            ty: Type::float(),
        };
    } else if target.kind != TypeKind::Float && current_ty.kind == TypeKind::Float && matches!(target.kind, TypeKind::Int | TypeKind::Char | TypeKind::LongLong) {
        let loc = *expr.loc();
        let target_ty = match target.kind {
            TypeKind::Char => Type::char(),
            TypeKind::LongLong => Type::long_long(),
            _ => Type::int(),
        };
        let old = std::mem::replace(expr, Expr::Literal { value: 0, loc, ty: Type::int() });
        *expr = Expr::Cast {
            expr: Box::new(old),
            target_type: target_ty.clone(),
            loc,
            ty: target_ty,
        };
    } else if target.kind == TypeKind::LongLong && current_ty.kind != TypeKind::LongLong && matches!(current_ty.kind, TypeKind::Int | TypeKind::Char) {
        let loc = *expr.loc();
        let old = std::mem::replace(expr, Expr::Literal { value: 0, loc, ty: Type::int() });
        *expr = Expr::Cast {
            expr: Box::new(old),
            target_type: Type::long_long(),
            loc,
            ty: Type::long_long(),
        };
    } else if target.kind != TypeKind::LongLong && current_ty.kind == TypeKind::LongLong && matches!(target.kind, TypeKind::Int | TypeKind::Char) {
        let loc = *expr.loc();
        let target_ty = if target.kind == TypeKind::Char { Type::char() } else { Type::int() };
        let old = std::mem::replace(expr, Expr::Literal { value: 0, loc, ty: Type::int() });
        *expr = Expr::Cast {
            expr: Box::new(old),
            target_type: target_ty.clone(),
            loc,
            ty: target_ty,
        };
    }
}

impl TypeChecker {

    pub fn check(mut self, program: &mut ProgramNode) -> (Vec<TypeError>, Vec<TypeError>, Vec<TypeError>) {
        // Pass 1: Register structs and unions
        for s in &program.structs {
            if self.structs.contains_key(&s.name) {
                self.report_error(&format!("结构体 '{}' 重复定义", s.name), &s.loc, ErrorCode::E3002_StructRedeclared);
                continue;
            }
            let sym = StructSymbol { fields: s.fields.iter().map(|f| (f.ty.clone(), f.name.clone())).collect() };
            self.structs.insert(s.name.clone(), sym);
        }
        for u in &program.unions {
            if self.unions.contains_key(&u.name) {
                self.report_error(&format!("联合体 '{}' 重复定义", u.name), &u.loc, ErrorCode::E3002_StructRedeclared);
                continue;
            }
            let sym = StructSymbol { fields: u.fields.iter().map(|f| (f.ty.clone(), f.name.clone())).collect() };
            self.unions.insert(u.name.clone(), sym);
        }

        // Pass 2: Register function signatures
        for f in &program.funcs {
            let new_sym = FuncSymbol {
                return_type: f.return_type.clone(),
                param_types: f.params.iter().map(|p| p.ty.clone()).collect(),
            };
            if let Some(existing) = self.funcs.get(&f.name) {
                if existing.return_type != new_sym.return_type || existing.param_types != new_sym.param_types {
                    self.report_error(&format!("函数 '{}' 的声明与之前定义签名不一致", f.name), &f.loc, ErrorCode::E3003_FuncRedeclared);
                }
                continue;
            }
            self.funcs.insert(f.name.clone(), new_sym);
        }

        // Pass 2.5: Register globals and check initializers
        self.enter_scope();
        for g in &mut program.globals {
            self.declare_var(&g.name, &g.ty, true);
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
                        self.report_error(&format!("类型不匹配：无法将 '{}' 赋值给 '{}'", init_type, g.ty), &g.loc, ErrorCode::E3004_TypeMismatch);
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

        self.exit_scope();
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

    fn declare_var(&mut self, name: &str, ty: &Type, is_global: bool) {
        if self.scopes.is_empty() {
            self.scopes.push(HashMap::new());
        }
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(name) {
            self.report_error(&format!("变量 '{}' 已在此作用域中声明", name), &SourceLoc { line: 0, column: 0 }, ErrorCode::E3001_VarRedeclared);
            return;
        }
        scope.insert(name.to_string(), VarSymbol { ty: ty.clone(), is_global });
    }

    fn lookup_var(&self, name: &str) -> Option<VarSymbol> {
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

    fn report_error(&mut self, msg: &str, loc: &SourceLoc, code: ErrorCode) {
        self.errors.push(TypeError { message: msg.to_string(), line: loc.line, column: loc.column, code: code as i32 });
    }

    fn report_warning(&mut self, msg: &str, loc: &SourceLoc, code: ErrorCode) {
        self.warnings.push(TypeError { message: msg.to_string(), line: loc.line, column: loc.column, code: code as i32 });
    }

    fn report_hint(&mut self, msg: &str, loc: &SourceLoc, code: ErrorCode) {
        self.hints.push(TypeError { message: msg.to_string(), line: loc.line, column: loc.column, code: code as i32 });
    }

    fn is_int(&self, t: &Type) -> bool {
        matches!(t.kind, TypeKind::Int | TypeKind::Char)
    }
    fn is_scalar(&self, t: &Type) -> bool {
        matches!(t.kind, TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double | TypeKind::LongLong)
    }

    fn is_comparable(&self, a: &Type, b: &Type) -> bool {
        if matches!(a.kind, TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double) && matches!(b.kind, TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double) { return true; }
        if matches!(a.kind, TypeKind::Pointer) && matches!(b.kind, TypeKind::Pointer) { return true; }
        if matches!(a.kind, TypeKind::Pointer) && matches!(b.kind, TypeKind::Array) { return true; }
        if matches!(a.kind, TypeKind::Array) && matches!(b.kind, TypeKind::Pointer) { return true; }
        if matches!(a.kind, TypeKind::Pointer) && matches!(b.kind, TypeKind::Int) { return true; }
        if matches!(a.kind, TypeKind::Int) && matches!(b.kind, TypeKind::Pointer) { return true; }
        false
    }

    fn check_assignable(&mut self, target: &Type, value: &Type, loc: &SourceLoc) -> bool {
        if target == value { return true; }
        if matches!(target.kind, TypeKind::Pointer) && matches!(value.kind, TypeKind::Array)
            && target.base_kind == value.base_kind && target.name == value.name {
            self.report_warning("数组隐式转换为指针。数组名在表达式中会自动退化为指向首元素的指针。", loc, ErrorCode::W3052_ArrayToPointerDecay);
            return true;
        }
        if matches!(target.kind, TypeKind::Array) && matches!(value.kind, TypeKind::Pointer)
            && target.base_kind == value.base_kind {
            return true;
        }
        if matches!(target.kind, TypeKind::Array) && matches!(value.kind, TypeKind::Array)
            && target.base_kind == value.base_kind && target.name == value.name {
            let check_count = target.dims.len().min(value.dims.len());
            let mut dims_compatible = true;
            for i in 0..check_count {
                if target.dims[i] > 0 && target.dims[i] != value.dims[i] {
                    dims_compatible = false;
                    break;
                }
            }
            if dims_compatible { return true; }
        }
        if (matches!(target.kind, TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double)) && (matches!(value.kind, TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double)) {
            // 警告可能丢失精度的情况
            if matches!(target.kind, TypeKind::Char) && matches!(value.kind, TypeKind::Int | TypeKind::Float | TypeKind::Double) {
                self.report_warning("被隐式转换为 char，可能会丢失精度。", loc, ErrorCode::W3053_ImplicitScalarConversion);
            }
            if matches!(target.kind, TypeKind::Int) && matches!(value.kind, TypeKind::Float | TypeKind::Double) {
                self.report_warning(&format!("{} 被隐式转换为 int，可能会丢失精度。", value), loc, ErrorCode::W3053_ImplicitScalarConversion);
            }
            if matches!(target.kind, TypeKind::Float) && matches!(value.kind, TypeKind::Double) {
                self.report_warning("double 被隐式转换为 float，可能会丢失精度。", loc, ErrorCode::W3053_ImplicitScalarConversion);
            }
            // 提示安全的隐式提升
            if matches!(target.kind, TypeKind::Int) && matches!(value.kind, TypeKind::Char) {
                self.report_hint("char 被隐式提升为 int。", loc, ErrorCode::H3057_ImplicitConversionHint);
            }
            if matches!(target.kind, TypeKind::Float) && matches!(value.kind, TypeKind::Int | TypeKind::Char) {
                let src = if matches!(value.kind, TypeKind::Char) { "char" } else { "int" };
                self.report_hint(&format!("{} 被隐式提升为 float。", src), loc, ErrorCode::H3057_ImplicitConversionHint);
            }
            if matches!(target.kind, TypeKind::Double) && matches!(value.kind, TypeKind::Int | TypeKind::Char | TypeKind::Float) {
                let src = match value.kind {
                    TypeKind::Char => "char",
                    TypeKind::Float => "float",
                    _ => "int",
                };
                self.report_hint(&format!("{} 被隐式提升为 double。", src), loc, ErrorCode::H3057_ImplicitConversionHint);
            }
            return true;
        }
        if matches!(target.kind, TypeKind::Pointer) && matches!(value.kind, TypeKind::Int) {
            self.report_warning("整数被隐式转换为指针。建议确保这是有意义的地址值（如 NULL = 0）。", loc, ErrorCode::W3054_IntToPointerCast);
            return true;
        }
        if matches!(target.kind, TypeKind::Pointer) && matches!(value.kind, TypeKind::Pointer) && value.name.is_empty() {
            if value.base_kind == TypeKind::Void {
                self.report_hint("void* 被隐式转换为具体指针类型。", loc, ErrorCode::H3057_ImplicitConversionHint);
            }
            return true;
        }
        false
    }

    fn get_struct_field_type(&self, struct_name: &str, field_name: &str) -> Option<Type> {
        let sym = self.structs.get(struct_name)?;
        for (fty, fname) in &sym.fields {
            if fname == field_name { return Some(fty.clone()); }
        }
        None
    }
    fn get_union_field_type(&self, union_name: &str, field_name: &str) -> Option<Type> {
        let sym = self.unions.get(union_name)?;
        for (fty, fname) in &sym.fields {
            if fname == field_name { return Some(fty.clone()); }
        }
        None
    }

    fn expr_involves_array_or_pointer(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Index { .. } => true,
            Expr::Identifier { name, .. } => {
                self.lookup_var(name).map(|s| s.ty.is_array() || s.ty.is_pointer()).unwrap_or(false)
            }
            Expr::Binary { left, right, .. } => {
                self.expr_involves_array_or_pointer(left) || self.expr_involves_array_or_pointer(right)
            }
            Expr::Unary { operand, .. } => self.expr_involves_array_or_pointer(operand),
            Expr::Assign { left, right, .. } => {
                self.expr_involves_array_or_pointer(left) || self.expr_involves_array_or_pointer(right)
            }
            Expr::Ternary { cond, then_branch, else_branch, .. } => {
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
                self.report_error(&format!("类型不匹配：无法将 '{}' 赋值给 '{}'", init_type, struct_type), loc, ErrorCode::E3004_TypeMismatch);
            }
            return;
        }
        let elements = match init {
            Expr::InitList { elements, .. } => elements.as_mut_slice(),
            _ => return,
        };
        let fields = match self.structs.get(&struct_type.name) {
            Some(s) => s.fields.clone(),
            None => {
                self.report_error(&format!("未知的结构体类型 '{}'", struct_type.name), loc, ErrorCode::E3004_TypeMismatch);
                return;
            }
        };
        if elements.len() > fields.len() {
            self.report_error("初始化列表元素数量超过结构体字段数", loc, ErrorCode::E3005_ArrayInitTooMany);
        }
        for (i, elem) in elements.iter_mut().enumerate() {
            if i >= fields.len() { break; }
            let e_type = self.resolve_expr_type(elem);
            if !self.check_assignable(&fields[i].0, &e_type, loc) {
                self.report_error(&format!("结构体初始化类型不匹配：字段 '{}' 期望 '{}'，实际 '{}'", fields[i].1, fields[i].0, e_type), loc, ErrorCode::E3006_ArrayInitTypeMismatch);
            }
        }
    }

    fn validate_nested_init_list(&mut self, dims: &[i32], init: &mut Expr, loc: &SourceLoc, base_kind: &TypeKind, struct_name: &str) -> bool {
        if dims.is_empty() {
            let expected = match base_kind {
                TypeKind::Struct => Type::struct_type(struct_name),
                TypeKind::Char => Type::char(),
                _ => Type::int(),
            };
            let e_type = self.resolve_expr_type(init);
            if !self.check_assignable(&expected, &e_type, loc) {
                self.report_error(&format!("数组初始化元素类型不匹配：期望 '{}'，实际 '{}'", expected, e_type), loc, ErrorCode::E3006_ArrayInitTypeMismatch);
                return false;
            }
            insert_implicit_cast(init, &expected);
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
            if !self.validate_nested_init_list(&dims[1..], elem, loc, base_kind, struct_name) {
                return false;
            }
        }
        true
    }

    fn check_array_initializer(&mut self, arr_type: &mut Type, init: &mut Expr, loc: &SourceLoc) {
        let elem_type = if arr_type.base_kind == TypeKind::Struct {
            Type::struct_type(&arr_type.name)
        } else {
            Type { kind: arr_type.base_kind, ..Type::default() }
        };

        if !arr_type.dims.is_empty() && arr_type.dims.len() > 1 {
            if let Expr::InitList { elements, .. } = init {
                if arr_type.dims[0] <= 0 {
                    arr_type.dims[0] = elements.len() as i32;
                    arr_type.array_size = arr_type.total_elements();
                }
                self.validate_nested_init_list(&arr_type.dims, init, loc, &arr_type.base_kind, &arr_type.name);
            } else {
                let init_type = self.resolve_expr_type(init);
                self.report_error(&format!("多维数组初始化必须使用嵌套初始化列表，不能是 '{}'", init_type), loc, ErrorCode::E3009_InvalidArrayInit);
            }
            return;
        }

        if let Expr::InitList { elements, .. } = init {
            let mut expected_size = arr_type.array_size;
            if expected_size <= 0 {
                expected_size = elements.len() as i32;
                arr_type.array_size = expected_size;
            }
            if elements.len() > expected_size as usize {
                self.report_error("初始化列表元素数量超过数组大小", loc, ErrorCode::E3005_ArrayInitTooMany);
            }
            for elem in elements.iter_mut() {
                let e_type = self.resolve_expr_type(elem);
                if !self.check_assignable(&elem_type, &e_type, loc) {
                    self.report_error(&format!("数组初始化元素类型不匹配：期望 '{}'，实际 '{}'", elem_type, e_type), loc, ErrorCode::E3006_ArrayInitTypeMismatch);
                } else {
                    insert_implicit_cast(elem, &elem_type);
                }
            }
        } else if let Expr::StringLiteral { value, .. } = init {
            if elem_type.kind != TypeKind::Char {
                self.report_error("字符串字面量只能用于初始化 char 数组", loc, ErrorCode::E3007_StringInitNonCharArray);
                return;
            }
            let str_len = value.len() as i32;
            if arr_type.array_size <= 0 {
                arr_type.array_size = str_len + 1;
            } else if str_len + 1 > arr_type.array_size {
                self.report_error("字符串字面量长度超过数组大小", loc, ErrorCode::E3008_StringTooLong);
            }
        } else {
            let init_type = self.resolve_expr_type(init);
            self.report_error(&format!("数组初始化必须使用初始化列表或字符串字面量，不能是 '{}'", init_type), loc, ErrorCode::E3009_InvalidArrayInit);
        }
    }

    // =========================================================================
    // Function / Statement visitors
    // =========================================================================

    fn visit_func_decl(&mut self, node: &mut FuncDecl) {
        self.current_func_return = node.return_type.clone();
        self.enter_scope();
        for p in &node.params {
            self.declare_var(&p.name, &p.ty, false);
        }
        if let Some(ref mut body) = node.body {
            self.dispatch_stmt(body);
        }
        self.exit_scope();
    }

    fn dispatch_stmt(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::Block { stmts, .. } => {
                self.enter_scope();
                for s in stmts { self.dispatch_stmt(s); }
                self.exit_scope();
            }
            Stmt::VarDecl { var_type, name, init, extra_vars, loc } => {
                if var_type.is_unsigned {
                    self.report_warning("unsigned 类型被映射为 int，暂不支持无符号语义", loc, ErrorCode::W3056_UnsignedToInt);
                }
                if let Some(ref mut init_expr) = init {
                    if var_type.is_array() {
                        self.check_array_initializer(var_type, init_expr, loc);
                    } else if var_type.is_struct() && matches!(init_expr, Expr::InitList { .. }) {
                        self.check_struct_initializer(var_type, init_expr, loc);
                    } else {
                        let init_type = self.resolve_expr_type(init_expr);
                        if !self.check_assignable(var_type, &init_type, loc) {
                            self.report_error(&format!("类型不匹配：无法将 '{}' 赋值给 '{}'", init_type, var_type), loc, ErrorCode::E3004_TypeMismatch);
                        } else {
                            insert_implicit_cast(init_expr, var_type);
                        }
                    }
                }
                self.declare_var(name, var_type, false);
                for (ety, ename, einit) in extra_vars.iter_mut() {
                    if let Some(ref mut init_expr) = einit {
                        if ety.is_array() {
                            self.check_array_initializer(ety, init_expr, loc);
                        } else if ety.is_struct() && matches!(init_expr, Expr::InitList { .. }) {
                            self.check_struct_initializer(ety, init_expr, loc);
                        } else {
                            let init_type = self.resolve_expr_type(init_expr);
                            if !self.check_assignable(ety, &init_type, loc) {
                                self.report_error(&format!("类型不匹配：无法将 '{}' 赋值给 '{}'", init_type, ety), loc, ErrorCode::E3004_TypeMismatch);
                            } else {
                                insert_implicit_cast(init_expr, ety);
                            }
                        }
                    }
                    self.declare_var(ename, ety, false);
                }
            }
            Stmt::Expr { expr, .. } => { self.resolve_expr_type(expr); }
            Stmt::If { cond, then_stmt, else_stmt, loc } => {
                self.check_condition(cond, "if 条件", loc);
                self.dispatch_stmt(then_stmt);
                if let Some(ref mut e) = else_stmt { self.dispatch_stmt(e); }
            }
            Stmt::While { cond, body, loc } => {
                self.check_condition(cond, "while 条件", loc);
                self.loop_depth += 1;
                self.dispatch_stmt(body);
                self.loop_depth -= 1;
            }
            Stmt::DoWhile { body, cond, loc } => {
                self.loop_depth += 1;
                self.dispatch_stmt(body);
                self.loop_depth -= 1;
                self.check_condition(cond, "do...while 条件", loc);
            }
            Stmt::For { init, cond, step, body, loc } => {
                self.enter_scope();
                if let Some(ref mut i) = init { self.dispatch_stmt(i); }
                if let Some(ref mut c) = cond {
                    self.check_condition(c, "for 条件", loc);
                    if let Expr::Binary { op: BinaryOp::Le, left, right, .. } = c {
                        if self.expr_involves_array_or_pointer(left) || self.expr_involves_array_or_pointer(right) {
                            self.report_warning("循环条件中使用了 '<='，如果用于数组索引，可能导致越界（off-by-one 错误）。你是否想使用 '<'？", loc, ErrorCode::W3051_ArrayBoundOffByOne);
                        }
                    }
                }
                if let Some(ref mut s) = step { self.resolve_expr_type(s); }
                self.loop_depth += 1;
                self.dispatch_stmt(body);
                self.loop_depth -= 1;
                self.exit_scope();
            }
            Stmt::Return { value, loc } => {
                if self.current_func_return.is_void() {
                    if value.is_some() {
                        self.report_error("void 函数不能有返回值", loc, ErrorCode::E3012_VoidFuncReturnValue);
                    }
                } else {
                    if let Some(ref mut v) = value {
                        let val_type = self.resolve_expr_type(v);
                        let expected = self.current_func_return.clone();
                        if !self.check_assignable(&expected, &val_type, loc) {
                            self.report_error(&format!("返回类型不匹配：期望 '{}'，实际 '{}'", self.current_func_return, val_type), loc, ErrorCode::E3014_ReturnTypeMismatch);
                        }
                    } else {
                        self.report_error("非 void 函数必须返回一个值", loc, ErrorCode::E3013_MissingReturnValue);
                    }
                }
            }
            Stmt::Break { loc } => {
                if self.loop_depth <= 0 && self.switch_depth <= 0 {
                    self.report_error("break 只能在循环或 switch 体内使用", loc, ErrorCode::E3010_BreakOutsideLoop);
                }
            }
            Stmt::Continue { loc } => {
                if self.loop_depth <= 0 {
                    self.report_error("continue 只能在循环体内使用", loc, ErrorCode::E3011_ContinueOutsideLoop);
                }
            }
            Stmt::Switch { cond, body, loc } => {
                self.switch_depth += 1;
                let cond_type = self.resolve_expr_type(cond);
                if !self.is_int(&cond_type) {
                    self.report_error("switch 条件必须是整数类型", loc, ErrorCode::E3046_SwitchCondType);
                }
                self.dispatch_stmt(body);
                self.switch_depth -= 1;
            }
            Stmt::Case { label, stmt, loc } => {
                if let Some(ref mut l) = label {
                    let label_type = self.resolve_expr_type(l);
                    if !self.is_int(&label_type) {
                        self.report_error("case 标签必须是整数常量", loc, ErrorCode::E3047_CaseNotConstant);
                    }
                }
                self.dispatch_stmt(stmt);
            }
        }
    }

    fn check_condition(&mut self, cond: &mut Expr, ctx: &str, loc: &SourceLoc) {
        let ty = self.resolve_expr_type(cond);
        if !self.is_scalar(&ty) && !matches!(ty.kind, TypeKind::Pointer | TypeKind::Array) {
            self.report_error(&format!("{} 必须是整数、浮点数或指针类型", ctx), loc, ErrorCode::E3015_InvalidCondition);
        }
        let is_assign_expr = |e: &Expr| matches!(e, Expr::Assign { op: AssignOp::Assign, .. });
        if is_assign_expr(cond) {
            self.report_warning("条件中使用了赋值运算符 '='，你是否想使用比较运算符 '=='？", loc, ErrorCode::W3050_AssignInCondition);
        } else if let Expr::Binary { left, right, .. } = cond {
            if is_assign_expr(left) || is_assign_expr(right) {
                self.report_warning("条件中包含了赋值表达式，你是否想使用比较运算符 '=='？", loc, ErrorCode::W3050_AssignInCondition);
            }
        }
    }

    // =========================================================================
    // Expression type resolution
    // =========================================================================

    pub fn resolve_expr_type(&mut self, expr: &mut Expr) -> Type {
        match expr {
            Expr::Binary { op, left, right, loc, ty } => {
                let left_type = self.resolve_expr_type(left);
                let right_type = self.resolve_expr_type(right);
                *ty = match op {
                    BinaryOp::Add | BinaryOp::Sub => {
                        if self.is_scalar(&left_type) && self.is_scalar(&right_type) {
                            if left_type.kind == TypeKind::Double || right_type.kind == TypeKind::Double { Type::double() }
                            else if left_type.kind == TypeKind::Float || right_type.kind == TypeKind::Float { Type::float() }
                            else if left_type.kind == TypeKind::LongLong || right_type.kind == TypeKind::LongLong { Type::long_long() }
                            else { Type::int() }
                        } else if left_type.is_pointer() && self.is_int(&right_type) {
                            left_type.clone()
                        } else if self.is_int(&left_type) && right_type.is_pointer() && matches!(op, BinaryOp::Add) {
                            right_type.clone()
                        } else if left_type.is_pointer() && right_type.is_pointer() && matches!(op, BinaryOp::Sub) {
                            Type::int()
                        } else {
                            self.report_error("算术运算要求两边都是 int 类型，或指针与整数", loc, ErrorCode::E3016_ArithmeticTypeError);
                            Type::int()
                        }
                    }
                    BinaryOp::Mul | BinaryOp::Div => {
                        if !self.is_scalar(&left_type) || !self.is_scalar(&right_type) {
                            self.report_error("乘除运算要求两边都是 int 或 float 类型", loc, ErrorCode::E3016_ArithmeticTypeError);
                        }
                        if left_type.kind == TypeKind::Double || right_type.kind == TypeKind::Double { Type::double() }
                        else if left_type.kind == TypeKind::Float || right_type.kind == TypeKind::Float { Type::float() }
                        else if left_type.kind == TypeKind::LongLong || right_type.kind == TypeKind::LongLong { Type::long_long() }
                        else { Type::int() }
                    }
                    BinaryOp::Mod => {
                        if !self.is_int(&left_type) && !matches!(left_type.kind, TypeKind::LongLong) || !self.is_int(&right_type) && !matches!(right_type.kind, TypeKind::LongLong) {
                            self.report_error("取模运算要求两边都是 int 类型", loc, ErrorCode::E3016_ArithmeticTypeError);
                        }
                        Type::int()
                    }
                    BinaryOp::Eq | BinaryOp::Ne => {
                        if !self.is_comparable(&left_type, &right_type) {
                            self.report_error("类型不兼容，无法比较", loc, ErrorCode::E3017_ComparisonTypeError);
                        }
                        Type::int()
                    }
                    BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                        let left_is_ptrlike = matches!(left_type.kind, TypeKind::Pointer | TypeKind::Array);
                        let right_is_ptrlike = matches!(right_type.kind, TypeKind::Pointer | TypeKind::Array);
                        if !(self.is_scalar(&left_type) && self.is_scalar(&right_type) || left_is_ptrlike && right_is_ptrlike) {
                            self.report_error("关系运算要求两边都是 int/float 类型或同类型指针", loc, ErrorCode::E3018_RelationTypeError);
                        }
                        Type::int()
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if !self.is_scalar(&left_type) || !self.is_scalar(&right_type) {
                            self.report_error("逻辑运算要求两边都是 int 或 float 类型", loc, ErrorCode::E3019_LogicTypeError);
                        }
                        Type::int()
                    }
                    BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor | BinaryOp::Shl | BinaryOp::Shr => {
                        if !self.is_int(&left_type) || !self.is_int(&right_type) {
                            self.report_error("位运算要求两边都是 int 类型", loc, ErrorCode::E3048_BitOpTypeError);
                        }
                        Type::int()
                    }
                };
                ty.clone()
            }
            Expr::Ternary { cond, then_branch, else_branch, loc, ty } => {
                let cond_type = self.resolve_expr_type(cond);
                if !self.is_scalar(&cond_type) && !matches!(cond_type.kind, TypeKind::Pointer | TypeKind::Array) {
                    self.report_error("三目运算符条件必须是 int、float 或指针类型", loc, ErrorCode::E3020_UnaryTypeError);
                }
                let then_type = self.resolve_expr_type(then_branch);
                let else_type = self.resolve_expr_type(else_branch);
                if then_type.kind != else_type.kind || then_type.name != else_type.name || then_type.base_kind != else_type.base_kind {
                    self.report_error("三目运算符分支类型不匹配", loc, ErrorCode::E3004_TypeMismatch);
                }
                *ty = then_type;
                ty.clone()
            }
            Expr::Unary { op, operand, loc, ty } => {
                let operand_type = self.resolve_expr_type(operand);
                *ty = match op {
                    UnaryOp::Neg => {
                        if !self.is_scalar(&operand_type) {
                            self.report_error("取负运算要求操作数是 int 或 float 类型", loc, ErrorCode::E3020_UnaryTypeError);
                        }
                        if operand_type.kind == TypeKind::Double { Type::double() }
                        else if operand_type.kind == TypeKind::Float { Type::float() }
                        else { Type::int() }
                    }
                    UnaryOp::Not => {
                        if !self.is_scalar(&operand_type) && !operand_type.is_pointer() {
                            self.report_error("逻辑非要求操作数是标量或指针类型", loc, ErrorCode::E3020_UnaryTypeError);
                        }
                        Type::int()
                    }
                    UnaryOp::BitNot => {
                        if !self.is_int(&operand_type) {
                            self.report_error("按位取反要求操作数是 int 类型", loc, ErrorCode::E3020_UnaryTypeError);
                        }
                        Type::int()
                    }
                    UnaryOp::Addr => {
                        Type { kind: TypeKind::Pointer, name: operand_type.name.clone(), base_kind: operand_type.kind, ..Type::default() }
                    }
                    UnaryOp::Deref => {
                        if !operand_type.is_pointer() && !operand_type.is_array() {
                            self.report_error("解引用要求指针类型", loc, ErrorCode::E3021_DerefNonPointer);
                            Type::int()
                        } else if operand_type.base_kind == TypeKind::Struct {
                            Type::struct_type(&operand_type.name)
                        } else {
                            Type { kind: operand_type.base_kind, ..Type::default() }
                        }
                    }
                    UnaryOp::PreInc | UnaryOp::PreDec | UnaryOp::PostInc | UnaryOp::PostDec => {
                        if !self.is_int(&operand_type) && operand_type.kind != TypeKind::Float && operand_type.kind != TypeKind::Double && !operand_type.is_pointer() {
                            self.report_error("自增/自减要求 int 类型或指针类型", loc, ErrorCode::E3022_IncDecTypeError);
                        }
                        if let Expr::Identifier { name, .. } = operand.as_ref() {
                            if let Some(sym) = self.lookup_var(name) {
                                if sym.ty.is_const {
                                    self.report_error(&format!("不能修改常量变量 '{}'", name), loc, ErrorCode::E3049_AssignToConst);
                                }
                            }
                        }
                        if operand_type.is_pointer() {
                            operand_type.clone()
                        } else {
                            Type::int()
                        }
                    }
                };
                ty.clone()
            }
            Expr::Literal { .. } => Type::int(),
            Expr::FloatLiteral { .. } => Type::float(),
            Expr::LongLiteral { .. } => Type::long_long(),
            Expr::StringLiteral { .. } => Type::pointer(TypeKind::Char, "char"),
            Expr::Identifier { name, loc, ty } => {
                if let Some(sym) = self.lookup_var(name) {
                    *ty = sym.ty;
                } else if self.funcs.contains_key(name) {
                    // Function name used as value (function pointer)
                    *ty = Type::int();
                } else {
                    self.report_error(&format!("未声明的变量 '{}'", name), loc, ErrorCode::E3023_UndeclaredVar);
                    *ty = Type::int();
                }
                ty.clone()
            }
            Expr::Call { name, args, loc, ty } => {
                *ty = self.visit_call(name, args, loc);
                ty.clone()
            }
            Expr::Index { array, index, loc, ty } => {
                let arr_type = self.resolve_expr_type(array);
                let idx_type = self.resolve_expr_type(index);
                if !self.is_int(&idx_type) {
                    self.report_error("数组索引必须是 int 类型", loc, ErrorCode::E3039_ArrayIndexType);
                    *ty = Type::int();
                } else if !arr_type.is_array() && !arr_type.is_pointer() {
                    self.report_error("不能对非数组/指针类型进行索引", loc, ErrorCode::E3040_IndexNonArray);
                    *ty = Type::int();
                } else if (arr_type.is_array() && !arr_type.dims.is_empty()) || arr_type.is_pointer() {
                    *ty = arr_type.subscript_type();
                } else if arr_type.base_kind == TypeKind::Struct {
                    *ty = Type::struct_type(&arr_type.name);
                } else if arr_type.base_kind == TypeKind::Char {
                    *ty = Type::char();
                } else {
                    *ty = Type::int();
                }
                ty.clone()
            }
            Expr::Member { object, member, loc, ty } => {
                let obj_type = self.resolve_expr_type(object);
                let (type_name, is_union) = if obj_type.is_struct() || (obj_type.is_pointer() && !obj_type.name.is_empty() && obj_type.base_kind == TypeKind::Struct) {
                    (obj_type.name.clone(), false)
                } else if obj_type.is_union() || (obj_type.is_pointer() && !obj_type.name.is_empty() && obj_type.base_kind == TypeKind::Union) {
                    (obj_type.name.clone(), true)
                } else {
                    self.report_error("'.' 和 '->' 只能用于结构体或联合体类型", loc, ErrorCode::E3041_MemberNonStruct);
                    *ty = Type::int();
                    return ty.clone();
                };
                let field_type = if is_union {
                    self.get_union_field_type(&type_name, member)
                } else {
                    self.get_struct_field_type(&type_name, member)
                };
                if let Some(ft) = field_type {
                    *ty = ft;
                } else {
                    let kind_str = if is_union { "联合体" } else { "结构体" };
                    self.report_error(&format!("{} '{}' 没有成员 '{}'", kind_str, type_name, member), loc, ErrorCode::E3042_UnknownMember);
                    *ty = Type::int();
                }
                ty.clone()
            }
            Expr::Assign { op, left, right, loc, ty } => {
                let right_type = self.resolve_expr_type(right);
                let left_type = self.resolve_expr_type(left);
                let is_lvalue = matches!(left.as_ref(),
                    Expr::Identifier { .. } | Expr::Index { .. } | Expr::Member { .. } |
                    Expr::Unary { op: UnaryOp::Deref, .. });
                if !is_lvalue {
                    self.report_error("赋值左边必须是可修改的左值", loc, ErrorCode::E3043_AssignToRValue);
                }
                if let Expr::Identifier { name, .. } = left.as_ref() {
                    if let Some(sym) = self.lookup_var(name) {
                        if sym.ty.is_const {
                            self.report_error(&format!("不能给常量变量 '{}' 赋值", name), loc, ErrorCode::E3049_AssignToConst);
                        }
                    }
                }
                if !self.check_assignable(&left_type, &right_type, loc) {
                    self.report_error(&format!("类型不匹配：无法将 '{}' 赋值给 '{}'", right_type, left_type), loc, ErrorCode::E3044_AssignTypeMismatch);
                } else {
                    insert_implicit_cast(right, &left_type);
                }
                if *op != AssignOp::Assign && (!self.is_scalar(&left_type) || !self.is_scalar(&right_type)) {
                    self.report_error("复合赋值要求两边都是 int 或 float 类型", loc, ErrorCode::E3045_CompoundAssignType);
                }
                *ty = left_type.clone();
                ty.clone()
            }
            Expr::Sizeof { operand, ty, .. } => {
                if let Some(ref mut op) = operand {
                    self.resolve_expr_type(op);
                }
                *ty = Type::int();
                ty.clone()
            }
            Expr::Cast { expr, target_type, ty, .. } => {
                self.resolve_expr_type(expr);
                *ty = target_type.clone();
                ty.clone()
            }
            Expr::InitList { elements, ty, .. } => {
                for elem in elements.iter_mut() {
                    self.resolve_expr_type(elem);
                }
                *ty = Type::void();
                ty.clone()
            }
        }
    }

    fn visit_call(&mut self, name: &str, args: &mut [Expr], loc: &SourceLoc) -> Type {
        match name {
            "malloc" => self.check_builtin_malloc(args, loc),
            "free" => self.check_builtin_free(args, loc),
            "print_int" | "__cide_output" | "__cide_step" => {
                self.check_builtin_print_int(args, loc, name)
            }
            "printf" => self.check_builtin_printf(args, loc),
            "scanf" => self.check_builtin_scanf(args, loc),
            "strlen" => self.check_builtin_strlen(args, loc),
            "strcpy" => self.check_builtin_strcpy(args, loc),
            "strcmp" => self.check_builtin_strcmp(args, loc),
            "getchar" => self.check_builtin_getchar(args, loc),
            "putchar" => self.check_builtin_putchar(args, loc),
            "rand" => self.check_builtin_rand(args, loc),
            "srand" => self.check_builtin_srand(args, loc),
            "memset" => self.check_builtin_memset(args, loc),
            "exit" => self.check_builtin_exit(args, loc),
            "strcat" => self.check_builtin_strcat(args, loc),
            "atoi" => self.check_builtin_atoi(args, loc),
            "fprintf" => self.check_builtin_fprintf(args, loc),
            "realloc" => self.check_builtin_realloc(args, loc),
            "qsort" => self.check_builtin_qsort(args, loc),
            _ => self.check_user_func(name, args, loc),
        }
    }

    // ---------- 内建函数检查器 ----------

    fn check_builtin_malloc(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 1 {
            self.report_error("malloc 需要一个参数", loc, ErrorCode::E3024_MallocArgCount);
        } else {
            let expected = Type::int();
            let arg_type = self.resolve_expr_type(&mut args[0]);
            if !self.check_assignable(&expected, &arg_type, loc) {
                self.report_error("malloc 参数必须是 int", loc, ErrorCode::E3025_MallocArgType);
            } else {
                insert_implicit_cast(&mut args[0], &expected);
            }
        }
        Type::pointer(TypeKind::Void, "")
    }

    fn check_builtin_free(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 1 {
            self.report_error("free 需要一个参数", loc, ErrorCode::E3026_FreeArgCount);
        } else {
            let arg_type = self.resolve_expr_type(&mut args[0]);
            if !arg_type.is_pointer() && !arg_type.is_array() {
                self.report_error("free 参数必须是指针", loc, ErrorCode::E3027_FreeArgType);
            }
        }
        Type::void()
    }

    fn check_builtin_print_int(
        &mut self,
        args: &mut [Expr],
        loc: &SourceLoc,
        name: &str,
    ) -> Type {
        if args.len() != 1 {
            self.report_error(
                &format!("{} 需要一个参数", name),
                loc,
                ErrorCode::E3028_BuiltInArgCount,
            );
        } else {
            let expected = Type::int();
            let arg_type = self.resolve_expr_type(&mut args[0]);
            if !self.check_assignable(&expected, &arg_type, loc) {
                self.report_error(
                    &format!("{} 参数必须是 int", name),
                    loc,
                    ErrorCode::E3029_BuiltInArgType,
                );
            } else {
                insert_implicit_cast(&mut args[0], &expected);
            }
        }
        Type::void()
    }

    fn check_builtin_printf(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.is_empty() {
            self.report_error(
                "printf 至少需要 1 个参数（格式字符串）",
                loc,
                ErrorCode::E3030_PrintfArgCount,
            );
        } else {
            let fmt_type = self.resolve_expr_type(&mut args[0]);
            if !fmt_type.is_pointer() && !fmt_type.is_array() {
                self.report_error(
                    "printf 的第一个参数必须是字符串",
                    loc,
                    ErrorCode::E3031_PrintfFirstArg,
                );
            }
            for (i, arg) in args.iter_mut().enumerate().skip(1) {
                let arg_type = self.resolve_expr_type(arg);
                if !self.is_scalar(&arg_type) && !arg_type.is_pointer() && !arg_type.is_array() {
                    self.report_error(
                        &format!("printf 的第 {} 个参数必须是 int、float、char 或指针", i + 1),
                        loc,
                        ErrorCode::E3032_PrintfArgType,
                    );
                }
            }
        }
        Type::void()
    }

    fn check_builtin_scanf(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() < 2 {
            self.report_error(
                "scanf 至少需要 2 个参数（格式字符串和地址）",
                loc,
                ErrorCode::E3033_ScanfArgCount,
            );
        } else {
            let fmt_type = self.resolve_expr_type(&mut args[0]);
            if !fmt_type.is_pointer() && !fmt_type.is_array() {
                self.report_error(
                    "scanf 的第一个参数必须是字符串",
                    loc,
                    ErrorCode::E3034_ScanfFirstArg,
                );
            }
            for (i, arg) in args.iter_mut().enumerate().skip(1) {
                let arg_type = self.resolve_expr_type(arg);
                if !arg_type.is_pointer() && !arg_type.is_array() {
                    self.report_error(
                        &format!("scanf 的第 {} 个参数必须是指针", i + 1),
                        loc,
                        ErrorCode::E3035_ScanfArgType,
                    );
                }
            }
        }
        Type::void()
    }

    fn check_builtin_strlen(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 1 {
            self.report_error("strlen 需要一个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            let arg_type = self.resolve_expr_type(&mut args[0]);
            if !arg_type.is_pointer() && !arg_type.is_array() {
                self.report_error(
                    "strlen 参数必须是指针或数组",
                    loc,
                    ErrorCode::E3029_BuiltInArgType,
                );
            }
        }
        Type::int()
    }

    fn check_builtin_strcpy(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 2 {
            self.report_error("strcpy 需要两个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            for (i, arg) in args.iter_mut().enumerate() {
                let arg_type = self.resolve_expr_type(arg);
                if !arg_type.is_pointer() && !arg_type.is_array() {
                    self.report_error(
                        &format!("strcpy 的第 {} 个参数必须是指针或数组", i + 1),
                        loc,
                        ErrorCode::E3029_BuiltInArgType,
                    );
                }
            }
        }
        Type::void()
    }

    fn check_builtin_strcmp(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 2 {
            self.report_error("strcmp 需要两个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            for (i, arg) in args.iter_mut().enumerate() {
                let arg_type = self.resolve_expr_type(arg);
                if !arg_type.is_pointer() && !arg_type.is_array() {
                    self.report_error(
                        &format!("strcmp 的第 {} 个参数必须是指针或数组", i + 1),
                        loc,
                        ErrorCode::E3029_BuiltInArgType,
                    );
                }
            }
        }
        Type::int()
    }

    fn check_builtin_getchar(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if !args.is_empty() {
            self.report_error("getchar 不需要参数", loc, ErrorCode::E3028_BuiltInArgCount);
        }
        Type::int()
    }

    fn check_builtin_putchar(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 1 {
            self.report_error("putchar 需要一个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            let expected = Type::int();
            let arg_type = self.resolve_expr_type(&mut args[0]);
            if !self.check_assignable(&expected, &arg_type, loc) {
                self.report_error("putchar 参数必须是 int", loc, ErrorCode::E3029_BuiltInArgType);
            } else {
                insert_implicit_cast(&mut args[0], &expected);
            }
        }
        Type::void()
    }

    fn check_builtin_rand(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if !args.is_empty() {
            self.report_error("rand 不需要参数", loc, ErrorCode::E3028_BuiltInArgCount);
        }
        Type::int()
    }

    fn check_builtin_srand(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 1 {
            self.report_error("srand 需要一个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            let expected = Type::int();
            let arg_type = self.resolve_expr_type(&mut args[0]);
            if !self.check_assignable(&expected, &arg_type, loc) {
                self.report_error("srand 参数必须是 int", loc, ErrorCode::E3029_BuiltInArgType);
            } else {
                insert_implicit_cast(&mut args[0], &expected);
            }
        }
        Type::void()
    }

    fn check_builtin_memset(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 3 {
            self.report_error("memset 需要三个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            let ptr_type = self.resolve_expr_type(&mut args[0]);
            if !ptr_type.is_pointer() && !ptr_type.is_array() {
                self.report_error(
                    "memset 第一个参数必须是指针",
                    loc,
                    ErrorCode::E3029_BuiltInArgType,
                );
            }
            for i in 1..3 {
                let expected = Type::int();
                let t = self.resolve_expr_type(&mut args[i]);
                if !self.check_assignable(&expected, &t, loc) {
                    self.report_error(
                        &format!("memset 的第 {} 个参数必须是 int", i + 1),
                        loc,
                        ErrorCode::E3029_BuiltInArgType,
                    );
                } else {
                    insert_implicit_cast(&mut args[i], &expected);
                }
            }
        }
        Type::pointer(TypeKind::Void, "")
    }

    fn check_builtin_exit(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 1 {
            self.report_error("exit 需要一个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            let expected = Type::int();
            let arg_type = self.resolve_expr_type(&mut args[0]);
            if !self.check_assignable(&expected, &arg_type, loc) {
                self.report_error("exit 参数必须是 int", loc, ErrorCode::E3029_BuiltInArgType);
            } else {
                insert_implicit_cast(&mut args[0], &expected);
            }
        }
        Type::void()
    }

    fn check_builtin_strcat(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 2 {
            self.report_error("strcat 需要两个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            for (i, arg) in args.iter_mut().enumerate() {
                let arg_type = self.resolve_expr_type(arg);
                if !arg_type.is_pointer() && !arg_type.is_array() {
                    self.report_error(
                        &format!("strcat 的第 {} 个参数必须是指针或数组", i + 1),
                        loc,
                        ErrorCode::E3029_BuiltInArgType,
                    );
                }
            }
        }
        Type::pointer(TypeKind::Char, "char")
    }

    fn check_builtin_atoi(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 1 {
            self.report_error("atoi 需要一个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            let arg_type = self.resolve_expr_type(&mut args[0]);
            if !arg_type.is_pointer() && !arg_type.is_array() {
                self.report_error(
                    "atoi 参数必须是指针或数组",
                    loc,
                    ErrorCode::E3029_BuiltInArgType,
                );
            }
        }
        Type::int()
    }

    fn check_builtin_fprintf(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() < 2 {
            self.report_error(
                "fprintf 至少需要 2 个参数（文件指针和格式字符串）",
                loc,
                ErrorCode::E3030_PrintfArgCount,
            );
        } else {
            let stream_type = self.resolve_expr_type(&mut args[0]);
            if !stream_type.is_pointer() && !matches!(stream_type.kind, TypeKind::Int) {
                self.report_error(
                    "fprintf 的第一个参数必须是文件指针或整数",
                    loc,
                    ErrorCode::E3029_BuiltInArgType,
                );
            }
            let fmt_type = self.resolve_expr_type(&mut args[1]);
            if !fmt_type.is_pointer() && !fmt_type.is_array() {
                self.report_error(
                    "fprintf 的第二个参数必须是字符串",
                    loc,
                    ErrorCode::E3031_PrintfFirstArg,
                );
            }
            for (i, arg) in args.iter_mut().enumerate().skip(2) {
                let arg_type = self.resolve_expr_type(arg);
                if !self.is_scalar(&arg_type) && !arg_type.is_pointer() && !arg_type.is_array() {
                    self.report_error(
                        &format!("fprintf 的第 {} 个参数必须是 int、float、char 或指针", i + 1),
                        loc,
                        ErrorCode::E3032_PrintfArgType,
                    );
                }
            }
        }
        Type::void()
    }

    fn check_builtin_realloc(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 2 {
            self.report_error("realloc 需要两个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            let ptr_type = self.resolve_expr_type(&mut args[0]);
            if !ptr_type.is_pointer() && !matches!(ptr_type.kind, TypeKind::Int) {
                self.report_error(
                    "realloc 第一个参数必须是指针",
                    loc,
                    ErrorCode::E3029_BuiltInArgType,
                );
            }
            let size_type = self.resolve_expr_type(&mut args[1]);
            if !self.check_assignable(&Type::int(), &size_type, loc) {
                self.report_error(
                    "realloc 第二个参数必须是 int",
                    loc,
                    ErrorCode::E3029_BuiltInArgType,
                );
            } else {
                insert_implicit_cast(&mut args[1], &Type::int());
            }
        }
        Type::pointer(TypeKind::Void, "")
    }

    fn check_builtin_qsort(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 4 {
            self.report_error("qsort 需要四个参数", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            let base_type = self.resolve_expr_type(&mut args[0]);
            if !base_type.is_pointer() && !base_type.is_array() {
                self.report_error(
                    "qsort 第一个参数必须是指针或数组",
                    loc,
                    ErrorCode::E3029_BuiltInArgType,
                );
            }
            for i in 1..3 {
                let t = self.resolve_expr_type(&mut args[i]);
                if !self.check_assignable(&Type::int(), &t, loc) {
                    self.report_error(
                        &format!("qsort 第 {} 个参数必须是 int", i + 1),
                        loc,
                        ErrorCode::E3029_BuiltInArgType,
                    );
                } else {
                    insert_implicit_cast(&mut args[i], &Type::int());
                }
            }
            let compar_type = self.resolve_expr_type(&mut args[3]);
            if !matches!(compar_type.kind, TypeKind::Int) && !compar_type.is_pointer() {
                self.report_error(
                    "qsort 第四个参数必须是函数指针",
                    loc,
                    ErrorCode::E3029_BuiltInArgType,
                );
            }
        }
        Type::void()
    }

    fn check_user_func(&mut self, name: &str, args: &mut [Expr], loc: &SourceLoc) -> Type {
        let sym = self.funcs.get(name).cloned();
        if let Some(sym) = sym {
            if args.len() != sym.param_types.len() {
                self.report_error(
                    &format!(
                        "函数 '{}' 参数数量不匹配：期望 {}，实际 {}",
                        name,
                        sym.param_types.len(),
                        args.len()
                    ),
                    loc,
                    ErrorCode::E3037_FuncArgCount,
                );
            } else {
                for (i, (arg, expected)) in
                    args.iter_mut().zip(sym.param_types.iter()).enumerate()
                {
                    let arg_type = self.resolve_expr_type(arg);
                    if !self.check_assignable(expected, &arg_type, loc) {
                        self.report_error(
                            &format!("函数 '{}' 第 {} 个参数类型不匹配", name, i + 1),
                            loc,
                            ErrorCode::E3038_FuncArgType,
                        );
                    } else {
                        insert_implicit_cast(arg, expected);
                    }
                }
            }
            sym.return_type.clone()
        } else {
            self.report_error(
                &format!("未定义的函数 '{}'", name),
                loc,
                ErrorCode::E3036_UndefinedFunc,
            );
            Type::void()
        }
    }
}
