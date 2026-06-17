//! VM 内存访问：安全读写、脏页追踪、变量快照与数组快照。

use super::state::{CideVM, FreedRegionInfo, VMSymbol, MEM_SIZE, NULL_TRAP_SIZE};
use crate::compiler::ast::TypeKind;
use crate::shared::type_utils::base_kind;
use crate::vm::instruction::SourceLoc;

impl CideVM {
    /// 将 C 风格字符串安全写入 VM 内存的指定地址（含 null 终止符）。
    /// 若目标地址超出边界则静默跳过。
    ///
    /// 注意：边界检查 `a + bytes.len() < len` 已隐含为末尾的 null 终止符预留了 1 字节空间，
    /// 因此当 `addr + bytes.len() == MEM_SIZE` 时会正确拒绝写入，避免越界。
    pub fn write_cstring(&mut self, addr: u32, s: &str) {
        let a = addr as usize;
        let bytes = s.as_bytes();
        let total = bytes.len() + 1;
        // 统一边界检查：NULL 区、上界、UAF。
        if !self.check_mem_access(addr, total as u32, &SourceLoc::default(), true) {
            return;
        }
        if let Some(log) = self.check_uaf(addr, total as u32) {
            let msg = self.format_uaf_message(log, true);
            self.trap(&msg, &SourceLoc::default());
            return;
        }
        self.memory[a..a + bytes.len()].copy_from_slice(bytes);
        self.memory[a + bytes.len()] = 0;
        self.mark_dirty_page(addr, total as u32);
    }

    /// 安全写入字节切片到 VM 内存（带 NULL Trap 和边界检查）
    pub fn write_memory(&mut self, addr: u32, data: &[u8]) -> bool {
        let end = addr as usize + data.len();
        if addr < NULL_TRAP_SIZE || end > self.memory.len() {
            return false;
        }
        if let Some(log) = self.check_uaf(addr, data.len() as u32) {
            let msg = self.format_uaf_message(log, true);
            self.trap(&msg, &SourceLoc::default());
            return false;
        }
        self.memory[addr as usize..end].copy_from_slice(data);
        self.mark_dirty_page(addr, data.len() as u32);
        true
    }

    /// 安全读取字节切片从 VM 内存到外部缓冲区
    pub fn read_memory_to(&self, addr: u32, buf: &mut [u8]) -> bool {
        let end = addr as usize + buf.len();
        if addr < NULL_TRAP_SIZE || end > self.memory.len() {
            return false;
        }
        buf.copy_from_slice(&self.memory[addr as usize..end]);
        true
    }

    /// 安全地在 VM 内存内部复制（src → dst），带完整边界检查
    pub fn copy_memory(&mut self, dst: u32, src: u32, len: usize) -> bool {
        let dst_end = dst as usize + len;
        let src_end = src as usize + len;
        if dst < NULL_TRAP_SIZE || src < NULL_TRAP_SIZE || dst_end > self.memory.len() || src_end > self.memory.len() {
            return false;
        }
        let len_u32 = len as u32;
        if let Some(log) = self.check_uaf(dst, len_u32) {
            let msg = self.format_uaf_message(log, true);
            self.trap(&msg, &SourceLoc::default());
            return false;
        }
        if let Some(log) = self.check_uaf(src, len_u32) {
            let msg = self.format_uaf_message(log, false);
            self.trap(&msg, &SourceLoc::default());
            return false;
        }
        // 用临时 buffer 避免重叠问题
        let tmp = self.memory[src as usize..src_end].to_vec();
        self.memory[dst as usize..dst_end].copy_from_slice(&tmp);
        self.mark_dirty_page(dst, len as u32);
        true
    }

    pub fn load_i32(&mut self, addr: u32, loc: &SourceLoc) -> i32 {
        if !self.check_mem_access(addr, 4, loc, false) {
            return 0;
        }
        if let Some(log) = self.check_uaf(addr, 4) {
            let msg = self.format_uaf_message(log, false);
            self.trap(&msg, loc);
            return 0;
        }
        i32::from_le_bytes([
            self.memory[addr as usize],
            self.memory[addr as usize + 1],
            self.memory[addr as usize + 2],
            self.memory[addr as usize + 3],
        ])
    }

    pub fn store_i32(&mut self, addr: u32, val: i32, loc: &SourceLoc) {
        if !self.check_mem_access(addr, 4, loc, true) {
            return;
        }
        if let Some(log) = self.check_uaf(addr, 4) {
            let msg = self.format_uaf_message(log, true);
            self.trap(&msg, loc);
            return;
        }
        let bytes = val.to_le_bytes();
        self.memory[addr as usize..addr as usize + 4].copy_from_slice(&bytes);
        self.mark_dirty_page(addr, 4);
    }

