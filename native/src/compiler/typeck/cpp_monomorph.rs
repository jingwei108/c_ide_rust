use super::*;

impl TypeChecker {
    /// 尝试根据实参类型隐式实例化函数模板。
    /// 返回实例化后的 mangled 函数名（如 "max__int"），若无法匹配则返回 None。
    /// Try to monomorphize a function template. Returns (mangled_name, FuncDecl) if successful.
    /// Returns `Some((mangled_name, Some(func_decl)))` for a new instantiation,
    /// `Some((mangled_name, None))` if already instantiated,
    /// or `None` if the name is not a template or inference failed.
    pub(crate) fn try_monomorphize_func(
        &mut self,
        name: &str,
        arg_types: &[Type],
    ) -> Option<(String, Option<FuncDecl>)> {
        let template = self.templates.get(name)?.clone();
        let func_decl = match &template.decl {
            Templateable::Func(f) => f.as_ref(),
            _ => return None,
        };

        if template.params.is_empty() {
            return None;
        }

        // Derive template arguments from parameter types
        let mut type_map: HashMap<String, Type> = HashMap::new();
        for (param, arg_ty) in func_decl.params.iter().zip(arg_types.iter()) {
            self.infer_template_arg(&param.ty, arg_ty, &mut type_map);
        }
        // Also infer from return type if it's a template param (for cases like T foo())
        if let Some(first_arg) = arg_types.first() {
            self.infer_template_arg(&func_decl.return_type, first_arg, &mut type_map);
        }

        // Check all template params have been inferred
        let mut type_args: Vec<Type> = Vec::new();
        for tp in &template.params {
            if let Some(t) = type_map.get(&tp.name) {
                type_args.push(t.clone());
            } else {
                return None; // Could not infer all params
            }
        }

        let mangled = Self::mangle_template_name(name, &type_args);
        if self.funcs.contains_key(&mangled) {
            return Some((mangled, None)); // Already instantiated
        }

        // Instantiate: deep clone and replace template params
        let mut new_func = func_decl.clone();
        new_func.name = mangled.clone();
        self.replace_template_types_in_func(&mut new_func, &type_map);

        self.funcs.insert(
            mangled.clone(),
            FuncSymbol {
                return_type: new_func.return_type.clone(),
                param_types: new_func.params.iter().map(|p| p.ty.clone()).collect(),
            },
        );

        Some((mangled, Some(new_func)))
    }

    /// Mangling rule: func__T1_T2_...
    fn mangle_template_name(base: &str, type_args: &[Type]) -> String {
        let mut result = base.to_string();
        for t in type_args {
            result.push_str("__");
            result.push_str(&t.mangle_name());
        }
        result
    }

    /// Infer template argument by matching a formal parameter type against an actual type.
    fn infer_template_arg(&self, formal: &Type, actual: &Type, map: &mut HashMap<String, Type>) {
        match formal {
            Type::Class { name, .. }
                // Treat Class types with names not found in self.classes as template params
                // (Parser registers template params as Type::Class { name: param_name })
                if !self.classes.contains_key(name) && !self.structs.contains_key(name) => {
                    map.insert(name.clone(), actual.clone());
                }
            Type::Pointer { pointee, .. } => {
                match actual {
                    Type::Pointer { pointee: actual_pointee, .. } => {
                        self.infer_template_arg(pointee, actual_pointee, map);
                    }
                    Type::Array { element: actual_elem, .. } => {
                        // C array-to-pointer decay: T formal vs T[5] actual
                        self.infer_template_arg(pointee, actual_elem, map);
                    }
                    _ => {}
                }
            }
            Type::Array { element, .. } => {
                if let Type::Array { element: actual_elem, .. } = actual {
                    self.infer_template_arg(element, actual_elem, map);
                }
            }
            Type::Reference { base, .. } => {
                if let Type::Reference { base: actual_base, .. } = actual {
                    self.infer_template_arg(base, actual_base, map);
                }
            }
            _ => {}
        }
    }

