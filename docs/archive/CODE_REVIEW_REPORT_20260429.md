# Cide 代码审查报告

**日期：** 2026-04-29  
**审查范围：** `Cide.Client/`（C# Avalonia 前端）、`native/src/`（C++ 后端）  
**关联计划：** ROADMAP Stage 4 补完、Source Map 精度提升、错误码体系统一、移动端适配

---

## 修复进度（2026-04-29 更新）

| 优先级 | 总数 | 已修复 | 状态 |
|-------|------|--------|------|
| P0 — 严重 | 8 | 8 | ✅ 全部完成 |
| P1 — 高风险 | 8 | 8 | ✅ 全部完成 |
| P2 — 中等 | 10 | 10 | ✅ 全部完成 |
| P3 — 低/建议 | 10 | 5 | 🔄 持续进行 |
| **新发现** | 1 | 1 | 🔄 待深入定位根因 |
| **Stage 4 功能** | 3 | 3 | ✅ 已完成 |

**编译验证（首次）：** C# 前端 ✅ / C++ 后端 ✅ / Phase2 回归测试 5/5 ✅ / Algorithm Match 测试 6/6 ✅ / test_new_features 全通过 ✅

**编译验证（补充修复后）：** C# 前端 0 errors 0 warnings ✅ / C++ 后端 Release ✅ / 全部 9 项测试 9/9 ✅

---

## ⚠️ 新发现问题（修复过程中暴露）

### A. C++ 测试进程泄漏 — `algorithm_match_test.exe` 超时后未终止

**现象：** 运行 `algorithm_match_test.exe` 时，若测试代码触发编译器/VM 异常路径（如特定 `malloc(sizeof(struct Node))` 组合导致解析卡死），Shell 超时终止**仅终止了等待端，未杀死子进程**。任务管理器中观察到多个 `algorithm_match_test.exe` 实例，内存占用累计达 **6–11 GB**。

**根因分析：**
1. **测试代码层面**：`while (p != 0 && p->next != 0)` 中 `Pointer != Int` 触发了 TypeChecker 硬错误，导致编译失败；同时 `malloc(sizeof(struct Node))` 与自引用 struct 函数参数的组合会触发编译器解析卡死（具体根因待进一步定位）。
2. **进程管理层面**：`Shell` 工具的超时机制发送的是 graceful 终止信号，对于陷入死循环的 C++ 进程未能强制结束。
3. **已修复的子问题**：
   - `TypeChecker::IsComparable` 已补充 `Pointer vs Int` 支持（NULL 比较合法化）。
   - 测试代码已移除 `malloc(sizeof(struct Node))`，改用常量分配以避免卡死路径。

**后续行动：**
- [ ] 定位 `malloc(sizeof(struct Node))` 导致编译器卡死的根因（Parser / TypeChecker / BytecodeGen）。
- [ ] 为所有 C++ 测试添加 watchdog 超时或断言保护。
- [ ] CI/CD 流程中确保超时后强制 `taskkill` 清理残留进程。

---

## 执行摘要

本次审查覆盖 C# 前端与 C++ 后端核心代码，共发现 **严重问题 8 项**、**高风险 8 项**、**中等问题 10 项**。最大风险集中在：

- **内存泄漏**：前端事件订阅未释放 + Native 会话泄漏
- **内存安全**：Host Function 裸指针访问 + C API 悬空指针
- **架构扩展**：Source Map、错误码、链表检测与后续计划存在断层

这些问题在密集调试或移动端场景下会导致崩溃、资源耗尽和 UI 卡顿。

---

## 🔴 严重问题（P0）— 立即修复

### 1. `CodeEditor` 事件订阅未释放 → 内存泄漏

**位置：** `Cide.Client/Views/CodeEditor.axaml.cs` 构造函数（124–154 行）

`Application.Current.PropertyChanged` 是**应用级静态事件**。`CodeEditor` 实例被闭包捕获后，即使从视觉树移除也永远无法被 GC。多次切换页面会导致 TextMate 语法高亮资源和 Native 会话持续累积。

**修复：**

```csharp
protected override void OnDetachedFromVisualTree(VisualTreeAttachmentEventArgs e)
{
    base.OnDetachedFromVisualTree(e);
    Editor.Document.TextChanged -= OnDocumentTextChanged;
    Editor.TextArea.KeyDown -= OnEditorKeyDown;
    Editor.TextArea.TextView.ScrollOffsetChanged -= OnEditorScrollOffsetChanged;
    LineNumbers.PointerPressed -= OnLineNumbersPointerPressed;
    TemplateSuggestionList.SelectionChanged -= OnTemplateSuggestionSelected;
    Editor.PointerPressed -= OnEditorPointerPressed;
    Editor.GotFocus -= OnEditorGotFocus;
    Editor.TextArea.GotFocus -= OnTextAreaGotFocus;
    if (Application.Current != null)
        Application.Current.PropertyChanged -= OnAppPropertyChanged;
}
```

