use crate::shared::SourceLoc;
use crate::vm::opcode::OpCode;

use super::super::{BytecodeGen, ClassVarEntry};

impl BytecodeGen {
    /// 记录当前 scope 中声明的类类型局部变量，供作用域退出时析构。
    pub(crate) fn record_class_var(&mut self, offset: i32, class_name: &str) {
        if let Some(frame) = self.local_scope_stack.last_mut() {
            frame.class_vars.push(ClassVarEntry {
                offset,
                class_name: class_name.to_string(),
            });
        }
    }

    /// 生成对栈上指定偏移处类对象的析构函数调用。
    pub(crate) fn emit_class_dtor(&mut self, class_name: &str, offset: i32, loc: &SourceLoc) {
        let dtor_name = format!("__dtor__{}", class_name);
        if let Some(&idx) = self.func_index.get(&dtor_name) {
            self.emit(OpCode::GetFrameBase, 0, loc);
            self.emit(OpCode::PushConst, offset, loc);
            self.emit(OpCode::Add, 0, loc);
            self.emit(OpCode::Call, idx, loc);
        }
    }

    /// 生成对栈上指定偏移处类对象的构造函数调用（无参默认构造函数）。
    pub(crate) fn emit_class_default_ctor(&mut self, class_name: &str, offset: i32, loc: &SourceLoc) {
        let ctor_name = format!("__ctor__{}", class_name);
        if let Some(&idx) = self.func_index.get(&ctor_name) {
            self.emit(OpCode::GetFrameBase, 0, loc);
            self.emit(OpCode::PushConst, offset, loc);
            self.emit(OpCode::Add, 0, loc);
            self.emit(OpCode::Call, idx, loc);
        }
    }

    /// 按从内到外的顺序，生成从当前 scope 向下退到 target_depth（包含 target_depth）之间
    /// 所有 scope 的析构函数调用。
    /// target_depth 是目标 scope 在 `local_scope_stack` 中的索引：
    /// - 0 表示函数最外层 block 之前的 scope（函数参数）
    /// - 1 表示函数最外层 block
    pub(crate) fn emit_dtors_for_scope_exit(&mut self, target_depth: usize, loc: &SourceLoc) {
        let current_depth = self.local_scope_stack.len();
        if current_depth == 0 || current_depth < target_depth {
            return;
        }
        // target_depth 是目标 scope 深度（0 表示函数参数层，不含在 local_scope_stack 中）。
        // 实际 frame 索引为 depth-1，因此有效的 frame 索引范围是 max(target_depth, 1)-1 ..= current_depth-1。
        let start_frame_idx = target_depth.saturating_sub(1);
        // 先收集所有需要析构的类变量信息，避免 borrow 冲突
        let mut dtors: Vec<(String, i32)> = Vec::new();
        for frame_idx in (start_frame_idx..current_depth).rev() {
            let frame = &self.local_scope_stack[frame_idx];
            for cv in frame.class_vars.iter().rev() {
                dtors.push((cv.class_name.clone(), cv.offset));
            }
        }
        for (class_name, offset) in dtors {
            self.emit_class_dtor(&class_name, offset, loc);
        }
    }
}
