use std::collections::{HashMap, HashSet};

use crate::session::{FreeBlock, MemoryRegion, RuntimeState, TraceEntry, VisEvent};
use crate::vm::vm::CallFrame;

/// VM 内存快照：全量或页级增量（页大小 4KB）。
#[derive(Clone)]
pub enum MemoryImage {
    /// 完整 1MB 内存拷贝。
    Full(Vec<u8>),
    /// 相对于某全量检查点的脏页集合。
    /// 页索引 0..255，每页 4096 字节。
    Delta {
        base_step: i32,
        pages: Vec<(u16, Vec<u8>)>,
    },
}

impl MemoryImage {
    /// 获取内存总字节数（用于调试/统计）。
    pub fn byte_size(&self) -> usize {
        match self {
            MemoryImage::Full(v) => v.len(),
            MemoryImage::Delta { pages, .. } => pages.iter().map(|(_, p)| p.len()).sum(),
        }
    }

    /// 重建完整 1MB 内存，写入到提供的 buffer 中。
    /// 调用者应确保 `dst` 长度至少为 1MB，且已填充基础内存内容。
    pub fn apply_to(&self, dst: &mut [u8]) {
        match self {
            MemoryImage::Full(v) => {
                let len = v.len().min(dst.len());
                dst[..len].copy_from_slice(&v[..len]);
            }
            MemoryImage::Delta { pages, .. } => {
                for (page_idx, page_data) in pages {
                    let offset = (*page_idx as usize) * PAGE_SIZE;
                    if offset + page_data.len() <= dst.len() {
                        dst[offset..offset + page_data.len()].copy_from_slice(page_data);
                    }
                }
            }
        }
    }
}

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_COUNT: usize = 256; // 1MB / 4KB

/// VM 全量/增量快照。
///
/// 注意：快照**不保存**编译期常量（bytecode、函数表、符号表等），
/// 因为这些可以从 `Session.compile` 重建。
/// 使用快照前，必须先调用 `setup_vm()` 确保 VM 已加载程序。
#[derive(Clone)]
pub struct VMSnapshot {
    // VM 核心运行时状态
    pub memory: MemoryImage,
    pub stack: Vec<u64>,
    pub call_stack: Vec<CallFrame>,
    pub ip: usize,
    pub mem_stack_top: u32,
    pub step_count: i32,
    pub current_line: i32,
    pub finished: bool,
    pub exit_code: i32,
    pub error: String,
    pub paused: bool,
    pub cancelled: bool,
    pub step_event_hit: bool,
    pub last_snapshot_step: i32,
    pub snapshot_vars: HashMap<String, u64>,
    pub qsort_depth: i32,
    pub vis_event_queue: Vec<VisEvent>,
    pub breakpoints: HashSet<i32>,
    pub global_count: usize,
    pub freed_logs: Vec<crate::vm::vm::FreedRegionInfo>,

    // Session 运行时状态（随 VM 一起恢复）
    pub runtime: RuntimeSnapshot,
    pub memory_state: MemorySnapshot,
}

#[derive(Clone)]
pub struct RuntimeSnapshot {
    pub output_lines: Vec<String>,
    pub trace: Vec<TraceEntry>,
    pub current_line: i32,
    pub input_index: usize,
    pub input_char_offset: usize,
    pub waiting_input: bool,
    pub rand_seed: u32,
    pub vis_event_cache: Vec<VisEvent>,
}

#[derive(Clone)]
pub struct MemorySnapshot {
    pub regions: Vec<MemoryRegion>,
    pub free_list: Vec<FreeBlock>,
    pub heap_offset: u32,
    pub alloc_counter: i32,
}

impl From<&RuntimeState> for RuntimeSnapshot {
    fn from(rt: &RuntimeState) -> Self {
        Self {
            output_lines: rt.output_lines.clone(),
            trace: rt.trace.clone(),
            current_line: rt.current_line,
            input_index: rt.input_index,
            input_char_offset: rt.input_char_offset,
            waiting_input: rt.waiting_input,
            rand_seed: rt.rand_seed,
            vis_event_cache: rt.vis_event_cache.clone(),
        }
    }
}

impl From<&crate::session::MemoryState> for MemorySnapshot {
    fn from(mem: &crate::session::MemoryState) -> Self {
        Self {
            regions: mem.regions.clone(),
            free_list: mem.free_list.clone(),
            heap_offset: mem.heap_offset,
            alloc_counter: mem.alloc_counter,
        }
    }
}
