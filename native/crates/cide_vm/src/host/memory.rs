use super::*;
use crate::VmContext;

pub fn host_malloc(vm: &mut CideVM, session: &mut VmContext<'_>) {
    let size = vm.pop() as i32;
    if size == 0 {
        session.runtime.output_lines.push("[warning] malloc(0) 返回 NULL。在 C 标准中，malloc(0) 的行为是实现定义的，可能返回 NULL 也可能返回一个不可解引用的非空指针。".to_string());
        vm.push(0);
        return;
    }
    if size < 0 {
        vm.push(0);
        return;
    }
    let aligned_size = ((size as u32) + 3) & !3;
    let addr = match session.memory.allocate_raw(aligned_size, vm.get_memory_size()) {
        Some(a) => a,
        None => {
            vm.push(0);
            return;
        }
    };
    // 清理被新分配重用的 freed_logs
    let new_end = addr.saturating_add(aligned_size);
    vm.freed_logs.retain(|log| {
        let log_end = log.addr.saturating_add(log.size);
        log_end <= addr || log.addr >= new_end
    });
    // reuse or add region
    let mut reused = false;
    for r in &mut session.memory.regions {
        if r.addr == addr && r.is_freed {
            r.is_freed = false;
            r.size = size;
            reused = true;
            break;
        }
    }
    if !reused {
        session.memory.alloc_counter += 1;
        session.memory.regions.push(MemoryRegionData {
            addr,
            size,
            name: format!("heap_{}", session.memory.alloc_counter),
            ty: "int".to_string(),
            is_heap: true,
            is_freed: false,
            alloc_line: vm.get_current_line(),
            alloc_by: "malloc".to_string(),
        });
    } else {
        // 复用已释放的 region 时更新分配信息
        for r in &mut session.memory.regions {
            if r.addr == addr && !r.is_freed {
                r.alloc_line = vm.get_current_line();
                r.alloc_by = "malloc".to_string();
                break;
            }
        }
    }
    vm.push(addr as u64);
}

pub fn host_free(vm: &mut CideVM, session: &mut VmContext<'_>) {
    let addr = vm.pop() as u32;
    if addr == 0 {
        return;
    }
    // Double-Free 检测
    if let Some(log) = vm.freed_logs.iter().find(|log| log.addr == addr) {
        let msg = format!("🔁 Double-Free (E3061)：你正在 free 一块已经在第 {} 行被释放过的内存（由第 {} 行的 malloc/realloc 分配）。\n\n💡 原因：同一块内存被释放了两次，这通常是因为 free(p) 后没有将 p 置为 NULL，或者两个指针指向同一块内存且都被释放了。\n✅ 解决方法：每次 free(p) 后立刻写 p = NULL;。对 NULL 指针重复 free 是安全的。", log.freed_line, log.alloc_line);
        vm.trap(&msg, &SourceLoc::default());
        return;
    }
    for r in &mut session.memory.regions {
        if r.addr == addr && !r.is_freed {
            r.is_freed = true;
            let aligned_size = ((r.size as u32) + 3) & !3;
            vm.freed_logs.push(FreedRegionInfo {
                addr: r.addr,
                size: aligned_size,
                alloc_line: r.alloc_line,
                freed_line: vm.get_current_line(),
                alloc_step: 0,
                freed_step: vm.get_executed_steps(),
            });
            session.memory.free_list.push(FreeBlock {
                addr: r.addr,
                size: aligned_size as i32,
            });
            session.memory.merge_free_list();
            break;
        }
    }
}

