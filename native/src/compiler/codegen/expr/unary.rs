use super::*;

fn gen_mem_inc_dec(gen: &mut BytecodeGen, is_inc: bool, is_pre: bool, step: i32, loc: &SourceLoc) {
    // stack top: address
    let addr_temp = gen.get_temp_slot(2);
    gen.emit(OpCode::StoreLocal, addr_temp, loc); // save address
    gen.emit(OpCode::LoadLocal, addr_temp, loc);
    gen.emit(OpCode::LoadMem, 0, loc); // read current value
    if !is_pre {
        gen.emit(OpCode::Dup, 0, loc); // keep old value for post
    }
    gen.emit(OpCode::PushConst, step, loc);
    gen.emit(if is_inc { OpCode::Add } else { OpCode::Sub }, 0, loc);
    let val_temp = gen.get_temp_slot(0);
    gen.emit(OpCode::StoreLocal, val_temp, loc); // save new value
    gen.emit(OpCode::LoadLocal, addr_temp, loc);
    gen.emit(OpCode::LoadLocal, val_temp, loc);
    gen.emit(OpCode::StoreMem, 0, loc); // write new value
    if is_pre {
        gen.emit(OpCode::LoadLocal, addr_temp, loc);
        gen.emit(OpCode::LoadMem, 0, loc); // return new value
    }
    // for post, old value is already on stack
}

