use std::collections::HashMap;

/// 总内存空间（1MB）。
pub const MEM_SIZE: u32 = 1024 * 1024;
/// NULL 陷阱区大小。
pub const NULL_TRAP_SIZE: u32 = 0x1000;
/// 全局区起始地址。
pub const GLOBAL_START: u32 = 0x1000;
/// 堆区起始地址。
pub const HEAP_START: u32 = 0x5000;
/// 栈区起始地址（从高地址向低地址增长）。
pub const STACK_START: u32 = MEM_SIZE;
/// 快照间隔步数。
pub const SNAPSHOT_INTERVAL: i32 = 100_000;
/// 最大调用栈深度。
pub const MAX_STACK_DEPTH: usize = 10_000;

/// 内存区域基础数据：VM 内部使用；`cide_native` 会定义带 `#[frb]` 的同名包装。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryRegionData {
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

/// 内存碎片基础数据。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryFragmentData {
    pub addr: u32,
    pub size: i32,
}

/// 堆统计信息基础数据。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HeapStatsData {
    /// 总堆空间（heap_offset - HEAP_START），字节
    pub total_heap: i32,
    /// 已分配且未释放的堆内存，字节
    pub allocated: i32,
    /// 外部碎片（free_list 中所有块之和），字节
    pub fragmented: i32,
    /// 碎片率（0~100）
    pub fragmentation_rate: i32,
}

/// 空闲内存块。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FreeBlock {
    pub addr: u32,
    pub size: i32,
}

/// 内存状态：跟踪堆分配、空闲块与堆顶偏移。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryState {
    pub regions: Vec<MemoryRegionData>,
    pub free_list: Vec<FreeBlock>,
    pub heap_offset: u32,
    pub alloc_counter: i32,
}

impl Default for MemoryState {
    fn default() -> Self {
        Self {
            regions: Vec::new(),
            free_list: Vec::new(),
            heap_offset: HEAP_START,
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
                // 若该 free block 恰位于 heap 顶部，更新 heap_offset 避免后续分配冲突
                if addr == self.heap_offset {
                    self.heap_offset = block.addr;
                }
            } else {
                self.free_list.remove(idx);
                // 若整块被分配且位于 heap 顶部，推进 heap_offset
                if addr == self.heap_offset {
                    self.heap_offset = addr + aligned_size;
                }
            }
        } else {
            addr = self.heap_offset;
            let new_offset = addr as u64 + aligned_size as u64;
            if new_offset > mem_limit as u64 || new_offset > u32::MAX as u64 {
                return None;
            }
            self.heap_offset = new_offset as u32;
        }
        // 清理 free_list 中与刚分配区域重叠的 stale 块
        let alloc_end = addr + aligned_size;
        let mut i = 0;
        while i < self.free_list.len() {
            let block = &self.free_list[i];
            let block_end = block.addr + block.size as u32;
            if block.addr >= alloc_end || block_end <= addr {
                i += 1;
                continue;
            }
            if block.addr >= addr && block_end <= alloc_end {
                // 完全被覆盖
                self.free_list.remove(i);
            } else if block.addr < addr && block_end > alloc_end {
                // 分配在块内部：拆分为前后两部分
                let tail_size = block_end - alloc_end;
                let head_size = addr - block.addr;
                self.free_list[i].size = head_size as i32;
                if tail_size > 0 {
                    self.free_list.push(FreeBlock {
                        addr: alloc_end,
                        size: tail_size as i32,
                    });
                }
                i += 1;
            } else if block.addr < addr {
                // 覆盖块的后部
                self.free_list[i].size = (addr - block.addr) as i32;
                i += 1;
            } else {
                // 覆盖块的前部
                self.free_list[i].addr = alloc_end;
                self.free_list[i].size = (block_end - alloc_end) as i32;
                i += 1;
            }
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

/// 从 regions 中按地址查找指定内存块（供测试与诊断使用）。
pub fn find_region_by_addr(regions: &[MemoryRegionData], addr: u32) -> Option<&MemoryRegionData> {
    regions.iter().find(|r| r.addr == addr)
}

/// 计算当前已分配但未释放的堆内存总量。
pub fn total_allocated(regions: &[MemoryRegionData]) -> i32 {
    regions.iter().filter(|r| !r.is_freed).map(|r| r.size).sum()
}

/// 估算外部碎片大小（简化版：free_list 总和）。
pub fn total_fragmented(free_list: &[FreeBlock]) -> i32 {
    free_list.iter().map(|b| b.size).sum()
}

/// 计算碎片率（0~100）。
pub fn fragmentation_rate(free_list: &[FreeBlock], heap_offset: u32) -> i32 {
    let heap_total = heap_offset.saturating_sub(HEAP_START);
    if heap_total == 0 {
        return 0;
    }
    let fragmented = total_fragmented(free_list) as u64;
    let rate = (fragmented * 100) / (heap_total as u64);
    rate.min(100) as i32
}

/// 构建教学用的 `HeapStatsData` 快照。
pub fn build_heap_stats(regions: &[MemoryRegionData], free_list: &[FreeBlock], heap_offset: u32) -> HeapStatsData {
    let total_heap = heap_offset.saturating_sub(HEAP_START) as i32;
    let allocated = total_allocated(regions);
    let fragmented = total_fragmented(free_list);
    let rate = fragmentation_rate(free_list, heap_offset);
    HeapStatsData {
        total_heap,
        allocated,
        fragmented,
        fragmentation_rate: rate,
    }
}

/// 保留 `MemoryRegionData` 作为键的兼容性辅助：按地址分组。
pub fn group_regions_by_name(regions: &[MemoryRegionData]) -> HashMap<String, Vec<&MemoryRegionData>> {
    let mut map: HashMap<String, Vec<&MemoryRegionData>> = HashMap::new();
    for r in regions {
        map.entry(r.name.clone()).or_default().push(r);
    }
    map
}
