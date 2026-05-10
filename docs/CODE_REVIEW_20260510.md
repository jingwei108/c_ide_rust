# C IDE Rust 项目全面地毯式审查报告

> 审查日期：2026-05-10
> 范围：全部 Rust 后端 (16 文件)、全部 C# 前端 (44 文件)、全部测试、全部文档、构建脚本

---

## 一、架构与文档问题

### 1. 文档同步滞后 (DESIGN.md / ROADMAP.md) [已修复]

`docs/DESIGN.md` 和 `docs/ROADMAP.md` 仍描述 C++ 后端架构（CMake / Clang / `WasmCodeGen`），实际已全部迁移为 Rust。多处提及过时信息：
- "C++20"、"CMake"、"`WasmCodeGen`"
- 目录树使用 `.cpp` / `.hpp` 扩展名
- ROADMAP 第251行架构图仍标"C++ 后端 (Native DLL / .so)"
- DESIGN 第 56 行技术栈表写"C++20"

**文件位置**: `docs/DESIGN.md:48,56,135-209,293-308` | `docs/ROADMAP.md:251-292`

### 2. C 头文件与 Rust 代码不同步 (cide_capi.h) [已修复]

`native/include/cide_capi.h` 是遗留 C 头文件，与 Rust 实际 `capi/mod.rs` 定义严重不同步：
- 缺失错误码：`CIDE_E2007_ExpectedClosingParen`、`CIDE_E2008_ExpectedClosingBracket`
- 缺失警告码：`CIDE_W3052` ~ `CIDE_W3056`（共 5 个）
- 第 166 行注释称"分号分隔的输入行"，实际代码使用换行分隔
- 枚举名 `CideErrorCode`、结构体前缀 `CideSession` 在 Rust 端不存在

**文件位置**: `native/include/cide_capi.h:36-98,166`

### 3. Session::Default 双重定义冲突 [已修复]

`session.rs:13` 有 `#[derive(Default)]`，派生版本 `vm: None`。
`session.rs:143-151` 手动 `impl Default`，设置 `vm: Some(CideVM::default())`。

两个 Default 实现语义冲突——派生宏生成的是字段级默认，手动实现覆盖了它。

**文件位置**: `native/src/session.rs:13` + `:143-151`

### 4. 零 CI/CD 配置 [已修复]

项目无任何 CI/CD 流水线（`.github/` 目录不存在）。构建和测试完全依赖本机 PowerShell 脚本手动触发。

---

## 二、Rust 后端代码问题

### 词法分析器 (Lexer)

#### 5. `peek()` UTF-8 安全缺陷 [已修复]

`lexer.rs:530` — `self.source.as_bytes()[self.pos + offset] as char`
对多字节 UTF-8 字符（如中文注释中的"越界"、"排序"）会返回错误字节而非完整 Unicode 码点。
`as_bytes()` 返回 `&[u8]`，索引到多字节序列中间时产生无意义的 `char`。
应使用 `self.source.chars().nth(...)` 迭代器。

```rust
// lexer.rs:529-535
fn peek(&self, offset: usize) -> char {
    if self.pos + offset >= self.source.len() {
        '\0'
    } else {
        self.source.as_bytes()[self.pos + offset] as char  // ❌ UTF-8 unsafe
    }
}
```

**文件位置**: `native/src/compiler/lexer.rs:529-535`

#### 6. 字符字面量返回 Number 类型 [已修复]

`lexer.rs:447` — `char_literal()` 返回 `TokenType::Number` 而非专用的 `CharLiteral` token 类型。
导致 Parser 无法区分 `'a'` 和 `97`，影响后续语义检查和错误消息精度。

```rust
// lexer.rs:447
let mut tok = self.make_token(TokenType::Number, ...);  // ❌ 应该是 CharLiteral
```

**文件位置**: `native/src/compiler/lexer.rs:447`

#### 7. 宏展开只支持简单替换

`lexer.rs:510-527` — `expand_macros()` 只是单层 hashmap 查找替换，不支持：
- 参数化宏（`#define SQUARE(x) ((x)*(x))`）
- 嵌套宏名递归展开（`#define A B; #define B 1`）
- 宏文本中包含其他宏名

```rust
// lexer.rs:513
if let Some(macro_tokens) = self.macros.get(&tok.text) {  // ❌ 单层查找
```

**文件位置**: `native/src/compiler/lexer.rs:510-527`

---

