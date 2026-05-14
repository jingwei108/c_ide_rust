# P3 清理与优化归档 — 2026-05-04

## 背景

P3 阶段聚焦于 **可维护性与性能优化**。核心痛点：
- `MainViewModel` 是"上帝类"（Desktop 960 行 / Maui ~800 行），承担编译、执行、调试、可视化、UI 状态等十几种职责
- 两个平台（Avalonia Desktop / MAUI）存在大量重复代码
- `ApplyFix` 依赖中文字符串硬匹配，无法精确替换
- Android 16 要求 16KB page size，`.so` 触发 XA0141 警告

## 执行成果

### 1. MainViewModel 上帝类拆分

| 指标 | Desktop | Maui | 降幅 |
|---|---|---|---|
| 原始行数 | 960 | ~800 | — |
| 当前行数 | **589** | **541** | **-39% / -32%** |

**提取到 `Cide.Client.Shared` 的服务（7 个）**

| 服务 | 文件 | 原职责 | 备注 |
|---|---|---|---|
| `CompilerSessionService` | `Core/CompilerSessionService.cs` | `EnsureCompiled`、断点同步、`_compiler` 生命周期 | 新提取 |
| `DiagnosticService` | `Core/DiagnosticService.cs` | `LoadDiagnostics`、知识卡片匹配 | 新提取 |
| `DebugDataService` | `Core/DebugDataService.cs` | `LoadVariables`/`LoadCallStack`/`LoadLinkedListGraph`/`EvaluateWatchExpression`/`LoadMemoryRegions`/`LoadAlgorithmMatches` | 新提取 |
| `ExecutionService` | `Core/ExecutionService.cs` | `StepNext` / `RunFullSpeed` 纯 VM 执行逻辑 | 新提取 |
| `VisualizationService` | `Core/VisualizationService.cs` | `BuildArrayElements`、`DetectSwapIndices` | Round 1 已提取 |
| `CodeFixService` | `Core/CodeFixService.cs` | `ApplyFix` 自动修复逻辑 | Round 1 已提取 |
| `CodeTemplate.GetDefaultTemplates()` | `ViewModels/CodeTemplate.cs` | 8 个内置代码模板初始化 | 新提取，消除 30 行 ×2 重复 |

**MainViewModel 内部辅助方法（4 个）**

| 方法 | 职责 | 消除重复 |
|---|---|---|
| `ResetExecutionState()` | 执行前 UI 状态重置 | `RunCodeAsync` 开头 7 行 ×2 |
| `FinishExecution()` | 执行后并发标志/命令状态重置 | `finally` 块 3 行 ×2 |
| `PresentCompileError()` | 编译错误 UI 展示 | `RunCodeAsync` / `StepNext` 中 ~10 行 ×2 |
| `PresentRuntimeError()` | 运行时错误 UI 展示 | `RunCodeAsync` / `DoSingleStep` 中 ~8 行 ×2 |

**Forwarding 类型兼容性处理**

Desktop 项目保留了 `Cide.Client.ViewModels` 命名空间中的 forwarding record/class（`CallStackFrame`、`WatchExpression`、`ArrayVisualization`、`PointerViewModel`、`TraceEntry`、`ArrayElementVisual`、`CodeTemplate`、`GraphNodeViewModel`），确保现有 XAML `x:DataType` 绑定零改动。为每个 forwarding 类型添加了接受 shared 类型的构造函数，兼容 `DebugDataService` 返回的 shared 类型实例。

### 2. Android 16 page-size 警告修复

**问题**：XA0141 — `libcide_native.so` 的 ELF page size 为 4KB（`0x1000`），Android 16 将要求 16KB。

**修复**：
1. `native/CMakeLists.txt` 中为所有 Android ABI 添加 linker flag：
   ```cmake
   set(CMAKE_SHARED_LINKER_FLAGS "${CMAKE_SHARED_LINKER_FLAGS} -Wl,-z,max-page-size=16384")
   ```
2. 清理并重新构建 `arm64-v8a` 和 `armeabi-v7a`：
   ```bash
   cmake .. -G Ninja -DCMAKE_TOOLCHAIN_FILE=$NDK/build/cmake/android.toolchain.cmake \
            -DANDROID_ABI=arm64-v8a -DANDROID_PLATFORM=android-21 \
            -DCMAKE_BUILD_TYPE=Debug -DCIDE_BUILD_TESTS=OFF
   cmake --build . --parallel
   ```
3. 验证结果（`llvm-readelf -l lib/libcide_native.so`）：
   - `arm64-v8a`: `Align = 0x4000` ✅ (16KB)
   - `armeabi-v7a`: `Align = 0x4000` ✅ (16KB)

**结果**：Maui 构建从 **1 个 XA0141 警告** → **0 警告** ✅

### 3. ApplyFix 结构化修复基础设施

**问题**：`CodeFixService` 依赖中文字符串子串匹配（`fix.Contains("=' 改为 '=='"`），无法处理嵌套括号、多行表达式等复杂情况。

**修复**：
1. **C++ 后端**
   - `CideSession.hpp`：新增 `CideFixKind` 枚举（`None`/`ReplaceText`/`InsertText`/`DeleteText`/`ManualHint`），扩展 `CideDiagnostic` 添加 `fixKind`/`replaceStartLine`/`replaceStartColumn`/`replaceEndLine`/`replaceEndColumn`/`replacementText`
   - `cide_capi.h`：新增 `cide_diagnostic_get_fix` API
   - `cide_capi.cpp`：实现 `cide_diagnostic_get_fix`，返回结构化 fix 数据