    pub fn load_i64(&mut self, addr: u32, loc: &SourceLoc) -> u64 {
        if !self.check_mem_access(addr, 8, loc, false) {
            return 0;
        }
        if let Some(log) = self.check_uaf(addr, 8) {
            let msg = self.format_uaf_message(log, false);
            self.trap(&msg, loc);
            return 0;
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.memory[addr as usize..addr as usize + 8]);
        u64::from_le_bytes(bytes)
    }

    pub fn store_i64(&mut self, addr: u32, val: u64, loc: &SourceLoc) {
        if !self.check_mem_access(addr, 8, loc, true) {
            return;
        }
        if let Some(log) = self.check_uaf(addr, 8) {
            let msg = self.format_uaf_message(log, true);
            self.trap(&msg, loc);
            return;
        }
        let bytes = val.to_le_bytes();
        self.memory[addr as usize..addr as usize + 8].copy_from_slice(&bytes);
        self.mark_dirty_page(addr, 8);
    }

    pub fn load_i8(&mut self, addr: u32, loc: &SourceLoc) -> i32 {
        if !self.check_mem_access(addr, 1, loc, false) {
            return 0;
        }
        if let Some(log) = self.check_uaf(addr, 1) {
            let msg = self.format_uaf_message(log, false);
            self.trap(&msg, loc);
            return 0;
        }
        self.memory[addr as usize] as i8 as i32
    }

    pub fn store_i8(&mut self, addr: u32, val: i32, loc: &SourceLoc) {
        if !self.check_mem_access(addr, 1, loc, true) {
            return;
        }
        if let Some(log) = self.check_uaf(addr, 1) {
            let msg = self.format_uaf_message(log, true);
            self.trap(&msg, loc);
            return;
        }
        self.memory[addr as usize] = val as u8;
        self.mark_dirty_page(addr, 1);
    }

    /// 标记脏页（ addr 为起始地址，len 为字节数 ）。
    pub(crate) fn mark_dirty_page(&mut self, addr: u32, len: u32) {
        if len == 0 {
            return;
        }
        let start_page = (addr as usize) / crate::vm::snapshot::PAGE_SIZE;
        let end_page = ((addr as usize) + (len as usize) - 1) / crate::vm::snapshot::PAGE_SIZE;
        for p in start_page..=end_page {
            if p < crate::vm::snapshot::PAGE_COUNT {
                let word = p / 64;
                let bit = p % 64;
                self.dirty_pages[word] |= 1u64 << bit;
            }
        }
    }

    /// 清空脏页记录。
    pub fn clear_dirty_pages(&mut self) {
        self.dirty_pages = [0; 4];
    }

    pub(crate) fn check_uaf(&self, addr: u32, size: u32) -> Option<&FreedRegionInfo> {
        self.freed_logs
            .iter()
            .find(|log| addr < log.addr + log.size && addr + size > log.addr)
    }

    pub(crate) fn format_uaf_message(&self, log: &FreedRegionInfo, is_write: bool) -> String {
        let action = if is_write { "写入" } else { "读取" };
        format!(
            "💥 Use-After-Free (E3060)：你正在向一块已经在第 {} 行（第 {} 步）被 free 的内存{}（由第 {} 行的 malloc/realloc 分配）。\n\n⏱️ 时间轴：分配 → 释放（第 {} 步）→ 继续访问（第 {} 步）。\n\n💡 原因：指针在 free 后没有置为 NULL，或者存在别名指针（另一个指针也指向同一块内存）。\n✅ 解决方法：free(p) 后立即写 p = NULL;，并确保不再通过其他指针访问这块内存。",
            log.freed_line, log.freed_step, action, log.alloc_line, log.freed_step, self.step_count
        )
    }

    pub(crate) fn check_mem_access(&mut self, addr: u32, size: u32, loc: &SourceLoc, is_write: bool) -> bool {
        if addr < NULL_TRAP_SIZE {
            let msg = if is_write {
                format!("向 NULL 指针区域写入（地址 0x{:04X}）。请确认指针已被正确初始化。", addr)
            } else {
                format!(
                    "访问了 NULL 指针区域（地址 0x{:04X}）。NULL 指针不能解引用。请确认指针已被正确初始化。",
                    addr
                )
            };
            self.trap(&msg, loc);
            return false;
        }
        if addr as u64 + size as u64 > MEM_SIZE as u64 {
            self.trap(&self.format_bounds_error(addr), loc);
            return false;
        }
        true
    }

