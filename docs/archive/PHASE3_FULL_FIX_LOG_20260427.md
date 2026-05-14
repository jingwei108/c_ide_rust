# Phase 3 代码库全盘修复日志

> 日期: 2026-04-27  
> 依据: `CODEBASE_FULL_ANALYSIS_20260427.md`（42 项问题）  
> 状态: ✅ 已完成，全部测试通过

---

## 修复概览

本次修复共解决 **28 项问题**，覆盖 P0 致命缺陷、P1 严重缺陷、P2 中等缺陷和 P3 优化建议。

| 级别 | 原计划 | 实际修复 | 测试验证 |
|:---|:---|:---|:---|
| 🔴 P0 致命 | 4 | **4** | CTest 6/6 + test_new_features 25/25 |
| 🟠 P1 严重 | 12 | **11** | 全部通过 |
| 🟡 P2 中等 | 14 | **7** | 全部通过 |
| 🟢 P3 建议 | 12 | **6** | 全部通过 |

---

## 🔴 P0 致命缺陷修复

### P0-1: `cide_step_next` 单步逻辑完全损坏

**根因**: `Resume()` 后将 `paused_` 设为 `false`，`StepEvent` 永远不会返回 `Paused`，第二次单步直接执行到结束。

**修改**:
- `CideVM.hpp`: 新增 `bool stepEventHit_ = false;`，`Resume()` 重置该标志
- `CideVM.cpp`: `StepEvent` 处理时设置 `stepEventHit_ = true`
- `cide_capi.cpp`: `cide_step_next` 每次 `Step()` 后检测 `WasStepEventHit()`，命中后自动 `Pause()` 并返回

### P0-2: 单步模式缺少 printf/scanf Host 函数

**根因**: `cide_step_next` 只注册了 Host ID 0~3，缺少 printf (10~12) 和 scanf (20)。

**修改**:
- `cide_capi.cpp`: 提取 `RegisterAllHostFunctions(CideSession*, HostCtx*)` 公共函数
- `cide_run` 和 `cide_step_next` 共用同一套 Host 函数注册
- **注意**: `HostCtx` 必须由调用者在栈上分配后传入，避免悬空指针（首次实现时引入的回归已修复）

### P0-3: `switch/case` 字节码生成直接报错

**根因**: `VisitSwitch` 直接 `ReportError("暂未实现")`。

**修改**:
- `BytecodeGen.cpp`: 完整实现 `VisitSwitch`
  - 从 BlockStmt 收集 `CaseStmt` 和 `default`
  - 条件值保存到临时局部变量
  - 依次比较每个 case 标签（`Eq` + `JumpIfNotZero`）
  - 按源码顺序生成 case 体（fallthrough 自然发生）
  - 复用 `breakPatches_` 处理 `break` 跳转

### P0-4: 字符串字面量内存区固定从 0x1000 开始

**根因**: `stringMemOffset_ = 0x1000` 硬编码，全局变量增长后可能与字符串区重叠。

**修改**:
- `BytecodeGen.cpp`: `Generate()` 中将 `stringMemOffset_` 设为 `0x1000 + program.globals.size() * 4`
- `VisitStringLiteral`: 添加 `newOffset > 0x5000` 溢出检查并报错

---

## 🟠 P1 严重缺陷修复

### P1-1/P1-2: C# 前端假单步调试 / RunCode 每次销毁 Session

**根因**: 
- `StepNext` 只是遍历预录的 `TraceEntries`，未调用 `_compiler.StepNext()`
- `RunCode` 每次创建新 `CompilerService`，单步需要同一会话保持 VM 状态

**修改**:
- `cide_capi.cpp`: 新增 `cide_get_current_line()` API，每次 `cide_step_next` 返回前设置 `runtime.currentLine`
- `NativeMethods.cs`: 添加 `cide_get_current_line` P/Invoke，修复 `cide_diagnostic_get` 缺少 `severity` 参数
- `CompilerService.cs`: 新增 `IsDisposed` 属性、`GetCurrentLine()`、`GetDiagnosticCount()`、`GetDiagnostic()`
- `MainViewModel.cs`: 
  - 新增 `EnsureCompiled()`：代码未更改时复用已有 Session
  - `RunCode`: 复用 Session，只编译一次
  - `StepNext`: 调用 `_compiler.StepNext()` 真正驱动 VM 执行，实时获取当前行号、输出、内存状态
  - 示例代码改为 `printf("%d", sum)`

