//! Cide 字节码生成器。
//!
//! 从 `cide_native::compiler::codegen` 拆分而来，负责将类型检查后的 AST 编译为 CideVM 字节码。

// TODO(#D08): `codegen/stmt.rs` 已拆分为 `stmt/` 子模块；`codegen/mod.rs` 当前 755 行已达标。
// 未来如继续偿还债务，可将全局变量/类/vtable 注册逻辑下沉到 `codegen/decl.rs`。
use cide_ast::*;
use cide_runtime::instruction::Instruction;
use cide_runtime::opcode::OpCode;
use cide_runtime::type_utils::{base_kind, immediate_base_kind};
use cide_runtime::{FuncMeta, Symbol};
use cide_shared::SourceLoc;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
struct ShadowEntry {
    name: String,
    old_offset: Option<i32>,
    old_type: Option<Type>,
    old_sym_idx: Option<i32>,
}

#[derive(Debug, Clone)]
struct ClassVarEntry {
    offset: i32,
    class_name: String,
}

#[derive(Debug, Clone, Default)]
struct ScopeFrame {
    shadows: Vec<ShadowEntry>,
    /// 在当前 scope 中声明的类类型局部变量，按声明顺序排列。
    /// 作用域退出时按 LIFO（逆序）调用析构函数。
    class_vars: Vec<ClassVarEntry>,
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
    static_local_indices: HashMap<String, i32>,
    static_local_types: HashMap<String, Type>,
    next_local_offset: i32,
    local_scope_stack: Vec<ScopeFrame>,
    /// 当前 loop 对应的 scope 深度栈，与 loop_start_ips 同步 push/pop。
    /// 用于 break/continue 时计算需要析构的 scope 层数。
    loop_scope_depths: Vec<usize>,
    temp_slot0: i32,
    temp_slot1: i32,
    temp_slot2: i32,
    temp_slot3: i32,
    globals_init_32: Vec<(u32, i32)>,
    globals_init_64: Vec<(u32, u64)>,
    next_global_offset: i32,
    f64_constants: Vec<f64>,
    i64_constants: Vec<i64>,
    symbols: Vec<Symbol>,
    sym_index: HashMap<String, i32>,
    struct_defs: HashMap<String, Vec<StructField>>,
    union_defs: HashMap<String, Vec<StructField>>,
    class_defs: HashMap<String, ClassDecl>,
    class_sizes: HashMap<String, i32>,
    class_vtables: HashMap<String, u32>,
    string_data: Vec<(u32, String)>,
    /// 全局变量初始化中遇到的字符串字面量，需在 Pass 1 结束后统一分配地址并回填。
    /// 避免字符串区与全局变量区重叠。
    pending_string_inits: Vec<(u32, String)>,
    source_map: Vec<(u32, SourceLoc)>,
    break_patches: Vec<usize>,
    continue_patches: Vec<usize>,
    loop_start_ips: Vec<usize>,
    goto_patches: HashMap<String, Vec<usize>>,
    label_ips: HashMap<String, usize>,
    // Lambda class name -> set of by-reference captured field names
    lambda_by_ref_fields: HashMap<String, HashSet<String>>,
}

impl Default for BytecodeGen {
    fn default() -> Self {
        Self::new()
    }
}

impl BytecodeGen {
    pub fn new() -> Self {
        Self::with_mode(false)
    }

