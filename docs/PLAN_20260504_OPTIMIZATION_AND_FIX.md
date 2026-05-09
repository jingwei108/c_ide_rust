# C IDE 代码优化与修复计划（2026-05-04）

## 背景

项目当前处于 **Stage 4（零侵入算法可视化）补完阶段**，后端编译器+VM+诊断系统已成熟，核心剩余工作集中在：
- 客户端架构优化（消除代码重复、线程安全）
- 原生层安全加固
- 移动端前端体验完善
- 与后续 Stage 5/6 计划接轨

---

## 一、已确认现状

### 前端架构
| 平台 | 当前技术 | 说明 |
|------|----------|------|
| 桌面端 | Avalonia 11.x + AvaloniaEdit + TextMate | 保持不变，无 CodeMirror 6 |
| 移动端 | MAUI Blazor Hybrid + CodeMirror 6 | 当前代码已集成 `GaelJ.BlazorCodeMirror6` |

> 注：文档中曾提及 Monaco Editor POC，但当前代码实际集成为 CodeMirror 6。如需调整编辑器方案，需单独决策。

---

## 二、修复任务清单（按优先级排序）

### P0 — 原生层安全漏洞（严重） ✅ 已完成

#### 2.1 `malloc` 负数参数导致整数溢出
- **位置**：`native/src/vm/HostFunctions.cpp:34-35`
- **问题**：`int32_t size` 为负数时（如 `-1`），转换为 `uint32_t alignedSize` 会变为 `0xFFFFFFFC`，绕过 `newOffset > memSize` 检查，导致 VM 内存状态混乱。
- **修复**：在 `malloc` host 函数入口处增加 `size <= 0` 检查，直接返回 NULL。
- **状态**：✅ 已编译通过，10/10 测试通过

#### 2.2 Stack-Heap 碰撞未检测
- **位置**：`native/src/vm/CideVM.cpp:611-620`
- **问题**：`Call` 指令分配栈帧时，未检查栈顶与堆偏移的碰撞。深层递归 + 大量 `malloc` 会损坏堆数据。
- **修复**：在 `Call` 指令中加入 `memStackTop_ - frameSize < heapOffset` 碰撞检测，触发中文栈溢出提示。
- **状态**：✅ 已编译通过，10/10 测试通过

### P1 — 客户端线程安全与架构

#### 2.3 `RunCodeAsync` 无并发保护 ✅ 已完成
- **位置**：`Cide.Client/ViewModels/MainViewModel.cs` 与 `Cide.Client.Maui/ViewModels/MainViewModel.cs`
- **问题**：`_isRunInProgress` 是普通 `bool`，在 async/await 上下文中存在竞态条件。
- **修复**：使用 `Interlocked.CompareExchange(ref _isRunInProgress, 1, 0)` 原子锁 + `RunCodeCommand.NotifyCanExecuteChanged()` 在 `finally` 中触发。
- **状态**：✅ 已编译通过（Desktop + Maui）

#### 2.4 提取共享库消除代码重复 ✅ 已完成
- **位置**：`Cide.Client/Core/*` 与 `Cide.Client.Maui/Core/*` 逐字符相同；`MainViewModel` 中数据模型也大量重复
- **问题**：任何 bug 修复需要改两处，维护成本翻倍，极易遗漏。
- **修复**：
  1. 新建 `Cide.Client.Shared` 类库项目（`net10.0`），两个前端项目均引用它
  2. 移入 `Core/CompilerService.cs`、`Core/NativeMethods.cs`（P/Invoke 声明）
  3. 移入公共 ViewModel 基类、数据模型（`Diagnostic`、`VariableSnapshot`、`TraceEntry`、`PointerViewModel`、`ArrayVisualization`、`CallStackFrame`、`WatchExpression` 等）
  4. 统一 `KnowledgeCardLoader`：定义 `IKnowledgeCardResourceProvider` 接口，各平台实现资源加载（Avalonia `AssetLoader` / MAUI `GetManifestResourceStream`），核心解析逻辑放入 Shared
  5. 保留平台 forwarding 类型（`Cide.Client.ViewModels.CodeTemplate`、`GraphNodeViewModel` 等），使现有 XAML 绑定零改动
- **状态**：✅ Desktop / Maui / Shared 均编译通过，原生 9/9 测试通过