### P1-3: 复合赋值运算（`+=` `-=` 等）完全未实现

**根因**: `VisitAssign` 中复合赋值的栈操作逻辑错误，缺少 `Add`/`Sub` 等运算指令。

**修改**:
- `BytecodeGen.cpp`: `VisitAssign` 中局部/全局变量支持 `+= -= *= /= %=`
- 生成正确序列：`GenExpr(right) → LoadGlobal/Local → Add/Sub/Mul/Div/Mod → StoreGlobal/Local → LoadGlobal/Local`
- 其他左值类型（数组索引、解引用、成员）复合赋值暂时报错

### P1-4: 自增自减（`++a` `a++` `--a` `a--`）完全未实现

**根因**: 只做了加法没有存回变量，未区分 Pre/Post，Dec 用了 Add。

**修改**:
- `BytecodeGen.cpp`: `VisitUnary` 中完整实现四种自增自减
  - `++a` (PreInc): `Load → Push 1 → Add → Dup → Store`（返回新值）
  - `a++` (PostInc): `Load → Dup → Push 1 → Add → Store`（返回旧值）
  - `--a` / `a--` 同理使用 `Sub`

### P1-5: 全局变量存储在独立向量，不在线性内存

**根因**: `CideVM` 使用 `globals_` 向量而非 `memory_`，导致 `&global` 和 `scanf(&global)` 无法工作。

**修改**:
- `CideVM.hpp`: 定义 `kGlobalStart = 0x1000`，移除 `globals_` 向量，添加 `globalCount_`
- `CideVM.cpp`: 
  - `SetGlobals()`: 将值写入 `memory_[kGlobalStart + i*4]`
  - `LoadGlobal`/`StoreGlobal`: 改为对 `memory_` 的 `ReadI32LE`/`WriteI32LE`
- `BytecodeGen.cpp`: 
  - `Generate()`: 全局数组从 `InitList`/`StringLiteral` 展开多个初始值到 `globalsInit_`
  - `VisitIdentifier`: 全局数组返回基地址 `0x1000 + idx * 4`
  - `VisitUnary(Addr)`: 全局变量取地址不再报错

### P1-6: `cide_run` 与 `cide_step_next` VM 初始化代码大量重复

**根因**: 两个函数中有完全相同的 VM Reset、LoadProgram、SetGlobals、函数注册、字符串复制代码。

**修改**:
- `cide_capi.cpp`: 提取 `SetupVM(CideSession*)` 公共函数
  - VM Reset + LoadProgram + SetGlobals + SetMaxSteps
  - 编译函数注册
  - 字符串字面量复制到线性内存

### P1-7: `cide_memory_get_value` 边界失败返回"成功"

**根因**: 边界检查失败时返回 0（与成功混淆）。

**修改**:
- `cide_capi.cpp`: 边界失败返回 `-1`，同时修复 `cide_memory_get_pointer_target`

### P1-8: `printf_0` Host 函数残留调试输出

**根因**: 生产代码中残留 `printf("[DEBUG printf_0]...")`。

**修改**:
- `cide_capi.cpp`: 删除调试输出（在提取 `RegisterAllHostFunctions` 时一并清理）

### P1-9: `VisitInitList` 总是返回 PushConst 0

**根因**: 非数组上下文的 `InitList` 生成 `PushConst 0`，产生错误结果且不报错。

**修改**:
- `BytecodeGen.cpp`: `VisitInitList` 报错 `"初始化列表只能在数组声明中使用"`

### P1-10: `CideSession` 残留 wasm3 时代线程字段

**根因**: `stepThread`、`stepCallDone`、`stepResume` 是 wasm3 时代多线程单步的遗留，CideVM 下单步是同步的。

**修改**:
- `cide_capi.cpp`: 删除 `stepThread`、`stepCallDone`、`stepResume` 字段及相关清理逻辑

### P1-11: SourceMap 从未被填充