    pub(crate) fn read_variable(&self, sym: &VMSymbol) -> i64 {
        let vaddr = if sym.is_local {
            if let Some(frame) = self.call_stack.last() {
                frame.locals_base + sym.addr
            } else {
                return 0;
            }
        } else {
            super::state::GLOBAL_START + sym.addr
        };
        if vaddr + 4 > MEM_SIZE || vaddr < NULL_TRAP_SIZE {
            return 0;
        }
        if matches!(sym.ty.kind(), TypeKind::Double | TypeKind::LongLong) {
            if vaddr + 8 > MEM_SIZE {
                return 0;
            }
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&self.memory[vaddr as usize..vaddr as usize + 8]);
            u64::from_le_bytes(bytes) as i64
        } else {
            i32::from_le_bytes([
                self.memory[vaddr as usize],
                self.memory[vaddr as usize + 1],
                self.memory[vaddr as usize + 2],
                self.memory[vaddr as usize + 3],
            ]) as i64
        }
    }

    pub fn get_variable_snapshot(&self) -> Vec<crate::session::VariableSnapshot> {
        self.symbols
            .iter()
            .filter_map(|sym| {
                let vaddr = if sym.is_local {
                    if let Some(frame) = self.call_stack.last() {
                        frame.locals_base + sym.addr
                    } else {
                        return None;
                    }
                } else {
                    super::state::GLOBAL_START + sym.addr
                };
                if vaddr + 4 > MEM_SIZE || vaddr < NULL_TRAP_SIZE {
                    return None;
                }
                let val = if matches!(sym.ty.kind(), TypeKind::Double) {
                    if vaddr + 8 > MEM_SIZE {
                        return None;
                    }
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&self.memory[vaddr as usize..vaddr as usize + 8]);
                    u64::from_le_bytes(bytes) as i64
                } else {
                    i32::from_le_bytes([
                        self.memory[vaddr as usize],
                        self.memory[vaddr as usize + 1],
                        self.memory[vaddr as usize + 2],
                        self.memory[vaddr as usize + 3],
                    ]) as i64
                };
                Some(crate::session::VariableSnapshot {
                    name: sym.name.clone(),
                    addr: vaddr,
                    is_local: sym.is_local,
                    ty: sym.ty.clone(),
                    value: val,
                })
            })
            .collect()
    }

    /// 获取所有数组变量的元素快照（用于算法可视化条形图）。
    pub fn get_array_snapshots(&self) -> Vec<crate::unified::types::ArraySnapshot> {
        let mut result = Vec::new();
        for sym in &self.symbols {
            if sym.ty.kind() != TypeKind::Array {
                continue;
            }
            let vaddr = if sym.is_local {
                if let Some(frame) = self.call_stack.last() {
                    frame.locals_base + sym.addr
                } else {
                    continue;
                }
            } else {
                super::state::GLOBAL_START + sym.addr
            };
            let array_size = sym.ty.array_size();
            if array_size <= 0 {
                continue;
            }
            let base_kind = base_kind(&sym.ty);
            let elem_size = match base_kind {
                TypeKind::Char => 1,
                TypeKind::Int | TypeKind::Pointer | TypeKind::Float => 4,
                TypeKind::Double | TypeKind::LongLong => 8,
                _ => 4,
            };
            let mut elements = Vec::with_capacity(array_size as usize);
            for i in 0..array_size {
                let addr = vaddr + (i as u32) * (elem_size as u32);
                let elem_size_u32 = elem_size as u32;
                // NULL 区检查；上界在后续索引前已经判断。
                if addr < NULL_TRAP_SIZE || addr + elem_size_u32 > MEM_SIZE {
                    break;
                }
                let val_str = match base_kind {
                    TypeKind::Char => {
                        let b = self.memory[addr as usize];
                        if (32..=126).contains(&b) {
                            format!("'{}'", b as char)
                        } else {
                            format!("{}", b as i8)
                        }
                    }
                    TypeKind::Int => {
                        let bytes = [
                            self.memory[addr as usize],
                            self.memory[addr as usize + 1],
                            self.memory[addr as usize + 2],
                            self.memory[addr as usize + 3],
                        ];
                        i32::from_le_bytes(bytes).to_string()
                    }
                    TypeKind::Float => {
                        let bytes = [
                            self.memory[addr as usize],
                            self.memory[addr as usize + 1],
                            self.memory[addr as usize + 2],
                            self.memory[addr as usize + 3],
                        ];
                        format!("{:.2}", f32::from_le_bytes(bytes))
                    }
                    TypeKind::Double => {
                        let mut bytes = [0u8; 8];
                        bytes.copy_from_slice(&self.memory[addr as usize..addr as usize + 8]);
                        format!("{:.2}", f64::from_le_bytes(bytes))
                    }
                    TypeKind::LongLong => {
                        let mut bytes = [0u8; 8];
                        bytes.copy_from_slice(&self.memory[addr as usize..addr as usize + 8]);
                        i64::from_le_bytes(bytes).to_string()
                    }
                    _ => "?".to_string(),
                };
                elements.push(val_str);
            }
            let element_ty = match base_kind {
                TypeKind::Int => "int",
                TypeKind::Char => "char",
                TypeKind::Float => "float",
                TypeKind::Double => "double",
                TypeKind::LongLong => "long long",
                _ => "unknown",
            }
            .to_string();
            result.push(crate::unified::types::ArraySnapshot {
                name: sym.name.clone(),
                element_ty,
                elements,
            });
        }
        result
    }
}