pub(crate) fn gen_unary(gen: &mut BytecodeGen, expr: &mut Expr) {
    let loc = *expr.loc();
    if let Expr::Unary { op, operand, .. } = expr {
        match op {
            UnaryOp::Neg => {
                gen.gen_expr(operand);
                if operand.ty().kind() == TypeKind::Double {
                    gen.emit(OpCode::NegD, 0, &loc);
                } else if operand.ty().kind() == TypeKind::Float {
                    gen.emit(OpCode::NegF, 0, &loc);
                } else if operand.ty().kind() == TypeKind::LongLong {
                    gen.emit(OpCode::NegQ, 0, &loc);
                } else if operand.ty().is_unsigned() {
                    gen.emit(OpCode::UNeg, 0, &loc);
                } else {
                    gen.emit(OpCode::Neg, 0, &loc);
                }
            }
            UnaryOp::Not => {
                gen.gen_expr(operand);
                gen.emit(OpCode::Not, 0, &loc);
            }
            UnaryOp::BitNot => {
                gen.gen_expr(operand);
                gen.emit(OpCode::BitNot, 0, &loc);
            }
            UnaryOp::Addr => {
                match operand.as_mut() {
                    Expr::Identifier { name, .. } => {
                        if let Some(&offset) = gen.local_indices.get(name) {
                            gen.emit(OpCode::GetFrameBase, 0, &loc);
                            gen.emit(OpCode::PushConst, offset, &loc);
                            gen.emit(OpCode::Add, 0, &loc);
                        } else if let Some(&offset) = gen.static_local_indices.get(name) {
                            gen.emit(OpCode::PushConst, crate::vm::core::GLOBAL_START as i32 + offset, &loc);
                        } else if let Some(&offset) = gen.global_indices.get(name) {
                            gen.emit(OpCode::PushConst, crate::vm::core::GLOBAL_START as i32 + offset, &loc);
                        } else if let Some(&idx) = gen.func_index.get(name) {
                            // &func_name — 取函数地址
                            gen.emit(OpCode::PushConst, idx, &loc);
                        } else {
                            gen.report_error("取地址暂不支持此表达式", &loc);
                            gen.emit(OpCode::PushConst, 0, &loc);
                        }
                    }
                    Expr::Index { array, index, ty, .. } => {
                        gen.gen_index(array, index, ty, &loc, true);
                    }
                    Expr::Member { object, member, .. } => {
                        gen.gen_member_addr(object, member, &loc);
                    }
                    Expr::Unary {
                        op: UnaryOp::Deref,
                        operand: inner,
                        ..
                    } => {
                        gen.gen_expr(inner);
                    }
                    _ => {
                        gen.report_error("取地址暂不支持此表达式", &loc);
                        gen.emit(OpCode::PushConst, 0, &loc);
                    }
                }
            }
            UnaryOp::Deref => {
                gen.gen_expr(operand);
                let base_ty = if operand.ty().is_pointer() {
                    immediate_base_kind(operand.ty())
                } else {
                    TypeKind::Int
                };
                if base_ty == TypeKind::Function {
                    // Function pointer dereference: *fp yields the function itself,
                    // which immediately decays back to the same pointer. No load.
                } else if base_ty == TypeKind::Char {
                    gen.emit(OpCode::LoadMemByte, 0, &loc);
                } else if base_ty == TypeKind::Double {
                    gen.emit(OpCode::LoadMemD, 0, &loc);
                } else if base_ty == TypeKind::LongLong {
                    gen.emit(OpCode::LoadMemQ, 0, &loc);
                } else {
                    gen.emit(OpCode::LoadMem, 0, &loc);
                }
            }
            UnaryOp::PreInc | UnaryOp::PostInc | UnaryOp::PreDec | UnaryOp::PostDec => {
                let is_inc = matches!(op, UnaryOp::PreInc | UnaryOp::PostInc);
                let is_pre = matches!(op, UnaryOp::PreInc | UnaryOp::PreDec);
                match operand.as_mut() {
                    Expr::Identifier { name, .. } => {
                        let step = if let Some(ty) = gen.local_types.get(name) {
                            gen.ptr_step_size(ty)
                        } else if let Some(ty) = gen.global_types.get(name) {
                            gen.ptr_step_size(ty)
                        } else if let Some(ty) = gen.static_local_types.get(name) {
                            gen.ptr_step_size(ty)
                        } else {
                            1
                        };
                        if let Some(&static_idx) = gen.static_local_indices.get(name) {
                            gen.emit(OpCode::LoadGlobal, static_idx, &loc);
                            if !is_pre {
                                gen.emit(OpCode::Dup, 0, &loc);
                            }
                            gen.emit(OpCode::PushConst, step, &loc);
                            gen.emit(if is_inc { OpCode::Add } else { OpCode::Sub }, 0, &loc);
                            if is_pre {
                                gen.emit(OpCode::Dup, 0, &loc);
                            }
                            gen.emit(OpCode::StoreGlobal, static_idx, &loc);
                        } else {
                            let local_idx = gen.resolve_local(name);
                            if local_idx >= 0 {
                                gen.emit(OpCode::LoadLocal, local_idx, &loc);
                                if !is_pre {
                                    gen.emit(OpCode::Dup, 0, &loc);
                                }
                                gen.emit(OpCode::PushConst, step, &loc);
                                gen.emit(if is_inc { OpCode::Add } else { OpCode::Sub }, 0, &loc);
                                if is_pre {
                                    gen.emit(OpCode::Dup, 0, &loc);
                                }
                                gen.emit(OpCode::StoreLocal, local_idx, &loc);
                            } else {
                                let global_idx = gen.resolve_global(name);
                                if global_idx >= 0 {
                                    gen.emit(OpCode::LoadGlobal, global_idx, &loc);
                                    if !is_pre {
                                        gen.emit(OpCode::Dup, 0, &loc);
                                    }
                                    gen.emit(OpCode::PushConst, step, &loc);
                                    gen.emit(if is_inc { OpCode::Add } else { OpCode::Sub }, 0, &loc);
                                    if is_pre {
                                        gen.emit(OpCode::Dup, 0, &loc);
                                    }
                                    gen.emit(OpCode::StoreGlobal, global_idx, &loc);
                                } else {
                                    gen.report_error("自增/自减暂只支持简单变量", &loc);
                                    gen.emit(OpCode::PushConst, 0, &loc);
                                }
                            }
                        }
                    }
                    Expr::Index { array, index, ty, .. } => {
                        let result_ty = ty.clone();
                        let step = gen.ptr_step_size(ty);
                        gen.gen_index(array, index, &result_ty, &loc, true);
                        gen_mem_inc_dec(gen, is_inc, is_pre, step, &loc);
                    }
                    Expr::Member { object, member, ty, .. } => {
                        let step = gen.ptr_step_size(ty);
                        gen.gen_member_addr(object, member, &loc);
                        gen_mem_inc_dec(gen, is_inc, is_pre, step, &loc);
                    }
                    Expr::Unary {
                        op: UnaryOp::Deref,
                        operand: inner,
                        ..
                    } => {
                        let step = gen.ptr_step_size(inner.ty());
                        gen.gen_expr(inner);
                        gen_mem_inc_dec(gen, is_inc, is_pre, step, &loc);
                    }
                    _ => {
                        gen.report_error("自增/自减暂只支持简单变量", &loc);
                        gen.emit(OpCode::PushConst, 0, &loc);
                    }
                }
            }
        }
    }
}
