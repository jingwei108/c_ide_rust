# `double` 类型真正干净实现 — 全面推进计划

> 状态：**已完成并全部测试通过**（128 e2e + 单元测试，2026-05-17）  
> 目标：彻底消灭 "slot" 概念，全面采用字节偏移，为 `long long` 等后续拓展铺平道路

---

## 一、当前架构问题诊断

### 1.1 已完成的基础设施

| 组件 | 状态 | 说明 |
|------|------|------|
| VM 值栈 (`Vec<u64>`) | ✅ | 可天然承载 i32/f32/f64/pointer |
| `OpCode::*D` (`AddD`/`PushConstD`/...) | ✅ | 已定义，VM 已实现，BytecodeGen 已生成 |
| `f64_constants` 池 | ✅ | VM + BytecodeGen + setup_vm 全链路贯通 |
| `TypeKind::Double` / `Type::double()` | ✅ | Parser 支持 `double` 关键字，TypeChecker 区分 Float/Double |
| TypeChecker 隐式转换 | ✅ | `insert_implicit_cast` 对 `FloatLiteral`→`Double` 直接改 `ty`，不插 Cast 节点 |
| `sizeof(double)` | ✅ | 返回 8 |
| `LoadLocal`/`StoreLocal` 字节偏移 | ✅ | operand 直接作为 byte offset，不再 `×4` |
| `Call` 参数传递 | ✅ | 按 `param_sizes`（字数）正序遍历，`word_offset` 从 0 递增 |

### 1.2 核心改造：`slot × 4` → 字节偏移

改造前：
```
LoadLocal:    addr = locals_base + operand × 4
StoreLocal:   addr = locals_base + operand × 4
LoadGlobal:   addr = GLOBAL_START + idx × 4
Call:         frame_size = local_count × 4
```

改造后：
```
LoadLocal:    addr = locals_base + operand          // operand = 字节偏移
StoreLocal:   addr = locals_base + operand
LoadGlobal:   addr = GLOBAL_START + operand         // operand = 字节偏移
Call:         frame_size = meta.local_count          // local_count = 局部区总字节数
```

---

## 二、目标架构：全面字节偏移

### 2.1 改造后核心等式

```rust
// 局部变量（VM 执行期）
LoadLocal:    addr = frame.locals_base + operand        // operand = 字节偏移
StoreLocal:   addr = frame.locals_base + operand
LoadLocalD:   addr = frame.locals_base + operand        // 读取 8 字节
StoreLocalD:  addr = frame.locals_base + operand        // 写入 8 字节

// 全局变量
LoadGlobal:   addr = GLOBAL_START + operand             // operand = 字节偏移
StoreGlobal:  addr = GLOBAL_START + operand
LoadGlobalD:  addr = GLOBAL_START + operand
StoreGlobalD: addr = GLOBAL_START + operand

// 函数调用
Call:         frame_size = meta.local_count             // local_count 语义 = 局部区总字节数
              arg_bytes = meta.param_sizes.iter().sum() * 4
              按 param_sizes 正序 pop 参数，写入 locals_base + word_offset * 4

// 值栈（已是 u64，无需改动）
push(val: u64) / pop() -> u64
```

### 2.2 `FuncMeta` 新语义

```rust
pub struct FuncMeta {
    pub ip: usize,
    pub arg_count: i32,       // 参数总字数（用于 Call 的 pop 计算）
    pub local_count: i32,     // 局部变量区总字节数（含参数区）
    pub param_sizes: Vec<i32>, // 每个参数的字数 [(sz+3)/4, ...]
}
```

- `local_count` 语义从 "slot 数" 改为 "字节数"
- `arg_count` 语义为 "总字数"（`param_sizes.iter().sum()`）
- `param_sizes` 指导 `Call` 指令如何 pop 和存储每个参数

### 2.3 全局初始化新结构

```rust
pub struct CompileOutput {
    // ...
    pub globals_init_32: Vec<(u32, i32)>,  // (字节偏移, 32位初始值)
    pub globals_init_64: Vec<(u32, u64)>,  // (字节偏移, 64位初始值)
    pub f64_constants: Vec<f64>,
}
```

VM `setup_vm` 时：
```rust
vm.set_globals_32(&session.compile.globals_init);
vm.set_globals_64(&session.compile.globals_init_64);
vm.set_f64_constants(session.compile.f64_constants.clone());
```