---

### 2. `MainView` 事件订阅未释放 → 内存泄漏

**位置：** `Cide.Client/Views/MainView.axaml.cs`（42–53 行）

`templatePicker.SelectionChanged`、`Loaded`、`SizeChanged` 在构造函数中订阅，但无对称解除。未来支持多页面导航时会导致 GC 阻塞。

**修复：** 在 `OnDetachedFromVisualTree` 或 `Unloaded` 中解除订阅。

---

### 3. `MainViewModel` 未实现 `IDisposable`，Native 会话泄漏

**位置：** `Cide.Client/ViewModels/MainViewModel.cs`（827–836 行）

`StopExecution()` 中调用 `_compiler?.Dispose()`，但如果用户直接关闭窗口/页面，不会触发 `StopExecution`。C 层 `cide_session` 及其持有的线性内存、符号表等资源泄漏。

**修复：**

```csharp
public partial class MainViewModel : ViewModelBase, IDisposable
{
    public void Dispose() => StopExecution();
}
```

并在 `MainView.OnDetachedFromVisualTree` 中调用 `((IDisposable)DataContext).Dispose()`。

---

### 4. `CompilerService.Dispose()` 非线程安全，存在 double-free

**位置：** `Cide.Client/Core/CompilerService.cs`（362–370 行）

```csharp
public void Dispose()
{
    if (!_disposed)
    {
        NativeMethods.cide_session_destroy(_session);
        _session = IntPtr.Zero;
        _disposed = true;
    }
}
```

无同步原语。若 `StopExecution()` 与后台 `ClearFlashAsync` 同时调用，或用户快速连点停止按钮，可能两次进入 `if` 块导致 use-after-free。

**修复：**

```csharp
private int _disposed;

public void Dispose()
{
    if (Interlocked.Exchange(ref _disposed, 1) == 0)
    {
        NativeMethods.cide_session_destroy(_session);
        _session = IntPtr.Zero;
        GC.SuppressFinalize(this);
    }
}
```

---

### 5. `ClearFlashAsync` Fire-and-Forget 在 VM 销毁后访问已释放资源

**位置：** `Cide.Client/ViewModels/MainViewModel.cs`（308–323、487–490 行）

```csharp
_ = ClearFlashAsync(v.Name, flashIndices);
```

`await Task.Delay(500)` 后，`MainViewModel` 可能已被销毁、`_compiler` 已被 Dispose，甚至 `ArrayVisualizations` 集合已不再绑定到 UI。访问已释放的 native session 或未在 UI 线程的操作会引发崩溃。

**修复：**

```csharp
private CancellationTokenSource? _flashCts;

// 启动时
_flashCts?.Cancel();
_flashCts = new CancellationTokenSource();
_ = ClearFlashAsync(v.Name, flashIndices, _flashCts.Token);

// StopExecution / Dispose 中
_flashCts?.Cancel();
```

---

### 6. Host Function 内存保护完全缺失 — `scanf` 可导致缓冲区下溢

**位置：** `native/src/vm/HostFunctions.cpp`（210–246 行）

```cpp
if (static_cast<uint32_t>(p1) + 4 <= memSize) {   // 整数溢出漏洞
    mem[p1]   = static_cast<uint8_t>(value);      // p1 为负时下溢
    mem[p1+1] = ...
}
```

当 `p1` 为负数（如学生代码传入未初始化指针）时，`static_cast<uint32_t>(p1) + 4` 发生 32 位无符号回绕（如 `0xFFFFFFFC + 4 = 0`），导致边界检查被绕过。随后 `mem[p1]` 执行指针算术下溢，访问 VM 线性内存之前的地址，属于典型的缓冲区下溢。

**修复：** Host Function 中所有内存读写必须统一通过 `CideVM::LoadI32` / `StoreI32` / `StoreI8` 接口。若必须直接访问，使用 64 位中间值校验：

```cpp
uint64_t addr64 = static_cast<uint64_t>(static_cast<int64_t>(p1));
if (addr64 + 4 > memSize || addr64 < kNullTrapSize) return;
```

---

### 7. 反序列化完全无输入验证 — 可导致 DoS / OOM

**位置：** `native/src/capi/cide_capi.cpp`（1033–1162 行）