#### 2.5 `MainViewModel` 上帝类拆分 — 基本完成 ✅
- **位置**：`Cide.Client/ViewModels/MainViewModel.cs`（~584 行，已从 960 行下降 **39%**）
- **问题**：承担编译、执行、调试、可视化、UI 状态等十几种职责。
- **已提取到 Shared 的服务**：
  - `CodeFixService` — 自动修复逻辑（原 ~90 行 ×2）
  - `VisualizationService` — 数组元素构建 + swap 检测（`BuildArrayElements`、`DetectSwapIndices`）
  - `DiagnosticService` — 诊断加载 + 知识卡片匹配（`LoadDiagnostics`）
  - `DebugDataService` — 变量/内存/调用栈/链表图/监视表达式加载（`LoadVariables`、`LoadCallStack`、`LoadLinkedListGraph`、`EvaluateWatchExpression`、`LoadMemoryRegions`、`LoadAlgorithmMatches`）
  - `ExecutionService` — 单步执行 + 全速运行纯逻辑（`StepNext`、`RunFullSpeed`）
  - `CompilerSessionService` — 编译会话生命周期（`EnsureCompiled`、断点同步、Dispose）
  - `CodeTemplate.GetDefaultTemplates()` — 默认代码模板（消除 30 行 ×2 重复）
- **MainViewModel 内部提取的辅助方法**：
  - `PresentCompileError` / `PresentRuntimeError` — 统一编译/运行时错误 UI 展示
  - `ResetExecutionState` — 统一执行前 UI 状态重置
- **当前状态**：`MainViewModel` 已缩减为纯 UI 状态协调器（属性声明、命令绑定、UI 更新），所有领域逻辑均已下沉到 Shared 服务。

### P2 — 移动端前端补完

#### 2.6 Maui 编辑器缺失核心 IDE 功能 ✅ 已完成
- **位置**：`Cide.Client.Maui/Components/Editor/CodeMirrorEditor.razor`
- **问题**：3 个 TODO — 断点装饰、错误行高亮、当前执行行高亮均未实现。
- **修复**：通过 JS Interop 直接操作 CodeMirror 6 DOM（`CMInstances[id]`）：
  1. `SetBreakpoints` — gutter 红点装饰
  2. `SetErrorLines` — 行背景红色高亮
  3. `SetHighlightLine` — 当前执行行黄色高亮 + 自动滚动
- **状态**：✅ Maui 编译通过

#### 2.7 编码问题
- **位置**：`Cide.Client.Maui/Core/NativeMethods.cs:74`
- **问题**：注释出现乱码 `诊断与修�?`
- **修复**：修正为 `诊断与修复`，并统一文件为 UTF-8 编码。

### P3 — 可维护性与性能

#### 2.8 自动修复逻辑提取 ✅ 已完成
- **位置**：两个 `MainViewModel.cs` 中各有一份 `ApplyFix`（~90 行）
- **问题**：基于中文字符串硬匹配（`fix.Contains("=' 改为 '=='"`），无法处理嵌套括号；且逻辑在两个平台重复维护。
- **已完成的修复**：
  1. 将 `ApplyFix` 核心逻辑提取到 `Cide.Client.Shared/Core/CodeFixService.cs`
  2. 返回结构化结果 `CodeFixResult(Applied, NewSourceCode, Message)`，由 `MainViewModel` 统一应用
  3. **新增 C API** `cide_diagnostic_get_fix` — 后端可返回 `FixKind`/`ReplaceRange`/`ReplacementText`
  4. **扩展 `CideDiagnostic`** — C++ 端已支持结构化 fix 字段（默认 `None`，待 Parser/TypeChecker 精确填充）
  5. **扩展 C# `Diagnostic`** — 新增 `FixKind`、`ReplaceStartLine/Column`、`ReplaceEndLine/Column`、`ReplacementText`
  6. **升级 `CodeFixService`** — 优先尝试结构化修复，fallback 到原有字符串匹配
  7. **后端精确填充** `PopulateStructuredFix` — 根据错误码和源码计算 `fixKind`/`replaceRange`/`replacementText`：
     - `E2005_ExpectedSemicolon` → `InsertText` `;`（在行尾或上一行末尾）
     - `E2006_ExpectedClosingBrace` → `InsertText` `}`
     - `E2007_ExpectedClosingParen` → `InsertText` `)`
     - `E2008_ExpectedClosingBracket` → `InsertText` `]`
     - `E1001_UnknownChar` → `DeleteText`（删除非法字符）
     - `E1002_UnterminatedString` → `InsertText` `"`（行尾）
     - `E1004_UnsupportedOp` → `ReplaceText` `||`/`&&`
     - `W3050_AssignInCondition` → `ReplaceText` `==`
     - `W3051_ArrayBoundOffByOne` → `ReplaceText` `<`
  8. **Parser::Consume** — 根据期望的 token 类型选择错误码（`RBrace`/`RParen`/`RBracket`）
  9. **Native 测试** `test_new_features.cpp` — `missing_semicolon`/`unsupported_op` 结构化 fix 验证