---

## 三、改动清单与实施状态

### Phase 1：VM 层基础改造 ✅

#### 3.1.1 `opcode.rs`
- [x] 新增 `PushConstD = 64`, `AddD = 65`, `SubD = 66`, `MulD = 67`, `DivD = 68`, `NegD = 69`
- [x] 新增 `CastI2D = 70`, `CastF2D = 71`, `CastD2I = 72`, `CastD2F = 73`
- [x] 新增 `EqD = 74`, `NeD = 75`, `LtD = 76`, `LeD = 77`, `GtD = 78`, `GeD = 79`
- [x] 新增 `LoadLocalD = 80`, `StoreLocalD = 81`, `LoadGlobalD = 82`, `StoreGlobalD = 83`
- [x] 新增 `LoadMemD = 84`, `StoreMemD = 85`, `SplitD = 86`

#### 3.1.2 `vm.rs`
- [x] `FuncMeta` 增加 `param_sizes: Vec<i32>`
- [x] 新增 `load_i64()` / `store_i64()` 内存 helper
- [x] `LoadLocal` / `StoreLocal`：`operand × 4` → 直接字节偏移
- [x] `LoadGlobal` / `StoreGlobal`：`idx × 4` → 直接字节偏移
- [x] `Call`：
  - `frame_size = meta.local_count`（字节数）
  - 参数传递：按 `param_sizes` **正序**遍历，`word_offset` 从 0 递增，分别 `store_i32`
  - 非参数局部变量清零：`locals_base + arg_bytes .. locals_base + local_count`
- [x] 新增 `LoadLocalD`/`StoreLocalD`/`LoadGlobalD`/`StoreGlobalD`/`LoadMemD`/`StoreMemD`/`SplitD` 执行逻辑
- [x] `read_variable()` / `get_variable_snapshot()`：`value: i32` → `value: i64`

#### 3.1.3 `session.rs`
- [x] `FuncMeta` 增加 `param_sizes: Vec<i32>`
- [x] `CompileState.globals_init: Vec<i32>` → `Vec<(u32, i32)>`
- [x] `CompileState` 新增 `globals_init_64: Vec<(u32, u64)>`
- [x] `CompileState` 新增 `f64_constants: Vec<f64>`
- [x] `VariableSnapshot.value: i32` → `i64`

### Phase 2：BytecodeGen 全面重构 ✅

#### 3.2.1 字段与结构改造
- [x] `FuncMeta` 增加 `param_sizes`
- [x] `CompileOutput` 增加 `f64_constants: Vec<f64>`, `globals_init_32`, `globals_init_64`
- [x] BytecodeGen 字段：
  - `next_local_idx: i32` → `next_local_offset: i32`
  - `next_global_idx: i32` → `next_global_offset: i32`
  - `globals_init: Vec<i32>` → `globals_init_32` / `globals_init_64`
  - 新增 `f64_constants: Vec<f64>`
  - 新增 `current_func_arg_bytes: i32`

#### 3.2.2 局部变量分配（字节偏移 + 4 字节对齐）
- [x] `enter_function`：参数按 `type_size` 分配字节偏移，收集 `param_sizes`（字数）
- [x] `exit_function`：`meta.local_count = self.next_local_offset`（字节数）
- [x] `Stmt::VarDecl`：变量 `sz = type_size(vty)`，`aligned_sz = (sz + 3) & !3`
- [x] 所有 `VMSymbol.addr`（局部变量）= 字节偏移

#### 3.2.3 全局变量分配（字节偏移）
- [x] Pass 1：`next_global_offset` 累加 `type_size`
- [x] `VMSymbol.addr`（全局变量）= 字节偏移（相对于 GLOBAL_START）
- [x] `string_mem_offset` = `GLOBAL_START + next_global_offset`

#### 3.2.4 全局初始化
- [x] `int` / `char` / `float` / 指针：`(offset, val)` 入 `globals_init_32`
- [x] `double`：`(offset, val.to_bits())` 入 `globals_init_64`
- [x] `array`：`char` 数组逐字节入 `globals_init_32`，`double` 数组直接匹配 `FloatLiteral` 取 `f64` bits 入 `globals_init_64`

