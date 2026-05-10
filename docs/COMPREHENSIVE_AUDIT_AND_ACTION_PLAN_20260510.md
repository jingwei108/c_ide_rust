# Cide 项目全面审查报告与行动计划

**审查日期**: 2026-05-10  
**审查范围**: Rust Native (后端) + C# Frontend (Desktop/Maui/Shared) + 测试覆盖 + 编译警告  
**审查者**: Kimi Code CLI

---

## 一、已完成项（本次会话）

| # | 事项 | 状态 | 说明 |
|---|------|------|------|
| 1 | Rust `cargo check` 警告清零 | ✅ | 修复了 error_codes 命名风格、未使用变量/字段/方法等 66+ 条警告 |
| 2 | TypeChecker 错误代码勘误 | ✅ | 修复了 `W3050`/`W3051` 被滥用于不相关警告的问题，新增 `W3052`~`W3055` |
| 3 | clippy `int_plus_one` | ✅ | `capi/mod.rs` 边界检查简化 |

---

## 二、Rust Native 层问题清单

### 🔴 P0 — 高优先级（稳定性 / 死循环 / Panic）

| # | 问题 | 位置 | 影响 | 修复方案 |
|---|------|------|------|----------|
| P0-1 | **`parse_case_stmt` 死循环** | `parser.rs:862-865` | case 体内遇非法 token 时 `pos` 不前进，while 无限循环 | 添加 `stmt_checkpoint` + `pos == checkpoint` 保护 |
| P0-2 | **`advance()` 空 token 列表 panic** | `parser.rs:63-66` | `pos` 为 0 时 `pos - 1` 产生 usize 下溢 | 添加 `pos == 0` 保护分支 |
| P0-3 | **`synchronize()` 从未被调用** | `parser.rs:96-109` | 解析错误后无错误恢复，导致级联错误（cascading errors） | 在 `parse_statement` fallback 或关键 consume 失败后调用 `synchronize()` |
| P0-4 | **VM `write_i32` 无边界检查** | `vm/vm.rs:328-331` | 内部辅助函数直接写内存，越界风险 | 添加 `addr + 4 <= MEM_SIZE` 检查 |
| P0-5 | **host `scanf` 仅支持单参数** | `vm/host_funcs.rs:261-287` | `scanf("%d %d", &a, &b)` 只能读第一个 | 扩展为 `host_scanf_n`，支持多参数解析 |

### 🟡 P1 — 中优先级（功能缺陷 / 不一致）

| # | 问题 | 位置 | 影响 | 修复方案 |
|---|------|------|------|----------|
| P1-1 | **`parse_case_stmt` 使用 `unwrap()`** | `parser.rs:869` | `stmts.into_iter().next().unwrap()` | 改为 `expect` 或更安全的取法 |
| P1-2 | **`parse_enum_decl` 误用非标识符 token** | `parser.rs:~895` | `consume(Identifier)` 失败后继续用逗号等作成员名 | 检查 token 类型后再插入 |
| P1-3 | **`parse_typedef` 失败静默** | `parser.rs:~876` | 类型解析失败后仍插入 void typedef | 仅在类型解析成功时插入 |
| P1-4 | **`cide_memory_get_value` / `get_pointer_target` 为 stub** | `capi/mod.rs:646-668` | 始终返回 -1，前端内存读取功能失效 | 实现通过 VM memory slice 的读取逻辑 |
| P1-5 | **`cide_session_save/load` 为 stub** | `capi/mod.rs:40-48` | 返回 -1，会话持久化未实现 | 实现 JSON/二进制序列化或标记为暂不支持 |
| P1-6 | **TypeChecker `check_condition` 仅检测顶层赋值** | `type_checker.rs:474-481` | `if (a == (b = 1))` 中的赋值不会被警告 | 递归检测赋值表达式 |

### 🟢 P2 — 低优先级（优化 / 代码质量）

