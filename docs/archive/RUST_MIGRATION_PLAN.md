# Cide Native 层 Rust 迁移计划

> 目标：将 `native/src` 下全部 C++ 代码（~8,000 行）迁移为 Rust，保持 C ABI 兼容，C# 前端零改动。

---

## 1. 现状分析

### 1.1 代码分布

| 模块 | 行数 | 文件 | 职责 |
|------|------|------|------|
| `compiler/` | 3,777 | 9 | Lexer → Parser → TypeChecker → BytecodeGen |
| `diagnostics/` | 1,540 | 3 | AlgorithmMatcher、错误码、诊断格式化 |
| `capi/` | 1,513 | 2 | C FFI 封装层（供 C# P/Invoke） |
| `vm/` | 1,227 | 6 | CideVM 字节码解释器、HostFunctions |
| **合计** | **~8,057** | **20** | |

### 1.2 外部依赖

- **C# 前端**：通过 `cide_capi.h` 中 30+ 个 `extern "C"` 函数调用，输入 C 字符串，输出状态到 `CideSession`。
- **构建系统**：
  - Desktop：`CMake + Ninja/MSVC/Clang` → `cide_native.dll`
  - Android：`CMake + NDK toolchain` → `libcide_native.so`（arm64-v8a / armeabi-v7a）
- **测试**：C++ 可执行文件链接 `cide_native`，通过 C API 做端到端测试。

### 1.3 关键约束

1. **ABI 零破坏**：`cide_capi.h` 中的函数签名、枚举值、结构体内存布局必须完全一致。
2. **Bytecode 格式稳定**：Rust VM 必须能执行 C++ BytecodeGen 生成的旧字节码（迁移期），反之亦然。
3. **Session 序列化兼容**：`cide_session_save/load` 的二进制格式需要兼容（或设计版本化迁移）。
4. **输出路径不变**：`dist/desktop/cide_native.dll`、`Cide.Client.Maui/lib/*/libcide_native.so`。

---

## 2. Rust 项目结构设计

```
native/
├── Cargo.toml                    # workspace root，cdylib target
├── .cargo/
│   └── config.toml               # Android linker 配置
├── cbindgen.toml                 # 生成 cide_capi.h（仅校验，手工维护头文件）
├── build.rs                      # 可选：注入版本号/git hash
└── src/
    ├── lib.rs                    # crate-type = ["cdylib", "staticlib"]
    ├── capi/
    │   ├── mod.rs                # #[no_mangle] extern "C" 入口
    │   ├── session.rs            # CideSession opaque pointer 管理
    │   ├── compile.rs            # cide_compile / cide_compile_all
    │   ├── runtime.rs            # cide_run / cide_step_next
    │   ├── debug_view.rs         # 调用栈、变量、内存、trace、vis events
    │   ├── diagnostics.rs        # 诊断、自动修复、算法匹配查询
    │   └── serialize.rs          # session_save / load
    ├── compiler/
    │   ├── mod.rs
    │   ├── ast.rs                # AST enum + SourceLoc + Type
    │   ├── lexer.rs              # Token / Lexer
    │   ├── parser.rs             # Parser（递归下降 + 零进度保护）
    │   ├── type_checker.rs       # TypeChecker
    │   └── bytecode_gen.rs       # BytecodeGen（AstVisitor 等效物）
    ├── vm/
    │   ├── mod.rs
    │   ├── opcode.rs             # #[repr(u8)] enum OpCode
    │   ├── instruction.rs        # Instruction { op, operand, loc }
    │   ├── vm.rs                 # CideVM 核心（线性内存、栈、执行循环）
    │   └── host_funcs.rs         # malloc/free/printf/scanf/__cide_*
    └── diagnostics/
        ├── mod.rs
        ├── error_codes.rs        # ErrorCode enum（与 C 头文件一致）
        └── algorithm_matcher.rs  # AST 模式识别
```

