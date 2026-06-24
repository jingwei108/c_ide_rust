use super::*;

pub(crate) fn gen_call(gen: &mut BytecodeGen, expr: &mut Expr) {
    let loc = *expr.loc();
    if let Expr::Call { name, args, ty, .. } = expr {
        let is_struct_ret = ty.is_struct() || ty.is_class();
        let ret_temp_offset = if is_struct_ret {
            let sz = (gen.type_size(ty) + 3) & !3;
            let offset = gen.next_local_offset;
            gen.next_local_offset += sz;
            Some(offset)
        } else {
            None
        };
        for arg in args.iter_mut().rev() {
            let arg_ty_kind = arg.ty().kind();
            let arg_ty = arg.ty();
            if arg_ty.is_struct() || arg_ty.is_class() {
                let is_lambda = arg_ty.is_class() && arg_ty.name().starts_with("__lambda_");
                let sz = gen.type_size(arg_ty);
                let words = (sz + 3) / 4;
                if let Expr::Identifier { name: arg_name, .. } = arg {
                    if let Some(&offset) = gen.local_indices.get(arg_name) {
                        if is_lambda {
                            // Lambda closure variables store the closure address, not the object itself.
                            gen.emit(OpCode::LoadLocal, offset, &loc);
                        } else {
                            for i in 0..words {
                                gen.emit(OpCode::LoadLocal, offset + i * 4, &loc);
                            }
                        }
                    } else if let Some(&offset) = gen.static_local_indices.get(arg_name) {
                        if is_lambda {
                            gen.emit(OpCode::LoadGlobal, offset, &loc);
                        } else {
                            for i in 0..words {
                                gen.emit(OpCode::LoadGlobal, offset + i * 4, &loc);
                            }
                        }
                    } else if let Some(&offset) = gen.global_indices.get(arg_name) {
                        if is_lambda {
                            gen.emit(OpCode::LoadGlobal, offset, &loc);
                        } else {
                            for i in 0..words {
                                gen.emit(OpCode::LoadGlobal, offset + i * 4, &loc);
                            }
                        }
                    } else if matches!(arg, Expr::Call { .. } | Expr::CallPtr { .. }) {
                        gen.gen_expr(arg);
                        let addr_temp = gen.get_temp_slot(0);
                        gen.emit(OpCode::StoreLocal, addr_temp, &loc);
                        for i in 0..words {
                            gen.emit(OpCode::LoadLocal, addr_temp, &loc);
                            if i > 0 {
                                gen.emit(OpCode::PushConst, i * 4, &loc);
                                gen.emit(OpCode::Add, 0, &loc);
                            }
                            gen.emit(OpCode::LoadMem, 0, &loc);
                        }
                    } else {
                        gen.gen_expr(arg);
                        for _ in 1..words {
                            gen.emit(OpCode::PushConst, 0, &loc);
                        }
                    }
                } else if matches!(arg, Expr::Call { .. } | Expr::CallPtr { .. }) {
                    gen.gen_expr(arg);
                    let addr_temp = gen.get_temp_slot(0);
                    gen.emit(OpCode::StoreLocal, addr_temp, &loc);
                    for i in 0..words {
                        gen.emit(OpCode::LoadLocal, addr_temp, &loc);
                        if i > 0 {
                            gen.emit(OpCode::PushConst, i * 4, &loc);
                            gen.emit(OpCode::Add, 0, &loc);
                        }
                        gen.emit(OpCode::LoadMem, 0, &loc);
                    }
                } else {
                    gen.gen_expr(arg);
                    for _ in 1..words {
                        gen.emit(OpCode::PushConst, 0, &loc);
                    }
                }
            } else if arg_ty.kind() == TypeKind::Double {
                gen.gen_expr(arg);
                if gen.func_index.contains_key(name) {
                    gen.emit(OpCode::SplitD, 0, &loc);
                }
            } else if arg_ty.kind() == TypeKind::LongLong {
                gen.gen_expr(arg);
                if gen.func_index.contains_key(name) {
                    gen.emit(OpCode::SplitQ, 0, &loc);
                }
            } else {
                gen.gen_expr(arg);
                if (name == "printf" || name == "fprintf") && arg_ty_kind == TypeKind::Float {
                    gen.emit(OpCode::CastF2D, 0, &loc);
                }
            }
        }
        if let Some(offset) = ret_temp_offset {
            gen.emit(OpCode::GetFrameBase, 0, &loc);
            gen.emit(OpCode::PushConst, offset, &loc);
            gen.emit(OpCode::Add, 0, &loc);
        }
        if let Some(&idx) = gen.func_index.get(name) {
            gen.emit(OpCode::Call, idx, &loc);
        } else {
            if let Some(host_id) = cide_runtime::host_func_id::by_user_name(name.as_str()) {
                gen.emit(OpCode::CallHost, host_id as i32, &loc);
            } else {
                gen.report_error(&format!("未定义的函数 '{}'", name), &loc);
                gen.emit(OpCode::PushConst, 0, &loc);
            }
        }
        if let Some(offset) = ret_temp_offset {
            gen.emit(OpCode::GetFrameBase, 0, &loc);
            gen.emit(OpCode::PushConst, offset, &loc);
            gen.emit(OpCode::Add, 0, &loc);
        }
    }
}

