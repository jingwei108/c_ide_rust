use super::*;

impl TypeChecker {
    pub(crate) fn resolve_new(&mut self, expr: &mut Expr) -> Type {
        let (elem_type, size_expr, init, loc) = if let Expr::New {
            elem_type,
            size_expr,
            init,
            loc,
            ..
        } = expr
        {
            (&mut *elem_type, size_expr.as_deref_mut(), init.as_deref_mut(), *loc)
        } else {
            unreachable!()
        };

        // Class template instantiation for new
        *elem_type = self.resolve_template_id(elem_type, &loc);
        if let Some(se) = size_expr {
            let size_ty = self.resolve_expr_type(se);
            if !self.is_int(&size_ty) {
                self.report_error("new[] 的大小必须是 int 类型", &loc, ErrorCode::E4027_InvalidNewType);
            }
        }
        if let Some(i) = init {
            // C++ 类类型 new：将占位符 __ctor__* 解析为具体 mangled 构造函数名
            if matches!(elem_type, Type::Class { .. }) {
                let mut ctor_class_name = String::new();
                let mut ctor_name = String::new();
                if let Expr::Call { name, args, .. } = i {
                    if name.starts_with("__ctor__") {
                        if let Type::Class { name: class_name, .. } = elem_type {
                            let arg_count = args.len();
                            if let Some(resolved) = self.resolve_constructor_overload(class_name, arg_count, loc) {
                                *name = resolved.clone();
                                ctor_class_name = class_name.clone();
                                ctor_name = resolved;
                            } else {
                                self.report_error(
                                    &format!("类 '{}' 没有接受 {} 个参数的构造函数", class_name, arg_count),
                                    &loc,
                                    ErrorCode::E3003_FuncRedeclared,
                                );
                            }
                        }
                    }
                }
                // 构造函数符号包含隐式 this 指针，但 new 表达式的 init 中尚未插入。
                // 因此这里直接根据类方法签名检查用户参数，避免 resolve_expr_type 因缺少 this 而报错。
                if !ctor_name.is_empty() {
                    if let Some(sigs) = self.find_class_method_sigs(&ctor_class_name, &ctor_name) {
                        // Constructor MethodSig stores only user parameter types
                        // (the implicit this pointer is added by the caller).
                        let expected = &sigs.first().map(|s| s.param_types.clone()).unwrap_or_default();
                        if let Expr::Call { args, .. } = i {
                            if expected.len() != args.len() {
                                self.report_error(
                                    &format!(
                                        "构造函数 '{}' 参数数量不匹配：期望 {}，实际 {}",
                                        ctor_name,
                                        expected.len(),
                                        args.len()
                                    ),
                                    &loc,
                                    ErrorCode::E3037_FuncArgCount,
                                );
                            } else {
                                for (arg, exp) in args.iter_mut().zip(expected.iter()) {
                                    let arg_ty = self.resolve_expr_type(arg);
                                    if !self.check_assignable(exp, &arg_ty, &loc) {
                                        self.report_error(
                                            &format!("构造函数参数类型不匹配：期望 '{}'，实际 '{}'", exp, arg_ty),
                                            &loc,
                                            ErrorCode::E3004_TypeMismatch,
                                        );
                                    } else {
                                        insert_implicit_cast(arg, exp);
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                let init_ty = self.resolve_expr_type(i);
                if !self.check_assignable(elem_type, &init_ty, &loc) {
                    self.report_error(
                        &format!("new 的初始化类型不匹配：期望 '{}'，实际 '{}'", elem_type, init_ty),
                        &loc,
                        ErrorCode::E4027_InvalidNewType,
                    );
                }
            }
        }

        let result = Type::Pointer {
            pointee: Box::new(elem_type.clone()),
            is_const: false,
        };
        if let Expr::New { ty, .. } = expr {
            *ty = result.clone();
        }
        result
    }

    pub(crate) fn resolve_delete(&mut self, expr: &mut Expr) -> Type {
        let (inner, loc) = if let Expr::Delete { expr: inner, loc, .. } = expr {
            (inner.as_mut(), *loc)
        } else {
            unreachable!()
        };
        let expr_ty = self.resolve_expr_type(inner);
        if !expr_ty.is_pointer() {
            self.report_error("delete 只能用于指针类型", &loc, ErrorCode::E4028_InvalidDeleteType);
        }
        if let Expr::Delete { ty, .. } = expr {
            *ty = Type::void();
            ty.clone()
        } else {
            unreachable!()
        }
    }

    pub(crate) fn resolve_move(&mut self, expr: &mut Expr) -> Type {
        let inner = if let Expr::Move { expr: inner, .. } = expr {
            inner.as_mut()
        } else {
            unreachable!()
        };
        let expr_ty = self.resolve_expr_type(inner);
        let result = Type::RValueRef { base: Box::new(expr_ty) };
        if let Expr::Move { ty, .. } = expr {
            *ty = result.clone();
        }
        result
    }

    pub(crate) fn resolve_lambda(&mut self, expr: &mut Expr) -> Type {
        if let Expr::Lambda {
            params,
            body,
            unique_id,
            loc,
            ty,
            capture,
        } = expr
        {
            // Generate a closure struct type
            let lambda_name = format!("__lambda_{}", unique_id);
            let result = Type::Class {
                name: lambda_name.clone(),
                is_const: false,
            };
            // Collect capture variable types from current scope
            let mut capture_info = Vec::new();
            let mut fields = Vec::new();
            for cap in capture.iter() {
                match cap {
                    CaptureMode::ByValue(name) | CaptureMode::ByReference(name) => {
                        if let Some(sym) = self.lookup_var(name) {
                            let is_by_ref = matches!(cap, CaptureMode::ByReference(_));
                            capture_info.push((name.clone(), sym.ty.clone(), is_by_ref));
                            fields.push((sym.ty.clone(), name.clone(), AccessSpec::Public));
                        }
                    }
                    CaptureMode::Implicit => {}
                }
            }
            // Register the lambda type as a class symbol with capture fields
            if !self.classes.contains_key(&lambda_name) {
                let size = fields.iter().map(|(fty, _, _)| self.compute_type_size(fty)).sum();
                self.classes.insert(
                    lambda_name.clone(),
                    ClassSymbol {
                        fields,
                        static_fields: vec![],
                        methods: HashMap::new(),
                        base: None,
                        vtable: None,
                        size,
                        has_resource: false,
                    },
                );
            }
            // Register lambda call function
            let call_name = format!("{}__call", lambda_name);
            if !self.funcs.contains_key(&call_name) {
                let mut call_params = vec![Param {
                    name: "this".to_string(),
                    ty: Type::Pointer {
                        pointee: Box::new(result.clone()),
                        is_const: false,
                    },
                    loc: *loc,
                }];
                call_params.extend(params.iter().cloned());
                self.funcs.insert(
                    call_name.clone(),
                    FuncSymbol {
                        return_type: Type::int(),
                        param_types: call_params.iter().map(|p| p.ty.clone()).collect(),
                        is_variadic: false,
                    },
                );
            }
            // Record pending lambda for later lifting
            self.pending_lambdas.push(super::LambdaInfo {
                id: *unique_id,
                captures: capture_info,
                params: params.clone(),
                body: body.as_ref().clone(),
                loc: *loc,
            });

            *ty = result.clone();
            return result;
        }
        unreachable!()
    }
}