| # | 问题 | 位置 | 影响 | 修复方案 |
|---|------|------|------|----------|
| P2-1 | **`SourceLoc` 多余 `clone()`** | 多处 | `Copy` trait 已存在，无需 clone | 批量替换 `.clone()` 为直接拷贝 |
| P2-2 | **`Type::to_string` 与标准 trait 冲突** | `ast.rs:91` | 应实现 `std::fmt::Display` | 保留 `to_string` 并添加 `Display` impl |
| P2-3 | **缺少 `Default` for `BytecodeGen` / `TypeChecker`** | 多处 | clippy 建议 | 添加 `#[derive(Default)]` 或手动实现 |
| P2-4 | **`unsigned` 映射为 `int` 无提示** | `parser.rs:269` | 语义简化但用户不知情 | TypeChecker 中添加 `unsigned -> int` 隐式转换提示 |

---

## 三、C# 前端问题清单

### 🔴 P0 — 高优先级（内存泄漏 / 崩溃风险）

| # | 问题 | 位置 | 影响 | 修复方案 |
|---|------|------|------|----------|
| P0-1 | **Desktop `ConsoleOutput` 无上限** | `MainViewModel.cs` | 无限输出导致内存膨胀 | 同步 Maui 的 `MaxConsoleOutputLength = 50000` + `TruncateOutput()` |
| P0-2 | **Desktop CTS 未完整 Dispose** | `MainViewModel.cs` | `CancelAllAnimationsAndSnap`/`StopExecution`/`FinishExecution` 仅 Cancel | 追加 `Dispose()` + `null` 赋值 |

### 🟡 P1 — 中优先级（功能缺失）

| # | 问题 | 位置 | 影响 | 修复方案 |
|---|------|------|------|----------|
| P1-1 | **Maui 不支持 `scanf` 输入** | `MainViewModel.cs` (Maui) | 始终 `SetInput(string.Empty)` | 添加 `InputText` 属性并绑定到输入框 |
| P1-2 | **C# 前端无任何测试** | 全局 | 无回归保护 | 新建 `Cide.Client.Tests` / `Cide.Client.Shared.Tests` 项目 |
| P1-3 | **P/Invoke 缺失多文件编译 API** | `NativeMethods.cs` | 不支持 `cide_compile_unit` / `cide_compile_all` | 补全声明并在 `CompilerService` 暴露多文件接口 |

### 🟢 P2 — 低优先级

| # | 问题 | 位置 | 影响 | 修复方案 |
|---|------|------|------|----------|
| P2-1 | **Maui `MainPage` 未显式 dispose VM** | `MainPage.xaml.cs` | Singleton 模式下风险较低，但页面重建时可能悬空 | 在 `OnDisappearing` 或页面卸载时调用 `VM.Dispose()` |
| P2-2 | **`LineNumberItem.IsDark` static 设计不佳** | `CodeEditor.axaml.cs` | 运行时主题变化时所有实例共享同一逻辑 | 改为实例属性或绑定 |

---

## 四、端到端测试覆盖缺口

### 当前状态
- **compile_pipeline_test.rs**: 12 个测试 ✅
- **end_to_end_test.rs**: 17 个测试 ✅
- **C# 前端测试**: 0 个 ❌

### 建议新增的 Rust E2E 测试

| # | 测试场景 | 优先级 | 备注 |
|---|----------|--------|------|
| 1 | `do-while` 循环 | P1 | 基础控制流 |
| 2 | `break` / `continue` 嵌套 | P1 | 循环控制 |
| 3 | 全局变量读写 | P1 | 数据段 |
| 4 | 字符串字面量 / `char` 数组 | P1 | `char str[] = "hello";` |
| 5 | `sizeof` 运算符 | P1 | `sizeof(int)`, `sizeof(struct)` |
| 6 | 多参数函数调用 | P1 | `add(1, 2, 3)` |
| 7 | 指针算术 (`p + i`, `p - i`) | P1 | **需拓展子集支持** |
| 8 | 除零错误陷阱 | P1 | 运行时诊断 |
| 9 | 空指针解引用陷阱 | P1 | 运行时诊断 |
| 10 | 多维数组初始化列表 | P1 | `int arr[2][3] = {{1,2,3},{4,5,6}};` |
| 11 | 结构体初始化列表 | P1 | `struct Point p = {3, 4};` |
| 12 | 嵌套 `if` / `else if` | P2 | 控制流 |
| 13 | 逻辑运算符 `&&` / `||` | P2 | 短路求值 |
| 14 | 数组作为函数参数 | P2 | `void f(int arr[], int n)` |
| 15 | 负数和算术溢出 | P2 | `INT_MIN / -1` |
| 16 | 多变量声明 | P2 | `int a = 1, b = 2, c;` |
| 17 | `enum` 声明与使用 | P2 | `enum Color { R, G, B };` |
| 18 | `typedef` 使用 | P2 | `typedef int Integer;` |
| 19 | 无限循环检测（步数限制） | P2 | `while(1);` |