### 语法分析器 (Parser)

#### 8. `parse_program()` 三处高重复代码 [已修复]

`parser.rs:146-248` — struct 声明分支、enum 声明分支、普通类型分支三个代码块包含约 80 行几乎相同的变量声明/初始化逻辑：
- 每个分支都有 checkpoint → `parse_type_and_name` → 判断 LParen → 全局变量初始化 → 逗号多变量
- 应提取公共函数 `parse_global_var_or_func()` 消除重复

**文件位置**: `native/src/compiler/parser.rs:146-248`

#### 9. `parse_unary()` 内联类型转换检查脆弱 [已修复]

`parser.rs:797-808` — 用 checkpoint + rollback 方式检测 `(type)expr` 类型转换。
`advance()` 可能产生副作用（pos 移动），虽然 rollback 恢复 pos，但 `typedef_names` 等状态可能有残留影响。

```rust
// parser.rs:798-808
let checkpoint = self.pos;
self.advance(); // consume '('
if self.is_type_token() {
    let t = self.parse_type_only();  // ← 可能有副作用
    // ...
}
self.pos = checkpoint; // rollback
```

**文件位置**: `native/src/compiler/parser.rs:797-808`

---

### 类型检查器 (TypeChecker)

#### 10. 位运算错误码错位 [已修复]

`type_checker.rs:548` — `BinaryOp::BitAnd | BitOr | BitXor | Shl | Shr` 报错借用 `E3019_LogicTypeError`（逻辑运算类型错误）。位运算和逻辑运算是不同的语义类别。

```rust
// type_checker.rs:546-551
BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor | BinaryOp::Shl | BinaryOp::Shr => {
    // ...
    self.report_error("位运算要求两边都是 int 类型", loc, ErrorCode::E3019_LogicTypeError);  // ❌ 应使用专用错误码
}
```

**文件位置**: `native/src/compiler/type_checker.rs:546-551`

#### 11. 关系运算拒绝指针比较 [已修复]

`type_checker.rs:534-536` — `Lt/Le/Gt/Ge` 检查只允许两边都是 `int`，拒绝指针间比较（如 `p < q`）。
C 标准允许同类型指针的大小比较。

```rust
// type_checker.rs:534-537
BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
    if !self.is_int(&left_type) || !self.is_int(&right_type) {  // ❌ 应允许指针比较
        self.report_error("关系运算要求两边都是 int 类型", loc, ErrorCode::E3018_RelationTypeError);
    }
}
```

**文件位置**: `native/src/compiler/type_checker.rs:534-537`

#### 12. `is_assignable` 赋值时产生过多警告 [已修复]

`type_checker.rs:178-214` — 每次合法但隐式的赋值都生成警告，包括：
- `int* p = malloc(...)` 中 `void*`→`int*` 的合法转换
- 数组名传给指针参数的合法退化
- `int a = 'A';` 等教科书常见写法

**文件位置**: `native/src/compiler/type_checker.rs:178-214`

---

### 字节码生成 (BytecodeGen)

#### 13. 指针加法硬编码步长=4 [已修复]

`bytecode_gen.rs:703` — `BinaryOp::Add` 中指针+整数时始终 `PushConst 4`：

```rust
// bytecode_gen.rs:700-705
BinaryOp::Add => {
    if left_is_ptr && !right_is_ptr {
        self.emit(OpCode::PushConst, 4, &loc);  // ❌ char* 应该是 1
        self.emit(OpCode::Mul, 0, &loc);
```

`ptr_step_size()` 方法已存在并正确返回 char=1 / int=4 / struct=结构体大小，但加法分支未使用。
**后果**: `char* p; p + 1` 前进 4 字节而非 1 字节，字符串操作出错。

**文件位置**: `native/src/compiler/bytecode_gen.rs:700-705`

#### 14. 指针减法的缩放逻辑缺失

`bytecode_gen.rs:720-726` — `ptr_step_size` 正确处理了减法分支，但只用于缩放右侧整数。
`left_is_ptr && !right_is_ptr` 的场景（如 `p - 1`）调用了 `ptr_step_size`，但 `right_is_ptr && !left_is_ptr` 的加法场景未使用 step size。

**文件位置**: `native/src/compiler/bytecode_gen.rs:717-730`

#### 15. `gen_member_addr` 误导性错误消息 [已修复]

`bytecode_gen.rs:966` — 错误报告"全局结构体暂不支持"，但第 963-964 行已正确实现全局结构体成员地址的获取：