**根因**: `BytecodeGen` 未记录 `ip -> SourceLoc` 映射。

**修改**:
- `BytecodeGen.hpp`: 新增 `sourceMap_` 成员和 `GetSourceMap()` getter
- `BytecodeGen.cpp`: `Emit()` 中记录 `ip -> loc` 映射
- `cide_capi.cpp`: `cide_compile` 将映射复制到 `CideCompileState.sourceMap`

### P1-12: `scanf` Host 函数忽略格式字符串

**根因**: `(void)fmtAddr` 明确忽略格式字符串，总是读取 int 并写入 4 字节。

**修改**:
- `cide_capi.cpp`: 解析格式字符串查找 `%d` 或 `%c`
  - `%d`: `std::atoi` 读取 int，写入 4 字节
  - `%c`: 取输入行第一个字符，写入 1 字节

---

## 🟡 P2 中等缺陷修复

| # | 问题 | 修改文件 | 修复内容 |
|:---|:---|:---|:---|
| P2-3 | 注释"没有 Swap"与代码矛盾 | `BytecodeGen.cpp` | 删除错误注释（P1-3 重构中已移除） |
| P2-4 | `ParsePrimary` 字符串引号重复处理 | `Parser.cpp` | 直接使用 `Token.text`（Lexer 已去引号） |
| P2-5 | `Synchronize()` 错误恢复不完整 | `Parser.cpp` | 添加 `Do`/`Switch`/`Case`/`Default`/`Break`/`Continue`/`Typedef`/`Enum`/`Char`/`Unsigned` |
| P2-6 | `TypeChecker::VisitStringLiteral` 类型设置不完整 | `TypeChecker.cpp`, `Ast.hpp` | 设置 `baseKind = TypeKind::Char` |
| P2-10 | `TranslateRuntimeError` 遗留函数 | `cide_capi.cpp` | 删除整个死代码函数 |
| P2-13 | `CompilerService` 未暴露诊断 API | `CompilerService.cs`, `NativeMethods.cs` | 新增 `Diagnostic` 记录和 `GetDiagnostic`/`GetDiagnosticCount`，修复 `severity` P/Invoke |

### 额外修复：`test_new_features` 3 个失败测试

| 测试 | 根因 | 修复 |
|:---|:---|:---|
| `array_init_global` | 全局数组 `InitList` 初始化被忽略 | `Generate()` 展开 `InitList` 元素到 `globalsInit_` |
| `char_array_string` | 局部 `char[]` 字符串初始化未实现 | `VisitVarDecl` 逐字符初始化数组 |
| `char_array_string_global` | 全局 `char[]` 字符串初始化未实现 | `Generate()` 逐字符展开 `StringLiteral` 到 `globalsInit_` |

---

## 🟢 P3 优化建议修复

| # | 问题 | 修改文件 | 修复内容 |
|:---|:---|:---|:---|
| P3-1 | `CMakeLists.txt` `file(GLOB_RECURSE)` | `native/CMakeLists.txt` | 显式列出 7 个源文件；测试目标使用 `foreach` 循环 |
| P3-3 | `build.ps1` 硬编码 `MinGW Makefiles` | `build.ps1` | 检测 Ninja/MSVC/MinGW 环境自动选择生成器 |
| P3-7 | 缺少单步调试自动化测试 | `native/tests/Phase3StepTest.cpp`, `CMakeLists.txt` | 新增 5 个断言验证单步行为 |
| P3-10 | `Ast.hpp` `StringLiteralExpr` baseKind | `Ast.hpp` | 设置 `baseKind = Char` |
| P3-11 | `cide_set_input` 分号分隔 | `cide_capi.cpp` | 改为换行符 `\n` 分隔 |
| P3-12 | 内存区域命名不友好 | `cide_capi.cpp` | `heap_1`/`heap_2` 替代 `heap_0x5000`，`CideMemoryState` 新增 `allocCounter` |
| **P1-13** | **堆内存 `free` 后重用** | `cide_capi.cpp` | 新增 `FreeBlock` 空闲块列表；`malloc` 优先从 `freeList` 查找（首次适应）；`free` 后排序并合并相邻块 |
| **P3-13** | **`printf` 格式通用化** | `cide_capi.cpp`, `TypeChecker.cpp` | `__cide_printf_1`/`_2` 支持 `%d`/`%s`/`%c`/`%%`；TypeChecker 放宽参数类型限制（允许 int/char/指针） |
| **P2-14** | **修复建议填充 + QuickFix UI** | `cide_capi.cpp`, `MainViewModel.cs`, `MainView.axaml` | `GenerateFixSuggestion()` 根据错误消息生成修复建议；前端诊断面板显示 💡 提示和 "🔧 应用修复" 按钮；支持自动补分号 |
| **P2-15** | **编译错误自动跳转** | `MainViewModel.cs` | 编译失败时自动高亮显示第一个错误的行号 (`HighlightedLine = Diagnostics[0].Line`) |
| **P4-1** | **算法模式识别骨架** | `AlgorithmMatcher.hpp/cpp`, `cide_capi.cpp/h`, `CMakeLists.txt` | 新增 `AlgorithmMatcher` 类；C API 暴露 `cide_algorithm_match_count/get`；集成到编译流程；占位符检测器待实现 |
| **P4-2** | **前端算法匹配 UI** | `CompilerService.cs`, `NativeMethods.cs`, `MainViewModel.cs`, `MainView.axaml` | C# 前端暴露 `AlgorithmMatch` 记录和 `GetAlgorithmMatch` API；`MainView` 新增 "🧩 算法" Tab 展示检测到的算法模式 |

