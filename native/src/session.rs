use crate::compiler::ast::{SourceLoc, Type};
use crate::vm::instruction::Instruction;
use crate::vm::vm::CideVM;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompileUnit {
    pub filename: String,
    pub source: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    pub line: i32,
    pub column: i32,
    pub error_code: i32,
    pub severity: i32,
    pub message: String,
    pub fix_suggestion: String,
    pub fix_kind: i32,
    pub replace_start_line: i32,
    pub replace_start_column: i32,
    pub replace_end_line: i32,
    pub replace_end_column: i32,
    pub replacement_text: String,
}


#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FuncMeta {
    pub ip: usize,
    pub arg_count: i32,
    pub local_count: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Symbol {
    pub name: String,
    pub addr: u32,
    pub is_local: bool,
    pub ty: Type,
    pub scope_depth: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AlgorithmMatch {
    pub name: String,
    pub display_name: String,
    pub func_name: String,
    pub confidence: i32,
    pub suggestion: String,
    pub line: i32,
    pub vis_events: Vec<(i32, i32, String)>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CompileState {
    pub errors: String,
    pub errors_buffer: String,
    pub compile_units: Vec<CompileUnit>,
    pub compiled: bool,
    pub bytecode: Vec<Instruction>,
    pub globals_init: Vec<i32>,
    pub diagnostics: Vec<Diagnostic>,
    pub source_map: Vec<(u32, SourceLoc)>,
    pub func_table: HashMap<String, FuncMeta>,
    pub func_index: HashMap<String, i32>,
    pub string_data: Vec<(u32, String)>,
    pub symbols: Vec<Symbol>,
    pub algorithm_matches: Vec<AlgorithmMatch>,
    pub struct_fields: HashMap<String, Vec<(String, i32)>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TraceEntry {
    pub line: i32,
    pub operation: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VariableSnapshot {
    pub name: String,
    pub addr: u32,
    pub is_local: bool,
    pub ty: Type,
    pub value: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisEvent {
    pub ty: i32,
    pub line: i32,
    pub extra: [i32; 3],
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RuntimeState {
    pub error: String,
    pub output_lines: Vec<String>,
    pub running: bool,
    pub trace: Vec<TraceEntry>,
    pub current_line: i32,
    pub input_lines: Vec<String>,
    pub input_index: usize,
    pub step_mode: bool,
    pub step_count: i32,
    pub variable_snapshot: Vec<VariableSnapshot>,
    pub vis_event_cache: Vec<VisEvent>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryRegion {
    pub addr: u32,
    pub size: i32,
    pub name: String,
    pub ty: String,
    pub is_heap: bool,
    pub is_freed: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FreeBlock {
    pub addr: u32,
    pub size: i32,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MemoryState {
    pub regions: Vec<MemoryRegion>,
    pub free_list: Vec<FreeBlock>,
    pub heap_offset: u32,
    pub alloc_counter: i32,
}

pub struct Session {
    pub compile: CompileState,
    pub runtime: RuntimeState,
    pub memory: MemoryState,
    pub vm: Option<CideVM>,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            compile: CompileState::default(),
            runtime: RuntimeState::default(),
            memory: MemoryState::default(),
            vm: Some(CideVM::default()),
        }
    }
}
