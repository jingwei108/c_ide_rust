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
    static_func_sigs: HashMap<String, FuncSymbol>,
    static_func_files: HashMap<String, Vec<String>>,
    structs: HashMap<String, StructSymbol>,
    unions: HashMap<String, StructSymbol>,
    scopes: Vec<HashMap<String, VarSymbol>>,
    current_func_return: Type,
    current_file: String,
    loop_depth: i32,
    switch_depth: i32,
    current_func_params: HashSet<String>,
}

/// 根据 (from, to) 类型对判断是否允许隐式转换，并返回转换后的目标类型。
fn implicit_cast_target(from: TypeKind, to: TypeKind) -> Option<Type> {
    use TypeKind::*;
    match (from, to) {
        (Int | Char | Float | LongLong, Double) => Some(Type::double()),
        (Double, Int) => Some(Type::int()),
        (Double, Char) => Some(Type::char()),
        (Double, Float) => Some(Type::float()),
        (Double, LongLong) => Some(Type::long_long()),
        (Int | Char | LongLong, Float) => Some(Type::float()),
        (Float, Int) => Some(Type::int()),
        (Float, Char) => Some(Type::char()),
        (Float, LongLong) => Some(Type::long_long()),
        (Int | Char, LongLong) => Some(Type::long_long()),
        (LongLong, Int) => Some(Type::int()),
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
    if let Some(target_ty) = implicit_cast_target(current_ty.kind(), target.kind()) {
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
            if f.is_static {
                if let Some(existing) = self.static_func_sigs.get(&f.name) {
                    if existing.return_type != new_sym.return_type || existing.param_types != new_sym.param_types {
                        self.report_error(&format!("函数 '{}' 的声明与之前定义签名不一致", f.name), &f.loc, ErrorCode::E3003_FuncRedeclared);
                    }
                } else {
                    self.static_func_sigs.insert(f.name.clone(), new_sym);
                }
                self.static_func_files.entry(f.name.clone()).or_default().push(f.source_file.clone());
            } else {
                if let Some(existing) = self.funcs.get(&f.name) {
                    if existing.return_type != new_sym.return_type || existing.param_types != new_sym.param_types {
                        self.report_error(&format!("函数 '{}' 的声明与之前定义签名不一致", f.name), &f.loc, ErrorCode::E3003_FuncRedeclared);
                    }
                    continue;
                }
                self.funcs.insert(f.name.clone(), new_sym);
            }
        }

        // Pass 2.5: Register globals and check initializers
        self.enter_scope();
        for g in &mut program.globals {
            self.declare_var(&g.name, &g.ty, true, g.is_extern);
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

    fn declare_var(&mut self, name: &str, ty: &Type, is_global: bool, is_extern: bool) {
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
                scope.insert(name.to_string(), VarSymbol { ty: ty.clone(), is_global, is_extern });
                return;
            }
            self.report_error(
                &format!("变量 '{}' 已在此作用域中声明", name),
                &SourceLoc { line: 0, column: 0 },
                ErrorCode::E3001_VarRedeclared,
            );
            return;
        }
        scope.insert(name.to_string(), VarSymbol { ty: ty.clone(), is_global, is_extern });
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
        self.errors.push(TypeError { message: msg.to_string(), line: loc.line, column: loc.column, code: code as i32 });
    }

    pub(crate) fn report_warning(&mut self, msg: &str, loc: &SourceLoc, code: ErrorCode) {
        self.warnings.push(TypeError { message: msg.to_string(), line: loc.line, column: loc.column, code: code as i32 });
    }

    fn report_hint(&mut self, msg: &str, loc: &SourceLoc, code: ErrorCode) {
        self.hints.push(TypeError { message: msg.to_string(), line: loc.line, column: loc.column, code: code as i32 });
    }

    pub(crate) fn is_int(&self, t: &Type) -> bool {
        matches!(t.kind(), TypeKind::Int | TypeKind::Char)
    }
    pub(crate) fn is_scalar(&self, t: &Type) -> bool {
        matches!(t.kind(), TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double | TypeKind::LongLong)
    }

    pub(crate) fn is_comparable(&self, a: &Type, b: &Type) -> bool {
        if matches!(a.kind(), TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double) && matches!(b.kind(), TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double) { return true; }
        if matches!(a.kind(), TypeKind::Pointer) && matches!(b.kind(), TypeKind::Pointer) { return true; }
        if matches!(a.kind(), TypeKind::Pointer) && matches!(b.kind(), TypeKind::Array) { return true; }
        if matches!(a.kind(), TypeKind::Array) && matches!(b.kind(), TypeKind::Pointer) { return true; }
        if matches!(a.kind(), TypeKind::Pointer) && matches!(b.kind(), TypeKind::Int) { return true; }
        if matches!(a.kind(), TypeKind::Int) && matches!(b.kind(), TypeKind::Pointer) { return true; }
        false
    }

    fn check_array_pointer_assignable(&mut self, target: &Type, value: &Type, loc: &SourceLoc) -> bool {
        if matches!(target.kind(), TypeKind::Pointer) && matches!(value.kind(), TypeKind::Array) {
            if let (Type::Pointer { pointee: t_pointee, .. }, Type::Array { element: v_element, .. }) = (target, value) {
                if t_pointee.as_ref() == v_element.as_ref() {
                    self.report_warning("数组隐式转换为指针。数组名在表达式中会自动退化为指向首元素的指针。", loc, ErrorCode::W3052_ArrayToPointerDecay);
                    return true;
                }
                // Multidimensional array decay: int arr[3][3] -> int (*)[3]
                if t_pointee.as_ref() == &value.subscript_type() {
                    self.report_warning("数组隐式转换为指针。数组名在表达式中会自动退化为指向首元素的指针。", loc, ErrorCode::W3052_ArrayToPointerDecay);
                    return true;
                }
            }
        }
        if matches!(target.kind(), TypeKind::Array) && matches!(value.kind(), TypeKind::Pointer) {
            if let (Type::Array { element: t_element, .. }, Type::Pointer { pointee: v_pointee, .. }) = (target, value) {
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
                    if dims_compatible { return true; }
                }
            }
        }
        false
    }

    fn check_function_pointer_assignable(&mut self, target: &Type, value: &Type, loc: &SourceLoc) -> bool {
        if target.is_function_pointer() && value.is_function_pointer() {
            if let (Type::Pointer { pointee: t_pointee, .. }, Type::Pointer { pointee: v_pointee, .. }) = (target, value) {
                if let (Type::Function { return_type: t_ret, param_types: t_params, .. },
                        Type::Function { return_type: v_ret, param_types: v_params, .. }) = (t_pointee.as_ref(), v_pointee.as_ref()) {
                    if t_params.len() == v_params.len() {
                        let params_compatible = t_params.iter().zip(v_params.iter()).all(|(a, b)| a == b);
                        if params_compatible && t_ret == v_ret {
                            return true;
                        }
                    }
                }
            }
            self.report_warning("函数指针类型不完全匹配，赋值可能存在风险", loc, ErrorCode::W3053_ImplicitScalarConversion);
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
        if !matches!(target.kind(), TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double | TypeKind::LongLong) {
            return false;
        }
        if !matches!(value.kind(), TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double | TypeKind::LongLong) {
            return false;
        }
        // 警告可能丢失精度的情况
        if matches!(target.kind(), TypeKind::Char) && matches!(value.kind(), TypeKind::Int | TypeKind::Float | TypeKind::Double | TypeKind::LongLong) {
            self.report_warning("被隐式转换为 char，可能会丢失精度。", loc, ErrorCode::W3053_ImplicitScalarConversion);
        }
        if matches!(target.kind(), TypeKind::Int) && matches!(value.kind(), TypeKind::Float | TypeKind::Double | TypeKind::LongLong) {
            self.report_warning(&format!("{} 被隐式转换为 int，可能会丢失精度。", value), loc, ErrorCode::W3053_ImplicitScalarConversion);
        }
        if matches!(target.kind(), TypeKind::Float) && matches!(value.kind(), TypeKind::Double | TypeKind::LongLong) {
            self.report_warning("double 被隐式转换为 float，可能会丢失精度。", loc, ErrorCode::W3053_ImplicitScalarConversion);
        }
        // 提示安全的隐式提升
        if matches!(target.kind(), TypeKind::Int) && matches!(value.kind(), TypeKind::Char) {
            self.report_hint("char 被隐式提升为 int。", loc, ErrorCode::H3057_ImplicitConversionHint);
        }
        if matches!(target.kind(), TypeKind::Float) && matches!(value.kind(), TypeKind::Int | TypeKind::Char) {
            let src = if matches!(value.kind(), TypeKind::Char) { "char" } else { "int" };
            self.report_hint(&format!("{} 被隐式提升为 float。", src), loc, ErrorCode::H3057_ImplicitConversionHint);
        }
        if matches!(target.kind(), TypeKind::Double) && matches!(value.kind(), TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::LongLong) {
            let src = match value.kind() {
                TypeKind::Char => "char",
                TypeKind::Float => "float",
                TypeKind::LongLong => "long long",
                _ => "int",
            };
            self.report_hint(&format!("{} 被隐式提升为 double。", src), loc, ErrorCode::H3057_ImplicitConversionHint);
        }
        true
    }

    fn check_pointer_assignable(&mut self, target: &Type, value: &Type, loc: &SourceLoc) -> bool {
        if matches!(target.kind(), TypeKind::Pointer) && matches!(value.kind(), TypeKind::Int) {
            self.report_warning("整数被隐式转换为指针。建议确保这是有意义的地址值（如 NULL = 0）。", loc, ErrorCode::W3054_IntToPointerCast);
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
            if let (Type::Pointer { is_const: t_const, .. }, Type::Pointer { pointee: v_pointee, is_const: v_const }) = (target, value) {
                if matches!(v_pointee.as_ref(), Type::Void { .. }) {
                    self.report_hint("void* 被隐式转换为具体指针类型。", loc, ErrorCode::H3057_ImplicitConversionHint);
                }
                if *v_const && !*t_const {
                    self.report_warning("将 const 指针赋值给非 const 指针，可能通过后者修改 const 数据。", loc, ErrorCode::W3053_ImplicitScalarConversion);
                }
            }
            return true;
        }
        false
    }

    pub(crate) fn check_assignable(&mut self, target: &Type, value: &Type, loc: &SourceLoc) -> bool {
        if target == value { return true; }
        if self.check_array_pointer_assignable(target, value, loc) { return true; }
        if self.check_function_pointer_assignable(target, value, loc) { return true; }
        if self.check_scalar_assignable(target, value, loc) { return true; }
        if self.check_pointer_assignable(target, value, loc) { return true; }
        false
    }

    pub(crate) fn get_struct_field_type(&self, struct_name: &str, field_name: &str) -> Option<Type> {
        let sym = self.structs.get(struct_name)?;
        for (fty, fname) in &sym.fields {
            if fname == field_name { return Some(fty.clone()); }
        }
        None
    }
    pub(crate) fn get_union_field_type(&self, union_name: &str, field_name: &str) -> Option<Type> {
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
        let fields = match self.structs.get(struct_type.name()) {
            Some(s) => s.fields.clone(),
            None => {
                self.report_error(&format!("未知的结构体类型 '{}'", struct_type.name()), loc, ErrorCode::E3004_TypeMismatch);
                return;
            }
        };
        if elements.len() > fields.len() {
            self.report_error("初始化列表元素数量超过结构体字段数", loc, ErrorCode::E3005_ArrayInitTooMany);
        }
        for (i, elem) in elements.iter_mut().enumerate() {
            if i >= fields.len() { break; }
            if fields[i].0.is_struct() && matches!(elem, Expr::InitList { .. }) {
                self.check_struct_initializer(&fields[i].0, elem, loc);
            } else if fields[i].0.is_array() && matches!(elem, Expr::InitList { .. }) {
                let mut sub_ty = fields[i].0.clone();
                self.check_array_initializer(&mut sub_ty, elem, loc);
            } else {
                let e_type = self.resolve_expr_type(elem);
                if !self.check_assignable(&fields[i].0, &e_type, loc) {
                    self.report_error(&format!("结构体初始化类型不匹配：字段 '{}' 期望 '{}'，实际 '{}'", fields[i].1, fields[i].0, e_type), loc, ErrorCode::E3006_ArrayInitTypeMismatch);
                }
            }
        }
    }

    fn validate_nested_init_list(&mut self, dims: &[i32], init: &mut Expr, loc: &SourceLoc, element_type: &Type) -> bool {
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
                self.report_error(&format!("数组初始化元素类型不匹配：期望 '{}'，实际 '{}'", element_type, e_type), loc, ErrorCode::E3006_ArrayInitTypeMismatch);
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
                self.report_error(&format!("多维数组初始化必须使用嵌套初始化列表，不能是 '{}'", init_type), loc, ErrorCode::E3009_InvalidArrayInit);
            }
            return;
        }

        if let Expr::InitList { elements, .. } = init {
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
                if elem_type.is_struct() && matches!(elem, Expr::InitList { .. }) {
                    self.check_struct_initializer(&elem_type, elem, loc);
                } else if elem_type.is_array() && matches!(elem, Expr::InitList { .. }) {
                    let mut sub_ty = elem_type.clone();
                    self.check_array_initializer(&mut sub_ty, elem, loc);
                } else {
                    let e_type = self.resolve_expr_type(elem);
                    if !self.check_assignable(&elem_type, &e_type, loc) {
                        self.report_error(&format!("数组初始化元素类型不匹配：期望 '{}'，实际 '{}'", elem_type, e_type), loc, ErrorCode::E3006_ArrayInitTypeMismatch);
                    } else {
                        insert_implicit_cast(elem, &elem_type);
                    }
                }
            }
        } else if let Expr::StringLiteral { value, .. } = init {
            if elem_type.kind() != TypeKind::Char {
                self.report_error("字符串字面量只能用于初始化 char 数组", loc, ErrorCode::E3007_StringInitNonCharArray);
                return;
            }
            let str_len = value.len() as i32;
            if arr_type.array_size() <= 0 {
                if let Type::Array { array_size, .. } = arr_type { *array_size = str_len + 1; }
            } else if str_len + 1 > arr_type.array_size() {
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
        self.current_file = node.source_file.clone();
        self.current_func_return = node.return_type.clone();
        self.current_func_params.clear();
        self.enter_scope();
        for p in &node.params {
            self.current_func_params.insert(p.name.clone());
            self.declare_var(&p.name, &p.ty, false, false);
        }
        if let Some(ref mut body) = node.body {
            self.dispatch_stmt(body);
        }
        self.exit_scope();
        self.current_func_params.clear();
    }

    fn dispatch_stmt(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::Block { stmts, .. } => {
                self.enter_scope();
                for s in stmts { self.dispatch_stmt(s); }
                self.exit_scope();
            }
            Stmt::VarDecl { var_type, name, init, extra_vars, loc, .. } => {
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
                self.declare_var(name, var_type, false, false);
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
                    self.declare_var(ename, ety, false, false);
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
                for s in step { self.resolve_expr_type(s); }
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
        if !self.is_scalar(&ty) && !matches!(ty.kind(), TypeKind::Pointer | TypeKind::Array) {
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


    pub(crate) fn is_builtin_func(&self, name: &str) -> bool {
        crate::vm::host_func_id::is_builtin(name)
    }

    pub(crate) fn visit_call(&mut self, name: &str, args: &mut [Expr], loc: &SourceLoc) -> Type {
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
            "fopen" => self.check_builtin_fopen(args, loc),
            "fread" => self.check_builtin_fread(args, loc),
            "fwrite" => self.check_builtin_fwrite(args, loc),
            "fclose" => self.check_builtin_fclose(args, loc),
            "feof" => self.check_builtin_feof(args, loc),
            "fgets" => self.check_builtin_fgets(args, loc),
            "fputs" => self.check_builtin_fputs(args, loc),
            "fprintf" => self.check_builtin_fprintf(args, loc),
            "realloc" => self.check_builtin_realloc(args, loc),
            "qsort" => {
                if self.funcs.contains_key(name) {
                    self.check_user_func(name, args, loc)
                } else {
                    self.check_builtin_qsort(args, loc)
                }
            }
            // math.h functions are resolved through stub declarations in funcs
            _ => self.check_user_func(name, args, loc),
        }
    }

    // ---------- 内建函数检查器辅助方法 ----------

    fn builtin_check_count(&mut self, args: &[Expr], expected: usize, name: &str, loc: &SourceLoc) -> bool {
        if args.len() != expected {
            self.report_error(&format!("{} 需要{}个参数", name, expected), loc, ErrorCode::E3028_BuiltInArgCount);
            false
        } else {
            true
        }
    }

    fn builtin_check_pointer(&mut self, arg: &mut Expr, idx: usize, name: &str, loc: &SourceLoc) {
        let arg_type = self.resolve_expr_type(arg);
        if !arg_type.is_pointer() && !arg_type.is_array() {
            self.report_error(&format!("{} 的第 {} 个参数必须是指针或数组", name, idx + 1), loc, ErrorCode::E3029_BuiltInArgType);
        }
    }

    fn builtin_check_int(&mut self, arg: &mut Expr, idx: usize, name: &str, loc: &SourceLoc) {
        let expected = Type::int();
        let arg_type = self.resolve_expr_type(arg);
        if !self.check_assignable(&expected, &arg_type, loc) {
            self.report_error(&format!("{} 的第 {} 个参数必须是 int", name, idx + 1), loc, ErrorCode::E3029_BuiltInArgType);
        } else {
            insert_implicit_cast(arg, &expected);
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
        Type::pointer_to(Type::void())
    }

    fn check_builtin_free(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "free", loc) {
            self.builtin_check_pointer(&mut args[0], 0, "free", loc);
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

    /// 解析 printf/scanf 格式字符串，返回非 %% 的格式说明符列表。
    /// 每个元素为 (spec_char, length_mod) 例如 ('d', "") 或 ('f', "l")
    fn parse_format_specs(fmt: &str) -> Vec<(char, String)> {
        let mut specs = Vec::new();
        let mut chars = fmt.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '%' {
                if let Some(&next) = chars.peek() {
                    if next == '%' {
                        chars.next(); // skip %%
                        continue;
                    }
                }
                // skip flags
                while let Some(&c) = chars.peek() {
                    if c == '-' || c == '+' || c == ' ' || c == '#' || c == '0' {
                        chars.next();
                    } else { break; }
                }
                // skip width
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '*' { chars.next(); } else { break; }
                }
                // skip precision
                if let Some(&'.') = chars.peek() {
                    chars.next();
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_digit() || c == '*' { chars.next(); } else { break; }
                    }
                }
                // length modifier
                let mut len_mod = String::new();
                if let Some(&c) = chars.peek() {
                    if c == 'l' || c == 'h' || c == 'L' || c == 'z' || c == 'j' || c == 't' {
                        chars.next();
                        len_mod.push(c);
                        if c == 'l' || c == 'h' {
                            if let Some(&c2) = chars.peek() {
                                if c2 == c {
                                    chars.next();
                                    len_mod.push(c2);
                                }
                            }
                        }
                    }
                }
                if let Some(&spec) = chars.peek() {
                    chars.next();
                    specs.push((spec, len_mod));
                }
            }
        }
        specs
    }

    fn check_printf_format(&mut self, fmt: &str, args: &mut [Expr], loc: &SourceLoc) {
        let specs = Self::parse_format_specs(fmt);
        if specs.len() != args.len() {
            self.report_error(
                &format!("printf 格式说明符数量（{}）与参数数量（{}）不匹配", specs.len(), args.len()),
                loc,
                ErrorCode::E3032_PrintfArgType,
            );
        }
        for (i, ((spec, len_mod), arg)) in specs.iter().zip(args.iter_mut()).enumerate() {
            let arg_type = self.resolve_expr_type(arg);
            let ok = match (*spec, len_mod.as_str()) {
                ('d' | 'i' | 'u' | 'x' | 'X' | 'o' | 'n', "") => {
                    matches!(arg_type.kind(), TypeKind::Int | TypeKind::Char)
                }
                ('d' | 'i' | 'u' | 'x' | 'X' | 'o', "l" | "ll") => {
                    matches!(arg_type.kind(), TypeKind::LongLong | TypeKind::Int)
                }
                ('f' | 'e' | 'g' | 'E' | 'G', "") => {
                    matches!(arg_type.kind(), TypeKind::Float | TypeKind::Double)
                }
                ('f' | 'e' | 'g' | 'E' | 'G', "l") | ('F', "") => {
                    matches!(arg_type.kind(), TypeKind::Double | TypeKind::Float)
                }
                ('c', "") => {
                    matches!(arg_type.kind(), TypeKind::Int | TypeKind::Char)
                }
                ('s', "") => {
                    arg_type.is_pointer() || arg_type.is_array()
                }
                ('p', "") => {
                    arg_type.is_pointer() || arg_type.is_array()
                }
                _ => true, // unknown spec, be permissive
            };
            if !ok {
                self.report_error(
                    &format!("printf 格式 '%{}' 与第 {} 个参数类型 '{}' 不匹配", spec, i + 2, arg_type),
                    loc,
                    ErrorCode::E3062_PrintfFormatMismatch,
                );
            }
        }
    }

    fn check_scanf_format(&mut self, fmt: &str, args: &mut [Expr], loc: &SourceLoc) {
        let specs = Self::parse_format_specs(fmt);
        if specs.len() != args.len() {
            self.report_error(
                &format!("scanf 格式说明符数量（{}）与参数数量（{}）不匹配", specs.len(), args.len()),
                loc,
                ErrorCode::E3035_ScanfArgType,
            );
        }
        for (i, ((spec, len_mod), arg)) in specs.iter().zip(args.iter_mut()).enumerate() {
            let arg_type = self.resolve_expr_type(arg);
            // scanf args must be pointers
            let pointee = if let Type::Pointer { pointee, .. } = &arg_type {
                Some((**pointee).clone())
            } else if let Type::Array { element, .. } = &arg_type {
                Some((**element).clone())
            } else {
                None
            };
            let ok = match (*spec, len_mod.as_str()) {
                ('d' | 'i' | 'u' | 'x' | 'X' | 'o' | 'n', "") => {
                    pointee.as_ref().is_some_and(|t| matches!(t.kind(), TypeKind::Int | TypeKind::Char))
                }
                ('d' | 'i' | 'u' | 'x' | 'X' | 'o', "l" | "ll") => {
                    pointee.as_ref().is_some_and(|t| matches!(t.kind(), TypeKind::LongLong | TypeKind::Int))
                }
                ('f' | 'e' | 'g' | 'E' | 'G', "") => {
                    pointee.as_ref().is_some_and(|t| matches!(t.kind(), TypeKind::Float | TypeKind::Int))
                }
                ('f' | 'e' | 'g' | 'E' | 'G', "l") | ('F', "") => {
                    pointee.as_ref().is_some_and(|t| matches!(t.kind(), TypeKind::Double | TypeKind::Float | TypeKind::Int))
                }
                ('c', "") => {
                    pointee.as_ref().is_some_and(|t| matches!(t.kind(), TypeKind::Char | TypeKind::Int))
                }
                ('s', "") => {
                    arg_type.is_pointer() || arg_type.is_array()
                }
                ('p', "") => {
                    arg_type.is_pointer() || arg_type.is_array()
                }
                _ => true,
            };
            if !ok {
                self.report_error(
                    &format!("scanf 格式 '%{}' 与第 {} 个参数类型 '{}' 不匹配", spec, i + 2, arg_type),
                    loc,
                    ErrorCode::E3063_ScanfFormatMismatch,
                );
            }
        }
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
            // 如果格式字符串是字面量，进行格式-参数类型匹配检查
            let fmt_str = if let Expr::StringLiteral { ref value, .. } = args[0] {
                Some(value.clone())
            } else {
                None
            };
            if let Some(fmt) = fmt_str {
                self.check_printf_format(&fmt, &mut args[1..], loc);
            } else {
                // 非字面量格式字符串，只做粗略检查
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
            let fmt_str = if let Expr::StringLiteral { ref value, .. } = args[0] {
                Some(value.clone())
            } else {
                None
            };
            if let Some(fmt) = fmt_str {
                self.check_scanf_format(&fmt, &mut args[1..], loc);
            } else {
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
        }
        Type::void()
    }

    fn check_builtin_strlen(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "strlen", loc) {
            self.builtin_check_pointer(&mut args[0], 0, "strlen", loc);
        }
        Type::int()
    }

    fn check_builtin_strcpy(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 2, "strcpy", loc) {
            self.builtin_check_pointer(&mut args[0], 0, "strcpy", loc);
            self.builtin_check_pointer(&mut args[1], 1, "strcpy", loc);
        }
        Type::pointer_to(Type::char())
    }

    fn check_builtin_strcmp(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 2, "strcmp", loc) {
            self.builtin_check_pointer(&mut args[0], 0, "strcmp", loc);
            self.builtin_check_pointer(&mut args[1], 1, "strcmp", loc);
        }
        Type::int()
    }

    fn check_builtin_getchar(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        self.builtin_check_count(args, 0, "getchar", loc);
        Type::int()
    }

    fn check_builtin_putchar(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "putchar", loc) {
            self.builtin_check_int(&mut args[0], 0, "putchar", loc);
        }
        Type::void()
    }

    fn check_builtin_rand(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        self.builtin_check_count(args, 0, "rand", loc);
        Type::int()
    }

    fn check_builtin_srand(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "srand", loc) {
            self.builtin_check_int(&mut args[0], 0, "srand", loc);
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
        Type::pointer_to(Type::void())
    }

    fn check_builtin_exit(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "exit", loc) {
            self.builtin_check_int(&mut args[0], 0, "exit", loc);
        }
        Type::void()
    }

    fn check_builtin_strcat(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 2, "strcat", loc) {
            self.builtin_check_pointer(&mut args[0], 0, "strcat", loc);
            self.builtin_check_pointer(&mut args[1], 1, "strcat", loc);
        }
        Type::pointer_to(Type::char())
    }

    fn check_builtin_atoi(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "atoi", loc) {
            self.builtin_check_pointer(&mut args[0], 0, "atoi", loc);
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
            if !stream_type.is_pointer() && !matches!(stream_type.kind(), TypeKind::Int) {
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
            if !ptr_type.is_pointer() && !matches!(ptr_type.kind(), TypeKind::Int) {
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
        Type::pointer_to(Type::void())
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
            if !matches!(compar_type.kind(), TypeKind::Int) && !compar_type.is_pointer() {
                self.report_error(
                    "qsort 第四个参数必须是函数指针",
                    loc,
                    ErrorCode::E3029_BuiltInArgType,
                );
            }
        }
        Type::void()
    }

    fn check_builtin_fopen(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 2 {
            self.report_error("fopen 需要两个参数（路径和模式）", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            for (i, arg) in args.iter_mut().enumerate() {
                let arg_type = self.resolve_expr_type(arg);
                if !arg_type.is_pointer() && !arg_type.is_array() {
                    self.report_error(
                        &format!("fopen 第 {} 个参数必须是字符串", i + 1),
                        loc,
                        ErrorCode::E3029_BuiltInArgType,
                    );
                }
            }
        }
        Type::pointer_to(Type::void())
    }

    fn check_builtin_fread(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 4 {
            self.report_error("fread 需要四个参数（缓冲区、元素大小、元素数量、文件指针）", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            let buf_type = self.resolve_expr_type(&mut args[0]);
            if !buf_type.is_pointer() && !buf_type.is_array() {
                self.report_error("fread 第一个参数必须是指针或数组", loc, ErrorCode::E3029_BuiltInArgType);
            }
            for i in 1..3 {
                let t = self.resolve_expr_type(&mut args[i]);
                if !self.check_assignable(&Type::int(), &t, loc) {
                    self.report_error(&format!("fread 第 {} 个参数必须是 int", i + 1), loc, ErrorCode::E3029_BuiltInArgType);
                } else {
                    insert_implicit_cast(&mut args[i], &Type::int());
                }
            }
            let stream_type = self.resolve_expr_type(&mut args[3]);
            if !stream_type.is_pointer() && !matches!(stream_type.kind(), TypeKind::Int) {
                self.report_error("fread 第四个参数必须是文件指针", loc, ErrorCode::E3029_BuiltInArgType);
            }
        }
        Type::int()
    }

    fn check_builtin_fwrite(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if args.len() != 4 {
            self.report_error("fwrite 需要四个参数（缓冲区、元素大小、元素数量、文件指针）", loc, ErrorCode::E3028_BuiltInArgCount);
        } else {
            let buf_type = self.resolve_expr_type(&mut args[0]);
            if !buf_type.is_pointer() && !buf_type.is_array() {
                self.report_error("fwrite 第一个参数必须是指针或数组", loc, ErrorCode::E3029_BuiltInArgType);
            }
            for i in 1..3 {
                let t = self.resolve_expr_type(&mut args[i]);
                if !self.check_assignable(&Type::int(), &t, loc) {
                    self.report_error(&format!("fwrite 第 {} 个参数必须是 int", i + 1), loc, ErrorCode::E3029_BuiltInArgType);
                } else {
                    insert_implicit_cast(&mut args[i], &Type::int());
                }
            }
            let stream_type = self.resolve_expr_type(&mut args[3]);
            if !stream_type.is_pointer() && !matches!(stream_type.kind(), TypeKind::Int) {
                self.report_error("fwrite 第四个参数必须是文件指针", loc, ErrorCode::E3029_BuiltInArgType);
            }
        }
        Type::int()
    }

    fn check_builtin_fclose(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "fclose", loc) {
            let stream_type = self.resolve_expr_type(&mut args[0]);
            if !stream_type.is_pointer() && !matches!(stream_type.kind(), TypeKind::Int) {
                self.report_error("fclose 参数必须是文件指针", loc, ErrorCode::E3029_BuiltInArgType);
            }
        }
        Type::int()
    }

    fn check_builtin_feof(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 1, "feof", loc) {
            let stream_type = self.resolve_expr_type(&mut args[0]);
            if !stream_type.is_pointer() && !matches!(stream_type.kind(), TypeKind::Int) {
                self.report_error("feof 参数必须是文件指针", loc, ErrorCode::E3029_BuiltInArgType);
            }
        }
        Type::int()
    }

    fn check_builtin_fgets(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 3, "fgets", loc) {
            let buf_type = self.resolve_expr_type(&mut args[0]);
            if !buf_type.is_pointer() && !buf_type.is_array() {
                self.report_error("fgets 第一个参数必须是指针或数组", loc, ErrorCode::E3029_BuiltInArgType);
            }
            let n_type = self.resolve_expr_type(&mut args[1]);
            if !self.check_assignable(&Type::int(), &n_type, loc) {
                self.report_error("fgets 第二个参数必须是 int", loc, ErrorCode::E3029_BuiltInArgType);
            } else {
                insert_implicit_cast(&mut args[1], &Type::int());
            }
            let stream_type = self.resolve_expr_type(&mut args[2]);
            if !stream_type.is_pointer() && !matches!(stream_type.kind(), TypeKind::Int) {
                self.report_error("fgets 第三个参数必须是文件指针", loc, ErrorCode::E3029_BuiltInArgType);
            }
        }
        Type::pointer_to(Type::char())
    }

    fn check_builtin_fputs(&mut self, args: &mut [Expr], loc: &SourceLoc) -> Type {
        if self.builtin_check_count(args, 2, "fputs", loc) {
            let s_type = self.resolve_expr_type(&mut args[0]);
            if !s_type.is_pointer() && !s_type.is_array() {
                self.report_error("fputs 第一个参数必须是字符串", loc, ErrorCode::E3029_BuiltInArgType);
            }
            let stream_type = self.resolve_expr_type(&mut args[1]);
            if !stream_type.is_pointer() && !matches!(stream_type.kind(), TypeKind::Int) {
                self.report_error("fputs 第二个参数必须是文件指针", loc, ErrorCode::E3029_BuiltInArgType);
            }
        }
        Type::int()
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
            return sym.return_type.clone();
        }

        if let Some(sym) = self.static_func_sigs.get(name).cloned() {
            if let Some(files) = self.static_func_files.get(name) {
                if !files.contains(&self.current_file) {
                    self.report_error(
                        &format!("static 函数 '{}' 在其他文件中不可见", name),
                        loc,
                        ErrorCode::E3058_StaticFuncAccess,
                    );
                    return Type::void();
                }
            }
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
            return sym.return_type.clone();
        }

        self.report_error(
            &format!("未定义的函数 '{}'", name),
            loc,
            ErrorCode::E3036_UndefinedFunc,
        );
        Type::void()
    }
}

mod expr;
