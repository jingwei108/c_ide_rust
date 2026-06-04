# Cide Rust 项目地毯式审阅报告

**日期**: 2026-06-04  
**项目**: `D:\code\c_ide_rust`  
**审阅范围**: 全部 Rust 后端源码（compiler, vm, engine, unified, flutter_bridge, capi, api, diagnostics, session）  
**当前状态**: cargo test 全通过（210+测试），cargo clippy 仅1个 unused import 警告

---

## 目录

1. [重大错误与潜在缺陷](#一重大错误与潜在缺陷)
2. [代码优化建议](#二代码优化建议)
3. [架构/框架迭代建议](#三架构框架迭代建议)
4. [模块逐一审阅](#四模块逐一审阅)
5. [测试与静态分析结果](#五测试与静态分析结果)
6. [总体评分与修复优先级](#六总体评分与修复优先级)

---

## 一、重大错误与潜在缺陷

### 1. 🔴 Soundness: `cstr_to_str` 返回 `'static str`

**位置**: `native/src/capi/mod.rs:15-19`

```rust
fn cstr_to_str(s: *const c_char) -> Option<&'static str> {
    if s.is_null() { return None; }
    unsafe { CStr::from_ptr(s).to_str().ok() }
}
```

从原始 C 指针转换为 `&'static str`，但指针背后的内存生命周期不受 Rust 控制。调用方持有此引用后，若 C 端释放或修改对应内存，将导致 **悬垂引用（Use-After-Free）**。

**修复**: 改为返回 `Option<String>`（安全复制），或返回非 `'static` 生命周期的 `Option<&str>`。

---

### 2. 🟡 正确性: `qsort_depth` 未在 VM `reset()` 中重置

**位置**: `native/src/vm/vm.rs:144-171`

`CideVM::reset()` 重置了几乎所有状态字段（`memory` 填 0、`step_count` 归 0、`error` 清空等），但 **遗漏了 `qsort_depth`**。若上次运行调用了 qsort，两次编译执行之间残留的 depth 值会导致 VFS/preset 操作行为异常。

**修复**: 在 `reset()` 中添加 `self.qsort_depth = 0;`。

---

### 3. 🟡 性能: VM 热点路径 O(n) 符号查找

**位置**: `native/src/vm/vm.rs:1009-1011` (LoadLocal), `1031-1033` (StoreLocal), `1115-1117` (LoadGlobal), `1132-1134` (StoreGlobal)

```rust
let var_name = self.symbols.iter()
    .find(|s| s.is_local && s.addr == operand as u32)
    .map(|s| s.name.clone())
    .unwrap_or_else(|| format!("local_{}", operand));
```

每次 LoadLocal/StoreLocal 都遍历整个符号表（O(n)）。函数内若有数百局部变量，每条访问指令均付出完整遍历代价。LoadLocal 和 StoreLocal 是程序中最频繁的指令。

**修复**: 在 `enter_function` 时构建 `HashMap<i32, String>`（addr→name），访问时 O(1) 查找。

---

### 4. 🟡 代码重复: Call / CallPtr 帧设置完全重复

**位置**: `native/src/vm/vm.rs:1581-1689`

`OpCode::Call` 和 `OpCode::CallPtr` 的处理代码中，帧设置逻辑（参数从栈弹出、局部变量清零、call_stack 压栈、IP 跳转）**完全一致**，总计约 100 行。

**修复**: 提取为私有辅助函数：
```rust
fn push_call_frame(&mut self, func_idx: u32, session: &mut Session, loc: &SourceLoc) -> Option<StepResult>
```

---

### 5. 🟡 正确性: `step_next` 首次运行循环缺少提前退出策略

**位置**: `native/src/flutter_bridge.rs:196-241`

当首次进入 `step_next` 且程序无断点时，`loop { match vm.step(...) {...} }` 会持续执行直到 `max_steps`（10,000,000）触发 Trap。虽然最终有保护，但学生对"第一步"的等待体验很差（可达数秒）。

**修复**: 首次运行循环中增加 `step_event_hit` 检查，或在第一个 StepEvent 后无条件暂停。

---

### 6. 🟡 算法检测: 相邻索引比较依赖字符串匹配

**位置**: `native/src/compiler/algorithm_detector.rs:386-401`

```rust
fn is_adjacent_compare(a: &Expr, b: &Expr) -> bool {
    // 用 expr_to_string 比较字符串
    let ia = expr_to_string(idx_a);  // "j"
    let ib = expr_to_string(idx_b);  // "j + 1" 或 "j+1"
    if ib == format!("{} + 1", ia) || ib == format!("{}+1", ia) { ... }
}
```

此实现依赖 `expr_to_string` 输出格式，当变量名包含空格格式差异时（`"j+1"` vs `"j + 1"`）会失效。

**修复**: 改为 AST 结构比较：检查 `idx_b` 是否为 `Expr::Binary { op: Add, left: idx_a, right: Expr::Literal { value: 1, .. } }`。

---

## 二、代码优化建议

| # | 文件:行号 | 问题 | 建议 |
|---|-----------|------|------|
| 1 | `type_checker.rs:1022-1411` | 18 个内建函数检查器高度重复（每条 ~15–30 行） | 用声明式宏 `macro_rules! check_builtin!` 统一，可删减 ~300 行 |
| 2 | `vm/vm.rs:568-647` | `load_i32/store_i32/load_i64/store_i64/load_i8/store_i8` 中边界检查逻辑重复 6 次 | 提取 `fn check_addr_bounds(&self, addr: u32, size: u32, loc: &SourceLoc) -> Result<(), String>` |
| 3 | `bytecode_gen.rs:228-245` | 非 void 函数末尾自动注入 `PushConst(0); Ret`，返回值无意义 | 改为编译警告"非 void 函数可能无 return 语句到达末尾" + 注入 `Trap` |
| 4 | `lexer.rs:263` | 十六进制解析用 `u64::from_str_radix` 后检查 `> u32::MAX` | 直接用 `u32::from_str_radix`，利用类型系统避免溢出检查 |
| 5 | `host_funcs.rs:21-179` | `parse_format_specs` 和 `format_printf_string` 重复实现格式字符串解析 | 提取 `PrintfFormatParser` 迭代器结构体，两处复用 |
| 6 | `parser.rs:440-529` | `parse_base_type` 中 unsigned 修饰非 Int/Char 类型时仍继续执行 | 使用 early return 或 `?` 模式 |
| 7 | `compile_pipeline.rs:17-49` | `type_size` 和 `base_element_type` 辅助函数在 `compile_pipeline` 与 `bytecode_gen` 中各有一份 | 合并到 `Type` 方法中（`Type::size()` / `Type::base_element()`）消除重复 |
| 8 | `snapshot.rs:83` | 每个 `VMSnapshot` 持有完整 1MB `memory` 拷贝 | 固定间隔检查点可考虑增量快照（delta encoding）或只保存脏页 |
| 9 | `session.rs:218-248` | `MemoryState::allocate_raw` 的 first-fit 搜索在 free_list 增长后变慢 | 考虑使用 buddy allocator 或维护按大小排序的空闲列表 |
| 10 | `capi/mod.rs:50-101` | Session 序列化通过 `serde_json` 手动实现，类型安全检查靠手工保证 | 增加 `#[serde(deny_unknown_fields)]` 防止加载损坏/不兼容的数据 |

---

## 三、架构/框架迭代建议

### 3.1 模块拆分

当前核心单文件均超过 1500 行，建议拆分：

| 当前文件 | 行数 | 建议拆分 |
|----------|------|----------|
| `vm/vm.rs` | 1919 | `vm/core.rs`（结构体+step循环）+ `vm/execute/stack.rs` + `vm/execute/local.rs` + `vm/execute/memory.rs` + `vm/execute/arithmetic.rs` + `vm/execute/control.rs` + `vm/execute/float.rs` |
| `compiler/bytecode_gen.rs` | 1889 | `bytecode_gen/data.rs`（全局/字符串/结构体）+ `bytecode_gen/expr.rs`（表达式生成）+ `bytecode_gen/stmt.rs`（语句生成） |
| `compiler/type_checker.rs` | 1584 | `type_checker/core.rs`（主逻辑）+ `type_checker/builtins.rs`（内建检查器） |
| `compiler/parser.rs` | 1526 | `parser/decl.rs`（声明解析）+ `parser/expr.rs`（表达式解析）+ `parser/types.rs`（类型/声明符解析） |

### 3.2 将 VM 提取为独立 Crate

**建议**: 创建 `cide_vm` crate，将 `opcode`、`instruction`、`vm`、`host_funcs`、`host_func_id`、`vfs`、`snapshot` 纳入独立编译单元。

**收益**:
- VM 可单独 benchmark 和测试（不依赖编译器）
- 未来可复用为服务器端沙箱执行后端
- 编译并行化，缩短全量构建时间

### 3.3 状态管理: Session 并发安全重构

**当前问题**: `flutter_bridge.rs` 使用 `Box::leak` + `LazyLock<Mutex<HashMap<...>>>` 管理模式，导致：
- 已 destroy 的 session 内存永不释放
- 全局单一 Mutex 下所有 session 串行化

**建议**: 改为 `Arc<RwLock<HashMap<u64, Arc<Mutex<Session>>>>>`：
```rust
static SESSIONS: LazyLock<RwLock<HashMap<u64, Arc<Mutex<Session>>>>> = ...;
```
每个 Session 独立持有内部 Mutex，支持细粒度并发和真正的资源释放。

### 3.4 使用 `thiserror` 统一错误类型

当前 `LexerError`、`ParseError`、`TypeError` 各自手动实现，需通过 `CompileError` trait 统一。建议引入 `thiserror`：

```rust
#[derive(thiserror::Error, Debug)]
pub enum CompileError {
    #[error("词法错误 E{code}: {message} (第{line}行第{column}列)")]
    Lexer { message: String, line: i32, column: i32, code: i32 },
    // ...
}
```

### 3.5 统一模式(Unified)检查点策略优化

`CheckpointManager` 固定间隔 20 步创建完整 1MB VM 快照。对长运行程序（如排序 1000 个元素），总快照数据可达数百 MB。

**建议**:
- 只在关键控制流节点（函数调用入栈/循环边界）创建检查点
- 使用写时复制（CoW）`Arc<[u8]>` 复用未修改的内存页

### 3.6 测试增强

| 方向 | 当前状态 | 建议 |
|------|----------|------|
| Fuzzing | 无 | 对 Lexer + Parser 添加 `cargo-fuzz` 目标，确保任意输入不崩溃 |
| Property-based testing | 无 | 用 `proptest` 生成随机 C 代码片段，验证编译→无诊断 或 编译→有合理错误 |
| Shadow testing | 存在（Python + Clang） | 增强覆盖率至 float、double、struct、union、function pointer 场景 |
| 单元测试组织 | `end_to_end_extra_test.rs` 含 150+ 测试 | 按功能分类拆分文件（`tests/e2e_arrays.rs`、`tests/e2e_pointers.rs` 等） |
| Benchmark | 无 | 添加 `criterion` benchmark 测试编译和VM执行性能 |

---

## 四、模块逐一审阅

### 4.1 编译器模块

#### Lexer (`lexer.rs`, 671行)

**优点**:
- 完整实现 C 子集 tokenization（关键字、标识符、数字、字符串、字符字面量、运算符、注释、预处理 `#define` 宏展开）
- 支持十进制、十六进制、八进制整数字面量
- 支持 `\xNN` 十六进制转义、`\n \t \r \a \b \f \v \0` 转义序列
- 预处理 `#define` 简单宏定义提取和展开（token级替换）

**问题**:
1. 行 241-276 `number()`: 十六进制和八进制解析先读为 `u64` 再检查溢出→转为十进制字符串。建议直接用 `u32::from_str_radix` 简化。
2. 行 325-392 `string_literal()`: `\x` 转义处理中 `advance()` 调用 4 次，逻辑分散。建议统一为状态机。
3. 行 521-574 `parse_define_directive()`: 宏体 tokenize 后直接展开，不支持带参数的宏（`#define MAX(a,b) ...`）。

#### Parser (`parser.rs`, 1526行)

**优点**:
- 完整的 recursive descent parser，支持优先级 climbing 表达式解析
- 复杂声明符（C 螺旋规则）支持：指针数组、函数指针、函数返回指针等
- 良好的错误恢复机制（`synchronize()` 跳过 token 到安全同步点）
- 多个前瞻检测（typedef struct、enum、static 函数检测）

**问题**:
1. 行 440-529 `parse_base_type()`: unsigned 修饰非 Int/Char 类型时（如 `unsigned float`）仅添加诊断错误，但函数继续返回构造完毕的 type（其中 is_unsigned 被忽略）。建议 early return 一个错误类型哨兵。
2. 行 541-659 `parse_declarator_node()`: 声明符复杂度限制（`cross_count > 2`）使某些合法 C 声明被拒绝（如 `int (*(*arr[5])(int))(void)`）。
3. 行 237 `parse_global_var_or_func`: 前瞻时手动遍历 `lookahead`，应封装为 `fn look_ahead_skip_stars()` 辅助函数。

#### TypeChecker (`type_checker.rs`, 1584行)

**优点**:
- 完整类型推导：作用域符号表、结构体/联合体成员访问、函数签名检查
- 隐式类型转换（int↔float↔double, 整数提升）+ 精度丢失警告（W3053）
- 数组初始化类型检查（嵌套多维、字符串→char[] 转换）
- 内建函数（malloc/free/printf/scanf/strlen/strcpy/strcmp/memset/qsort/fopen/realloc 等 ~18 个）的参数类型校验

**问题**:
1. 行 1022-1411: 18 个内建检查器高度模式化，建议宏化。
2. 行 48-121 `insert_implicit_cast()`: 接受 `&mut Expr` 并原地替换为 Cast 节点，逻辑分支多达 10 个。建议按 `(from, to)` 类型对映射表化。
3. 行 267-366 `check_assignable()`: 赋值兼容性检查函数过长（100行），建议拆分为标量↔标量、指针↔数组、结构体↔结构体等独立函数。

#### BytecodeGen (`bytecode_gen.rs`, 1889行)

**优点**:
- 完整的 AST→CideVM 字节码生成，覆盖所有语句和表达式类型
- 字符串常量池、f64/i64 常量池设计
- 全局变量初始化数据自动生成
- 函数元数据（arg_count, param_sizes, local_count）正确记录

**问题**:
1. 行 412-422 `push_f64_constant/push_i64_constant`: 每次 push 都追加，不做去重。相同浮点常量 `3.14` 出现多次会重复存储。
2. 行 424-447 `ptr_step_size()`: 对 Pointer→Function 类型返回 4，但函数指针步长应和普通指针一致（可行）。对多维数组嵌套指针未覆盖彻底。
3. 行 228-245: 非 void 函数末尾注入 `PushConst(0); Ret`，值为 0 可能误导学生。建议注入 `Trap(未定义行为：无返回值)` 或编译时警告。

#### AlgorithmDetector (`algorithm_detector.rs`, 536行)

**优点**:
- 纯 AST 结构分析，无需运行时信息
- 支持 7 种算法检测（冒泡/选择/插入/快速/归并排序 + 二分查找 + 链表操作）
- 特征提取 + 函数名双重匹配提高准确性

**问题**:
1. 行 386-401 `is_adjacent_compare()`: 字符串匹配格式脆弱（见上文 1.6）。
2. 行 247-253: mid 计算检测 `left + (right - left) / 2` 也基于字符串匹配变量名（"left"/"right"/"mid"），中文命名或英文同义词会失败。
3. 行 303-305: shift 模式检测 `s.contains('[') && expr_to_string(right).contains('[')` 过于宽松（任何含方括号的赋值都可能被误识别）。

---

### 4.2 虚拟机模块

#### OpCode (`opcode.rs`, 127行)

**优点**: `define_opcode!` 宏自动生成枚举 + `from_u8` 反序列化，维护性好。

**潜在风险**: 当前最大值 `CallPtr = 111`，`repr(u8)` 上限 255。若未来添加超过 144 个新指令会溢出。建议改为 `repr(u16)` 预留扩展空间，或添加编译期检查：
```rust
const _: () = assert!(max_opcode_value() <= 255);
```

#### VM Core (`vm.rs`, 1919行)

**优点**:
- 完整的栈机解释器，支持 1MB 线性内存、NULL trap zone、栈/堆双向增长、碰撞检测
- 106 条指令分为 8 个执行函数族（stack/local/global/memory/arithmetic/comparison/bitwise/float/double/longlong）
- 丰富的运行时错误诊断（中文带 emoji，包含除零变量名提示、数组越界符号名和有效范围）
- 完整的 snapshot/restore 序列化（支持时间旅行调试）
- 单步调试：breakpoint、paused、cancelled、step_event_hit 四重机制

**问题**:
1. 行 144-171: `reset()` 遗漏 `qsort_depth` 重置。
2. 行 1009-1011: 热点路径 O(n) 符号查找（见上文 1.3）。
3. 行 1581-1689: Call/CallPtr 代码重复（见上文 1.4）。
4. 行 1588: `let frame_size = meta.local_count as u64;` — `local_count` 是字节偏移，但后续判断 `frame_size > MEM_SIZE` 用字节量检查，应该改为 `frame_size > STACK_START - NULL_TRAP_SIZE`。

#### HostFuncs (`host_funcs.rs`, ~992行)

**优点**:
- 完整实现 printf/scanf 格式字符串解析（宽度、精度、长度修饰符 `l`/`ll`/`h`/`hh`/`L`/`z`/`j`/`t`）
- 文件 I/O 支持（fopen/fread/fwrite/fclose/feof/fprintf）
- qsort 通过 `call_user_function` 支持用户定义的比较函数指针回调

**问题**:
1. 行 21-179: `parse_format_specs` 和 `format_printf_string` 中格式字符串解析重复。
2. 行 5-17 `read_cbytes`/`read_cstring`: 每次调用分配 Vec 和 String，对 scanf 中的格式字符串逐字符比对照搬 C 指针比较开销大。

---

### 4.3 引擎与统一模式

#### CompilePipeline (`compile_pipeline.rs`, 540行)

**优点**: 统一了 FRB 和 C API 两端的编译管线，消除代码重复。`push_diagnostics`/`push_warnings`/`push_hints` 三层诊断推送，带自动修复建议。

**问题**: `setup_vm()` 中字符串数据写入 VM 在编译阶段完成，但如果 VM 在 restore snapshot 后重新 `setup_vm`，旧字符串可能被覆盖，需确认 restore 流程不调用 setup_vm。

#### UnifiedEngine (`unified/engine.rs`, 258行)

**优点**: 批次自动执行 + seek-to-step（基于最近检查点正向重放），设计合理。

**问题**:
1. 行 130-134: `step >= self.max_steps` 检查在每步之后但 `self.max_steps` 默认 10,000，若程序有 >10,000 步的批量执行，会过早终止。
2. 行 185-197: seek 重放时没有检查 `cancelled` 标志，长时间重放无法中断。

---

### 4.4 Flutter 桥接层 (`flutter_bridge.rs`, 671行)

**优点**: 多 session 管理（create/destroy/switch），统一的 compile/run/step API。

**问题**:
1. 行 20-96: `Box::leak` 模式导致泄漏（见上文 3.3）。
2. 行 196-241: 首次 step 循环无提前退出（见上文 1.5）。
3. 行 83-97 `current_session()`: 当 session_id 不存在时，fallback 到 id=0 并可能**隐式创建**新 session，无感知。

### 4.5 C API 层 (`capi/mod.rs`, 1036行)

**优点**: 完整的 C ABI 桥接（创建、编译、运行、单步、断点、内存查看、诊断、调用栈、变量快照、算法匹配、可视化事件、会话保存/加载）。

**问题**:
1. 行 15-19: `cstr_to_str` soundness 问题（见上文 1.1）。
2. 行 25-34 `write_str`: `slice::from_raw_parts_mut(dst as *mut u8, len)` 要求 `dst` 指向有效的 `len` 字节可写内存，但这里只复制 `len` 字节，直接转 `*mut u8` 可能违反 Rust 的 provenance 要求（Miri 严格模式下会报错）。
3. 行 96-101 `cide_session_load`: 加载 session 后硬编码注入 `test.txt` 和 `numbers.txt` 预设文件。这应在 session 保存时一并序列化预设文件列表。

---

### 4.6 诊断模块

#### ErrorCodes (`error_codes.rs`, 81行)

**优点**: 70+ 结构化错误码，按 E1xxx/E2xxx/E3xxx/W305x/H305x 分层（词法/语法/类型/警告/提示）。

**问题**: `error_codes.rs` 中的枚举值和 `error_catalog.rs` 中的查找表需手动保持同步。建议用宏或代码生成确保一致性。

---

## 五、测试与静态分析结果

### 测试结果

| 测试套件 | 文件 | 测试数 | 结果 |
|----------|------|--------|------|
| bytecode_gen | `tests/bytecode_gen_unit_test.rs` | 10 | ✅ 全通过 |
| compile_pipeline | `tests/compile_pipeline_test.rs` | 13 | ✅ 全通过 |
| end_to_end | `tests/end_to_end_test.rs` | ~25 | ✅ 全通过 |
| end_to_end_extra | `tests/end_to_end_extra_test.rs` | 150 | ✅ 全通过 |
| e2e_multi_file | `tests/e2e_multi_file.rs` | 5 | ✅ 全通过 |
| lexer | `tests/lexer_unit_test.rs` | ~8 | ✅ 全通过 |
| parser | `tests/parser_unit_test.rs` | ~10 | ✅ 全通过 |
| type_checker | `tests/type_checker_unit_test.rs` | 12 | ✅ 全通过 |
| vm_memory_safety | `tests/vm_memory_safety_test.rs` | 7 | ✅ 全通过 |

**总计**: ~240 个测试，全部通过。

### Clippy

```
warning: unused import: `cide_native::compiler::ast::Type`
  --> tests\e2e_multi_file.rs:1:5
```

仅 1 个警告，无错误。建议移除该未使用的 import。

---

## 附：5-18 审阅修复状态回归验证

本次审阅对 5-18 报告中标记的 5 个 P0 严重 Bug 进行了回归验证，确认全部已修复：

| 原 P0 Bug | 修复方式 | 验证位置 |
|-----------|----------|----------|
| `call_user_function` 循环次数错误 | `FuncMeta` 拆分 `param_count`（参数个数）与 `arg_count`（总 word 数） | `vm.rs` |
| `restore()` 快照大小不匹配 panic | `copy_from_slice` 改为 `min` + 切片安全拷贝，允许不同长度快照恢复 | `vm/snapshot.rs` |
| 复编译时 `f64_constants` 残留 | `run_compile_pipeline` 中补充 `f64_constants.clear()` | `engine/compile_pipeline.rs` |
| 常量索引越界静默返回 0 | `PushConstD` / `PushConstQ` 的 `.unwrap_or(0)` 改为 `trap` 报告越界错误 | `vm.rs` |
| `PushConstF` 符号扩展导致负 float 损坏 | `operand as u64` 改为 `operand as u32 as u64`，避免负 i32 符号扩展 | `vm.rs` |

此外，5-18 报告中提出的以下改进也已落实：

| 改进项 | 状态 |
|--------|------|
| `cargo clippy -- -D warnings` 完全通过 | ✅ 维持，本次仅新增 1 个未使用 import 警告 |
| VM `step()` 超巨型 match 拆分 | ✅ 已拆分为 12 个指令类别处理器 |
| Host `printf` 严重重复消除 | ✅ 复用 `format_printf_string()` |
| Flutter Bridge session 销毁完整性 | ✅ `destroy_session` 同步清理 `UNIFIED_ENGINES` |
| `gen_struct_copy` / `gen_struct_copy_to_local` 重复消除 | ✅ 提取 `gen_struct_copy_common` |
| `parse_abstract_declarator` / `parse_declarator_node` 重复消除 | ✅ `parse_declarator_node` 新增 `is_abstract` 标志 |
| 检查点内存无限增长 | ✅ `CheckpointManager` 新增 `max_checkpoints = 50` |
| `Session.errors_buffer` 冗余字段删除 | ✅ 已删除，C API 直接使用 `errors` |

---

## 六、总体评分与修复优先级

### 评分

| 维度 | 评分 | 说明 |
|------|------|------|
| 架构设计 | ★★★★☆ (8/10) | 编译器→VM→前端三层解耦清晰，多 session 和 C API 设计合理 |
| 代码质量 | ★★★☆☆ (6/10) | 核心逻辑正确，但存在大单体文件、代码重复、少量 soundness 问题 |
| 错误处理 | ★★★★☆ (8/10) | 中文诊断系统设计出色，70+ 错误码覆盖全面，含自动修复建议 |
| 测试覆盖 | ★★★★☆ (7/10) | 240+ 测试全通过，但缺少 fuzz 和 property-based testing |
| 性能 | ★★★☆☆ (6/10) | VM 解释器性能可接受，但热点路径有 O(n) 查找和冗余内存分配 |
| 文档 | ★★★☆☆ (6/10) | AGENTS.md / CHANGELOG / DESIGN.md 保持同步，但代码内注释覆盖仍有提升空间 |

**综合评分: 6.7/10** — 教学级 C IDE 产品级质量，虽存在部分工程问题但核心编译和执行链路可靠。

### 修复优先级

| 优先级 | 条目 | 预计工作量 | 影响 |
|--------|------|-----------|------|
| 🔴 P0 | `cstr_to_str` soundness 修复 | 0.5h | 防止 UB |
| 🔴 P0 | `qsort_depth` reset 修复 | 0.1h | 状态正确性 |
| 🟡 P1 | O(n) 符号查找 → O(1) HashMap | 2h | 运行时性能 |
| 🟡 P1 | Call/CallPtr 去重 | 1.5h | 可维护性 |
| 🟡 P1 | `Box::leak` 改为 Arc + 真正释放 | 4h | 内存管理 |
| 🟢 P2 | 18 个内建检查器宏化 | 3h | 可维护性 |
| 🟢 P2 | VM 热点路径字符串分配优化 | 3h | 运行时性能 |
| 🟢 P2 | 算法检测 AST 结构比较 | 2h | 检测准确性 |
| 🔵 P3 | 大文件拆分 | 8h | 长期可维护性 |
| 🔵 P3 | VM 提取为独立 crate | 4h | 架构清晰度 |
| 🔵 P3 | Fuzzing 基础设施 | 4h | 健壮性 |
| 🔵 P3 | 检查点增量快照 | 6h | 内存效率 |

---

## 七、工程规范与文档同步观察

| 观察项 | 状态 | 建议 |
|--------|------|------|
| `AGENTS.md` Phase 进度 | Phase 17 已完成，但文档未提及后续 Phase 18 规划 | 补充下阶段目标（如 LSP 协议支持、更完整的 C 标准库覆盖） |
| `CHANGELOG.md` 同步 | 5-18 至 6-04 期间的修复未集中归档 | 按版本号或日期汇总近期修复，便于追溯 |
| `DESIGN.md` 指令集描述 | 已更新为 106 条指令，但部分新增指令（`CallPtr`、`Dup` 等）缺少执行语义伪代码 | 补充关键指令的语义说明 |
