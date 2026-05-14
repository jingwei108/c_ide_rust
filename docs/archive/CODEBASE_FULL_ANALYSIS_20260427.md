# C IDE 代码库全盘分析报告

> 分析日期：2026-04-27  
> 更新日期：2026-04-27  
> 分析范围：native/ (C++ 后端) + Cide.Client/ (C# Avalonia 前端) + 构建系统 + 测试套件  
> 方法：静态代码审查 + 文档与实现交叉对比 + 架构一致性检查

---

## 修复状态速查（2026-04-27 更新）

本次后续审查确认，原报告中 **绝大多数 P0/P1/P2 问题已在代码中修复**，当前测试套件（8 个测试目标，共 40+ 测试用例）**全部通过**。

| 级别 | 原问题数 | 已修复 | 剩余 |
|:---|:---|:---|:---|
| 🔴 **P0 致命** | 4 | 4 | 0 |
| 🟠 **P1 严重** | 12 | 12 | 0 |
| 🟡 **P2 中等** | 14 | 13 | 1（指针运算与文档矛盾，已决定更新文档承认支持）|
| 🟢 **P3 建议** | 12 | — | 12（质量改进，不影响功能）|

**本轮额外修复**（不在原报告内）：
- `TypeChecker::VisitIndex` 成功路径未设置 `node.type`，导致所有数组索引表达式被误判为 `void` → **已修复**
- `BytecodeGen::Generate` 中 `stringMemOffset_` 初始计算未考虑数组全局变量，可能导致字符串区与全局变量区重叠 → **已修复**

**当前状态**：Stage 2（运行时中文诊断）已完成，进入 Stage 3（变量面板 + 指针追踪 + 内存 Canvas）。

---

## 目录

1. [执行摘要](#1-执行摘要)
2. [问题总览](#2-问题总览)
3. [P0 致命缺陷](#3-p0-致命缺陷)
4. [P1 严重缺陷](#4-p1-严重缺陷)
5. [P2 中等缺陷](#5-p2-中等缺陷)
6. [P3 优化建议](#6-p3-优化建议)
7. [文档与实现不一致清单](#7-文档与实现不一致清单)
8. [模块交叉影响矩阵](#8-模块交叉影响矩阵)
9. [推荐修复路线图](#9-推荐修复路线图)
10. [附录：详细代码引用](#10-附录详细代码引用)

---

## 1. 执行摘要

本次全盘审查共发现 **42 项问题**，其中：

| 级别 | 数量 | 说明 |
|:---|:---|:---|
| 🔴 **P0 致命** | 4 | 导致核心功能完全不可用或数据损坏 |
| 🟠 **P1 严重** | 12 | 导致功能缺失、行为错误或架构断裂 |
| 🟡 **P2 中等** | 14 | 导致维护困难、性能下降或边缘情况失败 |
| 🟢 **P3 建议** | 12 | 代码质量、一致性、可扩展性改进 |

**最关键的发现**：
1. **单步调试完全损坏**：后端 `cide_step_next` 逻辑错误导致第二次调用直接执行到结束；前端 `StepNext` 命令根本未调用 Native API
2. **语言特性大量未实现**：`switch/case` 字节码生成直接报错、`++`/`--` 和 `+=`/`-=` 复合赋值未实现
3. **全局变量架构缺陷**：全局变量存储在独立向量而非线性内存，导致 `&global`、`scanf(&global)` 无法工作
4. **文档与代码严重脱节**：至少 6 处文档声称"已支持/已修复"，但代码未实现

---

## 2. 问题总览

### 按模块分布

```
native/src/vm/CideVM.cpp         ████████  8 项
native/src/capi/cide_capi.cpp    ████████████  12 项
native/src/compiler/BytecodeGen  ████████  8 项
native/src/compiler/Parser.cpp   ██  2 项
native/src/compiler/TypeChecker  █  1 项
Cide.Client/ (C# 前端)          ██████  6 项
测试套件                          ████  4 项
构建系统                          ██  2 项
```

### 问题清单速查表

| # | 级别 | 模块 | 问题简述 |
|:---|:---|:---|:---|
| 1 | P0 | CideVM/C API | `cide_step_next` 单步逻辑完全损坏 |
| 2 | P0 | CideVM/C API | `cide_step_next` 单步模式缺少 printf/scanf Host 函数 |
| 3 | P0 | BytecodeGen | `switch/case` 字节码生成直接报错（文档说已支持） |
| 4 | P0 | BytecodeGen | 字符串字面量内存区固定从 0x1000 开始，可能溢出覆盖堆区 |
| 5 | P1 | C# 前端 | `StepNext` 命令完全未调用 Native API，只是遍历轨迹 |
| 6 | P1 | C# 前端 | `RunCode` 每次运行都销毁 Session，无法后续单步 |
| 7 | P1 | BytecodeGen | 复合赋值（`+=`、`-=` 等）完全未实现 |
| 8 | P1 | BytecodeGen | 自增自减（`++a`/`a++`/`--a`/`a--`）完全未实现 |
| 9 | P1 | CideVM | 全局变量存储在独立 `globals_` 向量，不在线性内存 |
| 10 | P1 | C API | `cide_run` 与 `cide_step_next` VM 初始化代码大量重复 |
| 11 | P1 | C API | `cide_memory_get_value` 边界失败时返回 0（无法区分成功/失败） |
| 12 | P1 | C API | `printf_0` Host 函数残留 `printf` 调试输出 |
| 13 | P1 | BytecodeGen | `VisitInitList` 总是返回 PushConst 0（非数组上下文） |
| 14 | P1 | 架构 | `CideSession` 残留 wasm3 时代线程字段（stepThread/stepResume 等） |
| 15 | P1 | 架构 | SourceMap 从未被填充，`cide_sourcemap_lookup` 始终返回失败 |
| 16 | P1 | C API | `scanf` Host 函数忽略格式字符串，总是读取 int |
| 17 | P2 | BytecodeGen | 全局变量初始化仅支持 `Literal`，表达式初始化被忽略 |
| 18 | P2 | BytecodeGen | 指针运算（`ptr + int`）被实现，与文档"不支持指针运算"矛盾 |
| 19 | P2 | BytecodeGen | 注释与代码矛盾："没有 Swap"但实际使用了 `OpCode::Swap` |
| 20 | P2 | CideVM | `Ret`/`RetVoid` 从 main 返回时，`Run()` 检查 `stack_.empty()` 逻辑 |
| 21 | P2 | Parser | `ParsePrimary` 中 `StringLiteral` 构造重复处理引号（Lexer 已处理） |
| 22 | P2 | C API | `TranslateRuntimeError` 遗留函数，CideVM 已直接输出中文 |
| 23 | P2 | C# 前端 | `CompilerService` 未暴露诊断 API（diagnostic_count/get） |
| 24 | P2 | C# 前端 | `MainViewModel` 示例代码使用 `__cide_output` 而非 `printf` |
| 25 | P2 | 测试 | `test_new_features.cpp` 包含 switch 测试，但字节码未实现，测试会失败 |
| 26 | P2 | 测试 | `Phase3Batch3Test` 使用 `*(p + 1)` 指针运算，与文档教学子集定义矛盾 |
| 27 | P2 | 测试 | 测试解析返回值方式脆弱（依赖中文/英文字符串匹配） |
| 28 | P2 | CideVM | `StepEvent` 在 `Run()` 模式下每次都被检查 `paused_` 状态 |
| 29 | P2 | TypeChecker | `VisitStringLiteral` 设置 `type = Type{TypeKind::Pointer, "char"}` 但 `baseKind` 为 Void |
| 30 | P2 | Parser | `Synchronize()` 错误恢复未包含 `Do`、`Switch`、`Case`、`Default` |
| 31 | P3 | CMake | `CMakeLists.txt` 使用 `file(GLOB_RECURSE)` 反模式 |
| 32 | P3 | CMake | 测试目标逐个手动添加，重复代码多，应使用循环 |
| 33 | P3 | C API | `cide_sourcemap_lookup` 参数名 `wasm_offset` 应改为 `bytecode_offset` |
| 34 | P3 | 架构 | `CideCompileState.sourceMap` 类型使用 `std::pair<uint32_t, SourceLoc>`，应封装为结构体 |
| 35 | P3 | CideVM | `CideVM::Step()` switch-case 可优化为跳转表或函数指针数组 |
| 36 | P3 | C# 前端 | `NativeMethods` 中 `cide_diagnostic_get` 缺少 `severity` 参数暴露 |
| 37 | P3 | C API | `cide_run` 自动追加 "程序运行完成，返回值：X" 到所有输出 |
| 38 | P3 | 构建 | `build.ps1` 硬编码 `MinGW Makefiles`，在纯 MSVC 环境可能不工作 |
| 39 | P3 | CideVM | 堆内存 `free` 后不复用，`heapOffset` 只增不减 |
| 40 | P3 | C API | `BeautifyCompileError` 中的字符串匹配条件可能与实际错误消息不匹配 |
| 41 | P3 | 测试 | 缺少针对单步调试（`cide_step_next`）的自动化测试 |
| 42 | P3 | 文档 | `CODE_REVIEW_PLAN.md` 中提到的 "Parser pos_-- 清理" 已修复，但文件未更新状态 |

---

## 3. P0 致命缺陷

### P0-1: `cide_step_next` 单步逻辑完全损坏

**位置**: `native/src/capi/cide_capi.cpp` (第 598~613 行)

**问题描述**:
`cide_step_next` 的"继续执行到下一步"逻辑存在致命缺陷：

```cpp
// Continue to next step
s->vm.Resume();  // paused_ = false
while (true) {
    auto result = s->vm.Step();
    if (result == cide::CideVM::StepResult::Paused) {
        return 0;  // ❌ 永远不会触发！
    }
    ...
}
```

`CideVM::Step()` 中 `StepEvent` 的处理：
```cpp
case OpCode::StepEvent: {
    currentLine_ = inst.operand;
    if (paused_) {       // 只有 paused_ == true 才返回 Paused
        return StepResult::Paused;
    }
    break;
}
```

由于 `Resume()` 已将 `paused_` 设为 `false`，`StepEvent` 永远不会返回 `Paused`。循环会持续执行直到程序结束或 trap。

**影响**: 单步调试在第一次调用后完全失效，第二次调用会直接执行到程序结束。

**修复建议**:
1. 在 `CideVM` 中增加 `bool stepEventHit_` 标志
2. `StepEvent` 处理时设置 `stepEventHit_ = true`
3. `cide_step_next` 中检测 `stepEventHit_` 后自动 `Pause()` 并返回

---

### P0-2: `cide_step_next` 单步模式缺少 printf/scanf Host 函数

**位置**: `native/src/capi/cide_capi.cpp` (第 540~577 行)

**问题描述**:
`cide_step_next` 只注册了 host function id 0~3：
- 0: `__cide_output`
- 1: `__cide_step`
- 2: `malloc`
- 3: `free`

但 `cide_run` 还注册了：
- 10: `__cide_printf_0`
- 11: `__cide_printf_1`
- 12: `__cide_printf_2`
- 20: `__cide_scanf_1`

**影响**: 在单步调试模式下调用 `printf`/`scanf` 会触发 `"CallHost: 未知宿主函数"` 错误。

**修复建议**: 提取公共的 Host 函数注册逻辑，确保 `cide_run` 和 `cide_step_next` 使用完全相同的 Host 函数集。

---

### P0-3: `switch/case` 字节码生成直接报错

**位置**: `native/src/compiler/BytecodeGen.cpp` (第 453~460 行)

**问题描述**:
```cpp
void BytecodeGen::VisitSwitch(SwitchStmt& node) {
    ReportError("switch 语句的字节码生成暂未实现", node.loc);
}
```

Parser 和 TypeChecker 已完整支持 `switch/case/default`（包括 fallthrough 语义检查），但 BytecodeGen 直接报错，导致包含 switch 的代码无法编译运行。

**影响**: `switch/case` 语言特性完全不可用。

**文档矛盾**: `C_SUBSET_SPEC.md` 明确将 `switch/case/default` 标记为"已支持"。

**修复建议**: 实现 `switch` 的字节码生成。可采用"跳转链"策略：
1. 计算 switch 条件值
2. 依次与每个 case 标签比较（`Eq` + `JumpIfNotZero`）
3. 匹配失败则跳转到 default 或 switch 结束
4. 每个 case 体结束后需要 fallthrough 或 break 处理

---

### P0-4: 字符串字面量内存区固定从 0x1000 开始，可能溢出

**位置**: `native/src/compiler/BytecodeGen.hpp` (第 93 行)

**问题描述**:
```cpp
uint32_t stringMemOffset_ = 0x1000;
```

字符串字面量从固定地址 `0x1000` 开始分配，堆区从 `0x5000` 开始，中间只有 16KB 空间。如果源代码包含大量字符串（如长格式字符串、多个字符串字面量），`stringMemOffset_` 可能超过 `0x5000`，与堆区重叠。

**文档矛盾**: `PHASE3_P0_FIX_LOG.md` 声称"已在 WasmCodeGen 中修复"（将字符串区动态接在全局变量区之后），但迁移到 `BytecodeGen` 后修复丢失。

**影响**: 字符串数据覆盖堆区，导致 malloc 分配的内存数据被意外修改。

**修复建议**: 
1. 将全局变量也放入线性内存（解决 P1-9），或
2. 在 `BytecodeGen::Generate()` 中计算全局变量占用的空间后，将 `stringMemOffset_` 设为全局变量区结束地址；或
3. 在 `VisitStringLiteral` 中添加 `stringMemOffset_ >= 0x5000` 的溢出检查并报错

---

## 4. P1 严重缺陷

### P1-1: 前端 `StepNext` 完全未调用 Native 单步 API

**位置**: `Cide.Client/ViewModels/MainViewModel.cs` (第 125~147 行)

**问题描述**:
```csharp
[RelayCommand]
private void StepNext()
{
    if (TraceEntries.Count == 0)
    {
        ConsoleOutput = "请先运行代码以生成执行轨迹。\n";
        return;
    }
    if (CurrentStepIndex < TraceEntries.Count - 1)
    {
        CurrentStepIndex++;
        var entry = TraceEntries[CurrentStepIndex];
        HighlightedLine = entry.Line;
        ...
    }
}
```

`StepNext` 只是遍历 `RunCode()` 时 `__cide_step` host function 预先记录的 `TraceEntries`。这不是真正的单步执行——它不能逐条语句驱动 VM 执行，也不能在单步过程中观察内存变化。

**影响**: 前端"单步调试"功能是完全虚假的，用户以为在单步执行，实际上只是在播放预录的轨迹。

**修复建议**: 重写 `StepNext` 流程：
1. `Compile()` 成功后保留 `_compiler` 实例（不要立即 Dispose）
2. `StepNext` 调用 `_compiler.StepNext()`
3. 每次返回后更新 `HighlightedLine`、读取当前内存状态、刷新输出

---

### P1-2: `RunCode` 每次运行都销毁 Session

**位置**: `Cide.Client/ViewModels/MainViewModel.cs` (第 64~66 行)

**问题描述**:
```csharp
_compiler?.Dispose();
_compiler = new Core.CompilerService();
```

每次点击"运行"都创建新的 `CompilerService`（进而创建新的 Native Session），旧的 Session 被销毁。这意味着用户无法在 Run 之后进行 StepNext——因为 StepNext 需要同一个 Session 保持 VM 状态。

**影响**: Run 和 StepNext 两个功能在架构上互斥，无法组合使用。

**修复建议**: 
- 分离"编译"和"运行/单步"两个阶段
- 点击"编译"时创建 Session 并编译
- 点击"运行"时在同一会话上调用 `Run()`
- 点击"单步"时在同一会话上调用 `StepNext()`
- Session 生命周期由"停止"或"重新编译"触发销毁

---

### P1-3: 复合赋值运算（`+=`、`-=` 等）完全未实现

**位置**: `native/src/compiler/BytecodeGen.cpp` (第 692~699 行)

**问题描述**:
```cpp
if (node.op != AssignExpr::Op::Assign) {
    Emit(OpCode::Dup, 0, node.loc);
    Emit(OpCode::LoadLocal, localIdx, node.loc);
    Emit(OpCode::Swap, 0, node.loc); // We don't have Swap... need to handle differently
    // TODO: proper compound assignment
}
Emit(OpCode::StoreLocal, localIdx, node.loc);
Emit(OpCode::LoadLocal, localIdx, node.loc);
```

这段代码存在三个问题：
1. 注释说"没有 Swap"，但 `OpCode::Swap` 已定义且可用（`CideVM` 已实现）
2. 即使使用 Swap，栈操作逻辑也是错误的（没有执行 `Add`/`Sub` 等运算）
3. 最终效果是 `a += b` 被编译为 `a = b`

**影响**: `+=`、`-=`、`*=`、`/=`、`%=` 全部产生错误结果。

**修复建议**:
对于 `a += b`，生成正确的字节码序列：
```
GenExpr(b)           // [b]
LoadLocal(a)         // [b, a]
Add                  // [b+a]
StoreLocal(a)        // []
LoadLocal(a)         // [result]  (赋值表达式返回新值)
```

---

### P1-4: 自增自减（`++a`/`a++`/`--a`/`a--`）完全未实现

**位置**: `native/src/compiler/BytecodeGen.cpp` (第 587~595 行)

**问题描述**:
```cpp
case UnaryExpr::Op::PreInc:
case UnaryExpr::Op::PostInc:
case UnaryExpr::Op::PreDec:
case UnaryExpr::Op::PostDec:
    GenExpr(*node.operand);
    Emit(OpCode::PushConst, 1, node.loc);
    Emit(OpCode::Add, 0, node.loc);  // ❌ 只做了加法，没有存回变量！
    break;
```

问题：
1. 没有将新值存回变量（缺少 `StoreLocal`/`StoreMem`）
2. 没有区分 `PreInc`（返回新值）和 `PostInc`（返回旧值）
3. `PreDec`/`PostDec` 使用了 `Add` 而不是 `Sub`

**影响**: `++`/`--` 完全不工作。

**修复建议**:
对于 `++a`（PreInc）：
```
LoadLocal(a)    // [old]
PushConst 1     // [old, 1]
Add             // [new]
Dup             // [new, new]
StoreLocal(a)   // [new]   (返回新值)
```

对于 `a++`（PostInc）：
```
LoadLocal(a)    // [old]
Dup             // [old, old]
PushConst 1     // [old, old, 1]
Add             // [old, new]
StoreLocal(a)   // [old]   (返回旧值)
```

---

### P1-5: 全局变量存储在独立向量，不在线性内存

**位置**: `native/src/vm/CideVM.hpp` (第 109 行)

**问题描述**:
```cpp
std::vector<int32_t> globals_;  // 独立存储
std::vector<uint8_t> memory_;   // 线性内存（256KB）
```

`CideVM` 使用独立的 `globals_` 向量存储全局变量，而非线性内存 `memory_`。这导致：

1. **全局变量取地址无法工作**: BytecodeGen 直接报错 `"全局变量取地址暂不支持"`
2. **scanf 全局变量失败**: `scanf("%d", &global_var)` 无法写入全局变量
3. **内存视图中看不到全局变量**: `cide_memory_region_count` 只返回堆区域
4. **与文档卖点矛盾**: `DESIGN.md` 强调"局部变量映射到线性内存支持 `&x`"，但全局变量被遗漏

**影响**: 全局变量的指针操作和内存可视化完全不可用。

**修复建议**: 将全局变量也放入线性内存：
1. 在 `memory_` 的 `0x1000` 区域分配全局变量空间
2. `LoadGlobal`/`StoreGlobal` 改为对 `memory_` 的读写
3. BytecodeGen 中全局变量初始化直接写入 `memory_`（通过 C API 在编译后复制）

---

### P1-6: `cide_run` 与 `cide_step_next` VM 初始化代码大量重复

**位置**: `native/src/capi/cide_capi.cpp` (第 291~488 行 和 第 490~614 行)

**问题描述**:
两个函数中有几乎完全相同的代码块：
- VM Reset + LoadProgram + SetGlobals
- 函数注册（`RegisterFunction`）
- 字符串字面量复制到内存
- Host 函数注册（导致 P0-2）
- 如果某个 Host 函数行为需要修改，必须在两处同步修改

**影响**: 维护困难，极易引入不一致（已导致 printf/scanf 在单步中不可用）。

**修复建议**: 提取 `SetupVM(CideSession* s)` 公共函数，包含所有 VM 初始化逻辑。

---

### P1-7: `cide_memory_get_value` 边界失败返回"成功"

**位置**: `native/src/capi/cide_capi.cpp` (第 688~699 行)

**问题描述**:
```cpp
extern "C" int cide_memory_get_value(CideSession* s, unsigned int addr, int* out_val) {
    if (!s || !out_val) return -1;
    uint8_t* mem = s->vm.GetMemory();
    uint32_t memSize = s->vm.GetMemorySize();
    if (mem && addr + 4 <= memSize) {
        *out_val = ...;
        return 0;
    }
    *out_val = 0;
    return 0;  // ❌ 应该返回 -1
}
```

边界检查失败时返回 0（成功），但 `*out_val = 0`。调用者无法区分"内存值确实是 0"和"读取失败"。

**影响**: 内存视图中越界地址会显示为 0，误导用户。

**修复建议**: 边界失败时返回 `-1`。

---

### P1-8: `printf_0` Host 函数残留调试输出

**位置**: `native/src/capi/cide_capi.cpp` (第 393 行)

**问题描述**:
```cpp
printf("[DEBUG printf_0] addr=%d out='%s'\n", fmtAddr, out.c_str());
```

生产代码中残留 `printf` 调试输出，会污染控制台/日志。

**影响**: 输出中出现 `[DEBUG printf_0]` 等调试信息，影响用户体验。

**修复建议**: 删除该调试输出。

---

### P1-9: `VisitInitList` 总是返回 PushConst 0

**位置**: `native/src/compiler/BytecodeGen.cpp` (第 795~797 行)

**问题描述**:
```cpp
void BytecodeGen::VisitInitList(InitListExpr& node) {
    Emit(OpCode::PushConst, 0, node.loc);
}
```

`VisitInitList` 只在 `VisitVarDecl` 的数组初始化中被正确处理。如果初始化列表出现在其他上下文（如赋值右侧、函数参数），会生成 `PushConst 0` 的错误代码。

**影响**: 非声明场景的初始化列表使用会产生错误结果。

**修复建议**: `VisitInitList` 应该报错（当前子集不支持非声明场景的初始化列表），或者正确实现数组赋值语义。

---

### P1-10: `CideSession` 残留 wasm3 时代线程字段

**位置**: `native/src/capi/cide_capi.cpp` (第 68~87 行)

**问题描述**:
```cpp
struct CideRuntimeState {
    ...
    std::atomic<bool> stepResume{false};  // 已无用
    ...
};

struct CideSession {
    ...
    std::atomic<bool> stepCallDone{false};  // 永远不会被设置
    std::thread* stepThread = nullptr;      // 永远不会被创建
};
```

这些字段是 wasm3 时代多线程单步调试的遗留。CideVM 架构下单步是同步的，不需要线程。

`cide_session_destroy` 中还有针对 `stepThread` 的清理逻辑：
```cpp
if (s->stepThread && s->stepThread->joinable()) {
    s->runtime.stepResume = true;
    s->stepThread->join();
    delete s->stepThread;
    s->stepThread = nullptr;
}
```

这段代码是死代码（`stepThread` 始终为 `nullptr`），但会造成维护混淆。

**影响**: 新开发者会误以为后端使用了多线程单步架构；结构体占用不必要的内存。

**修复建议**: 删除 `stepThread`、`stepCallDone`、`stepResume` 及相关清理逻辑。

---

### P1-11: SourceMap 从未被填充

**位置**: `native/src/capi/cide_capi.cpp` (第 756~776 行)

**问题描述**:
1. `CideCompileState` 声明了 `std::vector<std::pair<uint32_t, cide::SourceLoc>> sourceMap`
2. `cide_compile` 中从未填充 `sourceMap`
3. `BytecodeGen` 中没有生成 source map 的逻辑
4. `cide_sourcemap_lookup` 虽然存在，但数据为空，始终返回失败

**影响**: 运行时错误无法精确映射到源码行列号（CideVM 的 `Instruction.loc` 有行号，但没有通过 SourceMap API 暴露）。

**修复建议**:
1. 在 `BytecodeGen::Emit()` 中记录 `ip -> loc` 映射
2. `cide_compile` 将映射复制到 `CideCompileState.sourceMap`
3. 参数名 `wasm_offset` 改为 `bytecode_offset`

---

### P1-12: `scanf` Host 函数忽略格式字符串

**位置**: `native/src/capi/cide_capi.cpp` (第 452~469 行)

**问题描述**:
```cpp
s->vm.RegisterHostFunction(20, [](std::vector<int32_t>& stack, cide::CideVM* vm, void* ud) {
    auto* c = static_cast<HostCtx*>(ud);
    int32_t fmtAddr = stack.back(); stack.pop_back();
    int32_t p1 = stack.back(); stack.pop_back();
    (void)fmtAddr;  // ❌ 忽略格式字符串！
    int value = 0;
    if (c->session->runtime.inputIndex < c->session->runtime.inputLines.size()) {
        value = std::atoi(...);
    }
    // 总是写入 4 字节 int
    mem[p1] = ...;
    ...
});
```

`(void)fmtAddr` 明确忽略了格式字符串。无论传入 `"%d"`、`"%c"` 还是其他格式，行为都是读取一个整数并写入 4 字节。

**影响**: `scanf("%c", &c)` 会覆盖 4 字节内存（可能破坏相邻变量）。

**修复建议**: 至少解析格式字符串的第一个格式说明符：
- `%d` → 读取 int，写入 4 字节
- `%c` → 读取 char，写入 1 字节（通过 `StoreMemByte` 语义）

---

## 5. P2 中等缺陷

### P2-1: 全局变量初始化仅支持 `Literal`

**位置**: `native/src/compiler/BytecodeGen.cpp` (第 106~116 行)

**问题描述**:
```cpp
for (auto& g : program.globals) {
    globalIndices_[g.name] = nextGlobalIdx_;
    globalTypes_[g.name] = g.type;
    int32_t initVal = 0;
    if (g.init && g.init->kind == ExprKind::Literal) {
        initVal = static_cast<LiteralExpr&>(*g.init).value;
    }
    globalsInit_.push_back(initVal);
    nextGlobalIdx_++;
}
```

仅当全局初始化是 `Literal` 时才取值，其他情况（如 `int a = 1 + 2;`、`int b = sizeof(int);`）被静默忽略并设为 0。

**影响**: 复杂全局初始化产生错误结果，且不报错。

**修复建议**: 对非 `Literal` 初始化报错，或在字节码生成阶段计算常量表达式。

---

### P2-2: 指针运算被实现，与文档矛盾

**位置**: `native/src/compiler/BytecodeGen.cpp` (第 514~534 行)

**问题描述**:
```cpp
case BinaryExpr::Op::Add:
    if (leftIsPointer && !rightIsPointer) {
        Emit(OpCode::PushConst, 4, node.loc);
        Emit(OpCode::Mul, 0, node.loc);
        Emit(OpCode::Add, 0, node.loc);  // ptr + int
    }
```

`C_SUBSET_SPEC.md` 明确声明：
> "教学子集不支持指针运算，请使用数组索引 arr[i] 代替"

但 BytecodeGen 完整实现了 `ptr + int` 和 `ptr - int` 的指针运算。

**影响**: 文档误导用户；教学中可能让学生形成不良习惯。

**修复建议**: 
- **方案 A**: 在 TypeChecker 中增加错误检查，禁止指针运算（但会破坏已有测试 `Phase3Batch1Test` 中的 `"Pointer to array elem"` 测试，该测试使用 `*(p + 2)`）
- **方案 B**: 更新文档，承认指针运算是支持的（教学子集实际上需要它来实现数组遍历）

> 推荐方案 B，因为数组到指针的退化（`arr` → `int*`）和指针算术是 C 教学中不可避免的概念。

---

### P2-3: 注释与代码矛盾："没有 Swap"

**位置**: `native/src/compiler/BytecodeGen.cpp` (第 696 行)

**问题描述**:
```cpp
Emit(OpCode::Swap, 0, node.loc); // We don't have Swap... need to handle differently
```

实际上 `OpCode::Swap` 已定义（`OpCode.hpp` 第 19 行），且 `CideVM::Step()` 已实现 Swap 指令。

**影响**: 误导开发者，增加维护成本。

**修复建议**: 删除错误注释。

---

### P2-4: `ParsePrimary` 中字符串引号重复处理

**位置**: `native/src/compiler/Parser.cpp` (第 714~722 行)

**问题描述**:
```cpp
if (Match(TokenType::String)) {
    std::string raw = tokens_[pos_ - 1].text;
    std::string value;
    if (raw.size() >= 2 && raw.front() == '"' && raw.back() == '"') {
        value = raw.substr(1, raw.length() - 2);
    } else {
        value = raw;
    }
    return std::make_unique<StringLiteralExpr>(std::move(value));
}
```

Lexer 的 `StringLiteral()` 已经将转义序列解析并去掉了引号，`Token.text` 存储的是解析后的值（不含引号）。Parser 再次检查引号是多余的。

**影响**: 轻微，如果 Lexer 行为变更可能导致 bug。

**修复建议**: 直接使用 `tokens_[pos_ - 1].text`。

---

### P2-5: `Synchronize()` 错误恢复不完整

**位置**: `native/src/compiler/Parser.cpp` (第 57~74 行)

**问题描述**:
```cpp
void Parser::Synchronize() {
    while (!IsAtEnd()) {
        if (Previous().type == TokenType::Semicolon) return;
        switch (Current().type) {
            case TokenType::Int:
            case TokenType::Void:
            case TokenType::If:
            case TokenType::While:
            case TokenType::For:
            case TokenType::Return:
            case TokenType::Struct:
            case TokenType::RBrace:
                return;
            default:
                Advance();
        }
    }
}
```

`Synchronize()` 未包含 `Do`、`Switch`、`Case`、`Default`、`Break`、`Continue`、`Typedef`、`Enum` 等可作为同步点的关键字。

**影响**: 语法错误恢复时可能跳过过多的 token，导致级联错误报告。

**修复建议**: 将所有语句起始关键字加入同步点列表。

---

### P2-6: `TypeChecker::VisitStringLiteral` 类型设置不完整

**位置**: `native/src/compiler/TypeChecker.cpp` (第 537~539 行)

**问题描述**:
```cpp
void TypeChecker::VisitStringLiteral(StringLiteralExpr& node) {
    node.type = Type{TypeKind::Pointer, "char"};
}
```

`Type` 的结构为 `{kind, name, arraySize, baseKind}`。这里使用 aggregate initialization 设置 `kind=Pointer, name="char"`，但 `baseKind` 为默认的 `TypeKind::Void`。

正确的 `char*` 类型应为：
```cpp
node.type = Type{TypeKind::Pointer, "char", 0, TypeKind::Char};
```

**影响**: 如果后续代码依赖 `baseKind` 判断指针指向类型，可能产生错误。

**修复建议**: 显式设置 `baseKind`。

---

### P2-7: `Phase3Batch3Test` 使用指针运算

**位置**: `native/tests/Phase3Batch3Test.cpp` (第 13 行)

**问题描述**:
```cpp
*(p + 1) = 200;  // 指针运算
```

测试代码使用了 `p + 1` 指针运算，这与 `C_SUBSET_SPEC.md` 中"不支持指针运算"的声明矛盾。

**影响**: 如果按文档实现禁止指针运算，此测试会失败。

**修复建议**: 将测试改为使用数组索引：`p[1] = 200;`。

---

### P2-8: `test_new_features.cpp` 包含 switch 测试但字节码未实现

**位置**: `native/tests/test_new_features.cpp` (第 69~73 行)

**问题描述**:
```cpp
test("switch_basic", "int main() { int x = 2; ... switch (x) { case 1: ... } return r; }", 20);
```

该测试调用 `cide_compile` + `cide_run`，但 BytecodeGen 对 `switch` 直接报错。此测试在当前代码下必然失败。

**影响**: 测试套件自身包含注定失败的测试，降低测试可信度。

**修复建议**: 暂时注释掉 switch 相关测试，或优先实现 switch 字节码生成。

---

### P2-9: 测试解析返回值方式脆弱

**位置**: 多个测试文件

**问题描述**:
```cpp
size_t pos = out.find("return value:");
if (pos == std::string::npos) pos = out.find("\xe8\xbf\x94\xe5\x9b\x9e\xe5\x80\xbc\xef\xbc\x9a");
if (pos != std::string::npos) actual = std::atoi(out.c_str() + pos + 12);
```

测试通过字符串匹配解析返回值。如果 `cide_run` 的输出格式变更（如 P3-7 移除自动追加的返回值信息），所有测试都会失败。

**影响**: 测试脆弱，与输出格式强耦合。

**修复建议**: 
- 方案 A: 增加 `cide_get_return_value` C API
- 方案 B: 让 `cide_run` 的返回值直接作为 int 返回（当前返回 0/-1 表示成功/失败）

---

### P2-10: `TranslateRuntimeError` 遗留函数

**位置**: `native/src/capi/cide_capi.cpp` (第 98~136 行)

**问题描述**:
`TranslateRuntimeError` 将英文错误翻译为中文，但 CideVM 已经直接生成中文诊断信息（`FormatDivZeroError`、`FormatBoundsError` 等）。此函数仅在 `cide_get_runtime_error` 中未被实际调用的路径存在。

**影响**: 死代码，维护负担。

**修复建议**: 删除 `TranslateRuntimeError` 函数。

---

### P2-11: `StepEvent` 在 `Run()` 模式下效率损失

**位置**: `native/src/vm/CideVM.cpp` (第 427~434 行)

**问题描述**:
```cpp
case OpCode::StepEvent: {
    currentLine_ = inst.operand;
    if (paused_) {
        return StepResult::Paused;
    }
    break;
}
```

`Run()` 模式下 `paused_` 始终为 `false`，但每条语句前的 `StepEvent` 都会检查 `paused_` 状态。

**影响**: 轻微性能损失（教学代码很短，影响可忽略）。

**修复建议**: 可优化为编译期开关或运行时标志，跳过 `Run()` 模式下的 `StepEvent` 检查。

---

### P2-12: `MainViewModel` 示例代码使用 `__cide_output`

**位置**: `Cide.Client/ViewModels/MainViewModel.cs` (第 22~31 行)

**问题描述**:
```csharp
private string _sourceCode = """
    int main() {
        ...
        __cide_output(sum);
        return sum;
    }
    """;
```

示例代码使用内部 host function `__cide_output` 而非用户可见的 `printf`。用户看到示例代码中的 `__cide_output` 会感到困惑，因为这不是标准 C 函数。

**影响**: 教学场景中示例代码不标准，增加认知负担。

**修复建议**: 将示例代码改为使用 `printf("%d\n", sum);`。

---

### P2-13: `CompilerService` 未暴露诊断 API

**位置**: `Cide.Client/Core/CompilerService.cs`

**问题描述**:
`CompilerService` 没有封装 `cide_diagnostic_count` 和 `cide_diagnostic_get` C API。前端无法获取结构化的诊断信息（错误码、severity、修复建议）。

**影响**: 前端只能获取纯文本错误字符串，无法实现分级诊断（L1/L2/L3）和一键修复。

**修复建议**: 在 `CompilerService` 中添加：
```csharp
public int GetDiagnosticCount() => NativeMethods.cide_diagnostic_count(_session);
public Diagnostic GetDiagnostic(int index) { ... }
```

---

### P2-14: `cide_run` 自动追加返回值信息

**位置**: `native/src/capi/cide_capi.cpp` (第 485 行)

**问题描述**:
```cpp
s->runtime.outputLines.push_back("程序运行完成，返回值：" + std::to_string(retValue) + "\n");
```

这行中文信息自动追加到所有输出末尾。虽然有用，但：
1. 不是用户代码的输出
2. 与测试解析逻辑耦合（P2-9）
3. 用户可能不需要或希望自定义此消息

**影响**: 输出被污染；国际化困难。

**修复建议**: 增加独立的 `cide_get_return_value` API，或让前端自行组装提示信息。

---

## 6. P3 优化建议

### P3-1: CMakeLists.txt 使用 `file(GLOB_RECURSE)`

**位置**: `native/CMakeLists.txt` (第 26~29 行)

**问题描述**:
```cmake
file(GLOB_RECURSE CIDE_SOURCES "src/*.cpp" "src/*.c")
```

CMake 反模式：新增/删除源文件不会自动触发重新配置，需要手动删除 build 目录或 touch CMakeLists.txt。

**修复建议**: 显式列出所有源文件。

---

### P3-2: 测试目标逐个手动添加

**位置**: `native/CMakeLists.txt` (第 66~112 行)

**问题描述**:
6 个测试目标使用几乎完全相同的代码重复添加。

**修复建议**: 使用 CMake 循环：
```cmake
foreach(test_name phase2_regression_test phase3_batch1_test ...)
    add_executable(${test_name} tests/${test_name}.cpp)
    target_link_libraries(${test_name} PRIVATE cide_native)
    ...
endforeach()
```

---

### P3-3: `build.ps1` 硬编码 `MinGW Makefiles`

**位置**: `build.ps1` (第 64 行)

**问题描述**:
```powershell
cmake .. -G "MinGW Makefiles" -DCMAKE_BUILD_TYPE=$Configuration
```

在纯 MSVC 环境（Visual Studio 开发者命令提示符）中，`MinGW Makefiles` 生成器不可用。

**修复建议**: 检测环境并选择合适的生成器：
```powershell
if (Get-Command "ninja" -ErrorAction SilentlyContinue) {
    cmake .. -G "Ninja" ...
} elseif ($env:VisualStudioVersion) {
    cmake .. ...  # 使用默认生成器（Visual Studio）
} else {
    cmake .. -G "MinGW Makefiles" ...
}
```

---

### P3-4: `CideVM::Step()` switch-case 可优化

**位置**: `native/src/vm/CideVM.cpp` (第 220~435 行)

**问题描述**:
`Step()` 使用大型 `switch` 语句分发指令。对于 ~30 条指令的解释器，现代编译器通常能优化为跳转表，但可进一步确保性能。

**修复建议**: 考虑使用函数指针数组（dispatch table）替代 switch，或添加 `[[clang::always_inline]]` 等提示。

---

### P3-5: 堆内存 `free` 后不复用

**位置**: `native/src/capi/cide_capi.cpp` (第 354~366 行)

**问题描述**:
`malloc` 使用简单的 bump allocator，`heapOffset` 只增不减。`free` 只标记 region 为 freed，但不回收空间。

**影响**: 大量 `malloc/free` 循环会导致堆耗尽。

**修复建议**: 实现简单的首次适应（first-fit）分配器，维护空闲块链表。

---

### P3-6: `BeautifyCompileError` 字符串匹配条件脆弱

**位置**: `native/src/capi/cide_capi.cpp` (第 162~197 行)

**问题描述**:
`BeautifyCompileError` 使用子字符串匹配来决定美化策略：
```cpp
if (raw.find("预期") != std::string::npos && raw.find(";'") != std::string::npos)
```

如果编译器错误消息格式微调，这些匹配条件会失效。

**修复建议**: 使用错误码（`LexerError.code`、`ParseError.code`、`TypeError.code`）进行匹配，而非字符串。

---

### P3-7: 缺少单步调试自动化测试

**位置**: `native/tests/`

**问题描述**:
所有测试都使用 `cide_run`，没有测试调用 `cide_step_next`。

**修复建议**: 添加 `Phase3StepTest.cpp`，验证：
1. 首次 `cide_step_next` 在第一条语句暂停
2. 后续 `cide_step_next` 逐条语句推进
3. 单步过程中变量值正确变化
4. 单步到结束正确返回

---

### P3-8: `CODE_REVIEW_PLAN.md` 状态未更新

**位置**: `docs/CODE_REVIEW_PLAN.md`

**问题描述**:
该文档声称所有 Phase A/B/C 已完成，但其中提到的 `Parser pos_--` 清理实际上在 Parser.cpp 中并不存在（当前代码使用 `checkpoint` 回退机制，不是 `pos_--`）。文档与代码状态不一致。

---

### P3-9: `cide_diagnostic_get` 参数设计问题

**位置**: `native/include/cide_capi.h` (第 84~88 行)

**问题描述**:
```cpp
CIDE_API void cide_diagnostic_get(
    CideSession* s, int index,
    int* line, int* column, int* error_code, int* severity,
    char* message, int msg_size,
    char* fix_suggestion, int fix_size);
```

参数过多，且 `message` 和 `fix_suggestion` 需要调用者预分配缓冲区。没有获取所需缓冲区大小的 API。

**修复建议**: 参考 Windows API 设计模式：
```cpp
int cide_diagnostic_message_length(CideSession* s, int index);  // 获取所需长度
```

---

### P3-10: `Ast.hpp` 中 `StringLiteralExpr` 类型构造

**位置**: `native/src/compiler/Ast.hpp` (第 148~155 行)

**问题描述**:
```cpp
explicit StringLiteralExpr(std::string v)
    : Expr(ExprKind::StringLiteral), value(std::move(v)) {
    type = Type{TypeKind::Pointer, "char"};
}
```

与 P2-6 类似，`baseKind` 未设置。

---

### P3-11: `cide_set_input` 使用分号分隔

**位置**: `native/src/capi/cide_capi.cpp` (第 808~824 行)

**问题描述**:
```cpp
for (const char* p = input; *p != '\0'; p++) {
    if (*p == ';') {
        s->runtime.inputLines.push_back(current);
        current.clear();
    }
```

输入行使用分号 `;` 分隔。如果用户输入的数据包含分号（如字符串 `"a;b"`），会被错误分割。

**修复建议**: 使用更稳健的分隔方式，如换行符 `\n` 或长度前缀协议。

---

### P3-12: `CideSession` 内存区域命名不友好

**位置**: `native/src/capi/cide_capi.cpp` (第 361 行)

**问题描述**:
```cpp
c->session->memory.regions.push_back({addr, size, "heap_" + std::to_string(addr), "int", true, false});
```

`malloc` 分配的内存区域命名为 `"heap_0x5000"` 等，对前端展示不友好。

**修复建议**: 维护一个分配序号，生成 `"heap_1"`、`"heap_2"` 等更友好的名称。

---

## 7. 文档与实现不一致清单

| 文档 | 声称 | 实际实现 | 差异严重程度 |
|:---|:---|:---|:---|
| `C_SUBSET_SPEC.md` | `switch/case/default` 已支持 | BytecodeGen 直接报错 | 🔴 高 |
| `C_SUBSET_SPEC.md` | 不支持指针运算 | BytecodeGen 完整实现 `ptr ± int` | 🟠 中 |
| `DESIGN.md` | 自研 VM 零线程 | `CideSession` 残留线程字段 | 🟡 低 |
| `PHASE3_P0_FIX_LOG.md` | 字符串内存区已修复 | BytecodeGen 仍硬编码 0x1000 | 🔴 高 |
| `PHASE3_P0_FIX_LOG.md` | 全局变量取地址已修复 | BytecodeGen 仍报错"暂不支持" | 🔴 高 |
| `CODE_REVIEW_PLAN.md` | Parser pos_-- 已清理 | 当前代码从未使用 pos_-- | 🟢 低 |
| `DESIGN.md` | 单步调试已完成 | `cide_step_next` 逻辑损坏 | 🔴 高 |
| `DESIGN.md` | `&x` 取地址支持 | 全局变量取地址不支持 | 🟠 中 |

---

## 8. 模块交叉影响矩阵

```
                        前端VM   C API   BytecodeGen   Parser   TypeChecker   C#前端   测试
P0-1 step_next 逻辑      ●        ●                                                 
P0-2 step缺少printf               ●                                                 
P0-3 switch未实现                          ●                                         
P0-4 字符串溢出                        ●   ●                                         
P1-1 前端StepNext假调试                                                    ●        
P1-2 Run销毁Session                                                        ●        
P1-3 复合赋值未实现                        ●                                         
P1-4 自增自减未实现                        ●                                         
P1-5 全局变量不在内存     ●        ●   ●                                         
P1-6 run/step重复               ●                                                 
P1-7 memory_get_value         ●                                                 
P1-8 printf调试输出           ●                                                 
P1-9 InitList返回0                       ●                                         
P1-10 残留线程字段        ●        ●                                                 
P1-11 SourceMap空                 ●   ●                                         
P1-12 scanf忽略格式           ●                                                 
P2-1 全局初始化仅Literal                 ●                                         
P2-2 指针运算矛盾                        ●        ●                                 
P2-7 Batch3指针运算                                                 ●                
P2-8 test_new_features                                              ●                
```

（● 表示该模块需要修改）

---

## 9. 推荐修复路线图

### 阶段一：P0 致命修复（1~2 天）

1. **修复 `cide_step_next` 单步逻辑** (P0-1)
   - 在 `CideVM` 增加 `stepEventHit_` 标志
   - 修改 `cide_step_next` 继续逻辑
   
2. **统一 Host 函数注册** (P0-2 + P1-6)
   - 提取 `SetupVM()` 公共函数
   - 确保 `cide_run` 和 `cide_step_next` 使用相同的 Host 函数集

3. **修复字符串内存区溢出** (P0-4)
   - 添加溢出检查，或动态计算字符串起始地址

### 阶段二：语言特性补齐（2~3 天）

4. **实现 `switch/case` 字节码生成** (P0-3)
5. **实现复合赋值** (P1-3)
6. **实现自增自减** (P1-4)
7. **修复全局变量取地址** (P1-5) - 将全局变量放入线性内存

### 阶段三：前端架构修复（1~2 天）

8. **重写 `StepNext` 流程** (P1-1 + P1-2)
   - 分离编译/运行/单步生命周期
   - `StepNext` 真正调用 `cide_step_next`

### 阶段四：清理与优化（1~2 天）

9. **清理残留线程字段** (P1-10)
10. **修复 `cide_memory_get_value` 返回值** (P1-7)
11. **删除 `printf_0` 调试输出** (P1-8)
12. **填充 SourceMap** (P1-11)
13. **修复测试** (P2-7, P2-8, P2-9)

---

## 10. 附录：详细代码引用

### A. `cide_step_next` 损坏逻辑
```cpp
// native/src/capi/cide_capi.cpp:598-613
s->vm.Resume();
while (true) {
    auto result = s->vm.Step();
    if (result == cide::CideVM::StepResult::Paused) {
        return 0;  // 永远不会触发
    }
    ...
}
```

### B. `CideVM::StepEvent` 处理
```cpp
// native/src/vm/CideVM.cpp:427-434
case OpCode::StepEvent: {
    currentLine_ = inst.operand;
    if (paused_) {
        return StepResult::Paused;
    }
    break;
}
```

### C. BytecodeGen `switch` 未实现
```cpp
// native/src/compiler/BytecodeGen.cpp:453-456
void BytecodeGen::VisitSwitch(SwitchStmt& node) {
    ReportError("switch 语句的字节码生成暂未实现", node.loc);
}
```

### D. 前端假单步调试
```csharp
// Cide.Client/ViewModels/MainViewModel.cs:125-147
[RelayCommand]
private void StepNext()
{
    if (CurrentStepIndex < TraceEntries.Count - 1)
    {
        CurrentStepIndex++;
        var entry = TraceEntries[CurrentStepIndex];
        HighlightedLine = entry.Line;
        ...
    }
}
```

### E. 复合赋值 TODO
```cpp
// native/src/compiler/BytecodeGen.cpp:692-699
if (node.op != AssignExpr::Op::Assign) {
    Emit(OpCode::Dup, 0, node.loc);
    Emit(OpCode::LoadLocal, localIdx, node.loc);
    Emit(OpCode::Swap, 0, node.loc); // We don't have Swap...
    // TODO: proper compound assignment
}
```

### F. 自增自减无存储
```cpp
// native/src/compiler/BytecodeGen.cpp:587-595
case UnaryExpr::Op::PreInc:
case UnaryExpr::Op::PostInc:
    GenExpr(*node.operand);
    Emit(OpCode::PushConst, 1, node.loc);
    Emit(OpCode::Add, 0, node.loc);
    break;  // 没有 Store 回变量！
```

---

> **报告完成**。本报告基于 2026-04-27 的代码快照。建议在每次重大变更后重新运行全盘分析。
