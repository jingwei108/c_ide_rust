use super::*;

impl TypeChecker {
    pub fn resolve_expr_type(&mut self, expr: &mut Expr) -> Type {
        match expr {
            Expr::Binary { op, left, right, loc, ty } => {
                let left_type = self.resolve_expr_type(left);
                let right_type = self.resolve_expr_type(right);
                *ty = match op {
                    BinaryOp::Add | BinaryOp::Sub => {
                        if self.is_scalar(&left_type) && self.is_scalar(&right_type) {
                            if left_type.kind() == TypeKind::Double || right_type.kind() == TypeKind::Double { Type::double() }
                            else if left_type.kind() == TypeKind::Float || right_type.kind() == TypeKind::Float { Type::float() }
                            else if left_type.kind() == TypeKind::LongLong || right_type.kind() == TypeKind::LongLong { Type::long_long() }
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
                        if left_type.kind() == TypeKind::Double || right_type.kind() == TypeKind::Double { Type::double() }
                        else if left_type.kind() == TypeKind::Float || right_type.kind() == TypeKind::Float { Type::float() }
                        else if left_type.kind() == TypeKind::LongLong || right_type.kind() == TypeKind::LongLong { Type::long_long() }
                        else { Type::int() }
                    }
                    BinaryOp::Mod => {
                        if !self.is_int(&left_type) && !matches!(left_type.kind(), TypeKind::LongLong) || !self.is_int(&right_type) && !matches!(right_type.kind(), TypeKind::LongLong) {
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
                        let left_is_ptrlike = matches!(left_type.kind(), TypeKind::Pointer | TypeKind::Array);
                        let right_is_ptrlike = matches!(right_type.kind(), TypeKind::Pointer | TypeKind::Array);
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
                if !self.is_scalar(&cond_type) && !matches!(cond_type.kind(), TypeKind::Pointer | TypeKind::Array) {
                    self.report_error("三目运算符条件必须是 int、float 或指针类型", loc, ErrorCode::E3020_UnaryTypeError);
                }
                let then_type = self.resolve_expr_type(then_branch);
                let else_type = self.resolve_expr_type(else_branch);
                if then_type.kind() != else_type.kind() || then_type.name() != else_type.name() || then_type != else_type {
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
                        if operand_type.kind() == TypeKind::Double { Type::double() }
                        else if operand_type.kind() == TypeKind::Float { Type::float() }
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
                        Type::pointer_to(operand_type.clone())
                    }
                    UnaryOp::Deref => {
                        if !operand_type.is_pointer() && !operand_type.is_array() {
                            self.report_error("解引用要求指针类型", loc, ErrorCode::E3021_DerefNonPointer);
                            Type::int()
                        } else {
                            match operand_type {
                                Type::Pointer { pointee, .. } | Type::Array { element: pointee, .. } => *pointee.clone(),
                                _ => Type::int(),
                            }
                        }
                    }
                    UnaryOp::PreInc | UnaryOp::PreDec | UnaryOp::PostInc | UnaryOp::PostDec => {
                        if !self.is_int(&operand_type) && operand_type.kind() != TypeKind::Float && operand_type.kind() != TypeKind::Double && !operand_type.is_pointer() {
                            self.report_error("自增/自减要求 int 类型或指针类型", loc, ErrorCode::E3022_IncDecTypeError);
                        }
                        if let Expr::Identifier { name, .. } = operand.as_ref() {
                            if let Some(sym) = self.lookup_var(name) {
                                if sym.ty.is_const() {
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
            Expr::StringLiteral { value, .. } => {
                let array_size = value.len() as i32 + 1;
                Type::Array { element: Box::new(Type::char()), array_size, dims: vec![array_size], is_const: false }
            }
            Expr::Identifier { name, loc, ty } => {
                if let Some(sym) = self.lookup_var(name) {
                    *ty = sym.ty;
                } else if let Some(sym) = self.funcs.get(name).cloned() {
                    // Function name used as value (function pointer)
                    *ty = Type::function_pointer(sym.return_type, sym.param_types);
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
            Expr::CallPtr { callee, args, loc, ty } => {
                // Direct named function call: identifier is a known function
                if let Expr::Identifier { name, .. } = callee.as_ref() {
                    if self.funcs.contains_key(name) || self.static_func_sigs.contains_key(name) || self.is_builtin_func(name) {
                        *ty = self.visit_call(name, args, loc);
                        return ty.clone();
                    }
                }
                let callee_ty = self.resolve_expr_type(callee);
                if let Type::Pointer { pointee, .. } = &callee_ty {
                    if let Type::Function { param_types, return_type, .. } = pointee.as_ref() {
                        if args.len() != param_types.len() {
                            self.report_error(&format!("函数指针调用参数数量不匹配：期望 {}，实际 {}", param_types.len(), args.len()), loc, ErrorCode::E3037_FuncArgCount);
                        } else {
                            for (i, (arg, expected)) in args.iter_mut().zip(param_types.iter()).enumerate() {
                                let arg_type = self.resolve_expr_type(arg);
                                if !self.check_assignable(expected, &arg_type, loc) {
                                    self.report_error(&format!("函数指针调用第 {} 个参数类型不匹配", i + 1), loc, ErrorCode::E3038_FuncArgType);
                                } else {
                                    insert_implicit_cast(arg, expected);
                                }
                            }
                        }
                        *ty = return_type.as_ref().clone();
                    } else {
                        // Allow calling through generic pointer (with warning)
                        self.report_warning("通过通用指针调用函数，建议显式转换为函数指针", loc, ErrorCode::W3055_VoidPointerCast);
                        for arg in args.iter_mut() { self.resolve_expr_type(arg); }
                        *ty = Type::int();
                    }
                } else {
                    self.report_error("不能对非函数指针类型进行调用", loc, ErrorCode::E3045_CompoundAssignType);
                    for arg in args.iter_mut() { self.resolve_expr_type(arg); }
                    *ty = Type::int();
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
                let (type_name, is_union) = if obj_type.is_struct() {
                    (obj_type.name().to_string(), false)
                } else if obj_type.is_union() {
                    (obj_type.name().to_string(), true)
                } else if let Type::Pointer { pointee, .. } = &obj_type {
                    if let Type::Struct { name, .. } = pointee.as_ref() {
                        (name.clone(), false)
                    } else if let Type::Union { name, .. } = pointee.as_ref() {
                        (name.clone(), true)
                    } else {
                        self.report_error("'.' 和 '->' 只能用于结构体或联合体类型", loc, ErrorCode::E3041_MemberNonStruct);
                        *ty = Type::int();
                        return ty.clone();
                    }
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
                        if sym.ty.is_const() {
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
                    self.report_error("复合赋值要求两边都是标量类型（int、float、double 或 long long）", loc, ErrorCode::E3045_CompoundAssignType);
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
                    self.resolve_expr_type(elem);
                }
                *ty = Type::void();
                ty.clone()
            }
        }
    }
}