### 2.1 Cargo.toml

```toml
[package]
name = "cide_native"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "staticlib"]

[dependencies]
# 当前不需要外部库；纯 std 实现。
# 如需序列化优化，后续可引入 byteorder / bincode。

[profile.release]
debug = true          # 生成 PDB/DWARF，便于问题定位
lto = "thin"          # 链接时优化，控制 so/dll 体积
strip = false         # 保留符号，方便 attach 调试

[profile.dev]
opt-level = 1         # 开发时保持一定性能，避免 VM 跑太慢
```

---

## 3. 关键技术决策

### 3.1 AST 表示：从 `std::unique_ptr` + 虚函数到 Rust enum

C++ 当前：
```cpp
struct Expr { virtual ~Expr() = default; };
struct BinaryExpr : Expr { Op op; std::unique_ptr<Expr> left, right; };
// Visitor 模式：class AstVisitor { virtual void VisitBinary(BinaryExpr&) = 0; ... }
```

Rust 方案：
```rust
pub enum Expr {
    Binary { op: BinOp, left: Box<Expr>, right: Box<Expr>, loc: SourceLoc, ty: Type },
    Unary  { op: UnaryOp, operand: Box<Expr>, loc: SourceLoc, ty: Type },
    Literal(i32, SourceLoc),
    StringLiteral(String, SourceLoc),
    Identifier(String, SourceLoc),
    Call { name: String, args: Vec<Expr>, loc: SourceLoc, ty: Type },
    // ... 其余节点
}
```

**理由**：
- 消除 `std::unique_ptr` 的显式内存管理，Rust `Box` 自动处理。
- enum 的 `match` 比虚函数分发更清晰，且编译器可做穷尽检查。
- BytecodeGen 和 TypeChecker 不再用 Visitor trait，而是写递归函数 `gen_expr(&mut self, expr: &Expr)`，代码更扁平。

### 3.2 错误处理：从累积向量到统一 `Diagnostic`

C++ 当前：`LexerError`、`ParseError`、`TypeError` 三个独立结构，在 `capi` 层统一格式化。

Rust 方案：
```rust
#[derive(Clone)]
pub struct Diagnostic {
    pub line: i32,
    pub column: i32,
    pub code: ErrorCode,
    pub severity: Severity,      // Error / Warning / Hint
    pub message: String,
    pub fix_suggestion: String,
    pub structured_fix: Option<StructuredFix>,
}

pub enum StructuredFix {
    Insert { line: i32, column: i32, text: String },
    Replace { start: Pos, end: Pos, text: String },
    Delete { start: Pos, end: Pos },
}
```

**理由**：统一结构减少 C API 层的转换胶水代码，同时保留自动修复的元数据。

### 3.3 FFI 边界：Opaque Pointer + 手工字符串

`CideSession` 在 Rust 侧是完整结构体，通过 `Box::into_raw` / `from_raw` 暴露为不透明指针：

```rust
pub struct Session {
    pub compile: CompileState,
    pub runtime: RuntimeState,
    pub memory: MemoryState,
    pub vm: CideVM,
}

#[no_mangle]
pub extern "C" fn cide_session_create() -> *mut Session {
    Box::into_raw(Box::new(Session::default()))
}

#[no_mangle]
pub unsafe extern "C" fn cide_session_destroy(s: *mut Session) {
    if !s.is_null() { drop(Box::from_raw(s)); }
}
```

**字符串策略**：
- 输入 `const char*` → `unsafe { CStr::from_ptr(src).to_str() }`，失败时返回 UTF-8 解码错误。
- 输出字符串（如 `cide_get_compile_errors`）→ Session 内部持有 `CString` 缓存，返回 `.as_ptr()`，生命周期与 Session 绑定。

### 3.4 VM 线性内存

C++ 当前：`std::vector<uint8_t> memory_(256 * 1024, 0);`

