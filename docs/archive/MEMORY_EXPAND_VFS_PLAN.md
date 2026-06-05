# 内存扩容 + File I/O (VFS) 实现计划

> **状态**：Phase 0（内存扩容 256KB → 1MB）✅ 已完成（2026-05-17）  
> VFS（File I/O）Phase 1-6 待后续排期

## 背景

影子验证缺口 Top 2 为 `function_pointer` (3) 和 `file_io` (3)。用户选择优先实现 `file_io`，核心诉求是**教学可视化**——学生能在内存 Canvas 中看到文件缓冲区的实际读写过程。

但在 VFS 实现之前，必须先进行**内存扩容**。原因：
1. `ZERO_INTRUSIVE_VISUALIZATION.md` 数据结构动画需要大量内存存储状态快照
2. 当前内存布局存在一个严重问题：61% 的内存（156KB/256KB）完全未被使用
3. 先做扩容再做 VFS，避免两次改动内存布局

---

## Phase 0：VM 内存扩容（Rust + Flutter，~3h）

### 0.1 当前内存布局的问题

当前 `native/src/vm/vm.rs` 中：

```rust
pub const MEM_SIZE: u32 = 256 * 1024;     // 0x40000 = 256 KB
pub const STACK_START: u32 = 0x10000;      // 64 KB 处
```

- Heap 从 `0x5000` 向**高地址**增长，由 `heap_offset` 管理
- Stack 从 `0x10000` 向**低地址**增长，由 `mem_stack_top` 管理
- 碰撞检查：`heap_offset + alloc_size <= mem_stack_top`

**问题**：Stack 起始固定在 `0x10000`，导致 `0x10000 ~ 0x3FFFF`（156KB = 61%）**完全空闲**，Heap 最多只能用 44KB，Stack 最多只能用 60KB。

### 0.2 修复方案：Stack 顶对齐 MEM_SIZE

将 `STACK_START` 改为 `MEM_SIZE`，让 Stack 从内存顶部向下增长：

```rust
pub const MEM_SIZE: u32 = 1024 * 1024;     // 0x100000 = 1 MB
pub const STACK_START: u32 = MEM_SIZE;      // Stack 从顶部开始
```

扩容后布局（1MB）：

```
NULL Trap   0x00000 - 0x00FFF    4 KB
Global      0x01000 - 0x04FFF   16 KB
Heap        0x05000 - ...        向高地址增长（上限 ~1MB）
Stack       ... - 0xFFFFF        向低地址增长（下限 0x1000）
```

- Heap + Stack 共享约 1020KB 空间（程序行为决定各自占用）
- 相比当前，真正可用的动态内存从 ~104KB 提升到 ~1020KB（**~10x**）

### 0.3 修改文件清单

| 文件 | 改动 |
|------|------|
| `native/src/vm/vm.rs` | `MEM_SIZE: 256*1024 → 1024*1024`; `STACK_START: 0x10000 → MEM_SIZE` |
| `native/src/vm/vm.rs` | `reset()`、`new()` 中 `mem_stack_top = STACK_START`（已正确，确认即可） |
| `native/src/vm/vm.rs` | 所有 `MEM_SIZE` 直接引用检查（如 `format_bounds_error`）— 已通过 `get_memory_size()` 读取，通常无需改 |
| `native/src/api/cide.rs` | 新增 `#[frb] pub fn get_memory_size() -> u32` |
| `native/src/flutter_bridge.rs` | 新增 `pub fn get_memory_size() -> u32` |
| `CideFlutter/lib/widgets/memory_map_visualizer.dart` | 硬编码 `256 * 1024` 等 → 通过 Provider/参数传入，或调用 `getMemorySize()` |

### 0.4 前端内存可视化适配

当前 `memory_map_visualizer.dart` 中硬编码：
```dart
const memorySize = 256 * 1024;
const blockSize = 4096;
const blockCount = memorySize ~/ blockSize; // 64
const crossAxisCount = 8;
```

1MB 下变为 256 块。若保持 `crossAxisCount = 8`，则需要 32 行，GridView 可滚动，适合。

**推荐方案**：前端通过 FRB 调用 `getMemorySize()` 动态获取，计算 `blockCount`，保持 `blockSize = 4096` 不变。这样以后改 `MEM_SIZE` 只需改 Rust 一处。

