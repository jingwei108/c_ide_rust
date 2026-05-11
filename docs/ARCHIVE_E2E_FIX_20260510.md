# 端到端测试修复归档 — 2026-05-10

## 背景

本轮修复聚焦于 **5 个持续失败的端到端测试**：
- `test_e2e_strlen` — `strlen(s)` 返回 6（预期 5）
- `test_e2e_atoi` — `atoi(s)` 返回 0（预期 12345）
- `test_e2e_escape_sequences` — `printf("%d", s[i])` 输出乱码大整数
- `test_e2e_qsort` — `qsort` 后数组变为降序 `9 8 5 2 1`（预期升序 `1 2 5 8 9`）
- `test_e2e_qsort_struct_array` — `qsort` 后结构体数组完全未改变

通过临时调试测试（直接打印 VM 内存、字节码、host 函数调用参数），定位到 **4 个互相独立的根本缺陷**。

---

## 根本缺陷与修复

### 1. `Type::total_elements()` 对推断大小数组返回 1

**根因**：
- Parser 遇到 `char s[]` / `int arr[]` 时，`dims` 被 push `-1` 作为占位符。
- TypeChecker 的 `check_array_initializer` 在推断出实际大小后，只更新 `array_size`，从不更新 `dims`。
- `total_elements()` 看到 `dims = [-1]`，`-1 <= 0` 被映射为 1，`product() = 1`。
- BytecodeGen 的 `elem_count = (type_size + 3) / 4 = 1`，导致推断大小数组**只分配 1 个 slot**（4 字节）。

**后果**：
- `char s[] = "hello"`（需 6 字节）只分配 4 字节；`StoreMemByte` 循环写入第 5 个字节时覆盖用于保存基地址的临时 slot；后续 `LoadLocal` 读取被破坏的地址，导致 `strlen` 找不到 `\0`，返回 6。
- `int arr[] = {5,2,8,1,9}`（需 20 字节）只分配 4 字节；`InitList` 初始化只存储 `arr[0]`，其余元素为 0，`qsort` 实际在排序 `[5,0,0,0,0]`。

**修复** (`native/src/compiler/ast.rs`)：
```rust
pub fn total_elements(&self) -> i32 {
    if !self.is_array() { return 1; }
    if !self.dims.is_empty() {
        let has_negative = self.dims.iter().any(|&d| d < 0);
        if has_negative && self.array_size > 0 {
            return self.array_size;  // 回退到已推断的 array_size
        }
        self.dims.iter().map(|&d| if d > 0 { d } else { 1 }).product()
    } else if self.array_size > 0 {
        self.array_size
    } else {
        1
    }
}
```

---

### 2. `call_user_function` 参数顺序与 `Call` 指令约定不一致

**根因**：
- `vm.rs` 中 `call_user_function` 的注释声称 "last param is at locals_base + 0"，并据此反转参数写入顺序。
- 但 `Call` 指令的实际实现是：先 `pop()` 的值（即最后压栈的参数）写入 `locals_base + 0`。
- 正常调用链（`gen_expr` 从右到左压栈 → `Call` 弹出）的结果是 **第一个参数在 `locals_base + 0`**。
- `call_user_function` 把 `args[1]`（最后一个参数）写入了 `locals_base + 0`，导致 `cmp(a,b)` 实际接收到 `a=addr_b, b=addr_a`。

**后果**：
- `cmp` 计算 `ib - ia`，`qsort` 执行降序排序。
- 结构体数组 `qsort` 的比较函数也因参数反序而返回错误结果，排序无效。

**修复** (`native/src/vm/vm.rs`)：
```rust
// 移除反转逻辑，直接按 args[i] → locals_base + i*4 写入
for i in 0..meta.arg_count {
    let arg = if (i as usize) < args.len() { args[i as usize] } else { 0 };
    let arg_addr = (locals_base as u64) + (i as u64) * 4;
    self.write_i32(arg_addr as u32, arg);
}
```

---

### 3. `char` 数组元素读写误用 4 字节指令

