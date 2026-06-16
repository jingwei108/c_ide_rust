use super::*;

mod literal;
mod binary;
mod unary;
mod call;
mod array;

/// Returns true if `expr` denotes an object with storage (an lvalue in C++ terms).
pub(crate) fn is_lvalue_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Identifier { .. }
            | Expr::Index { .. }
            | Expr::Member { .. }
            | Expr::Unary { op: UnaryOp::Deref, .. }
            | Expr::This { .. }
    )
}

pub(crate) trait ExprGen {
    fn gen_expr(&mut self, expr: &mut Expr);
    fn gen_nested_init(&mut self, base_temp: i32, offset: i32, target_ty: &Type, init: &mut Expr, loc: &SourceLoc);
    fn gen_member_addr(&mut self, object: &mut Expr, member: &str, loc: &SourceLoc);
    fn gen_index(&mut self, array: &mut Expr, index: &mut Expr, result_ty: &Type, loc: &SourceLoc, is_assign: bool);
    fn gen_vla_stride(&mut self, arr_type: &Type, loc: &SourceLoc);
    fn gen_expr_with_cast(&mut self, expr: &mut Expr, target_is_fp: bool, target_is_double: bool, loc: &SourceLoc);
    fn gen_addr(&mut self, expr: &mut Expr, loc: &SourceLoc);
    fn gen_struct_copy_common<F: FnMut(&mut Self, &SourceLoc, i32)>(
        &mut self,
        size: i32,
        src_expr: &mut Expr,
        dst_emit: F,
        loc: &SourceLoc,
    );
    fn gen_struct_copy(&mut self, left: &mut Expr, right: &mut Expr, loc: &SourceLoc);
    fn gen_struct_copy_to_local(&mut self, local_offset: i32, right: &mut Expr, loc: &SourceLoc);
    fn gen_assign(&mut self, op: &AssignOp, left: &mut Expr, right: &mut Expr, loc: &SourceLoc);
}