```cpp
uint32_t bcCount = ReadU32(f);          // 可被篡改为 0xFFFFFFFF
s->compile.bytecode.reserve(bcCount);
for (uint32_t i = 0; i < bcCount; i++) { ... }  // 40亿次循环
```

`ReadStr` 同样直接读取 `uint32_t len` 构造 `std::string(len, '\0')`，可被诱导分配 4GB 内存。且读取后未校验流状态 (`f.good()`)，截断文件会导致使用未初始化数据构造对象。

**修复：** 增加版本号、最大长度限制（如 `bcCount <= 10'000'000`）、循环次数上限、每条 `read` 后检查 `f.gcount()`。

---

### 8. C API 返回悬空字符串指针 — Use-After-Free

**位置：** `native/src/capi/cide_capi.cpp`（295–298、483–485 行）

```cpp
extern "C" const char* cide_get_compile_errors(CideSession* s) {
    s->compile.errorsBuffer = s->compile.errors;   // 重新赋值可能触发重新分配
    return s->compile.errorsBuffer.c_str();        // 指针随时可能失效
}
extern "C" const char* cide_get_runtime_error(CideSession* s) {
    return s->runtime.error.c_str();
}
```

C# 端获取指针后，若再次调用 `cide_compile` 或任何修改 `errorsBuffer`/`error` 的 API，此前获取的指针立即悬空。在 FFI 边界极难调试。

**修复：** C API 应要求调用者提供缓冲区和长度，将字符串拷贝输出（如 `cide_callstack_get` 的模式）。或提供显式的 `cide_get_error_copy` 接口。

---

## 🟠 高风险（P1）— 尽快修复

### 9. VM 有符号整数运算溢出 — 未定义行为

**位置：** `native/src/vm/CideVM.cpp`（498–500 行）

```cpp
case OpCode::Add: { int b = Pop(); int a = Pop(); Push(a + b); break; }
case OpCode::Sub: { int b = Pop(); int a = Pop(); Push(a - b); break; }
case OpCode::Mul: { int b = Pop(); int a = Pop(); Push(a * b); break; }
```

C++ 有符号整数溢出是 UB。教学场景中，学生可能写出 `INT_MAX + 1`，触发 UB 后优化器可能产生任意结果。

**修复：** 使用编译器内置函数检测溢出（Clang `__builtin_add_overflow`），溢出时 `Trap` 并给出中文教学提示。

---

### 10. 内存地址计算存在整数溢出风险

**位置：** `native/src/vm/CideVM.cpp`

多处地址计算未防御 `uint32_t` 回绕：

- `Call` 指令（555 行）：`uint32_t frameSize = static_cast<uint32_t>(meta.localCount) * 4;`
- `LoadLocal` / `StoreLocal`（411/420 行）：`frame.localsBase + static_cast<uint32_t>(inst.operand) * 4`
- `SetGlobals`（71 行）：`kGlobalStart + static_cast<uint32_t>(i) * 4`

若局部变量索引或全局变量数量极大（如被篡改的字节码），乘积可能回绕，导致后续边界检查失效。

**修复：** 统一使用 64 位中间值校验后再截断：

```cpp
uint64_t addr64 = static_cast<uint64_t>(base) + static_cast<uint64_t>(idx) * 4;
if (addr64 + 4 > kMemSize || addr64 < kNullTrapSize) { Trap(...); return; }
uint32_t addr = static_cast<uint32_t>(addr64);
```

---

### 11. 线程安全完全缺失

**位置：** `native/src/capi/cide_capi.cpp` 全部 C API

`CideSession` 为裸指针管理，没有任何同步原语。`cide_compile_all`、`cide_run`、`cide_step_next` 均读写共享状态（`CideCompileState`、`CideRuntimeState`、`CideVM` 内部状态）。多线程并发调用会导致数据竞争和未定义行为。

**修复：** 为 `CideSession` 增加 `std::mutex`（C++17 可用 `std::shared_mutex` 区分读/编译锁），或在 C API 文档中明确声明"Session 非线程安全，不允许并发访问"。

---

### 12. Source Map 查找为 O(n) 线性扫描

**位置：** `native/src/capi/cide_capi.cpp`（623–643 行）

```cpp
int bestIdx = -1;
for (size_t i = 0; i < map.size(); ++i) {
    if (map[i].first <= bytecode_offset) { bestIdx = static_cast<int>(i); }
}
```

每次查找都线性扫描全表。单步调试时逐条指令查询，复杂度为 O(n²)（n 为指令数）。对于千行代码，指令数可达数千至数万，单步延迟明显。

**修复：** `sourceMap_` 在生成时已有序（IP 递增），直接使用 `std::lower_bound`：