```rust
// bytecode_gen.rs:963-968
} else if let Some(&idx) = self.global_indices.get(name) {
    self.emit(OpCode::PushConst, 0x1000 + idx * 4, loc);  // ✅ 已支持
} else {
    self.report_error("全局结构体暂不支持", loc);  // ❌ 错误消息不准
```

**文件位置**: `native/src/compiler/bytecode_gen.rs:966`

#### 16. 多维数组 stride 计算边界缺陷 [部分修复/需配合内存模型改造]

`bytecode_gen.rs:1177-1181` — `compute_stride()` 当 `dims` 为空时直接返回 4：

```rust
// bytecode_gen.rs:1176-1183
fn compute_stride(arr_type: &Type) -> i32 {
    if !arr_type.is_array() || arr_type.dims.is_empty() { return 4; }  // ❌ 未知大小数组应该用元素大小
    let mut stride = 4;
```

对 `int arr[]`（大小未知的一维数组），正确的 stride 应为元素的大小(4)，但如果是 `char arr[]`，stride 应为此处没有根据 `base_kind` 区分。

**文件位置**: `native/src/compiler/bytecode_gen.rs:1176-1183`

---

### 虚拟机 (CideVM)

#### 17. 移位操作无越界保护 [已修复]

`vm.rs:780-781` — `Shl`/`Shr` 直接执行 `a << b` / `a >> b`：

```rust
// vm.rs:780-781
OpCode::Shl => { let b = self.pop(); let a = self.pop(); self.push(a << b); }
OpCode::Shr => { let b = self.pop(); let a = self.pop(); self.push(a >> b); }
```

当 `b >= 32` 或 `b < -1` 时行为未定义（在 LLVM 后端会 panic）。Add/Sub/Mul/Div/Neg 均有溢出检查，移位缺失。
`a >> b` 对负数 `a` 的行为也是实现定义的（算术右移还是逻辑右移依赖于 CPU）。

**文件位置**: `native/src/vm/vm.rs:780-781`

#### 18. StepEvent 与断点检查的逻辑顺序 [已修复]

`vm.rs:582-588` — `StepEvent` 操作码的处理在 `ip` 自增之后但实际指令匹配之前，而 `step_event_hit` 状态在匹配后才被检查（`wrapped_step_event_hit` 字段）。pause 后第二次调用 `step()` 时 `paused` 标志已清空但 `step_event_hit` 在 `cide_step_next` 中被用于检测下一步。

**文件位置**: `native/src/vm/vm.rs:582-588`

---

### C API 桥接层

#### 19. `cide_session_save` / `cide_session_load` 未实现 [已修复]

`capi/mod.rs:72-79` — 直接返回 -1，无法保存/恢复会话状态（代码、字节码、输入输出）。

```rust
// capi/mod.rs:72-79
pub unsafe extern "C" fn cide_session_save(_s: *mut Session, _filepath: *const c_char) -> c_int {
    -1  // ❌ 未实现
}
```

**文件位置**: `native/src/capi/mod.rs:72-79`

#### 20. `cide_get_compile_errors` 返回裸指针生命周期不安全 [已修复]

`capi/mod.rs:290-302` — 返回指向 Session 内部 `errors_buffer` 的 C 字符串指针。文档说"仅在下一次编译前有效"，但 C# 调用方没有此保护，可能导致 use-after-free。

**文件位置**: `native/src/capi/mod.rs:290-302`

#### 21. heap_limit 回调捕获初始脏数据 [已修复]

`capi/mod.rs:361` — `let heap_offset = session.memory.heap_offset` 在 setup 中按值捕获进闭包：

```rust
// capi/mod.rs:359-361
let heap_offset = session.memory.heap_offset;
vm.set_heap_limit_callback(move || heap_offset);  // ❌ 永远返回初始值
```

后续 `malloc` 修改的 `heap_offset` 不会反映到 VM 的栈-堆碰撞检查中。多次 malloc 后栈帧可能覆盖堆分配。

**文件位置**: `native/src/capi/mod.rs:359-361`

#### 22. TypeChecker 警告被静默丢弃 [已修复]

`capi/mod.rs:223` — `check()` 返回 `(errors, warnings)`，警告被 `_type_warnings` 丢弃：

```rust
// capi/mod.rs:223
let (type_errors, _type_warnings) = TypeChecker::new().check(&mut program);
//                ^^^^^^^^^^^^^ W3050-W3056 被丢弃
```