    /// Recursively replace template parameter types in a function declaration.
    fn replace_template_types_in_func(&self, func: &mut FuncDecl, type_map: &HashMap<String, Type>) {
        func.return_type = Self::replace_template_type(&func.return_type, type_map);
        for p in &mut func.params {
            p.ty = Self::replace_template_type(&p.ty, type_map);
        }
        if let Some(ref mut body) = func.body {
            self.replace_template_types_in_stmt(body, type_map);
        }
    }

    fn replace_template_type(ty: &Type, type_map: &HashMap<String, Type>) -> Type {
        match ty {
            Type::Class { name, .. } if type_map.contains_key(name) => type_map[name].clone(),
            Type::TemplateId { base, args, .. } => {
                // 类模板体内出现同模板类实例化，如 unique_ptr<T>，需要把参数替换后生成 mangled 类名。
                let new_args: Vec<Type> = args.iter().map(|a| Self::replace_template_type(a, type_map)).collect();
                let mangled = Self::mangle_template_name(base, &new_args);
                Type::Class { name: mangled, is_const: false }
            }
            Type::Pointer { pointee, is_const } => Type::Pointer {
                pointee: Box::new(Self::replace_template_type(pointee, type_map)),
                is_const: *is_const,
            },
            Type::Array {
                element,
                array_size,
                dims,
                is_const,
                is_vla,
                vla_dims,
            } => Type::Array {
                element: Box::new(Self::replace_template_type(element, type_map)),
                array_size: *array_size,
                dims: dims.clone(),
                is_const: *is_const,
                is_vla: *is_vla,
                vla_dims: vla_dims.clone(),
            },
            Type::Reference { base, is_const } => Type::Reference {
                base: Box::new(Self::replace_template_type(base, type_map)),
                is_const: *is_const,
            },
            Type::RValueRef { base } => Type::RValueRef {
                base: Box::new(Self::replace_template_type(base, type_map)),
            },
            Type::Function {
                return_type,
                param_types,
                is_const,
            } => Type::Function {
                return_type: Box::new(Self::replace_template_type(return_type, type_map)),
                param_types: param_types.iter().map(|t| Self::replace_template_type(t, type_map)).collect(),
                is_const: *is_const,
            },
            _ => ty.clone(),
        }
    }

    fn replace_template_types_in_stmt(&self, stmt: &mut Stmt, type_map: &HashMap<String, Type>) {
        match stmt {
            Stmt::VarDecl { var_type, init, extra_vars, .. } => {
                *var_type = Self::replace_template_type(var_type, type_map);
                for (ety, _, einit) in extra_vars.iter_mut() {
                    *ety = Self::replace_template_type(ety, type_map);
                    if let Some(ref mut e) = einit {
                        self.replace_template_types_in_expr(e, type_map);
                    }
                }
                if let Some(ref mut e) = init {
                    self.replace_template_types_in_expr(e, type_map);
                }
            }
            Stmt::Block { stmts, .. } => {
                for s in stmts {
                    self.replace_template_types_in_stmt(s, type_map);
                }
            }
            Stmt::Switch { body, .. } => {
                self.replace_template_types_in_stmt(body, type_map);
            }
            Stmt::If { cond, then_stmt, else_stmt, .. } => {
                self.replace_template_types_in_expr(cond, type_map);
                self.replace_template_types_in_stmt(then_stmt, type_map);
                if let Some(ref mut e) = else_stmt {
                    self.replace_template_types_in_stmt(e, type_map);
                }
            }
            Stmt::While { cond, body, .. } => {
                self.replace_template_types_in_expr(cond, type_map);
                self.replace_template_types_in_stmt(body, type_map);
            }
            Stmt::DoWhile { body, cond, .. } => {
                self.replace_template_types_in_stmt(body, type_map);
                self.replace_template_types_in_expr(cond, type_map);
            }
            Stmt::For { init, cond, step, body, .. } => {
                if let Some(ref mut i) = init {
                    self.replace_template_types_in_stmt(i, type_map);
                }
                if let Some(ref mut c) = cond {
                    self.replace_template_types_in_expr(c, type_map);
                }
                for s in step {
                    self.replace_template_types_in_expr(s, type_map);
                }
                self.replace_template_types_in_stmt(body, type_map);
            }
            Stmt::Return { value: Some(ref mut v), .. } => {
                self.replace_template_types_in_expr(v, type_map);
            }
            Stmt::Expr { expr, .. } => {
                self.replace_template_types_in_expr(expr, type_map);
            }
            Stmt::RangeFor { var_type, iter, body, .. } => {
                *var_type = Self::replace_template_type(var_type, type_map);
                self.replace_template_types_in_expr(iter, type_map);
                self.replace_template_types_in_stmt(body, type_map);
            }
            Stmt::Case { stmt, .. } => {
                self.replace_template_types_in_stmt(stmt, type_map);
            }
            Stmt::Label { stmt, .. } => {
                self.replace_template_types_in_stmt(stmt, type_map);
            }
            _ => {}
        }
    }

