use crate::compiler::ast::*;
use crate::vm::instruction::{Instruction, SourceLoc as VMSourceLoc};
use crate::vm::opcode::OpCode;
use crate::vm::vm::VMSymbol;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FuncMeta {
    pub ip: usize,
    pub arg_count: i32,
    pub local_count: i32,
    pub return_type: Type,
}

pub struct BytecodeGen {
    code: Vec<Instruction>,
    errors: Vec<String>,
    func_table: HashMap<String, FuncMeta>,
    func_index: HashMap<String, i32>,
    next_func_idx: i32,
    current_func: String,
    current_func_arg_count: i32,
    global_indices: HashMap<String, i32>,
    global_types: HashMap<String, Type>,
    local_indices: HashMap<String, i32>,
    local_types: HashMap<String, Type>,
    next_local_idx: i32,
    temp_slot0: i32,
    temp_slot1: i32,
    temp_slot2: i32,
    globals_init: Vec<i32>,
    next_global_idx: i32,
    symbols: Vec<VMSymbol>,
    sym_index: HashMap<String, i32>,
    struct_defs: HashMap<String, Vec<StructField>>,
    string_data: Vec<(u32, String)>,
    string_mem_offset: u32,
    source_map: Vec<(u32, VMSourceLoc)>,
    break_patches: Vec<usize>,
    continue_patches: Vec<usize>,
    loop_start_ips: Vec<usize>,
}

impl Default for BytecodeGen {
    fn default() -> Self {
        Self::new()
    }
}