> **注意**：FRB 生成代码需要重新运行 `flutter_rust_bridge_codegen generate`（或在构建脚本中自动处理）。

### 0.5 测试

- [ ] 现有 Rust 单元测试全部通过（`cargo test --release`）
- [ ] `cargo clippy` 零警告
- [ ] 影子验证框架无需改动（它不依赖内存大小）
- [ ] E2E 测试：`test_e2e_malloc_large`、`test_e2e_deep_recursion` 等需要大内存的用例应该更稳定

---

## Phase 1：VFS 核心数据结构（Rust，~2h）

在扩容后的内存基础上实现 VFS。

### 设计变更：文件数据放在 VM Heap 中

原方案把文件数据放在 Rust 端 `HashMap<String, Vec<u8>>`，但这样前端 Canvas 看不到文件内容。

**修正方案**：
- VFS **元数据**在 Rust 端：文件名→(fd, heap_addr, size, cursor) 映射
- VFS **文件数据**在 VM Heap 中：通过 `host_malloc` 分配，前端 Canvas 自动显示为橙色块

新建 `native/src/vm/vfs.rs`：

```rust
pub struct VirtualFileSystem {
    files: HashMap<String, VfsFile>,      // 文件名 -> 文件元数据
    descriptors: HashMap<u32, VfsDesc>,    // fd -> 描述符状态
    next_fd: u32,
}

pub struct VfsFile {
    heap_addr: u32,    // VM Heap 中文件数据的起始地址
    size: usize,       // 当前文件大小
    capacity: usize,   // 分配的 Heap 块大小
}

pub struct VfsDesc {
    name: String,
    mode: VfsMode,     // Read / Write / Append
    cursor: usize,
    eof: bool,
}
```

预设文件注入（教学用，写入 VM Heap）：
- `test.txt` = `"hello\nworld\n"`
- `numbers.txt` = `"1 2 3 4 5\n"`

在 `Session` 中添加 `vfs: VirtualFileSystem`。

---

## Phase 2：Host 函数实现（Rust，~3h）

新增 host func ID（`host_func_id.rs`）：`FOPEN`、`FREAD`、`FWRITE`、`FCLOSE`、`FEOF`。

在 `host_funcs.rs` 中实现：

**`host_fopen(vm, session)`**
1. Pop `mode` (char*), `path` (char*)
2. 从 VM 内存读取 path/mode 字符串
3. 在 `session.vfs` 中查找或创建文件
4. **如果是新文件或 Write 模式**：通过 `host_malloc` 在 VM Heap 分配初始容量（如 256B）
5. **如果是预设文件**：数据已在 Heap 中
6. 在 VM Heap 中分配 4 字节存储 fd 索引（`malloc(4)`）
7. 将 fd 写入该 Heap 块
8. Push Heap 地址作为 `FILE*` 返回值
9. 失败时 Push 0 (NULL)

**`host_fread(vm, session)`**
1. Pop `stream` (FILE*), `nmemb`, `size`, `buf` (void*)
2. 从 stream 地址读取 fd
3. 从 VFS 文件 cursor 位置读取 `size * nmemb` 字节
4. **源地址 = 文件数据在 VM Heap 中的地址 + cursor**
5. 写入 VM 内存 `buf` 地址
6. 更新 cursor
7. Push 实际读取的 nmemb 数

**`host_fwrite(vm, session)`**
1. Pop `stream`, `nmemb`, `size`, `buf`
2. 从 VM 内存 `buf` 读取数据
3. 如果 `cursor + write_size > capacity`，通过 `realloc` 扩容 Heap 块
4. 写入 VFS 文件数据区（VM Heap 地址 + cursor）
5. 更新 size / cursor
6. Push 实际写入的 nmemb 数

**`host_fclose(vm, session)`**
1. Pop `stream`
2. 读取 fd，从描述符表中移除
3. 释放 Heap 上的 fd 结构体（`free`）
4. Push 0 (成功)

**`host_feof(vm, session)`**
1. Pop `stream`
2. Push 1/0 (是否 EOF)

---

## Phase 3：编译器接线（Rust，~1h）

在 `bytecode_gen.rs` 的 host func 识别列表中添加：
- `fopen` → `FOPEN`
- `fread` → `FREAD`
- `fwrite` → `FWRITE`
- `fclose` → `FCLOSE`
- `feof` → `FEOF`