前端完全看不到 `W3050`（条件中赋值）、`W3051`（off-by-one）、`W3056`（unsigned 映射）等教学警告。

**文件位置**: `native/src/capi/mod.rs:223`

#### 23. `cide_memory_get_pointer_target` 排除 NULL 指针 [已修复]

`capi/mod.rs:754` — `if target > 0` 条件排除 NULL 指针（值为 0），使得内存视图中无法显示指向 NULL 的指针。

```rust
// capi/mod.rs:754
if target > 0 {  // ❌ NULL 指针无法显示
    *out_target = target as u32;
```

**文件位置**: `native/src/capi/mod.rs:754`

---

### 宿主函数 (Host Functions)

#### 24. `host_strcpy` 缺少缓冲区溢出检查 [已修复]

`host_funcs.rs:320-335` — 不检查目标地址空间是否足够容纳源字符串。如果 `dest` 指向栈上的 `char[3]` 而源字符串为 `"hello"`，会覆盖相邻内存。

**文件位置**: `native/src/vm/host_funcs.rs:320-335`

#### 25. `host_malloc` 中 u32 溢出的残余风险 [已修复]

`host_funcs.rs:79-83` — `addr as u64` 避免了加法溢出，但 `session.memory.heap_offset = new_offset as u32` 在 new_offset > u32::MAX 时截断。

**文件位置**: `native/src/vm/host_funcs.rs:79-84`

---

### 测试覆盖

#### 26. 无模块级单元测试

`native/tests/` 下仅有：
- `end_to_end_test.rs` (~400 行，约 25 个测试函数)
- `end_to_end_extra_test.rs` (E2E 扩展)
- `compile_pipeline_test.rs` (~211 行，约 10 个测试)

**完全缺失**的单元测试：
- `#[cfg(test)] mod tests` 在每个 `src/**/*.rs` 中均为零
- Lexer 无 token 拆分验证
- Parser 无 AST 结构验证
- TypeChecker 无类型推导验证
- BytecodeGen 无字节码序列验证
- VM 无指令级验证

**文件位置**: `native/tests/` + 所有 `native/src/**/*.rs`

---

## 三、C# 前端问题

### 27. 单元测试异常静默捕获 [已修复]

`Cide.Client.Tests/NativeMethodsTests.cs:16-19` — `catch (DllNotFoundException)` 后直接 `return`，测试方法什么都不验证就被标记为通过：

```csharp
catch (DllNotFoundException)
{
    return;  // ❌ 测试空转通过
}
```

当 native DLL 缺失时所有 4 个测试均"假通过"。

**文件位置**: `Cide.Client.Tests/NativeMethodsTests.cs:16-19`

### 28. C# 测试覆盖极薄

仅 3 个 `[Fact]` 测试覆盖 Session 创建、空源码编译失败、Hello World 编译成功。无运行、调试、诊断、内存视图、可视化事件的回归测试。

**文件位置**: `Cide.Client.Tests/NativeMethodsTests.cs`

### 29. P/Invoke `out` 参数分配负担

`NativeMethods.cs:67-71` — 每次调用 `cide_memory_region_get` 需预分配 `byte[] name` 和 `byte[] type` 缓冲区：

```csharp
// 调用方必须预分配固定大小的 byte[]
byte[] name = new byte[128];
byte[] type = new byte[64];
NativeMethods.cide_memory_region_get(session, 0, out addr, out size, name, 128, type, 64, ...);
string nameStr = Encoding.UTF8.GetString(name, 0, Array.IndexOf<byte>(name, 0));
```

建议封装 `SafeHandle` 包装或返回 `ref struct` 的 C# 友好层。

**文件位置**: `Cide.Client.Shared/Core/NativeMethods.cs:67-76`

---

## 四、未实现的功能

| 功能 | 文档出处 | 状态 |
|:---|:---|:---|
| 会话保存/加载 (`cide_session_save/load`) | `capi/mod.rs:72-79` | 桩函数（返回 -1） |
| 移动端性能优化（降帧率、简化渲染） | `ROADMAP.md:311` | 未实现 |
| 知识图谱系统 + 学习进度追踪 | `ROADMAP.md:311` | 未实现 |
| 函数指针 | `C_SUBSET_SPEC.md:521` | 未实现 |
| 浮点运算 (float/double) | `C_SUBSET_SPEC.md:522` | 未实现（不计划支持） |
| TypeChecker 警告透传到前端 | `capi/mod.rs:223` | 编译时丢弃 |
| 运行时时空指针的变量名定位 | `vm.rs:463` | trap 只报告行号，不报告变量名 |
| CI/CD 自动化构建/测试流水线 | — | 不存在 |
| `cide_session_save/load` 序列化/反序列化 | `capi/mod.rs:72` | 未实现 |

