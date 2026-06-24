use crate::engine::completion::CompletionSnapshot;
use crate::vm::core::CideVM;
use crate::vm::vfs::VirtualFileSystem;
use cide_runtime::instruction::Instruction;
use flutter_rust_bridge::frb;
use std::collections::HashMap;
use std::ffi::CString;

pub use cide_runtime::{
    CodeFile, CompileUnit, FreeBlock, FuncMeta, InputMode, MemoryState, RuntimeState, Symbol, GLOBAL_START, HEAP_START,
    MAX_STACK_DEPTH, MEM_SIZE, NULL_TRAP_SIZE, SNAPSHOT_INTERVAL, STACK_START,
};

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
    /// 最近一次 `cide_get_compile_errors` 返回的 C 字符串缓存，避免返回 `String` 内部指针导致悬垂。
    pub last_errors_cstring: Option<CString>,
    pub compile_units: Vec<CompileUnit>,
    pub compiled: bool,
    pub bytecode: Vec<Instruction>,
    pub globals_init: Vec<(u32, i32)>,
    pub globals_init_64: Vec<(u32, u64)>,
    pub f64_constants: Vec<f64>,
    pub i64_constants: Vec<i64>,
    pub diagnostics: Vec<Diagnostic>,
    pub source_map: Vec<(u32, cide_shared::source_loc::SourceLoc)>,
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

impl From<cide_runtime::TraceEntryData> for TraceEntry {
    fn from(value: cide_runtime::TraceEntryData) -> Self {
        Self {
            line: value.line,
            operation: value.operation,
        }
    }
}

impl From<TraceEntry> for cide_runtime::TraceEntryData {
    fn from(value: TraceEntry) -> Self {
        Self {
            line: value.line,
            operation: value.operation,
        }
    }
}

#[frb]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VariableSnapshot {
    pub name: String,
    pub addr: u32,
    pub is_local: bool,
    pub ty: cide_ast::Type,
    pub value: i64,
}

impl From<cide_runtime::VariableSnapshotData> for VariableSnapshot {
    fn from(value: cide_runtime::VariableSnapshotData) -> Self {
        Self {
            name: value.name,
            addr: value.addr,
            is_local: value.is_local,
            ty: value.ty,
            value: value.value,
        }
    }
}

impl From<VariableSnapshot> for cide_runtime::VariableSnapshotData {
    fn from(value: VariableSnapshot) -> Self {
        Self {
            name: value.name,
            addr: value.addr,
            is_local: value.is_local,
            ty: value.ty,
            value: value.value,
        }
    }
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

impl From<cide_runtime::VisEventData> for VisEvent {
    fn from(value: cide_runtime::VisEventData) -> Self {
        Self {
            ty: value.ty,
            line: value.line,
            extra0: value.extra0,
            extra1: value.extra1,
            extra2: value.extra2,
            context: value.context,
        }
    }
}

impl From<VisEvent> for cide_runtime::VisEventData {
    fn from(value: VisEvent) -> Self {
        Self {
            ty: value.ty,
            line: value.line,
            extra0: value.extra0,
            extra1: value.extra1,
            extra2: value.extra2,
            context: value.context,
        }
    }
}

/// 执行路径热力图：记录每行源代码被执行的次数。
pub use cide_runtime::ExecutionHeatmap;

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

impl From<cide_runtime::MemoryRegionData> for MemoryRegion {
    fn from(value: cide_runtime::MemoryRegionData) -> Self {
        Self {
            addr: value.addr,
            size: value.size,
            name: value.name,
            ty: value.ty,
            is_heap: value.is_heap,
            is_freed: value.is_freed,
            alloc_line: value.alloc_line,
            alloc_by: value.alloc_by,
        }
    }
}

impl From<MemoryRegion> for cide_runtime::MemoryRegionData {
    fn from(value: MemoryRegion) -> Self {
        Self {
            addr: value.addr,
            size: value.size,
            name: value.name,
            ty: value.ty,
            is_heap: value.is_heap,
            is_freed: value.is_freed,
            alloc_line: value.alloc_line,
            alloc_by: value.alloc_by,
        }
    }
}

#[frb]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryFragment {
    pub addr: u32,
    pub size: i32,
}

impl From<cide_runtime::MemoryFragmentData> for MemoryFragment {
    fn from(value: cide_runtime::MemoryFragmentData) -> Self {
        Self {
            addr: value.addr,
            size: value.size,
        }
    }
}

impl From<MemoryFragment> for cide_runtime::MemoryFragmentData {
    fn from(value: MemoryFragment) -> Self {
        Self {
            addr: value.addr,
            size: value.size,
        }
    }
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

impl From<cide_runtime::HeapStatsData> for HeapStats {
    fn from(value: cide_runtime::HeapStatsData) -> Self {
        Self {
            total_heap: value.total_heap,
            allocated: value.allocated,
            fragmented: value.fragmented,
            fragmentation_rate: value.fragmentation_rate,
        }
    }
}

impl From<HeapStats> for cide_runtime::HeapStatsData {
    fn from(value: HeapStats) -> Self {
        Self {
            total_heap: value.total_heap,
            allocated: value.allocated,
            fragmented: value.fragmented,
            fragmentation_rate: value.fragmentation_rate,
        }
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

impl Session {
    /// 构造 VM 执行上下文，将 VM 所需的运行时/内存/VFS 可变引用聚合起来。
    pub fn as_vm_context(&mut self) -> crate::vm::context::VmContext<'_> {
        crate::vm::context::VmContext {
            runtime: &mut self.runtime,
            memory: &mut self.memory,
            vfs: &mut self.vfs,
        }
    }
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

impl cide_algorithm_steps::AlgorithmContext for Session {
    fn source_line(&self, line: i32) -> Option<String> {
        let unit = self.compile.compile_units.first()?;
        let line = unit.source.lines().nth((line - 1) as usize)?;
        Some(line.trim().to_string())
    }

    fn find_algorithm(&self, func_name: &str) -> Option<cide_algorithm_steps::AlgorithmMatch> {
        self.compile
            .algorithm_matches
            .iter()
            .find(|m| m.func_name == func_name)
            .map(|m| cide_algorithm_steps::AlgorithmMatch {
                name: m.name.clone(),
                display_name: m.display_name.clone(),
                func_name: m.func_name.clone(),
                confidence: m.confidence,
                suggestion: m.suggestion.clone(),
                line: m.line,
            })
    }
}
