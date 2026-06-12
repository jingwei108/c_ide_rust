use super::*;
use crate::compiler::codegen::expr::ExprGen;

impl BytecodeGen {
    pub(crate) fn gen_lambda(&mut self, expr: &mut Expr, loc: &SourceLoc) {
        let Expr::Lambda { unique_id, capture, .. } = expr else {
            self.report_error("gen_lambda 期望 Lambda 表达式", loc);
            self.emit(OpCode::PushConst, 0, loc);
            return;
        };

        let lambda_name = format!("__lambda_{}", unique_id);
        let mut by_ref_fields = std::collections::HashSet::new();
        for cap in capture.iter() {
            if let CaptureMode::ByReference(name) = cap {
                by_ref_fields.insert(name.clone());
            }
        }
        self.lambda_by_ref_fields.insert(lambda_name.clone(), by_ref_fields);
        let class_decl = self.class_defs.get(&lambda_name);

        // Compute closure size and field offsets (no vptr for lambda)
        let mut class_size = 0i32;
        let mut field_offsets = Vec::new();
        if let Some(decl) = class_decl {
            for member in &decl.members {
                if let ClassMember::Field { name, ty, .. } = member {
                    field_offsets.push((name.clone(), class_size, ty.clone()));
                    class_size += self.type_size(ty);
                }
            }
        }
        class_size = (class_size + 3) & !3;

        // Allocate closure on stack as a temporary
        let closure_offset = self.next_local_offset;
        self.next_local_offset += class_size;

        // Initialize capture fields
        for (field_name, field_offset, _field_ty) in field_offsets {
            let cap_mode = capture.iter().find(|cap| match cap {
                CaptureMode::ByValue(n) | CaptureMode::ByReference(n) => n == &field_name,
                _ => false,
            });

            if let Some(cap) = cap_mode {
                // Compute destination address: frame_base + closure_offset + field_offset
                self.emit(OpCode::GetFrameBase, 0, loc);
                self.emit(OpCode::PushConst, closure_offset + field_offset, loc);
                self.emit(OpCode::Add, 0, loc);

                if matches!(cap, CaptureMode::ByReference(_)) {
                    // Store address of captured variable
                    if let Some(&local_offset) = self.local_indices.get(&field_name) {
                        self.emit(OpCode::GetFrameBase, 0, loc);
                        self.emit(OpCode::PushConst, local_offset, loc);
                        self.emit(OpCode::Add, 0, loc);
                    } else if let Some(&global_offset) = self.global_indices.get(&field_name) {
                        self.emit(OpCode::PushConst, crate::vm::vm::GLOBAL_START as i32 + global_offset, loc);
                    } else if let Some(&static_offset) = self.static_local_indices.get(&field_name) {
                        self.emit(OpCode::PushConst, crate::vm::vm::GLOBAL_START as i32 + static_offset, loc);
                    } else {
                        self.report_error(&format!("Lambda 捕获变量 '{}' 未找到", field_name), loc);
                        self.emit(OpCode::PushConst, 0, loc);
                    }
                } else {
                    // Store value of captured variable
                    let mut id_expr = Expr::Identifier {
                        name: field_name.clone(),
                        loc: *loc,
                        ty: Type::int(),
                    };
                    self.gen_expr(&mut id_expr);
                }

                self.emit(OpCode::StoreMem, 0, loc);
            }
        }

        // Push closure address
        self.emit(OpCode::GetFrameBase, 0, loc);
        self.emit(OpCode::PushConst, closure_offset, loc);
        self.emit(OpCode::Add, 0, loc);
    }
}