    /// 若类型（含嵌套）为 TemplateId，触发类模板实例化并返回对应的 Class 类型；否则返回原类型。
    pub(crate) fn resolve_template_id(&mut self, ty: &Type, loc: &SourceLoc) -> Type {
        match ty.clone() {
            Type::TemplateId { base, args, .. } => {
                if let Some((mangled, new_class)) = self.try_monomorphize_class(&base, &args) {
                    self.pending_class_instantiations.push((mangled.clone(), new_class));
                    Type::Class { name: mangled, is_const: false }
                } else if !self.templates.contains_key(&base) {
                    self.report_error(&format!("未知模板类 '{}'", base), loc, ErrorCode::E3023_UndeclaredVar);
                    Type::int()
                } else {
                    let mangled = format!(
                        "{}__{}",
                        base,
                        args.iter().map(|a| a.mangle_name()).collect::<Vec<_>>().join("__")
                    );
                    Type::Class { name: mangled, is_const: false }
                }
            }
            Type::Pointer { pointee, is_const } => {
                let new_pointee = self.resolve_template_id(&pointee, loc);
                Type::Pointer {
                    pointee: Box::new(new_pointee),
                    is_const,
                }
            }
            Type::Array {
                element,
                array_size,
                dims,
                is_const,
                is_vla,
                vla_dims,
            } => {
                let new_element = self.resolve_template_id(&element, loc);
                Type::Array {
                    element: Box::new(new_element),
                    array_size,
                    dims,
                    is_const,
                    is_vla,
                    vla_dims,
                }
            }
            Type::Reference { base: inner, is_const } => {
                let new_inner = self.resolve_template_id(&inner, loc);
                Type::Reference {
                    base: Box::new(new_inner),
                    is_const,
                }
            }
            Type::RValueRef { base: inner } => {
                let new_inner = self.resolve_template_id(&inner, loc);
                Type::RValueRef { base: Box::new(new_inner) }
            }
            _ => ty.clone(),
        }
    }

    /// 尝试实例化类模板。返回 (mangled_name, ClassDecl) 若成功。
    pub(crate) fn try_monomorphize_class(&mut self, base: &str, args: &[Type]) -> Option<(String, ClassDecl)> {
        let template = self.templates.get(base)?.clone();
        let class_decl = match &template.decl {
            Templateable::Class(c) => c.as_ref(),
            _ => return None,
        };

        if template.params.is_empty() || template.params.len() != args.len() {
            return None;
        }

        let mut type_map: HashMap<String, Type> = HashMap::new();
        for (tp, arg) in template.params.iter().zip(args.iter()) {
            type_map.insert(tp.name.clone(), arg.clone());
        }

        let mangled = Self::mangle_template_name(base, args);
        if self.classes.contains_key(&mangled) || self.structs.contains_key(&mangled) {
            return None; // Already instantiated
        }

        let mut new_class = class_decl.clone();
        new_class.name = mangled.clone();
        self.replace_template_types_in_class(&mut new_class, &type_map);

        // Register layout immediately so Pass 3 can resolve members
        self.register_single_class_layout(&mangled, &new_class);

        Some((mangled, new_class))
    }