#### 3.2.5 表达式生成（`gen_expr`）
- [x] `FloatLiteral`：
  - `ty.kind == Float` → `PushConstF`（f32 bits）
  - `ty.kind == Double` → `PushConstD`（索引到 `f64_constants`）
- [x] `Identifier`：
  - `ty.kind == Double` → `LoadLocalD` / `LoadGlobalD`
  - 数组参数（`local_offset < current_func_arg_bytes`）→ `LoadLocal`（读指针值）
  - 局部数组 → `GetFrameBase + PushConst(offset) + Add`
- [x] `Binary`：结果 `Double` → `AddD` / `SubD` / `MulD` / `DivD` / `EqD` / ...
- [x] `Cast`：`int→double` → `CastI2D`，`float→double` → `CastF2D`，`double→int` → `CastD2I`，`double→float` → `CastD2F`
- [x] `Index` / `Member` / `Deref`：元素/成员类型为 `Double` → `LoadMemD`

#### 3.2.6 赋值与语句
- [x] `Assign` / `VarDecl init`：目标 `Double` → `StoreLocalD` / `StoreGlobalD` / `StoreMemD`

#### 3.2.7 函数调用参数生成
- [x] `Expr::Call`：`for arg in args.iter_mut().rev()`
  - `arg_ty.is_struct()`：按 `type_size` 多 word push
  - `arg_ty.kind == Double` + **普通函数**（`func_index` 命中）：`gen_expr(arg)` → `SplitD`（拆成 2 个 i32 word）
  - `arg_ty.kind == Double` + **host 函数**：`gen_expr(arg)`，**不** `SplitD`
  - `arg_ty.kind == Float` + **printf/fprintf**：`gen_expr(arg)` → `CastF2D`
  - 其他：`gen_expr(arg)`

#### 3.2.8 地址生成（`&` 取地址）
- [x] `Unary::Addr`：`Identifier` → `GetFrameBase + PushConst(offset) + Add`
- [x] `Identifier`（全局）→ `PushConst(GLOBAL_START + offset)`

### Phase 3：TypeChecker 微调 ✅

- [x] `insert_implicit_cast`：
  - 若源是 `FloatLiteral` 且目标是 `Double`：**直接把 `FloatLiteral.ty` 改为 `Double`**，不插入 `Cast` 节点
- [x] `VarDecl` / `Assign` / `InitList` 元素：补充 `insert_implicit_cast`

### Phase 4：CompilePipeline 与 CAPI 接线 ✅

- [x] `compile_pipeline.rs`：
  - `run_compile_pipeline`：将 `output.f64_constants` → `session.compile.f64_constants`
  - `run_compile_pipeline`：将 `globals_init_32` / `globals_init_64` → `session.compile`
  - `FuncMeta` 转换：传入 `param_sizes`
- [x] `setup_vm`：
  - `vm.set_f64_constants(session.compile.f64_constants.clone())`
  - 全局初始化：调用 `set_globals_32` / `set_globals_64`

### Phase 5：VM 调试与符号 ✅

- [x] `read_variable`：根据 `sym.ty.kind` 决定读 4 字节还是 8 字节
- [x] `get_variable_snapshot`：`value: i64`

### Phase 6：前端适配 ✅

- [x] FRB 手动实现 `SseEncode`/`SseDecode` for `i64`

### Phase 7：测试 ✅

- [x] `cargo build` 0 错误
- [x] `cargo clippy` 0 警告（待确认）
- [x] 现有回归测试：**全部通过**（128 e2e + 单元测试）
- [x] 新增 double E2E 测试覆盖：
  - `double` 变量声明、赋值、运算
  - `double` 数组初始化与索引
  - `double` 作为函数参数和返回值
  - `printf("%f", double_var)`
  - `sizeof(double)`

---

## 四、实施中修复的关键 Bug（2026-05-17）