Rust 方案：`Box<[u8; 256 * 1024]>` 或 `Vec<u8>`（固定容量）。

```rust
pub const MEM_SIZE: u32 = 256 * 1024;
pub const NULL_TRAP_SIZE: u32 = 0x1000;
pub const GLOBAL_START: u32 = 0x1000;
pub const HEAP_START: u32 = 0x5000;
pub const STACK_START: u32 = 0x10000;

pub struct CideVM {
    memory: Vec<u8>,       // len = MEM_SIZE，不动态扩容
    stack: Vec<i32>,       // 运算栈（与线性内存分离）
    mem_stack_top: u32,    // 内存中的栈顶指针（向下增长）
    // ...
}
```

**Load/Store 统一走 checked 方法**，Rust 的切片索引检查自动防止越界：
```rust
fn load_i32(&self, addr: u32) -> Result<i32, Trap> {
    if addr < NULL_TRAP_SIZE { return Err(Trap::NullDeref(addr)); }
    let bytes = self.memory.get(addr as usize..addr as usize + 4)
        .ok_or(Trap::OutOfBounds(addr))?;
    Ok(i32::from_le_bytes(bytes.try_into().unwrap()))
}
```

### 3.5 Host Functions

C++ 当前用 `std::function<void(std::vector<int32_t>&, CideVM*, void*)>` 注册回调。

Rust 方案：用函数指针表（`[Option<HostFn>; N]`）或 `HashMap<u32, HostFn>`。由于 host function 集合固定（id 0~N），直接用数组索引更快：

```rust
type HostFn = fn(stack: &mut Vec<i32>, vm: &mut CideVM, ctx: &mut HostCtx);

pub struct CideVM {
    host_funcs: [Option<HostFn>; MAX_HOST_FUNC_ID],
    host_ctx: *mut c_void,   //  opaque，实际指向 capi 层的 HostCtx
}
```

`HostCtx` 在 `capi` 层定义，包含 `outputLines`、`memory.regions` 等可变状态，通过 `unsafe` 指针透传给 host function。

---

## 4. 分阶段迁移计划

### 总时间预估：4~5 周（1 人全职）

---

### Phase 0：基础设施 + 空壳运行（第 1 周）

**目标**：建立 Rust 项目骨架，C API 全部函数实现为返回 `-1` 的空壳，C# 前端能正常启动不崩溃，`build.ps1` 能产出 `cide_native.dll`。

**任务清单**：

| # | 任务 | 验收标准 |
|---|------|----------|
| 0.1 | 创建 `native/Cargo.toml`、`.cargo/config.toml`、目录结构 | `cargo check` 通过 |
| 0.2 | 定义 Rust 侧的 `Session`、`CompileState`、`RuntimeState`、`MemoryState` | 结构体字段与 C++ 侧语义对齐 |
| 0.3 | 实现 `cide_session_create/destroy` | C# 启动不崩溃 |
| 0.4 | 实现所有 C API 函数的**空壳**（返回 -1 / 0 / nullptr） | `cide_capi.h` 中每个函数在 Rust 侧都有 `#[no_mangle]` 对应 |
| 0.5 | 修改 `build.ps1`：Desktop 路径调用 `cargo build --release` | 输出 `native/target/release/cide_native.dll` 到 `dist/desktop/` |
| 0.6 | 配置 Android 交叉编译：`aarch64-linux-android`、`armv7-linux-androideabi` | `cargo ndk` 或手动配置 linker，输出 `.so` 到 Maui 的 `lib/` 目录 |
| 0.7 | 移植 `MinimalTest.cpp` 为 Rust `#[test]` | `cargo test minimal` 通过（此时只测 create/destroy/compile 空壳返回 -1） |