**根因**：
- `gen_index` 和 `gen_assign` 的 `Expr::Index` 分支对所有非数组结果类型统一使用 `LoadMem` / `StoreMem`（4 字节）。
- 对 `char` 类型应使用 `LoadMemByte` / `StoreMemByte`（1 字节）。

**后果**：
- `s[0] = '\r'` 实际写入 `memory[0xFFF4..0xFFF8] = [0x0D, 0x00, 0x00, 0x00]`，覆盖后续 3 个字节。
- 后续 `s[1] = '\a'` 写入 `memory[0xFFF5..0xFFF9]`，再次覆盖邻居。
- `printf("%d", s[0])` 使用 `LoadMem` 读取 4 字节，得到 `0x0C08070D = 201852685` 等乱码。

**修复** (`native/src/compiler/bytecode_gen.rs`)：
- `gen_index` 末尾：
  ```rust
  if result_ty.kind == TypeKind::Char {
      self.emit(OpCode::LoadMemByte, 0, loc);
  } else {
      self.emit(OpCode::LoadMem, 0, loc);
  }
  ```
- `gen_assign` 的 `Expr::Index` 分支：
  - 复合赋值前的 `LoadMem` 改为 `LoadMemByte`
  - 最终存储的 `StoreMem` 改为 `StoreMemByte`

---

### 4. 函数索引起始值与 NULL 指针冲突

**根因**：
- BytecodeGen 的 `next_func_idx` 初始为 0，第一个用户函数（通常是 `cmp`）索引为 0。
- `host_qsort` 中 `if compar == 0` 被解释为 "无比较函数，使用默认字节比较"。

**后果**：
- 当 `cmp` 是第一个函数时，`compar = 0`，`qsort` 完全忽略用户回调，走默认字节比较。
- 对 `int` 数组默认比较可能产生错误排序；对 `struct` 数组默认比较按 `id` 字段排序（因为 `id` 在前面），与预期的 `score` 排序不一致。

**修复** (`native/src/compiler/bytecode_gen.rs`)：
- `next_func_idx` 初始化为 `1`，保留索引 `0` 表示 NULL 函数指针。
- VM 的 `func_table[0]` 天然为 `ip = 0` 的 dummy，`call_user_function` 对 `idx=0` 返回 `None`。

---

## 调试方法（关键）

本轮修复没有依赖猜测，而是采取了**直接观测**的方法：

1. **VM 内存快照**：在 `host_strlen` 中插入 `eprintln!` 打印 `addr` 和周围 10 字节内存，立即发现 `mem[0xFFF9] = 0xFF`（不是 `\0`）。
2. **代码生成阶段打印**：在 `bytecode_gen.rs` 的 `emit_one` 中打印 `next_local_idx` / `elem_count` / `type_size`，发现 `elem_count=1`（预期 2）。
3. **Type 结构体审计**：继续打印 `dims` 和 `array_size`，发现 `dims=[-1]` 但 `array_size=6`，锁定 `total_elements()` 缺陷。
4. **Host 函数调用追踪**：在 `host_qsort` 中打印每次 `call_user_function` 的 `addr_a` / `addr_b` / `result`，发现 `cmp` 返回 `3` 当 `a=arr[1]=2, b=arr[0]=5`，说明参数被交换。
5. **Call 约定对照**：对比 `Call` 指令的栈操作与 `call_user_function` 的内存写入，发现两者参数映射方向相反。

---

## 文件变更清单

### 修改文件

```
native/src/compiler/ast.rs              (+ total_elements 负数 dims 回退逻辑)
native/src/compiler/bytecode_gen.rs     (+ next_func_idx=1; char 数组 LoadMemByte/StoreMemByte)
native/src/vm/vm.rs                     (call_user_function 参数顺序修复)
```

### 删除文件

```
native/tests/tmp_debug_strlen.rs
native/tests/tmp_debug_strlen2.rs
native/tests/tmp_debug_qsort.rs
```

---

## 验证结果

| 验证项 | 结果 |
|---|---|
| `cargo test` 全部 Rust 测试 | ✅ **117/117** |
| `end_to_end_extra_test.rs` | ✅ **83/83** |
| `end_to_end_test.rs` | ✅ **17/17** |
| `compile_pipeline_test.rs` | ✅ **13/13** |
