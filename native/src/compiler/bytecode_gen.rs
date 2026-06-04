use crate::compiler::ast::*;
use crate::vm::instruction::{Instruction, SourceLoc as VMSourceLoc};
use crate::vm::opcode::OpCode;
use crate::vm::vm::VMSymbol;
use std::collections::HashMap;

fn base_kind(ty: &Type) -> TypeKind {
    match ty {
        Type::Pointer { pointee, .. } => pointee.kind(),
        Type::Array { element, .. } => base_kind(element),
        _ => ty.kind(),
    }
}

#[derive(Debug, Clone)]
pub struct FuncMeta {
    pub ip: usize,
    /// 参数总 word 数（以 4-byte words 计），供 Call 指令弹栈使用。
    pub arg_count: i32,
    /// 参数个数（供 call_user_function 使用，与总 word 数不同）。
    pub param_count: i32,
    pub local_count: i32,
    pub param_sizes: Vec<i32>,
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
    current_func_arg_bytes: i32,
    global_indices: HashMap<String, i32>,
    global_types: HashMap<String, Type>,
    local_indices: HashMap<String, i32>,
    local_types: HashMap<String, Type>,
    next_local_offset: i32,
    temp_slot0: i32,
    temp_slot1: i32,
    temp_slot2: i32,
    globals_init_32: Vec<(u32, i32)>,
    globals_init_64: Vec<(u32, u64)>,
    next_global_offset: i32,
    f64_constants: Vec<f64>,
    i64_constants: Vec<i64>,
    symbols: Vec<VMSymbol>,
    sym_index: HashMap<String, i32>,
    struct_defs: HashMap<String, Vec<StructField>>,
    union_defs: HashMap<String, Vec<StructField>>,
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
            current_func_arg_bytes: 0,
            global_indices: HashMap::new(),
            global_types: HashMap::new(),
            local_indices: HashMap::new(),
            local_types: HashMap::new(),
            next_local_offset: 0,
            temp_slot0: -1,
            temp_slot1: -1,
            temp_slot2: -1,
            globals_init_32: Vec::new(),
            globals_init_64: Vec::new(),
            next_global_offset: 0,
            f64_constants: Vec::new(),
            i64_constants: Vec::new(),
            symbols: Vec::new(),
            sym_index: HashMap::new(),
            struct_defs: HashMap::new(),
            union_defs: HashMap::new(),
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
        for u in &program.unions {
            self.union_defs.insert(u.name.clone(), u.fields.clone());
        }

        // Pass 1: Register globals (byte offsets)
        for g in &program.globals {
            let sz = self.type_size(&g.ty);
            let offset = self.next_global_offset;
            self.global_indices.insert(g.name.clone(), offset);
            self.global_types.insert(g.name.clone(), g.ty.clone());
            if let Some(ref init) = g.init {
                match init {
                    Expr::InitList { elements, .. } => {
                        if base_kind(&g.ty) == TypeKind::Char {
                            let values = flatten_init_list(elements, &mut self.errors);
                            for i in 0..sz as usize {
                                self.globals_init_32.push((offset as u32 + i as u32, values.get(i).copied().unwrap_or(0)));
                            }
                        } else {
                            // Non-char array: handle element-by-element
                            let elem_size = self.elem_type_size(&g.ty);
                            let count = g.ty.total_elements();
                            if elem_size == 8 {
                                for i in 0..count as usize {
                                    let addr = offset as u32 + (i as u32) * elem_size as u32;
                                    let val64 = if let Some(elem) = elements.get(i) {
                                        match elem {
                                            Expr::FloatLiteral { value, ty, .. } => {
                                                if ty.kind() == TypeKind::Double {
                                                    value.to_bits()
                                                } else {
                                                    (*value).to_bits()
                                                }
                                            }
                                            Expr::LongLiteral { value, .. } => (*value as f64).to_bits(),
                                            Expr::Literal { value, .. } => (*value as f64).to_bits(),
                                            Expr::Unary { op: UnaryOp::Neg, operand, .. } => {
                                                if let Expr::FloatLiteral { value, .. } = operand.as_ref() {
                                                    (-*value).to_bits()
                                                } else if let Expr::LongLiteral { value, .. } = operand.as_ref() {
                                                    (-(*value as f64)).to_bits()
                                                } else if let Expr::Literal { value, .. } = operand.as_ref() {
                                                    (-(*value as f64)).to_bits()
                                                } else {
                                                    0
                                                }
                                            }
                                            _ => 0,
                                        }
                                    } else {
                                        0
                                    };
                                    self.globals_init_64.push((addr, val64));
                                }
                            } else {
                                let values = flatten_init_list(elements, &mut self.errors);
                                for i in 0..count as usize {
                                    let addr = offset as u32 + (i as u32) * elem_size as u32;
                                    let val = values.get(i).copied().unwrap_or(0);
                                    self.globals_init_32.push((addr, val));
                                }
                            }
                        }
                    }
                    Expr::StringLiteral { value, .. } => {
                        for i in 0..sz as usize {
                            let byte = if i < value.len() { value.as_bytes()[i] as i32 } else { 0 };
                            self.globals_init_32.push((offset as u32 + i as u32, byte));
                        }
                    }
                    Expr::Literal { value, .. } => {
                        self.globals_init_32.push((offset as u32, *value));
                    }
                    Expr::LongLiteral { value, .. } => {
                        self.globals_init_64.push((offset as u32, *value as u64));
                    }
                    Expr::FloatLiteral { value, .. } => {
                        if g.ty.kind() == TypeKind::Double {
                            self.globals_init_64.push((offset as u32, value.to_bits()));
                        } else {
                            self.globals_init_32.push((offset as u32, (*value as f32).to_bits() as i32));
                        }
                    }
                    _ => {}
                }
            }
            self.sym_index.insert(g.name.clone(), self.symbols.len() as i32);
            self.symbols.push(VMSymbol {
                name: g.name.clone(),
                addr: offset as u32,
                is_local: false,
                ty: g.ty.clone(),
                scope_depth: 0,
                func_name: String::new(),
            });
            self.next_global_offset += sz;
        }

