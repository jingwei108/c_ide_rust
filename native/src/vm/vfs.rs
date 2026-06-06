//! 轻量级虚拟文件系统（VFS-lite）
//!
//! 教学友好的文件 I/O 实现：
//! - 文件元数据（fd、cursor、mode）在 Rust 端管理
//! - 文件数据存储在 VM Heap 中，前端内存 Canvas 可直接可视化
//! - 支持预设文件注入（如 test.txt），用于教学演示

use std::collections::HashMap;

use crate::session::{FreeBlock, MemoryRegion, MemoryState};
use crate::vm::vm::CideVM;



#[derive(Debug, Clone, Default)]
pub struct VirtualFileSystem {
    files: HashMap<String, VfsFileMeta>,
    descriptors: HashMap<u32, VfsDesc>,
    next_fd: u32,
}

#[derive(Debug, Clone)]
struct VfsFileMeta {
    heap_addr: u32,
    size: usize,
    capacity: usize,
}

#[derive(Debug, Clone)]
struct VfsDesc {
    file_name: String,
    mode: VfsMode,
    cursor: usize,
    eof: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VfsMode {
    Read,
    Write,
    Append,
}

impl VirtualFileSystem {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注入预设文件到 VM Heap。通常在 `setup_vm` 之后调用。
    pub fn inject_preset_file(
        &mut self,
        name: &str,
        data: &[u8],
        vm: &mut CideVM,
        memory: &mut MemoryState,
    ) -> bool {
        if self.files.contains_key(name) {
            return false;
        }
        let size = data.len();
        let aligned = align4(size);
        let addr = match malloc_raw(memory, aligned, vm.get_memory_size()) {
            Some(a) => a,
            None => return false,
        };
        // 写入数据到 VM 内存
        if !vm.write_memory(addr, data) {
            free_raw(memory, addr);
            return false;
        }
        // 更新 region 名称
        for r in &mut memory.regions {
            if r.addr == addr {
                r.name = format!("vfs:{}", name);
                break;
            }
        }
        self.files.insert(
            name.to_string(),
            VfsFileMeta {
                heap_addr: addr,
                size,
                capacity: aligned,
            },
        );
        true
    }

    /// fopen：打开或创建文件，返回 fd（0 表示失败）
    pub fn fopen(
        &mut self,
        path: &str,
        mode: &str,
        vm: &mut CideVM,
        memory: &mut MemoryState,
    ) -> u32 {
        let mode = match mode {
            "r" | "rb" => VfsMode::Read,
            "w" | "wb" => VfsMode::Write,
            "a" | "ab" => VfsMode::Append,
            _ => return 0,
        };

        if mode == VfsMode::Read && !self.files.contains_key(path) {
            return 0; // 读模式文件不存在
        }

        if mode == VfsMode::Write {
            // 覆盖模式：清空现有文件或创建新文件
            if let Some(meta) = self.files.get_mut(path) {
                meta.size = 0;
            } else {
                let initial_cap = 256usize;
                let addr = match malloc_raw(memory, initial_cap, vm.get_memory_size()) {
                    Some(a) => a,
                    None => return 0,
                };
                // 更新 region 名称
                for r in &mut memory.regions {
                    if r.addr == addr {
                        r.name = format!("vfs:{}", path);
                        break;
                    }
                }
                self.files.insert(
                    path.to_string(),
                    VfsFileMeta {
                        heap_addr: addr,
                        size: 0,
                        capacity: initial_cap,
                    },
                );
        }

        if mode == VfsMode::Append && !self.files.contains_key(path) {
                let initial_cap = 256usize;
                let addr = match malloc_raw(memory, initial_cap, vm.get_memory_size()) {
                    Some(a) => a,
                    None => return 0,
                };
                for r in &mut memory.regions {
                    if r.addr == addr {
                        r.name = format!("vfs:{}", path);
                        break;
                    }
                }
                self.files.insert(
                    path.to_string(),
                    VfsFileMeta {
                        heap_addr: addr,
                        size: 0,
                        capacity: initial_cap,
                    },
                );
            }
        }

        let fd = self.next_fd + 1;
        self.next_fd = fd;

        let cursor = if mode == VfsMode::Append {
            self.files.get(path).map(|m| m.size).unwrap_or(0)
        } else {
            0
        };

        self.descriptors.insert(
            fd,
            VfsDesc {
                file_name: path.to_string(),
                mode,
                cursor,
                eof: false,
            },
        );
        fd
    }