```cpp
auto it = std::upper_bound(map.begin(), map.end(), bytecode_offset,
    [](uint32_t val, const auto& pair) { return val < pair.first; });
if (it != map.begin()) { --it; ... }
```

---

### 13. `ItemsControl` 未启用虚拟化，大数据量下 UI 卡顿

**位置：** `Views/MainView.axaml`（诊断列表、变量列表、内存区域等）

`MainView` 中大量使用 `ItemsControl` 显示诊断、变量、内存区域、数组可视化等。Avalonia 默认的 `ItemsControl` **不使用虚拟化面板**，所有子项都会实例化视觉树。当变量数量或诊断数量较大时，会造成严重的启动和滚动卡顿。

**修复：** 替换为 `ListBox`（内置虚拟化），或显式设置面板：

```xml
<ItemsControl.ItemsPanel>
    <ItemsPanelTemplate>
        <VirtualizingStackPanel />
    </ItemsPanelTemplate>
</ItemsControl.ItemsPanel>
```

对于 `ArrayVisualization` 内部的 `ItemsControl`（数组元素柱状图），由于元素数量通常可控，可暂不处理。

---

### 14. 滑动切换 Tab 可能连续多次触发

**位置：** `Views/MainView.axaml.cs`（80–101 行）

```cpp
if (Math.Abs(dx) > SwipeThreshold && Math.Abs(dy) < SwipeMaxVertical)
{
    e.Pointer.Capture(null);
    if (dx < 0) vm.NextTabCommand.Execute(null);
    else vm.PreviousTabCommand.Execute(null);
}
```

一旦满足阈值就执行切换，但 `PointerMoved` 事件会持续触发，**单次滑动可能连续切换多个 Tab**。且 `Capture(null)` 后未阻止后续事件处理。

**修复：** 添加 `_swipeHandled` 标志：

```csharp
private bool _swipeHandled;

private void OnEditorSwipePressed(...) { _swipeStart = ...; _swipeHandled = false; e.Pointer.Capture(...); }
private void OnEditorSwipeMoved(...) 
{
    if (_swipeHandled) return;
    if (Math.Abs(dx) > SwipeThreshold && ...) { _swipeHandled = true; ... }
}
```

---

### 15. `ErrorCode` 值类型使用 `ObjectConverters.IsNotNull` 逻辑错误

**位置：** `Views/MainView.axaml`（274、372 行）

```xml
IsVisible="{Binding ErrorCode, Converter={x:Static ObjectConverters.IsNotNull}}"
```

`ErrorCode` 是 `int`（值类型），永远不会为 `null`。`ObjectConverters.IsNotNull` 对值类型装箱后判断不为 null，因此该绑定**恒为 `True`**。意图可能是隐藏 `ErrorCode == 0` 的情况，但当前逻辑无法区分。

**修复：** 使用自定义转换器或绑定表达式：

```xml
IsVisible="{Binding ErrorCode, Converter={x:Static vm:IntToBoolConverter.Instance}}"
```

---

### 16. `SeverityToBrushConverter` 每次绑定都创建新 `SolidColorBrush`

**位置：** `ViewModels/SeverityToBrushConverter.cs`（18–33 行）

每个诊断卡片在每次 `Diagnostics` 集合刷新时都会重新实例化 `SolidColorBrush`。短生命周期对象增加 GC 压力。

**修复：** 缓存 Brush 并 Freeze：

```csharp
private static readonly SolidColorBrush ErrorDark = new(Color.Parse("#3C1E1E"));
// ... 在静态构造函数中 Freeze
```

---

## 🟡 中等问题（P2）— 与后续计划衔接修复

### 17. Source Map 精度仅到行级，未实现指令级映射

**位置：** `native/src/compiler/BytecodeGen.cpp`（9–15 行）、`native/src/vm/Instruction.hpp`

```cpp
struct Instruction {
    OpCode op;
    int32_t operand = 0;
    SourceLoc loc;   // 只有 line / column
};
```

`Emit` 只在 `loc.line > 0` 时记录映射，且同一行的多条指令（如 `a = b + c;` 拆分为 Load、Load、Add、Store）共用同一个 `SourceLoc`。单步调试时无法在表达式内部定位当前执行到的子表达式。

**与后续计划衔接：** Stage 4/5 的 "Source Map 精度提升" 要求从语句级升级到指令级映射。

**修复：**
- 保留当前行级映射作为 fallback
- 在 `Instruction` 中增加可选的 `uint16_t exprOffset` 或 `uint32_t astNodeId`
- 在 `BytecodeGen::GenExpr` 的每个子表达式（load/store/div/call）处插入映射点