impl ExprGen for BytecodeGen {
    fn gen_expr(&mut self, expr: &mut Expr) {
        let loc = *expr.loc();
        match expr {
            Expr::Literal { value, .. } => literal::gen_literal(self, *value, &loc),
            Expr::FloatLiteral { value, ty, .. } => literal::gen_float_literal(self, *value, ty, &loc),
            Expr::LongLiteral { value, .. } => literal::gen_long_literal(self, *value, &loc),
            Expr::StringLiteral { value, .. } => literal::gen_string_literal(self, value, &loc),
            Expr::Identifier { name, .. } => {
                // Function name used as value (function pointer)
                if let Some(&idx) = self.func_index.get(name) {
                    self.emit(OpCode::PushConst, idx, &loc);
                    return;
                }
                // C++ reference auto-dereference
                let base_ty = self
                    .local_types
                    .get(name)
                    .or_else(|| self.global_types.get(name))
                    .or_else(|| self.static_local_types.get(name))
                    .and_then(|t| t.reference_base().cloned());
                if let Some(base_ty) = base_ty {
                    self.gen_addr(expr, &loc);
                    match base_ty.kind() {
                        TypeKind::Char => self.emit(OpCode::LoadMemByte, 0, &loc),
                        TypeKind::Double => self.emit(OpCode::LoadMemD, 0, &loc),
                        TypeKind::LongLong => self.emit(OpCode::LoadMemQ, 0, &loc),
                        _ => self.emit(OpCode::LoadMem, 0, &loc),
                    }
                    return;
                }
                // static local variable
                if let Some(&static_offset) = self.static_local_indices.get(name) {
                    if let Some(ty) = self.static_local_types.get(name) {
                        if ty.is_array() {
                            self.emit(OpCode::PushConst, crate::vm::core::GLOBAL_START as i32 + static_offset, &loc);
                        } else if ty.kind() == TypeKind::Double {
                            self.emit(OpCode::LoadGlobalD, static_offset, &loc);
                        } else if ty.kind() == TypeKind::LongLong {
                            self.emit(OpCode::LoadGlobalQ, static_offset, &loc);
                        } else {
                            self.emit(OpCode::LoadGlobal, static_offset, &loc);
                        }
                    } else {
                        self.emit(OpCode::LoadGlobal, static_offset, &loc);
                    }
                    return;
                }
                let local_offset = self.resolve_local(name);
                if local_offset >= 0 {
                    if let Some(ty) = self.local_types.get(name) {
                        if ty.is_array() {
                            if ty.is_vla() {
                                // VLA is stored as a pointer on stack
                                self.emit(OpCode::LoadLocal, local_offset, &loc);
                            } else if local_offset < self.current_func_arg_bytes {
                                // Array parameter decayed to pointer
                                self.emit(OpCode::LoadLocal, local_offset, &loc);
                            } else {
                                // Local array: compute base address
                                self.emit(OpCode::GetFrameBase, 0, &loc);
                                self.emit(OpCode::PushConst, local_offset, &loc);
                                self.emit(OpCode::Add, 0, &loc);
                            }
                        } else if ty.kind() == TypeKind::Double {
                            self.emit(OpCode::LoadLocalD, local_offset, &loc);
                        } else if ty.kind() == TypeKind::LongLong {
                            self.emit(OpCode::LoadLocalQ, local_offset, &loc);
                        } else {
                            self.emit(OpCode::LoadLocal, local_offset, &loc);
                        }
                    } else {
                        self.emit(OpCode::LoadLocal, local_offset, &loc);
                    }
                } else {
                    let global_offset = self.resolve_global(name);
                    if global_offset >= 0 {
                        if let Some(ty) = self.global_types.get(name) {
                            if ty.is_array() {
                                self.emit(
                                    OpCode::PushConst,
                                    crate::vm::core::GLOBAL_START as i32 + global_offset,
                                    &loc,
                                );
                            } else if ty.kind() == TypeKind::Double {
                                self.emit(OpCode::LoadGlobalD, global_offset, &loc);
                            } else if ty.kind() == TypeKind::LongLong {
                                self.emit(OpCode::LoadGlobalQ, global_offset, &loc);
                            } else {
                                self.emit(OpCode::LoadGlobal, global_offset, &loc);
                            }
                        } else {
                            self.emit(OpCode::LoadGlobal, global_offset, &loc);
                        }
                    } else {
                        self.report_error(&format!("未声明的标识符 '{}'", name), &loc);
                        self.emit(OpCode::PushConst, 0, &loc);
                    }
                }
            }
            Expr::Binary { op, left, right, ty, .. } => binary::gen_binary(self, op, left, right, ty, &loc),
            Expr::Unary { .. } => unary::gen_unary(self, expr),
            Expr::Call { .. } => call::gen_call(self, expr),
            Expr::CallPtr { .. } => call::gen_call_ptr(self, expr),
            Expr::Index { .. } => array::gen_index_expr(self, expr),
            Expr::Member { object, member, ty, .. } => {
                self.gen_member_addr(object, member, &loc);
                // Lambda by-reference capture: need to load the captured pointer first
                if object.ty().is_pointer() {
                    if let Type::Pointer { pointee, .. } = object.ty() {
                        if let Type::Class { name, .. } = pointee.as_ref() {
                            if let Some(by_ref_fields) = self.lambda_by_ref_fields.get(name) {
                                if by_ref_fields.contains(member) {
                                    self.emit(OpCode::LoadMem, 0, &loc);
                                }
                            }
                        }
                    }
                }
                if !ty.is_array() {
                    if ty.kind() == TypeKind::Char {
                        self.emit(OpCode::LoadMemByte, 0, &loc);
                    } else if ty.kind() == TypeKind::Double {
                        self.emit(OpCode::LoadMemD, 0, &loc);
                    } else if ty.kind() == TypeKind::LongLong {
                        self.emit(OpCode::LoadMemQ, 0, &loc);
                    } else {
                        self.emit(OpCode::LoadMem, 0, &loc);
                    }
                }
            }
            Expr::Ternary {
                cond, then_branch, else_branch, ..
            } => {
                self.gen_expr(cond);
                let else_jump = self.current_ip();
                self.emit(OpCode::JumpIfZero, 0, &loc);
                self.gen_expr(then_branch);
                let end_jump = self.current_ip();
                self.emit(OpCode::Jump, 0, &loc);
                let else_ip = self.current_ip();
                self.patch_jump(else_jump, else_ip);
                self.gen_expr(else_branch);
                let end_ip = self.current_ip();
                self.patch_jump(end_jump, end_ip);
            }
            Expr::Assign { op, left, right, .. } => {
                self.gen_assign(op, left, right, &loc);
            }
            Expr::Sizeof { target_type, operand, .. } => {
                let is_vla = target_type.as_ref().map(|t| t.is_vla()).unwrap_or(false)
                    || operand.as_ref().map(|op| op.ty().is_vla()).unwrap_or(false);
                if is_vla {
                    let array_info = if let Some(ref op) = operand {
                        if let Type::Array { dims, vla_dims, .. } = op.ty() {
                            Some((dims.clone(), vla_dims.clone(), self.elem_type_size(op.ty())))
                        } else {
                            None
                        }
                    } else if let Some(ref t) = target_type {
                        if let Type::Array { dims, vla_dims, .. } = t {
                            Some((dims.clone(), vla_dims.clone(), self.elem_type_size(t)))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Some((dims, mut vla_dims, elem_size)) = array_info {
                        if dims.is_empty() {
                            self.emit(OpCode::PushConst, 0, &loc);
                        } else {
                            let mut vla_idx = 0;
                            for &dim in dims.iter() {
                                if dim > 0 {
                                    self.emit(OpCode::PushConst, dim, &loc);
                                } else if let Some(dim_expr) = vla_dims.get_mut(vla_idx) {
                                    self.gen_expr(dim_expr);
                                    vla_idx += 1;
                                } else {
                                    self.emit(OpCode::PushConst, 0, &loc);
                                }
                            }
                            for _ in 1..dims.len() {
                                self.emit(OpCode::Mul, 0, &loc);
                            }
                            if elem_size > 1 {
                                self.emit(OpCode::PushConst, elem_size, &loc);
                                self.emit(OpCode::Mul, 0, &loc);
                            }
                        }
                    } else {
                        self.emit(OpCode::PushConst, 4, &loc);
                    }
                } else {
                    let size = if let Some(ref t) = target_type {
                        self.type_size(t)
                    } else if let Some(ref op) = operand {
                        self.type_size(op.ty())
                    } else {
                        0
                    };
                    self.emit(OpCode::PushConst, size, &loc);
                }
            }
            Expr::Cast { expr, target_type, .. } => {
                self.gen_expr(expr);
                if target_type.kind() == TypeKind::Double
                    && expr.ty().kind() != TypeKind::Float
                    && expr.ty().kind() != TypeKind::Double
                    && expr.ty().kind() != TypeKind::LongLong
                {
                    self.emit(OpCode::CastI2D, 0, &loc);
                } else if target_type.kind() == TypeKind::Double && expr.ty().kind() == TypeKind::Float {
                    self.emit(OpCode::CastF2D, 0, &loc);
                } else if target_type.kind() == TypeKind::Double && expr.ty().kind() == TypeKind::LongLong {
                    self.emit(OpCode::CastQ2D, 0, &loc);
                } else if target_type.kind() == TypeKind::Float
                    && expr.ty().kind() != TypeKind::Float
                    && expr.ty().kind() != TypeKind::Double
                    && expr.ty().kind() != TypeKind::LongLong
                {
                    self.emit(OpCode::CastI2F, 0, &loc);
                } else if target_type.kind() == TypeKind::Float && expr.ty().kind() == TypeKind::Double {
                    self.emit(OpCode::CastD2F, 0, &loc);
                } else if target_type.kind() == TypeKind::LongLong
                    && expr.ty().kind() != TypeKind::LongLong
                    && expr.ty().kind() != TypeKind::Double
                    && expr.ty().kind() != TypeKind::Float
                {
                    self.emit(OpCode::CastI2Q, 0, &loc);
                } else if target_type.kind() == TypeKind::LongLong && expr.ty().kind() == TypeKind::Double {
                    self.emit(OpCode::CastD2Q, 0, &loc);
                } else if target_type.kind() != TypeKind::Float
                    && target_type.kind() != TypeKind::Double
                    && target_type.kind() != TypeKind::LongLong
                    && expr.ty().kind() == TypeKind::Double
                {
                    self.emit(OpCode::CastD2I, 0, &loc);
                } else if target_type.kind() != TypeKind::Float
                    && target_type.kind() != TypeKind::Double
                    && target_type.kind() != TypeKind::LongLong
                    && expr.ty().kind() == TypeKind::Float
                {
                    self.emit(OpCode::CastF2I, 0, &loc);
                } else if target_type.kind() != TypeKind::Float
                    && target_type.kind() != TypeKind::Double
                    && target_type.kind() != TypeKind::LongLong
                    && expr.ty().kind() == TypeKind::LongLong
                {
                    self.emit(OpCode::CastQ2I, 0, &loc);
                }
            }
            Expr::InitList { .. } => {
                self.report_error("初始化列表只能在变量声明中使用", &loc);
                self.emit(OpCode::PushConst, 0, &loc);
            }
            Expr::Offsetof { target_type, field, .. } => {
                let mut offset = 0;
                let mut found = false;
                if let Some(fields) = self.struct_defs.get(target_type.name()) {
                    for f in fields {
                        if f.name == *field {
                            found = true;
                            break;
                        }
                        offset += self.type_size(&f.ty);
                    }
                } else if let Some(fields) = self.union_defs.get(target_type.name()) {
                    if fields.iter().any(|f| f.name == *field) {
                        offset = 0;
                        found = true;
                    }
                }
                if !found {
                    self.report_error(
                        &format!("offsetof: 未知的结构体/联合体 '{}' 或字段 '{}'", target_type.name(), field),
                        &loc,
                    );
                }
                self.emit(OpCode::PushConst, offset, &loc);
            }
            // === C++ 新增 (Phase 33) ===
            Expr::This { .. } => self.gen_this(expr, &loc),
            Expr::MemberCall { .. } => self.gen_member_call(expr, &loc),
            Expr::New { .. } => self.gen_new(expr, &loc),
            Expr::Delete { .. } => self.gen_delete(expr, &loc),
            Expr::Move { .. } => self.gen_move(expr, &loc),
            Expr::Lambda { .. } => self.gen_lambda(expr, &loc),
        }
    }

    /// Generate initialization code for a nested InitList at a base address stored in a local temp slot.
    fn gen_nested_init(&mut self, base_temp: i32, offset: i32, target_ty: &Type, init: &mut Expr, loc: &SourceLoc) {
        match init {
            Expr::InitList { elements, .. } => {
                if target_ty.is_struct() || target_ty.is_class() {
                    let fields = if target_ty.is_struct() {
                        self.struct_defs.get(target_ty.name()).cloned().unwrap_or_default()
                    } else {
                        self.class_defs
                            .get(target_ty.name())
                            .map(|c| {
                                c.members
                                    .iter()
                                    .filter_map(|m| match m {
                                        ClassMember::Field { name, ty, .. } => Some(StructField {
                                            name: name.clone(),
                                            ty: ty.clone(),
                                        }),
                                        _ => None,
                                    })
                                    .collect()
                            })
                            .unwrap_or_default()
                    };
                    for (i, elem) in elements.iter_mut().enumerate() {
                        if i >= fields.len() {
                            break;
                        }
                        let field_offset = fields.iter().take(i).map(|f| self.type_size(&f.ty)).sum::<i32>();
                        self.gen_nested_init(base_temp, offset + field_offset, &fields[i].ty, &mut elem.value, loc);
                    }
                } else if target_ty.is_array() {
                    let elem_size = self.elem_type_size(target_ty);
                    let inner_ty = target_ty.subscript_type();
                    for (i, elem) in elements.iter_mut().enumerate() {
                        let elem_offset = offset + (i as i32) * elem_size;
                        self.gen_nested_init(base_temp, elem_offset, &inner_ty, &mut elem.value, loc);
                    }
                } else {
                    if let Some(first) = elements.first_mut() {
                        self.gen_nested_init(base_temp, offset, target_ty, &mut first.value, loc);
                    }
                }
            }
            _ => {
                self.emit(OpCode::LoadLocal, base_temp, loc);
                if offset != 0 {
                    self.emit(OpCode::PushConst, offset, loc);
                    self.emit(OpCode::Add, 0, loc);
                }
                self.gen_expr(init);
                match target_ty.kind() {
                    TypeKind::Double => self.emit(OpCode::StoreMemD, 0, loc),
                    TypeKind::LongLong => self.emit(OpCode::StoreMemQ, 0, loc),
                    TypeKind::Char => self.emit(OpCode::StoreMemByte, 0, loc),
                    _ => self.emit(OpCode::StoreMem, 0, loc),
                }
            }
        }
    }

    fn gen_member_addr(&mut self, object: &mut Expr, member: &str, loc: &SourceLoc) {
        if object.ty().is_pointer() {
            self.gen_expr(object);
        } else if object.ty().is_reference() || object.ty().is_rvalue_ref() {
            // Reference/RValueRef: gen_addr yields the address it stores,
            // which is the object address we need for member access.
            self.gen_addr(object, loc);
        } else if let Expr::Index { array, index, ty, .. } = object {
            self.gen_index(array, index, ty, loc, true);
        } else if let Expr::Member { object: inner, member: m, .. } = object {
            self.gen_member_addr(inner, m, loc);
        } else if let Expr::Identifier { name, .. } = object {
            // TypeChecker 会对引用变量做 auto-dereference，导致 object.ty() 变成 Class。
            // 这里检查实际变量类型，如果是引用则加载其存储的地址。
            let actual_ty = self
                .local_types
                .get(name)
                .or_else(|| self.global_types.get(name))
                .or_else(|| self.static_local_types.get(name));
            if actual_ty.map(|t| t.is_reference() || t.is_rvalue_ref()).unwrap_or(false) {
                self.gen_addr(object, loc);
            } else if let Some(&offset) = self.local_indices.get(name) {
                self.emit(OpCode::GetFrameBase, 0, loc);
                self.emit(OpCode::PushConst, offset, loc);
                self.emit(OpCode::Add, 0, loc);
            } else if let Some(&offset) = self.global_indices.get(name) {
                self.emit(OpCode::PushConst, crate::vm::core::GLOBAL_START as i32 + offset, loc);
            } else {
                self.report_error("未声明的结构体变量", loc);
                self.emit(OpCode::PushConst, 0, loc);
            }
        } else if object.ty().is_struct() || object.ty().is_class() {
            // 函数按值返回结构体/类等复杂表达式，gen_expr 会留下地址
            self.gen_expr(object);
        } else {
            self.report_error("复杂结构体表达式暂不支持", loc);
            self.emit(OpCode::PushConst, 0, loc);
        }
        let offset = self.get_member_offset(object.ty(), member);
        if offset > 0 {
            self.emit(OpCode::PushConst, offset, loc);
            self.emit(OpCode::Add, 0, loc);
        }
    }

    fn gen_index(&mut self, array: &mut Expr, index: &mut Expr, result_ty: &Type, loc: &SourceLoc, is_assign: bool) {
        let mut bound_size = -1;
        let mut sym_idx = -1;
        if let Expr::Identifier { name, .. } = array {
            if let Some(ty) = self.local_types.get(name) {
                if ty.is_array() {
                    bound_size = if ty.dims().is_empty() {
                        ty.array_size()
                    } else {
                        ty.dims()[0]
                    };
                    sym_idx = self.resolve_symbol_index(name);
                }
            } else if let Some(ty) = self.global_types.get(name) {
                if ty.is_array() {
                    bound_size = if ty.dims().is_empty() {
                        ty.array_size()
                    } else {
                        ty.dims()[0]
                    };
                    sym_idx = self.resolve_symbol_index(name);
                }
            }
        } else if let Expr::Index { .. } = array {
            if array.ty().is_array() && !array.ty().dims().is_empty() {
                bound_size = array.ty().dims()[0];
            }
        }
        let stride = super::compute_stride(array.ty(), self.elem_type_size(array.ty()));
        if stride == 0 && !array.ty().is_vla() {
            self.report_error("数组索引步长计算失败：存在无效维度", loc);
        }
        self.gen_expr(array);
        self.gen_expr(index);

        if bound_size > 0 {
            if sym_idx >= 0 {
                self.emit(OpCode::TrapBounds, sym_idx, loc);
            } else {
                self.emit(OpCode::TrapBounds, -bound_size, loc);
            }
        }

        if stride > 0 {
            self.emit(OpCode::PushConst, stride, loc);
        } else if array.ty().is_vla() {
            self.gen_vla_stride(array.ty(), loc);
        } else {
            self.emit(OpCode::PushConst, self.elem_type_size(array.ty()), loc);
        }
        self.emit(OpCode::Mul, 0, loc);
        self.emit(OpCode::Add, 0, loc);
        if !is_assign && !result_ty.is_array() {
            if result_ty.kind() == TypeKind::Char {
                self.emit(OpCode::LoadMemByte, 0, loc);
            } else if result_ty.kind() == TypeKind::Double {
                self.emit(OpCode::LoadMemD, 0, loc);
            } else if result_ty.kind() == TypeKind::LongLong {
                self.emit(OpCode::LoadMemQ, 0, loc);
            } else {
                self.emit(OpCode::LoadMem, 0, loc);
            }
        }
    }

    fn gen_vla_stride(&mut self, arr_type: &Type, loc: &SourceLoc) {
        if let Type::Array { dims, vla_dims, .. } = arr_type {
            let mut vla_idx = 0;
            for dim in dims.iter().take(1.min(dims.len())) {
                if *dim == 0 {
                    vla_idx += 1;
                }
            }
            let mut vla_dims_clone = vla_dims.clone();
            let mut first = true;
            for dim in dims.iter().skip(1) {
                if !first {
                    self.emit(OpCode::Mul, 0, loc);
                }
                if *dim > 0 {
                    self.emit(OpCode::PushConst, *dim, loc);
                } else if let Some(dim_expr) = vla_dims_clone.get_mut(vla_idx) {
                    self.gen_expr(dim_expr);
                    vla_idx += 1;
                } else {
                    self.emit(OpCode::PushConst, 0, loc);
                }
                first = false;
            }
            let elem_size = self.elem_type_size(arr_type);
            if first {
                self.emit(OpCode::PushConst, elem_size, loc);
            } else {
                self.emit(OpCode::PushConst, elem_size, loc);
                self.emit(OpCode::Mul, 0, loc);
            }
        } else {
            self.emit(OpCode::PushConst, self.elem_type_size(arr_type), loc);
        }
    }

    fn gen_expr_with_cast(&mut self, expr: &mut Expr, target_is_fp: bool, target_is_double: bool, loc: &SourceLoc) {
        self.gen_expr(expr);
        let _target_is_long_long = !target_is_fp
            && expr.ty().kind() != TypeKind::Int
            && expr.ty().kind() != TypeKind::Char
            && expr.ty().kind() != TypeKind::Float
            && expr.ty().kind() != TypeKind::Double;
        // Note: target_is_long_long heuristic is approximate; caller ensures correct cast via Cast nodes
        if target_is_double
            && expr.ty().kind() != TypeKind::Float
            && expr.ty().kind() != TypeKind::Double
            && expr.ty().kind() != TypeKind::LongLong
        {
            self.emit(OpCode::CastI2D, 0, loc);
        } else if target_is_double && expr.ty().kind() == TypeKind::Float {
            self.emit(OpCode::CastF2D, 0, loc);
        } else if target_is_double && expr.ty().kind() == TypeKind::LongLong {
            self.emit(OpCode::CastQ2D, 0, loc);
        } else if !target_is_double
            && target_is_fp
            && expr.ty().kind() != TypeKind::Float
            && expr.ty().kind() != TypeKind::Double
            && expr.ty().kind() != TypeKind::LongLong
        {
            self.emit(OpCode::CastI2F, 0, loc);
        } else if !target_is_fp && expr.ty().kind() == TypeKind::Double {
            self.emit(OpCode::CastD2I, 0, loc);
        } else if !target_is_fp && expr.ty().kind() == TypeKind::Float {
            self.emit(OpCode::CastF2I, 0, loc);
        } else if !target_is_fp && expr.ty().kind() == TypeKind::LongLong {
            self.emit(OpCode::CastQ2I, 0, loc);
        }
    }

    fn gen_addr(&mut self, expr: &mut Expr, loc: &SourceLoc) {
        match expr {
            Expr::Identifier { name, .. } => {
                if let Some(&offset) = self.local_indices.get(name) {
                    if let Some(ty) = self.local_types.get(name) {
                        if ty.is_reference() || ty.is_rvalue_ref() {
                            self.emit(OpCode::LoadLocal, offset, loc);
                            return;
                        }
                    }
                    self.emit(OpCode::GetFrameBase, 0, loc);
                    self.emit(OpCode::PushConst, offset, loc);
                    self.emit(OpCode::Add, 0, loc);
                } else if let Some(&offset) = self.global_indices.get(name) {
                    if let Some(ty) = self.global_types.get(name) {
                        if ty.is_reference() || ty.is_rvalue_ref() {
                            self.emit(OpCode::LoadGlobal, offset, loc);
                            return;
                        }
                    }
                    self.emit(OpCode::PushConst, crate::vm::core::GLOBAL_START as i32 + offset, loc);
                } else {
                    self.report_error("未声明的变量", loc);
                    // 错误已被记录，提前返回以避免生成无意义指令。
                    // 编译管线末端的 has_errors() 守卫会拦截并丢弃错误字节码。
                }
            }
            Expr::Index { array, index, ty, .. } => {
                self.gen_index(array, index, ty, loc, true);
            }
            Expr::Member { object, member, .. } => {
                self.gen_member_addr(object, member, loc);
            }
            Expr::Unary {
                op: UnaryOp::Deref, operand, ..
            } => {
                self.gen_expr(operand);
            }
            Expr::Call { ty, .. } | Expr::CallPtr { ty, .. }
                if ty.is_struct() || ty.is_class() || ty.is_reference() || ty.is_rvalue_ref() =>
            {
                // For std::move(x), gen_addr should yield the address of x,
                // not the value that gen_expr would leave.
                if let Expr::CallPtr { callee, args, .. } = expr {
                    if let Expr::Identifier { name, .. } = callee.as_ref() {
                        if name == "std__move" && args.len() == 1 {
                            self.gen_addr(&mut args[0], loc);
                            return;
                        }
                    }
                }
                // 函数按值返回结构体 / 返回引用，gen_expr 已在栈顶留下地址
                self.gen_expr(expr);
            }
            Expr::MemberCall { ty, .. } if ty.is_reference() || ty.is_rvalue_ref() => {
                // 成员函数返回引用：gen_expr 已在栈顶留下目标地址
                self.gen_expr(expr);
            }
            Expr::MemberCall { ty, .. } if ty.is_struct() || ty.is_class() => {
                // 成员函数按值返回结构体/类：gen_expr 已在栈顶留下临时对象地址
                self.gen_expr(expr);
            }
            Expr::Lambda { .. } => {
                // Lambda 表达式已在栈顶留下临时闭包对象的地址
                self.gen_expr(expr);
            }
            Expr::Move { expr: inner, .. } => {
                // std::move(x) — address of the moved-from object
                self.gen_addr(inner, loc);
            }
            _ => {
                self.report_error("不支持的地址生成", loc);
            }
        }
    }

    /// 通用结构体/union 拷贝循环：通过闭包生成目标地址加载指令。
    fn gen_struct_copy_common<F: FnMut(&mut Self, &SourceLoc, i32)>(
        &mut self,
        size: i32,
        src_expr: &mut Expr,
        mut dst_emit: F,
        loc: &SourceLoc,
    ) {
        if size <= 0 {
            return;
        }
        let src_temp = self.get_temp_slot(0);
        self.gen_addr(src_expr, loc);
        self.emit(OpCode::StoreLocal, src_temp, loc);
        for i in 0..size / 4 {
            dst_emit(self, loc, i);
            self.emit(OpCode::LoadLocal, src_temp, loc);
            if i > 0 {
                self.emit(OpCode::PushConst, i * 4, loc);
                self.emit(OpCode::Add, 0, loc);
            }
            self.emit(OpCode::LoadMem, 0, loc);
            self.emit(OpCode::StoreMem, 0, loc);
        }
    }

    fn gen_struct_copy(&mut self, left: &mut Expr, right: &mut Expr, loc: &SourceLoc) {
        let size = self.type_size(left.ty());
        let dst_temp = self.get_temp_slot(1);
        self.gen_addr(left, loc);
        self.emit(OpCode::StoreLocal, dst_temp, loc);
        self.gen_struct_copy_common(
            size,
            right,
            |gen, loc, i| {
                gen.emit(OpCode::LoadLocal, dst_temp, loc);
                if i > 0 {
                    gen.emit(OpCode::PushConst, i * 4, loc);
                    gen.emit(OpCode::Add, 0, loc);
                }
            },
            loc,
        );
    }

    fn gen_struct_copy_to_local(&mut self, local_offset: i32, right: &mut Expr, loc: &SourceLoc) {
        let size = self.type_size(right.ty());
        self.gen_struct_copy_common(
            size,
            right,
            |gen, loc, i| {
                gen.emit(OpCode::GetFrameBase, 0, loc);
                gen.emit(OpCode::PushConst, local_offset + i * 4, loc);
                gen.emit(OpCode::Add, 0, loc);
            },
            loc,
        );
    }

    fn gen_assign(&mut self, op: &AssignOp, left: &mut Expr, right: &mut Expr, loc: &SourceLoc) {
        let left_is_double = left.ty().kind() == TypeKind::Double;
        let left_is_float = left.ty().kind() == TypeKind::Float;
        let left_is_long_long = left.ty().kind() == TypeKind::LongLong;
        let left_is_unsigned = left.ty().is_unsigned();
        let left_is_fp = left_is_double || left_is_float;
        if (left.ty().is_struct() || left.ty().is_class()) && *op == AssignOp::Assign {
            self.gen_struct_copy(left, right, loc);
            return;
        }
        let emit_compound = |this: &mut Self, loc: &SourceLoc| match op {
            AssignOp::AddAssign => {
                if left_is_double {
                    this.emit(OpCode::AddD, 0, loc);
                } else if left_is_float {
                    this.emit(OpCode::AddF, 0, loc);
                } else if left_is_unsigned {
                    this.emit(OpCode::UAdd, 0, loc);
                } else {
                    this.emit(OpCode::Add, 0, loc);
                }
            }
            AssignOp::SubAssign => {
                if left_is_double {
                    this.emit(OpCode::SubD, 0, loc);
                } else if left_is_float {
                    this.emit(OpCode::SubF, 0, loc);
                } else if left_is_unsigned {
                    this.emit(OpCode::USub, 0, loc);
                } else {
                    this.emit(OpCode::Sub, 0, loc);
                }
            }
            AssignOp::MulAssign => {
                if left_is_double {
                    this.emit(OpCode::MulD, 0, loc);
                } else if left_is_float {
                    this.emit(OpCode::MulF, 0, loc);
                } else if left_is_unsigned {
                    this.emit(OpCode::UMul, 0, loc);
                } else {
                    this.emit(OpCode::Mul, 0, loc);
                }
            }
            AssignOp::DivAssign => {
                if left_is_double {
                    this.emit(OpCode::DivD, 0, loc);
                } else if left_is_float {
                    this.emit(OpCode::DivF, 0, loc);
                } else if left_is_unsigned {
                    this.emit(OpCode::UDiv, 0, loc);
                } else {
                    this.emit(OpCode::Div, 0, loc);
                }
            }
            AssignOp::ModAssign => {
                if left_is_long_long {
                    this.emit(OpCode::ModQ, 0, loc);
                } else if left_is_unsigned {
                    this.emit(OpCode::UMod, 0, loc);
                } else {
                    this.emit(OpCode::Mod, 0, loc);
                }
            }
            AssignOp::AndAssign => {
                this.emit(OpCode::BitAnd, 0, loc);
            }
            AssignOp::OrAssign => {
                this.emit(OpCode::BitOr, 0, loc);
            }
            AssignOp::XorAssign => {
                this.emit(OpCode::BitXor, 0, loc);
            }
            AssignOp::ShlAssign => {
                this.emit(OpCode::Shl, 0, loc);
            }
            AssignOp::ShrAssign => {
                if left_is_unsigned {
                    this.emit(OpCode::LShr, 0, loc);
                } else {
                    this.emit(OpCode::Shr, 0, loc);
                }
            }
            _ => {}
        };

        // C++ 引用返回的函数调用 / 成员调用作为左值：调用结果已在栈顶留下目标地址
        if (matches!(left, Expr::Call { .. } | Expr::CallPtr { .. } | Expr::MemberCall { .. }))
            && (left.ty().is_reference() || left.ty().is_rvalue_ref())
        {
            if *op != AssignOp::Assign {
                self.report_error("复合赋值暂不支持引用返回的调用结果", loc);
            }
            let base_ty = left.ty().reference_base().cloned().unwrap_or(Type::int());
            let base_is_double = base_ty.kind() == TypeKind::Double;
            let base_is_float = base_ty.kind() == TypeKind::Float;
            let base_is_fp = base_is_double || base_is_float;
            self.gen_addr(left, loc);
            self.emit(OpCode::Dup, 0, loc);
            let addr_temp = self.get_temp_slot(0);
            self.emit(OpCode::StoreLocal, addr_temp, loc);
            self.gen_expr_with_cast(right, base_is_fp, base_is_double, loc);
            match base_ty.kind() {
                TypeKind::Char => self.emit(OpCode::StoreMemByte, 0, loc),
                TypeKind::Double => self.emit(OpCode::StoreMemD, 0, loc),
                TypeKind::LongLong => self.emit(OpCode::StoreMemQ, 0, loc),
                _ => self.emit(OpCode::StoreMem, 0, loc),
            }
            self.emit(OpCode::LoadLocal, addr_temp, loc);
            match base_ty.kind() {
                TypeKind::Char => self.emit(OpCode::LoadMemByte, 0, loc),
                TypeKind::Double => self.emit(OpCode::LoadMemD, 0, loc),
                TypeKind::LongLong => self.emit(OpCode::LoadMemQ, 0, loc),
                _ => self.emit(OpCode::LoadMem, 0, loc),
            }
            return;
        }

        if let Expr::Identifier { name, .. } = left {
            let is_ref = self
                .local_types
                .get(name)
                .map(|t| t.is_reference() || t.is_rvalue_ref())
                .unwrap_or(false)
                || self
                    .global_types
                    .get(name)
                    .map(|t| t.is_reference() || t.is_rvalue_ref())
                    .unwrap_or(false)
                || self
                    .static_local_types
                    .get(name)
                    .map(|t| t.is_reference() || t.is_rvalue_ref())
                    .unwrap_or(false);
            if is_ref {
                let base_ty = self
                    .local_types
                    .get(name)
                    .or_else(|| self.global_types.get(name))
                    .or_else(|| self.static_local_types.get(name))
                    .and_then(|t| t.reference_base().cloned())
                    .unwrap_or(Type::int());
                let base_is_double = base_ty.kind() == TypeKind::Double;
                let base_is_float = base_ty.kind() == TypeKind::Float;
                let base_is_long_long = base_ty.kind() == TypeKind::LongLong;
                let base_is_fp = base_is_double || base_is_float;
                self.gen_addr(left, loc);
                if *op != AssignOp::Assign {
                    self.emit(OpCode::Dup, 0, loc);
                    match base_ty.kind() {
                        TypeKind::Char => self.emit(OpCode::LoadMemByte, 0, loc),
                        TypeKind::Double => self.emit(OpCode::LoadMemD, 0, loc),
                        TypeKind::LongLong => self.emit(OpCode::LoadMemQ, 0, loc),
                        _ => self.emit(OpCode::LoadMem, 0, loc),
                    }
                    self.emit(OpCode::Swap, 0, loc);
                    let addr_temp = self.get_temp_slot(0);
                    self.emit(OpCode::StoreLocal, addr_temp, loc);
                    self.gen_expr_with_cast(right, base_is_fp, base_is_double, loc);
                    match op {
                        AssignOp::AddAssign => {
                            if base_is_double {
                                self.emit(OpCode::AddD, 0, loc);
                            } else if base_is_float {
                                self.emit(OpCode::AddF, 0, loc);
                            } else if base_ty.is_unsigned() {
                                self.emit(OpCode::UAdd, 0, loc);
                            } else {
                                self.emit(OpCode::Add, 0, loc);
                            }
                        }
                        AssignOp::SubAssign => {
                            if base_is_double {
                                self.emit(OpCode::SubD, 0, loc);
                            } else if base_is_float {
                                self.emit(OpCode::SubF, 0, loc);
                            } else if base_ty.is_unsigned() {
                                self.emit(OpCode::USub, 0, loc);
                            } else {
                                self.emit(OpCode::Sub, 0, loc);
                            }
                        }
                        AssignOp::MulAssign => {
                            if base_is_double {
                                self.emit(OpCode::MulD, 0, loc);
                            } else if base_is_float {
                                self.emit(OpCode::MulF, 0, loc);
                            } else if base_ty.is_unsigned() {
                                self.emit(OpCode::UMul, 0, loc);
                            } else {
                                self.emit(OpCode::Mul, 0, loc);
                            }
                        }
                        AssignOp::DivAssign => {
                            if base_is_double {
                                self.emit(OpCode::DivD, 0, loc);
                            } else if base_is_float {
                                self.emit(OpCode::DivF, 0, loc);
                            } else if base_ty.is_unsigned() {
                                self.emit(OpCode::UDiv, 0, loc);
                            } else {
                                self.emit(OpCode::Div, 0, loc);
                            }
                        }
                        AssignOp::ModAssign => {
                            if base_is_long_long {
                                self.emit(OpCode::ModQ, 0, loc);
                            } else if base_ty.is_unsigned() {
                                self.emit(OpCode::UMod, 0, loc);
                            } else {
                                self.emit(OpCode::Mod, 0, loc);
                            }
                        }
                        AssignOp::AndAssign => {
                            self.emit(OpCode::BitAnd, 0, loc);
                        }
                        AssignOp::OrAssign => {
                            self.emit(OpCode::BitOr, 0, loc);
                        }
                        AssignOp::XorAssign => {
                            self.emit(OpCode::BitXor, 0, loc);
                        }
                        AssignOp::ShlAssign => {
                            self.emit(OpCode::Shl, 0, loc);
                        }
                        AssignOp::ShrAssign => {
                            if base_ty.is_unsigned() {
                                self.emit(OpCode::LShr, 0, loc);
                            } else {
                                self.emit(OpCode::Shr, 0, loc);
                            }
                        }
                        _ => {}
                    }
                    self.emit(OpCode::LoadLocal, addr_temp, loc);
                    self.emit(OpCode::Swap, 0, loc);
                    match base_ty.kind() {
                        TypeKind::Char => self.emit(OpCode::StoreMemByte, 0, loc),
                        TypeKind::Double => self.emit(OpCode::StoreMemD, 0, loc),
                        TypeKind::LongLong => self.emit(OpCode::StoreMemQ, 0, loc),
                        _ => self.emit(OpCode::StoreMem, 0, loc),
                    }
                    self.emit(OpCode::LoadLocal, addr_temp, loc);
                    match base_ty.kind() {
                        TypeKind::Char => self.emit(OpCode::LoadMemByte, 0, loc),
                        TypeKind::Double => self.emit(OpCode::LoadMemD, 0, loc),
                        TypeKind::LongLong => self.emit(OpCode::LoadMemQ, 0, loc),
                        _ => self.emit(OpCode::LoadMem, 0, loc),
                    }
                } else {
                    self.emit(OpCode::Dup, 0, loc);
                    let addr_temp = self.get_temp_slot(0);
                    self.emit(OpCode::StoreLocal, addr_temp, loc);
                    self.gen_expr_with_cast(right, base_is_fp, base_is_double, loc);
                    match base_ty.kind() {
                        TypeKind::Char => self.emit(OpCode::StoreMemByte, 0, loc),
                        TypeKind::Double => self.emit(OpCode::StoreMemD, 0, loc),
                        TypeKind::LongLong => self.emit(OpCode::StoreMemQ, 0, loc),
                        _ => self.emit(OpCode::StoreMem, 0, loc),
                    }
                    self.emit(OpCode::LoadLocal, addr_temp, loc);
                    match base_ty.kind() {
                        TypeKind::Char => self.emit(OpCode::LoadMemByte, 0, loc),
                        TypeKind::Double => self.emit(OpCode::LoadMemD, 0, loc),
                        TypeKind::LongLong => self.emit(OpCode::LoadMemQ, 0, loc),
                        _ => self.emit(OpCode::LoadMem, 0, loc),
                    }
                }
                return;
            }
            if let Some(&static_offset) = self.static_local_indices.get(name) {
                if *op != AssignOp::Assign {
                    if left_is_double {
                        self.emit(OpCode::LoadGlobalD, static_offset, loc);
                    } else if left_is_long_long {
                        self.emit(OpCode::LoadGlobalQ, static_offset, loc);
                    } else {
                        self.emit(OpCode::LoadGlobal, static_offset, loc);
                    }
                    self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                    emit_compound(self, loc);
                } else {
                    self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                }
                if left_is_double {
                    self.emit(OpCode::StoreGlobalD, static_offset, loc);
                } else if left_is_long_long {
                    self.emit(OpCode::StoreGlobalQ, static_offset, loc);
                } else {
                    self.emit(OpCode::StoreGlobal, static_offset, loc);
                }
                if left_is_double {
                    self.emit(OpCode::LoadGlobalD, static_offset, loc);
                } else if left_is_long_long {
                    self.emit(OpCode::LoadGlobalQ, static_offset, loc);
                } else {
                    self.emit(OpCode::LoadGlobal, static_offset, loc);
                }
                return;
            }
            let local_offset = self.resolve_local(name);
            if local_offset >= 0 {
                if *op != AssignOp::Assign {
                    if left_is_double {
                        self.emit(OpCode::LoadLocalD, local_offset, loc);
                    } else if left_is_long_long {
                        self.emit(OpCode::LoadLocalQ, local_offset, loc);
                    } else {
                        self.emit(OpCode::LoadLocal, local_offset, loc);
                    }
                    self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                    emit_compound(self, loc);
                } else {
                    self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                }
                if left_is_double {
                    self.emit(OpCode::StoreLocalD, local_offset, loc);
                } else if left_is_long_long {
                    self.emit(OpCode::StoreLocalQ, local_offset, loc);
                } else {
                    self.emit(OpCode::StoreLocal, local_offset, loc);
                }
                if left_is_double {
                    self.emit(OpCode::LoadLocalD, local_offset, loc);
                } else if left_is_long_long {
                    self.emit(OpCode::LoadLocalQ, local_offset, loc);
                } else {
                    self.emit(OpCode::LoadLocal, local_offset, loc);
                }
                return;
            }
            let global_offset = self.resolve_global(name);
            if global_offset >= 0 {
                if *op != AssignOp::Assign {
                    if left_is_double {
                        self.emit(OpCode::LoadGlobalD, global_offset, loc);
                    } else if left_is_long_long {
                        self.emit(OpCode::LoadGlobalQ, global_offset, loc);
                    } else {
                        self.emit(OpCode::LoadGlobal, global_offset, loc);
                    }
                    self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                    emit_compound(self, loc);
                } else {
                    self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                }
                if left_is_double {
                    self.emit(OpCode::StoreGlobalD, global_offset, loc);
                } else if left_is_long_long {
                    self.emit(OpCode::StoreGlobalQ, global_offset, loc);
                } else {
                    self.emit(OpCode::StoreGlobal, global_offset, loc);
                }
                if left_is_double {
                    self.emit(OpCode::LoadGlobalD, global_offset, loc);
                } else if left_is_long_long {
                    self.emit(OpCode::LoadGlobalQ, global_offset, loc);
                } else {
                    self.emit(OpCode::LoadGlobal, global_offset, loc);
                }
                return;
            }
        } else if let Expr::Index { array, index, ty, .. } = left {
            let result_ty = ty.clone();
            self.gen_index(array, index, &result_ty, loc, true);
            if *op != AssignOp::Assign {
                self.emit(OpCode::Dup, 0, loc);
                if result_ty.kind() == TypeKind::Char {
                    self.emit(OpCode::LoadMemByte, 0, loc);
                } else if result_ty.kind() == TypeKind::Double {
                    self.emit(OpCode::LoadMemD, 0, loc);
                } else if result_ty.kind() == TypeKind::LongLong {
                    self.emit(OpCode::LoadMemQ, 0, loc);
                } else {
                    self.emit(OpCode::LoadMem, 0, loc);
                }
                self.emit(OpCode::Swap, 0, loc);
                let addr_temp = self.get_temp_slot(0);
                self.emit(OpCode::StoreLocal, addr_temp, loc);
                self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                emit_compound(self, loc);
                self.emit(OpCode::LoadLocal, addr_temp, loc);
                self.emit(OpCode::Swap, 0, loc);
                if result_ty.kind() == TypeKind::Char {
                    self.emit(OpCode::StoreMemByte, 0, loc);
                } else if result_ty.kind() == TypeKind::Double {
                    self.emit(OpCode::StoreMemD, 0, loc);
                } else if result_ty.kind() == TypeKind::LongLong {
                    self.emit(OpCode::StoreMemQ, 0, loc);
                } else {
                    self.emit(OpCode::StoreMem, 0, loc);
                }
                self.emit(OpCode::LoadLocal, addr_temp, loc);
                if result_ty.kind() == TypeKind::Char {
                    self.emit(OpCode::LoadMemByte, 0, loc);
                } else if result_ty.kind() == TypeKind::Double {
                    self.emit(OpCode::LoadMemD, 0, loc);
                } else if result_ty.kind() == TypeKind::LongLong {
                    self.emit(OpCode::LoadMemQ, 0, loc);
                } else {
                    self.emit(OpCode::LoadMem, 0, loc);
                }
            } else {
                self.emit(OpCode::Dup, 0, loc);
                let addr_temp = self.get_temp_slot(0);
                self.emit(OpCode::StoreLocal, addr_temp, loc);
                self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                if result_ty.kind() == TypeKind::Char {
                    self.emit(OpCode::StoreMemByte, 0, loc);
                } else if result_ty.kind() == TypeKind::Double {
                    self.emit(OpCode::StoreMemD, 0, loc);
                } else if result_ty.kind() == TypeKind::LongLong {
                    self.emit(OpCode::StoreMemQ, 0, loc);
                } else {
                    self.emit(OpCode::StoreMem, 0, loc);
                }
                self.emit(OpCode::LoadLocal, addr_temp, loc);
                if result_ty.kind() == TypeKind::Char {
                    self.emit(OpCode::LoadMemByte, 0, loc);
                } else if result_ty.kind() == TypeKind::Double {
                    self.emit(OpCode::LoadMemD, 0, loc);
                } else if result_ty.kind() == TypeKind::LongLong {
                    self.emit(OpCode::LoadMemQ, 0, loc);
                } else {
                    self.emit(OpCode::LoadMem, 0, loc);
                }
            }
            return;
        } else if let Expr::Unary {
            op: UnaryOp::Deref, operand, ..
        } = left
        {
            self.gen_expr(operand);
            let left_is_char = left.ty().kind() == TypeKind::Char;
            if *op != AssignOp::Assign {
                self.emit(OpCode::Dup, 0, loc);
                if left_is_char {
                    self.emit(OpCode::LoadMemByte, 0, loc);
                } else if left_is_double {
                    self.emit(OpCode::LoadMemD, 0, loc);
                } else if left_is_long_long {
                    self.emit(OpCode::LoadMemQ, 0, loc);
                } else {
                    self.emit(OpCode::LoadMem, 0, loc);
                }
                self.emit(OpCode::Swap, 0, loc);
                let addr_temp = self.get_temp_slot(0);
                self.emit(OpCode::StoreLocal, addr_temp, loc);
                self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                emit_compound(self, loc);
                self.emit(OpCode::LoadLocal, addr_temp, loc);
                self.emit(OpCode::Swap, 0, loc);
                if left_is_char {
                    self.emit(OpCode::StoreMemByte, 0, loc);
                } else if left_is_double {
                    self.emit(OpCode::StoreMemD, 0, loc);
                } else if left_is_long_long {
                    self.emit(OpCode::StoreMemQ, 0, loc);
                } else {
                    self.emit(OpCode::StoreMem, 0, loc);
                }
                self.emit(OpCode::LoadLocal, addr_temp, loc);
                if left_is_char {
                    self.emit(OpCode::LoadMemByte, 0, loc);
                } else if left_is_double {
                    self.emit(OpCode::LoadMemD, 0, loc);
                } else if left_is_long_long {
                    self.emit(OpCode::LoadMemQ, 0, loc);
                } else {
                    self.emit(OpCode::LoadMem, 0, loc);
                }
            } else {
                self.emit(OpCode::Dup, 0, loc);
                let addr_temp = self.get_temp_slot(0);
                self.emit(OpCode::StoreLocal, addr_temp, loc);
                self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                if left_is_char {
                    self.emit(OpCode::StoreMemByte, 0, loc);
                } else if left_is_double {
                    self.emit(OpCode::StoreMemD, 0, loc);
                } else if left_is_long_long {
                    self.emit(OpCode::StoreMemQ, 0, loc);
                } else {
                    self.emit(OpCode::StoreMem, 0, loc);
                }
                self.emit(OpCode::LoadLocal, addr_temp, loc);
                if left_is_char {
                    self.emit(OpCode::LoadMemByte, 0, loc);
                } else if left_is_double {
                    self.emit(OpCode::LoadMemD, 0, loc);
                } else if left_is_long_long {
                    self.emit(OpCode::LoadMemQ, 0, loc);
                } else {
                    self.emit(OpCode::LoadMem, 0, loc);
                }
            }
            return;
        } else if let Expr::Member { object, member, .. } = left {
            self.gen_member_addr(object, member, loc);
            // Lambda by-reference capture: load the captured pointer so StoreMem writes through it
            if object.ty().is_pointer() {
                if let Type::Pointer { pointee, .. } = object.ty() {
                    if let Type::Class { name, .. } = pointee.as_ref() {
                        if let Some(by_ref_fields) = self.lambda_by_ref_fields.get(name) {
                            if by_ref_fields.contains(member) {
                                self.emit(OpCode::LoadMem, 0, loc);
                            }
                        }
                    }
                }
            }
            if *op != AssignOp::Assign {
                self.emit(OpCode::Dup, 0, loc);
                if left_is_double {
                    self.emit(OpCode::LoadMemD, 0, loc);
                } else if left_is_long_long {
                    self.emit(OpCode::LoadMemQ, 0, loc);
                } else {
                    self.emit(OpCode::LoadMem, 0, loc);
                }
                self.emit(OpCode::Swap, 0, loc);
                let addr_temp = self.get_temp_slot(0);
                self.emit(OpCode::StoreLocal, addr_temp, loc);
                self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                emit_compound(self, loc);
                self.emit(OpCode::LoadLocal, addr_temp, loc);
                self.emit(OpCode::Swap, 0, loc);
                if left_is_double {
                    self.emit(OpCode::StoreMemD, 0, loc);
                } else if left_is_long_long {
                    self.emit(OpCode::StoreMemQ, 0, loc);
                } else {
                    self.emit(OpCode::StoreMem, 0, loc);
                }
                self.emit(OpCode::LoadLocal, addr_temp, loc);
                if left_is_double {
                    self.emit(OpCode::LoadMemD, 0, loc);
                } else if left_is_long_long {
                    self.emit(OpCode::LoadMemQ, 0, loc);
                } else {
                    self.emit(OpCode::LoadMem, 0, loc);
                }
            } else {
                self.emit(OpCode::Dup, 0, loc);
                let addr_temp = self.get_temp_slot(0);
                self.emit(OpCode::StoreLocal, addr_temp, loc);
                self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                if left_is_double {
                    self.emit(OpCode::StoreMemD, 0, loc);
                } else if left_is_long_long {
                    self.emit(OpCode::StoreMemQ, 0, loc);
                } else {
                    self.emit(OpCode::StoreMem, 0, loc);
                }
                self.emit(OpCode::LoadLocal, addr_temp, loc);
                if left_is_double {
                    self.emit(OpCode::LoadMemD, 0, loc);
                } else if left_is_long_long {
                    self.emit(OpCode::LoadMemQ, 0, loc);
                } else {
                    self.emit(OpCode::LoadMem, 0, loc);
                }
            }
            return;
        }

        self.report_error("赋值目标不支持", loc);
        self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
        self.emit(OpCode::Pop, 0, loc);
        self.emit(OpCode::PushConst, 0, loc);
    }
}