**关键技术点**：
- `.cargo/config.toml` 中配置 Android NDK linker：
  ```toml
  [target.aarch64-linux-android]
  linker = "C:/Android/ndk/27.0.1/toolchains/llvm/prebuilt/windows-x86_64/bin/aarch64-linux-android21-clang.cmd"
  [target.armv7-linux-androideabi]
  linker = "C:/Android/ndk/.../bin/armv7a-linux-androideabi21-clang.cmd"
  ```
- `build.ps1` Desktop 分支：
  ```powershell
  cargo build --manifest-path native/Cargo.toml --release
  Copy-Item native/target/release/cide_native.dll dist/desktop/
  ```
- `build.ps1` Android 分支：
  ```powershell
  cargo ndk --target aarch64-linux-android --platform 21 build --release
  cargo ndk --target armeabi-v7a --platform 21 build --release
  ```

**回退策略**：保留 `native/src_cpp_backup/` 完整备份，随时可以切回 CMake 构建。

---

### Phase 1：VM 核心 + Host Functions（第 2 周）

**目标**：Rust VM 能完整执行现有 C++ 编译器生成的字节码。这是风险最高的模块，但边界最清晰。

**任务清单**：

| # | 任务 | 验收标准 |
|---|------|----------|
| 1.1 | 移植 `OpCode`、`Instruction` 到 Rust（`#[repr(u8)]` enum） | 指令编码与 C++ 完全一致 |
| 1.2 | 实现 `CideVM` 核心：内存、栈、寄存器、Reset/LoadProgram/SetGlobals | 单元测试：内存读写边界检查正确 |
| 1.3 | 实现 `Run()` 主循环 + `Step()` | 用 `match opcode` 替代 C++ 的 switch |
| 1.4 | 实现所有算术/比较/逻辑/控制流指令 | `Phase2RegressionTest` 中纯算术用例通过 |
| 1.5 | 实现 `StepEvent`、`Breakpoint`、`Pause/Resume/Cancel` | 单步调试行为与 C++ 一致 |
| 1.6 | 移植 Host Functions：`__cide_output`、`__cide_step`、`malloc`、`free`、`printf` 系列、`scanf` 系列 | I/O 测试通过 |
| 1.7 | 移植 `CallStack`、`VariableSnapshot`、`VisEvents` | 调试面板数据正常 |
| 1.8 | 实现 `cide_run`、`cide_step_next` 等 C API | `Phase2RegressionTest` 全部通过 |

**关键设计**：
- VM 的 `Run()` 返回 `Result<i32, RuntimeError>`，C API 层转换为 `int`。
- `Trap` 统一为 Rust `enum`，避免 C++ 的字符串拼接错误信息：
  ```rust
  pub enum RuntimeError {
      DivByZero { loc: SourceLoc },
      NullDeref { addr: u32 },
      OutOfBounds { addr: u32 },
      StackUnderflow,
      InfiniteLoop,
      Unimplemented(OpCode),
  }
  ```
- Host function 中的 `malloc`/`free` 管理 `MemoryState.regions` 和 `freeList`，逻辑直接翻译。

**验收测试**：
```bash
# Rust 侧
 cargo test --release vm::

# C++ 侧（链接 Rust DLL）
# 需要临时修改 CMakeLists.txt，让测试链接 Rust 生成的 .dll/.lib
# 或直接用 Rust 重写 Phase2RegressionTest
```

---

### Phase 2：编译器前端（第 3~4 周）

**目标**：实现完整编译管线：Lexer → Parser → TypeChecker → BytecodeGen。

**2.1 Lexer（2 天）**

- 直接翻译：`source_` 字符串 → `chars().peekable()` 迭代器。
- `#define` 宏展开：用 `HashMap<String, Vec<Token>>` 保存宏定义。
- **零进度保护**：C++ 中 `if (pos_ == checkpoint) Advance()`，Rust 中 Parser 负责处理（Lexer 本身不会死循环）。

**验收**：`cide_compile` 对非法字符、未闭合字符串等返回正确错误码。

**2.2 Parser（5 天）**