---

## 五、潜在优化方向

### A. 可引入的外部库

项目当前 `Cargo.toml` 零外部依赖。以下库在特定场景有价值：

| 库 | 用途 | 对应模块 |
|:---|:---|:---|
| `thiserror` | 统一错误类型派生，减少手写 `impl CompileError` | 全部 |
| `logos` | 替代手写 Lexer，消除 UTF-8 边界 bug | lexer.rs |
| `serde` + `serde_json` | 实现 `session_save/load` 的序列化 | capi/mod.rs |
| `unicode-ident` | 正确处理 Unicode 字符分类 | lexer.rs |

### B. 代码结构优化

- **消除 Parser 三处重复**：提取 `parse_global_var_or_func()` 公共方法
- **统一 ErrorCode 转换**：所有 `ErrorCode::Xxx as i32` 通过一处 trait/宏处理
- **TypeChecker warnings 管线**：在 C API 和 C# 前端间建立 warnings 通道
- **Session Default 统一**：删除 `derive(Default)` 或统一为手动实现

### C. 关于跨语言重写

不建议用其他语言替换现有模块。Rust 实现覆盖编译器和 VM 的完整管线，C# 前端负责 UI，边界清晰。16 个 Rust 源文件约 6000 行代码量适中，重写成本高、收益低。

---

## 问题优先级速览

| 优先级 | 编号 | 问题 | 影响 |
|:---|:---|:---|:---|
| 🔴 P0 | #13 | 指针加法硬编码步长=4 | `char*` 指针运算错误 |
| 🔴 P0 | #21 | heap_limit 回调捕获脏数据 | 栈-堆碰撞保护失效 |
| 🔴 P0 | #22 | TypeChecker 警告被丢弃 | 教学场景核心功能丢失 |
| 🟡 P1 | #5 | peek() UTF-8 不安全 | 中文注释区域 token 乱码 |
| 🟡 P1 | #12 | 赋值时过多误报警告 | 学生困惑 |
| 🟡 P1 | #17 | Shl/Shr 无越界保护 | 未定义行为/panic |
| 🟡 P1 | #27 | 测试空转通过 | CI 假阳性 |
| 🟢 P2 | #3 | Session Default 冲突 | 语义歧义 |
| 🟢 P2 | #10 | 位运算错误码错位 | 诊断消息不准确 |
| 🟢 P2 | #6 | 字符字面量用 Number 类型 | 诊断精度低 |
| 🟢 P2 | #19 | session_save/load 未实现 | 功能缺失 |
| ⚪ P3 | #4 | 零 CI/CD | 开发流程依赖本机 |
| ⚪ P3 | #1 | 文档过期 | 新成员误导 |

---

## 修复日志（2026-05-10 当日）