    /// `is_library_mode` 为 true 时（预编译 Bytecode Libc 本身），不预注册固定索引，
    /// 让函数索引从 0 开始顺序分配，避免与当前内嵌的 bytecode_libc_index.rs 产生循环依赖。
    pub fn with_mode(is_library_mode: bool) -> Self {
        use cide_runtime::bytecode_libc_index::{
            bytecode_libc_index, BYTECODE_LIBC_ALL_FUNCS, BYTECODE_LIBC_BASE_INDEX, BYTECODE_LIBC_FUNC_COUNT,
            BYTECODE_LIBC_GLOBALS_RESERVED,
        };

        let mut func_index = HashMap::new();
        let next_func_idx = if is_library_mode {
            0
        } else {
            // 预注册 Bytecode Libc 函数到固定索引段
            for &name in BYTECODE_LIBC_ALL_FUNCS.iter() {
                if let Some(idx) = bytecode_libc_index(name) {
                    func_index.insert(name.to_string(), idx);
                }
            }
            BYTECODE_LIBC_BASE_INDEX + BYTECODE_LIBC_FUNC_COUNT as i32 + 1
        };

        Self {
            code: Vec::new(),
            errors: Vec::new(),
            func_table: HashMap::new(),
            func_index,
            next_func_idx,
            current_func: String::new(),
            current_func_arg_count: 0,
            current_func_arg_bytes: 0,
            global_indices: HashMap::new(),
            global_types: HashMap::new(),
            local_indices: HashMap::new(),
            local_types: HashMap::new(),
            static_local_indices: HashMap::new(),
            static_local_types: HashMap::new(),
            next_local_offset: 0,
            local_scope_stack: Vec::new(),
            loop_scope_depths: Vec::new(),
            temp_slot0: -1,
            temp_slot1: -1,
            temp_slot2: -1,
            temp_slot3: -1,
            globals_init_32: Vec::new(),
            globals_init_64: Vec::new(),
            next_global_offset: BYTECODE_LIBC_GLOBALS_RESERVED as i32,
            f64_constants: Vec::new(),
            i64_constants: Vec::new(),
            symbols: Vec::new(),
            sym_index: HashMap::new(),
            struct_defs: HashMap::new(),
            union_defs: HashMap::new(),
            class_defs: HashMap::new(),
            class_sizes: HashMap::new(),
            class_vtables: HashMap::new(),
            string_data: Vec::new(),
            pending_string_inits: Vec::new(),
            source_map: Vec::new(),
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
            loop_start_ips: Vec::new(),
            goto_patches: HashMap::new(),
            label_ips: HashMap::new(),
            lambda_by_ref_fields: HashMap::new(),
        }
    }