- 将 C++ 的 `std::unique_ptr<ProgramNode>` 替换为 Rust `Program { structs: Vec<StructDecl>, globals: Vec<VarDecl>, funcs: Vec<FuncDecl> }`。
- 递归下降 + precedence climbing 直接翻译。
- **关键改进**：Rust `match` + `Result` 替代 C++ 的错误累积模式。Parser 遇到错误后返回 `Err(ParseError)`，外层决定是否继续解析下一个 top-level 声明。
- **零进度保护**：每个循环/递归入口记录 `checkpoint = self.pos`，如果解析后 `self.pos == checkpoint`，强制 `self.pos += 1` 并报告错误。

**验收**：所有 Phase3 Batch1~4 测试用例编译通过（不运行，只看 compile 返回值和诊断）。

**2.3 TypeChecker（3 天）**

- 符号表：`Vec<HashMap<String, VarSymbol>>` 替代 C++ 的 `vector<unordered_map>`。
- 类型推导：`resolve_expr_type(&self, expr: &Expr) -> Result<Type, TypeError>`。
- 诊断累积：使用 `&mut Vec<Diagnostic>`，允许多个错误同时报告（如参数列表类型检查）。

**验收**：类型错误测试、数组初始化测试通过。

**2.4 BytecodeGen（3 天）**

- 将 C++ `AstVisitor` 替换为 Rust 的递归生成函数。
- **Jump patch 机制**：C++ 中 `breakPatches_` / `continuePatches_` 存储 `size_t` IP，Rust 中用 `Vec<usize>` 同样实现。
- **SourceMap**：每 Emit 一条指令记录 `ip → SourceLoc`。
- **函数表**：`HashMap<String, FuncMeta>` 直接翻译。

**验收**：`Phase2RegressionTest`、`Phase3StepTest`、`StructInitTest` 全部通过（编译 + 运行）。

---

### Phase 3：Diagnostics（第 4 周末）

**目标**：`AlgorithmMatcher` + 诊断格式化 + 自动修复建议。

| # | 任务 | 验收标准 |
|---|------|----------|
| 3.1 | 移植 `ErrorCodes.hpp` 到 Rust enum | 枚举值与 C 头文件一致 |
| 3.2 | 实现诊断美化和修复建议生成 | `Stage2DiagnosticTest` 通过 |
| 3.3 | 移植 `AlgorithmMatcher` | `AlgorithmMatchTest` 通过 |

---

### Phase 4：收尾 + 性能回归（第 5 周）

| # | 任务 | 验收标准 |
|---|------|----------|
| 4.1 | 实现 `cide_session_save/load` 序列化 | 格式与 C++ 兼容（或设计版本化头） |
| 4.2 | 将 C++ 测试全部迁移为 Rust `#[test]` | `cargo test` 一键通过所有用例 |
| 4.3 | 性能基准：对比 C++ 和 Rust 的编译 + 执行耗时 | 差异 < 20%（Rust VM 解释器通常持平或略快） |
| 4.4 | DLL/SO 体积检查 | Release 体积与 C++ 版本差异 < 30% |
| 4.5 | 清理：删除 `native/src/` 全部 C++ 文件、`CMakeLists.txt`、build 目录 | 只剩 Rust 代码 |
| 4.6 | 更新 `AGENTS.md`、文档 | 构建命令改为 cargo |

---

## 5. 测试迁移策略

### 策略：先"黑盒链接"，后"内化 Rust"

**阶段 A（Phase 0~2）**：保留 C++ 测试源码，修改构建脚本让测试链接 Rust 生成的 `cide_native.dll`。

临时 `native/tests_cpp/CMakeLists.txt`：
```cmake
# 不编译 native 源码，只编译测试可执行文件，链接预构建的 Rust DLL
add_executable(phase2_test Phase2RegressionTest.cpp)
target_link_libraries(phase2_test PRIVATE ${CMAKE_SOURCE_DIR}/../target/release/cide_native.lib)
```

