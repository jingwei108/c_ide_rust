use super::*;
use crate::compiler::codegen::expr::ExprGen;

impl BytecodeGen {
    pub(crate) fn gen_this(&mut self, _expr: &mut Expr, loc: &SourceLoc) {
        let this_offset = self.resolve_local("this");
        if this_offset >= 0 {
            self.emit(OpCode::LoadLocal, this_offset, loc);
        } else {
            self.report_error("this 在静态上下文中不可用", loc);
            self.emit(OpCode::PushConst, 0, loc);
        }
    }

    pub(crate) fn gen_new(&mut self, expr: &mut Expr, loc: &SourceLoc) {
        let Expr::New { elem_type, size_expr, init, .. } = expr else {
            self.report_error("gen_new 期望 New 表达式", loc);
            self.emit(OpCode::PushConst, 0, loc);
            return;
        };

        let elem_sz = self.type_size(elem_type);
        if let Some(size_expr) = size_expr {
            self.gen_expr(size_expr);
            self.emit(OpCode::PushConst, elem_sz, loc);
            self.emit(OpCode::Mul, 0, loc);
        } else {
            self.emit(OpCode::PushConst, elem_sz, loc);
        }
        self.emit(OpCode::CallHost, crate::vm::host_func_id::MALLOC as i32, loc);
        // If class type, initialize vptr and call constructor
        if let Type::Class { name, .. } = elem_type {
            let ptr_temp = self.get_temp_slot(0);
            self.emit(OpCode::StoreLocal, ptr_temp, loc);
            // Initialize vptr if class has a vtable
            if let Some(&vtable_offset) = self.class_vtables.get(name) {
                self.emit(OpCode::LoadLocal, ptr_temp, loc);
                self.emit(OpCode::PushConst, crate::vm::vm::GLOBAL_START as i32 + vtable_offset as i32, loc);
                self.emit(OpCode::StoreMem, 0, loc);
            }
            let ctor_name = format!("__ctor__{}", name);
            if self.func_index.contains_key(&ctor_name) {
                if let Some(init_expr) = init {
                    if let Expr::Call { args, .. } = init_expr.as_mut() {
                        for arg in args.iter_mut().rev() {
                            self.gen_expr(arg);
                        }
                    }
                }
                self.emit(OpCode::LoadLocal, ptr_temp, loc);
                if let Some(&idx) = self.func_index.get(&ctor_name) {
                    self.emit(OpCode::Call, idx, loc);
                }
                self.emit(OpCode::LoadLocal, ptr_temp, loc);
            } else {
                self.emit(OpCode::LoadLocal, ptr_temp, loc);
            }
        } else {
            // Non-class type: if init is present, store it directly
            if let Some(init_expr) = init {
                self.emit(OpCode::Dup, 0, loc);
                self.gen_expr(init_expr);
                self.emit(OpCode::StoreMem, 0, loc);
            }
        }
    }

    pub(crate) fn gen_delete(&mut self, expr: &mut Expr, loc: &SourceLoc) {
        let Expr::Delete { expr: inner, .. } = expr else {
            self.report_error("gen_delete 期望 Delete 表达式", loc);
            return;
        };

        self.gen_expr(inner);
        // If class type (or pointer to class), call destructor before free
        let (is_class, class_name) = if let Type::Class { name, .. } = inner.ty() {
            (true, name.clone())
        } else if let Type::Pointer { pointee, .. } = inner.ty() {
            if let Type::Class { name, .. } = pointee.as_ref() {
                (true, name.clone())
            } else {
                (false, String::new())
            }
        } else {
            (false, String::new())
        };
        if is_class {
            let dtor_name = format!("__dtor__{}", class_name);
            if let Some(&idx) = self.func_index.get(&dtor_name) {
                self.emit(OpCode::Dup, 0, loc);
                self.emit(OpCode::Call, idx, loc);
            }
        }
        self.emit(OpCode::CallHost, crate::vm::host_func_id::FREE as i32, loc);
    }

    pub(crate) fn gen_move(&mut self, expr: &mut Expr, _loc: &SourceLoc) {
        let Expr::Move { expr: inner, .. } = expr else {
            return;
        };
        self.gen_expr(inner);
    }
}