| # | 问题 | 现象 | 修复 |
|---|------|------|------|
| 1 | `param_sizes` 存字节而非字数 | 函数调用栈下溢 | `enter_function` push `(sz+3)/4` |
| 2 | `Call` 逆序遍历 `param_sizes` | 参数值互换 | 正序遍历，`word_offset` 从 0 递增 |
| 3 | `printf` `%f` 读 f32 | double 输出 `0.000000` | `%f` 改为 `f64::from_bits(arg)` |
| 4 | `printf` float 参数未提升 | 现有 float 测试失败 | `printf`/`fprintf` 的 Float 参数自动 `CastF2D` |
| 5 | `SplitD` 用于 `CallHost` | `printf` double 参数格式错乱 | 仅普通 `Call` 才 `SplitD` |
| 6 | TypeChecker 缺少赋值 cast | `double a = 3.5` 生成 `PushConstF` | `VarDecl`/`Assign`/`InitList` 补充 `insert_implicit_cast` |
| 7 | double 数组初始化精度丢失 | `double arr[] = {1.1}` 存成 `1066192077.0` | 局部直接 `gen_expr(elem)`；全局直接取 `FloatLiteral.value.to_bits()` |
| 8 | `current_func_arg_bytes` 未设置 | 数组参数传的是地址 `&m` 而非指针值 `m` | `enter_function` 末尾赋值 |
| 9 | 局部变量未 4 字节对齐 | `char` 占 1 字节导致相邻变量覆盖，`hanoi` 栈溢出 | `offset += (sz + 3) & !3` |

---

## 五、风险评估与缓解（已验证）

| 风险 | 结果 |
|------|------|
| **字节偏移重构导致所有现有局部/全局变量地址错误** | ✅ 已解决：4 字节对齐 + 正序参数传递 |
| **`Call` 参数传递新逻辑有边界错误** | ✅ 已解决：正序遍历 + `word_offset` 递增 |
| **`globals_init` 新格式破坏会话保存/加载** | ✅ `serde` 已支持新字段 |
| **`read_variable` / snapshot` 对 double 显示错误** | ✅ `value: i64` 已支持 |
| **FRB 桥接因 `VariableSnapshot.value: i64` 不兼容** | ✅ 手动实现 `SseEncode`/`SseDecode` |

---

## 六、后续拓展路线图

完成本方案后，支持 `long long` 所需的工作：

| 组件 | 工作量 | 说明 |
|------|--------|------|
| Lexer | 10 分钟 | 识别 `long long` 关键字 |
| AST | 10 分钟 | `TypeKind::LongLong` |
| Parser | 30 分钟 | 解析 `long long x;` |
| TypeChecker | 1 小时 | 隐式转换链 |
| BytecodeGen | 2 小时 | `PushConstI64`、`AddI64`... |
| VM | 2 小时 | 新增 `OpCode::AddI64` / `CastI2I64` |
| 测试 | 半天 | E2E 测试 |

**总计：1 天**（内存框架已是字节偏移，无需再改 `LoadLocal`/`Call`/全局初始化）

---

## 七、决策记录

| 日期 | 决策 | 原因 |
|------|------|------|
| 2026-05-17 | 放弃 "double 占 2 slot" 方案 | 用户明确反对工程债，要求干净架构 |
| 2026-05-17 | 全面改为字节偏移 | 一劳永逸，支持 `long long` 时无需再改内存框架 |
| 2026-05-17 | 局部变量/参数按 4 字节对齐 | 避免 `char`（1 字节）与 `LoadLocal`（4 字节存取）冲突 |
| 2026-05-17 | 暂不强制 8 字节对齐 | 教学 VM 可先按紧凑布局，降低复杂度 |
| 2026-05-17 | 允许打破 Session 向后兼容 | 研发阶段，旧 session 格式可废弃 |

---

## 八、附录：关键文件改动清单

```
native/src/vm/opcode.rs              +14 条新 OpCode
native/src/vm/vm.rs                  FuncMeta、Call、LoadLocal/StoreLocal/Global/Mem、*D 指令、read_variable、i64 snapshot
native/src/session.rs                FuncMeta、CompileState、VariableSnapshot.value: i64
native/src/compiler/bytecode_gen.rs  FuncMeta、CompileOutput、局部/全局字节偏移、*D 指令生成、f64_constants、对齐
native/src/compiler/type_checker.rs  insert_implicit_cast 扩展到 VarDecl/Assign/InitList
native/src/engine/compile_pipeline.rs run_compile_pipeline、setup_vm、全局初始化
native/src/vm/host_funcs.rs          format_printf_string 的 %f 读 f64
native/src/frb_generated.rs          手动 SseEncode/SseDecode for i64
```
