//! BytecodeGen 函数进入/退出相关方法。

use crate::compiler::ast::*;
use crate::shared::Symbol;

use super::BytecodeGen;

impl BytecodeGen {
    pub(crate) fn enter_function(&mut self, name: &str, params: &[Param]) {
        self.current_func = name.to_string();
        self.local_indices.clear();
        self.local_types.clear();
        self.goto_patches.clear();
        self.label_ips.clear();
        self.local_scope_stack.clear();
        self.next_local_offset = 0;
        let mut offset = 0;
        let mut param_sizes = Vec::new();
        let returns_struct = self
            .func_table
            .get(name)
            .map(|m| m.return_type.is_struct() || m.return_type.is_class())
            .unwrap_or(false);
        if returns_struct {
            param_sizes.push(1);
            self.local_indices.insert("__ret_ptr".to_string(), offset);
            self.local_types.insert("__ret_ptr".to_string(), Type::pointer_to(Type::int()));
            offset += 4;
        }
        for p in params.iter() {
            let sz = self.type_size(&p.ty);
            let aligned_sz = (sz + 3) & !3;
            let words = (sz + 3) / 4;
            param_sizes.push(words);
            self.local_indices.insert(p.name.clone(), offset);
            self.local_types.insert(p.name.clone(), p.ty.clone());
            self.sym_index.insert(p.name.clone(), self.symbols.len() as i32);
            self.symbols.push(Symbol {
                name: p.name.clone(),
                addr: offset as u32,
                is_local: true,
                ty: p.ty.clone(),
                scope_depth: 1,
                func_name: self.current_func.clone(),
            });
            offset += aligned_sz;
        }
        self.next_local_offset = offset;
        self.current_func_arg_bytes = offset;
        self.current_func_arg_count = params.len() as i32;
        if returns_struct {
            self.current_func_arg_count += 1;
        }
        self.temp_slot0 = -1;
        self.temp_slot1 = -1;
        self.temp_slot2 = -1;
        self.temp_slot3 = -1;
        if let Some(meta) = self.func_table.get_mut(name) {
            meta.param_sizes = param_sizes;
        }
    }

    pub(crate) fn exit_function(&mut self) {
        if !self.current_func.is_empty() {
            if let Some(meta) = self.func_table.get_mut(&self.current_func) {
                meta.local_count = self.next_local_offset;
                // arg_count = 参数总 word 数（供 Call 指令弹栈）
                meta.arg_count = meta.param_sizes.iter().sum();
                // param_count = 参数个数（供 call_user_function 使用）
                meta.param_count = meta.param_sizes.len() as i32;
            }
        }
        self.current_func.clear();
        self.local_indices.clear();
        self.local_types.clear();
        self.local_scope_stack.clear();
    }
}