**阶段 B（Phase 4）**：将测试翻译为 Rust `#[test]`：

```rust
#[test]
fn test_simple_add() {
    let mut session = Session::new();
    let code = "int main() { int a = 3; int b = 5; return a + b; }";
    assert!(session.compile(code).is_ok());
    assert_eq!(session.run().unwrap(), 8);
}
```

**优势**：
- 阶段 A 确保 ABI 兼容性被持续验证。
- 阶段 B 后不再需要 C++ 编译器跑测试，CI 更简单。

---

## 6. 构建系统改造对照表

| 场景 | 当前（C++） | 迁移后（Rust） |
|------|-------------|----------------|
| Desktop Debug | `cmake -S native -B native/build -G Ninja` | `cargo build` |
| Desktop Release | `cmake --build native/build --config Release` | `cargo build --release` |
| Android arm64 | `cmake -DANDROID_ABI=arm64-v8a + NDK toolchain` | `cargo ndk -t aarch64-linux-android build --release` |
| Android armv7 | `cmake -DANDROID_ABI=armeabi-v7a + NDK toolchain` | `cargo ndk -t armeabi-v7a build --release` |
| 运行测试 | `ctest` 或手动运行测试可执行文件 | `cargo test` |
| 输出 DLL | `native/build/bin/Release/cide_native.dll` | `native/target/release/cide_native.dll` |
| 输出 SO | `native/build-android-*/lib/libcide_native.so` | `native/target/{triple}/release/libcide_native.so` |

### build.ps1 关键改动点

```powershell
# Desktop Native
& cargo build --manifest-path "$root/native/Cargo.toml" --release
$dllSource = "$root/native/target/release/cide_native.dll"

# Android Native (需预装 cargo-ndk)
& cargo ndk --manifest-path "$root/native/Cargo.toml" `
    --target aarch64-linux-android --platform 21 build --release
& cargo ndk --manifest-path "$root/native/Cargo.toml" `
    --target armv7-linux-androideabi --platform 21 build --release
```

---

## 7. 风险与缓解

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| ABI 细微差异导致 C# P/Invoke 崩溃 | 中 | 高 | Phase 0 实现空壳时即做 ABI 对齐检查；保留 C++ 测试链接验证 |
| Android 交叉编译 toolchain 配置复杂 | 中 | 中 | 提前在 Phase 0 验证 cargo-ndk 或手动 linker 配置；保留 C++ Android 构建作为备份 |
| Parser 死循环/栈溢出（历史 bug） | 低 | 高 | Rust 的迭代器边界检查 + `Result` 错误处理从根本上避免；保留零进度保护 |
| VM 性能退化 | 低 | 中 | Phase 1 末尾做基准测试；Rust `match` 解释器通常与 C++ switch 持平 |
| Session 序列化格式不兼容 | 低 | 低 | Phase 4 专门处理；如不能兼容，在 magic header 中加入版本号，支持双版本读取 |
| 字符串/UTF-8 处理差异 | 中 | 中 | C API 层对 `CStr::to_str()` 失败情况返回明确错误；中文字符串测试覆盖 |

---

## 8. 开始执行的第一步（今天可做）

1. **备份**：`git checkout -b rust-migration`，保留完整 C++ 代码历史。
2. **安装工具链**：
   ```powershell
   rustup target add aarch64-linux-android
   rustup target add armv7-linux-androideabi
   cargo install cargo-ndk
   ```
3. **创建骨架**：建立 `native/Cargo.toml` 和 `src/lib.rs`，实现 `cide_session_create/destroy` 空壳。
4. **验证构建**：修改 `build.ps1` 的 Desktop 分支，确认能产出 `cide_native.dll` 并被 C# 加载。

---

*计划制定时间：2026-05-09*
*预期完成时间：4~5 周后*