impl BytecodeGen {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            errors: Vec::new(),
            func_table: HashMap::new(),
            func_index: HashMap::new(),
            next_func_idx: 1,
            current_func: String::new(),
            current_func_arg_count: 0,
            global_indices: HashMap::new(),
            global_types: HashMap::new(),
            local_indices: HashMap::new(),
            local_types: HashMap::new(),
            next_local_idx: 0,
            temp_slot0: -1,
            temp_slot1: -1,
            temp_slot2: -1,
            globals_init: Vec::new(),
            next_global_idx: 0,
            symbols: Vec::new(),
            sym_index: HashMap::new(),
            struct_defs: HashMap::new(),
            string_data: Vec::new(),
            string_mem_offset: 0x1000,
            source_map: Vec::new(),
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
            loop_start_ips: Vec::new(),
        }
    }

    pub fn generate(mut self, program: &mut ProgramNode) -> Result<CompileOutput, Vec<String>> {
        self.code.push(Instruction::new(OpCode::Nop, 0, VMSourceLoc::default()));

        for s in &program.structs {
            self.struct_defs.insert(s.name.clone(), s.fields.clone());
        }

        // Pass 1: Register globals
        for g in &program.globals {
            self.global_indices.insert(g.name.clone(), self.next_global_idx);
            self.global_types.insert(g.name.clone(), g.ty.clone());
            let mut elem_count = 1;
            if g.ty.is_array() {
                elem_count = g.ty.array_size;
            } else if g.ty.is_struct() {
                elem_count = self.type_size(&g.ty) / 4;
                if elem_count < 1 { elem_count = 1; }
            }
            if elem_count < 1 {
                if let Some(ref init) = g.init {
                    match init {
                        Expr::StringLiteral { value, .. } => elem_count = value.len() as i32 + 1,
                        Expr::InitList { elements, .. } => elem_count = elements.len() as i32,
                        _ => elem_count = 1,
                    }
                } else {
                    elem_count = 1;
                }
                if let Some(ty) = self.global_types.get_mut(&g.name) {
                    ty.array_size = elem_count;
                }
            }
            if let Some(ref init) = g.init {
                match init {
                    Expr::InitList { elements, .. } => {
                        let values = flatten_init_list(elements);
                        for i in 0..elem_count as usize {
                            self.globals_init.push(values.get(i).copied().unwrap_or(0));
                        }
                    }
                    Expr::StringLiteral { value, .. } => {
                        for i in 0..elem_count as usize {
                            if i < value.len() {
                                self.globals_init.push(value.as_bytes()[i] as i32);
                            } else {
                                self.globals_init.push(0);
                            }
                        }
                    }
                    Expr::Literal { value, .. } => {
                        self.globals_init.push(*value);
                        for _ in 1..elem_count {
                            self.globals_init.push(0);
                        }
                    }
                    Expr::FloatLiteral { value, .. } => {
                        let bits = (*value as f32).to_bits() as i32;
                        self.globals_init.push(bits);
                        for _ in 1..elem_count {
                            self.globals_init.push(0);
                        }
                    }
                    _ => {
                        for _ in 0..elem_count {
                            self.globals_init.push(0);
                        }
                    }
                }
            } else {
                for _ in 0..elem_count {
                    self.globals_init.push(0);
                }
            }
            let gi = self.global_indices[&g.name];
            self.sym_index.insert(g.name.clone(), self.symbols.len() as i32);
            self.symbols.push(VMSymbol {
                name: g.name.clone(),
                addr: 0x1000 + gi as u32 * 4,
                is_local: false,
                ty: g.ty.clone(),
                scope_depth: 0,
            });
            self.next_global_idx += elem_count;
        }

        self.string_mem_offset = 0x1000 + self.next_global_idx as u32 * 4;

        // Pass 2: Register function metadata
        for f in &program.funcs {
            if f.body.is_none() { continue; }
            self.func_index.insert(f.name.clone(), self.next_func_idx);
            self.next_func_idx += 1;
            let arg_slots: i32 = f.params.iter().map(|p| {
                if p.ty.is_array() { return 1; }
                let sz = (self.type_size(&p.ty) + 3) / 4;
                if sz < 1 { 1 } else { sz }
            }).sum();
            self.func_table.insert(f.name.clone(), FuncMeta {
                ip: 0,
                arg_count: arg_slots,
                local_count: 0,
                return_type: f.return_type.clone(),
            });
        }

        // Pass 3: Generate function bodies
        for f in &mut program.funcs {
            if f.body.is_none() { continue; }
            let func_ip = self.current_ip();
            if let Some(meta) = self.func_table.get_mut(&f.name) {
                meta.ip = func_ip;
            }
            self.enter_function(&f.name, &f.params);
            if let Some(ref mut body) = f.body {
                self.gen_stmt(body);
            }
            if f.return_type.is_void() {
                self.emit(OpCode::RetVoid, 0, &f.loc);
            } else {
                self.emit(OpCode::PushConst, 0, &f.loc);
                self.emit(OpCode::Ret, 0, &f.loc);
            }
            self.exit_function();
        }

        let wrapper_ip = self.current_ip();
        if let Some(&main_idx) = self.func_index.get("main") {
            self.emit(OpCode::Call, main_idx, &SourceLoc { line: 0, column: 0 });
            self.emit(OpCode::Ret, 0, &SourceLoc { line: 0, column: 0 });
            self.code[0] = Instruction::new(OpCode::Jump, wrapper_ip as i32, VMSourceLoc::default());
        } else {
            self.errors.push("缺少 main 函数入口".to_string());
        }

        if !self.errors.is_empty() {
            return Err(self.errors);
        }

        Ok(CompileOutput {
            code: self.code,
            globals_init: self.globals_init,
            func_table: self.func_table,
            func_index: self.func_index,
            string_data: self.string_data,
            source_map: self.source_map,
            symbols: self.symbols,
            struct_defs: self.struct_defs,
        })
    }

    fn emit(&mut self, op: OpCode, operand: i32, loc: &SourceLoc) {
        let ip = self.code.len() as u32;
        let vm_loc = VMSourceLoc { line: loc.line, column: loc.column };
        self.code.push(Instruction::new(op, operand, vm_loc));
        if loc.line > 0 {
            self.source_map.push((ip, vm_loc));
        }
    }

    fn current_ip(&self) -> usize { self.code.len() }

    fn patch_jump(&mut self, ip: usize, target: usize) {
        if ip < self.code.len() {
            self.code[ip].operand = target as i32;
        }
    }

    fn report_error(&mut self, msg: &str, loc: &SourceLoc) {
        self.errors.push(format!("第 {} 行：{}", loc.line, msg));
    }

    fn enter_function(&mut self, name: &str, params: &[Param]) {
        self.current_func = name.to_string();
        self.local_indices.clear();
        self.local_types.clear();
        self.next_local_idx = 0;
        let mut slot_offset = 0;
        for p in params.iter() {
            let slots = if p.ty.is_array() {
                1
            } else {
                let sz = (self.type_size(&p.ty) + 3) / 4;
                if sz < 1 { 1 } else { sz }
            };
            self.local_indices.insert(p.name.clone(), slot_offset);
            self.local_types.insert(p.name.clone(), p.ty.clone());
            self.sym_index.insert(p.name.clone(), self.symbols.len() as i32);
            self.symbols.push(VMSymbol {
                name: p.name.clone(),
                addr: slot_offset as u32 * 4,
                is_local: true,
                ty: p.ty.clone(),
                scope_depth: 1,
            });
            slot_offset += slots;
        }
        self.next_local_idx = slot_offset;
        self.current_func_arg_count = slot_offset;
        self.temp_slot0 = -1;
        self.temp_slot1 = -1;
        self.temp_slot2 = -1;
    }

    fn exit_function(&mut self) {
        if !self.current_func.is_empty() {
            if let Some(meta) = self.func_table.get_mut(&self.current_func) {
                meta.local_count = self.next_local_idx;
            }
        }
        self.current_func.clear();
        self.local_indices.clear();
        self.local_types.clear();
    }

    fn resolve_local(&self, name: &str) -> i32 {
        self.local_indices.get(name).copied().unwrap_or(-1)
    }

    fn resolve_global(&self, name: &str) -> i32 {
        self.global_indices.get(name).copied().unwrap_or(-1)
    }

    fn resolve_symbol_index(&self, name: &str) -> i32 {
        self.sym_index.get(name).copied().unwrap_or(-1)
    }

    fn get_temp_slot(&mut self, index: i32) -> i32 {
        let slot = match index {
            0 => &mut self.temp_slot0,
            1 => &mut self.temp_slot1,
            2 => &mut self.temp_slot2,
            _ => &mut self.temp_slot0,
        };
        if *slot < 0 {
            *slot = self.next_local_idx;
            self.next_local_idx += 1;
        }
        *slot
    }

    fn get_member_offset(&self, object_type: &Type, member_name: &str) -> i32 {
        let struct_name = match object_type.kind {
            TypeKind::Struct => &object_type.name,
            TypeKind::Pointer if object_type.base_kind == TypeKind::Struct => &object_type.name,
            _ => return 0,
        };
        let fields = match self.struct_defs.get(struct_name) {
            Some(f) => f,
            None => return 0,
        };
        let mut offset = 0;
        for field in fields {
            if field.name == member_name {
                return offset;
            }
            offset += self.type_size(&field.ty);
        }
        0
    }

    fn ptr_step_size(&self, ty: &Type) -> i32 {
        match ty.kind {
            TypeKind::Pointer => {
                match ty.base_kind {
                    TypeKind::Char => 1,
                    TypeKind::Int | TypeKind::Pointer => 4,
                    TypeKind::Struct => {
                        self.struct_defs.get(&ty.name).map(|f| {
                            f.iter().map(|field| self.type_size(&field.ty)).sum()
                        }).unwrap_or(4)
                    }
                    _ => 4,
                }
            }
            TypeKind::Array => compute_stride(ty, self.elem_type_size(ty)),
            _ => 1,
        }
    }

    fn elem_type_size(&self, arr_type: &Type) -> i32 {
        match arr_type.base_kind {
            TypeKind::Char => 1,
            TypeKind::Int | TypeKind::Pointer | TypeKind::Float => 4,
            TypeKind::Struct => {
                self.struct_defs.get(&arr_type.name).map(|f| {
                    f.iter().map(|field| self.type_size(&field.ty)).sum()
                }).unwrap_or(4)
            }
            _ => 4,
        }
    }

    fn type_size(&self, ty: &Type) -> i32 {
        match ty.kind {
            TypeKind::Void => 0,
            TypeKind::Int => 4,
            TypeKind::Char => 1,
            TypeKind::Float => 4,
            TypeKind::Pointer => 4,
            TypeKind::Array => {
                let elem_count = ty.total_elements();
                let elem_size = self.elem_type_size(ty);
                elem_count * elem_size
            }
            TypeKind::Struct => {
                self.struct_defs.get(&ty.name).map(|f| {
                    f.iter().map(|field| self.type_size(&field.ty)).sum()
                }).unwrap_or(0)
            }
        }
    }

    #[allow(dead_code)]
    fn elem_size(&self, ty: &Type) -> i32 {
        match ty.base_kind {
            TypeKind::Void => 4,
            TypeKind::Int => 4,
            TypeKind::Char => 1,
            TypeKind::Float => 4,
            TypeKind::Pointer => 4,
            TypeKind::Array => 4,
            TypeKind::Struct => {
                self.struct_defs.get(&ty.name).map(|f| {
                    f.iter().map(|field| self.type_size(&field.ty)).sum()
                }).unwrap_or(0)
            }
        }
    }

    // =====================================================================
    // Statement / Expression dispatch
    // =====================================================================

    fn gen_stmt(&mut self, stmt: &mut Stmt) {
        let loc = stmt_loc(stmt);
        if loc.line > 0 {
            self.emit(OpCode::StepEvent, loc.line, &loc);
        }
        match stmt {
            Stmt::Block { stmts, .. } => {
                for s in stmts { self.gen_stmt(s); }
            }
            Stmt::VarDecl { var_type, name, init, extra_vars, loc } => {
                let mut emit_one = |vty: &Type, n: &str, init: &mut Option<Expr>, loc: &SourceLoc| {
                    let elem_count = if vty.is_array() || vty.is_struct() {
                        let sz = (self.type_size(vty) + 3) / 4;
                        if sz < 1 { 1 } else { sz }
                    } else { 1 };
                    let local_idx = self.next_local_idx;
                    self.next_local_idx += elem_count;
                    self.local_indices.insert(n.to_string(), local_idx);
                    self.local_types.insert(n.to_string(), vty.clone());
                    self.sym_index.insert(n.to_string(), self.symbols.len() as i32);
                    self.symbols.push(VMSymbol {
                        name: n.to_string(),
                        addr: local_idx as u32 * 4,
                        is_local: true,
                        ty: vty.clone(),
                        scope_depth: 1,
                    });
                    if let Some(ref mut e) = init {
                        if vty.is_array() && matches!(e, Expr::InitList { .. }) {
                            if let Expr::InitList { ref mut elements, .. } = e {
                                let values = flatten_init_list(elements);
                                if vty.base_kind == TypeKind::Char {
                                    let base_temp = self.get_temp_slot(0);
                                    self.emit(OpCode::GetFrameBase, 0, loc);
                                    self.emit(OpCode::PushConst, local_idx * 4, loc);
                                    self.emit(OpCode::Add, 0, loc);
                                    self.emit(OpCode::StoreLocal, base_temp, loc);
                                    let byte_count = vty.array_size as usize;
                                    for i in 0..byte_count {
                                        self.emit(OpCode::LoadLocal, base_temp, loc);
                                        self.emit(OpCode::PushConst, i as i32, loc);
                                        self.emit(OpCode::Add, 0, loc);
                                        let byte = values.get(i).copied().unwrap_or(0);
                                        self.emit(OpCode::PushConst, byte, loc);
                                        self.emit(OpCode::StoreMemByte, 0, loc);
                                    }
                                } else {
                                    for i in 0..elem_count as usize {
                                        self.emit(OpCode::PushConst, values.get(i).copied().unwrap_or(0), loc);
                                        self.emit(OpCode::StoreLocal, local_idx + i as i32, loc);
                                    }
                                }
                            }
                        } else if vty.is_struct() && matches!(e, Expr::InitList { .. }) {
                            if let Expr::InitList { ref mut elements, .. } = e {
                                let base_temp = self.get_temp_slot(0);
                                self.emit(OpCode::GetFrameBase, 0, loc);
                                self.emit(OpCode::PushConst, local_idx * 4, loc);
                                self.emit(OpCode::Add, 0, loc);
                                self.emit(OpCode::StoreLocal, base_temp, loc);
                                let fields = self.struct_defs.get(&vty.name).cloned().unwrap_or_default();
                                for (i, elem) in elements.iter_mut().enumerate() {
                                    if i >= elem_count as usize || i >= fields.len() { break; }
                                    let offset = fields.iter().take(i).map(|f| self.type_size(&f.ty)).sum::<i32>();
                                    self.emit(OpCode::LoadLocal, base_temp, loc);
                                    if offset > 0 {
                                        self.emit(OpCode::PushConst, offset, loc);
                                        self.emit(OpCode::Add, 0, loc);
                                    }
                                    self.gen_expr(elem);
                                    self.emit(OpCode::StoreMem, 0, loc);
                                }
                            }
                        } else if vty.is_struct() {
                            self.gen_struct_copy_to_local(local_idx, e, loc);
                        } else if vty.is_array() && matches!(e, Expr::StringLiteral { .. }) {
                            if let Expr::StringLiteral { ref value, .. } = e {
                                let base_temp = self.get_temp_slot(0);
                                self.emit(OpCode::GetFrameBase, 0, loc);
                                self.emit(OpCode::PushConst, local_idx * 4, loc);
                                self.emit(OpCode::Add, 0, loc);
                                self.emit(OpCode::StoreLocal, base_temp, loc);
                                let byte_count = vty.array_size as usize;
                                for i in 0..byte_count {
                                    self.emit(OpCode::LoadLocal, base_temp, loc);
                                    self.emit(OpCode::PushConst, i as i32, loc);
                                    self.emit(OpCode::Add, 0, loc);
                                    let byte = if i < value.len() { value.as_bytes()[i] as i32 } else { 0 };
                                    self.emit(OpCode::PushConst, byte, loc);
                                    self.emit(OpCode::StoreMemByte, 0, loc);
                                }
                            }
                        } else {
                            self.gen_expr(e);
                            if vty.kind == TypeKind::Float && e.ty().kind != TypeKind::Float {
                                self.emit(OpCode::CastI2F, 0, loc);
                            }
                            self.emit(OpCode::StoreLocal, local_idx, loc);
                        }
                    } else {
                        for i in 0..elem_count {
                            self.emit(OpCode::PushConst, 0, loc);
                            self.emit(OpCode::StoreLocal, local_idx + i, loc);
                        }
                    }
                };
                emit_one(var_type, name, init, loc);
                for (ety, ename, einit) in extra_vars.iter_mut() {
                    emit_one(ety, ename, einit, loc);
                }
            }
            Stmt::Expr { expr, .. } => {
                self.gen_expr(expr);
                if !expr.ty().is_void() && !expr.ty().is_struct() {
                    self.emit(OpCode::Pop, 0, &loc);
                }
            }
            Stmt::If { cond, then_stmt, else_stmt, loc } => {
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
            }
            Stmt::DoWhile { body, cond, loc } => {
                let start_ip = self.current_ip();
                self.loop_start_ips.push(start_ip);
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
            }
            Stmt::For { init, cond, step, body, loc } => {
                if let Some(ref mut i) = init { self.gen_stmt(i); }
                let start_ip = self.current_ip();
                let mut cond_jump = 0;
                if let Some(ref mut c) = cond {
                    self.gen_expr(c);
                    cond_jump = self.current_ip();
                    self.emit(OpCode::JumpIfZero, 0, loc);
                }
                self.loop_start_ips.push(start_ip);
                let break_base = self.break_patches.len();
                let continue_base = self.continue_patches.len();
                self.gen_stmt(body);
                let continue_ip = self.current_ip();
                if let Some(ref mut s) = step {
                    self.gen_expr(s);
                    self.emit(OpCode::Pop, 0, loc);
                }
                self.emit(OpCode::Jump, start_ip as i32, loc);
                let end_ip = self.current_ip();
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
            }
            Stmt::Return { value, loc } => {
                if let Some(ref mut v) = value {
                    self.gen_expr(v);
                    let ret_is_float = self.func_table.get(&self.current_func).map(|m| m.return_type.kind == TypeKind::Float).unwrap_or(false);
                    if ret_is_float && v.ty().kind != TypeKind::Float {
                        self.emit(OpCode::CastI2F, 0, loc);
                    } else if !ret_is_float && v.ty().kind == TypeKind::Float {
                        self.emit(OpCode::CastF2I, 0, loc);
                    }
                    self.emit(OpCode::Ret, 0, loc);
                } else {
                    self.emit(OpCode::RetVoid, 0, loc);
                }
            }
            Stmt::Break { loc } => {
                let ip = self.current_ip();
                self.emit(OpCode::Jump, 0, loc);
                self.break_patches.push(ip);
            }
            Stmt::Continue { loc } => {
                let ip = self.current_ip();
                self.emit(OpCode::Jump, 0, loc);
                self.continue_patches.push(ip);
            }
            Stmt::Switch { cond, body, loc } => {
                self.gen_switch(cond, body, loc);
            }
            Stmt::Case { .. } => {}
        }
    }

    fn gen_switch(&mut self, cond: &mut Expr, body: &mut Stmt, loc: &SourceLoc) {
        let mut cases: Vec<(Option<Expr>, Box<Stmt>)> = Vec::new();
        let mut default_case: Option<Box<Stmt>> = None;

        fn collect_cases(stmt: &mut Stmt, cases: &mut Vec<(Option<Expr>, Box<Stmt>)>, default: &mut Option<Box<Stmt>>) {
            match stmt {
                Stmt::Block { stmts, .. } => {
                    for s in stmts { collect_cases(s, cases, default); }
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

    fn gen_expr(&mut self, expr: &mut Expr) {
        let loc = *expr.loc();
        match expr {
            Expr::Literal { value, .. } => {
                self.emit(OpCode::PushConst, *value, &loc);
            }
            Expr::FloatLiteral { value, .. } => {
                let bits = (*value as f32).to_bits() as i32;
                self.emit(OpCode::PushConstF, bits, &loc);
            }
            Expr::StringLiteral { value, .. } => {
                let addr = self.string_mem_offset;
                let new_offset = addr + value.len() as u32 + 1;
                let new_offset = (new_offset + 3) & !3;
                if new_offset > 0x5000 {
                    self.report_error("字符串字面量过多，超出内存限制", &loc);
                    self.emit(OpCode::PushConst, addr as i32, &loc);
                } else {
                    self.string_data.push((addr, value.clone()));
                    self.string_mem_offset = new_offset;
                    self.emit(OpCode::PushConst, addr as i32, &loc);
                }
            }
            Expr::Identifier { name, .. } => {
                // Function name used as value (function pointer)
                if let Some(&idx) = self.func_index.get(name) {
                    self.emit(OpCode::PushConst, idx, &loc);
                    return;
                }
                let local_idx = self.resolve_local(name);
                if local_idx >= 0 {
                    if let Some(ty) = self.local_types.get(name) {
                        if ty.is_array() {
                            if local_idx < self.current_func_arg_count {
                                self.emit(OpCode::LoadLocal, local_idx, &loc);
                            } else {
                                self.emit(OpCode::GetFrameBase, 0, &loc);
                                self.emit(OpCode::PushConst, local_idx * 4, &loc);
                                self.emit(OpCode::Add, 0, &loc);
                            }
                        } else {
                            self.emit(OpCode::LoadLocal, local_idx, &loc);
                        }
                    } else {
                        self.emit(OpCode::LoadLocal, local_idx, &loc);
                    }
                } else {
                    let global_idx = self.resolve_global(name);
                    if global_idx >= 0 {
                        if let Some(ty) = self.global_types.get(name) {
                            if ty.is_array() {
                                self.emit(OpCode::PushConst, 0x1000 + global_idx * 4, &loc);
                            } else {
                                self.emit(OpCode::LoadGlobal, global_idx, &loc);
                            }
                        } else {
                            self.emit(OpCode::LoadGlobal, global_idx, &loc);
                        }
                    } else {
                        self.report_error(&format!("未声明的标识符 '{}'", name), &loc);
                        self.emit(OpCode::PushConst, 0, &loc);
                    }
                }
            }
            Expr::Binary { op, left, right, ty, .. } => {
                let left_is_ptr = left.ty().is_pointer() || left.ty().is_array();
                let right_is_ptr = right.ty().is_pointer() || right.ty().is_array();
                let any_float = left.ty().kind == TypeKind::Float || right.ty().kind == TypeKind::Float;
                let result_is_float = ty.kind == TypeKind::Float;

                self.gen_expr(left);
                if any_float && !left_is_ptr && left.ty().kind != TypeKind::Float {
                    self.emit(OpCode::CastI2F, 0, &loc);
                }
                self.gen_expr(right);
                if any_float && !right_is_ptr && right.ty().kind != TypeKind::Float {
                    self.emit(OpCode::CastI2F, 0, &loc);
                }

                match op {
                    BinaryOp::Add => {
                        if left_is_ptr && !right_is_ptr {
                            let step = self.ptr_step_size(left.ty());
                            self.emit(OpCode::PushConst, step, &loc);
                            self.emit(OpCode::Mul, 0, &loc);
                            self.emit(OpCode::Add, 0, &loc);
                        } else if !left_is_ptr && right_is_ptr {
                            let step = self.ptr_step_size(right.ty());
                            self.emit(OpCode::Swap, 0, &loc);
                            self.emit(OpCode::PushConst, step, &loc);
                            self.emit(OpCode::Mul, 0, &loc);
                            self.emit(OpCode::Swap, 0, &loc);
                            self.emit(OpCode::Add, 0, &loc);
                        } else if result_is_float {
                            self.emit(OpCode::AddF, 0, &loc);
                        } else {
                            self.emit(OpCode::Add, 0, &loc);
                        }
                    }
                    BinaryOp::Sub => {
                        if left_is_ptr && right_is_ptr {
                            let step = self.ptr_step_size(left.ty());
                            self.emit(OpCode::Sub, 0, &loc);
                            self.emit(OpCode::PushConst, step, &loc);
                            self.emit(OpCode::Div, 0, &loc);
                        } else if left_is_ptr && !right_is_ptr {
                            let step = self.ptr_step_size(left.ty());
                            self.emit(OpCode::PushConst, step, &loc);
                            self.emit(OpCode::Mul, 0, &loc);
                            self.emit(OpCode::Sub, 0, &loc);
                        } else if result_is_float {
                            self.emit(OpCode::SubF, 0, &loc);
                        } else {
                            self.emit(OpCode::Sub, 0, &loc);
                        }
                    }
                    BinaryOp::Mul => {
                        if result_is_float { self.emit(OpCode::MulF, 0, &loc); }
                        else { self.emit(OpCode::Mul, 0, &loc); }
                    }
                    BinaryOp::Div => {
                        if result_is_float { self.emit(OpCode::DivF, 0, &loc); }
                        else { self.emit(OpCode::Div, 0, &loc); }
                    }
                    BinaryOp::Mod => self.emit(OpCode::Mod, 0, &loc),
                    BinaryOp::Eq => {
                        if any_float { self.emit(OpCode::EqF, 0, &loc); }
                        else { self.emit(OpCode::Eq, 0, &loc); }
                    }
                    BinaryOp::Ne => {
                        if any_float { self.emit(OpCode::NeF, 0, &loc); }
                        else { self.emit(OpCode::Ne, 0, &loc); }
                    }
                    BinaryOp::Lt => {
                        if any_float { self.emit(OpCode::LtF, 0, &loc); }
                        else { self.emit(OpCode::Lt, 0, &loc); }
                    }
                    BinaryOp::Le => {
                        if any_float { self.emit(OpCode::LeF, 0, &loc); }
                        else { self.emit(OpCode::Le, 0, &loc); }
                    }
                    BinaryOp::Gt => {
                        if any_float { self.emit(OpCode::GtF, 0, &loc); }
                        else { self.emit(OpCode::Gt, 0, &loc); }
                    }
                    BinaryOp::Ge => {
                        if any_float { self.emit(OpCode::GeF, 0, &loc); }
                        else { self.emit(OpCode::Ge, 0, &loc); }
                    }
                    BinaryOp::And => self.emit(OpCode::And, 0, &loc),
                    BinaryOp::Or => self.emit(OpCode::Or, 0, &loc),
                    BinaryOp::BitAnd => self.emit(OpCode::BitAnd, 0, &loc),
                    BinaryOp::BitOr => self.emit(OpCode::BitOr, 0, &loc),
                    BinaryOp::BitXor => self.emit(OpCode::BitXor, 0, &loc),
                    BinaryOp::Shl => self.emit(OpCode::Shl, 0, &loc),
                    BinaryOp::Shr => self.emit(OpCode::Shr, 0, &loc),
                }
            }
            Expr::Unary { op, operand, .. } => {
                match op {
                    UnaryOp::Neg => {
                        self.gen_expr(operand);
                        if operand.ty().kind == TypeKind::Float {
                            self.emit(OpCode::NegF, 0, &loc);
                        } else {
                            self.emit(OpCode::Neg, 0, &loc);
                        }
                    }
                    UnaryOp::Not => {
                        self.gen_expr(operand);
                        self.emit(OpCode::Not, 0, &loc);
                    }
                    UnaryOp::BitNot => {
                        self.gen_expr(operand);
                        self.emit(OpCode::BitNot, 0, &loc);
                    }
                    UnaryOp::Addr => {
                        match operand.as_mut() {
                            Expr::Identifier { name, .. } => {
                                if let Some(&idx) = self.local_indices.get(name) {
                                    self.emit(OpCode::GetFrameBase, 0, &loc);
                                    self.emit(OpCode::PushConst, idx * 4, &loc);
                                    self.emit(OpCode::Add, 0, &loc);
                                } else if let Some(&idx) = self.global_indices.get(name) {
                                    self.emit(OpCode::PushConst, 0x1000 + idx * 4, &loc);
                                } else {
                                    self.report_error("取地址暂不支持此表达式", &loc);
                                    self.emit(OpCode::PushConst, 0, &loc);
                                }
                            }
                            Expr::Index { array, index, ty, .. } => {
                                self.gen_index(array, index, ty, &loc, true);
                            }
                            Expr::Member { object, member, .. } => {
                                self.gen_member_addr(object, member, &loc);
                            }
                            Expr::Unary { op: UnaryOp::Deref, operand: inner, .. } => {
                                self.gen_expr(inner);
                            }
                            _ => {
                                self.report_error("取地址暂不支持此表达式", &loc);
                                self.emit(OpCode::PushConst, 0, &loc);
                            }
                        }
                    }
                    UnaryOp::Deref => {
                        self.gen_expr(operand);
                        self.emit(OpCode::LoadMem, 0, &loc);
                    }
                    UnaryOp::PreInc | UnaryOp::PostInc | UnaryOp::PreDec | UnaryOp::PostDec => {
                        let is_inc = matches!(op, UnaryOp::PreInc | UnaryOp::PostInc);
                        let is_pre = matches!(op, UnaryOp::PreInc | UnaryOp::PreDec);
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
                        match operand.as_mut() {
                            Expr::Identifier { name, .. } => {
                                let step = if let Some(ty) = self.local_types.get(name) {
                                    self.ptr_step_size(ty)
                                } else if let Some(ty) = self.global_types.get(name) {
                                    self.ptr_step_size(ty)
                                } else { 1 };
                                let local_idx = self.resolve_local(name);
                                if local_idx >= 0 {
                                    self.emit(OpCode::LoadLocal, local_idx, &loc);
                                    if !is_pre { self.emit(OpCode::Dup, 0, &loc); }
                                    self.emit(OpCode::PushConst, step, &loc);
                                    self.emit(if is_inc { OpCode::Add } else { OpCode::Sub }, 0, &loc);
                                    if is_pre { self.emit(OpCode::Dup, 0, &loc); }
                                    self.emit(OpCode::StoreLocal, local_idx, &loc);
                                } else {
                                    let global_idx = self.resolve_global(name);
                                    if global_idx >= 0 {
                                        self.emit(OpCode::LoadGlobal, global_idx, &loc);
                                        if !is_pre { self.emit(OpCode::Dup, 0, &loc); }
                                        self.emit(OpCode::PushConst, step, &loc);
                                        self.emit(if is_inc { OpCode::Add } else { OpCode::Sub }, 0, &loc);
                                        if is_pre { self.emit(OpCode::Dup, 0, &loc); }
                                        self.emit(OpCode::StoreGlobal, global_idx, &loc);
                                    } else {
                                        self.report_error("自增/自减暂只支持简单变量", &loc);
                                        self.emit(OpCode::PushConst, 0, &loc);
                                    }
                                }
                            }
                            Expr::Index { array, index, ty, .. } => {
                                let result_ty = ty.clone();
                                let step = self.ptr_step_size(ty);
                                self.gen_index(array, index, &result_ty, &loc, true);
                                gen_mem_inc_dec(self, is_inc, is_pre, step, &loc);
                            }
                            Expr::Member { object, member, ty, .. } => {
                                let step = self.ptr_step_size(ty);
                                self.gen_member_addr(object, member, &loc);
                                gen_mem_inc_dec(self, is_inc, is_pre, step, &loc);
                            }
                            Expr::Unary { op: UnaryOp::Deref, operand: inner, .. } => {
                                let step = self.ptr_step_size(inner.ty());
                                self.gen_expr(inner);
                                gen_mem_inc_dec(self, is_inc, is_pre, step, &loc);
                            }
                            _ => {
                                self.report_error("自增/自减暂只支持简单变量", &loc);
                                self.emit(OpCode::PushConst, 0, &loc);
                            }
                        }
                    }
                }
            }
            Expr::Call { name, args, .. } => {
                for arg in args.iter_mut().rev() {
                    let arg_ty = arg.ty();
                    if arg_ty.is_struct() {
                        let sz = (self.type_size(arg_ty) + 3) / 4;
                        let slots = if sz < 1 { 1 } else { sz };
                        if let Expr::Identifier { name: arg_name, .. } = arg {
                            if let Some(&idx) = self.local_indices.get(arg_name) {
                                for i in (0..slots).rev() {
                                    self.emit(OpCode::LoadLocal, idx + i, &loc);
                                }
                            } else if let Some(&idx) = self.global_indices.get(arg_name) {
                                for i in (0..slots).rev() {
                                    self.emit(OpCode::LoadGlobal, idx + i, &loc);
                                }
                            } else {
                                self.gen_expr(arg);
                                for _ in 1..slots {
                                    self.emit(OpCode::PushConst, 0, &loc);
                                }
                            }
                        } else {
                            self.gen_expr(arg);
                            for _ in 1..slots {
                                self.emit(OpCode::PushConst, 0, &loc);
                            }
                        }
                    } else {
                        self.gen_expr(arg);
                    }
                }
                if let Some(&idx) = self.func_index.get(name) {
                    self.emit(OpCode::Call, idx, &loc);
                } else {
                    let host_name = match name.as_str() {
                        "print_int" => "__cide_output",
                        "printf" => "__cide_printf_n",
                        "scanf" => "__cide_scanf_n",
                        "strlen" => "strlen",
                        "strcpy" => "strcpy",
                        "strcmp" => "strcmp",
                        "getchar" => "getchar",
                        "putchar" => "putchar",
                        "rand" => "rand",
                        "srand" => "srand",
                        "memset" => "memset",
                        "exit" => "exit",
                        "strcat" => "strcat",
                        "atoi" => "atoi",
                        "fprintf" => "fprintf",
                        "realloc" => "realloc",
                        "qsort" => "qsort",
                        _ => name.as_str(),
                    };
                    let host_id = match host_name {
                        "__cide_output" => crate::vm::host_func_id::OUTPUT,
                        "__cide_step" => crate::vm::host_func_id::STEP,
                        "malloc" => crate::vm::host_func_id::MALLOC,
                        "free" => crate::vm::host_func_id::FREE,
                        "__cide_printf_0" => crate::vm::host_func_id::PRINTF_0,
                        "__cide_printf_1" => crate::vm::host_func_id::PRINTF_1,
                        "__cide_printf_n" => crate::vm::host_func_id::PRINTF_N,
                        "__cide_scanf_n" => crate::vm::host_func_id::SCANF_N,
                        "strlen" => crate::vm::host_func_id::STRLEN,
                        "strcpy" => crate::vm::host_func_id::STRCPY,
                        "strcmp" => crate::vm::host_func_id::STRCMP,
                        "getchar" => crate::vm::host_func_id::GETCHAR,
                        "putchar" => crate::vm::host_func_id::PUTCHAR,
                        "rand" => crate::vm::host_func_id::RAND,
                        "srand" => crate::vm::host_func_id::SRAND,
                        "memset" => crate::vm::host_func_id::MEMSET,
                        "exit" => crate::vm::host_func_id::EXIT,
                        "strcat" => crate::vm::host_func_id::STRCAT,
                        "atoi" => crate::vm::host_func_id::ATOI,
                        "fprintf" => crate::vm::host_func_id::FPRINTF,
                        "realloc" => crate::vm::host_func_id::REALLOC,
                        "qsort" => crate::vm::host_func_id::QSORT,
                        _ => {
                            self.report_error(&format!("未定义的函数 '{}'", name), &loc);
                            self.emit(OpCode::PushConst, 0, &loc);
                            return;
                        }
                    };
                    self.emit(OpCode::CallHost, host_id as i32, &loc);
                }
            }
            Expr::Index { array, index, ty, .. } => {
                self.gen_index(array, index, ty, &loc, false);
            }
            Expr::Member { object, member, ty, .. } => {
                self.gen_member_addr(object, member, &loc);
                if !ty.is_array() {
                    self.emit(OpCode::LoadMem, 0, &loc);
                }
            }
            Expr::Ternary { cond, then_branch, else_branch, .. } => {
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
                let size = if let Some(ref t) = target_type {
                    self.type_size(t)
                } else if let Some(ref op) = operand {
                    self.type_size(op.ty())
                } else {
                    0
                };
                self.emit(OpCode::PushConst, size, &loc);
            }
            Expr::Cast { expr, target_type, .. } => {
                self.gen_expr(expr);
                if target_type.kind == TypeKind::Float && expr.ty().kind != TypeKind::Float {
                    self.emit(OpCode::CastI2F, 0, &loc);
                } else if target_type.kind != TypeKind::Float && expr.ty().kind == TypeKind::Float {
                    self.emit(OpCode::CastF2I, 0, &loc);
                }
            }
            Expr::InitList { .. } => {
                self.report_error("初始化列表只能在变量声明中使用", &loc);
                self.emit(OpCode::PushConst, 0, &loc);
            }
        }
    }

    fn gen_member_addr(&mut self, object: &mut Expr, member: &str, loc: &SourceLoc) {
        if object.ty().is_pointer() {
            self.gen_expr(object);
        } else if let Expr::Index { array, index, ty, .. } = object {
            self.gen_index(array, index, ty, loc, true);
        } else if let Expr::Member { object: inner, member: m, .. } = object {
            self.gen_member_addr(inner, m, loc);
        } else if let Expr::Identifier { name, .. } = object {
            if let Some(&idx) = self.local_indices.get(name) {
                self.emit(OpCode::GetFrameBase, 0, loc);
                self.emit(OpCode::PushConst, idx * 4, loc);
                self.emit(OpCode::Add, 0, loc);
            } else if let Some(&idx) = self.global_indices.get(name) {
                self.emit(OpCode::PushConst, 0x1000 + idx * 4, loc);
            } else {
                self.report_error("未声明的结构体变量", loc);
                self.emit(OpCode::PushConst, 0, loc);
            }
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
                    bound_size = if ty.dims.is_empty() { ty.array_size } else { ty.dims[0] };
                    sym_idx = self.resolve_symbol_index(name);
                }
            } else if let Some(ty) = self.global_types.get(name) {
                if ty.is_array() {
                    bound_size = if ty.dims.is_empty() { ty.array_size } else { ty.dims[0] };
                    sym_idx = self.resolve_symbol_index(name);
                }
            }
        } else if let Expr::Index { .. } = array {
            if array.ty().is_array() && !array.ty().dims.is_empty() {
                bound_size = array.ty().dims[0];
            }
        }
        let stride = compute_stride(array.ty(), self.elem_type_size(array.ty()));
        self.gen_expr(array);
        self.gen_expr(index);

        if bound_size > 0 {
            if sym_idx >= 0 {
                self.emit(OpCode::TrapBounds, sym_idx, loc);
            } else {
                self.emit(OpCode::TrapBounds, -bound_size, loc);
            }
        }

        self.emit(OpCode::PushConst, stride, loc);
        self.emit(OpCode::Mul, 0, loc);
        self.emit(OpCode::Add, 0, loc);
        if !is_assign && !result_ty.is_array() {
            if result_ty.kind == TypeKind::Char {
                self.emit(OpCode::LoadMemByte, 0, loc);
            } else {
                self.emit(OpCode::LoadMem, 0, loc);
            }
        }
    }

    fn gen_expr_with_cast(&mut self, expr: &mut Expr, target_is_float: bool, loc: &SourceLoc) {
        self.gen_expr(expr);
        if target_is_float && expr.ty().kind != TypeKind::Float {
            self.emit(OpCode::CastI2F, 0, loc);
        } else if !target_is_float && expr.ty().kind == TypeKind::Float {
            self.emit(OpCode::CastF2I, 0, loc);
        }
    }

    fn gen_addr(&mut self, expr: &mut Expr, loc: &SourceLoc) {
        match expr {
            Expr::Identifier { name, .. } => {
                if let Some(&idx) = self.local_indices.get(name) {
                    self.emit(OpCode::GetFrameBase, 0, loc);
                    self.emit(OpCode::PushConst, idx * 4, loc);
                    self.emit(OpCode::Add, 0, loc);
                } else if let Some(&idx) = self.global_indices.get(name) {
                    self.emit(OpCode::PushConst, 0x1000 + idx * 4, loc);
                } else {
                    self.report_error("未声明的变量", loc);
                    self.emit(OpCode::PushConst, 0, loc);
                }
            }
            Expr::Index { array, index, ty, .. } => {
                self.gen_index(array, index, ty, loc, true);
            }
            Expr::Member { object, member, .. } => {
                self.gen_member_addr(object, member, loc);
            }
            Expr::Unary { op: UnaryOp::Deref, operand, .. } => {
                self.gen_expr(operand);
            }
            _ => {
                self.report_error("不支持的地址生成", loc);
                self.emit(OpCode::PushConst, 0, loc);
            }
        }
    }

    fn gen_struct_copy(&mut self, left: &mut Expr, right: &mut Expr, loc: &SourceLoc) {
        let size = self.type_size(left.ty());
        if size <= 0 { return; }
        let src_temp = self.get_temp_slot(0);
        let dst_temp = self.get_temp_slot(1);
        self.gen_addr(right, loc);
        self.emit(OpCode::StoreLocal, src_temp, loc);
        self.gen_addr(left, loc);
        self.emit(OpCode::StoreLocal, dst_temp, loc);
        for i in 0..size / 4 {
            self.emit(OpCode::LoadLocal, dst_temp, loc);
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

    fn gen_struct_copy_to_local(&mut self, local_idx: i32, right: &mut Expr, loc: &SourceLoc) {
        let size = self.type_size(right.ty());
        if size <= 0 { return; }
        let src_temp = self.get_temp_slot(0);
        self.gen_addr(right, loc);
        self.emit(OpCode::StoreLocal, src_temp, loc);
        for i in 0..size / 4 {
            self.emit(OpCode::GetFrameBase, 0, loc);
            self.emit(OpCode::PushConst, (local_idx + i) * 4, loc);
            self.emit(OpCode::Add, 0, loc);
            self.emit(OpCode::LoadLocal, src_temp, loc);
            if i > 0 {
                self.emit(OpCode::PushConst, i * 4, loc);
                self.emit(OpCode::Add, 0, loc);
            }
            self.emit(OpCode::LoadMem, 0, loc);
            self.emit(OpCode::StoreMem, 0, loc);
        }
    }

    fn gen_assign(&mut self, op: &AssignOp, left: &mut Expr, right: &mut Expr, loc: &SourceLoc) {
        let left_is_float = left.ty().kind == TypeKind::Float;
        if left.ty().is_struct() && *op == AssignOp::Assign {
            self.gen_struct_copy(left, right, loc);
            return;
        }
        let emit_compound = |this: &mut Self, loc: &SourceLoc| {
            match op {
                AssignOp::AddAssign => this.emit(if left_is_float { OpCode::AddF } else { OpCode::Add }, 0, loc),
                AssignOp::SubAssign => this.emit(if left_is_float { OpCode::SubF } else { OpCode::Sub }, 0, loc),
                AssignOp::MulAssign => this.emit(if left_is_float { OpCode::MulF } else { OpCode::Mul }, 0, loc),
                AssignOp::DivAssign => this.emit(if left_is_float { OpCode::DivF } else { OpCode::Div }, 0, loc),
                AssignOp::ModAssign => this.emit(OpCode::Mod, 0, loc),
                _ => {}
            }
        };

        if let Expr::Identifier { name, .. } = left {
            let local_idx = self.resolve_local(name);
            if local_idx >= 0 {
                if *op != AssignOp::Assign {
                    self.emit(OpCode::LoadLocal, local_idx, loc);
                    self.gen_expr_with_cast(right, left_is_float, loc);
                    emit_compound(self, loc);
                } else {
                    self.gen_expr_with_cast(right, left_is_float, loc);
                }
                self.emit(OpCode::StoreLocal, local_idx, loc);
                self.emit(OpCode::LoadLocal, local_idx, loc);
                return;
            }
            let global_idx = self.resolve_global(name);
            if global_idx >= 0 {
                if *op != AssignOp::Assign {
                    self.emit(OpCode::LoadGlobal, global_idx, loc);
                    self.gen_expr_with_cast(right, left_is_float, loc);
                    emit_compound(self, loc);
                } else {
                    self.gen_expr_with_cast(right, left_is_float, loc);
                }
                self.emit(OpCode::StoreGlobal, global_idx, loc);
                self.emit(OpCode::LoadGlobal, global_idx, loc);
                return;
            }
        } else if let Expr::Index { array, index, ty, .. } = left {
            let result_ty = ty.clone();
            self.gen_index(array, index, &result_ty, loc, true);
            if *op != AssignOp::Assign {
                self.emit(OpCode::Dup, 0, loc);
                if result_ty.kind == TypeKind::Char {
                    self.emit(OpCode::LoadMemByte, 0, loc);
                } else {
                    self.emit(OpCode::LoadMem, 0, loc);
                }
                self.gen_expr_with_cast(right, left_is_float, loc);
                emit_compound(self, loc);
            } else {
                self.gen_expr_with_cast(right, left_is_float, loc);
            }
            let val_temp = self.get_temp_slot(0);
            self.emit(OpCode::StoreLocal, val_temp, loc);
            let addr_temp = self.get_temp_slot(2);
            self.emit(OpCode::StoreLocal, addr_temp, loc);
            self.emit(OpCode::LoadLocal, addr_temp, loc);
            self.emit(OpCode::LoadLocal, val_temp, loc);
            if result_ty.kind == TypeKind::Char {
                self.emit(OpCode::StoreMemByte, 0, loc);
            } else {
                self.emit(OpCode::StoreMem, 0, loc);
            }
            self.emit(OpCode::LoadLocal, addr_temp, loc);
            self.emit(OpCode::LoadMem, 0, loc);
            return;
        } else if let Expr::Unary { op: UnaryOp::Deref, operand, .. } = left {
            self.gen_expr(operand);
            if *op != AssignOp::Assign {
                self.emit(OpCode::Dup, 0, loc);
                self.emit(OpCode::LoadMem, 0, loc);
                self.gen_expr_with_cast(right, left_is_float, loc);
                emit_compound(self, loc);
            } else {
                self.gen_expr_with_cast(right, left_is_float, loc);
            }
            let val_temp = self.get_temp_slot(0);
            self.emit(OpCode::StoreLocal, val_temp, loc);
            let addr_temp = self.get_temp_slot(1);
            self.emit(OpCode::StoreLocal, addr_temp, loc);
            self.emit(OpCode::LoadLocal, addr_temp, loc);
            self.emit(OpCode::LoadLocal, val_temp, loc);
            self.emit(OpCode::StoreMem, 0, loc);
            self.emit(OpCode::LoadLocal, addr_temp, loc);
            self.emit(OpCode::LoadMem, 0, loc);
            return;
        } else if let Expr::Member { object, member, .. } = left {
            self.gen_member_addr(object, member, loc);
            if *op != AssignOp::Assign {
                self.emit(OpCode::Dup, 0, loc);
                self.emit(OpCode::LoadMem, 0, loc);
                self.gen_expr_with_cast(right, left_is_float, loc);
                emit_compound(self, loc);
            } else {
                self.gen_expr_with_cast(right, left_is_float, loc);
            }
            let val_temp = self.get_temp_slot(0);
            self.emit(OpCode::StoreLocal, val_temp, loc);
            let addr_temp = self.get_temp_slot(1);
            self.emit(OpCode::StoreLocal, addr_temp, loc);
            self.emit(OpCode::LoadLocal, addr_temp, loc);
            self.emit(OpCode::LoadLocal, val_temp, loc);
            self.emit(OpCode::StoreMem, 0, loc);
            self.emit(OpCode::LoadLocal, addr_temp, loc);
            self.emit(OpCode::LoadMem, 0, loc);
            return;
        }

        self.report_error("赋值目标不支持", loc);
        self.gen_expr_with_cast(right, left_is_float, loc);
        self.emit(OpCode::Pop, 0, loc);
        self.emit(OpCode::PushConst, 0, loc);
    }
}

fn stmt_loc(stmt: &Stmt) -> SourceLoc {
    match stmt {
        Stmt::Block { loc, .. } => *loc,
        Stmt::VarDecl { loc, .. } => *loc,
        Stmt::Expr { loc, .. } => *loc,
        Stmt::If { loc, .. } => *loc,
        Stmt::While { loc, .. } => *loc,
        Stmt::DoWhile { loc, .. } => *loc,
        Stmt::For { loc, .. } => *loc,
        Stmt::Return { loc, .. } => *loc,
        Stmt::Break { loc, .. } => *loc,
        Stmt::Continue { loc, .. } => *loc,
        Stmt::Switch { loc, .. } => *loc,
        Stmt::Case { loc, .. } => *loc,
    }
}

fn flatten_init_list(elements: &[Expr]) -> Vec<i32> {
    let mut result = Vec::new();
    for elem in elements {
        match elem {
            Expr::Literal { value, .. } => result.push(*value),
            Expr::FloatLiteral { value, .. } => result.push((*value as f32).to_bits() as i32),
            Expr::InitList { elements: sub, .. } => result.extend(flatten_init_list(sub)),
            Expr::Unary { op: UnaryOp::Neg, operand, .. } => {
                if let Expr::Literal { value, .. } = operand.as_ref() {
                    result.push(-*value);
                } else {
                    result.push(0);
                }
            }
            _ => result.push(0),
        }
    }
    result
}

fn compute_stride(arr_type: &Type, elem_size: i32) -> i32 {
    if !arr_type.is_array() || arr_type.dims.is_empty() { return elem_size; }
    let mut stride = elem_size;
    for i in 1..arr_type.dims.len() {
        stride *= if arr_type.dims[i] > 0 { arr_type.dims[i] } else { 0 };
    }
    stride
}

#[derive(Debug, Clone)]
pub struct CompileOutput {
    pub code: Vec<Instruction>,
    pub globals_init: Vec<i32>,
    pub func_table: HashMap<String, FuncMeta>,
    pub func_index: HashMap<String, i32>,
    pub string_data: Vec<(u32, String)>,
    pub source_map: Vec<(u32, VMSourceLoc)>,
    pub symbols: Vec<VMSymbol>,
    pub struct_defs: HashMap<String, Vec<StructField>>,
}