    pub fn generate(mut self, program: &mut ProgramNode) -> Result<CompileOutput, Vec<String>> {
        self.code.push(Instruction::new(OpCode::Nop, 0, SourceLoc::default()));

        for s in &program.structs {
            self.struct_defs.insert(s.name.clone(), s.fields.clone());
        }
        for u in &program.unions {
            self.union_defs.insert(u.name.clone(), u.fields.clone());
        }

        // Register classes (C++ extension)
        for c in &program.classes {
            self.class_defs.insert(c.name.clone(), c.clone());
        }
        // Register nested structs/classes from class members
        fn collect_nested(
            class: &cide_ast::ClassDecl,
            struct_defs: &mut HashMap<String, Vec<cide_ast::StructField>>,
            class_defs: &mut HashMap<String, cide_ast::ClassDecl>,
        ) {
            use cide_ast::ClassMember;
            for member in &class.members {
                match member {
                    ClassMember::NestedStruct { decl, .. } => {
                        struct_defs.insert(decl.name.clone(), decl.fields.clone());
                    }
                    ClassMember::NestedClass { decl, .. } => {
                        class_defs.insert(decl.name.clone(), decl.clone());
                        collect_nested(decl, struct_defs, class_defs);
                    }
                    _ => {}
                }
            }
        }
        for c in &program.classes {
            collect_nested(c, &mut self.struct_defs, &mut self.class_defs);
        }
        // Compute class sizes with topological ordering (base classes first)
        let mut pending: Vec<String> = self.class_defs.keys().cloned().collect();
        while !pending.is_empty() {
            let mut resolved = Vec::new();
            for class_name in &pending {
                let class = match self.class_defs.get(class_name) {
                    Some(c) => c,
                    None => continue,
                };
                let mut can_compute = true;
                if let Some(ref base) = class.base {
                    if !self.class_sizes.contains_key(base) {
                        can_compute = false;
                    }
                }
                if can_compute {
                    let needs_vptr = class.vtable.is_some();
                    let mut size = if needs_vptr { 4 } else { 0 };
                    if let Some(ref base_name) = class.base {
                        size = self
                            .class_sizes
                            .get(base_name)
                            .copied()
                            .unwrap_or(if needs_vptr { 4 } else { 0 });
                    }
                    for member in &class.members {
                        if let ClassMember::Field { ty, .. } = member {
                            size += self.type_size(ty);
                        }
                    }
                    self.class_sizes.insert(class_name.clone(), size);
                    resolved.push(class_name.clone());
                }
            }
            if resolved.is_empty() && !pending.is_empty() {
                // Circular inheritance or missing base — break to avoid infinite loop
                for class_name in &pending {
                    self.class_sizes.insert(class_name.clone(), 4);
                }
                break;
            }
            pending.retain(|n| !resolved.contains(n));
        }

        // Register builtin container sizes
        for (cpp_name, cide_name) in cide_cpp_frontend::builtin_layout::builtin_class_mappings() {
            if let Some(layout) = cide_cpp_frontend::builtin_layout::builtin_class_layout(cide_name) {
                for name in [cpp_name, cide_name] {
                    self.class_sizes.entry(name.to_string()).or_insert(layout.size);
                }
            }
        }

        // Pre-fill func_index so global initializers can resolve function names
        for f in &program.funcs {
            if f.body.is_none() {
                continue;
            }
            self.func_index.insert(f.name.clone(), self.next_func_idx);
            self.next_func_idx += 1;
        }

        // Pass 1: Register globals (byte offsets)
        // First: non-extern definitions
        for g in &program.globals {
            if g.is_extern {
                continue;
            }
            if g.ty.is_vla() {
                self.errors.push(format!("全局作用域不支持变长数组(VLA) '{}'", g.name));
                continue;
            }
            let sz = self.type_size(&g.ty);
            let offset = self.next_global_offset;
            self.global_indices.insert(g.name.clone(), offset);
            self.global_types.insert(g.name.clone(), g.ty.clone());
            if let Some(ref init) = g.init {
                match init {
                    Expr::InitList { elements, .. } => {
                        let has_designators = elements.iter().any(|e| !e.designators.is_empty());
                        if has_designators {
                            self.report_error("全局变量的 designated initializer 暂不支持", &g.loc);
                            continue;
                        }
                        let is_char_array = if let Type::Array { element, .. } = &g.ty {
                            element.kind() == TypeKind::Char
                        } else {
                            false
                        };
                        if is_char_array {
                            let values = flatten_init_list(elements, &mut self.errors);
                            for i in 0..sz as usize {
                                self.globals_init_32
                                    .push((offset as u32 + i as u32, values.get(i).copied().unwrap_or(0)));
                            }
                        } else if g.ty.is_struct()
                            || g.ty.is_class()
                            || (g.ty.is_array() && elements.iter().any(|e| matches!(&e.value, Expr::InitList { .. })))
                        {
                            // Struct or nested array init: use recursive expansion
                            self.flatten_global_init(&g.ty, init, offset as u32);
                        } else {
                            // Non-char array: handle element-by-element
                            let elem_size = self.elem_type_size(&g.ty);
                            let count = g.ty.total_elements();
                            if elem_size == 8 {
                                for i in 0..count as usize {
                                    let addr = offset as u32 + (i as u32) * elem_size as u32;
                                    let val64 = if let Some(elem) = elements.get(i) {
                                        match &elem.value {
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
                    Expr::Identifier { name, .. } => {
                        // 全局函数指针初始化：int (*fp)(int) = myFunc;
                        if let Some(&idx) = self.func_index.get(name) {
                            self.globals_init_32.push((offset as u32, idx));
                        }
                    }
                    _ => {}
                }
            }
            self.sym_index.insert(g.name.clone(), self.symbols.len() as i32);
            self.symbols.push(Symbol {
                name: g.name.clone(),
                addr: offset as u32,
                is_local: false,
                ty: g.ty.clone(),
                scope_depth: 0,
                func_name: String::new(),
            });
            self.next_global_offset += sz;
        }

        // Second: allocate placeholder for extern globals without a definition
        for g in &program.globals {
            if !g.is_extern || self.global_indices.contains_key(&g.name) {
                continue;
            }
            let sz = self.type_size(&g.ty).max(4);
            let offset = self.next_global_offset;
            self.global_indices.insert(g.name.clone(), offset);
            self.global_types.insert(g.name.clone(), g.ty.clone());
            self.sym_index.insert(g.name.clone(), self.symbols.len() as i32);
            self.symbols.push(Symbol {
                name: g.name.clone(),
                addr: offset as u32,
                is_local: false,
                ty: g.ty.clone(),
                scope_depth: 0,
                func_name: String::new(),
            });
            self.next_global_offset += sz;
        }

        // Allocate vtables in global memory for virtual dispatch (C++ extension)
        for c in &program.classes {
            if let Some(ref vtable) = c.vtable {
                let entries = &vtable.entries;
                let vtable_size = entries.len() as i32 * 4;
                let vtable_offset = self.next_global_offset;
                self.next_global_offset += vtable_size;
                self.class_vtables.insert(c.name.clone(), vtable_offset as u32);
                for (i, (method_name, _)) in entries.iter().enumerate() {
                    let mangled = format!("{}__{}", c.name, method_name);
                    let func_idx = self.func_index.get(&mangled).copied().unwrap_or(0);
                    self.globals_init_32.push((vtable_offset as u32 + i as u32 * 4, func_idx));
                }
            }
        }

        // 回填全局变量初始化中的字符串字面量地址
        let pending = std::mem::take(&mut self.pending_string_inits);
        for (base_offset, value) in pending {
            let aligned = ((value.len() + 1) as u32 + 3) & !3;
            let str_addr = cide_runtime::GLOBAL_START + self.next_global_offset as u32;
            self.string_data.push((str_addr, value));
            self.next_global_offset += aligned as i32;
            self.globals_init_32.push((base_offset, str_addr as i32));
        }

        // Pass 2: Register function metadata (func_index already filled above)
        for f in &program.funcs {
            if f.body.is_none() {
                continue;
            }
            let param_sizes: Vec<i32> = f.params.iter().map(|p| self.type_size(&p.ty)).collect();
            self.func_table.insert(
                f.name.clone(),
                FuncMeta {
                    ip: 0,
                    arg_count: f.params.len() as i32,
                    param_count: f.params.len() as i32,
                    local_count: 0,
                    param_sizes: param_sizes.clone(),
                    return_type: f.return_type.clone(),
                    is_variadic: f.is_variadic,
                },
            );
        }

        // Pass 3: Generate function bodies
        for f in &mut program.funcs {
            if f.body.is_none() {
                continue;
            }
            let func_ip = self.current_ip();
            if let Some(meta) = self.func_table.get_mut(&f.name) {
                meta.ip = func_ip;
            }
            self.enter_function(&f.name, &f.params, f.is_variadic);
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
            // 支持 `main(int argc, char *argv[])`：按 C 调用约定从右到左入栈，
            // VM do_call 先 pop 第一个参数，因此栈顶必须是 argc，下面是 argv。
            if let Some(main_meta) = self.func_table.get("main") {
                if main_meta.param_count == 2 {
                    self.emit(OpCode::PushArgv, 0, &SourceLoc::default());
                    self.emit(OpCode::PushArgc, 0, &SourceLoc::default());
                }
            }
            self.emit(OpCode::Call, main_idx, &SourceLoc { line: 0, column: 0 });
            self.emit(OpCode::Ret, 0, &SourceLoc { line: 0, column: 0 });
            self.code[0] = Instruction::new(OpCode::Jump, wrapper_ip as i32, SourceLoc::default());
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
        let vm_loc = SourceLoc {
            line: loc.line,
            column: loc.column,
        };
        self.code.push(Instruction::new(op, operand, vm_loc));
        if loc.line > 0 {
            self.source_map.push((ip, vm_loc));
        }
    }

    fn current_ip(&self) -> usize {
        self.code.len()
    }

    fn patch_jump(&mut self, ip: usize, target: usize) {
        if ip < self.code.len() {
            self.code[ip].operand = target as i32;
        }
    }

    fn report_error(&mut self, msg: &str, loc: &SourceLoc) {
        self.errors.push(format!("第 {} 行：{}", loc.line, msg));
    }

    fn enter_scope(&mut self) {
        self.local_scope_stack.push(ScopeFrame::default());
    }

    /// 在作用域退出时，先按 LIFO 顺序调用当前 scope 中类类型变量的析构函数，
    /// 再恢复被 shadow 的外部变量。
    fn exit_scope(&mut self) {
        if let Some(frame) = self.local_scope_stack.pop() {
            // 逆序调用析构函数（C++ 销毁顺序与构造顺序相反）
            for cv in frame.class_vars.iter().rev() {
                self.emit_class_dtor(&cv.class_name, cv.offset, &SourceLoc { line: 0, column: 0 });
            }
            for entry in frame.shadows {
                if let Some(old) = entry.old_offset {
                    self.local_indices.insert(entry.name.clone(), old);
                } else {
                    self.local_indices.remove(&entry.name);
                }
                if let Some(old) = entry.old_type {
                    self.local_types.insert(entry.name.clone(), old);
                } else {
                    self.local_types.remove(&entry.name);
                }
                if let Some(old) = entry.old_sym_idx {
                    self.sym_index.insert(entry.name, old);
                } else {
                    self.sym_index.remove(&entry.name);
                }
            }
        }
    }

    fn record_scope_var(&mut self, name: &str) {
        if let Some(frame) = self.local_scope_stack.last_mut() {
            let old_offset = self.local_indices.get(name).copied();
            let old_type = self.local_types.get(name).cloned();
            let old_sym_idx = self.sym_index.get(name).copied();
            frame.shadows.push(ShadowEntry {
                name: name.to_string(),
                old_offset,
                old_type,
                old_sym_idx,
            });
        }
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
            3 => &mut self.temp_slot3,
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
            TypeKind::Class | TypeKind::Pointer if base_kind(object_type) == TypeKind::Class => {
                self.get_class_member_offset(object_type.name(), member_name)
            }
            TypeKind::Reference | TypeKind::RValueRef => {
                if let Some(base) = object_type.reference_base() {
                    self.get_member_offset(base, member_name)
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    fn push_f64_constant(&mut self, val: f64) -> i32 {
        if let Some(idx) = self.f64_constants.iter().position(|&v| v.to_bits() == val.to_bits()) {
            return idx as i32;
        }
        let idx = self.f64_constants.len() as i32;
        self.f64_constants.push(val);
        idx
    }

    fn push_i64_constant(&mut self, val: i64) -> i32 {
        if let Some(idx) = self.i64_constants.iter().position(|&v| v == val) {
            return idx as i32;
        }
        let idx = self.i64_constants.len() as i32;
        self.i64_constants.push(val);
        idx
    }

    fn ptr_step_size(&self, ty: &Type) -> i32 {
        match ty.kind() {
            TypeKind::Pointer => {
                if let Type::Pointer { pointee, .. } = ty {
                    self.type_size(pointee)
                } else {
                    4
                }
            }
            TypeKind::Array => compute_stride(ty, self.elem_type_size(ty)),
            _ => 1,
        }
    }

    fn elem_type_size(&self, arr_type: &Type) -> i32 {
        let (elem_kind, elem_type) = if let Type::Array { element, .. } = arr_type {
            (element.kind(), element.as_ref())
        } else {
            (base_kind(arr_type), arr_type)
        };
        match elem_kind {
            TypeKind::Char => 1,
            TypeKind::Int | TypeKind::Pointer | TypeKind::Float => 4,
            TypeKind::Double | TypeKind::LongLong => 8,
            TypeKind::Struct => self
                .struct_defs
                .get(elem_type.name())
                .map(|f| f.iter().map(|field| self.type_size(&field.ty)).sum())
                .unwrap_or(4),
            TypeKind::Class => self.class_sizes.get(elem_type.name()).copied().unwrap_or(4),
            TypeKind::Union => self
                .union_defs
                .get(elem_type.name())
                .map(|f| f.iter().map(|field| self.type_size(&field.ty)).max().unwrap_or(0))
                .unwrap_or(4),
            _ => 4,
        }
    }

    fn resolve_host_func_id(&self, name: &str) -> i32 {
        cide_runtime::host_func_id::by_user_name(name).map(|id| id as i32).unwrap_or(-1)
    }

    fn type_size(&self, ty: &Type) -> i32 {
        compute_type_size(ty, &self.struct_defs, &self.union_defs, &self.class_sizes)
    }

    // =====================================================================
    // Statement / Expression dispatch
    // =====================================================================

    // =====================================================================
    // Statement / Expression dispatch
    // =====================================================================
}

mod cpp;
mod func;
mod init;
pub(crate) use init::{compute_stride, flatten_init_list, stmt_loc};
mod expr;
mod stmt;
pub(crate) use stmt::StmtGen;

#[derive(Debug, Clone)]
pub struct CompileOutput {
    pub code: Vec<Instruction>,
    pub globals_init_32: Vec<(u32, i32)>,
    pub globals_init_64: Vec<(u32, u64)>,
    pub func_table: HashMap<String, FuncMeta>,
    pub func_index: HashMap<String, i32>,
    pub string_data: Vec<(u32, String)>,
    pub source_map: Vec<(u32, SourceLoc)>,
    pub symbols: Vec<Symbol>,
    pub struct_defs: HashMap<String, Vec<StructField>>,
    pub union_defs: HashMap<String, Vec<StructField>>,
    pub f64_constants: Vec<f64>,
    pub i64_constants: Vec<i64>,
}

#[cfg(test)]
mod tests;