pub(crate) fn gen_call_ptr(gen: &mut BytecodeGen, expr: &mut Expr) {
    let loc = *expr.loc();
    if let Expr::CallPtr { callee, args, ty, .. } = expr {
        let is_struct_ret = ty.is_struct() || ty.is_class();
        // std::move(x) is a compile-time cast to RValueRef; no function call.
        if let Expr::Identifier { name, .. } = callee.as_ref() {
            if name == "std__move" && args.len() == 1 {
                gen.gen_expr(&mut args[0]);
                return;
            }
        }
        let ret_temp_offset = if is_struct_ret {
            let sz = (gen.type_size(ty) + 3) & !3;
            let offset = gen.next_local_offset;
            gen.next_local_offset += sz;
            Some(offset)
        } else {
            None
        };
        // Determine if this CallPtr will resolve to a user function call (needs SplitD/SplitQ)
        // or a host/built-in call (passes 64-bit values directly).
        let is_user_call = if let Expr::Identifier { name, .. } = callee.as_ref() {
            gen.func_index.contains_key(name) || gen.resolve_host_func_id(name) < 0
        } else {
            true // indirect calls are always user calls
        };
        for arg in args.iter_mut().rev() {
            let arg_ty = arg.ty().clone();
            if arg_ty.is_struct() || arg_ty.is_class() {
                let is_lambda = arg_ty.is_class() && arg_ty.name().starts_with("__lambda_");
                let sz = gen.type_size(&arg_ty);
                let words = (sz + 3) / 4;
                if let Expr::Identifier { name: arg_name, .. } = arg {
                    if let Some(&offset) = gen.local_indices.get(arg_name) {
                        if is_lambda {
                            gen.emit(OpCode::LoadLocal, offset, &loc);
                        } else {
                            for i in 0..words {
                                gen.emit(OpCode::LoadLocal, offset + i * 4, &loc);
                            }
                        }
                    } else if let Some(&offset) = gen.static_local_indices.get(arg_name) {
                        if is_lambda {
                            gen.emit(OpCode::LoadGlobal, offset, &loc);
                        } else {
                            for i in 0..words {
                                gen.emit(OpCode::LoadGlobal, offset + i * 4, &loc);
                            }
                        }
                    } else if let Some(&offset) = gen.global_indices.get(arg_name) {
                        if is_lambda {
                            gen.emit(OpCode::LoadGlobal, offset, &loc);
                        } else {
                            for i in 0..words {
                                gen.emit(OpCode::LoadGlobal, offset + i * 4, &loc);
                            }
                        }
                    } else if matches!(arg, Expr::Call { .. } | Expr::CallPtr { .. }) {
                        gen.gen_expr(arg);
                        let addr_temp = gen.get_temp_slot(0);
                        gen.emit(OpCode::StoreLocal, addr_temp, &loc);
                        for i in 0..words {
                            gen.emit(OpCode::LoadLocal, addr_temp, &loc);
                            if i > 0 {
                                gen.emit(OpCode::PushConst, i * 4, &loc);
                                gen.emit(OpCode::Add, 0, &loc);
                            }
                            gen.emit(OpCode::LoadMem, 0, &loc);
                        }
                    } else {
                        gen.gen_expr(arg);
                        for _ in 1..words {
                            gen.emit(OpCode::PushConst, 0, &loc);
                        }
                    }
                } else if matches!(arg, Expr::Call { .. } | Expr::CallPtr { .. }) {
                    gen.gen_expr(arg);
                    let addr_temp = gen.get_temp_slot(0);
                    gen.emit(OpCode::StoreLocal, addr_temp, &loc);
                    for i in 0..words {
                        gen.emit(OpCode::LoadLocal, addr_temp, &loc);
                        if i > 0 {
                            gen.emit(OpCode::PushConst, i * 4, &loc);
                            gen.emit(OpCode::Add, 0, &loc);
                        }
                        gen.emit(OpCode::LoadMem, 0, &loc);
                    }
                } else {
                    gen.gen_expr(arg);
                    for _ in 1..words {
                        gen.emit(OpCode::PushConst, 0, &loc);
                    }
                }
            } else if arg_ty.kind() == TypeKind::Double {
                gen.gen_expr(arg);
                if is_user_call {
                    gen.emit(OpCode::SplitD, 0, &loc);
                }
            } else if arg_ty.kind() == TypeKind::LongLong {
                gen.gen_expr(arg);
                if is_user_call {
                    gen.emit(OpCode::SplitQ, 0, &loc);
                }
            } else {
                gen.gen_expr(arg);
                if let Expr::Identifier { name, .. } = callee.as_ref() {
                    if (name == "printf" || name == "fprintf") && arg_ty.kind() == TypeKind::Float {
                        gen.emit(OpCode::CastF2D, 0, &loc);
                    }
                }
            }
        }
        if let Some(offset) = ret_temp_offset {
            gen.emit(OpCode::GetFrameBase, 0, &loc);
            gen.emit(OpCode::PushConst, offset, &loc);
            gen.emit(OpCode::Add, 0, &loc);
        }
        if let Expr::Identifier { name, .. } = callee.as_ref() {
            if let Some(&idx) = gen.func_index.get(name) {
                gen.emit(OpCode::Call, idx, &loc);
                if is_struct_ret {
                    // SAFETY: ret_temp_offset 在 is_struct_ret 时已在上方赋值。
                    #[allow(clippy::unwrap_used)]
                    gen.emit(OpCode::GetFrameBase, 0, &loc);
                    #[allow(clippy::unwrap_used)]
                    gen.emit(OpCode::PushConst, ret_temp_offset.unwrap(), &loc);
                    gen.emit(OpCode::Add, 0, &loc);
                }
                return;
            }
            // Host function: direct CallHost
            let host_id = gen.resolve_host_func_id(name);
            if host_id >= 0 {
                gen.emit(OpCode::CallHost, host_id, &loc);
                if is_struct_ret {
                    // SAFETY: ret_temp_offset 在 is_struct_ret 时已在上方赋值。
                    #[allow(clippy::unwrap_used)]
                    gen.emit(OpCode::GetFrameBase, 0, &loc);
                    #[allow(clippy::unwrap_used)]
                    gen.emit(OpCode::PushConst, ret_temp_offset.unwrap(), &loc);
                    gen.emit(OpCode::Add, 0, &loc);
                }
                return;
            }
        }
        gen.gen_expr(callee);
        gen.emit(OpCode::CallPtr, args.len() as i32, &loc);
        if is_struct_ret {
            // SAFETY: ret_temp_offset 在 is_struct_ret 时已在上方赋值。
            #[allow(clippy::unwrap_used)]
            gen.emit(OpCode::GetFrameBase, 0, &loc);
            #[allow(clippy::unwrap_used)]
            gen.emit(OpCode::PushConst, ret_temp_offset.unwrap(), &loc);
            gen.emit(OpCode::Add, 0, &loc);
        }
    }
}
