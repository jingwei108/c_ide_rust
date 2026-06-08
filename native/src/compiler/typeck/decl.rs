use super::*;

impl TypeChecker {
    // =========================================================================
    // Function / Statement visitors
    // =========================================================================

    pub(super) fn visit_func_decl(&mut self, node: &mut FuncDecl) {
        self.current_file = node.source_file.clone();
        self.current_func_return = node.return_type.clone();
        self.current_func_params.clear();
        self.func_labels.clear();
        self.pending_gotos.clear();
        self.enter_scope();
        for p in &node.params {
            self.current_func_params.insert(p.name.clone());
            self.declare_var(&p.name, &p.ty, false, false, false);
        }
        if let Some(ref mut body) = node.body {
            self.dispatch_stmt(body);
        }
        let unresolved: Vec<(String, SourceLoc)> = self.pending_gotos.iter()
            .filter(|(label, _)| !self.func_labels.contains_key(label))
            .map(|(label, loc)| (label.clone(), *loc))
            .collect();
        for (label, loc) in unresolved {
            self.report_error(&format!("goto 目标标签 '{}' 未定义", label), &loc, ErrorCode::E3071_UndefinedLabel);
        }
        self.pending_gotos.clear();
        self.func_labels.clear();
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
            Stmt::VarDecl { var_type, name, init, extra_vars, loc, is_static, .. } => {
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
                self.declare_var(name, var_type, false, false, *is_static);
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
                    self.declare_var(ename, ety, false, false, false);
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
            Stmt::Goto { label, loc } => {
                if self.func_labels.contains_key(label) {
                    // Label already defined, ok
                } else {
                    self.pending_gotos.push((label.clone(), *loc));
                }
            }
            Stmt::Label { label, stmt, loc } => {
                if self.func_labels.contains_key(label) {
                    self.report_error(&format!("标签 '{}' 重复定义", label), loc, ErrorCode::E3001_VarRedeclared);
                } else {
                    self.func_labels.insert(label.clone(), *loc);
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

    pub(crate) fn check_user_func(&mut self, name: &str, args: &mut [Expr], loc: &SourceLoc) -> Type {
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