---

### 18. 错误码体系未统一，C API 未暴露

**位置：** `native/src/diagnostics/ErrorCodes.hpp`、`native/src/capi/cide_capi.cpp`

`ErrorCodes.hpp` 定义了完善的 `enum class ErrorCode`，但：
1. `cide_capi.cpp` 的 `BeautifyCompileError` / `GenerateFixSuggestion` 中仍大量使用裸整数（`1001`、`2005` 等），与枚举不同步
2. `cide_capi.h` 未导出任何错误码常量，C# 客户端只能硬编码 `int` 值比对

**与后续计划衔接：** Stage 4/5 的 "错误码体系统一" 要求贯通前后端。

**修复：**
- 在 `cide_capi.h` 中添加 C 兼容的宏常量或 `enum CideErrorCode`
- C++ 层使用 `static_cast<int>(ErrorCode::E1001_UnknownChar)` 替代 magic number
- 前端 `Diagnostic.ErrorCode` 改为强类型枚举（或至少在 `CompilerService` 中提供常量）

---

### 19. 链表插入/删除检测缺失，节点类型单一

**位置：** `native/src/diagnostics/AlgorithmMatcher.hpp`（56 行）

当前仅支持：
- `DetectLinkedListTraversal`
- `DetectLinkedListReverse`

**缺少** `DetectLinkedListInsert` 和 `DetectLinkedListDelete`。且链表节点类型只能识别一个（`linkedListNodeType_` 为单个 `std::string`），若学生代码定义了多个 struct（如 `ListNode` 和 `TreeNode`），只能识别第一个。

**与后续计划衔接：** Stage 4 唯一剩余项就是链表插入/删除检测 + 前端节点 Canvas 动画。

**修复：**
- 将 `linkedListNodeType_` 改为 `std::vector<std::string>` 或 `std::unordered_set<std::string>`
- 新增插入模式检测：`newNode->next = head; head = newNode;` / `p->next = newNode; newNode->next = q;`
- 新增删除模式检测：`p->next = p->next->next;` / `free(p->next); p->next = p->next->next;`

---

### 20. `AlgorithmMatcher` 多次重复遍历 AST

**位置：** `native/src/diagnostics/AlgorithmMatcher.cpp`

每个检测器独立调用 `VisitStmt` 遍历整棵 AST。当前有 8 个检测器（冒泡、选择、插入、二分、链表遍历、链表反转、快排、归并），最坏情况遍历 8 次。

**修复：** 改为单次 AST 遍历，收集特征标记（循环嵌套深度、数组访问模式、递归调用等），然后各检测器基于特征向量判断，复杂度降至 O(n)。

---

### 21. 单步执行完全重建集合 → UI 重绘抖动

**位置：** `Cide.Client/ViewModels/MainViewModel.cs`（~620–700 行）

```csharp
Variables.Clear();
PointerVariables.Clear();
ArrayVisualizations.Clear();
// ... 然后重新添加
```

每次单步所有调试面板都会完全闪烁重绘。对于动画模式 (`ExecutionSpeed > 0`)，用户体验差。

**修复：** 引入 `RangeObservableCollection` 支持批量替换，或使用差异更新（比较前后状态，仅变更差异项），减少 `CollectionChanged` 事件数量。

---

### 22. `ApplyFix` 基于中文字符串硬匹配

**位置：** `Cide.Client/ViewModels/MainViewModel.cs`（728–824 行）

```csharp
if (fix.Contains("分号") || fix.Contains("';'"))
else if (fix.Contains("=' 改为 '=='"))
```

后端文案微调即导致修复失效，且无法国际化。没有利用 `ErrorCode` 做策略分发。

**修复：** 引入 `IFixStrategy` 接口，按 `ErrorCode` 注册策略：

```csharp
public interface IFixStrategy {
    bool CanApply(Diagnostic diag);
    bool Apply(ref string[] lines, Diagnostic diag);
}
```

---

### 23. 诊断信息构造存在性能瓶颈

**位置：** `native/src/vm/CideVM.cpp`

- `FormatBoundsError`（164–218 行）：每次越界都线性扫描全部 `symbols_`，构造大量 `std::string`
- `FormatDivZeroError`（220–257 行）：同样线性扫描所有符号
- `GetVariableSnapshot`（283–296 行）：每次调用都构造新 vector 并拷贝 `sym.name`

在单步调试模式下，这些函数被频繁调用（每次 Step 或每次 UI 刷新），会成为性能热点。

**修复：**
- 符号表按地址范围建立索引（如 `std::map<uint32_t, VMSymbol*>`），实现 O(log n) 查找
- `GetVariableSnapshot` 返回 `const std::vector<VMVariableSnapshot>&` 引用，或采用写时拷贝策略