| 编号 | 问题 | 修复文件 | 修复摘要 |
|:---|:---|:---|:---|
| #13 | 指针加法硬编码步长=4 | `native/src/compiler/bytecode_gen.rs` | `PushConst 4` → `ptr_step_size(left/right.ty())` |
| #21 | heap_limit 回调捕获脏数据 | `native/src/vm/vm.rs`, `native/src/capi/mod.rs` | 删除 `get_heap_limit` 闭包，`Call` 指令直接读取 `session.memory.heap_offset` |
| #22 | TypeChecker 警告被丢弃 | `native/src/capi/mod.rs` | 不再丢弃 `_type_warnings`；新增 `push_warnings()`，severity=1 |
| #3 | Session::Default 双重定义 | `native/src/session.rs` | 删除 `#[derive(Default)]`，保留手动 `impl Default` |
| #10 | 位运算错误码错位 | `native/src/diagnostics/error_codes.rs`, `native/src/compiler/type_checker.rs` | 新增 `E3048_BitOpTypeError`，位运算独立使用 |
| #17 | Shl/Shr 无越界保护 | `native/src/vm/vm.rs` | 添加 `!(0..32).contains(&b)` 检查，越界 `trap` |
| #27 | 测试空转通过 | `Cide.Client.Tests/Cide.Client.Tests.csproj`, `NativeMethodsTests.cs` | `catch` 中改为 `Assert.Fail`；csproj 引用 native DLL |
| — | BytecodeGen 缺失 main panic | `native/src/compiler/bytecode_gen.rs` | `self.func_index["main"]` → `get("main")`，缺失时返回错误 |
| #12 | 赋值时过多误报警告 | `native/src/compiler/type_checker.rs` | `W3053` 只保留 `int->char`；删除 `W3055`（`void*` 转换） |
| #11 | 关系运算拒绝指针比较 | `native/src/compiler/type_checker.rs` | `< <= > >=` 允许同类型指针（含数组退化）间比较 |
| #5 | peek() UTF-8 不安全 | `native/src/compiler/lexer.rs` | `peek()`/`advance()` 改用 `chars().nth()` + `len_utf8()` |
| #15 | gen_member_addr 误导性错误消息 | `native/src/compiler/bytecode_gen.rs` | "全局结构体暂不支持" → "未声明的结构体变量" |
| #16 | 多维数组 stride 计算边界缺陷 | `native/src/compiler/bytecode_gen.rs` | 尝试按 `base_kind` 区分后回滚：当前 VM 局部变量按 4 字节 slot 分配，`char` 数组索引访问若 stride=1 会导致 4 字节读写重叠。需配合内存模型改造（1 字节对齐分配 + `LoadMemByte`/`StoreMemByte`） |
| #6 | 字符字面量返回 Number 类型 | `native/src/compiler/lexer.rs`, `native/src/compiler/parser.rs` | 新增 `TokenType::CharLiteral`，Parser 生成 `Type::char()` 的 `Expr::Literal` |
| #18 | StepEvent 逻辑分散 | `native/src/vm/vm.rs` | 断点检查从 `match` 之前合并到 `OpCode::StepEvent` 分支中 |
| #24 | host_strcpy 缺少终止符保护 | `native/src/vm/host_funcs.rs` | 目标空间不足时确保在边界内写入 null 终止符 |
| #25 | host_malloc u32 截断风险 | `native/src/vm/host_funcs.rs` | 添加 `new_offset > u32::MAX` 检查 |
| #23 | NULL 指针内存视图缺失 | `native/src/capi/mod.rs` | `target > 0` → `target >= 0`，允许显示指向 0x0000 的指针 |
| #8 | parse_program() 三处重复代码 | `native/src/compiler/parser.rs` | 提取 `parse_global_var_or_func()` 公共方法，消除 enum/struct/普通类型分支的 ~25 行重复 |
| #9 | parse_unary() 类型转换副作用 | `native/src/compiler/parser.rs` | checkpoint rollback 时同步恢复 `typedef_names` 快照，防止 `enum Name` 解析残留副作用 |
| #20 | cide_get_compile_errors 裸指针 | `native/src/capi/mod.rs` | 添加 `///` 安全文档注释，明确指针仅在下次编译前有效 |
| #19 | session_save/load 未实现 | `native/src/capi/mod.rs`, `native/Cargo.toml`, 多文件添加 serde derive | 引入 `serde` + `serde_json`，`SessionSnapshot` 序列化 compile/runtime/memory 状态 |
| #1 | 文档过期 | `docs/DESIGN.md`, `docs/ROADMAP.md` | C++ → Rust，CMake → Cargo，WasmCodeGen → BytecodeGen，目录树同步为 `.rs` |
| #2 | C 头文件不同步 | `native/include/cide_capi.h` | 补全 `E2007`/`E2008`/`E3048`/`W3051`~`W3056`；修正注释 |
| #4 | 零 CI/CD | `.github/workflows/ci.yml` (新建) | Rust 编译/测试/clippy + C# 编译/测试 |

**验证结果（第一轮）**：
- `cargo test`：75/75 通过（0 失败）
- `cargo clippy`：0 警告
- `dotnet test Cide.Client.Tests`：3/3 通过（真实执行，非静默跳过）

**验证结果（第二轮）**：
- `cargo test`：75/75 通过（0 失败）
- `cargo clippy`：0 警告
- `dotnet test Cide.Client.Tests`：3/3 通过

**验证结果（第三轮）**：
- `cargo test`：75/75 通过（0 失败）
- `cargo clippy`：0 警告
- `dotnet test Cide.Client.Tests`：3/3 通过