        self.string_mem_offset = 0x1000 + self.next_global_offset as u32;

        // Pass 2: Register function metadata
        for f in &program.funcs {
            if f.body.is_none() { continue; }
            self.func_index.insert(f.name.clone(), self.next_func_idx);
            self.next_func_idx += 1;
            let param_sizes: Vec<i32> = f.params.iter().map(|p| {
                if p.ty.is_array() { 4 } else { self.type_size(&p.ty) }
            }).collect();
            self.func_table.insert(f.name.clone(), FuncMeta {
                ip: 0,
                arg_count: f.params.len() as i32,
                param_count: f.params.len() as i32,
                local_count: 0,
                param_sizes: param_sizes.clone(),
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
            globals_init_32: self.globals_init_32,
            globals_init_64: self.globals_init_64,
            func_table: self.func_table,
            func_index: self.func_index,
            string_data: self.string_data,
            source_map: self.source_map,
            symbols: self.symbols,
            struct_defs: self.struct_defs,
            union_defs: self.union_defs,
            f64_constants: self.f64_constants,
            i64_constants: self.i64_constants,
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
        self.next_local_offset = 0;
        let mut offset = 0;
        let mut param_sizes = Vec::new();
        for p in params.iter() {
            let sz = if p.ty.is_array() { 4 } else { self.type_size(&p.ty) };
            let aligned_sz = (sz + 3) & !3;
            let words = (sz + 3) / 4;
            param_sizes.push(words);
            self.local_indices.insert(p.name.clone(), offset);
            self.local_types.insert(p.name.clone(), p.ty.clone());
            self.sym_index.insert(p.name.clone(), self.symbols.len() as i32);
            self.symbols.push(VMSymbol {
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
        self.temp_slot0 = -1;
        self.temp_slot1 = -1;
        self.temp_slot2 = -1;
        if let Some(meta) = self.func_table.get_mut(name) {
            meta.param_sizes = param_sizes;
        }
    }

    fn exit_function(&mut self) {
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
            *slot = self.next_local_offset;
            self.next_local_offset += 4;
        }
        *slot
    }

    fn get_member_offset(&self, object_type: &Type, member_name: &str) -> i32 {
        match object_type.kind() {
            TypeKind::Union | TypeKind::Pointer if base_kind(object_type) == TypeKind::Union => {
                // All union members start at offset 0
                let _ = member_name;
                0
            }
            TypeKind::Struct => {
                let fields = match self.struct_defs.get(object_type.name()) {
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
            TypeKind::Pointer if base_kind(object_type) == TypeKind::Struct => {
                let fields = match self.struct_defs.get(object_type.name()) {
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
            _ => 0,
        }
    }

    fn push_f64_constant(&mut self, val: f64) -> i32 {
        let idx = self.f64_constants.len() as i32;
        self.f64_constants.push(val);
        idx
    }

    fn push_i64_constant(&mut self, val: i64) -> i32 {
        let idx = self.i64_constants.len() as i32;
        self.i64_constants.push(val);
        idx
    }

    fn ptr_step_size(&self, ty: &Type) -> i32 {
        match ty.kind() {
            TypeKind::Pointer => {
                match base_kind(ty) {
                    TypeKind::Char => 1,
                    TypeKind::Int | TypeKind::Pointer | TypeKind::Float => 4,
                    TypeKind::Double | TypeKind::LongLong => 8,
                    TypeKind::Struct => {
                        self.struct_defs.get(ty.name()).map(|f| {
                            f.iter().map(|field| self.type_size(&field.ty)).sum()
                        }).unwrap_or(4)
                    }
                    TypeKind::Union => {
                        self.union_defs.get(ty.name()).map(|f| {
                            f.iter().map(|field| self.type_size(&field.ty)).max().unwrap_or(0)
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
        match base_kind(arr_type) {
            TypeKind::Char => 1,
            TypeKind::Int | TypeKind::Pointer | TypeKind::Float => 4,
            TypeKind::Double | TypeKind::LongLong => 8,
            TypeKind::Struct => {
                self.struct_defs.get(arr_type.name()).map(|f| {
                    f.iter().map(|field| self.type_size(&field.ty)).sum()
                }).unwrap_or(4)
            }
            TypeKind::Union => {
                self.union_defs.get(arr_type.name()).map(|f| {
                    f.iter().map(|field| self.type_size(&field.ty)).max().unwrap_or(0)
                }).unwrap_or(4)
            }
            _ => 4,
        }
    }

    fn resolve_host_func_id(&self, name: &str) -> i32 {
        crate::vm::host_func_id::by_user_name(name).map(|id| id as i32).unwrap_or(-1)
    }

    fn type_size(&self, ty: &Type) -> i32 {
        match ty.kind() {
            TypeKind::Void => 0,
            TypeKind::Int => 4,
            TypeKind::Char => 1,
            TypeKind::Float => 4,
            TypeKind::Double | TypeKind::LongLong => 8,
            TypeKind::Pointer | TypeKind::Function => 4,
            TypeKind::Array => {
                let elem_count = ty.total_elements();
                let elem_size = self.elem_type_size(ty);
                elem_count * elem_size
            }
            TypeKind::Struct => {
                self.struct_defs.get(ty.name()).map(|f| {
                    f.iter().map(|field| self.type_size(&field.ty)).sum()
                }).unwrap_or(0)
            }
            TypeKind::Union => {
                self.union_defs.get(ty.name()).map(|f| {
                    f.iter().map(|field| self.type_size(&field.ty)).max().unwrap_or(0)
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
                    let sz = self.type_size(vty);
                    let aligned_sz = (sz + 3) & !3;
                    let local_offset = self.next_local_offset;
                    self.next_local_offset += aligned_sz;
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
                    if let Some(ref mut e) = init {
                        if vty.is_array() && matches!(e, Expr::InitList { .. }) {
                            if let Expr::InitList { ref mut elements, .. } = e {
                                let values = flatten_init_list(elements, &mut self.errors);
                                if base_kind(vty) == TypeKind::Char {
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
                                    let elem_size = self.elem_type_size(vty);
                                    let count = vty.total_elements();
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
                                            } else if matches!(elem, Expr::Identifier { .. }) {
                                                // 函数名等复杂表达式需要 gen_expr
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
                        } else if vty.is_struct() && matches!(e, Expr::InitList { .. }) {
                            if let Expr::InitList { ref mut elements, .. } = e {
                                let base_temp = self.get_temp_slot(0);
                                self.emit(OpCode::GetFrameBase, 0, loc);
                                self.emit(OpCode::PushConst, local_offset, loc);
                                self.emit(OpCode::Add, 0, loc);
                                self.emit(OpCode::StoreLocal, base_temp, loc);
                                let fields = self.struct_defs.get(vty.name()).cloned().unwrap_or_default();
                                for (i, elem) in elements.iter_mut().enumerate() {
                                    if i >= fields.len() { break; }
                                    let offset = fields.iter().take(i).map(|f| self.type_size(&f.ty)).sum::<i32>();
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
                        } else if vty.is_struct() {
                            self.gen_struct_copy_to_local(local_offset, e, loc);
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
                        } else {
                            self.gen_expr(e);
                            if vty.kind() == TypeKind::Float && e.ty().kind() != TypeKind::Float && e.ty().kind() != TypeKind::Double && e.ty().kind() != TypeKind::LongLong {
                                self.emit(OpCode::CastI2F, 0, loc);
                            }
                            if vty.kind() == TypeKind::Double && e.ty().kind() != TypeKind::Float && e.ty().kind() != TypeKind::Double && e.ty().kind() != TypeKind::LongLong {
                                self.emit(OpCode::CastI2D, 0, loc);
                            }
                            if vty.kind() == TypeKind::LongLong && e.ty().kind() != TypeKind::LongLong && e.ty().kind() != TypeKind::Double && e.ty().kind() != TypeKind::Float {
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
                        if sz == 8 && vty.kind() == TypeKind::Double {
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
                    let ret_is_float = self.func_table.get(&self.current_func).map(|m| m.return_type.kind() == TypeKind::Float || m.return_type.kind() == TypeKind::Double).unwrap_or(false);
                    if ret_is_float && v.ty().kind() != TypeKind::Float && v.ty().kind() != TypeKind::Double {
                        self.emit(OpCode::CastI2F, 0, loc);
                    } else if !ret_is_float && (v.ty().kind() == TypeKind::Float || v.ty().kind() == TypeKind::Double) {
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
            Expr::FloatLiteral { value, ty, .. } => {
                if ty.kind() == TypeKind::Double {
                    let idx = self.push_f64_constant(*value);
                    self.emit(OpCode::PushConstD, idx, &loc);
                } else {
                    let bits = (*value as f32).to_bits() as i32;
                    self.emit(OpCode::PushConstF, bits, &loc);
                }
            }
            Expr::LongLiteral { value, .. } => {
                let idx = self.push_i64_constant(*value);
                self.emit(OpCode::PushConstQ, idx, &loc);
            }
            Expr::StringLiteral { value, .. } => {
                let addr = self.string_mem_offset;
                let new_offset = addr + value.len() as u32 + 1;
                let new_offset = (new_offset + 3) & !3;
                if new_offset > crate::vm::vm::MEM_SIZE / 16 {
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
                let local_offset = self.resolve_local(name);
                if local_offset >= 0 {
                    if let Some(ty) = self.local_types.get(name) {
                        if ty.is_array() {
                            if local_offset < self.current_func_arg_bytes {
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
                                self.emit(OpCode::PushConst, crate::vm::vm::GLOBAL_START as i32 + global_offset, &loc);
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
            Expr::Binary { op, left, right, ty, .. } => {
                let left_is_ptr = left.ty().is_pointer() || left.ty().is_array();
                let right_is_ptr = right.ty().is_pointer() || right.ty().is_array();
                let result_is_double = ty.kind() == TypeKind::Double;
                let result_is_float = ty.kind() == TypeKind::Float;
                let result_is_long_long = ty.kind() == TypeKind::LongLong;
                let any_fp = result_is_double || result_is_float;

                // Short-circuit evaluation for && and ||
                if *op == BinaryOp::And || *op == BinaryOp::Or {
                    self.gen_expr(left);
                    match left.ty().kind() {
                        TypeKind::Float => self.emit(OpCode::CastF2I, 0, &loc),
                        TypeKind::Double => self.emit(OpCode::CastD2I, 0, &loc),
                        TypeKind::LongLong => self.emit(OpCode::CastQ2I, 0, &loc),
                        _ => {}
                    }
                    self.emit(OpCode::Dup, 0, &loc);
                    let end_jump = self.current_ip();
                    if *op == BinaryOp::And {
                        self.emit(OpCode::JumpIfZero, 0, &loc);
                    } else {
                        self.emit(OpCode::JumpIfNotZero, 0, &loc);
                    }
                    self.emit(OpCode::Pop, 0, &loc);
                    self.gen_expr(right);
                    match right.ty().kind() {
                        TypeKind::Float => self.emit(OpCode::CastF2I, 0, &loc),
                        TypeKind::Double => self.emit(OpCode::CastD2I, 0, &loc),
                        TypeKind::LongLong => self.emit(OpCode::CastQ2I, 0, &loc),
                        _ => {}
                    }
                    let end_ip = self.current_ip();
                    self.patch_jump(end_jump, end_ip);
                    return;
                }

                self.gen_expr(left);
                if any_fp && !left_is_ptr && left.ty().kind() != TypeKind::Float && left.ty().kind() != TypeKind::Double && left.ty().kind() != TypeKind::LongLong {
                    if result_is_double { self.emit(OpCode::CastI2D, 0, &loc); }
                    else { self.emit(OpCode::CastI2F, 0, &loc); }
                } else if result_is_double && left.ty().kind() == TypeKind::Float {
                    self.emit(OpCode::CastF2D, 0, &loc);
                } else if result_is_double && left.ty().kind() == TypeKind::LongLong {
                    self.emit(OpCode::CastQ2D, 0, &loc);
                } else if result_is_long_long && left.ty().kind() == TypeKind::Int {
                    self.emit(OpCode::CastI2Q, 0, &loc);
                }
                self.gen_expr(right);
                if any_fp && !right_is_ptr && right.ty().kind() != TypeKind::Float && right.ty().kind() != TypeKind::Double && right.ty().kind() != TypeKind::LongLong {
                    if result_is_double { self.emit(OpCode::CastI2D, 0, &loc); }
                    else { self.emit(OpCode::CastI2F, 0, &loc); }
                } else if result_is_double && right.ty().kind() == TypeKind::Float {
                    self.emit(OpCode::CastF2D, 0, &loc);
                } else if result_is_double && right.ty().kind() == TypeKind::LongLong {
                    self.emit(OpCode::CastQ2D, 0, &loc);
                } else if result_is_long_long && right.ty().kind() == TypeKind::Int {
                    self.emit(OpCode::CastI2Q, 0, &loc);
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
                        } else if result_is_double {
                            self.emit(OpCode::AddD, 0, &loc);
                        } else if result_is_float {
                            self.emit(OpCode::AddF, 0, &loc);
                        } else if result_is_long_long {
                            self.emit(OpCode::AddQ, 0, &loc);
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
                        } else if result_is_double {
                            self.emit(OpCode::SubD, 0, &loc);
                        } else if result_is_float {
                            self.emit(OpCode::SubF, 0, &loc);
                        } else if result_is_long_long {
                            self.emit(OpCode::SubQ, 0, &loc);
                        } else {
                            self.emit(OpCode::Sub, 0, &loc);
                        }
                    }
                    BinaryOp::Mul => {
                        if result_is_double { self.emit(OpCode::MulD, 0, &loc); }
                        else if result_is_float { self.emit(OpCode::MulF, 0, &loc); }
                        else if result_is_long_long { self.emit(OpCode::MulQ, 0, &loc); }
                        else { self.emit(OpCode::Mul, 0, &loc); }
                    }
                    BinaryOp::Div => {
                        if result_is_double { self.emit(OpCode::DivD, 0, &loc); }
                        else if result_is_float { self.emit(OpCode::DivF, 0, &loc); }
                        else if result_is_long_long { self.emit(OpCode::DivQ, 0, &loc); }
                        else { self.emit(OpCode::Div, 0, &loc); }
                    }
                    BinaryOp::Mod => {
                        if result_is_long_long { self.emit(OpCode::ModQ, 0, &loc); }
                        else { self.emit(OpCode::Mod, 0, &loc); }
                    }
                    BinaryOp::Eq => {
                        if result_is_double { self.emit(OpCode::EqD, 0, &loc); }
                        else if result_is_float { self.emit(OpCode::EqF, 0, &loc); }
                        else if result_is_long_long { self.emit(OpCode::EqQ, 0, &loc); }
                        else { self.emit(OpCode::Eq, 0, &loc); }
                    }
                    BinaryOp::Ne => {
                        if result_is_double { self.emit(OpCode::NeD, 0, &loc); }
                        else if result_is_float { self.emit(OpCode::NeF, 0, &loc); }
                        else if result_is_long_long { self.emit(OpCode::NeQ, 0, &loc); }
                        else { self.emit(OpCode::Ne, 0, &loc); }
                    }
                    BinaryOp::Lt => {
                        if result_is_double { self.emit(OpCode::LtD, 0, &loc); }
                        else if result_is_float { self.emit(OpCode::LtF, 0, &loc); }
                        else if result_is_long_long { self.emit(OpCode::LtQ, 0, &loc); }
                        else { self.emit(OpCode::Lt, 0, &loc); }
                    }
                    BinaryOp::Le => {
                        if result_is_double { self.emit(OpCode::LeD, 0, &loc); }
                        else if result_is_float { self.emit(OpCode::LeF, 0, &loc); }
                        else if result_is_long_long { self.emit(OpCode::LeQ, 0, &loc); }
                        else { self.emit(OpCode::Le, 0, &loc); }
                    }
                    BinaryOp::Gt => {
                        if result_is_double { self.emit(OpCode::GtD, 0, &loc); }
                        else if result_is_float { self.emit(OpCode::GtF, 0, &loc); }
                        else if result_is_long_long { self.emit(OpCode::GtQ, 0, &loc); }
                        else { self.emit(OpCode::Gt, 0, &loc); }
                    }
                    BinaryOp::Ge => {
                        if result_is_double { self.emit(OpCode::GeD, 0, &loc); }
                        else if result_is_float { self.emit(OpCode::GeF, 0, &loc); }
                        else if result_is_long_long { self.emit(OpCode::GeQ, 0, &loc); }
                        else { self.emit(OpCode::Ge, 0, &loc); }
                    }
                    BinaryOp::BitAnd => self.emit(OpCode::BitAnd, 0, &loc),
                    BinaryOp::BitOr => self.emit(OpCode::BitOr, 0, &loc),
                    BinaryOp::BitXor => self.emit(OpCode::BitXor, 0, &loc),
                    BinaryOp::Shl => self.emit(OpCode::Shl, 0, &loc),
                    BinaryOp::Shr => self.emit(OpCode::Shr, 0, &loc),
                    BinaryOp::And | BinaryOp::Or => {}, // handled above
                }
            }
            Expr::Unary { op, operand, .. } => {
                match op {
                    UnaryOp::Neg => {
                        self.gen_expr(operand);
                        if operand.ty().kind() == TypeKind::Double {
                            self.emit(OpCode::NegD, 0, &loc);
                        } else if operand.ty().kind() == TypeKind::Float {
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
                                if let Some(&offset) = self.local_indices.get(name) {
                                    self.emit(OpCode::GetFrameBase, 0, &loc);
                                    self.emit(OpCode::PushConst, offset, &loc);
                                    self.emit(OpCode::Add, 0, &loc);
                                } else if let Some(&offset) = self.global_indices.get(name) {
                                    self.emit(OpCode::PushConst, crate::vm::vm::GLOBAL_START as i32 + offset, &loc);
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
                        let base_ty = if operand.ty().is_pointer() {
                            base_kind(operand.ty())
                        } else {
                            TypeKind::Int
                        };
                        if base_ty == TypeKind::Double {
                            self.emit(OpCode::LoadMemD, 0, &loc);
                        } else if base_ty == TypeKind::LongLong {
                            self.emit(OpCode::LoadMemQ, 0, &loc);
                        } else {
                            self.emit(OpCode::LoadMem, 0, &loc);
                        }
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
                    let arg_ty_kind = arg.ty().kind();
                    let arg_ty = arg.ty();
                    if arg_ty.is_struct() {
                        let sz = self.type_size(arg_ty);
                        let words = (sz + 3) / 4;
                        if let Expr::Identifier { name: arg_name, .. } = arg {
                            if let Some(&offset) = self.local_indices.get(arg_name) {
                                for i in (0..words).rev() {
                                    self.emit(OpCode::LoadLocal, offset + i * 4, &loc);
                                }
                            } else if let Some(&offset) = self.global_indices.get(arg_name) {
                                for i in (0..words).rev() {
                                    self.emit(OpCode::LoadGlobal, offset + i * 4, &loc);
                                }
                            } else {
                                self.gen_expr(arg);
                                for _ in 1..words {
                                    self.emit(OpCode::PushConst, 0, &loc);
                                }
                            }
                        } else {
                            self.gen_expr(arg);
                            for _ in 1..words {
                                self.emit(OpCode::PushConst, 0, &loc);
                            }
                        }
                    } else if arg_ty.kind() == TypeKind::Double {
                        self.gen_expr(arg);
                        if self.func_index.contains_key(name) {
                            self.emit(OpCode::SplitD, 0, &loc);
                        }
                    } else if arg_ty.kind() == TypeKind::LongLong {
                        self.gen_expr(arg);
                        if self.func_index.contains_key(name) {
                            self.emit(OpCode::SplitQ, 0, &loc);
                        }
                    } else {
                        self.gen_expr(arg);
                        if (name == "printf" || name == "fprintf") && arg_ty_kind == TypeKind::Float {
                            self.emit(OpCode::CastF2D, 0, &loc);
                        }
                    }
                }
                if let Some(&idx) = self.func_index.get(name) {
                    self.emit(OpCode::Call, idx, &loc);
                } else {
                    if let Some(host_id) = crate::vm::host_func_id::by_user_name(name.as_str()) {
                        self.emit(OpCode::CallHost, host_id as i32, &loc);
                    } else {
                        self.report_error(&format!("未定义的函数 '{}'", name), &loc);
                        self.emit(OpCode::PushConst, 0, &loc);
                    }
                }
            }
            Expr::CallPtr { callee, args, .. } => {
                // Optimization: direct named call can use Call instead of CallPtr
                let is_direct_call = if let Expr::Identifier { name, .. } = callee.as_ref() {
                    self.func_index.contains_key(name)
                } else {
                    false
                };
                for arg in args.iter_mut().rev() {
                    let arg_ty = arg.ty().clone();
                    if arg_ty.is_struct() {
                        let sz = self.type_size(&arg_ty);
                        let words = (sz + 3) / 4;
                        if let Expr::Identifier { name: arg_name, .. } = arg {
                            if let Some(&offset) = self.local_indices.get(arg_name) {
                                for i in (0..words).rev() {
                                    self.emit(OpCode::LoadLocal, offset + i * 4, &loc);
                                }
                            } else if let Some(&offset) = self.global_indices.get(arg_name) {
                                for i in (0..words).rev() {
                                    self.emit(OpCode::LoadGlobal, offset + i * 4, &loc);
                                }
                            } else {
                                self.gen_expr(arg);
                                for _ in 1..words {
                                    self.emit(OpCode::PushConst, 0, &loc);
                                }
                            }
                        } else {
                            self.gen_expr(arg);
                            for _ in 1..words {
                                self.emit(OpCode::PushConst, 0, &loc);
                            }
                        }
                    } else if arg_ty.kind() == TypeKind::Double {
                        self.gen_expr(arg);
                        if is_direct_call {
                            self.emit(OpCode::SplitD, 0, &loc);
                        }
                    } else if arg_ty.kind() == TypeKind::LongLong {
                        self.gen_expr(arg);
                        if is_direct_call {
                            self.emit(OpCode::SplitQ, 0, &loc);
                        }
                    } else {
                        self.gen_expr(arg);
                        if let Expr::Identifier { name, .. } = callee.as_ref() {
                            if (name == "printf" || name == "fprintf") && arg_ty.kind() == TypeKind::Float {
                                self.emit(OpCode::CastF2D, 0, &loc);
                            }
                        }
                    }
                }
                if let Expr::Identifier { name, .. } = callee.as_ref() {
                    if let Some(&idx) = self.func_index.get(name) {
                        self.emit(OpCode::Call, idx, &loc);
                        return;
                    }
                    // Host function: direct CallHost
                    let host_id = self.resolve_host_func_id(name);
                    if host_id >= 0 {
                        self.emit(OpCode::CallHost, host_id, &loc);
                        return;
                    }
                }
                self.gen_expr(callee);
                self.emit(OpCode::CallPtr, args.len() as i32, &loc);
            }
            Expr::Index { array, index, ty, .. } => {
                self.gen_index(array, index, ty, &loc, false);
            }
            Expr::Member { object, member, ty, .. } => {
                self.gen_member_addr(object, member, &loc);
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
                if target_type.kind() == TypeKind::Double && expr.ty().kind() != TypeKind::Float && expr.ty().kind() != TypeKind::Double && expr.ty().kind() != TypeKind::LongLong {
                    self.emit(OpCode::CastI2D, 0, &loc);
                } else if target_type.kind() == TypeKind::Double && expr.ty().kind() == TypeKind::Float {
                    self.emit(OpCode::CastF2D, 0, &loc);
                } else if target_type.kind() == TypeKind::Double && expr.ty().kind() == TypeKind::LongLong {
                    self.emit(OpCode::CastQ2D, 0, &loc);
                } else if target_type.kind() == TypeKind::Float && expr.ty().kind() != TypeKind::Float && expr.ty().kind() != TypeKind::Double && expr.ty().kind() != TypeKind::LongLong {
                    self.emit(OpCode::CastI2F, 0, &loc);
                } else if target_type.kind() == TypeKind::Float && expr.ty().kind() == TypeKind::Double {
                    self.emit(OpCode::CastD2F, 0, &loc);
                } else if target_type.kind() == TypeKind::LongLong && expr.ty().kind() != TypeKind::LongLong && expr.ty().kind() != TypeKind::Double && expr.ty().kind() != TypeKind::Float {
                    self.emit(OpCode::CastI2Q, 0, &loc);
                } else if target_type.kind() == TypeKind::LongLong && expr.ty().kind() == TypeKind::Double {
                    self.emit(OpCode::CastD2Q, 0, &loc);
                } else if target_type.kind() != TypeKind::Float && target_type.kind() != TypeKind::Double && target_type.kind() != TypeKind::LongLong && expr.ty().kind() == TypeKind::Double {
                    self.emit(OpCode::CastD2I, 0, &loc);
                } else if target_type.kind() != TypeKind::Float && target_type.kind() != TypeKind::Double && target_type.kind() != TypeKind::LongLong && expr.ty().kind() == TypeKind::Float {
                    self.emit(OpCode::CastF2I, 0, &loc);
                } else if target_type.kind() != TypeKind::Float && target_type.kind() != TypeKind::Double && target_type.kind() != TypeKind::LongLong && expr.ty().kind() == TypeKind::LongLong {
                    self.emit(OpCode::CastQ2I, 0, &loc);
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
            if let Some(&offset) = self.local_indices.get(name) {
                self.emit(OpCode::GetFrameBase, 0, loc);
                self.emit(OpCode::PushConst, offset, loc);
                self.emit(OpCode::Add, 0, loc);
            } else if let Some(&offset) = self.global_indices.get(name) {
                self.emit(OpCode::PushConst, crate::vm::vm::GLOBAL_START as i32 + offset, loc);
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
                    bound_size = if ty.dims().is_empty() { ty.array_size() } else { ty.dims()[0] };
                    sym_idx = self.resolve_symbol_index(name);
                }
            } else if let Some(ty) = self.global_types.get(name) {
                if ty.is_array() {
                    bound_size = if ty.dims().is_empty() { ty.array_size() } else { ty.dims()[0] };
                    sym_idx = self.resolve_symbol_index(name);
                }
            }
        } else if let Expr::Index { .. } = array {
            if array.ty().is_array() && !array.ty().dims().is_empty() {
                bound_size = array.ty().dims()[0];
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

    fn gen_expr_with_cast(&mut self, expr: &mut Expr, target_is_fp: bool, target_is_double: bool, loc: &SourceLoc) {
        self.gen_expr(expr);
        let _target_is_long_long = !target_is_fp && expr.ty().kind() != TypeKind::Int && expr.ty().kind() != TypeKind::Char && expr.ty().kind() != TypeKind::Float && expr.ty().kind() != TypeKind::Double;
        // Note: target_is_long_long heuristic is approximate; caller ensures correct cast via Cast nodes
        if target_is_double && expr.ty().kind() != TypeKind::Float && expr.ty().kind() != TypeKind::Double && expr.ty().kind() != TypeKind::LongLong {
            self.emit(OpCode::CastI2D, 0, loc);
        } else if target_is_double && expr.ty().kind() == TypeKind::Float {
            self.emit(OpCode::CastF2D, 0, loc);
        } else if target_is_double && expr.ty().kind() == TypeKind::LongLong {
            self.emit(OpCode::CastQ2D, 0, loc);
        } else if !target_is_double && target_is_fp && expr.ty().kind() != TypeKind::Float && expr.ty().kind() != TypeKind::Double && expr.ty().kind() != TypeKind::LongLong {
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
                    self.emit(OpCode::GetFrameBase, 0, loc);
                    self.emit(OpCode::PushConst, offset, loc);
                    self.emit(OpCode::Add, 0, loc);
                } else if let Some(&offset) = self.global_indices.get(name) {
                    self.emit(OpCode::PushConst, crate::vm::vm::GLOBAL_START as i32 + offset, loc);
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

    /// 通用结构体/union 拷贝循环：通过闭包生成目标地址加载指令。
    fn gen_struct_copy_common<F>(&mut self, size: i32, src_expr: &mut Expr, mut dst_emit: F, loc: &SourceLoc)
    where
        F: FnMut(&mut Self, &SourceLoc, i32),
    {
        if size <= 0 { return; }
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
        self.gen_struct_copy_common(size, right, |gen, loc, i| {
            gen.emit(OpCode::LoadLocal, dst_temp, loc);
            if i > 0 {
                gen.emit(OpCode::PushConst, i * 4, loc);
                gen.emit(OpCode::Add, 0, loc);
            }
        }, loc);
    }

    fn gen_struct_copy_to_local(&mut self, local_offset: i32, right: &mut Expr, loc: &SourceLoc) {
        let size = self.type_size(right.ty());
        self.gen_struct_copy_common(size, right, |gen, loc, i| {
            gen.emit(OpCode::GetFrameBase, 0, loc);
            gen.emit(OpCode::PushConst, local_offset + i * 4, loc);
            gen.emit(OpCode::Add, 0, loc);
        }, loc);
    }

    fn gen_assign(&mut self, op: &AssignOp, left: &mut Expr, right: &mut Expr, loc: &SourceLoc) {
        let left_is_double = left.ty().kind() == TypeKind::Double;
        let left_is_float = left.ty().kind() == TypeKind::Float;
        let left_is_long_long = left.ty().kind() == TypeKind::LongLong;
        let left_is_fp = left_is_double || left_is_float;
        if left.ty().is_struct() && *op == AssignOp::Assign {
            self.gen_struct_copy(left, right, loc);
            return;
        }
        let emit_compound = |this: &mut Self, loc: &SourceLoc| {
            match op {
                AssignOp::AddAssign => {
                    if left_is_double { this.emit(OpCode::AddD, 0, loc); }
                    else if left_is_float { this.emit(OpCode::AddF, 0, loc); }
                    else { this.emit(OpCode::Add, 0, loc); }
                }
                AssignOp::SubAssign => {
                    if left_is_double { this.emit(OpCode::SubD, 0, loc); }
                    else if left_is_float { this.emit(OpCode::SubF, 0, loc); }
                    else { this.emit(OpCode::Sub, 0, loc); }
                }
                AssignOp::MulAssign => {
                    if left_is_double { this.emit(OpCode::MulD, 0, loc); }
                    else if left_is_float { this.emit(OpCode::MulF, 0, loc); }
                    else { this.emit(OpCode::Mul, 0, loc); }
                }
                AssignOp::DivAssign => {
                    if left_is_double { this.emit(OpCode::DivD, 0, loc); }
                    else if left_is_float { this.emit(OpCode::DivF, 0, loc); }
                    else { this.emit(OpCode::Div, 0, loc); }
                }
                AssignOp::ModAssign => {
                    if left_is_long_long { this.emit(OpCode::ModQ, 0, loc); }
                    else { this.emit(OpCode::Mod, 0, loc); }
                }
                _ => {}
            }
        };

        if let Expr::Identifier { name, .. } = left {
            let local_offset = self.resolve_local(name);
            if local_offset >= 0 {
                if *op != AssignOp::Assign {
                    if left_is_double { self.emit(OpCode::LoadLocalD, local_offset, loc); }
                    else if left_is_long_long { self.emit(OpCode::LoadLocalQ, local_offset, loc); }
                    else { self.emit(OpCode::LoadLocal, local_offset, loc); }
                    self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                    emit_compound(self, loc);
                } else {
                    self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                }
                if left_is_double { self.emit(OpCode::StoreLocalD, local_offset, loc); }
                else if left_is_long_long { self.emit(OpCode::StoreLocalQ, local_offset, loc); }
                else { self.emit(OpCode::StoreLocal, local_offset, loc); }
                if left_is_double { self.emit(OpCode::LoadLocalD, local_offset, loc); }
                else if left_is_long_long { self.emit(OpCode::LoadLocalQ, local_offset, loc); }
                else { self.emit(OpCode::LoadLocal, local_offset, loc); }
                return;
            }
            let global_offset = self.resolve_global(name);
            if global_offset >= 0 {
                if *op != AssignOp::Assign {
                    if left_is_double { self.emit(OpCode::LoadGlobalD, global_offset, loc); }
                    else if left_is_long_long { self.emit(OpCode::LoadGlobalQ, global_offset, loc); }
                    else { self.emit(OpCode::LoadGlobal, global_offset, loc); }
                    self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                    emit_compound(self, loc);
                } else {
                    self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                }
                if left_is_double { self.emit(OpCode::StoreGlobalD, global_offset, loc); }
                else if left_is_long_long { self.emit(OpCode::StoreGlobalQ, global_offset, loc); }
                else { self.emit(OpCode::StoreGlobal, global_offset, loc); }
                if left_is_double { self.emit(OpCode::LoadGlobalD, global_offset, loc); }
                else if left_is_long_long { self.emit(OpCode::LoadGlobalQ, global_offset, loc); }
                else { self.emit(OpCode::LoadGlobal, global_offset, loc); }
                return;
            }
        } else if let Expr::Index { array, index, ty, .. } = left {
            let result_ty = ty.clone();
            if *op != AssignOp::Assign {
                self.gen_index(array, index, &result_ty, loc, true);
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
                self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                emit_compound(self, loc);
                if result_ty.kind() == TypeKind::Char {
                    self.emit(OpCode::StoreMemByte, 0, loc);
                } else if result_ty.kind() == TypeKind::Double {
                    self.emit(OpCode::StoreMemD, 0, loc);
                } else if result_ty.kind() == TypeKind::LongLong {
                    self.emit(OpCode::StoreMemQ, 0, loc);
                } else {
                    self.emit(OpCode::StoreMem, 0, loc);
                }
            } else {
                self.gen_index(array, index, &result_ty, loc, true);
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
            }
            self.gen_index(array, index, &result_ty, loc, false);
            return;
        } else if let Expr::Unary { op: UnaryOp::Deref, operand, .. } = left {
            if *op != AssignOp::Assign {
                self.gen_expr(operand);
                self.emit(OpCode::Dup, 0, loc);
                if left_is_double { self.emit(OpCode::LoadMemD, 0, loc); }
                else if left_is_long_long { self.emit(OpCode::LoadMemQ, 0, loc); }
                else { self.emit(OpCode::LoadMem, 0, loc); }
                self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                emit_compound(self, loc);
                if left_is_double { self.emit(OpCode::StoreMemD, 0, loc); }
                else if left_is_long_long { self.emit(OpCode::StoreMemQ, 0, loc); }
                else { self.emit(OpCode::StoreMem, 0, loc); }
            } else {
                self.gen_expr(operand);
                self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                if left_is_double { self.emit(OpCode::StoreMemD, 0, loc); }
                else if left_is_long_long { self.emit(OpCode::StoreMemQ, 0, loc); }
                else { self.emit(OpCode::StoreMem, 0, loc); }
            }
            self.gen_expr(operand);
            if left_is_double { self.emit(OpCode::LoadMemD, 0, loc); }
            else if left_is_long_long { self.emit(OpCode::LoadMemQ, 0, loc); }
            else { self.emit(OpCode::LoadMem, 0, loc); }
            return;
        } else if let Expr::Member { object, member, .. } = left {
            if *op != AssignOp::Assign {
                self.gen_member_addr(object, member, loc);
                self.emit(OpCode::Dup, 0, loc);
                if left_is_double { self.emit(OpCode::LoadMemD, 0, loc); }
                else if left_is_long_long { self.emit(OpCode::LoadMemQ, 0, loc); }
                else { self.emit(OpCode::LoadMem, 0, loc); }
                self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                emit_compound(self, loc);
                if left_is_double { self.emit(OpCode::StoreMemD, 0, loc); }
                else if left_is_long_long { self.emit(OpCode::StoreMemQ, 0, loc); }
                else { self.emit(OpCode::StoreMem, 0, loc); }
            } else {
                self.gen_member_addr(object, member, loc);
                self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
                if left_is_double { self.emit(OpCode::StoreMemD, 0, loc); }
                else if left_is_long_long { self.emit(OpCode::StoreMemQ, 0, loc); }
                else { self.emit(OpCode::StoreMem, 0, loc); }
            }
            self.gen_member_addr(object, member, loc);
            if left_is_double { self.emit(OpCode::LoadMemD, 0, loc); }
            else if left_is_long_long { self.emit(OpCode::LoadMemQ, 0, loc); }
            else { self.emit(OpCode::LoadMem, 0, loc); }
            return;
        }

        self.report_error("赋值目标不支持", loc);
        self.gen_expr_with_cast(right, left_is_fp, left_is_double, loc);
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

fn flatten_init_list(elements: &[Expr], errors: &mut Vec<String>) -> Vec<i32> {
    let mut result = Vec::new();
    for elem in elements {
        match elem {
            Expr::Literal { value, .. } => result.push(*value),
            Expr::LongLiteral { value, .. } => {
                if *value < i32::MIN as i64 || *value > i32::MAX as i64 {
                    errors.push(format!("初始化列表中的 long long 常量 {} 超出 int 范围，无法用于此上下文", value));
                    result.push(0);
                } else {
                    result.push(*value as i32);
                }
            }
            Expr::FloatLiteral { value, .. } => result.push((*value as f32).to_bits() as i32),
            Expr::InitList { elements: sub, .. } => result.extend(flatten_init_list(sub, errors)),
            Expr::Unary { op: UnaryOp::Neg, operand, .. } => {
                if let Expr::Literal { value, .. } = operand.as_ref() {
                    result.push(-*value);
                } else if let Expr::LongLiteral { value, .. } = operand.as_ref() {
                    if *value < i32::MIN as i64 || *value > i32::MAX as i64 {
                        errors.push(format!("初始化列表中的 long long 常量 {} 超出 int 范围，无法用于此上下文", value));
                        result.push(0);
                    } else {
                        result.push(-*value as i32);
                    }
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
    if !arr_type.is_array() || arr_type.dims().is_empty() { return elem_size; }
    let mut stride = elem_size;
    for i in 1..arr_type.dims().len() {
        stride *= if arr_type.dims()[i] > 0 { arr_type.dims()[i] } else { 0 };
    }
    stride
}

#[derive(Debug, Clone)]
pub struct CompileOutput {
    pub code: Vec<Instruction>,
    pub globals_init_32: Vec<(u32, i32)>,
    pub globals_init_64: Vec<(u32, u64)>,
    pub func_table: HashMap<String, FuncMeta>,
    pub func_index: HashMap<String, i32>,
    pub string_data: Vec<(u32, String)>,
    pub source_map: Vec<(u32, VMSourceLoc)>,
    pub symbols: Vec<VMSymbol>,
    pub struct_defs: HashMap<String, Vec<StructField>>,
    pub union_defs: HashMap<String, Vec<StructField>>,
    pub f64_constants: Vec<f64>,
    pub i64_constants: Vec<i64>,
}