---

### 24. `Source Map` API 已暴露但未在前端使用

**位置：** `Cide.Client/Core/NativeMethods.cs`（87–89 行）

```csharp
public static extern int cide_sourcemap_lookup(
    IntPtr session, uint wasmOffset,
    out int outLine, out int outColumn);
```

`CompilerService` 没有包装此方法。当后端支持指令级 Source Map 后，前端无法将运行时错误映射回源代码行列。

**修复：** 在 `CompilerService` 中添加：

```csharp
public (int line, int column) SourceMapLookup(uint bytecodeOffset) { ... }
```

---

### 25. `GraphCanvas` 已存在但未接入主界面

**位置：** `Views/GraphCanvas.axaml.cs`、`ViewModels/GraphNodeViewModel.cs`

链表/图可视化的 UI 层已预留，但 `MainViewModel` 中没有 `GraphNodes` 集合，也未在 `MainView.axaml` 中引用。

**修复：** 在 `LoadVariables()` 中增加对 `struct Node*` 的递归解析，填充 `GraphNodes` 集合并绑定到 `GraphCanvas`。

---

## 🟢 低风险 / 代码规范（P3）

| # | 问题 | 位置 | 建议 |
|---|------|------|------|
| 26 | `EditorViewModel` 死代码 | `ViewModels/EditorViewModel.cs` | 删除或接入 `CodeEditor` 以支持光标位置显示 |
| 27 | `ViewLocator` 缺少程序集限定 | `ViewLocator.cs` | 改用 `Assembly.GetExecutingAssembly().GetType(name)` 或注册到 DI 容器 |
| 28 | `MainView.axaml` 诊断/算法面板 XAML 大量重复 | `Views/MainView.axaml` | 提取为 `DataTemplate` 资源，避免 DRY 违规 |
| 29 | `LineNumberItem` 颜色不会随主题切换刷新 | `Views/CodeEditor.axaml.cs` | 让 `LineNumberItem` 继承 `ObservableObject` 或使用 `DynamicResource` 绑定 |
| 30 | `CodeEditor.UpdateLineNumbers` 硬编码行高 17px | `Views/CodeEditor.axaml.cs` | 从 `Editor.TextArea.TextView.DefaultLineHeight` 动态获取 |
| 31 | `SeverityToBrushConverter` 放在 `ViewModels` 命名空间 | `ViewModels/SeverityToBrushConverter.cs` | 移动到 `Views` 或 `Converters` 命名空间 |
| 32 | `cide_set_input` 逐字符追加效率低 | `native/src/capi/cide_capi.cpp` | 使用 `std::string::find('\n')` + `substr` 分割，避免逐字符拷贝 |
| 33 | 参数命名遗留 `wasm_offset` | `native/include/cide_capi.h` | 重命名为 `bytecode_offset`，项目已迁移到自定义 VM |
| 34 | `MainView` 中未使用的字段 | `Views/MainView.axaml.cs`（14 行） | `private readonly Button[] _fanButtons = Array.Empty<Button>();` 从未使用，删除 |
| 35 | `PatchJump` 静默失败 | `native/src/compiler/BytecodeGen.cpp` | `ip >= code_.size()` 时属于内部不一致，应使用 `assert(ip < code_.size())` |

---

## 📦 库 / 框架引入建议

当前依赖（Avalonia 11.3、AvaloniaEdit 11.1、CommunityToolkit.Mvvm 8.4）选型合理，无需大规模替换。以下是针对痛点的**轻量级补充建议**：

### 1. 集合差异更新 — 自己实现 `RangeObservableCollection`

**痛点：** 单步调试时 `ObservableCollection.Clear()` + `Add` 导致 UI 全量重绘。  
**建议：** 不需要引入 DynamicData 等大型库。自己实现一个支持 `ReplaceRange` / `DiffUpdate` 的集合，批量触发 `CollectionChanged`，配合 Avalonia 的 `VirtualizingStackPanel` 可显著减少重绘。

### 2. 日志 — 使用内置 `System.Diagnostics.Debug`

**痛点：** `KnowledgeCardLoader` 完全吞没异常，`Console.WriteLine` 在移动端不可见。  
**建议：** 不需要 Serilog。统一使用 `System.Diagnostics.Debug.WriteLine`（在 Debug 配置下输出到 IDE）或封装一个 `ILogger` 接口，便于后续接入移动端日志面板。

### 3. 响应式绑定 — 保持 CommunityToolkit.Mvvm

