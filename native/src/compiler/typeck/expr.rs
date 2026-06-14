use super::*;

/// C usual arithmetic conversions: promote two scalar types to a common type.
fn promote_type(a: &Type, b: &Type) -> Type {
    use TypeKind::*;
    let rank = |t: &Type| match t.kind() {
        Double => 4,
        Float => 3,
        LongLong => 2,
        Int => 1,
        Char => 0,
        _ => -1,
    };
    let ra = rank(a);
    let rb = rank(b);
    let (higher, lower) = if ra >= rb { (a, b) } else { (b, a) };
    let is_unsigned = higher.is_unsigned() || lower.is_unsigned();
    match higher.kind() {
        Double => Type::double(),
        Float => Type::float(),
        LongLong => Type::LongLong { is_unsigned, is_const: false },
        Int => Type::Int { is_unsigned, is_const: false },
        Char => Type::Int { is_unsigned, is_const: false },
        _ => Type::int(),
    }
}

impl TypeChecker {
    /// Try to resolve an unqualified function call inside a class member function as a
    /// call to a class method (C++ name hiding). On success, returns the MemberCall
    /// expression on `this` and the result type so the caller can replace the original.
    fn try_resolve_unqualified_method_call(
        &mut self,
        name: &str,
        args: &mut [Expr],
        loc: &SourceLoc,
    ) -> Option<(Expr, Type)> {
        let class_name = self.current_class.clone()?;
        let has_method = self
            .classes
            .get(&class_name)
            .map(|s| s.methods.contains_key(name))
            .unwrap_or(false);
        if !has_method {
            return None;
        }
        let arg_types: Vec<Type> = args.iter_mut().map(|a| self.resolve_expr_type(a)).collect();
        let (sig, mangled) = self.resolve_method_overload(&class_name, name, &arg_types)?;
        let this_ty = Type::Pointer {
            pointee: Box::new(Type::Class {
                name: class_name,
                is_const: self.current_method_is_const,
            }),
            is_const: self.current_method_is_const,
        };
        let mut new_args = Vec::with_capacity(args.len());
        for arg in args.iter_mut() {
            new_args.push(std::mem::take(arg));
        }
        for (arg, expected) in new_args.iter_mut().zip(sig.param_types.iter()) {
            let arg_type = arg.ty().clone();
            if !self.check_assignable(expected, &arg_type, loc) {
                self.report_error(&format!("方法 '{}' 参数类型不匹配", name), loc, ErrorCode::E3038_FuncArgType);
            } else if expected.is_reference() && !arg_type.is_reference() && !arg_type.is_rvalue_ref() {
                let arg_loc = *arg.loc();
                let old = std::mem::take(arg);
                *arg = Expr::Unary {
                    op: UnaryOp::Addr,
                    operand: Box::new(old),
                    loc: arg_loc,
                    ty: expected.clone(),
                };
            } else {
                insert_implicit_cast(arg, expected);
            }
        }
        let ret = sig.ret.clone();
        let new_expr = Expr::MemberCall {
            object: Box::new(Expr::This { loc: *loc, ty: this_ty }),
            method: name.to_string(),
            args: new_args,
            is_virtual: sig.is_virtual,
            resolved_mangled: Some(mangled),
            loc: *loc,
            ty: ret.clone(),
        };
        Some((new_expr, ret))
    }