- **状态**：✅ Desktop + Maui + 原生 编译通过，10/10 测试通过

#### 2.11 Android 16 page-size 警告修复 ✅ 已完成
- **位置**：`native/CMakeLists.txt`
- **问题**：XA0141 — `libcide_native.so` 的 page size 不为 16KB，Android 16 将要求 16KB page size。
- **修复**：为所有 Android ABI 添加 linker flag `-Wl,-z,max-page-size=16384`；重新构建 `arm64-v8a` 和 `armeabi-v7a` 的 `.so`，验证 `Align = 0x4000`（16KB）。
- **状态**：✅ Maui 构建 0 警告，XA0141 消除

#### 2.9 魔法数字清理 ✅ 已完成
- **位置**：多处（`0x1000/0x40000` 内存范围、`4` 字节 int 步长、`4/0` struct 偏移等）
- **修复**：在 `Cide.Client.Shared/Core/Constants.cs` 中统一定义 `NullTrapEnd`、`LinearMemorySize`、`IntSize`、`DefaultStructNextOffset` / `DefaultStructDataOffset`，并在 `CompilerService.ReadArray`、两个 `MainViewModel` 的 `EvaluateWatchExpression`、`LoadVariables`、`LoadLinkedListGraph` 中全面替换。
- **状态**：✅ Desktop + Maui 编译通过

#### 2.10 `_lastArrayValues` 字典潜在泄漏 ✅ 已完成
- **位置**：两个 `MainViewModel.cs`
- **问题**：只增不减，重新编译或变量名变化时旧项残留。
- **修复**：在 `LoadVariables` 末尾收集当前存在的数组变量名集合，删除 `_lastArrayValues` 中不在集合里的 stale key。
- **状态**：✅ Desktop + Maui 编译通过

---

## 三、与后续 Roadmap 的接轨

### Stage 4（零侵入可视化）补完
- [x] VM `VisEvent` 与前端动画帧同步（CideVM 已发射事件，前端需精确消费）✅ 2026-05-05
- [x] 链表插入/删除检测的前端 `GraphCanvas` 动画细节完善 ✅ 2026-05-05
  - `GraphNodeViewModel` 添加 `FlashColor`/`EffectiveBackgroundColor`/`EffectiveBorderColor`
  - `DebugDataService.LoadLinkedListGraph` 根据 `NodeCreate`/`NodeAccess`/`NodeDelete` 事件标记节点（绿/蓝/红）
  - Desktop `GraphCanvas.axaml` + Maui `canvas-interop.js` 均支持 flash 高亮

### Stage 6（移动端适配）
- [x] 虚拟键盘适配（CodeMirror 6 在 Android WebView 中的软键盘唤起）✅ 2026-05-05
  - `MainActivity.cs` 使用 `WindowCompat.setDecorFitsSystemWindows(false)` + `WindowInsetsCompat.Type.ime()`
  - CSS 使用 `env(keyboard-inset-height, 0px)` 和 `dvh` 单位
- [ ] 验证 MAUI 端「编译→运行→单步→内存可视化」全链路（需实际设备）
- [x] Release 配置 AOT + Trim 压缩包体积 ✅ 2026-05-05
  - Maui APK: **303 MB → 80.89 MB** (-73.3%)
  - Desktop EXE: **单文件 34 MB** (Native AOT)
  - 新增 `build-release.ps1` 一键 Release 构建脚本
  - 修复 `InvariantGlobalization` 与 Profiled AOT 冲突

### 长期
- [ ] 子集渐进式解锁系统
- [ ] 知识图谱与概念关联
- [ ] iOS 平台扩展（MAUI 天然支持）

---

## 四、执行顺序建议

```
Step 1: P0 原生安全修复（立即）
Step 2: P1 并发安全修复（立即）
Step 3: P2 编码修复 + Maui 编辑器 TODO（当天）
Step 4: P1 提取 Shared 库（本周）
Step 5: P3 魔法数字 + 自动修复 + 字典泄漏（本周）
Step 6: MainViewModel 拆分（下周）
Step 7: Stage 4/6 功能补完（持续）
```

---

*本计划基于 2026-05-04 代码审查结果制定，执行过程中如遇架构冲突需回溯更新。*