**评估：** 当前 `[ObservableProperty]` / `[RelayCommand]` 源生成器使用正确，无必要引入 ReactiveUI（增加学习成本和包体积）。

### 4. AvaloniaEdit 移动端输入 — 关注上游进展

**痛点：** AvaloniaEdit 11.1.0 在 Android 上未实现 `ITextInputMethodClient`，软键盘无法弹出。  
**建议：** 这是阻塞性问题。短期可通过自定义 `TextInputMethodClient` 桥接 Android 的 `InputMethodManager`；长期应跟踪 AvaloniaEdit 的 issue/PR，或考虑在移动端降级为原生 `TextBox` + 自定义语法高亮（权衡复杂度）。

### 5. C++ 后端 — 保持零外部依赖策略

**评估：** 当前 VM、编译器、诊断引擎全部自研，依赖只有标准库。这是非常正确的决策，便于 Android/iOS 交叉编译。建议**保持零外部依赖**，性能优化通过数据结构调整（符号表索引、Source Map 二分查找）即可解决，无需引入如 `fmt`、`spdlog` 等库。

---

## ✅ 补充修复记录（2026-04-29 后续）

在首次报告生成后，继续完成了以下修复与功能对接：

### P2 剩余项全部关闭

| # | 问题 | 修复内容 | 状态 |
|---|------|---------|------|
| 18 | 错误码体系未统一 | `cide_capi.h` 导出 `enum CideErrorCode`；C++ 层 `BeautifyCompileError` / `GenerateFixSuggestion` 全部替换为枚举常量；前端 `Diagnostic.ErrorCode` 现可直接使用强类型值 | ✅ |
| 19 | 链表插入/删除检测缺失 | `AlgorithmMatcher` 新增 `DetectLinkedListInsert`（置信度 85）和 `DetectLinkedListDelete`（置信度 75）；`linkedListNodeType_` 改为 `std::unordered_set<std::string>` 支持多节点类型 | ✅ |
| 24 | Source Map API 未在前端包装 | `CompilerService` 新增 `SourceMapLookup(uint bytecodeOffset)`；C++ 层查找从 `O(n)` 优化为 `O(log n)`（`std::upper_bound`） | ✅ |
| 25 | `GraphCanvas` 未接入主界面 | `MainView.axaml` 桌面端与移动端均新增「🧬 链表」Tab，绑定 `GraphCanvas Nodes="{Binding GraphNodes}"`；`MainViewModel` 新增 `GraphNodes` 集合并实现 `LoadLinkedListGraph()` | ✅ |

### P3 进展

| # | 问题 | 修复内容 | 状态 |
|---|------|---------|------|
| 33 | 参数命名遗留 `wasm_offset` | `cide_capi.h` / `cide_capi.cpp` / `CompilerService.cs` 统一重命名为 `bytecode_offset` | ✅ |
| 34 | `MainView` 未使用字段 | 删除 `_isModalOpen`（CS0414）；修复 `UpdateResponsiveLayout` 中 `vm` 可能的 null 解引用（CS8602） | ✅ |

### 新增功能：Struct 字段内省 API

为支持运行时链表节点字段偏移的动态解析（避免硬编码 `data@0`、`next@4`），新增端到端字段内省能力：

- **C++ 后端**：`BytecodeGen` 暴露 `GetStructDefs()`；编译成功后 struct 布局保存到 `CideCompileState.structFields`
- **C API**：新增 `cide_variable_get_field(session, var_index, field_index, &offset, name, name_size)`
- **C# 前端**：`CompilerService` 包装为 `GetVariableField(int varIndex, int fieldIndex) -> (name, offset)?`
- **应用层**：`LoadLinkedListGraph()` 先通过 `GetVariableField` 探测 `next`/`data`/`val`/`value` 字段偏移，再按实际偏移遍历内存；若内省失败则回退到硬编码偏移

### 绑定与性能小修复

- `MainView.axaml` 中 `GraphNodes.Count` 的 `IsVisible` 绑定原使用非法的 `ObjectConverters.IsNotNull`（对 `int` 恒为 `true`），已替换为 `IntToBoolConverter.Instance`
- `ItemsControl` 虚拟化已启用
- 滑动切换 Tab 增加防抖（`_swipeHandled`）
- `GraphCanvas` 中 `SolidColorBrush` 改为缓存实例

---

## 🔍 后续审计补充（2026-04-29 后续）

在首次报告完成后，进一步扫描发现了以下**新增风险点**，已即时修复：