    pub fn resolve_expr_type(&mut self, expr: &mut Expr) -> Type {
        // Handle offsetof: compute offset at compile time and replace with Literal
        if let Expr::Offsetof { target_type, field, loc, .. } = expr {
            let type_name = target_type.name().to_string();
            let field_name = field.clone();
            let loc_val = *loc;
            let mut offset = 0;
            let mut found = false;
            if let Some(struct_sym) = self.structs.get(&type_name) {
                for (fty, fname) in &struct_sym.fields {
                    if *fname == field_name {
                        found = true;
                        break;
                    }
                    offset += self.compute_type_size(fty);
                }
                if !found {
                    self.report_error(
                        &format!("结构体 '{}' 没有字段 '{}'", type_name, field_name),
                        &loc_val,
                        ErrorCode::E3042_UnknownMember,
                    );
                }
            } else if let Some(union_sym) = self.unions.get(&type_name) {
                // Union: all fields start at offset 0
                if !union_sym.fields.iter().any(|(_, fname)| *fname == field_name) {
                    self.report_error(
                        &format!("联合体 '{}' 没有字段 '{}'", type_name, field_name),
                        &loc_val,
                        ErrorCode::E3042_UnknownMember,
                    );
                }
                offset = 0;
            } else {
                self.report_error(
                    &format!("未知的结构体/联合体类型 '{}'", type_name),
                    &loc_val,
                    ErrorCode::E3004_TypeMismatch,
                );
            }
            *expr = Expr::Literal {
                value: offset,
                loc: loc_val,
                ty: Type::int(),
            };
            return Type::int();
        }

        match expr {
            Expr::Binary { op, left, right, loc, ty } => {
                let left_type = self.resolve_expr_type(left);
                let right_type = self.resolve_expr_type(right);
                *ty = match op {
                    BinaryOp::Add | BinaryOp::Sub => {
                        let left_is_ptrlike = left_type.is_pointer() || left_type.is_array();
                        let right_is_ptrlike = right_type.is_pointer() || right_type.is_array();
                        if self.is_scalar(&left_type) && self.is_scalar(&right_type) {
                            promote_type(&left_type, &right_type)
                        } else if left_is_ptrlike && self.is_int(&right_type) {
                            if left_type.is_array() {
                                Type::pointer_to(left_type.subscript_type())
                            } else {
                                left_type.clone()
                            }
                        } else if self.is_int(&left_type) && right_is_ptrlike && matches!(op, BinaryOp::Add) {
                            if right_type.is_array() {
                                Type::pointer_to(right_type.subscript_type())
                            } else {
                                right_type.clone()
                            }
                        } else if left_is_ptrlike && right_is_ptrlike && matches!(op, BinaryOp::Sub) {
                            Type::int()
                        } else {
                            self.report_error(
                                "算术运算要求两边都是 int 类型，或指针与整数",
                                loc,
                                ErrorCode::E3016_ArithmeticTypeError,
                            );
                            Type::int()
                        }
                    }
                    BinaryOp::Mul | BinaryOp::Div => {
                        if !self.is_scalar(&left_type) || !self.is_scalar(&right_type) {
                            self.report_error(
                                "乘除运算要求两边都是 int 或 float 类型",
                                loc,
                                ErrorCode::E3016_ArithmeticTypeError,
                            );
                        }
                        promote_type(&left_type, &right_type)
                    }
                    BinaryOp::Mod => {
                        if !self.is_int(&left_type) && !matches!(left_type.kind(), TypeKind::LongLong)
                            || !self.is_int(&right_type) && !matches!(right_type.kind(), TypeKind::LongLong)
                        {
                            self.report_error(
                                "取模运算要求两边都是 int 类型",
                                loc,
                                ErrorCode::E3016_ArithmeticTypeError,
                            );
                        }
                        promote_type(&left_type, &right_type)
                    }
                    BinaryOp::Eq | BinaryOp::Ne => {
                        if !self.is_comparable(&left_type, &right_type) {
                            self.report_error("类型不兼容，无法比较", loc, ErrorCode::E3017_ComparisonTypeError);
                        }
                        Type::int()
                    }
                    BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                        let left_is_ptrlike = matches!(left_type.kind(), TypeKind::Pointer | TypeKind::Array);
                        let right_is_ptrlike = matches!(right_type.kind(), TypeKind::Pointer | TypeKind::Array);
                        if !(self.is_scalar(&left_type) && self.is_scalar(&right_type)
                            || left_is_ptrlike && right_is_ptrlike)
                        {
                            self.report_error(
                                "关系运算要求两边都是 int/float 类型或同类型指针",
                                loc,
                                ErrorCode::E3018_RelationTypeError,
                            );
                        }
                        Type::int()
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        let left_ok = self.is_scalar(&left_type) || left_type.is_pointer() || left_type.is_array();
                        let right_ok = self.is_scalar(&right_type) || right_type.is_pointer() || right_type.is_array();
                        if !left_ok || !right_ok {
                            self.report_error(
                                "逻辑运算要求两边都是标量、指针或数组类型",
                                loc,
                                ErrorCode::E3019_LogicTypeError,
                            );
                        }
                        Type::int()
                    }
                    BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => {
                        if !self.is_int(&left_type) || !self.is_int(&right_type) {
                            self.report_error("位运算要求两边都是 int 类型", loc, ErrorCode::E3048_BitOpTypeError);
                        }
                        promote_type(&left_type, &right_type)
                    }
                    BinaryOp::Shl | BinaryOp::Shr => {
                        if !self.is_int(&left_type) || !self.is_int(&right_type) {
                            self.report_error("位运算要求两边都是 int 类型", loc, ErrorCode::E3048_BitOpTypeError);
                        }
                        // Result type is the promoted left operand type (C semantics)
                        if left_type.kind() == TypeKind::Char {
                            Type::Int {
                                is_unsigned: left_type.is_unsigned(),
                                is_const: false,
                            }
                        } else {
                            left_type.clone()
                        }
                    }
                    BinaryOp::Comma => {
                        // Comma operator: evaluate left, discard result, return right's type
                        right_type.clone()
                    }
                };
                ty.clone()
            }
            Expr::Ternary {
                cond,
                then_branch,
                else_branch,
                loc,
                ty,
            } => {
                let cond_type = self.resolve_expr_type(cond);
                if !self.is_scalar(&cond_type) && !matches!(cond_type.kind(), TypeKind::Pointer | TypeKind::Array) {
                    self.report_error(
                        "三目运算符条件必须是 int、float 或指针类型",
                        loc,
                        ErrorCode::E3020_UnaryTypeError,
                    );
                }
                let then_type = self.resolve_expr_type(then_branch);
                let else_type = self.resolve_expr_type(else_branch);
                // 对数组类型执行通常转换：数组在大多数表达式中退化为指向首元素的指针。
                // 例如 `" "`（char[2]）和 `""`（char[1]）在三目运算符中应统一为 char*。
                let decay = |t: &Type| -> Type {
                    if let Type::Array { element, .. } = t {
                        Type::pointer_to(*element.clone())
                    } else {
                        t.clone()
                    }
                };
                let then_decayed = decay(&then_type);
                let else_decayed = decay(&else_type);
                if then_decayed.kind() != else_decayed.kind()
                    || then_decayed.name() != else_decayed.name()
                    || then_decayed != else_decayed
                {
                    self.report_error("三目运算符分支类型不匹配", loc, ErrorCode::E3004_TypeMismatch);
                }
                *ty = then_decayed.clone();
                ty.clone()
            }
            Expr::Unary { op, operand, loc, ty } => {
                let operand_type = self.resolve_expr_type(operand);
                *ty = match op {
                    UnaryOp::Neg => {
                        if !self.is_scalar(&operand_type) {
                            self.report_error(
                                "取负运算要求操作数是 int 或 float 类型",
                                loc,
                                ErrorCode::E3020_UnaryTypeError,
                            );
                        }
                        if operand_type.kind() == TypeKind::Double {
                            Type::double()
                        } else if operand_type.kind() == TypeKind::Float {
                            Type::float()
                        } else if operand_type.kind() == TypeKind::LongLong {
                            Type::long_long()
                        } else if operand_type.is_unsigned() {
                            Type::unsigned_int()
                        } else {
                            Type::int()
                        }
                    }
                    UnaryOp::Not => {
                        if !self.is_scalar(&operand_type) && !operand_type.is_pointer() && !operand_type.is_array() {
                            self.report_error(
                                "逻辑非要求操作数是标量、指针或数组类型",
                                loc,
                                ErrorCode::E3020_UnaryTypeError,
                            );
                        }
                        Type::int()
                    }
                    UnaryOp::BitNot => {
                        if !self.is_int(&operand_type) {
                            self.report_error("按位取反要求操作数是 int 类型", loc, ErrorCode::E3020_UnaryTypeError);
                        }
                        Type::int()
                    }
                    UnaryOp::Addr => Type::pointer_to(operand_type.clone()),
                    UnaryOp::Deref => {
                        if !operand_type.is_pointer() && !operand_type.is_array() {
                            self.report_error("解引用要求指针类型", loc, ErrorCode::E3021_DerefNonPointer);
                            Type::int()
                        } else {
                            match operand_type {
                                Type::Pointer { pointee, .. } | Type::Array { element: pointee, .. } => {
                                    *pointee.clone()
                                }
                                _ => Type::int(),
                            }
                        }
                    }
                    UnaryOp::PreInc | UnaryOp::PreDec | UnaryOp::PostInc | UnaryOp::PostDec => {
                        if !self.is_int(&operand_type)
                            && operand_type.kind() != TypeKind::Float
                            && operand_type.kind() != TypeKind::Double
                            && !operand_type.is_pointer()
                        {
                            self.report_error(
                                "自增/自减要求 int 类型或指针类型",
                                loc,
                                ErrorCode::E3022_IncDecTypeError,
                            );
                        }
                        if let Expr::Identifier { name, .. } = operand.as_ref() {
                            if let Some(sym) = self.lookup_var(name) {
                                if sym.ty.is_const() {
                                    self.report_error(
                                        &format!("不能修改常量变量 '{}'", name),
                                        loc,
                                        ErrorCode::E3049_AssignToConst,
                                    );
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
            Expr::Literal { ty, .. } => {
                if ty.is_unsigned() {
                    ty.clone()
                } else {
                    Type::int()
                }
            }
            Expr::FloatLiteral { .. } => Type::float(),
            Expr::LongLiteral { .. } => Type::long_long(),
            Expr::StringLiteral { value, .. } => {
                let array_size = value.len() as i32 + 1;
                Type::Array {
                    element: Box::new(Type::char()),
                    array_size,
                    dims: vec![array_size],
                    is_const: false,
                    is_vla: false,
                    vla_dims: vec![],
                }
            }
            Expr::Identifier { name, loc, ty } => {
                if let Some(sym) = self.lookup_var(name) {
                    if sym.is_static && sym.is_global {
                        if let Some(files) = self.static_global_files.get(name) {
                            if !files.contains(&self.current_file) {
                                self.report_error(
                                    &format!("static 全局变量 '{}' 在其他文件中不可见", name),
                                    loc,
                                    ErrorCode::E3059_StaticGlobalAccess,
                                );
                            }
                        }
                    }
                    *ty = sym.ty.clone();
                    // Auto-dereference reference types for expression value
                    if let Type::Reference { base, .. } | Type::RValueRef { base, .. } = &sym.ty {
                        *ty = *base.clone();
                    }
                } else if let Some(sym) = self.funcs.get(name).cloned() {
                    // Function name used as value (function pointer)
                    *ty = Type::function_pointer(sym.return_type, sym.param_types);
                } else {
                    // Check if it's a class field (implicit this->field)
                    if let Some(class_name) = self.current_class.clone() {
                        if let Some(class_sym) = self.classes.get(&class_name) {
                            if let Some((field_ty, _, _)) = class_sym.fields.iter().find(|(_, n, _)| n == name) {
                                let this_ty = Type::Pointer {
                                    pointee: Box::new(Type::Class {
                                        name: class_name.clone(),
                                        is_const: self.current_method_is_const,
                                    }),
                                    is_const: self.current_method_is_const,
                                };
                                *expr = Expr::Member {
                                    object: Box::new(Expr::Identifier {
                                        name: "this".to_string(),
                                        ty: this_ty.clone(),
                                        loc: *loc,
                                    }),
                                    member: name.clone(),
                                    ty: field_ty.clone(),
                                    loc: *loc,
                                };
                                return self.resolve_expr_type(expr);
                            }
                        }
                    }
                    self.report_error(&format!("未声明的变量 '{}'", name), loc, ErrorCode::E3023_UndeclaredVar);
                    *ty = Type::int();
                }
                ty.clone()
            }
            Expr::Call { name, args, loc, ty } => {
                // Inside a class member function, an unqualified call may refer to a class
                // method (C++ name hiding). Try method overload resolution first.
                if let Some((new_expr, ret)) = self.try_resolve_unqualified_method_call(name, args, loc) {
                    *expr = new_expr;
                    return ret;
                }
                *ty = self.visit_call(name, args, loc);
                ty.clone()
            }
            Expr::CallPtr { callee, args, loc, ty } => {
                // Direct named function call: identifier is a known function
                if let Expr::Identifier { name, .. } = callee.as_ref() {
                    if self.funcs.contains_key(name)
                        || self.static_func_sigs.contains_key(name)
                        || self.is_builtin_func(name)
                        || name.starts_with("std__")
                    {
                        *ty = self.visit_call(name, args, loc);
                        return ty.clone();
                    }
                    // Inside a class member function, an unqualified call may refer to a class method.
                    if let Some((new_expr, ret)) = self.try_resolve_unqualified_method_call(name, args, loc) {
                        *expr = new_expr;
                        return ret;
                    }
                    // Try template implicit instantiation
                    if !args.is_empty() {
                        let arg_types: Vec<Type> = args.iter_mut().map(|a| self.resolve_expr_type(a)).collect();
                        if let Some((mangled, maybe_new_func)) = self.try_instantiate_template(name, &arg_types) {
                            if let Some(new_func) = maybe_new_func {
                                self.pending_instantiations.push((mangled.clone(), new_func));
                            }
                            // Rewrite CallPtr -> Call so BytecodeGen can resolve by mangled name
                            let new_args = std::mem::take(args);
                            *expr = Expr::Call {
                                name: mangled,
                                args: new_args,
                                loc: *loc,
                                ty: Type::default(),
                            };
                            return self.resolve_expr_type(expr);
                        }
                    }
                    // Lambda call: f(args) -> f.__call(args)
                    if let Some(sym) = self.lookup_var(name) {
                        if let Type::Class { name: class_name, .. } = &sym.ty {
                            if class_name.starts_with("__lambda_") {
                                // Replace current expr and resolve
                                // Need to manually resolve MemberCall here
                                let _obj_ty = &sym.ty;
                                let call_name = format!("{}__call", class_name);
                                let ret_ty = if let Some(func_sym) = self.funcs.get(&call_name).cloned() {
                                    let expected = func_sym.param_types.clone();
                                    if args.len() + 1 != expected.len() {
                                        self.report_error(
                                            &format!(
                                                "Lambda 调用参数数量不匹配：期望 {} 个，实际 {} 个",
                                                expected.len() - 1,
                                                args.len()
                                            ),
                                            loc,
                                            ErrorCode::E3037_FuncArgCount,
                                        );
                                    } else {
                                        for (i, arg) in args.iter_mut().enumerate() {
                                            let arg_ty = self.resolve_expr_type(arg);
                                            let expected_ty = &expected[i + 1];
                                            if !self.check_assignable(expected_ty, &arg_ty, loc) {
                                                self.report_error(
                                                    &format!(
                                                        "Lambda 调用第 {} 个参数类型不匹配：期望 '{}'，实际 '{}'",
                                                        i + 1,
                                                        expected_ty,
                                                        arg_ty
                                                    ),
                                                    loc,
                                                    ErrorCode::E3004_TypeMismatch,
                                                );
                                            } else {
                                                insert_implicit_cast(arg, expected_ty);
                                            }
                                        }
                                    }
                                    // Rewrite CallPtr -> Call with this as first arg
                                    let mut new_args = vec![Expr::Identifier {
                                        name: name.clone(),
                                        ty: sym.ty.clone(),
                                        loc: *loc,
                                    }];
                                    new_args.extend(args.iter().cloned());
                                    *expr = Expr::Call {
                                        name: call_name,
                                        args: new_args,
                                        loc: *loc,
                                        ty: func_sym.return_type.clone(),
                                    };
                                    func_sym.return_type.clone()
                                } else {
                                    self.report_error(
                                        &format!("未找到 Lambda 调用函数 '{}'", call_name),
                                        loc,
                                        ErrorCode::E3036_UndefinedFunc,
                                    );
                                    *ty = Type::int();
                                    Type::int()
                                };
                                return ret_ty;
                            }
                        }
                    }
                }
                let callee_ty = self.resolve_expr_type(callee);
                let func_info = if let Type::Pointer { pointee, .. } = &callee_ty {
                    if let Type::Function { param_types, return_type, .. } = pointee.as_ref() {
                        Some((param_types.clone(), return_type.as_ref().clone()))
                    } else {
                        // Allow calling through generic pointer (with warning)
                        self.report_warning(
                            "通过通用指针调用函数，建议显式转换为函数指针",
                            loc,
                            ErrorCode::W3055_VoidPointerCast,
                        );
                        for arg in args.iter_mut() {
                            self.resolve_expr_type(arg);
                        }
                        *ty = Type::int();
                        None
                    }
                } else if let Type::Function { param_types, return_type, .. } = &callee_ty {
                    // Support (*fp)(args) where callee type is Function directly
                    Some((param_types.clone(), return_type.as_ref().clone()))
                } else {
                    self.report_error("不能对非函数指针类型进行调用", loc, ErrorCode::E3045_CompoundAssignType);
                    for arg in args.iter_mut() {
                        self.resolve_expr_type(arg);
                    }
                    *ty = Type::int();
                    None
                };
                if let Some((param_types, return_type)) = func_info {
                    if args.len() != param_types.len() {
                        self.report_error(
                            &format!("函数指针调用参数数量不匹配：期望 {}，实际 {}", param_types.len(), args.len()),
                            loc,
                            ErrorCode::E3037_FuncArgCount,
                        );
                    } else {
                        for (i, (arg, expected)) in args.iter_mut().zip(param_types.iter()).enumerate() {
                            let arg_type = self.resolve_expr_type(arg);
                            if !self.check_assignable(expected, &arg_type, loc) {
                                self.report_error(
                                    &format!("函数指针调用第 {} 个参数类型不匹配", i + 1),
                                    loc,
                                    ErrorCode::E3038_FuncArgType,
                                );
                            } else {
                                insert_implicit_cast(arg, expected);
                            }
                        }
                    }
                    *ty = return_type;
                }
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
                } else if arr_type.is_array() {
                    *ty = arr_type.subscript_type();
                } else if let Type::Pointer { pointee, .. } = arr_type {
                    *ty = *pointee.clone();
                } else {
                    *ty = Type::int();
                }
                ty.clone()
            }
            Expr::Member { object, member, loc, ty } => {
                let obj_type = self.resolve_expr_type(object);
                let (type_name, kind) = if obj_type.is_struct() {
                    (obj_type.name().to_string(), "struct")
                } else if obj_type.is_union() {
                    (obj_type.name().to_string(), "union")
                } else if obj_type.is_class() {
                    (obj_type.name().to_string(), "class")
                } else if let Type::Pointer { pointee, .. } = &obj_type {
                    if let Type::Struct { name, .. } = pointee.as_ref() {
                        (name.clone(), "struct")
                    } else if let Type::Union { name, .. } = pointee.as_ref() {
                        (name.clone(), "union")
                    } else if let Type::Class { name, .. } = pointee.as_ref() {
                        (name.clone(), "class")
                    } else {
                        self.report_error(
                            "'.' 和 '->' 只能用于结构体、联合体或类类型",
                            loc,
                            ErrorCode::E3041_MemberNonStruct,
                        );
                        *ty = Type::int();
                        return ty.clone();
                    }
                } else if let Type::Reference { base, .. } | Type::RValueRef { base } = &obj_type {
                    if let Type::Struct { name, .. } = base.as_ref() {
                        (name.clone(), "struct")
                    } else if let Type::Union { name, .. } = base.as_ref() {
                        (name.clone(), "union")
                    } else if let Type::Class { name, .. } = base.as_ref() {
                        (name.clone(), "class")
                    } else {
                        self.report_error(
                            "'.' 和 '->' 只能用于结构体、联合体或类类型",
                            loc,
                            ErrorCode::E3041_MemberNonStruct,
                        );
                        *ty = Type::int();
                        return ty.clone();
                    }
                } else {
                    self.report_error(
                        "'.' 和 '->' 只能用于结构体、联合体或类类型",
                        loc,
                        ErrorCode::E3041_MemberNonStruct,
                    );
                    *ty = Type::int();
                    return ty.clone();
                };
                let (field_type, access) = if kind == "union" {
                    (self.get_union_field_type(&type_name, member), None)
                } else if kind == "struct" {
                    (self.get_struct_field_type(&type_name, member), None)
                } else {
                    self.get_class_field_type_with_access(&type_name, member)
                };
                if let Some(ft) = field_type {
                    // Access control check for class members
                    if let Some(acc) = access {
                        if matches!(acc, AccessSpec::Private) && self.current_class.as_ref() != Some(&type_name) {
                            self.report_error(
                                &format!("无法访问类 '{}' 的私有成员 '{}'", type_name, member),
                                loc,
                                ErrorCode::E4024_PrivateMemberAccess,
                            );
                        }
                    }
                    *ty = ft;
                } else {
                    let kind_str = if kind == "union" {
                        "联合体"
                    } else if kind == "struct" {
                        "结构体"
                    } else {
                        "类"
                    };
                    self.report_error(
                        &format!("{} '{}' 没有成员 '{}'", kind_str, type_name, member),
                        loc,
                        ErrorCode::E3042_UnknownMember,
                    );
                    *ty = Type::int();
                }
                ty.clone()
            }
            Expr::Assign { op, left, right, loc, ty } => {
                let right_type = self.resolve_expr_type(right);
                let left_type = self.resolve_expr_type(left);
                let is_lvalue = matches!(
                    left.as_ref(),
                    Expr::Identifier { .. }
                        | Expr::Index { .. }
                        | Expr::Member { .. }
                        | Expr::Unary { op: UnaryOp::Deref, .. }
                ) || (matches!(
                    left.as_ref(),
                    Expr::Call { .. } | Expr::CallPtr { .. } | Expr::MemberCall { .. }
                ) && (left_type.is_reference() || left_type.is_rvalue_ref()));
                if !is_lvalue {
                    self.report_error("赋值左边必须是可修改的左值", loc, ErrorCode::E3043_AssignToRValue);
                }
                if let Expr::Identifier { name, .. } = left.as_ref() {
                    if let Some(sym) = self.lookup_var(name) {
                        if sym.ty.is_const() {
                            self.report_error(
                                &format!("不能给常量变量 '{}' 赋值", name),
                                loc,
                                ErrorCode::E3049_AssignToConst,
                            );
                        }
                    }
                }
                if let Expr::Unary {
                    op: UnaryOp::Deref, operand, ..
                } = left.as_ref()
                {
                    if let Type::Pointer { pointee, .. } = operand.ty() {
                        if pointee.is_const() {
                            self.report_error(
                                "不能通过非 const 指针修改 const 数据",
                                loc,
                                ErrorCode::E3065_ConstViolation,
                            );
                        }
                    }
                }
                if let Expr::Member { object, .. } = left.as_ref() {
                    let obj_ty = object.ty();
                    if let Type::Pointer { pointee, is_const } = &obj_ty {
                        if let Type::Class { is_const: ic, .. } = pointee.as_ref() {
                            if *ic || *is_const {
                                self.report_error("不能修改 const 对象的成员", loc, ErrorCode::E3065_ConstViolation);
                            }
                        }
                    } else if let Type::Class { is_const, .. } = &obj_ty {
                        if *is_const {
                            self.report_error("不能修改 const 对象的成员", loc, ErrorCode::E3065_ConstViolation);
                        }
                    }
                }
                if !self.check_assignable(&left_type, &right_type, loc) {
                    self.report_error(
                        &format!("类型不匹配：无法将 '{}' 赋值给 '{}'", right_type, left_type),
                        loc,
                        ErrorCode::E3044_AssignTypeMismatch,
                    );
                } else {
                    insert_implicit_cast(right, &left_type);
                }
                if *op != AssignOp::Assign && (!self.is_scalar(&left_type) || !self.is_scalar(&right_type)) {
                    self.report_error(
                        "复合赋值要求两边都是标量类型（int、float、double 或 long long）",
                        loc,
                        ErrorCode::E3045_CompoundAssignType,
                    );
                }
                *ty = left_type.clone();
                ty.clone()
            }
            Expr::Sizeof { operand, ty, loc, .. } => {
                if let Some(ref mut op) = operand {
                    self.resolve_expr_type(op);
                    if let Expr::Identifier { name, .. } = &**op {
                        if self.current_func_params.contains(name) {
                            if let Some(sym) = self.lookup_var(name) {
                                if sym.ty.is_pointer() {
                                    self.report_warning(
                                        "数组参数已退化为指针，sizeof 结果为指针大小，而非数组总大小。",
                                        loc,
                                        ErrorCode::W3052_ArrayToPointerDecay,
                                    );
                                }
                            }
                        }
                    }
                }
                *ty = Type::int();
                ty.clone()
            }
            Expr::Cast { expr, target_type, ty, loc, .. } => {
                let expr_ty = self.resolve_expr_type(expr);
                *ty = target_type.clone();
                // Warn on pointer <-> double cast (implementation-defined behavior)
                if (target_type.is_pointer() && matches!(expr_ty.kind(), TypeKind::Float | TypeKind::Double))
                    || (expr_ty.is_pointer() && matches!(target_type.kind(), TypeKind::Float | TypeKind::Double))
                {
                    self.report_warning(
                        &format!("将 '{}' 转换为 '{}' 是实现定义的行为，结果可能不可移植", expr_ty, target_type),
                        loc,
                        ErrorCode::W3064_DoublePointerCast,
                    );
                }
                ty.clone()
            }
            Expr::InitList { elements, ty, .. } => {
                for elem in elements.iter_mut() {
                    self.resolve_expr_type(&mut elem.value);
                }
                *ty = Type::void();
                ty.clone()
            }
            Expr::Offsetof { .. } => {
                // Should have been replaced by Literal above; unreachable
                Type::int()
            }
            Expr::This { loc, ty } => {
                if let Some(ref class_name) = self.current_class {
                    *ty = Type::Pointer {
                        pointee: Box::new(Type::Class {
                            name: class_name.clone(),
                            is_const: self.current_method_is_const,
                        }),
                        is_const: self.current_method_is_const,
                    };
                } else {
                    self.report_error("'this' 只能在类成员函数中使用", loc, ErrorCode::E4023_ThisOutsideClass);
                    *ty = Type::int();
                }
                ty.clone()
            }
            Expr::MemberCall {
                object,
                method,
                args,
                is_virtual,
                resolved_mangled,
                loc,
                ty,
            } => {
                let obj_type = self.resolve_expr_type(object);
                // Resolve class type from object (value, pointer, or reference)
                let class_name = if let Type::Class { name, .. } = &obj_type {
                    name.clone()
                } else if let Type::Pointer { pointee, .. } = &obj_type {
                    if let Type::Class { name, .. } = pointee.as_ref() {
                        name.clone()
                    } else {
                        self.report_error("成员调用只能用于类类型", loc, ErrorCode::E3041_MemberNonStruct);
                        *ty = Type::int();
                        return ty.clone();
                    }
                } else if let Type::Reference { base, .. } | Type::RValueRef { base } = &obj_type {
                    if let Type::Class { name, .. } = base.as_ref() {
                        name.clone()
                    } else {
                        self.report_error("成员调用只能用于类类型", loc, ErrorCode::E3041_MemberNonStruct);
                        *ty = Type::int();
                        return ty.clone();
                    }
                } else {
                    self.report_error("成员调用只能用于类类型", loc, ErrorCode::E3041_MemberNonStruct);
                    *ty = Type::int();
                    return ty.clone();
                };

                // Builtin container member calls are lowered to host function calls.
                if let Some((host_func, addr_expr, call_args, result_ty)) =
                    self.try_resolve_container_member_call(&class_name, method, object, args, loc)
                {
                    *is_virtual = false;
                    let mut full_args = vec![addr_expr];
                    full_args.extend(call_args);
                    let ret = result_ty.clone();
                    *expr = Expr::Call {
                        name: host_func,
                        args: full_args,
                        loc: *loc,
                        ty: result_ty,
                    };
                    return ret;
                }

                // Resolve user argument types first for overload resolution.
                let arg_types: Vec<Type> = args.iter_mut().map(|a| self.resolve_expr_type(a)).collect();
                match self.resolve_method_overload(&class_name, method, &arg_types) {
                    None => {
                        self.report_error(
                            &format!("类 '{}' 没有与参数匹配的方法 '{}'", class_name, method),
                            loc,
                            ErrorCode::E3042_UnknownMember,
                        );
                        *ty = Type::int();
                        ty.clone()
                    }
                    Some((sig, mangled)) => {
                        // Check access control (simplified: allow public, block private from outside)
                        if matches!(sig.access, AccessSpec::Private) && self.current_class.as_ref() != Some(&class_name)
                        {
                            self.report_error(
                                &format!("无法访问类 '{}' 的私有成员 '{}'", class_name, method),
                                loc,
                                ErrorCode::E4024_PrivateMemberAccess,
                            );
                        }
                        // Check const-correctness: const object cannot call non-const method
                        if !sig.is_const && !sig.is_static {
                            let obj_is_const = match &obj_type {
                                Type::Class { is_const, .. } => *is_const,
                                Type::Pointer { pointee, is_const } => {
                                    if let Type::Class { is_const: ic, .. } = pointee.as_ref() {
                                        *ic || *is_const
                                    } else {
                                        *is_const
                                    }
                                }
                                Type::Reference { base, is_const } => {
                                    if let Type::Class { is_const: ic, .. } = base.as_ref() {
                                        *ic || *is_const
                                    } else {
                                        *is_const
                                    }
                                }
                                _ => false,
                            };
                            if obj_is_const {
                                self.report_error(
                                    &format!("不能在 const 对象上调用非 const 方法 '{}'", method),
                                    loc,
                                    ErrorCode::E3065_ConstViolation,
                                );
                            }
                        }
                        // Apply implicit conversions / reference address-of
                        for (arg, expected) in args.iter_mut().zip(sig.param_types.iter()) {
                            let arg_type = arg.ty().clone();
                            if !self.check_assignable(expected, &arg_type, loc) {
                                self.report_error(
                                    &format!("方法 '{}' 参数类型不匹配", method),
                                    loc,
                                    ErrorCode::E3038_FuncArgType,
                                );
                            } else if expected.is_reference() && !arg_type.is_reference() && !arg_type.is_rvalue_ref() {
                                let arg_loc = *arg.loc();
                                let old = std::mem::take(arg);
                                *arg = Expr::Unary {
                                    op: UnaryOp::Addr,
                                    operand: Box::new(old),
                                    loc: arg_loc,
                                    ty: expected.clone(),
                                };
                            } else {
                                insert_implicit_cast(arg, expected);
                            }
                        }
                        *ty = sig.ret.clone();
                        *is_virtual = sig.is_virtual;
                        *resolved_mangled = Some(mangled);
                        ty.clone()
                    }
                }
            }
            Expr::New {
                elem_type,
                size_expr,
                init,
                loc,
                ty,
            } => {
                // Class template instantiation for new
                *elem_type = self.resolve_template_id(elem_type, loc);
                if let Some(ref mut se) = size_expr {
                    let size_ty = self.resolve_expr_type(se);
                    if !self.is_int(&size_ty) {
                        self.report_error("new[] 的大小必须是 int 类型", loc, ErrorCode::E4027_InvalidNewType);
                    }
                }
                if let Some(ref mut i) = init {
                    // C++ 类类型 new：将占位符 __ctor__* 解析为具体 mangled 构造函数名
                    if matches!(elem_type, Type::Class { .. }) {
                        let mut ctor_class_name = String::new();
                        let mut ctor_name = String::new();
                        if let Expr::Call { name, args, .. } = i.as_mut() {
                            if name.starts_with("__ctor__") {
                                if let Type::Class { name: class_name, .. } = elem_type {
                                    let arg_count = args.len();
                                    if let Some(resolved) =
                                        self.resolve_constructor_overload(class_name, arg_count, *loc)
                                    {
                                        *name = resolved.clone();
                                        ctor_class_name = class_name.clone();
                                        ctor_name = resolved;
                                    } else {
                                        self.report_error(
                                            &format!("类 '{}' 没有接受 {} 个参数的构造函数", class_name, arg_count),
                                            loc,
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
                                if let Expr::Call { args, .. } = i.as_mut() {
                                    if expected.len() != args.len() {
                                        self.report_error(
                                            &format!(
                                                "构造函数 '{}' 参数数量不匹配：期望 {}，实际 {}",
                                                ctor_name,
                                                expected.len(),
                                                args.len()
                                            ),
                                            loc,
                                            ErrorCode::E3037_FuncArgCount,
                                        );
                                    } else {
                                        for (arg, exp) in args.iter_mut().zip(expected.iter()) {
                                            let arg_ty = self.resolve_expr_type(arg);
                                            if !self.check_assignable(exp, &arg_ty, loc) {
                                                self.report_error(
                                                    &format!(
                                                        "构造函数参数类型不匹配：期望 '{}'，实际 '{}'",
                                                        exp, arg_ty
                                                    ),
                                                    loc,
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
                        if !self.check_assignable(elem_type, &init_ty, loc) {
                            self.report_error(
                                &format!("new 的初始化类型不匹配：期望 '{}'，实际 '{}'", elem_type, init_ty),
                                loc,
                                ErrorCode::E4027_InvalidNewType,
                            );
                        }
                    }
                }
                *ty = Type::Pointer {
                    pointee: Box::new(elem_type.clone()),
                    is_const: false,
                };
                ty.clone()
            }
            Expr::Delete { expr, is_array: _, loc, ty } => {
                let expr_ty = self.resolve_expr_type(expr);
                if !expr_ty.is_pointer() {
                    self.report_error("delete 只能用于指针类型", loc, ErrorCode::E4028_InvalidDeleteType);
                }
                *ty = Type::void();
                ty.clone()
            }
            Expr::Move { expr, loc: _, ty } => {
                let expr_ty = self.resolve_expr_type(expr);
                *ty = Type::RValueRef { base: Box::new(expr_ty) };
                ty.clone()
            }
            Expr::Lambda {
                params,
                body,
                unique_id,
                loc,
                ty,
                capture,
            } => {
                // Generate a closure struct type
                let lambda_name = format!("__lambda_{}", unique_id);
                *ty = Type::Class {
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
                            pointee: Box::new(ty.clone()),
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
                ty.clone()
            }
        }
    }
}