    fn replace_template_types_in_class(&self, class: &mut ClassDecl, type_map: &HashMap<String, Type>) {
        // Replace types in base class name? For now, base is not a template param in our subset
        for member in &mut class.members {
            match member {
                ClassMember::Field { ty, .. } => {
                    *ty = Self::replace_template_type(ty, type_map);
                }
                ClassMember::Method { ret, params, body, .. } => {
                    *ret = Self::replace_template_type(ret, type_map);
                    for p in params.iter_mut() {
                        p.ty = Self::replace_template_type(&p.ty, type_map);
                    }
                    if let Some(ref mut b) = body {
                        self.replace_template_types_in_stmt(b, type_map);
                    }
                }
                ClassMember::Constructor { params, body, .. } => {
                    for p in params.iter_mut() {
                        p.ty = Self::replace_template_type(&p.ty, type_map);
                    }
                    if let Some(ref mut b) = body {
                        self.replace_template_types_in_stmt(b, type_map);
                    }
                }
                ClassMember::Destructor { body, .. } => {
                    if let Some(ref mut b) = body {
                        self.replace_template_types_in_stmt(b, type_map);
                    }
                }
                ClassMember::NestedStruct { decl, .. } => {
                    for field in &mut decl.fields {
                        field.ty = Self::replace_template_type(&field.ty, type_map);
                    }
                }
                ClassMember::NestedClass { decl, .. } => {
                    self.replace_template_types_in_class(decl, type_map);
                }
            }
        }
    }

    fn replace_template_types_in_expr(&self, expr: &mut Expr, type_map: &HashMap<String, Type>) {
        match expr {
            Expr::Binary { left, right, .. } => {
                self.replace_template_types_in_expr(left, type_map);
                self.replace_template_types_in_expr(right, type_map);
            }
            Expr::Unary { operand, .. } => {
                self.replace_template_types_in_expr(operand, type_map);
            }
            Expr::Ternary {
                cond, then_branch, else_branch, ..
            } => {
                self.replace_template_types_in_expr(cond, type_map);
                self.replace_template_types_in_expr(then_branch, type_map);
                self.replace_template_types_in_expr(else_branch, type_map);
            }
            Expr::Assign { left, right, .. } => {
                self.replace_template_types_in_expr(left, type_map);
                self.replace_template_types_in_expr(right, type_map);
            }
            Expr::Call { args, .. } => {
                for a in args {
                    self.replace_template_types_in_expr(a, type_map);
                }
            }
            Expr::MemberCall { object, args, .. } => {
                self.replace_template_types_in_expr(object, type_map);
                for a in args {
                    self.replace_template_types_in_expr(a, type_map);
                }
            }
            Expr::Member { object, ty, .. } => {
                self.replace_template_types_in_expr(object, type_map);
                *ty = Self::replace_template_type(ty, type_map);
            }
            Expr::Index { array, index, ty, .. } => {
                self.replace_template_types_in_expr(array, type_map);
                self.replace_template_types_in_expr(index, type_map);
                *ty = Self::replace_template_type(ty, type_map);
            }
            Expr::Cast { expr, target_type, ty, .. } => {
                self.replace_template_types_in_expr(expr, type_map);
                *target_type = Self::replace_template_type(target_type, type_map);
                *ty = Self::replace_template_type(ty, type_map);
            }
            Expr::New { elem_type, size_expr, init, .. } => {
                *elem_type = Self::replace_template_type(elem_type, type_map);
                if let Some(ref mut s) = size_expr {
                    self.replace_template_types_in_expr(s, type_map);
                }
                if let Some(ref mut i) = init {
                    self.replace_template_types_in_expr(i, type_map);
                }
            }
            Expr::Delete { expr, .. } => {
                self.replace_template_types_in_expr(expr, type_map);
            }
            Expr::Lambda { params, body, .. } => {
                for p in params.iter_mut() {
                    p.ty = Self::replace_template_type(&p.ty, type_map);
                }
                self.replace_template_types_in_stmt(body, type_map);
            }
            Expr::InitList { elements, ty, .. } => {
                *ty = Self::replace_template_type(ty, type_map);
                for e in elements.iter_mut() {
                    self.replace_template_types_in_expr(&mut e.value, type_map);
                }
            }
            Expr::Sizeof { ty, .. } => {
                *ty = Self::replace_template_type(ty, type_map);
            }
            Expr::Offsetof { ty, .. } => {
                *ty = Self::replace_template_type(ty, type_map);
            }
            Expr::Move { expr, .. } => {
                self.replace_template_types_in_expr(expr, type_map);
            }
            _ => {}
        }
    }
}