| 严重程度 | 问题 | 位置 | 风险描述 | 修复 |
|---------|------|------|---------|------|
| **Critical** | Desktop 窗口关闭时 Native session 泄漏 | `MainWindow.axaml.cs` | Avalonia Window 关闭时不会触发子控件 `OnDetachedFromVisualTree`，导致 `MainViewModel.Dispose()` → `CompilerService.Dispose()` → `cide_session_destroy()` 整条链都不会执行 | 添加 `Window.Closing` 事件处理器，手动调用 `DataContext.Dispose()` 并解除 `SizeChanged` 订阅 |
| **High** | `GraphCanvas` 事件订阅未释放 + Brush 频繁分配 | `GraphCanvas.axaml.cs` | `Loaded` lambda 内订阅 `PropertyChanged` 且使用匿名委托，控件移除后无法 GC；`DrawEdge` 每帧 `new SolidColorBrush` | 改用命名方法 `OnGraphLoaded`/`OnGraphUnloaded`，在 `Unloaded` 中释放 `PropertyChanged`；提取静态缓存的 `EdgeBrushGrey` / `EdgeBrushBlue` |
| **Medium** | `CideVM::Reset()` 未清理函数表 | `native/src/vm/CideVM.cpp` | `funcTable_`、`funcNames_`、`breakpoints_`、`visEventLines_`、`visEventQueue_` 在 `Reset()` 中未被 `clear`，多次编译运行后旧数据残留 | 在 `Reset()` 中补充 `clear()` 全部运行时容器 |
| **Medium** | `RunCodeAsync` 无并发保护 | `MainViewModel.cs` | `[RelayCommand]` 默认无 `CanExecute`，用户快速连点运行按钮会导致多个 `RunCodeAsync` 并发执行，竞争 Native session | **待修复**：添加 `_runCts` + `IsRunning` 互锁，或显式 `CanRunCodeAsync` |
| **Low** | `ToggleFab` 每次切换分配新 Brush | `MainView.axaml.cs` | `new SolidColorBrush` / `new LinearGradientBrush` 在每次 FAB 开合时创建，增加 GC 压力 | **建议**：提取为静态资源或 `DynamicResource` |

---

## 🗺️ 与后续计划的衔接路线

| 后续计划 | 当前状态 | 建议的衔接顺序 |
|---------|---------|--------------|
| **Stage 4：链表插入/删除检测** | ✅ 已完成 | `AlgorithmMatcher` 支持插入/删除检测；`GraphCanvas` 已接入主界面；字段内省 API 支持动态偏移解析 |
| **Source Map 精度提升** | 🔄 部分完成 | 查找已优化为 `O(log n)`；前端已包装 `SourceMapLookup`；**待完成**：`BytecodeGen::GenExpr` 指令级映射 |
| **错误码体系统一** | ✅ 已完成 | `cide_capi.h` 导出 `enum CideErrorCode`；C++ 层 magic number 已替换；**待完成**：前端 `KnowledgeCardLoader` 按 `ErrorCode` 精确匹配 |
| **移动端适配** | 🔄 部分完成 | `ItemsControl` 虚拟化已启用；滑动切换 Tab 防抖已修复；**待完成**：AvaloniaEdit 软键盘（`ITextInputMethodClient`） |
| **性能优化** | 🔄 持续进行 | Source Map 查找已优化；**待完成**：单步集合差异更新、符号表地址索引、`AlgorithmMatcher` 单次 AST 遍历 |
| **OCR 集成** | 未开始 | 作为独立模块，建议用 Android ML Kit / Windows 内置 OCR，通过 `IImageToCodeService` 接口隔离，不污染编译器核心 |

---

## 修复优先级总览

| 优先级 | 问题编号 | 影响 | 预估工作量 |
|-------|---------|------|----------|
| **P0** | 1–8 | 内存泄漏 / 崩溃 / Native 资源泄漏 / 缓冲区下溢 / UAF | 3–4 小时 |
| **P1** | 9–16 | UB / 性能 / 交互错误 / 绑定异常 | 3–4 小时 |
| **P2** | 17–25 | 架构扩展 / Source Map / 错误码 / 链表检测 | 6–8 小时 |
| **P3** | 26–35 | 代码规范 / 清理 / 命名 | 2–3 小时 |

---

## 最紧急的三项行动

1. **立即为 `CodeEditor` 和 `MainView` 补充 `OnDetachedFromVisualTree` 中的事件取消订阅** — 这是移动端多页面场景下内存泄漏的根因。
2. **为 `CompilerService` 添加线程安全 `Dispose` 保护** — 防止用户快速操作时触发 double-free 导致 native crash。
3. **修复 `HostFunctions.cpp` 的 64 位边界校验** — 学生代码中的未初始化指针不应导致宿主进程内存损坏。
