use super::expr::ExprGen;
use super::*;

pub(crate) trait StmtGen {
    fn gen_stmt(&mut self, stmt: &mut Stmt);
    fn gen_switch(&mut self, cond: &mut Expr, body: &mut Stmt, loc: &SourceLoc);
}

impl StmtGen for BytecodeGen {
    fn gen_stmt(&mut self, stmt: &mut Stmt) {
        let loc = stmt_loc(stmt);
        if loc.line > 0 {
            self.emit(OpCode::StepEvent, loc.line, &loc);
        }
        match stmt {
            Stmt::Block { stmts, .. } => {
                self.enter_scope();
                for s in stmts {
                    self.gen_stmt(s);
                }
                self.exit_scope();
            }
            Stmt::VarDecl {
                var_type,
                name,
                init,
                extra_vars,
                is_static,
                loc,
            } => {
                let mut emit_one = |vty: &Type, n: &str, init: &mut Option<Expr>, loc: &SourceLoc, is_static: bool| {
                    if is_static {
                        if vty.is_vla() {
                            self.report_error("static 变量不支持变长数组(VLA)", loc);
                            return;
                        }
                        if self.static_local_indices.contains_key(n) {
                            return;
                        }
                        let sz = self.type_size(vty);
                        let aligned_sz = (sz + 3) & !3;
                        let global_offset = (self.string_mem_offset - crate::vm::vm::GLOBAL_START) as i32;
                        self.string_mem_offset += aligned_sz as u32;
                        self.static_local_indices.insert(n.to_string(), global_offset);
                        self.static_local_types.insert(n.to_string(), vty.clone());
                        self.sym_index.insert(n.to_string(), self.symbols.len() as i32);
                        self.symbols.push(VMSymbol {
                            name: n.to_string(),
                            addr: (crate::vm::vm::GLOBAL_START as i32 + global_offset) as u32,
                            is_local: false,
                            ty: vty.clone(),
                            scope_depth: 0,
                            func_name: self.current_func.clone(),
                        });
                        if let Some(e) = init {
                            match e {
                                Expr::InitList { elements, .. } => {
                                    let has_designators = elements.iter().any(|e| !e.designators.is_empty());
                                    if has_designators {
                                        self.report_error("静态局部变量的 designated initializer 暂不支持", loc);
                                        return;
                                    }
                                    let is_char_array = if let Type::Array { element, .. } = vty {
                                        element.kind() == TypeKind::Char
                                    } else {
                                        false
                                    };
                                    if is_char_array {
                                        let values = super::flatten_init_list(elements, &mut self.errors);
                                        for i in 0..sz as usize {
                                            self.globals_init_32.push((
                                                global_offset as u32 + i as u32,
                                                values.get(i).copied().unwrap_or(0),
                                            ));
                                        }
                                    } else {
                                        let elem_size = self.elem_type_size(vty);
                                        let count = vty.total_elements();
                                        if elem_size == 8 {
                                            for (i, elem) in elements.iter().enumerate() {
                                                let addr = global_offset as u32 + (i as u32) * elem_size as u32;
                                                if let Expr::FloatLiteral { value, .. } = &elem.value {
                                                    self.globals_init_64.push((addr, value.to_bits()));
                                                } else if let Expr::LongLiteral { value, .. } = &elem.value {
                                                    self.globals_init_64.push((addr, *value as u64));
                                                } else {
                                                    let val = super::flatten_init_list(
                                                        std::slice::from_ref(elem),
                                                        &mut self.errors,
                                                    )
                                                    .first()
                                                    .copied()
                                                    .unwrap_or(0);
                                                    self.globals_init_32.push((addr, val));
                                                }
                                            }
                                        } else {
                                            for (i, elem) in elements.iter().enumerate() {
                                                let addr = global_offset as u32 + (i as u32) * elem_size as u32;
                                                match &elem.value {
                                                    Expr::StringLiteral { value, .. } => {
                                                        let str_addr = self.string_mem_offset;
                                                        self.string_data.push((str_addr, value.clone()));
                                                        self.string_mem_offset += (value.len() + 1) as u32;
                                                        self.globals_init_32.push((addr, str_addr as i32));
                                                    }
                                                    Expr::FloatLiteral { value, .. } => {
                                                        self.globals_init_32
                                                            .push((addr, (*value as f32).to_bits() as i32));
                                                    }
                                                    Expr::LongLiteral { value, .. } => {
                                                        self.globals_init_32.push((addr, *value as i32));
                                                    }
                                                    Expr::Literal { value, .. } => {
                                                        self.globals_init_32.push((addr, *value));
                                                    }
                                                    Expr::Unary { op: UnaryOp::Neg, operand, .. } => {
                                                        if let Expr::Literal { value, .. } = operand.as_ref() {
                                                            self.globals_init_32.push((addr, -(*value)));
                                                        } else {
                                                            self.globals_init_32.push((addr, 0));
                                                        }
                                                    }
                                                    _ => {
                                                        let val = super::flatten_init_list(
                                                            std::slice::from_ref(elem),
                                                            &mut self.errors,
                                                        )
                                                        .first()
                                                        .copied()
                                                        .unwrap_or(0);
                                                        self.globals_init_32.push((addr, val));
                                                    }
                                                }
                                            }
                                            for i in elements.len()..count as usize {
                                                let addr = global_offset as u32 + (i as u32) * elem_size as u32;
                                                self.globals_init_32.push((addr, 0));
                                            }
                                        }
                                    }
                                }
                                Expr::StringLiteral { value, .. } => {
                                    for i in 0..sz as usize {
                                        let byte = if i < value.len() { value.as_bytes()[i] as i32 } else { 0 };
                                        self.globals_init_32.push((global_offset as u32 + i as u32, byte));
                                    }
                                }
                                Expr::Literal { value, .. } => {
                                    self.globals_init_32.push((global_offset as u32, *value));
                                }
                                Expr::LongLiteral { value, .. } => {
                                    self.globals_init_64.push((global_offset as u32, *value as u64));
                                }
                                Expr::FloatLiteral { value, .. } => {
                                    if vty.kind() == TypeKind::Double {
                                        self.globals_init_64.push((global_offset as u32, value.to_bits()));
                                    } else {
                                        self.globals_init_32
                                            .push((global_offset as u32, (*value as f32).to_bits() as i32));
                                    }
                                }
                                Expr::Identifier { name: id_name, .. } => {
                                    if let Some(&idx) = self.func_index.get(id_name) {
                                        self.globals_init_32.push((global_offset as u32, idx));
                                    }
                                }
                                _ => {
                                    self.gen_expr(e);
                                    if vty.kind() == TypeKind::Double {
                                        self.emit(OpCode::StoreGlobalD, global_offset, loc);
                                    } else if vty.kind() == TypeKind::LongLong {
                                        self.emit(OpCode::StoreGlobalQ, global_offset, loc);
                                    } else {
                                        self.emit(OpCode::StoreGlobal, global_offset, loc);
                                    }
                                }
                            }
                        }
                        return;
                    }
                    let sz = self.type_size(vty);
                    let aligned_sz = (sz + 3) & !3;
                    let local_offset = self.next_local_offset;
                    self.next_local_offset += aligned_sz;
                    self.record_scope_var(n);
                    self.local_indices.insert(n.to_string(), local_offset);
                    self.local_types.insert(n.to_string(), vty.clone());
                    self.sym_index.insert(n.to_string(), self.symbols.len() as i32);
                    self.symbols.push(VMSymbol {
                        name: n.to_string(),
                        addr: local_offset as u32,
                        is_local: true,
                        ty: vty.clone(),
                        scope_depth: 1,
                        func_name: self.current_func.clone(),
                    });
                    if vty.is_vla() {
                        if let Type::Array { dims, vla_dims, .. } = vty {
                            let mut vla_idx = 0;
                            let mut vla_dims_clone = vla_dims.clone();
                            let elem_size = self.elem_type_size(vty);
                            if dims.is_empty() {
                                self.emit(OpCode::PushConst, 0, loc);
                            } else {
                                for &dim in dims.iter() {
                                    if dim > 0 {
                                        self.emit(OpCode::PushConst, dim, loc);
                                    } else if let Some(dim_expr) = vla_dims_clone.get_mut(vla_idx) {
                                        self.gen_expr(dim_expr);
                                        vla_idx += 1;
                                    } else {
                                        self.emit(OpCode::PushConst, 0, loc);
                                    }
                                }
                                for _ in 1..dims.len() {
                                    self.emit(OpCode::Mul, 0, loc);
                                }
                                if elem_size > 1 {
                                    self.emit(OpCode::PushConst, elem_size, loc);
                                    self.emit(OpCode::Mul, 0, loc);
                                }
                            }
                            self.emit(OpCode::StackAlloc, 0, loc);
                            self.emit(OpCode::StoreLocal, local_offset, loc);
                        }
                        if let Some(ref mut e) = init {
                            if matches!(e, Expr::InitList { .. }) {
                                self.report_error("变长数组(VLA)不支持初始化列表", loc);
                            }
                            // Ignore other initializers for VLA for now
                        }
                    } else if let Some(ref mut e) = init {
                        if vty.is_array() && matches!(e, Expr::InitList { .. }) {
                            if let Expr::InitList { ref mut elements, .. } = e {
                                let is_char_array = if let Type::Array { element, .. } = vty {
                                    element.kind() == TypeKind::Char
                                } else {
                                    false
                                };
                                if is_char_array {
                                    let values = super::flatten_init_list(elements, &mut self.errors);
                                    let base_temp = self.get_temp_slot(0);
                                    self.emit(OpCode::GetFrameBase, 0, loc);
                                    self.emit(OpCode::PushConst, local_offset, loc);
                                    self.emit(OpCode::Add, 0, loc);
                                    self.emit(OpCode::StoreLocal, base_temp, loc);
                                    let byte_count = vty.array_size() as usize;
                                    for i in 0..byte_count {
                                        self.emit(OpCode::LoadLocal, base_temp, loc);
                                        self.emit(OpCode::PushConst, i as i32, loc);
                                        self.emit(OpCode::Add, 0, loc);
                                        let byte = values.get(i).copied().unwrap_or(0);
                                        self.emit(OpCode::PushConst, byte, loc);
                                        self.emit(OpCode::StoreMemByte, 0, loc);
                                    }
                                } else {
                                    let base_temp = self.get_temp_slot(0);
                                    self.emit(OpCode::GetFrameBase, 0, loc);
                                    self.emit(OpCode::PushConst, local_offset, loc);
                                    self.emit(OpCode::Add, 0, loc);
                                    self.emit(OpCode::StoreLocal, base_temp, loc);
                                    let inner_ty = vty.subscript_type();
                                    let has_nested_init =
                                        elements.iter().any(|e| matches!(&e.value, Expr::InitList { .. }));
                                    if has_nested_init && (inner_ty.is_struct() || inner_ty.is_array()) {
                                        // Nested struct/array init: each element is an inner_ty value
                                        let elem_stride = self.type_size(&inner_ty);
                                        for (i, elem) in elements.iter_mut().enumerate() {
                                            let addr_offset = (i as i32) * elem_stride;
                                            self.gen_nested_init(base_temp, addr_offset, &inner_ty, elem, loc);
                                        }
                                        // Zero-fill remaining elements
                                        let expected_count = if !vty.dims().is_empty() && vty.dims()[0] > 0 {
                                            vty.dims()[0] as usize
                                        } else {
                                            elements.len()
                                        };
                                        for (i, _) in (elements.len()..expected_count).enumerate() {
                                            let idx = elements.len() + i;
                                            let addr_offset = (idx as i32) * elem_stride;
                                            self.emit(OpCode::LoadLocal, base_temp, loc);
                                            if addr_offset > 0 {
                                                self.emit(OpCode::PushConst, addr_offset, loc);
                                                self.emit(OpCode::Add, 0, loc);
                                            }
                                            self.emit(OpCode::PushConst, 0, loc);
                                            if inner_ty.kind() == TypeKind::Double {
                                                self.emit(OpCode::StoreMemD, 0, loc);
                                            } else if inner_ty.kind() == TypeKind::LongLong {
                                                self.emit(OpCode::StoreMemQ, 0, loc);
                                            } else {
                                                self.emit(OpCode::StoreMem, 0, loc);
                                            }
                                        }
                                    } else {
                                        // Flat scalar init
                                        let elem_size = self.elem_type_size(vty);
                                        let count = vty.total_elements();
                                        let has_designators = elements.iter().any(|e| !e.designators.is_empty());
                                        if has_designators {
                                            // Zero-fill entire array before designated init
                                            self.emit(OpCode::PushConst, count * elem_size, loc);
                                            self.emit(OpCode::PushConst, 0, loc);
                                            self.emit(OpCode::LoadLocal, base_temp, loc);
                                            self.emit(OpCode::Memset, 0, loc);
                                            for elem in elements.iter_mut() {
                                                if elem.designators.is_empty() {
                                                    continue;
                                                }
                                                let idx = match &elem.designators[0] {
                                                    Designator::Index(idx_expr) => {
                                                        // Evaluate index expression at compile time if possible
                                                        if let Expr::Literal { value, .. } = idx_expr.as_ref() {
                                                            *value
                                                        } else {
                                                            self.report_error(
                                                                "数组 designated initializer 的索引必须是编译期常量",
                                                                loc,
                                                            );
                                                            continue;
                                                        }
                                                    }
                                                    _ => {
                                                        self.report_error(
                                                            "数组初始化只能使用 [index] 形式的 designator",
                                                            loc,
                                                        );
                                                        continue;
                                                    }
                                                };
                                                if idx < 0 || idx >= count {
                                                    self.report_error("数组 designated initializer 索引超出范围", loc);
                                                    continue;
                                                }
                                                let addr_offset = idx * elem_size;
                                                self.emit(OpCode::LoadLocal, base_temp, loc);
                                                if addr_offset > 0 {
                                                    self.emit(OpCode::PushConst, addr_offset, loc);
                                                    self.emit(OpCode::Add, 0, loc);
                                                }
                                                if elem_size == 8 {
                                                    self.gen_expr(&mut elem.value);
                                                    self.emit(OpCode::StoreMemD, 0, loc);
                                                } else {
                                                    self.gen_expr(&mut elem.value);
                                                    self.emit(OpCode::StoreMem, 0, loc);
                                                }
                                            }
                                        } else {
                                            let values = super::flatten_init_list(elements, &mut self.errors);
                                            for i in 0..count as usize {
                                                let addr_offset = (i as i32) * elem_size;
                                                self.emit(OpCode::LoadLocal, base_temp, loc);
                                                if addr_offset > 0 {
                                                    self.emit(OpCode::PushConst, addr_offset, loc);
                                                    self.emit(OpCode::Add, 0, loc);
                                                }
                                                if let Some(elem) = elements.get_mut(i) {
                                                    if elem_size == 8 {
                                                        self.gen_expr(elem);
                                                        self.emit(OpCode::StoreMemD, 0, loc);
                                                    } else if matches!(
                                                        &elem.value,
                                                        Expr::Identifier { .. } | Expr::StringLiteral { .. }
                                                    ) {
                                                        self.gen_expr(elem);
                                                        self.emit(OpCode::StoreMem, 0, loc);
                                                    } else {
                                                        let val = values.get(i).copied().unwrap_or(0);
                                                        self.emit(OpCode::PushConst, val, loc);
                                                        self.emit(OpCode::StoreMem, 0, loc);
                                                    }
                                                } else {
                                                    if elem_size == 8 {
                                                        self.emit(OpCode::PushConst, 0, loc);
                                                        self.emit(OpCode::CastI2D, 0, loc);
                                                        self.emit(OpCode::StoreMemD, 0, loc);
                                                    } else {
                                                        let val = values.get(i).copied().unwrap_or(0);
                                                        self.emit(OpCode::PushConst, val, loc);
                                                        self.emit(OpCode::StoreMem, 0, loc);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else if vty.is_struct() && matches!(e, Expr::InitList { .. }) {
                            if let Expr::InitList { ref mut elements, .. } = e {
                                let base_temp = self.get_temp_slot(0);
                                self.emit(OpCode::GetFrameBase, 0, loc);
                                self.emit(OpCode::PushConst, local_offset, loc);
                                self.emit(OpCode::Add, 0, loc);
                                self.emit(OpCode::StoreLocal, base_temp, loc);
                                let fields = self.struct_defs.get(vty.name()).cloned().unwrap_or_default();
                                let has_designators = elements.iter().any(|e| !e.designators.is_empty());
                                if has_designators {
                                    // Zero-fill entire struct before designated init
                                    self.emit(OpCode::PushConst, self.type_size(vty), loc);
                                    self.emit(OpCode::PushConst, 0, loc);
                                    self.emit(OpCode::LoadLocal, base_temp, loc);
                                    self.emit(OpCode::Memset, 0, loc);
                                    for elem in elements.iter_mut() {
                                        if elem.designators.is_empty() {
                                            continue;
                                        }
                                        let field_idx = match &elem.designators[0] {
                                            Designator::Field(name) => fields.iter().position(|f| &f.name == name),
                                            _ => {
                                                self.report_error("结构体初始化只能使用 .field 形式的 designator", loc);
                                                None
                                            }
                                        };
                                        let field_idx = match field_idx {
                                            Some(i) if i < fields.len() => i,
                                            _ => continue,
                                        };
                                        let offset =
                                            fields.iter().take(field_idx).map(|f| self.type_size(&f.ty)).sum::<i32>();
                                        if matches!(&elem.value, Expr::InitList { .. })
                                            && (fields[field_idx].ty.is_struct() || fields[field_idx].ty.is_array())
                                        {
                                            self.gen_nested_init(
                                                base_temp,
                                                offset,
                                                &fields[field_idx].ty,
                                                &mut elem.value,
                                                loc,
                                            );
                                        } else {
                                            self.emit(OpCode::LoadLocal, base_temp, loc);
                                            if offset > 0 {
                                                self.emit(OpCode::PushConst, offset, loc);
                                                self.emit(OpCode::Add, 0, loc);
                                            }
                                            self.gen_expr(&mut elem.value);
                                            if fields[field_idx].ty.kind() == TypeKind::Double {
                                                self.emit(OpCode::StoreMemD, 0, loc);
                                            } else if fields[field_idx].ty.kind() == TypeKind::LongLong {
                                                self.emit(OpCode::StoreMemQ, 0, loc);
                                            } else {
                                                self.emit(OpCode::StoreMem, 0, loc);
                                            }
                                        }
                                    }
                                } else {
                                    for (i, elem) in elements.iter_mut().enumerate() {
                                        if i >= fields.len() {
                                            break;
                                        }
                                        let offset = fields.iter().take(i).map(|f| self.type_size(&f.ty)).sum::<i32>();
                                        if matches!(&elem.value, Expr::InitList { .. })
                                            && (fields[i].ty.is_struct() || fields[i].ty.is_array())
                                        {
                                            self.gen_nested_init(base_temp, offset, &fields[i].ty, elem, loc);
                                        } else {
                                            self.emit(OpCode::LoadLocal, base_temp, loc);
                                            if offset > 0 {
                                                self.emit(OpCode::PushConst, offset, loc);
                                                self.emit(OpCode::Add, 0, loc);
                                            }
                                            self.gen_expr(elem);
                                            if fields[i].ty.kind() == TypeKind::Double {
                                                self.emit(OpCode::StoreMemD, 0, loc);
                                            } else if fields[i].ty.kind() == TypeKind::LongLong {
                                                self.emit(OpCode::StoreMemQ, 0, loc);
                                            } else {
                                                self.emit(OpCode::StoreMem, 0, loc);
                                            }
                                        }
                                    }
                                }
                            }
                        } else if vty.is_struct() || vty.is_class() {
                            // C++ 构造函数初始化语法：Type name(args);
                            // TypeChecker 已在 args 前插入 &name 作为 this 指针。
                            if let Expr::Call {
                                name: ctor_name,
                                args: ctor_args,
                                ..
                            } = e
                            {
                                if ctor_name.starts_with("__ctor__") {
                                    if let Type::Class { name: class_name, .. } = vty {
                                        // VM do_call pops args in parameter-declaration order.
                                        // Args already include this as the first parameter.
                                        for arg in ctor_args.iter_mut().rev() {
                                            // RValueRef arguments (e.g. std::move) must be passed
                                            // as the address of the source object.
                                            if arg.ty().is_rvalue_ref() {
                                                self.gen_addr(arg, loc);
                                            } else {
                                                self.gen_expr(arg);
                                            }
                                        }
                                        if let Some(&idx) = self.func_index.get(ctor_name) {
                                            self.emit(OpCode::Call, idx, loc);
                                        }
                                        self.record_class_var(name, local_offset, class_name);
                                    }
                                    return;
                                }
                            }
                            // Lambda 闭包：gen_lambda 在栈上推闭包对象地址，直接保存地址（不逐字段拷贝）
                            if matches!(e, Expr::Lambda { .. }) {
                                self.gen_expr(e);
                                self.emit(OpCode::StoreLocal, local_offset, loc);
                            } else if e.ty().is_rvalue_ref() || matches!(e, Expr::Move { .. }) {
                                // C++ implicit move ctor: call __ctor__{Class}__move when
                                // initializing from an rvalue (std::move or RValueRef).
                                if let Type::Class { name: class_name, .. } = vty {
                                    let move_ctor_name = format!("__ctor__{}__move", class_name);
                                    if self.func_index.contains_key(&move_ctor_name) {
                                        // VM do_call pops args in parameter-declaration order.
                                        // We must push them right-to-left so the first pop() gets 'this'.
                                        // other = source address (pushed first)
                                        self.gen_addr(e, loc);
                                        // this = &local_var (pushed second, popped first)
                                        self.emit(OpCode::GetFrameBase, 0, loc);
                                        self.emit(OpCode::PushConst, local_offset, loc);
                                        self.emit(OpCode::Add, 0, loc);
                                        if let Some(&idx) = self.func_index.get(&move_ctor_name) {
                                            self.emit(OpCode::Call, idx, loc);
                                        }
                                        self.record_class_var(name, local_offset, class_name);
                                    } else {
                                        self.gen_struct_copy_to_local(local_offset, e, loc);
                                    }
                                } else {
                                    self.gen_struct_copy_to_local(local_offset, e, loc);
                                }
                            } else {
                                self.gen_struct_copy_to_local(local_offset, e, loc);
                            }
                        } else if vty.is_array() && matches!(e, Expr::StringLiteral { .. }) {
                            if let Expr::StringLiteral { ref value, .. } = e {
                                let base_temp = self.get_temp_slot(0);
                                self.emit(OpCode::GetFrameBase, 0, loc);
                                self.emit(OpCode::PushConst, local_offset, loc);
                                self.emit(OpCode::Add, 0, loc);
                                self.emit(OpCode::StoreLocal, base_temp, loc);
                                let byte_count = vty.array_size() as usize;
                                for i in 0..byte_count {
                                    self.emit(OpCode::LoadLocal, base_temp, loc);
                                    self.emit(OpCode::PushConst, i as i32, loc);
                                    self.emit(OpCode::Add, 0, loc);
                                    let byte = if i < value.len() { value.as_bytes()[i] as i32 } else { 0 };
                                    self.emit(OpCode::PushConst, byte, loc);
                                    self.emit(OpCode::StoreMemByte, 0, loc);
                                }
                            }
                        } else if vty.is_reference() || vty.is_rvalue_ref() {
                            // C++ reference initialization: store address of initializer
                            self.gen_addr(e, loc);
                            self.emit(OpCode::StoreLocal, local_offset, loc);
                        } else {
                            self.gen_expr(e);
                            if vty.kind() == TypeKind::Float
                                && e.ty().kind() != TypeKind::Float
                                && e.ty().kind() != TypeKind::Double
                                && e.ty().kind() != TypeKind::LongLong
                            {
                                self.emit(OpCode::CastI2F, 0, loc);
                            }
                            if vty.kind() == TypeKind::Double
                                && e.ty().kind() != TypeKind::Float
                                && e.ty().kind() != TypeKind::Double
                                && e.ty().kind() != TypeKind::LongLong
                            {
                                self.emit(OpCode::CastI2D, 0, loc);
                            }
                            if vty.kind() == TypeKind::LongLong
                                && e.ty().kind() != TypeKind::LongLong
                                && e.ty().kind() != TypeKind::Double
                                && e.ty().kind() != TypeKind::Float
                            {
                                self.emit(OpCode::CastI2Q, 0, loc);
                            }
                            if vty.kind() == TypeKind::Double {
                                self.emit(OpCode::StoreLocalD, local_offset, loc);
                            } else if vty.kind() == TypeKind::LongLong {
                                self.emit(OpCode::StoreLocalQ, local_offset, loc);
                            } else {
                                self.emit(OpCode::StoreLocal, local_offset, loc);
                            }
                        }
                    } else {
                        // Zero-initialize
                        if sz == 0 {
                            // Nothing to do for zero-size types
                        } else if sz == 8 && vty.kind() == TypeKind::Double {
                            self.emit(OpCode::PushConst, 0, loc);
                            self.emit(OpCode::CastI2D, 0, loc);
                            self.emit(OpCode::StoreLocalD, local_offset, loc);
                        } else if sz <= 4 {
                            self.emit(OpCode::PushConst, 0, loc);
                            self.emit(OpCode::StoreLocal, local_offset, loc);
                        } else {
                            let base_temp = self.get_temp_slot(0);
                            self.emit(OpCode::GetFrameBase, 0, loc);
                            self.emit(OpCode::PushConst, local_offset, loc);
                            self.emit(OpCode::Add, 0, loc);
                            self.emit(OpCode::StoreLocal, base_temp, loc);
                            for i in 0..sz {
                                self.emit(OpCode::LoadLocal, base_temp, loc);
                                self.emit(OpCode::PushConst, i, loc);
                                self.emit(OpCode::Add, 0, loc);
                                self.emit(OpCode::PushConst, 0, loc);
                                self.emit(OpCode::StoreMemByte, 0, loc);
                            }
                        }
                        // C++ 栈对象 RAII：对 class 类型调用默认构造函数
                        if vty.is_class() {
                            if let Type::Class { name: class_name, .. } = vty {
                                self.record_class_var(name, local_offset, class_name);
                                self.emit_class_default_ctor(class_name, local_offset, loc);
                            }
                        }
                    }
                };
                emit_one(var_type, name, init, loc, *is_static);
                for (ety, ename, einit) in extra_vars.iter_mut() {
                    emit_one(ety, ename, einit, loc, *is_static);
                }
            }
            Stmt::Expr { expr, .. } => {
                self.gen_expr(expr);
                if !expr.ty().is_void() && !expr.ty().is_struct() {
                    self.emit(OpCode::Pop, 0, &loc);
                }
            }
            Stmt::If {
                cond,
                then_stmt,
                else_stmt,
                loc,
            } => {
                self.gen_expr(cond);
                let else_jump = self.current_ip();
                self.emit(OpCode::JumpIfZero, 0, loc);
                self.gen_stmt(then_stmt);
                let skip_else_jump = self.current_ip();
                self.emit(OpCode::Jump, 0, loc);
                let else_ip = self.current_ip();
                self.patch_jump(else_jump, else_ip);
                if let Some(ref mut e) = else_stmt {
                    self.gen_stmt(e);
                }
                let end_ip = self.current_ip();
                self.patch_jump(skip_else_jump, end_ip);
            }
            Stmt::While { cond, body, loc } => {
                let start_ip = self.current_ip();
                self.gen_expr(cond);
                let end_jump = self.current_ip();
                self.emit(OpCode::JumpIfZero, 0, loc);
                self.loop_start_ips.push(start_ip);
                self.loop_scope_depths.push(self.local_scope_stack.len());
                let break_base = self.break_patches.len();
                let continue_base = self.continue_patches.len();
                self.gen_stmt(body);
                self.emit(OpCode::Jump, start_ip as i32, loc);
                let end_ip = self.current_ip();
                self.patch_jump(end_jump, end_ip);
                for i in break_base..self.break_patches.len() {
                    self.patch_jump(self.break_patches[i], end_ip);
                }
                self.break_patches.resize(break_base, 0);
                for i in continue_base..self.continue_patches.len() {
                    self.patch_jump(self.continue_patches[i], start_ip);
                }
                self.continue_patches.resize(continue_base, 0);
                self.loop_start_ips.pop();
                self.loop_scope_depths.pop();
            }
            Stmt::DoWhile { body, cond, loc } => {
                let start_ip = self.current_ip();
                self.loop_start_ips.push(start_ip);
                self.loop_scope_depths.push(self.local_scope_stack.len());
                let break_base = self.break_patches.len();
                let continue_base = self.continue_patches.len();
                self.gen_stmt(body);
                let cond_ip = self.current_ip();
                self.gen_expr(cond);
                self.emit(OpCode::JumpIfNotZero, start_ip as i32, loc);
                let end_ip = self.current_ip();
                for i in break_base..self.break_patches.len() {
                    self.patch_jump(self.break_patches[i], end_ip);
                }
                self.break_patches.resize(break_base, 0);
                for i in continue_base..self.continue_patches.len() {
                    self.patch_jump(self.continue_patches[i], cond_ip);
                }
                self.continue_patches.resize(continue_base, 0);
                self.loop_start_ips.pop();
                self.loop_scope_depths.pop();
            }
            Stmt::For { init, cond, step, body, loc } => {
                self.enter_scope();
                if let Some(ref mut i) = init {
                    self.gen_stmt(i);
                }
                let start_ip = self.current_ip();
                let mut cond_jump = 0;
                if let Some(ref mut c) = cond {
                    self.gen_expr(c);
                    cond_jump = self.current_ip();
                    self.emit(OpCode::JumpIfZero, 0, loc);
                }
                self.loop_start_ips.push(start_ip);
                self.loop_scope_depths.push(self.local_scope_stack.len());
                let break_base = self.break_patches.len();
                let continue_base = self.continue_patches.len();
                self.gen_stmt(body);
                let continue_ip = self.current_ip();
                for s in step {
                    self.gen_expr(s);
                    self.emit(OpCode::Pop, 0, loc);
                }
                self.emit(OpCode::Jump, start_ip as i32, loc);
                let end_ip = self.current_ip();
                self.exit_scope();
                if cond.is_some() {
                    self.patch_jump(cond_jump, end_ip);
                }
                for i in break_base..self.break_patches.len() {
                    self.patch_jump(self.break_patches[i], end_ip);
                }
                self.break_patches.resize(break_base, 0);
                for i in continue_base..self.continue_patches.len() {
                    self.patch_jump(self.continue_patches[i], continue_ip);
                }
                self.continue_patches.resize(continue_base, 0);
                self.loop_start_ips.pop();
                self.loop_scope_depths.pop();
            }
            Stmt::Return { value, loc } => {
                if let Some(ref mut v) = value {
                    let ret_is_struct = self
                        .func_table
                        .get(&self.current_func)
                        .map(|m| m.return_type.is_struct())
                        .unwrap_or(false);
                    if ret_is_struct {
                        let ret_ptr_offset = self.resolve_local("__ret_ptr");
                        let size = self.type_size(v.ty());
                        if size > 0 {
                            let src_temp = self.get_temp_slot(0);
                            self.gen_addr(v, loc);
                            self.emit(OpCode::StoreLocal, src_temp, loc);
                            for i in 0..size / 4 {
                                self.emit(OpCode::LoadLocal, ret_ptr_offset, loc);
                                if i > 0 {
                                    self.emit(OpCode::PushConst, i * 4, loc);
                                    self.emit(OpCode::Add, 0, loc);
                                }
                                self.emit(OpCode::LoadLocal, src_temp, loc);
                                if i > 0 {
                                    self.emit(OpCode::PushConst, i * 4, loc);
                                    self.emit(OpCode::Add, 0, loc);
                                }
                                self.emit(OpCode::LoadMem, 0, loc);
                                self.emit(OpCode::StoreMem, 0, loc);
                            }
                        }
                        self.emit(OpCode::RetVoid, 0, loc);
                    } else {
                        let ret_is_ref = self
                            .func_table
                            .get(&self.current_func)
                            .map(|m| m.return_type.is_reference() || m.return_type.is_rvalue_ref())
                            .unwrap_or(false);
                        if ret_is_ref {
                            self.gen_addr(v, loc);
                        } else {
                            self.gen_expr(v);
                            let ret_is_float = self
                                .func_table
                                .get(&self.current_func)
                                .map(|m| {
                                    m.return_type.kind() == TypeKind::Float || m.return_type.kind() == TypeKind::Double
                                })
                                .unwrap_or(false);
                            if ret_is_float && v.ty().kind() != TypeKind::Float && v.ty().kind() != TypeKind::Double {
                                self.emit(OpCode::CastI2F, 0, loc);
                            } else if !ret_is_float
                                && (v.ty().kind() == TypeKind::Float || v.ty().kind() == TypeKind::Double)
                            {
                                self.emit(OpCode::CastF2I, 0, loc);
                            }
                        }
                        // C++ 栈对象 RAII：return 前按 LIFO 调用所有活跃 scope 的析构函数
                        self.emit_dtors_for_scope_exit(0, loc);
                        self.emit(OpCode::Ret, 0, loc);
                    }
                } else {
                    // C++ 栈对象 RAII：return 前按 LIFO 调用所有活跃 scope 的析构函数
                    self.emit_dtors_for_scope_exit(0, loc);
                    self.emit(OpCode::RetVoid, 0, loc);
                }
            }
            Stmt::Break { loc } => {
                // C++ 栈对象 RAII：break 前调用当前 loop 内部 scope 的析构函数
                let target_depth = self.loop_scope_depths.last().copied().unwrap_or(1);
                self.emit_dtors_for_scope_exit(target_depth, loc);
                let ip = self.current_ip();
                self.emit(OpCode::Jump, 0, loc);
                self.break_patches.push(ip);
            }
            Stmt::Continue { loc } => {
                // C++ 栈对象 RAII：continue 前调用当前 loop 内部 scope 的析构函数
                let target_depth = self.loop_scope_depths.last().copied().unwrap_or(1);
                self.emit_dtors_for_scope_exit(target_depth, loc);
                let ip = self.current_ip();
                self.emit(OpCode::Jump, 0, loc);
                self.continue_patches.push(ip);
            }
            Stmt::Switch { cond, body, loc } => {
                self.gen_switch(cond, body, loc);
            }
            Stmt::Case { .. } => {}
            Stmt::Goto { label, loc } => {
                if let Some(&target_ip) = self.label_ips.get(label) {
                    self.emit(OpCode::Jump, target_ip as i32, loc);
                } else {
                    let ip = self.current_ip();
                    self.emit(OpCode::Jump, 0, loc);
                    self.goto_patches.entry(label.clone()).or_default().push(ip);
                }
            }
            Stmt::Label { label, stmt, .. } => {
                let ip = self.current_ip();
                self.label_ips.insert(label.clone(), ip);
                // Patch any pending gotos to this label
                if let Some(patches) = self.goto_patches.remove(label) {
                    for patch_ip in patches {
                        self.patch_jump(patch_ip, ip);
                    }
                }
                self.gen_stmt(stmt);
            }
            // === C++ 新增 (Phase 33) ===
            Stmt::RangeFor { var, var_type, iter, body, .. } => {
                let iter_ty = iter.ty().clone();
                let is_container = matches!(&iter_ty, Type::Class { name, .. } if crate::compiler::cpp_frontend::type_map::is_builtin_container(name) || name.starts_with("cide_vec_") || name == "cide_string" || name == "cide_list_int");
                let is_array = matches!(&iter_ty, Type::Array { .. });
                if !is_array && !is_container {
                    self.report_error("RangeFor 目前只支持数组和内置容器类型", &loc);
                    return;
                }
                self.enter_scope();
                // Index temp
                let idx_offset = self.next_local_offset;
                self.next_local_offset += 4;
                self.emit(OpCode::PushConst, 0, &loc);
                self.emit(OpCode::StoreLocal, idx_offset, &loc);
                // Loop variable
                let var_sz = (self.type_size(var_type) + 3) & !3;
                let var_offset = self.next_local_offset;
                self.next_local_offset += var_sz;
                self.local_indices.insert(var.clone(), var_offset);
                self.local_types.insert(var.clone(), var_type.clone());

                let start_ip = self.current_ip();
                self.loop_start_ips.push(start_ip);
                self.loop_scope_depths.push(self.local_scope_stack.len());
                let break_base = self.break_patches.len();
                let continue_base = self.continue_patches.len();

                // Condition: idx < count
                self.emit(OpCode::LoadLocal, idx_offset, &loc);
                if is_array {
                    let elem_count = if let Type::Array { array_size, .. } = &iter_ty {
                        *array_size
                    } else {
                        0
                    };
                    self.emit(OpCode::PushConst, elem_count, &loc);
                } else {
                    // Container: call cide_vec_size_*(&iter)
                    let size_func = match iter_ty.name() {
                        "cide_vec_int" => "cide_vec_size_int",
                        "cide_vec_float" => "cide_vec_size_float",
                        "cide_vec_char" => "cide_vec_size_char",
                        "cide_string" => "cide_string_size",
                        "cide_list_int" => "cide_list_size_int",
                        _ => {
                            self.report_error("RangeFor: 不支持的内置容器类型", &loc);
                            self.exit_scope();
                            return;
                        }
                    };
                    if let Some(&idx) = self.func_index.get(size_func) {
                        // Push &iter
                        if let Expr::Identifier { name, .. } = iter.as_ref() {
                            if let Some(&offset) = self.local_indices.get(name) {
                                self.emit(OpCode::GetFrameBase, 0, &loc);
                                self.emit(OpCode::PushConst, offset, &loc);
                                self.emit(OpCode::Add, 0, &loc);
                            } else if let Some(&offset) = self.global_indices.get(name) {
                                self.emit(OpCode::PushConst, crate::vm::vm::GLOBAL_START as i32 + offset, &loc);
                            } else {
                                self.report_error("RangeFor: 未声明的容器变量", &loc);
                                self.exit_scope();
                                return;
                            }
                        } else {
                            self.report_error("RangeFor: 复杂的迭代表达式暂不支持", &loc);
                            self.exit_scope();
                            return;
                        }
                        self.emit(OpCode::Call, idx, &loc);
                    } else {
                        self.report_error(&format!("RangeFor: 未找到容器函数 '{}'", size_func), &loc);
                        self.exit_scope();
                        return;
                    }
                }
                self.emit(OpCode::Lt, 0, &loc);
                let cond_jump = self.current_ip();
                self.emit(OpCode::JumpIfZero, 0, &loc);

                // Load element: var = iter[idx]
                if is_array {
                    let elem_ty = if let Type::Array { element, .. } = &iter_ty {
                        element.clone()
                    } else {
                        Box::new(Type::int())
                    };
                    let elem_sz = self.type_size(&elem_ty);
                    if let Expr::Identifier { name, .. } = iter.as_ref() {
                        if let Some(&offset) = self.local_indices.get(name) {
                            self.emit(OpCode::GetFrameBase, 0, &loc);
                            self.emit(OpCode::PushConst, offset, &loc);
                            self.emit(OpCode::Add, 0, &loc);
                        } else if let Some(&offset) = self.global_indices.get(name) {
                            self.emit(OpCode::PushConst, crate::vm::vm::GLOBAL_START as i32 + offset, &loc);
                        } else {
                            self.report_error("RangeFor: 未声明的数组变量", &loc);
                            self.exit_scope();
                            return;
                        }
                    } else {
                        self.report_error("RangeFor: 复杂的迭代表达式暂不支持", &loc);
                        self.exit_scope();
                        return;
                    }
                    self.emit(OpCode::LoadLocal, idx_offset, &loc);
                    self.emit(OpCode::PushConst, elem_sz, &loc);
                    self.emit(OpCode::Mul, 0, &loc);
                    self.emit(OpCode::Add, 0, &loc);
                    self.emit(OpCode::LoadMem, 0, &loc);
                    self.emit(OpCode::StoreLocal, var_offset, &loc);
                } else {
                    // Container: call cide_vec_get_*(&iter, idx)
                    let get_func = match iter_ty.name() {
                        "cide_vec_int" => "cide_vec_get_int",
                        "cide_vec_float" => "cide_vec_get_float",
                        "cide_vec_char" => "cide_vec_get_char",
                        "cide_string" => "cide_string_get",
                        "cide_list_int" => "cide_list_get_int",
                        _ => {
                            self.report_error("RangeFor: 不支持的内置容器类型", &loc);
                            self.exit_scope();
                            return;
                        }
                    };
                    if let Some(&idx) = self.func_index.get(get_func) {
                        // Push idx first (will be second on stack after &iter)
                        self.emit(OpCode::LoadLocal, idx_offset, &loc);
                        // Push &iter
                        if let Expr::Identifier { name, .. } = iter.as_ref() {
                            if let Some(&offset) = self.local_indices.get(name) {
                                self.emit(OpCode::GetFrameBase, 0, &loc);
                                self.emit(OpCode::PushConst, offset, &loc);
                                self.emit(OpCode::Add, 0, &loc);
                            } else if let Some(&offset) = self.global_indices.get(name) {
                                self.emit(OpCode::PushConst, crate::vm::vm::GLOBAL_START as i32 + offset, &loc);
                            } else {
                                self.report_error("RangeFor: 未声明的容器变量", &loc);
                                self.exit_scope();
                                return;
                            }
                        } else {
                            self.report_error("RangeFor: 复杂的迭代表达式暂不支持", &loc);
                            self.exit_scope();
                            return;
                        }
                        self.emit(OpCode::Call, idx, &loc);
                        self.emit(OpCode::StoreLocal, var_offset, &loc);
                    } else {
                        self.report_error(&format!("RangeFor: 未找到容器函数 '{}'", get_func), &loc);
                        self.exit_scope();
                        return;
                    }
                }

                self.gen_stmt(body);

                // Continue: ++idx
                let continue_ip = self.current_ip();
                self.emit(OpCode::LoadLocal, idx_offset, &loc);
                self.emit(OpCode::PushConst, 1, &loc);
                self.emit(OpCode::Add, 0, &loc);
                self.emit(OpCode::StoreLocal, idx_offset, &loc);
                self.emit(OpCode::Jump, start_ip as i32, &loc);

                let end_ip = self.current_ip();
                self.exit_scope();
                self.patch_jump(cond_jump, end_ip);
                for i in break_base..self.break_patches.len() {
                    self.patch_jump(self.break_patches[i], end_ip);
                }
                self.break_patches.resize(break_base, 0);
                for i in continue_base..self.continue_patches.len() {
                    self.patch_jump(self.continue_patches[i], continue_ip);
                }
                self.continue_patches.resize(continue_base, 0);
                self.loop_start_ips.pop();
                self.loop_scope_depths.pop();
            }
            Stmt::Try { .. } => {
                self.report_error("Try/Catch 语句代码生成尚未实现（VM 不支持异常）", &loc);
            }
        }
    }

    fn gen_switch(&mut self, cond: &mut Expr, body: &mut Stmt, loc: &SourceLoc) {
        let mut cases: Vec<(Option<Expr>, Box<Stmt>)> = Vec::new();
        let mut default_case: Option<Box<Stmt>> = None;

        fn collect_cases(stmt: &mut Stmt, cases: &mut Vec<(Option<Expr>, Box<Stmt>)>, default: &mut Option<Box<Stmt>>) {
            match stmt {
                Stmt::Block { stmts, .. } => {
                    for s in stmts {
                        collect_cases(s, cases, default);
                    }
                }
                Stmt::Case { label, stmt, .. } => {
                    if label.is_some() {
                        cases.push((label.take(), stmt.clone()));
                    } else {
                        *default = Some(stmt.clone());
                    }
                }
                _ => {}
            }
        }

        collect_cases(body, &mut cases, &mut default_case);

        if cases.is_empty() && default_case.is_none() {
            self.gen_expr(cond);
            self.emit(OpCode::Pop, 0, loc);
            return;
        }

        self.gen_expr(cond);
        let cond_temp = self.get_temp_slot(0);
        self.emit(OpCode::StoreLocal, cond_temp, loc);

        let mut case_jump_ips = Vec::new();
        for (label, _) in &mut cases {
            self.emit(OpCode::LoadLocal, cond_temp, loc);
            if let Some(ref mut l) = label {
                self.gen_expr(l);
                if l.loc().line > 0 {
                    self.emit(OpCode::StepEvent, l.loc().line, l.loc());
                }
            }
            self.emit(OpCode::Eq, 0, loc);
            let jump_ip = self.current_ip();
            self.emit(OpCode::JumpIfNotZero, 0, loc);
            case_jump_ips.push(jump_ip);
        }

        let default_or_end_jump = self.current_ip();
        self.emit(OpCode::Jump, 0, loc);
        let break_base = self.break_patches.len();

        for (i, (_, ref mut stmt)) in cases.iter_mut().enumerate() {
            self.patch_jump(case_jump_ips[i], self.current_ip());
            self.gen_stmt(stmt);
        }

        if let Some(ref mut d) = default_case {
            self.patch_jump(default_or_end_jump, self.current_ip());
            self.gen_stmt(d);
        } else {
            self.patch_jump(default_or_end_jump, self.current_ip());
        }

        let end_ip = self.current_ip();
        for i in break_base..self.break_patches.len() {
            self.patch_jump(self.break_patches[i], end_ip);
        }
        self.break_patches.resize(break_base, 0);
    }
}