---

## 五、不支持功能拓展计划（子集支持）

### 5.1 指针算术（P1）
**现状**: AGENTS.md 声明不支持 `p++`、`p+i`。  
**拓展方案**:
- BytecodeGen: `指针 + 整数` 已部分支持（乘 4），但 `++`/`--` 仅支持简单变量标识符
- 修改 `UnaryOp::PreInc/PostInc/PreDec/PostDec` 的字节码生成，支持指针类型（增加 `sizeof(base_type)` 而非固定 1）
- 添加 `p++`、`p--`、`++p`、`--p` 的 E2E 测试

### 5.2 `sizeof` 运算符（P1）
**现状**: AST 有 `Sizeof` 节点，但 BytecodeGen 未处理（会触发 `report_error` 或生成错误字节码）。  
**拓展方案**:
- BytecodeGen 的 `gen_expr` 中匹配 `Expr::Sizeof`，在编译期计算大小并 `PushConst`
- 支持 `sizeof(int)`、`sizeof(struct S)`、`sizeof(变量)`

### 5.3 结构体初始化列表（P1）
**现状**: TypeChecker 已支持检查，但 BytecodeGen 仅生成部分代码（通过 `InitList` + `StoreMem`）。  
**拓展方案**:
- 验证全局/局部结构体初始化的字节码生成是否完整
- 添加 E2E 测试确认 `struct Point p = {3, 4}; printf("%d", p.x);`

### 5.4 `scanf` 多参数（P1）
**现状**: `host_scanf_1` 仅支持一个 `%` 格式符。  
**拓展方案**:
- 参照 `host_printf_n` 模式，实现 `host_scanf_n`
- 按格式符数量 pop 多个指针参数
- 解析输入行（按空白分割）并依次写入

### 5.5 `enum` 完整支持（P2）
**现状**: Parser 已解析 enum 为全局 int 常量，但 TypeChecker 和 BytecodeGen 可能未完全处理 enum 类型标识符。  
**拓展方案**:
- 确认 `enum Color { R, G, B }` 生成的全局变量是否正确
- 确认 `enum Color c = R;` 的类型检查

### 5.6 `typedef` 完整支持（P2）
**现状**: Parser 维护 `typedef_names` HashMap，`parse_base_type` 会查询。  
**拓展方案**:
- 确认 TypeChecker 和 BytecodeGen 对 typedef 类型的处理
- 添加 E2E 测试

---

## 六、执行优先级总览

```
Phase 1（立即执行）
├── 修复 Parser 死循环（parse_case_stmt checkpoint）
├── 修复 advance() panic
├── 添加 synchronize() 调用点
├── 修复 Desktop CTS Dispose + ConsoleOutput 上限
└── 添加 Maui InputText 支持

Phase 2（本周内）
├── 拓展指针算术 ++/-- 支持
├── 拓展 sizeof 运算符
├── 拓展 scanf 多参数
├── 修复 cide_memory_get_value stub
├── 添加 10+ 个 Rust E2E 测试
└── 新建 C# 前端单元测试项目

Phase 3（后续迭代）
├── 结构体初始化列表字节码验证
├── enum/typedef 完整 E2E 测试
├── 多文件编译 C# 接口补全
├── VM write_i32 边界检查加固
└── SourceLoc clone 清理等代码质量优化
```

---

## 七、附录：编译命令速查

```powershell
# Rust 检查
Set-Location d:\code\c_ide_rust\native
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test

# C# Desktop 构建
dotnet build Cide.Client.Desktop/Cide.Client.Desktop.csproj

# C# Maui 构建
dotnet build Cide.Client.Maui/Cide.Client.Maui.csproj --framework net10.0-android

# 全量构建
dotnet build Cide.slnx
```
