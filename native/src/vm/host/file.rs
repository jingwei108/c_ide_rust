use super::*;

pub fn host_fopen(vm: &mut CideVM, session: &mut Session) {
    // 参数从右到左压栈：path, mode → 栈顶是 path
    let path_addr = vm.pop() as u32;
    let mode_addr = vm.pop() as u32;
    let path = read_cstring(vm, path_addr);
    let mode = read_cstring(vm, mode_addr);
    let mut vfs = std::mem::take(&mut session.vfs);
    let fd = vfs.fopen(&path, &mode, vm, &mut session.memory);
    session.vfs = vfs;
    // 在 VM Heap 中分配 4 字节存储 fd，返回 FILE*
    if fd != 0 {
        let mut file_ptr = 0u32;
        let aligned = 4u32;
        let addr = session.memory.heap_offset;
        let new_offset = addr as u64 + aligned as u64;
        if new_offset <= vm.get_memory_size() as u64 && new_offset <= u32::MAX as u64 {
            session.memory.heap_offset = new_offset as u32;
            let mut reused = false;
            for r in &mut session.memory.regions {
                if r.addr == addr && r.is_freed {
                    r.is_freed = false;
                    r.size = 4;
                    r.name = format!("FILE:{}", path);
                    reused = true;
                    break;
                }
            }
            if !reused {
                session.memory.alloc_counter += 1;
                session.memory.regions.push(MemoryRegion {
                    addr,
                    size: 4,
                    name: format!("FILE:{}", path),
                    ty: "int".to_string(),
                    is_heap: true,
                    is_freed: false,
                    alloc_line: vm.get_current_line(),
                    alloc_by: "fopen".to_string(),
                });
            }
            // 写入 fd 到 FILE* 结构体
            let mem = vm.memory_ref_mut();
            let a = addr as usize;
            mem[a..a + 4].copy_from_slice(&(fd as i32).to_le_bytes());
            file_ptr = addr;
        }
        vm.push(file_ptr as u64);
    } else {
        vm.push(0);
    }
}

pub fn host_fread(vm: &mut CideVM, session: &mut Session) {
    // 参数从右到左压栈：buf, size, nmemb, stream → 栈顶是 buf
    let buf = vm.pop() as u32;
    let size = vm.pop() as usize;
    let nmemb = vm.pop() as usize;
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let mut vfs = std::mem::take(&mut session.vfs);
    let n = vfs.fread(fd, buf, size, nmemb, vm);
    session.vfs = vfs;
    vm.push(n as u64);
}

pub fn host_fwrite(vm: &mut CideVM, session: &mut Session) {
    // 参数从右到左压栈：buf, size, nmemb, stream → 栈顶是 buf
    let buf = vm.pop() as u32;
    let size = vm.pop() as usize;
    let nmemb = vm.pop() as usize;
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let mut vfs = std::mem::take(&mut session.vfs);
    let n = vfs.fwrite(fd, buf, size, nmemb, vm, &mut session.memory);
    session.vfs = vfs;
    vm.push(n as u64);
}

pub fn host_fclose(vm: &mut CideVM, session: &mut Session) {
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let mut vfs = std::mem::take(&mut session.vfs);
    let ret = vfs.fclose(fd, &mut session.memory);
    session.vfs = vfs;
    vm.push(ret as u64);
}

pub fn host_feof(vm: &mut CideVM, session: &mut Session) {
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let ret = session.vfs.feof(fd);
    vm.push(ret as u64);
}

pub fn host_fgets(vm: &mut CideVM, session: &mut Session) {
    // 参数从右到左压栈：buf, n, stream → 栈顶是 buf
    let buf = vm.pop() as u32;
    let n = vm.pop() as usize;
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let ret = session.vfs.fgets(fd, buf, n, vm);
    vm.push(ret as u64);
}

pub fn host_fputs(vm: &mut CideVM, session: &mut Session) {
    // 参数从右到左压栈：s, stream → 栈顶是 s
    let s_addr = vm.pop() as u32;
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let mut vfs = std::mem::take(&mut session.vfs);
    let ret = vfs.fputs(fd, s_addr, vm, &mut session.memory);
    session.vfs = vfs;
    vm.push(ret as u64);
}

/// 从 FILE* 指针（VM Heap 地址）读取 fd 索引
pub fn host_fgetc(vm: &mut CideVM, session: &mut Session) {
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let mut vfs = std::mem::take(&mut session.vfs);
    let ch = vfs.fgetc(fd, vm);
    session.vfs = vfs;
    vm.push(ch as i64 as u64);
}

pub fn host_fputc(vm: &mut CideVM, session: &mut Session) {
    let c = vm.pop() as i32;
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let mut vfs = std::mem::take(&mut session.vfs);
    let ret = vfs.fputc(fd, c, vm, &mut session.memory);
    session.vfs = vfs;
    vm.push(ret as i64 as u64);
}

pub fn host_fseek(vm: &mut CideVM, session: &mut Session) {
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let offset = vm.pop() as i32;
    let whence = vm.pop() as i32;
    let mut vfs = std::mem::take(&mut session.vfs);
    let ret = vfs.fseek(fd, offset, whence, vm);
    session.vfs = vfs;
    vm.push(ret as u64);
}

pub fn host_ftell(vm: &mut CideVM, session: &mut Session) {
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let vfs = std::mem::take(&mut session.vfs);
    let ret = vfs.ftell(fd, vm);
    session.vfs = vfs;
    vm.push(ret as i64 as u64);
}

pub fn host_rewind(vm: &mut CideVM, session: &mut Session) {
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let mut vfs = std::mem::take(&mut session.vfs);
    vfs.rewind(fd);
    session.vfs = vfs;
}

// ========== String / memory extensions ==========

pub fn host_fflush(vm: &mut CideVM, session: &mut Session) {
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let mut vfs = std::mem::take(&mut session.vfs);
    let ret = if fd == 0 {
        // fflush(NULL) — 刷新所有流，VFS 内存模式无缓冲，视为成功
        0
    } else {
        vfs.fflush(fd)
    };
    session.vfs = vfs;
    vm.push(ret as u64);
}

pub fn host_perror(vm: &mut CideVM, session: &mut Session) {
    let s_addr = vm.pop() as u32;
    let prefix = read_cstring(vm, s_addr);
    let msg = if prefix.is_empty() {
        "Error\n".to_string()
    } else {
        format!("{}: Error\n", prefix)
    };
    session.runtime.output_lines.push(msg);
}

pub fn host_clearerr(vm: &mut CideVM, session: &mut Session) {
    let stream = vm.pop() as u32;
    let fd = read_fd_from_stream(vm, stream);
    let mut vfs = std::mem::take(&mut session.vfs);
    vfs.clearerr(fd);
    session.vfs = vfs;
}

pub fn host_remove(vm: &mut CideVM, session: &mut Session) {
    let path_addr = vm.pop() as u32;
    let path = read_cstring(vm, path_addr);
    let mut vfs = std::mem::take(&mut session.vfs);
    let ret = vfs.remove(&path, &mut session.memory);
    session.vfs = vfs;
    vm.push(ret as u64);
}

pub fn host_rename(vm: &mut CideVM, session: &mut Session) {
    let old_path_addr = vm.pop() as u32;
    let new_path_addr = vm.pop() as u32;
    let old_path = read_cstring(vm, old_path_addr);
    let new_path = read_cstring(vm, new_path_addr);
    let mut vfs = std::mem::take(&mut session.vfs);
    let ret = vfs.rename(&old_path, &new_path);
    session.vfs = vfs;
    vm.push(ret as u64);
}