TypeChecker 最简方案：`fopen` 返回 `void*`（教学场景足够）。后续如需更严格类型检查，可新增 `TypeKind::FilePtr`。

---

## Phase 4：影子验证用例修复（Python，~30min）

更新 3 个 file_io 用例：

```python
ShadowCase("file_fopen", 'int main() { FILE* f = fopen("test.txt", "r"); if (f) printf("ok"); fclose(f); return 0; }', "file_io"),
ShadowCase("file_fread", 'int main() { FILE* f = fopen("test.txt", "r"); char buf[20]; fread(buf, 1, 5, f); buf[5] = 0; printf("%s", buf); fclose(f); return 0; }', "file_io"),
ShadowCase("file_fwrite", 'int main() { FILE* f = fopen("out.txt", "w"); fwrite("hello", 1, 5, f); fclose(f); printf("ok"); return 0; }', "file_io"),
```

---

## Phase 5：E2E 测试（Rust，~1h）

新增 5 个端到端测试：
- `test_e2e_file_read`：fopen + fread + fclose
- `test_e2e_file_write`：fopen + fwrite + fclose
- `test_e2e_file_append`：fopen("a") + fwrite
- `test_e2e_file_not_found`：fopen("noexist.txt", "r") 返回 NULL
- `test_e2e_file_heap_visible`：验证文件数据确实在 VM Heap 中（通过 memory regions 检查）

---

## Phase 6：前端可视化增强（可选，~2h）

### 方案 A：最小改动（推荐先实现）
- VFS 文件数据在 VM Heap 中，内存 Canvas 已自动显示为橙色块
- `host_fread`/`host_fwrite` 向 `output_lines` 推送标记如 `[VFS] fread 5 bytes from test.txt → 0x6000`
- 前端日志面板显示 I/O 操作记录

### 方案 B：高亮读写过程（后续增强）
- 新增 "文件 I/O" 可视化面板，显示 VFS 中的文件列表、内容、cursor 位置
- 内存 Canvas 中高亮当前操作涉及的 buf 区域和文件数据区域

---

## 内存扩容决策总结

| 项目 | 扩容前 | 扩容后 |
|------|--------|--------|
| MEM_SIZE | 256 KB | 1 MB |
| STACK_START | 0x10000（硬编码） | MEM_SIZE（动态顶部对齐） |
| Heap 可用 | ~44 KB | ~1020 KB（与 Stack 共享） |
| Stack 可用 | ~60 KB | ~1020 KB（与 Heap 共享） |
| 内存浪费 | 156 KB (61%) | 0 |
| 前端 blockCount | 64 | 256 |

**关键修复**：`STACK_START = MEM_SIZE` 消除了当前布局中的内存浪费，是真正的 bug 修复 + 扩容。

---

## 风险评估

| 风险 | 概率 | 缓解措施 |
|------|------|----------|
| `STACK_START = MEM_SIZE` 导致栈溢出检查逻辑异常 | 低 | 碰撞检查 `mem_stack_top < NULL_TRAP_SIZE + frame_size` 不变，只是起始位置变了 |
| 前端 GridView 256 块性能下降 | 低 | 256 个 Container 渲染在现代设备上无压力；可改用 `ListView` 或分页 |
| FRB 重新生成绑定失败 | 低 | `flutter_rust_bridge_codegen` 版本已在项目中锁定 |
| Heap/Stack 共享空间导致新碰撞 | 中 | 碰撞检查已存在；扩容后碰撞概率反而降低 |

---

## 工作量估算

| Phase | 内容 | 预估时间 |
|-------|------|----------|
| 0 | 内存扩容（Rust 常量 + 前端动态化 + 测试） | 3h |
| 1 | VFS 核心数据结构 | 2h |
| 2 | Host 函数实现（fopen/fread/fwrite/fclose/feof） | 3h |
| 3 | 编译器接线 | 1h |
| 4 | 影子验证用例修复 | 30min |
| 5 | E2E 测试 | 1h |
| 6 | 前端可视化增强（可选） | 2h |
| **总计** | | **~2 天**（不含可选 Phase 6） |

---

## 下一步建议

1. 启动分支 `feat/memory-expand-vfs`
2. 先实现 Phase 0（内存扩容），跑通全部测试
3. 再实现 Phase 1-3（VFS 后端核心）
4. 跑通 3 个 file_io 影子验证用例
5. 决定是否投入 Phase 6 前端增强
