# Host Function 契约测试失败记录

> **原则**：All in. Record don't hide. Fix real bugs, not test cases.
>
> 任何失败必须在此文件中按规范格式追加，禁止通过修改测试预期值来粉饰数据。

---

## 已修复（FIXED）

### host_atoi 前缀解析偏差

- **来源**: Host Contract
- **失败原因**: 运行时错误 / 标准一致性偏差
- **最小复现**:
  ```rust
  write_test_string(&mut vm, addr, "  -123abc");
  host_atoi(&mut vm, &mut session);
  // 预期: -123（C 标准行为）
  // 实际: 0（Rust 的 parse::<i32>() 对非纯数字字符串返回 Err）
  ```
- **是否 Cide 限制**: 否
- **是否标准库实现偏差**: **是**
- **学生影响评级**: P1（限制已知）— `atoi` 处理带后缀的字符串属于边缘场景，但在 K&R 示例中可能出现
- **修复**: 将 `parse::<i32>()` 替换为手动前缀解析（跳过前导空白 → 处理符号 → 读取连续数字 → 遇到非数字停止）。
- **修复提交**: Phase A 实施中同步修复

---

### host_realloc(NULL, size) 栈参数传递错误

- **来源**: Host Contract
- **失败原因**: 运行时错误
- **最小复现**:
  ```rust
  vm.push(64); // new_size
  vm.push(0);  // ptr
  host_realloc(&mut vm, &mut session);
  // 预期: 返回非 NULL（等价于 malloc(64)）
  // 实际: 返回 NULL 并伴随 malloc(0) 警告
  ```
- **是否 Cide 限制**: 否
- **是否标准库实现偏差**: **是**
- **学生影响评级**: P0（误导学生）— `realloc(ptr, 0)` 和 `realloc(NULL, size)` 是 C 标准中的常见用法，行为错误会导致学生困惑
- **根因**: `host_realloc` 在 `ptr == 0` 分支直接调用 `host_malloc(vm, session)`，但此时 `new_size` 已被 pop 出栈，`host_malloc` 因栈空而读到 size=0。
- **修复**: 在调用 `host_malloc` 前将 `new_size` push 回栈：`vm.push(new_size as u64); host_malloc(vm, session);`。
- **修复提交**: Phase A 实施中同步修复

---

### host_memset NULL 指针安全检查缺失

- **来源**: Host Contract
- **失败原因**: 安全检查失效
- **最小复现**:
  ```rust
  vm.push(1u64);
  vm.push(0x42u64);
  vm.push(0u64); // NULL ptr
  host_memset(&mut vm, &mut session);
  // 预期: 触发 NULL trap（地址 < 0x1000）
  // 实际: 静默写入 VM 内存地址 0，绕过所有安全检查
  ```
- **是否 Cide 限制**: 否
- **是否标准库实现偏差**: **是**
- **学生影响评级**: P0（误导学生）— 绕过 NULL trap 使学生无法发现未初始化指针的错误
- **根因**: `host_memset` 直接通过 `memory_ref_mut()` 操作原始内存切片，完全绕过了 `check_mem_access`。
- **修复**: 在填充前增加 NULL 指针检查（与 VM `store_i8` 行为保持一致）：
  ```rust
  if ptr < super::vm::NULL_TRAP_SIZE && write_len > 0 {
      vm.trap("向 NULL 指针区域写入...", ...);
      return;
  }
  ```
- **修复提交**: Phase A 实施中同步修复

---

## 已修复（FIXED）

### host_strcpy 越界触发 E3070 Buffer Overflow

- **来源**: Host Contract — `test_strcpy_overflow_must_trap`
- **失败原因**: 安全检查失效
- **最小复现**:
  ```c
  char *buf = malloc(3);
  strcpy(buf, "hello"); // 需要 6 字节（含 \0），但 buf 只有 3 字节
  ```
- **是否 Cide 限制**: 否
- **是否标准库实现偏差**: **是**
- **学生影响评级**: P0（误导学生）— strcpy 越界是 C 教学中最典型的缓冲区溢出示例，不触发 trap 意味着学生无法得到即时反馈
- **修复**: 在 `host_strcpy` 中通过 `session.memory.regions` 查找 `dest` 所属的已分配堆区域。若找到，验证 `src_len + 1 <= region.size - offset`；若越界则触发 `Trap` 并输出 `E3070 Buffer Overflow` 诊断信息，包含区域名、可用空间和修复建议。
- **限制**: 当前仅对 **堆分配区域**（`malloc`/`realloc`）生效。栈数组和全局数组的边界检查需依赖编译器符号表扩展，待后续实现。
- **修复提交**: Phase A KNOWN_FAILURE 修复

---

### host_strcat 越界触发 E3070 Buffer Overflow

- **来源**: Host Contract — `test_strcat_overflow_must_trap`
- **失败原因**: 安全检查失效
- **最小复现**:
  ```c
  char *buf = malloc(4);
  strcpy(buf, "ab");
  strcat(buf, " world"); // 已有 3 字节 + 需要 7 字节 = 10 字节，但 buf 只有 4 字节
  ```
- **是否 Cide 限制**: 否
- **是否标准库实现偏差**: **是**
- **学生影响评级**: P0（误导学生）
- **修复**: 与 `host_strcpy` 同理，通过 `session.memory.regions` 检查 `dest_len + src_len + 1 <= region.size - offset`，越界时触发 `E3070 Buffer Overflow`。
- **限制**: 同上，当前仅覆盖堆分配区域。
- **修复提交**: Phase A KNOWN_FAILURE 修复

---

## 待进一步分析（TODO）

### `printf("%.2f", 2.675)` 舍入偏差

- **来源**: Host Contract（尚未编写测试，待补充）
- **预期行为**: C99 标准不要求特定舍入方向，但常见实现（glibc/Clang）使用银行家舍入或向远离零舍入，输出 `"2.67"` 或 `"2.68"`。
- **当前状态**: 未测试。Cide 的 `format_printf_string` 使用 Rust 的 `format!("{:.2}")`，其行为是银行家舍入（round half to even）。
- **建议**: 编写契约测试记录实际行为，若与主流 Clang 行为偏差则标记。

### `memcpy` / `memmove` Host Contract 缺失

- **来源**: Host Contract
- **当前状态**: `memcpy` / `memmove` 尚未实现为 Host Function（仅有 `memset`）。
- **建议**: 当添加 `memcpy` / `memmove` Host Func 时，必须同步补充边界检查 + UAF 检查契约测试。

---

*文档状态：Phase A 实施中*
*最后更新：2026-06-07*
