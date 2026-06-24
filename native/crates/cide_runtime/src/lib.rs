//! Cide 运行时共享数据结构。
//!
//! 包含 VM 与会话（Session）共享的状态类型：运行时状态、内存状态、符号表、
//! 函数元数据等。该 crate 依赖 `cide_ast` 与 `cide_shared`，供 `cide_vm` 与
//! `cide_native` 共同使用，避免 `vm/` 与 `session.rs` 之间的循环依赖。
//!
//! 注意：本 crate 中不带 `Data` 后缀的类型用于 VM 内部；带 `Data` 后缀的类型
//! 是不含 `#[frb]` 的基础数据结构，`cide_native` 会在此基础上定义带 `#[frb]`
//! 的 Dart 绑定包装类型。

pub mod bytecode_libc_index;
pub mod bytecode_libc_sig;
pub mod func_meta;
pub mod host_func_id;
pub mod instruction;
pub mod memory_state;
pub mod opcode;
pub mod runtime_state;
pub mod symbol;
pub mod type_utils;
pub mod unified_types;

pub use func_meta::FuncMeta;
pub use memory_state::{
    build_heap_stats, find_region_by_addr, fragmentation_rate, FreeBlock, HeapStatsData, MemoryFragmentData,
    MemoryRegionData, MemoryState, GLOBAL_START, HEAP_START, MAX_STACK_DEPTH, MEM_SIZE, NULL_TRAP_SIZE,
    SNAPSHOT_INTERVAL, STACK_START,
};
pub use runtime_state::{
    ExecutionHeatmap, InputMode, RuntimeState, TraceEntryData, VariableSnapshotData, VisEventData,
};
pub use symbol::Symbol;
pub use type_utils::{base_kind, immediate_base_kind};
pub use unified_types::{AccessedVarData, ArraySnapshotData, PointerSnapshotData, PointerStatusData};

/// 编译单元：一次编译的单个源文件。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompileUnit {
    pub filename: String,
    pub source: String,
}

/// 代码文件：与 `CompileUnit` 语义相近，用于前端传递。
#[derive(Debug, Clone)]
pub struct CodeFile {
    pub filename: String,
    pub source: String,
}