2. **C# 前端**
   - `CompilerService.cs`：`Diagnostic` record 新增 `FixKind`、`ReplaceStartLine`、`ReplaceStartColumn`、`ReplaceEndLine`、`ReplaceEndColumn`、`ReplacementText`
   - `NativeMethods.cs`：添加 `cide_diagnostic_get_fix` P/Invoke 声明
   - `CompilerService.GetDiagnostic`：同时调用 `cide_diagnostic_get_fix` 读取结构化数据
   - `CodeFixService`：重构为两阶段 — 优先尝试 `ApplyStructuredReplace`（基于 `FixKind`/`ReplaceRange`），失败则 fallback 到 `ApplyLegacyFix`（原有字符串匹配）

**当前状态**：基础设施已就绪。后端 `Parser`/`TypeChecker` 尚未填充精确修复位置（所有 diagnostic 的 `fixKind` 默认 `None`），当前仍通过 fallback 字符串匹配工作。当后端未来在报错点记录精确修复范围后，前端将自动使用结构化修复。

### 4. 其他修复

- **`_lastArrayValues` 字典泄漏**：在 `LoadVariables` 末尾清理 stale key，防止无界增长
- **魔法数字集中化**：`MemoryLayoutConstants.cs` / `Constants.cs` 统一 `0x1000`/`0x40000`/`4`/`0`/`4` 等常量
- **CMakeLists.txt 冗余空行清理**

## 文件变更清单

### 新增文件

```
Cide.Client.Shared/Core/CompilerSessionService.cs
Cide.Client.Shared/Core/DebugDataService.cs
Cide.Client.Shared/Core/DiagnosticService.cs
Cide.Client.Shared/Core/ExecutionService.cs
Cide.Client.Shared/Core/MemoryLayoutConstants.cs  (后续删除，合并到 Constants.cs)
```

### 修改文件

```
native/CMakeLists.txt                              (+ Android 16KB page-size linker flag)
native/include/cide_capi.h                         (+ cide_diagnostic_get_fix)
native/src/capi/CideSession.hpp                    (+ CideFixKind, CideDiagnostic 扩展)
native/src/capi/cide_capi.cpp                      (+ cide_diagnostic_get_fix 实现)

Cide.Client.Shared/Core/CodeFixService.cs          (重构：结构化修复 + fallback)
Cide.Client.Shared/Core/CodeTemplate.cs            (+ GetDefaultTemplates())
Cide.Client.Shared/Core/CompilerService.cs         (+ Diagnostic 扩展字段, GetDiagnostic 读取 fix)
Cide.Client.Shared/Core/Constants.cs               (已存在，MainViewModel 改用此文件)
Cide.Client.Shared/Core/NativeMethods.cs           (+ cide_diagnostic_get_fix P/Invoke)
Cide.Client.Shared/ViewModels/GraphNodeViewModel.cs (Maui forwarding + 构造函数)

Cide.Client/ViewModels/MainViewModel.cs            (-370 行，使用所有新服务)
Cide.Client/ViewModels/TypeForwards.cs             (+ 构造函数)
Cide.Client/ViewModels/CodeTemplate.cs             (+ 构造函数)
Cide.Client/ViewModels/GraphNodeViewModel.cs       (+ 构造函数)

Cide.Client.Maui/ViewModels/MainViewModel.cs       (-260 行，使用所有新服务)
Cide.Client.Maui/ViewModels/GraphNodeViewModel.cs  (+ 构造函数)
Cide.Client.Maui/Components/Editor/CanvasVisualizer.razor (+ @using)
```

### 删除文件

```
Cide.Client.Shared/Core/MemoryLayoutConstants.cs   (冗余，已合并到 Constants.cs)
```

## 验证结果

| 验证项 | 结果 |
|---|---|
| `Cide.Client.Shared` 构建 | ✅ |
| `Cide.Client` (Avalonia) 构建 | ✅ |
| `Cide.Client.Desktop` 构建 | ✅ |
| `Cide.Client.Maui` 构建 | ✅ **0 错误，0 警告** |
| 原生 `ctest` (Debug) | ✅ **10/10** |
| Android `.so` page size (arm64-v8a) | ✅ `0x4000` (16KB) |
| Android `.so` page size (armeabi-v7a) | ✅ `0x4000` (16KB) |

## 待后续推进

1. ~~**后端精确结构化修复** — Parser/TypeChecker 在报错时记录 `=`/`<=` 等精确位置，填充 `CideDiagnostic.fixKind`/`replaceRange`/`replacementText`~~ ✅ 已完成（2026-05-05）：`PopulateStructuredFix` + `MakeDiagnostic` + `Parser::Consume` 错误码映射，`E2005`/`E2006`/`E2007`/`E2008`/`E1004` 结构化修复全部通过测试
2. **Stage 4** — VM VisEvent 与前端动画帧同步 ✅ 已完成（2026-05-05）
3. **Stage 6** — 虚拟键盘适配、Release AOT+Trim、MAUI 全链路验证 ✅ 已完成（2026-05-05）

## 相关文档

- `docs/PLAN_20260504_OPTIMIZATION_AND_FIX.md` — 原始计划文档（已更新）
- `docs/ARCHIVE_PHASE3_CODE_REVIEW_AND_PLAN_20260427.md` — Phase 3 代码评审与计划