    /// fread：从文件读取数据到 VM 内存 buf，返回实际读取的元素数
    pub fn fread(&mut self, fd: u32, buf: u32, size: usize, nmemb: usize, vm: &mut CideVM) -> usize {
        let desc = match self.descriptors.get_mut(&fd) {
            Some(d) => d,
            None => return 0,
        };
        let meta = match self.files.get(&desc.file_name) {
            Some(m) => m,
            None => return 0,
        };
        let to_read = size * nmemb;
        let remaining = meta.size.saturating_sub(desc.cursor);
        let actual = to_read.min(remaining);
        if actual == 0 {
            desc.eof = true;
            return 0;
        }
        let src_addr = meta.heap_addr + desc.cursor as u32;
        let mut tmp = vec![0u8; actual];
        if !vm.read_memory_to(src_addr, &mut tmp) || !vm.write_memory(buf, &tmp) {
            return 0;
        }
        desc.cursor += actual;
        if desc.cursor >= meta.size {
            desc.eof = true;
        }
        actual / size
    }

    /// fwrite：从 VM 内存 buf 写入数据到文件，返回实际写入的元素数
    pub fn fwrite(
        &mut self,
        fd: u32,
        buf: u32,
        size: usize,
        nmemb: usize,
        vm: &mut CideVM,
        memory: &mut MemoryState,
    ) -> usize {
        let desc = match self.descriptors.get_mut(&fd) {
            Some(d) => d,
            None => return 0,
        };
        if desc.mode == VfsMode::Read {
            return 0;
        }
        let to_write = size * nmemb;
        let meta = match self.files.get_mut(&desc.file_name) {
            Some(m) => m,
            None => return 0,
        };
        let new_size = desc.cursor + to_write;
        if new_size > meta.capacity {
            let new_cap = align4(new_size.max(meta.capacity * 2));
            if !realloc_vfs_file(memory, vm, meta, new_cap) {
                return 0;
            }
        }
        let dst_addr = meta.heap_addr + desc.cursor as u32;
        let mut tmp = vec![0u8; to_write];
        if !vm.read_memory_to(buf, &mut tmp) || !vm.write_memory(dst_addr, &tmp) {
            return 0;
        }
        desc.cursor += to_write;
        if desc.cursor > meta.size {
            meta.size = desc.cursor;
        }
        nmemb
    }

    /// fclose：关闭文件描述符
    pub fn fclose(&mut self, fd: u32, _memory: &mut MemoryState) -> i32 {
        if self.descriptors.remove(&fd).is_some() {
            0
        } else {
            -1
        }
    }

    /// feof：检查是否到达文件末尾
    pub fn feof(&self, fd: u32) -> i32 {
        match self.descriptors.get(&fd) {
            Some(d) if d.eof => 1,
            _ => 0,
        }
    }

    /// fgets：从文件读取一行（最多 n-1 字符），写入 buf 并追加 \0
    /// 返回 buf 地址（成功）或 0（失败/EOF）
    pub fn fgets(&mut self, fd: u32, buf: u32, n: usize, vm: &mut CideVM) -> u32 {
        let desc = match self.descriptors.get_mut(&fd) {
            Some(d) => d,
            None => return 0,
        };
        let meta = match self.files.get(&desc.file_name) {
            Some(m) => m,
            None => return 0,
        };
        if n == 0 {
            return 0;
        }
        let mut read_count = 0usize;
        let mut tmp = Vec::new();
        while read_count < n - 1 && desc.cursor < meta.size {
            let byte_addr = meta.heap_addr + desc.cursor as u32;
            let mut b = [0u8; 1];
            if !vm.read_memory_to(byte_addr, &mut b) {
                break;
            }
            tmp.push(b[0]);
            desc.cursor += 1;
            read_count += 1;
            if b[0] == b'\n' {
                break;
            }
        }
        if read_count == 0 {
            desc.eof = true;
            return 0;
        }
        tmp.push(0);
        if !vm.write_memory(buf, &tmp) {
            return 0;
        }
        if desc.cursor >= meta.size {
            desc.eof = true;
        }
        buf
    }