pub fn host_realloc(vm: &mut CideVM, session: &mut VmContext<'_>) {
    let ptr = vm.pop() as u32;
    let new_size = vm.pop() as i32;

    if new_size <= 0 {
        if ptr != 0 {
            // Equivalent to free
            for r in &mut session.memory.regions {
                if r.addr == ptr && !r.is_freed {
                    r.is_freed = true;
                    let aligned_size = ((r.size as u32) + 3) & !3;
                    vm.freed_logs.push(FreedRegionInfo {
                        addr: r.addr,
                        size: aligned_size,
                        alloc_line: r.alloc_line,
                        freed_line: vm.get_current_line(),
                        alloc_step: 0,
                        freed_step: vm.get_executed_steps(),
                    });
                    session.memory.free_list.push(FreeBlock {
                        addr: r.addr,
                        size: aligned_size as i32,
                    });
                    session.memory.merge_free_list();
                    break;
                }
            }
        }
        vm.push(0);
        return;
    }

    if ptr == 0 {
        // Equivalent to malloc
        vm.push(new_size as u64);
        host_malloc(vm, session);
        return;
    }

    // Find existing region
    let mut old_region = None;
    for r in &session.memory.regions {
        if r.addr == ptr && !r.is_freed {
            old_region = Some((r.addr, r.size));
            break;
        }
    }

    let Some((old_addr, old_size)) = old_region else {
        vm.push(0);
        return;
    };
    let aligned_new_size = ((new_size as u32) + 3) & !3;
    let aligned_old_size = ((old_size as u32) + 3) & !3;

    // In-place shrink: old block is at the end of heap
    if aligned_new_size <= aligned_old_size && old_addr + aligned_old_size == session.memory.heap_offset {
        for r in &mut session.memory.regions {
            if r.addr == old_addr && !r.is_freed {
                r.size = new_size;
                break;
            }
        }
        let shrink_by = aligned_old_size - aligned_new_size;
        if shrink_by > 0 {
            session.memory.heap_offset -= shrink_by;
            session.memory.free_list.push(FreeBlock {
                addr: session.memory.heap_offset,
                size: shrink_by as i32,
            });
            session.memory.merge_free_list();
        }
        vm.push(old_addr as u64);
        return;
    }

    // Allocate new memory
    let mut new_addr = 0u32;
    let mut found_idx = None;
    for (i, block) in session.memory.free_list.iter().enumerate() {
        if (block.size as u32) >= aligned_new_size {
            new_addr = block.addr;
            found_idx = Some(i);
            break;
        }
    }

    if let Some(idx) = found_idx {
        let block = &mut session.memory.free_list[idx];
        if (block.size as u32) > aligned_new_size {
            block.addr += aligned_new_size;
            block.size -= aligned_new_size as i32;
        } else {
            session.memory.free_list.remove(idx);
        }
    } else {
        let offset = session.memory.heap_offset;
        let new_offset = (offset as u64) + (aligned_new_size as u64);
        if new_offset > vm.get_memory_size() as u64 {
            vm.push(0);
            return;
        }
        if new_offset > u32::MAX as u64 {
            vm.push(0);
            return;
        }
        new_addr = offset;
        session.memory.heap_offset = new_offset as u32;
    }

    // 清理被新分配重用的 freed_logs（必须在写入新内存之前执行，
    // 否则 store_i8 会触发 Use-After-Free 误报）
    let new_end = new_addr.saturating_add(aligned_new_size);
    vm.freed_logs.retain(|log| {
        let log_end = log.addr.saturating_add(log.size);
        log_end <= new_addr || log.addr >= new_end
    });

    // Copy old data
    let copy_size = (old_size as u32).min(aligned_new_size);
    let copy_buf = {
        let mem = vm.get_memory_slice();
        mem[old_addr as usize..(old_addr + copy_size) as usize].to_vec()
    };
    for i in 0..copy_size {
        vm.store_i8(new_addr + i, copy_buf[i as usize] as i32, &SourceLoc::default());
    }

    // Zero remaining bytes
    for i in copy_size..aligned_new_size {
        vm.store_i8(new_addr + i, 0, &SourceLoc::default());
    }

    // Free old region
    for r in &mut session.memory.regions {
        if r.addr == old_addr && !r.is_freed {
            r.is_freed = true;
            vm.freed_logs.push(FreedRegionInfo {
                addr: r.addr,
                size: aligned_old_size,
                alloc_line: r.alloc_line,
                freed_line: vm.get_current_line(),
                alloc_step: 0,
                freed_step: vm.get_executed_steps(),
            });
            session.memory.free_list.push(FreeBlock {
                addr: r.addr,
                size: aligned_old_size as i32,
            });
            session.memory.merge_free_list();
            break;
        }
    }

    // 若 realloc 恰好复用了旧地址（如 heap_offset 回退后），需清理刚添加的 freed_log
    let new_end = new_addr.saturating_add(aligned_new_size);
    vm.freed_logs.retain(|log| {
        let log_end = log.addr.saturating_add(log.size);
        log_end <= new_addr || log.addr >= new_end
    });

    // Track new region
    session.memory.regions.push(MemoryRegionData {
        addr: new_addr,
        size: new_size,
        name: String::new(),
        ty: String::new(),
        is_heap: true,
        is_freed: false,
        alloc_line: vm.get_current_line(),
        alloc_by: "realloc".to_string(),
    });

    vm.push(new_addr as u64);
}

pub fn host_calloc(vm: &mut CideVM, session: &mut VmContext<'_>) {
    let size = vm.pop() as i32;
    let nmemb = vm.pop() as i32;
    if size <= 0 || nmemb <= 0 {
        vm.push(0);
        return;
    }
    let total = (nmemb as u32).saturating_mul(size as u32);
    let aligned_size = (total + 3) & !3;
    let addr = match session.memory.allocate_raw(aligned_size, vm.get_memory_size()) {
        Some(a) => a,
        None => {
            vm.push(0);
            return;
        }
    };
    // zero-initialize
    for i in 0..aligned_size {
        vm.store_i8(addr + i, 0, &SourceLoc::default());
    }
    // clean freed_logs
    let new_end = addr.saturating_add(aligned_size);
    vm.freed_logs.retain(|log| {
        let log_end = log.addr.saturating_add(log.size);
        log_end <= addr || log.addr >= new_end
    });
    session.memory.alloc_counter += 1;
    session.memory.regions.push(MemoryRegionData {
        addr,
        size: total as i32,
        name: format!("heap_{}", session.memory.alloc_counter),
        ty: "int".to_string(),
        is_heap: true,
        is_freed: false,
        alloc_line: vm.get_current_line(),
        alloc_by: "calloc".to_string(),
    });
    vm.push(addr as u64);
}