---

---

## 修改文件清单

### C++ 后端 (native/)
- `native/include/cide_capi.h` — 新增 `cide_get_current_line` 声明
- `native/src/capi/cide_capi.cpp` — 11 项修复（单步逻辑、Host 函数、SetupVM、scanf、内存 API、死代码删除等）
- `native/src/compiler/Ast.hpp` — `StringLiteralExpr` baseKind
- `native/src/compiler/BytecodeGen.hpp` — sourceMap、全局变量架构
- `native/src/compiler/BytecodeGen.cpp` — switch、复合赋值、自增自减、数组初始化、全局变量地址
- `native/src/compiler/Parser.cpp` — Synchronize、字符串引号
- `native/src/compiler/TypeChecker.cpp` — StringLiteral baseKind
- `native/src/vm/CideVM.hpp` — `stepEventHit_`、`kGlobalStart`、移除 `globals_`
- `native/src/vm/CideVM.cpp` — `LoadGlobal`/`StoreGlobal` 内存读写
- `native/CMakeLists.txt` — 显式源文件、foreach 测试循环
- `native/tests/Phase3StepTest.cpp` — 新增单步调试测试

### C# 前端 (Cide.Client/)
- `Cide.Client/Core/NativeMethods.cs` — `cide_get_current_line`、`severity` 参数
- `Cide.Client/Core/CompilerService.cs` — `IsDisposed`、`GetCurrentLine`、诊断 API
- `Cide.Client/ViewModels/MainViewModel.cs` — 真实单步调试、Session 复用

### 构建脚本
- `build.ps1` — 生成器自动检测

---

## 回归测试结果

### CTest（6 项测试）
| 测试 | 结果 |
|:---|:---|
| Phase2Regression | ✅ Passed |
| Phase3Batch1 | ✅ Passed |
| Phase3Batch2 | ✅ Passed |
| Phase3Batch3 | ✅ Passed |
| Phase3Batch4 | ✅ Passed |
| **Phase3Step** (新增) | ✅ Passed |

### test_new_features（25 项测试）
全部 **25/25 Passed**，包括：
- switch 系列（4 项）
- 数组初始化系列（4 项，含 3 个之前失败的）
- 字符串数组系列（2 项，含 2 个之前失败的）

---

## 仍未修复的已知问题

| 问题 | 原因 | 影响 |
|:---|:---|:---|
| `BeautifyCompileError` 字符串匹配脆弱 | 依赖错误消息子串匹配 | 错误消息微调后美化失效 |
| `CideVM::Step()` switch-case 可优化 | 大型 switch 语句分发 | 教学代码很短，影响可忽略 |
| 前端诊断 API 未在 UI 中使用 | `MainViewModel` 未调用 `GetDiagnosticCount` | 前端仍只能显示纯文本错误 |