    /// fputs：将字符串 s 写入文件
    /// 返回非负数（成功）或 EOF（-1，失败）
    pub fn fputs(&mut self, fd: u32, s_addr: u32, vm: &mut CideVM, memory: &mut MemoryState) -> i32 {
        let desc = match self.descriptors.get_mut(&fd) {
            Some(d) => d,
            None => return -1,
        };
        if desc.mode == VfsMode::Read {
            return -1;
        }
        let meta = match self.files.get_mut(&desc.file_name) {
            Some(m) => m,
            None => return -1,
        };
        // 读取 C 字符串长度
        let mut len = 0usize;
        loop {
            let mut b = [0u8; 1];
            if !vm.read_memory_to(s_addr + len as u32, &mut b) || b[0] == 0 {
                break;
            }
            len += 1;
        }
        let to_write = len;
        let new_size = desc.cursor + to_write;
        if new_size > meta.capacity {
            let new_cap = align4(new_size.max(meta.capacity * 2));
            if !realloc_vfs_file(memory, vm, meta, new_cap) {
                return -1;
            }
        }
        let dst_addr = meta.heap_addr + desc.cursor as u32;
        let mut tmp = vec![0u8; to_write];
        if !vm.read_memory_to(s_addr, &mut tmp) || !vm.write_memory(dst_addr, &tmp) {
            return -1;
        }
        desc.cursor += to_write;
        if desc.cursor > meta.size {
            meta.size = desc.cursor;
        }
        0
    }

    /// 获取文件描述符对应的文件名（用于调试）
    pub fn get_file_name(&self, fd: u32) -> Option<&str> {
        self.descriptors.get(&fd).map(|d| d.file_name.as_str())
    }

    /// 序列化所有预设文件的内容（用于 Session 保存）。
    pub fn snapshot_files(&self, vm: &CideVM) -> Vec<(String, Vec<u8>)> {
        let mut result = Vec::new();
        for (name, meta) in &self.files {
            let mut buf = vec![0u8; meta.size];
            if vm.read_memory_to(meta.heap_addr, &mut buf) {
                result.push((name.clone(), buf));
            }
        }
        result
    }

    /// 从快照恢复预设文件（用于 Session 加载）。
    pub fn restore_files(&mut self, vm: &mut CideVM, memory: &mut MemoryState, files: &[(String, Vec<u8>)]) {
        self.files.clear();
        self.descriptors.clear();
        for (name, data) in files {
            self.inject_preset_file(name, data, vm, memory);
        }
    }
}

// ========== 内部辅助函数 ==========

fn align4(size: usize) -> usize {
    (size + 3) & !3
}

/// 从 MemoryState 分配原始内存（类似 host_malloc 但不操作 VM 栈）
fn malloc_raw(memory: &mut MemoryState, aligned_size: usize, mem_size: u32) -> Option<u32> {
    let addr = memory.allocate_raw(aligned_size as u32, mem_size)?;
    // reuse or add region
    let mut reused = false;
    for r in &mut memory.regions {
        if r.addr == addr && r.is_freed {
            r.is_freed = false;
            r.size = aligned_size as i32;
            reused = true;
            break;
        }
    }
    if !reused {
        memory.alloc_counter += 1;
        memory.regions.push(MemoryRegion {
            addr,
            size: aligned_size as i32,
            name: format!("heap_{}", memory.alloc_counter),
            ty: "int".to_string(),
            is_heap: true,
            is_freed: false,
            alloc_line: 0,
            alloc_by: "vfs".to_string(),
        });
    }
    Some(addr)
}

/// 释放原始内存到 MemoryState（类似 host_free）
fn free_raw(memory: &mut MemoryState, addr: u32) {
    for r in &mut memory.regions {
        if r.addr == addr && !r.is_freed {
            r.is_freed = true;
            let aligned_size = ((r.size as u32) + 3) & !3;
            memory.free_list.push(FreeBlock {
                addr: r.addr,
                size: aligned_size as i32,
            });
            memory.merge_free_list();
            break;
        }
    }
}

/// 为 VFS 文件扩容（类似 realloc，原地缩容/扩容）
fn realloc_vfs_file(
    memory: &mut MemoryState,
    vm: &mut CideVM,
    meta: &mut VfsFileMeta,
    new_cap: usize,
) -> bool {
    if new_cap <= meta.capacity {
        return true;
    }
    let new_addr = match malloc_raw(memory, new_cap, vm.get_memory_size()) {
        Some(a) => a,
        None => return false,
    };
    if !vm.copy_memory(new_addr, meta.heap_addr, meta.size) {
        free_raw(memory, new_addr);
        return false;
    }
    free_raw(memory, meta.heap_addr);
    meta.heap_addr = new_addr;
    meta.capacity = new_cap;
    // 更新 region 名称和大小
    for r in &mut memory.regions {
        if r.addr == new_addr && !r.is_freed {
            r.size = meta.size as i32;
            break;
        }
    }
    true
}
