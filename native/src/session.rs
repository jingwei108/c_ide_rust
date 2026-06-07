use crate::compiler::ast::{SourceLoc, Type};
use crate::engine::completion::CompletionSnapshot;
use crate::vm::instruction::Instruction;
use crate::vm::vfs::VirtualFileSystem;
use crate::vm::vm::CideVM;
use flutter_rust_bridge::frb;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompileUnit {
    pub filename: String,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct CodeFile {
    pub filename: String,
    pub source: String,
}

#[frb]
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
    pub filename: String,
}


#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FuncMeta {
    pub ip: usize,
    /// 参数总 word 数（以 4-byte words 计），供 Call 指令弹栈使用。
    pub arg_count: i32,
    /// 参数个数（供 call_user_function 使用，与总 word 数不同）。
    pub param_count: i32,
    pub local_count: i32,
    pub param_sizes: Vec<i32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Symbol {
    pub name: String,
    pub addr: u32,
    pub is_local: bool,
    pub ty: Type,
    pub scope_depth: i32,
    pub func_name: String,
}

#[frb]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AlgorithmMatch {
    pub name: String,
    pub display_name: String,
    pub func_name: String,
    pub confidence: i32,
    pub suggestion: String,
    pub line: i32,
    pub vis_events: Vec<VisEvent>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CompileState {
    pub errors: String,
    pub compile_units: Vec<CompileUnit>,
    pub compiled: bool,
    pub bytecode: Vec<Instruction>,
    pub globals_init: Vec<(u32, i32)>,
    pub globals_init_64: Vec<(u32, u64)>,
    pub f64_constants: Vec<f64>,
    pub i64_constants: Vec<i64>,
    pub diagnostics: Vec<Diagnostic>,
    pub source_map: Vec<(u32, SourceLoc)>,
    pub func_table: HashMap<String, FuncMeta>,
    pub func_index: HashMap<String, i32>,
    pub string_data: Vec<(u32, String)>,
    pub symbols: Vec<Symbol>,
    pub algorithm_matches: Vec<AlgorithmMatch>,
    pub struct_fields: HashMap<String, Vec<(String, i32)>>,
    /// 智能补全快照：每次成功编译后从 AST 提取的符号表
    pub completion_snapshot: CompletionSnapshot,
}

#[frb]
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
    pub value: i64,
}

#[frb]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisEvent {
    pub ty: i32,
    pub line: i32,
    pub extra0: i32,
    pub extra1: i32,
    pub extra2: i32,
    pub context: String,
}

/// 执行路径热力图：记录每行源代码被执行的次数。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ExecutionHeatmap {
    pub line_counts: HashMap<i32, u64>,
}

impl ExecutionHeatmap {
    pub fn record(&mut self, line: i32) {
        if line > 0 {
            *self.line_counts.entry(line).or_insert(0) += 1;
        }
    }

    pub fn max_count(&self) -> u64 {
        self.line_counts.values().copied().max().unwrap_or(0)
    }

    pub fn clear(&mut self) {
        self.line_counts.clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum InputMode {
    #[default]
    Interactive,
    Batch,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RuntimeState {
    pub error: String,
    pub error_buffer: String,
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
    pub rand_seed: u32,
    pub input_char_offset: usize,
    pub waiting_input: bool,
    pub heatmap: ExecutionHeatmap,
    pub input_mode: InputMode,
    pub ungetc_char: Option<i32>,
}

impl RuntimeState {
    pub fn output(&self) -> String {
        self.output_lines.join("\n")
    }
}

#[frb]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryRegion {
    pub addr: u32,
    pub size: i32,
    pub name: String,
    pub ty: String,
    pub is_heap: bool,
    pub is_freed: bool,
    /// 分配时的源码行号（教学用途）
    pub alloc_line: i32,
    /// 分配方式，如 "malloc" / "realloc" / "fopen"
    pub alloc_by: String,
}

#[frb]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryFragment {
    pub addr: u32,
    pub size: i32,
}

#[frb]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HeapStats {
    /// 总堆空间（heap_offset - HEAP_START），字节
    pub total_heap: i32,
    /// 已分配且未释放的堆内存，字节
    pub allocated: i32,
    /// 外部碎片（free_list 中所有块之和），字节
    pub fragmented: i32,
    /// 碎片率（0~100）
    pub fragmentation_rate: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FreeBlock {
    pub addr: u32,
    pub size: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryState {
    pub regions: Vec<MemoryRegion>,
    pub free_list: Vec<FreeBlock>,
    pub heap_offset: u32,
    pub alloc_counter: i32,
}

impl Default for MemoryState {
    fn default() -> Self {
        Self {
            regions: Vec::new(),
            free_list: Vec::new(),
            heap_offset: crate::vm::vm::HEAP_START,
            alloc_counter: 0,
        }
    }
}

impl MemoryState {
    /// 从 free_list 或 heap 顶部分配 `aligned_size` 字节。
    /// 成功返回地址，失败返回 None。
    pub fn allocate_raw(&mut self, aligned_size: u32, mem_limit: u32) -> Option<u32> {
        if aligned_size == 0 {
            return Some(0);
        }
        let mut addr = 0u32;
        let mut found_idx = None;
        for (i, block) in self.free_list.iter().enumerate() {
            if (block.size as u32) >= aligned_size {
                addr = block.addr;
                found_idx = Some(i);
                break;
            }
        }
        if let Some(idx) = found_idx {
            let block = &mut self.free_list[idx];
            if (block.size as u32) > aligned_size {
                block.addr += aligned_size;
                block.size -= aligned_size as i32;
            } else {
                self.free_list.remove(idx);
            }
        } else {
            addr = self.heap_offset;
            let new_offset = addr as u64 + aligned_size as u64;
            if new_offset > mem_limit as u64 || new_offset > u32::MAX as u64 {
                return None;
            }
            self.heap_offset = new_offset as u32;
        }
        Some(addr)
    }

    /// 合并 free_list 中地址相邻的空闲块。
    pub fn merge_free_list(&mut self) {
        self.free_list.sort_by_key(|b| b.addr);
        let mut merged: Vec<FreeBlock> = Vec::new();
        for block in self.free_list.drain(..) {
            if let Some(last) = merged.last_mut() {
                if (last.addr as u64) + (last.size as u64) == (block.addr as u64) {
                    last.size += block.size;
                } else {
                    merged.push(block);
                }
            } else {
                merged.push(block);
            }
        }
        self.free_list = merged;
    }
}

#[frb]
#[derive(Debug, Clone)]
pub struct CompileResult {
    pub success: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub algorithm_matches: Vec<AlgorithmMatch>,
}

#[frb]
#[derive(Debug, Clone)]
pub struct RunResult {
    pub success: bool,
    pub output: String,
    pub waiting_input: bool,
    pub error: Option<String>,
}

#[frb]
#[derive(Debug, Clone)]
pub struct StepResult {
    pub status: StepStatus,
    pub current_line: i32,
    pub output: String,
    pub waiting_input: bool,
}

#[frb]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Paused,
    WaitingInput,
    Finished,
    Trap,
}

pub struct Session {
    pub compile: CompileState,
    pub runtime: RuntimeState,
    pub memory: MemoryState,
    pub vm: Option<CideVM>,
    pub vfs: VirtualFileSystem,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            compile: CompileState::default(),
            runtime: RuntimeState::default(),
            memory: MemoryState::default(),
            vm: Some(CideVM::default()),
            vfs: VirtualFileSystem::new(),
        }
    }
}
