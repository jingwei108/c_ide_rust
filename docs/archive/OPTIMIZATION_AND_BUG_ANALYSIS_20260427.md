# C IDE 代码审查：Bug、框架优化与竞品增强方案

> 审查日期：2026-04-27  
> 审查范围：native/ (C++ 后端) + Cide.Client/ (C# Avalonia 前端) + 构建系统  
> 方法：地毯式代码审查 + 路线图对齐分析

---

## 一、Bug（按严重程度排序）

### B1: `nextLocalIdx_` 临时变量污染函数帧大小 🔴

**位置**: `native/src/compiler/BytecodeGen.cpp:560,857,976,982,1003,1022,1025,1044,1066`

**问题描述**:
`VisitAssign`、`VisitIndex`、`VisitSwitch` 中使用 `nextLocalIdx_++` 分配临时局部变量（如 `condTemp`、`valTemp`、`idxTemp`、`addrTemp`）。这些索引永不回收，被计入函数的 `localCount`。

后果：
- 每次 `Call` 指令分配 `localCount * 4` 字节帧空间，包含大量已无用的临时变量
- 多个语句的临时变量累积，栈帧持续膨胀
- 函数帧可能超过 256KB VM 内存导致栈溢出

```cpp
// BytecodeGen.cpp:560 — VisitSwitch
int condTemp = nextLocalIdx_++;  // 永久分配，永不回收

// BytecodeGen.cpp:976 — VisitAssign (数组赋值)
int valTemp = nextLocalIdx_++;   // ditto
int idxTemp = nextLocalIdx_++;   // ditto
int addrTemp = nextLocalIdx_++;  // ditto
```

**修复建议**:
方案 A：使用独立的 `maxTempIndex_` 跟踪，函数退出时重置为 0  
方案 B：复用固定槽位 `temp0_`、`temp1_`（当前子集最多 2 个临时变量）  
方案 C：临时变量放在 `localCount` 之后，不占用正式局部变量空间

---

### B2: `cide_get_compile_errors` 返回悬垂指针 🔴

**位置**: `native/src/capi/cide_capi.cpp:587-590`

**问题描述**:
```cpp
extern "C" const char* cide_get_compile_errors(CideSession* s) {
    if (!s || s->compile.errors.empty()) return nullptr;
    return s->compile.errors.c_str();  // std::string 内部 buffer
}
```

每次重新调用 `cide_compile` 会清空并重写 `s->compile.errors`，使之前通过 `cide_get_compile_errors` 获取的 `const char*` 指针成为悬垂指针。前端 C# 的 `Marshal.PtrToStringUTF8` 可能在 C++ 端重编译期间访问已释放的内存。

**修复建议**:
方案 A：返回时直接 `strdup`，让调用方 `free`（但 C API 不好管理）  
方案 B：使用内部静态缓冲区（线程不安全但教学场景单线程可接受）  
方案 C：改为 `cide_get_compile_errors(session, char* buf, int bufsize)` 缓冲区模式

---

### B3: 数组写操作缺少 `index >= 0` 运行时边界检查 🟠

**位置**: `native/src/compiler/BytecodeGen.cpp:985-998`

**问题描述**:
`VisitAssign` 的 Index 写分支（`arr[i] = x`）生成的字节码只检查 `index < arraySize`：
```cpp
// 只检查上界
Emit(OpCode::LoadLocal, idxTemp, node.loc);
Emit(OpCode::PushConst, arraySize, node.loc);
Emit(OpCode::Lt, 0, node.loc);
Emit(OpCode::Not, 0, node.loc);
// ❌ 缺少: index >= 0 的检查
```

如果用户写 `arr[-1] = 5`，运行时不触发 Trap，负数索引会绕到线性内存的低地址区域（全局变量区或 NULL 陷阱区），可能静默破坏数据。

**修复建议**: 增加 `0 <= index` 检查：
```cpp
// Check index >= 0
Emit(OpCode::LoadLocal, idxTemp, node.loc);
Emit(OpCode::PushConst, 0, node.loc);
Emit(OpCode::Ge, 0, node.loc);
Emit(OpCode::Not, 0, node.loc);
size_t jumpNeg = CurrentIP();
Emit(OpCode::JumpIfZero, 0, node.loc);
Emit(OpCode::LoadLocal, idxTemp, node.loc);
Emit(OpCode::TrapBounds, symIdx, node.loc);
PatchJump(jumpNeg, CurrentIP());
```

---

### B4: 数组读操作同样缺少 `index >= 0` 边界检查 🟠

**位置**: `native/src/compiler/BytecodeGen.cpp:860-867`（`VisitIndex`）

**问题描述**: 与 B3 相同，`arr[i]` 读操作也只检查上界 `index < arraySize`，未检查下界。

**修复建议**: 同 B3。

---

### B5: `CideVM::Ret` 释放栈帧空间逻辑有误 🟠

**位置**: `native/src/vm/CideVM.cpp:578-579`

**问题描述**:
```cpp
case OpCode::Ret: {
    // ...
    memStackTop_ = frame.localsBase + static_cast<uint32_t>(frame.localCount) * 4;
    // ...
}
```

Return 时应释放当前帧空间，即 `memStackTop_` 应恢复到帧之前的`位置（localsBase 之上）。当前写法是 `localsBase + frameSize`，即向**上**移动了 `frameSize` 字节，而非向下恢复。幸运的是，下一个 `Call` 指令会重新 `memStackTop_ -= frameSize`，所以实际行为可能因为后续 Call 覆盖而"巧合正确"，但若函数不调用任何子函数直接返回，栈顶指针就是错的。

**修复建议**:
```cpp
memStackTop_ = frame.localsBase;  // 恢复到帧开始位置
```

---

### B6: switch-case 的 `JumpIfNotZero` 路径缺少 StepEvent 🟡

**位置**: `native/src/compiler/BytecodeGen.cpp:563-571`

**问题描述**:
```cpp
for (auto* caseStmt : cases) {
    Emit(OpCode::LoadLocal, condTemp, node.loc);
    GenExpr(*caseStmt->label);
    Emit(OpCode::Eq, 0, node.loc);
    size_t jumpIP = CurrentIP();
    Emit(OpCode::JumpIfNotZero, 0, node.loc);  // ← 无 StepEvent
    caseJumpIPs.push_back(jumpIP);
}
```

`Eq` 和 `JumpIfNotZero` 是表达式指令而非语句，没有 `StepEvent`。单步调试时无法在 `case 2:` 的标签比较位置暂停。

**影响**: 单步调试时 switch 语句会"跳过"标签比较，直接跳到 case 体。

**修复建议**: 在 `JumpIfNotZero` 前插入 `StepEvent`，或确保 `GenExpr` 内已产生 StepEvent。

---

### B7: `CideVM::Step()` 中 `currentLine_` 被非 StepEvent 指令覆盖 🟡

**位置**: `native/src/vm/CideVM.cpp:382-384`

**问题描述**:
```cpp
const Instruction& inst = code_[ip_];
ip_++;
if (inst.loc.line > 0) {
    currentLine_ = inst.loc.line;  // 任何指令的 loc.line 都会覆盖！
}
```

`PushConst`、`Add`、`LoadLocal` 等表达式指令的 `loc.line` 也会更新 `currentLine_`。运行时 Trap（除零/越界）时报告的行号可能是某个子表达式的行号而非当前语句行号。

**修复建议**: 将 `currentLine_` 更新限制为 StepEvent 指令：
```cpp
if (inst.op == OpCode::StepEvent) {
    currentLine_ = inst.operand;
}
```
（当前 `StepEvent` 的 `case` 分支又在 switch 内重复设置了，可统一到此）

---

## 二、框架优化

### F1: 死代码清理 🔴

| 位置 | 问题 |
|:---|:---|
| `CMakeLists.txt:32` | `WasmCodeGen.cpp` 仍列为源文件，但 wasm3 已完全废弃 |
| `cide_capi.cpp:11` | `#include <thread>` — CideVM 零线程，不需要 |
| `BytecodeGen.cpp:2` | `#include <iostream>` —— 仅为一个注释掉的 `std::cerr` debug 输出 |
| `BytecodeGen.cpp:973` | 注释掉的 debug 输出 `// std::cerr << "DEBUG VisitAssign Index..."` |

**修复**: 删除以上引入和注释。

---

### F2: `capi` 与 `vm` 层职责混乱 🟠

**位置**: `native/src/capi/cide_capi.cpp:121-400`

**问题描述**:
Host 函数（printf/scanf/malloc/free）的完整实现（~280 行 lambda）全部写在 `cide_capi.cpp` 中，直接操作 `CideVM` 内部状态（`vm->GetMemory()`、`session->memory.regions` 等）。职责混乱：
- `capi` 层应该只做 C API → C++ 的桥接
- `.vm` 层应该封装所有执行语义

**修复建议**: 提取到 `native/src/vm/HostFunctions.cpp/hpp`，`capi` 只做注册调用：
```cpp
#include "vm/HostFunctions.hpp"
// capi 中:
HostFunctions::RegisterAll(s, &s->vm);
```

---

### F3: BytecodeGen 入口跳转偏移修正逻辑脆弱 🟠

**位置**: `native/src/compiler/BytecodeGen.cpp:220-237`

**问题描述**:
```cpp
// 插入 Jump 到 wrapper 前，手动偏移所有跳转目标和函数 IP
for (auto& inst : code_) {
    if (inst.op == Jump || JumpIfZero || JumpIfNotZero)
        inst.operand++;  // 所有跳转 +1
}
for (auto& kv : funcTable_) { kv.second.ip++; }  // 所有函数 IP +1
code_.insert(code_.begin(), Instruction(OpCode::Jump, wrapperIP + 1, {}));
```

这是倒转步骤：先插入再偏移更简单，但这里先偏移再插入。如果未来有新的跳转类型（例如 `Ret` 也有跳转语义？）忘记加在这里，会产生难以调试的 +1 偏差 bug。

**修复建议**: 翻转为先 `insert(begin())` 再统一偏移，或在 Generate 开始时预留 `code_[0]` 占位。

---

### F4: `cide_sourcemap_lookup` 参数命名残留 wasm 🟡

**位置**: `native/src/capi/cide_capi.cpp:853-854`

```cpp
extern "C" int cide_sourcemap_lookup(
    CideSession* s, unsigned int wasm_offset,  // ← 应改为 bytecode_offset
    int* out_line, int* out_column) {
```

**修复**: 改为 `bytecode_offset`。

---

### F5: CMake 测试注册仍逐个 `add_test` 🟡

**位置**: `native/CMakeLists.txt:85-104`

**问题描述**: 85~97 行已经是 `foreach` 循环 `add_executable`，但 98~104 行又逐个 `add_test`:
```cmake
add_test(NAME Phase2Regression COMMAND phase2_regression_test)
add_test(NAME Phase3Batch1    COMMAND phase3_batch1_test)
# ...重复 8 个
```

**修复建议**: 合并到同一循环中：
```cmake
foreach(test_mapping ${CIDE_TESTS})
    string(REPLACE ":" ";" parts ${test_mapping})
    list(GET parts 0 test_name)
    list(GET parts 1 test_file)
    add_executable(${test_name} tests/${test_file})
    # ...
    add_test(NAME ${test_name} COMMAND ${test_name})  # ← 加这里
endforeach()
```

---

### F6: P/Invoke `LPUTF8Str` 内存管理风险 🟡

**位置**: `Cide.Client/Core/NativeMethods.cs:25,42`

**问题描述**:
```csharp
[DllImport(LibName)]
public static extern int cide_compile(IntPtr session,
    [MarshalAs(UnmanagedType.LPUTF8Str)] string source);
```

`LPUTF8Str` Marshal 在 .NET 运行时自动分配零终止 UTF-8 内存，调用结束后释放。但如果 C API 内部保存了传入字符串的指针（例如存储到 `std::string` 后仍然有人引用原指针），后续访问就是悬垂指针。

当前检查：`cide_compile` 将 `source` 传给 `Lexer(source)`（值拷贝到 `std::string`），调用结束后释放是安全的。但需确保所有新增 API 也遵循"立即拷贝"模式。

**建议**: 在 `cide_capi.h` 注释中声明约定："调用方传入的字符串指针仅在被调用函数返回前有效"。

---

### F7: `TypeChecker::VisitStringLiteral` 和 `Ast.hpp` 中 `StringLiteralExpr` 的 `baseKind` 未设置 🟡

**位置**:
- `native/src/compiler/TypeChecker.cpp:537`
- `native/src/compiler/Ast.hpp:148-155`

**问题描述**:
```cpp
node.type = Type{TypeKind::Pointer, "char"};       // baseKind 默认 = Void
// 以及:
type = Type{TypeKind::Pointer, "char"};             // 同上
```

字符串字面量类型是 `char*`，但 `baseKind` 字段未显式设置为 `TypeKind::Char`。如果后续代码依赖 `baseKind` 判断（如类型转换检查），可能误判。

**修复**:
```cpp
node.type = Type{TypeKind::Pointer, "char", 0, TypeKind::Char};
```

---

## 三、路线图对齐框架改进

### R1: MainView 和 MainWindow 布局重叠 🔴

**位置**: `MainView.axaml:11` + `MainWindow.axaml:47-98`

**问题描述**:
- `MainView.axaml` 固定双栏 `ColumnDefinitions="*, 300"`（代码编辑器 + 右侧调试面板），内部有独立工具栏（Run/Step/Stop 按钮）
- `MainWindow.axaml` 桌面布局（`IsDesktop` 分支）**又画了一套**工具栏、编辑器占位、控制台占位
- 桌面端运行时，MainWindow 的 `ContentControl Content="{Binding}"` 嵌入 MainView，导致**双份工具栏**同时渲染

```
当前渲染树（桌面）:
MainWindow
├── Panel(IsDesktop)
│   ├── 左栏：文件树
│   ├── 中栏：ContentControl → MainView
│   │   └── Grid(*, 300)
│   │       ├── 工具栏 ← 第1份 Run/Step/Stop
│   │       ├── CodeEditor
│   │       └── 底部面板(输出+诊断)
│   └── 右栏：占位 TabControl ← 第2份 工具栏(第67行)
```

**修复建议**: 重构为：
- `MainView` → 纯内容组件（编辑器 + 输出 + 调试面板），不含外层壳
- `MainWindow` → 唯一的布局壳，消费 `ResponsiveLayoutViewModel` 切换手机/平板/桌面三种布局
- 手机布局用 `TabControl` 底导切换
- 平板/桌面用 `Grid` 多栏

---

### R2: VisEvent 类型系统不支持图/树结构 🔴

**位置**: `native/src/vm/CideVM.hpp:109-113` + `CideVM.cpp:598-611`

**问题描述**:
当前 VisEvent 类型定义：
```cpp
struct VisEvent {
    int type;   // 1=Compare, 2=Swap, 3=Update
    int line;
};
```

仅支持线性数组操作。Stage 4 计划实现的链表和二叉树可视化需要：
- `NodeCreate`  — 创建节点
- `EdgeConnect` — 建立指针连接
- `NodeAccess`  — 访问/高亮节点
- `NodeDelete`  — 删除/释放节点

**修复建议**: 将 type 改为枚举或结构化数据：
```cpp
struct VisEvent {
    enum Type : int { Compare = 1, Swap = 2, Update = 3,
                      NodeCreate = 4, EdgeConnect = 5,
                      NodeAccess = 6, NodeDelete = 7 };
    int type;
    int line;
    int extra[3];  // 扩展数据：节点地址、目标地址、值
};
```

---

### R3: 数组柱状图用 ItemsControl 布局，链表/树需要自由定位 Canvas 🔴

**位置**: `MainView.axaml:162-184`

**问题描述**:
当前柱状图用 `StackPanel Orientation="Horizontal"` 排列 Item，每个 Item 是 `Grid + Border`（线性排列）。链表和二叉树需要节点自由定位 + 连线绘制，这需要 Avalonia Canvas 的 `Canvas.Left` / `Canvas.Top` 绝对定位 + `Line` 绘制。

当前零 Canvas 基础设施。需要新增 `GraphCanvas` 组件（参考 `CodeEditor.axaml.cs` 的自定义控件模式）。

**建议**: 新建 `Cide.Client/Views/GraphCanvas.axaml` + `.cs`，绑定 `GraphNode[]` 集合：
```csharp
public record GraphNode(
    uint Address, string Label, int X, int Y,
    uint? NextAddr, bool IsHighlighted);
```

---

### R4: 缺少 Android NDK 交叉编译路径 🔴

**位置**: `build.ps1` + `native/CMakeLists.txt`

**问题描述**:
- `build.ps1` Android 分支仅做 `dotnet publish`，不编译 native `.so`
- `CMakeLists.txt:17-18` 检测了 `ANDROID` 但无 NDK 配置

Stage 6 移动端发布需要至少编译 ARM64 + ARMv7 两套 `.so`。

**修复建议**:
```powershell
# build.ps1 Android 分支增加:
$ndkHome = $env:ANDROID_NDK_HOME
cmake .. -G "Ninja" `
    -DCMAKE_TOOLCHAIN_FILE="$ndkHome/build/cmake/android.toolchain.cmake" `
    -DANDROID_ABI=arm64-v8a `
    -DANDROID_PLATFORM=android-21
```

---

### R5: C API 仅支持单源文件编译 🟠

**位置**: `native/include/cide_capi.h` + `cide_capi.cpp:507`（`cide_compile`）

**问题描述**: `cide_compile(session, source)` 一次只接收一个字符串。多文件 C 项目（`main.c` + `utils.c`）无法编译。Stage 6 计划中的文件树/模板库功能依赖此能力。

**修复建议**:
```cpp
// 方案 A：追加编译
int cide_compile_unit(CideSession* s, const char* filename, const char* source);

// 方案 B：批量编译
int cide_compile_multi(CideSession* s, const char** filenames, 
                       const char** sources, int count);
```

---

### R6: 知识卡片系统硬编码在 C# 前端 🟠

**位置**: `Cide.Client/ViewModels/KnowledgeCardViewModel.cs`

**问题描述**: 知识卡片内容（L2 解释、L3 原理动画、练习题）通过静态工厂方法 `FromErrorCode` 硬编码在 C# 字符串中。Stage 5 的知识图谱/渐进式学习系统无数据化基础。教师无法自定义知识点。

**修复建议**: 改为 JSON 资源文件驱动：
```
Assets/KnowledgeCards/
    E2005_missing_semicolon.json
    E3023_undeclared_variable.json
    R0001_array_bounds.json
```
JSON 结构：
```json
{
    "errorCode": 2005,
    "emoji": "😵",
    "title": "缺少分号",
    "plainExplanation": "C 语言中每条语句末尾需要写分号...",
    "wrongCode": "int a = 5",
    "correctCode": "int a = 5;",
    "exercise": "以下代码有什么问题...",
    "difficulty": 1
}
```

---

### R7: 缺少会话持久化 API 🟡

**位置**: `native/include/cide_capi.h`

**问题描述**: `CideSession` 无法序列化/反序列化。学生在手机上调试到一半，系统可能杀进程释放内存，再次启动丢失所有状态（代码、编译结果、单步位置）。Stage 6 移动场景尤其关键。

**修复建议**:
```cpp
// 导出会话状态
int cide_session_save(CideSession* s, const char* filepath);
int cide_session_load(CideSession* s, const char* filepath);
```

---

### R8: FluentTheme 未启用暗色/亮色切换能力 🟡

**位置**: `Cide.Client/App.axaml:10`

**问题描述**:
```xml
<FluentTheme />  <!-- 仅默认 Light -->
```

教学场景中学生在教室/夜晚不同光照下使用，需要一键切换暗色模式。

**修复建议**:
```xml
<FluentTheme Mode="{Binding ThemeMode}" />
```
其中 `ThemeMode` 绑定到 `MainViewModel.ThemeMode`（FluentThemeMode.Dark / Light）。

---

## 四、竞品增强建议（按价值/复杂度排序）

### C1: 集成 AvaloniaEdit 语法高亮 ⭐⭐⭐⭐⭐

**理由**: 竞品 Cxxdroid 有语法高亮，这是 IDE 的基础体验。当前自定义 TextBox 零高亮。

**实现**: 安装 `AvaloniaEdit` NuGet 包，配置 C 语言语法高亮规则，提供行号区、括号匹配、代码折叠。

**工时**: 1-2 天

---

### ~~C2: 行号区点击设置断点~~ ✅ 已完成（2026-04-28）

**理由**: 竞品 OnlineGDB 有断点，这是调试的基础体验。CideVM 已有 `paused_` 机制 + `StepEvent`，后端改动极小。

**实现**:
- `CideVM` 新增 `breakpoints_` 集合，`Step()` 中 `StepEvent` 处理时检查断点命中 → 自动 `Pause()`
- C API 新增 `cide_breakpoint_add/remove/clear`
- `CodeEditor` 行号区 `PointerPressed` 事件点击切换红色圆点（`Ellipse`），`BreakpointLines` 双向绑定
- `MainViewModel` 维护 `BreakpointLines` 集合，编译/运行/单步前同步到后端

**工时**: 0.5 天（实际）

---

### ~~C3: 内置代码模板库~~ ✅ 已完成（2026-04-28）

**理由**: 新手连 C 语法都不会，模板一键导入降低门槛。无竞品认真做。

**实现**:
- PC端：CodeEditor 监听 `KeyDown` Tab 键，自动将光标前的单词（如 `bubble`）展开为完整模板代码
- 移动端/备选：工具栏 `ComboBox` 下拉选择模板，点击后插入到编辑器光标位置
- `CodeTemplate` 记录定义 `Key`/`DisplayName`/`Category`/`Code`
- `MainViewModel` 初始化 7 个教学模板（冒泡/选择/插入排序、阶乘、斐波那契、交换变量、数组逆序）

**工时**: 0.5 天（实际）

---

### ~~C4: 调用栈视图~~ ✅ 已完成（2026-04-28）

**理由**: CideVM 已有 `callStack_` 结构体（IP + localsBase + localCount），前端零展示。教学场景中理解函数调用链至关重要。

**实现**:
- `CideVM::CallFrame` 新增 `funcName` 字段；`Call` 指令处理时记录函数名
- C API 新增 `cide_callstack_count` / `cide_callstack_get`，通过 `sourceMap` 将 `returnIP` 映射回源码行号
- 前端右侧调试面板新增「📞 调用栈」Tab，显示函数名 + 返回行号；当前帧标注 `➤` 箭头
- 点击调用栈项自动跳转高亮对应代码行

**工时**: 0.5 天（实际）

---

### ~~C5: 执行速度滑块~~ ✅ 已完成（2026-04-28）

**理由**: 算法动画目前全速播放或手动单步，缺少可控速度播放。类比 Scratch 的"速度"滑块。

**实现**:
- `MainViewModel` 新增 `ExecutionSpeed`（0-500ms）；`RunCode` 改为 `async Task`
- 滑块在 0 时：全速运行（原生 `_compiler.Run()`）
- 滑块 >0 时：动画模式，循环调用 `_compiler.StepNext()` + `await Task.Delay(ExecutionSpeed)`
- 每步后实时更新变量、调用栈、高亮行、visEvent 日志，实现算法动画的逐帧播放效果
- `StepNext` 核心逻辑提取为 `DoSingleStep()`，被单步按钮和动画模式共用

**工时**: 0.5 天（实际）

---

### ~~C6: Watch 表达式~~ ✅ 已完成（2026-04-28）

**理由**: 变量面板只能看简单变量，不能自定义表达式（如 `arr[i]`、`p->next->val`）。

**实现**:
- 前端「🔍 Watch」Tab：输入框 + 表达式列表 + 添加/删除按钮
- `EvaluateWatchExpression()` 支持：直接变量名（`sum`）、数组索引（`arr[2]`）、指针解引用（`*p`）、取地址（`&var`）
- 每次单步后自动调用 `RefreshWatchExpressions()` 刷新所有 Watch 值
- 零后端改动，纯前端通过已有 `ReadMemoryValue` + 变量面板数据实现

**工时**: 0.5 天（实际）

---

## 五、总结优先级矩阵

### 修复/优化

| 编号 | 级别 | 类别 | 问题 |
|:---|:---|:---|:---|
| B1 | 🔴 P0 | Bug | `nextLocalIdx_` 临时变量污染帧大小 |
| B2 | 🔴 P0 | Bug | `cide_get_compile_errors` 悬垂指针 |
| F1 | 🔴 P0 | 框架 | 死代码清理 |
| R1 | 🔴 P0 | 框架 | MainView/MainWindow 布局重叠 |
| R2 | 🔴 P0 | 框架 | VisEvent 不支持图/树 |
| R3 | 🔴 P0 | 框架 | 缺 Canvas 图渲染组件 |
| R4 | 🔴 P0 | 框架 | 缺 Android NDK 编译 |
| B3 | 🟠 P1 | Bug | 数组写缺 index>=0 检查 |
| B4 | 🟠 P1 | Bug | 数组读缺 index>=0 检查 |
| B5 | 🟠 P1 | Bug | Ret 帧释放逻辑有误 |
| F2 | 🟠 P1 | 框架 | capi/vm 职责混乱 |
| F3 | 🟠 P1 | 框架 | 入口跳转偏移逻辑脆弱 |
| R5 | 🟠 P1 | 框架 | 仅支持单源文件编译 |
| R6 | 🟠 P1 | 框架 | 知识卡片硬编码 |
| B6 | 🟡 P2 | Bug | switch JumpIfNotZero 缺 StepEvent |
| B7 | 🟡 P2 | Bug | currentLine_ 被非 StepEvent 覆盖 |
| F4 | 🟡 P2 | 框架 | sourcemap 参数命名残留 wasm |
| F5 | 🟡 P2 | 框架 | CMake 重复 add_test |
| F6 | 🟡 P2 | 框架 | P/Invoke LPUTF8Str 风险 |
| F7 | 🟡 P2 | 框架 | StringLiteral baseKind 未设 |
| R7 | 🟡 P2 | 框架 | 缺会话持久化 |
| R8 | 🟡 P2 | 框架 | 主题不可切换 |

### 竞品增强

| 编号 | 优先级 | 功能 | 工时 |
|:---|:---|:---|:---|
| ~~C1~~ | ~~P0~~ | ~~AvaloniaEdit 语法高亮~~ | ~~1-2 天~~ | ✅ 已完成（2026-04-28）|
| ~~C2~~ | ~~P0~~ | ~~断点调试~~ | ~~1-2 天~~ | ✅ 已完成（2026-04-28）|
| ~~C3~~ | ~~P0~~ | ~~代码模板库~~ | ~~1 天~~ | ✅ 已完成（2026-04-28）|
| ~~C4~~ | ~~P1~~ | ~~调用栈视图~~ | ~~1 天~~ | ✅ 已完成（2026-04-28）|
| ~~C5~~ | ~~P1~~ | ~~执行速度滑块~~ | ~~0.5 天~~ | ✅ 已完成（2026-04-28）|
| ~~C6~~ | ~~P2~~ | ~~Watch 表达式~~ | ~~1 天~~ | ✅ 已完成（2026-04-28）|


---

## 六、修复进度日志

> 本章节记录实际修复进展，按推进批次更新。

### 批次一：P0 全部完成（2026-04-27）

| 编号 | 状态 | 核心设计决策 | 验证 |
|:---|:---|:---|:---|
| B1 | ✅ 已完成 | 引入 `GetTempSlot(int)` 按需延迟分配固定槽位（最大 3 个并发），未使用临时变量的函数零额外开销 | native DLL + 全部测试编译通过 |
| B2 | ✅ 已完成 | `CideCompileState` 内新增 `errorsBuffer` 持久缓冲区，`cide_get_compile_errors` 返回前先拷贝 | native DLL 编译通过 |
| F1 | ✅ 已完成 | 移除 `WasmCodeGen.cpp` 引用、冗余 `<iostream>` / `<thread>` include、注释掉的 debug 输出 | native DLL 编译通过 |
| R1 | ✅ 已完成 | 桌面布局从三栏（含重复工具栏/输出/占位右栏）简化为双栏（文件树 \| MainView），消除双份渲染 | C# Client 编译通过 |
| R2 | ✅ 已完成 | `VisEvent` 扩展 `extra[3]` + 枚举（`NodeCreate/EdgeConnect/NodeAccess/NodeDelete`）；C API **新增 `cide_vis_event_get_ex`**，**保留旧 API 不变**；C# 新增 `GetVisEventEx()` | native DLL + 全部测试 + C# Client 编译通过 |
| R3 | ✅ 已完成 | 新建 `GraphCanvas`（`GraphNodeViewModel` + XAML `ItemsControl` + 代码后台动态 `Line` 边），支持链表/二叉树双模式 | C# Client 编译通过 |
| R4 | ✅ 已完成 | `build.ps1` Android 分支增加 NDK 交叉编译（`arm64-v8a` + `armeabi-v7a`）→ 复制 `.so` → `dotnet publish`；`csproj` 通过 `<AndroidNativeLibrary>` 打包 | PowerShell 语法验证通过 |

### 待推进项（P1 / P2）

| 编号 | 级别 | 类别 | 问题 |
|:---|:---|:---|:---|
| B3 | 🟠 P1 | Bug | 数组写缺 `index>=0` 检查 |
| B4 | 🟠 P1 | Bug | 数组读缺 `index>=0` 检查 |
| B5 | 🟠 P1 | Bug | `Ret` 帧释放逻辑有误 |
| F2 | 🟠 P1 | 框架 | capi/vm 职责混乱（提取 HostFunctions） |
| F3 | 🟠 P1 | 框架 | 入口跳转偏移逻辑脆弱 |
| R5 | 🟠 P1 | 框架 | 仅支持单源文件编译 |
| R6 | 🟠 P1 | 框架 | 知识卡片硬编码（JSON 资源驱动） |
| ~~B6~~ | ~~🟡 P2~~ | ~~Bug~~ | ~~switch `JumpIfNotZero` 缺 `StepEvent`~~ ✅ 已完成 |
| ~~B7~~ | ~~🟡 P2~~ | ~~Bug~~ | ~~`currentLine_` 被非 `StepEvent` 覆盖~~ ✅ 已完成 |
| ~~F4~~ | ~~🟡 P2~~ | ~~框架~~ | ~~sourcemap 参数命名残留 wasm~~ ✅ 已完成 |
| ~~F5~~ | ~~🟡 P2~~ | ~~框架~~ | ~~CMake 重复 `add_test`~~ ✅ 已完成 |
| ~~F6~~ | ~~🟡 P2~~ | ~~框架~~ | ~~P/Invoke `LPUTF8Str` 风险~~ ✅ 已完成 |
| ~~F7~~ | ~~🟡 P2~~ | ~~框架~~ | ~~`StringLiteral` `baseKind` 未设~~ ✅ 已完成（无需改动） |
| ~~R7~~ | ~~🟡 P2~~ | ~~框架~~ | ~~缺会话持久化~~ ✅ 已完成 |
| ~~R8~~ | ~~🟡 P2~~ | ~~框架~~ | ~~主题不可切换~~ ✅ 已完成 |
| ~~C1~~ | ~~P0~~ | ~~竞品增强~~ | ~~AvaloniaEdit 语法高亮~~ ✅ 已完成 |
| ~~C2~~ | ~~P0~~ | ~~竞品增强~~ | ~~断点调试~~ ✅ 已完成 |
| ~~C3~~ | ~~P0~~ | ~~竞品增强~~ | ~~代码模板库~~ ✅ 已完成 |
| ~~C4~~ | ~~P1~~ | ~~竞品增强~~ | ~~调用栈视图~~ ✅ 已完成 |
| ~~C5~~ | ~~P1~~ | ~~竞品增强~~ | ~~执行速度滑块~~ ✅ 已完成 |
| ~~C6~~ | ~~P2~~ | ~~竞品增强~~ | ~~Watch 表达式~~ ✅ 已完成 |

### 修改文件清单（批次一）

**Native (C++)**
- `native/src/compiler/BytecodeGen.hpp` — 新增 `tempSlot0/1/2_` + `GetTempSlot()` 声明
- `native/src/compiler/BytecodeGen.cpp` — 实现按需临时槽位分配；清理 `<iostream>` 和 debug 注释
- `native/src/vm/CideVM.hpp` — `VisEvent` 扩展 `extra[3]` + 枚举
- `native/src/vm/CideVM.cpp` — `VisEvent` 初始化清零
- `native/src/capi/cide_capi.cpp` — `errorsBuffer` 防悬垂；新增 `cide_vis_event_get_ex`
- `native/include/cide_capi.h` — 新增 `cide_vis_event_get_ex` 声明
- `native/CMakeLists.txt` — 移除 `WasmCodeGen.cpp`

**Frontend (C#)**
- `Cide.Client/Views/MainWindow.axaml` — 桌面布局去重
- `Cide.Client/Core/NativeMethods.cs` — 新增 `cide_vis_event_get_ex` P/Invoke
- `Cide.Client/Core/CompilerService.cs` — 新增 `GetVisEventEx()`
- `Cide.Client/ViewModels/GraphNodeViewModel.cs` — **新建**
- `Cide.Client/Views/GraphCanvas.axaml` — **新建**
- `Cide.Client/Views/GraphCanvas.axaml.cs` — **新建**

**Build / Android**
- `build.ps1` — Android NDK 交叉编译 + Clean 目录更新
- `Cide.Client.Android/Cide.Client.Android.csproj` — `<AndroidNativeLibrary>` 引用



### 批次二：P1 全部完成（2026-04-27）

| 编号 | 状态 | 核心设计决策 | 验证 |
|:---|:---|:---|:---|
| B3 | ✅ 已完成 | 数组写操作 `VisitAssign` Index 分支增加 `index >= 0` 检查（Ge + Not + JumpIfZero），与原有 `index < arraySize` 形成完整双边界检查 | native 全部 7 项测试通过 |
| B4 | ✅ 已完成 | 数组读操作 `VisitIndex` 同步增加 `index >= 0` 检查，与 B3 共用同一套检查模式 | native 全部 7 项测试通过 |
| B5 | ✅ 已完成 | `Ret` 和 `RetVoid` 的帧释放从 `localsBase + frameSize` 修正为 `localsBase`，恢复栈顶到帧开始位置 | native 全部 7 项测试通过 |
| F2 | ✅ 已完成 | 提取 `native/src/vm/HostFunctions.cpp/hpp`，将 ~240 行 Host lambda 移出 capi 层；`cide_capi.cpp` 仅做桥接调用；`CideSession` 定义提取到 `capi/CideSession.hpp` 供 vm 层共享 | native 全部 7 项测试通过 |
| F3 | ✅ 已完成 | 废弃脆弱的先偏移再插入逻辑，改为 `Generate()` 开头预留 `Nop` 占位，wrapper 生成后直接 `code_[0] = Jump(wrapperIP)`；零偏移、零特殊处理 | native 全部 7 项测试通过 |
| R5 | ✅ 已完成 | 新增 `cide_compile_unit` + `cide_compile_all` C API；内部将多单元 AST 合并（move structs/globals/funcs）后统一 TypeCheck + BytecodeGen；`cide_compile` 向后兼容（清空 units 后单文件编译） | native 全部 7 项测试通过 |
| R6 | ✅ 已完成 | 新建 `Assets/KnowledgeCards/*.json` 资源文件 + `KnowledgeCardLoader`（`AssetLoader` 加载 + `System.Text.Json` 解析）；`KnowledgeCardViewModel.FromErrorCode` 改为数据驱动查找；硬编码内容全部迁移到 JSON | C# Client 编译通过 |

### 批次二修改文件清单

**Native (C++)**
- `native/src/compiler/BytecodeGen.cpp` — B3/B4 双边界检查；F3 Nop 占位入口跳转
- `native/src/vm/CideVM.cpp` — B5 Ret 帧释放修正；F3 Nop case
- `native/src/vm/OpCode.hpp` — F3 新增 `Nop`
- `native/src/capi/CideSession.hpp` — **新建**（F2 提取共享结构体；R5 添加 `CideCompileUnit`）
- `native/src/vm/HostFunctions.hpp` — **新建**（F2 Host 函数声明）
- `native/src/vm/HostFunctions.cpp` — **新建**（F2 Host 函数实现）
- `native/src/capi/cide_capi.cpp` — F2 移除 Host lambda + 桥接调用；R5 多文件编译 API
- `native/include/cide_capi.h` — R5 新增 `cide_compile_unit` / `cide_compile_all`
- `native/CMakeLists.txt` — F2 添加 `HostFunctions.cpp`

**Frontend (C#)**
- `Cide.Client/Assets/KnowledgeCards/*.json` — **新建 ×4**（R6 JSON 资源）
- `Cide.Client/Core/KnowledgeCardLoader.cs` — **新建**（R6 资源加载器）
- `Cide.Client/ViewModels/KnowledgeCardViewModel.cs` — R6 移除硬编码工厂方法

### 批次三：P2 全部完成（2026-04-28）

> 稳步推进，不急于见效，力求代码健壮。

| 编号 | 状态 | 核心设计决策 | 验证 |
|:---|:---|:---|:---|
| B6 | ✅ 已完成 | `VisitSwitch` 中每个 `case` 标签比较前插入 `StepEvent`，单步调试可在 case 条件判断处暂停 | native 全部测试通过 |
| B7 | ✅ 已完成 | `CideVM::Step()` 中 `currentLine_` 更新限制为仅 `StepEvent` 指令，避免表达式指令（`PushConst`、`Add` 等）覆盖语句行号；`StepEvent` 的 `operand` 直接存行号 | native 全部测试通过 |
| F4 | ✅ 已完成 | `cide_sourcemap_lookup` 参数名从 `wasm_offset` 改为 `bytecode_offset`，消除 wasm3 废弃后的命名残留 | native DLL 编译通过 |
| F5 | ✅ 已完成 | CMake `add_test` 合并进已有的 `foreach` 循环，消除 8 个逐个硬编码 | CMake 配置验证通过 |
| F6 | ✅ 已完成 | `cide_capi.h` 中 `cide_compile` 文档注释声明约定：`source` 字符串指针仅在调用期间有效；`cide_compile` 内部立即拷贝到 `std::string`，无悬垂风险 | 代码审查通过 |
| F7 | ✅ 已完成（无需改动） | `TypeChecker::VisitStringLiteral` 已正确设置 `baseKind = TypeKind::Char`；`Ast.hpp` 构造亦正确。文档审查确认无遗漏 | 代码审查通过 |
| R7 | ✅ 已完成 | 新增 `cide_session_save` / `cide_session_load` C API；内部实现 `SerializeSession` / `DeserializeSession`，覆盖编译单元、字节码、全局变量、运行时状态 | native 全部测试通过 |
| R8 | ✅ 已完成 | `MainViewModel` 新增 `IsDarkMode` 可观察属性；`OnIsDarkModeChanged` 绑定 `Application.Current.RequestedThemeVariant` 切换 `ThemeVariant.Dark` / `Light` | C# Client 编译通过 |

### 批次三修改文件清单

**Native (C++)**
- `native/src/compiler/BytecodeGen.cpp` — B6 switch-case 体前插入 `StepEvent`
- `native/src/vm/CideVM.cpp` — B7 `currentLine_` 仅在 `StepEvent` 时更新
- `native/src/capi/cide_capi.cpp` — F4 `bytecode_offset` 命名；R7 `SerializeSession` / `DeserializeSession` 实现
- `native/include/cide_capi.h` — F6 字符串指针有效期注释；R7 `cide_session_save` / `cide_session_load` 声明
- `native/CMakeLists.txt` — F5 `add_test` 合并到 `foreach`

**Frontend (C#)**
- `Cide.Client/ViewModels/MainViewModel.cs` — R8 `IsDarkMode` + `RequestedThemeVariant` 主题切换

### 批次四：断点调试 C2（2026-04-28）

> 稳步推进，不急于见效，力求代码健壮。

| 编号 | 状态 | 核心设计决策 | 验证 |
|:---|:---|:---|:---|
| C2 | ✅ 已完成 | `CideVM::Step()` 中 `StepEvent` 处理时检查 `breakpoints_` 集合，命中则 `paused_ = true` + `stepEventHit_ = true`，单步/运行均可在断点行暂停；前端行号区点击切换红色圆点，双向绑定到 `BreakpointLines`；编译/运行/单步前同步断点到后端 session | 全部 CTest 8/8 通过 + C# Client 编译通过 |

### 批次四修改文件清单

**Native (C++)**
- `native/src/vm/CideVM.hpp` — 新增 `breakpoints_` 集合 + `AddBreakpoint`/`RemoveBreakpoint`/`ClearBreakpoints`/`HasBreakpoint`
- `native/src/vm/CideVM.cpp` — `Step()` 中 `StepEvent` 分支增加断点命中检查
- `native/src/capi/cide_capi.cpp` — 新增 `cide_breakpoint_add` / `cide_breakpoint_remove` / `cide_breakpoint_clear`
- `native/include/cide_capi.h` — 新增断点 C API 声明

**Frontend (C#)**
- `Cide.Client/Views/CodeEditor.axaml` — 行号区 `Grid` 布局：左侧 `Ellipse` 断点圆点 + 右侧行号文本
- `Cide.Client/Views/CodeEditor.axaml.cs` — `LineNumberItem` 新增 `IsBreakpoint`/`CircleColor`；`CodeEditor` 新增 `BreakpointLines` 依赖属性 + `PointerPressed` 点击切换
- `Cide.Client/Core/NativeMethods.cs` — 新增 `cide_breakpoint_add` / `remove` / `clear` P/Invoke
- `Cide.Client/Core/CompilerService.cs` — 新增 `AddBreakpoint` / `RemoveBreakpoint` / `ClearBreakpoints` 封装
- `Cide.Client/ViewModels/MainViewModel.cs` — 新增 `BreakpointLines` 集合；`EnsureCompiled`/`RunCode`/`StepNext` 中同步断点
- `Cide.Client/Views/MainView.axaml` — `CodeEditor` 绑定 `BreakpointLines`

### 批次五：代码模板库 C3（2026-04-28）

> 稳步推进，不急于见效，力求代码健壮。

| 编号 | 状态 | 核心设计决策 | 验证 |
|:---|:---|:---|:---|
| C3 | ✅ 已完成 | PC端 `CodeEditor` 监听 `KeyDown` Tab 键，提取光标前单词匹配 `Templates` 字典，命中则替换为完整代码；移动端通过工具栏 `ComboBox` 下拉选择模板，`MainView` code-behind 处理 `SelectionChanged` 调用 `CodeEditor.InsertTemplate()`；`CodeTemplate` 数据模型支持 `Key`/`DisplayName`/`Category`/`Code` | 全部 CTest 8/8 通过 + C# Client 编译通过 |

### 批次五修改文件清单

**Frontend (C#)**
- `Cide.Client/ViewModels/CodeTemplate.cs` — **新建** `CodeTemplate` 记录
- `Cide.Client/ViewModels/MainViewModel.cs` — 新增 `Templates` 集合 + `InitializeTemplates()`；7 个教学模板数据
- `Cide.Client/Views/CodeEditor.axaml.cs` — 新增 `Templates` 依赖属性；`OnEditorKeyDown` Tab 键模板展开；`TryExpandTemplate()` 单词匹配替换；`InsertTemplate()` 公共方法供移动端调用
- `Cide.Client/Views/MainView.axaml` — 工具栏新增 `ComboBox` 模板选择器；`CodeEditor` 绑定 `Templates`
- `Cide.Client/Views/MainView.axaml.cs` — `OnTemplateSelected` 处理 ComboBox 选择，调用 `CodeEditor.InsertTemplate()`

### 批次六：调用栈视图 C4（2026-04-28）

> 稳步推进，不急于见效，力求代码健壮。

| 编号 | 状态 | 核心设计决策 | 验证 |
|:---|:---|:---|:---|
| C4 | ✅ 已完成 | `CideVM::CallFrame` 新增 `funcName`；`Call` 指令压栈时通过 `funcNames_` 向量记录函数名；C API `cide_callstack_get` 结合 `sourceMap` 将 `returnIP` 映射为源码行号；前端「📞 调用栈」Tab 展示函数名 + 返回行，当前帧 `➤` 标识；点击项自动跳转高亮代码行 | 全部 CTest 8/8 通过 + C# Client 编译通过 |

### 批次六修改文件清单

**Native (C++)**
- `native/src/vm/CideVM.hpp` — `CallFrame` 新增 `funcName`；`GetCallStack()` 移入 public；新增 `funcNames_` 向量 + `RegisterFunctionName()`
- `native/src/vm/CideVM.cpp` — `RegisterFunctionName()` 实现；`Call` 指令压栈时记录 `funcName`
- `native/src/capi/cide_capi.cpp` — `SetupVM` 注册函数名；新增 `cide_callstack_count` / `cide_callstack_get`
- `native/include/cide_capi.h` — 新增调用栈 C API 声明

**Frontend (C#)**
- `Cide.Client/ViewModels/MainViewModel.cs` — 新增 `CallStackFrame` 记录 + `CallStackFrames` 集合；`LoadCallStack()` / `JumpToCallStackFrame()`
- `Cide.Client/Core/NativeMethods.cs` — 新增 `cide_callstack_count` / `cide_callstack_get` P/Invoke
- `Cide.Client/Core/CompilerService.cs` — 新增 `GetCallStackCount()` / `GetCallStackFrame()` 封装
- `Cide.Client/Views/MainView.axaml` — 右侧调试面板新增「📞 调用栈」Tab
- `Cide.Client/Views/MainView.axaml.cs` — `OnCallStackFramePressed` 点击跳转

### 批次七：执行速度滑块 C5（2026-04-28）

> 稳步推进，不急于见效，力求代码健壮。

| 编号 | 状态 | 核心设计决策 | 验证 |
|:---|:---|:---|:---|
| C5 | ✅ 已完成 | `RunCode` 改为 `async Task`；`ExecutionSpeed` 滑块 0-500ms；0ms 时调用原生 `_compiler.Run()` 全速执行，>0ms 时循环 `DoSingleStep()` + `await Task.Delay(speed)`；`StepNext` 核心逻辑提取为 `DoSingleStep()` 供单步按钮和动画模式共用；每步实时更新变量/调用栈/高亮行/visEvent | 全部 CTest 8/8 通过 + C# Client 编译通过 |

### 批次七修改文件清单

**Frontend (C#)**
- `Cide.Client/ViewModels/MainViewModel.cs` — 新增 `ExecutionSpeed`；`RunCode` → `RunCodeAsync`；提取 `DoSingleStep()` 共用逻辑
- `Cide.Client/Views/MainView.axaml` — 工具栏新增 Slider 速度滑块（🐢 0~500ms 🐇）+ 数值显示

### 批次八：Watch 表达式 C6（2026-04-28）

> 稳步推进，不急于见效，力求代码健壮。

| 编号 | 状态 | 核心设计决策 | 验证 |
|:---|:---|:---|:---|
| C6 | ✅ 已完成 | 纯前端实现，零后端改动；`EvaluateWatchExpression()` 支持变量名、数组索引 `arr[i]`、指针解引用 `*p`、取地址 `&var`；`RefreshWatchExpressions()` 在每次 `DoSingleStep()` 后自动刷新所有 Watch 值；右侧「🔍 Watch」Tab 提供输入框 + 列表 + 增删按钮 | 全部 CTest 8/8 通过 + C# Client 编译通过 |

### 批次九：AvaloniaEdit 语法高亮 C1（2026-04-28）

> 稳步推进，不急于见效，力求代码健壮。

| 编号 | 状态 | 核心设计决策 | 验证 |
|:---|:---|:---|:---|
| C1 | ✅ 已完成 | `TextBox` 替换为 `AvaloniaEdit.TextEditor`；通过 `AvaloniaEdit.TextMate` + `TextMateSharp.Grammars` 加载 `DarkPlus` 主题与 C 语言语法规则，零自定义语法文件；保留自定义 `ItemsControl` 行号区（断点/错误/警告），通过 `ScrollOffsetChanged` 事件同步编辑器与行号区垂直滚动；`CodeText`/`ErrorLines`/`WarningLines`/`BreakpointLines`/`Templates` 全部依赖属性及交互（Tab 模板展开、Popup 智能提示、断点点击）完整迁移 | C# Client + Desktop 编译通过，运行时启动正常 |

### 批次九修改文件清单

**Frontend (C#)**
- `Cide.Client/Cide.Client.csproj` — 新增 `Avalonia.AvaloniaEdit` + `AvaloniaEdit.TextMate` NuGet 包
- `Cide.Client/Views/CodeEditor.axaml` — `TextBox` → `ae:TextEditor`；行号区包入 `ScrollViewer` 以支持滚动同步
- `Cide.Client/Views/CodeEditor.axaml.cs` — 构造函数初始化 `TextMate`（`DarkPlus` 主题 + C 语法）；`Document.TextChanged`/`TextArea.KeyDown`/`ScrollOffsetChanged` 事件处理；Popup 定位改用 `TextView.GetVisualPosition`；所有文本操作从 `TextBox.Text`/`CaretIndex` 迁移到 `Document.Text`/`CaretOffset`

### 批次八修改文件清单

**Frontend (C#)**
- `Cide.Client/ViewModels/MainViewModel.cs` — 新增 `WatchExpression` 记录 + `WatchExpressions` 集合 + `NewWatchExpression`；`EvaluateWatchExpression()` / `RefreshWatchExpressions()` / `AddWatchExpressionCommand` / `RemoveWatchExpressionCommand`
- `Cide.Client/Views/MainView.axaml` — 右侧调试面板新增「🔍 Watch」Tab（输入框 + 列表 + 删除按钮）


---

## 七、新增工具

### `test-mobile.ps1` — 移动端测试一键脚本（2026-04-28）

**目的**：解决 Android 端缺少自动化构建/部署/测试工具的问题，提供从 Native `.so` 编译 → APK 打包 → 设备安装 → 应用启动 → Logcat 抓取的完整流水线。

**位置**：项目根目录 `test-mobile.ps1`

**详细文档**：[docs/BUILD_SCRIPTS.md](BUILD_SCRIPTS.md) — 包含 `build.ps1` 和 `test-mobile.ps1` 的完整参数说明、使用示例、常见问题排查（FAQ）、手动应急命令。

**功能概要**：
| 参数 | 说明 |
|:---|:---|
| `-Configuration` | `Debug` / `Release`，默认 `Debug` |
| `-SkipNativeBuild` | 跳过 NDK 原生 `.so` 编译（复用已有库） |
| `-Install` | APK 构建完成后自动安装到连接的设备/模拟器 |
| `-Run` | 安装后自动启动应用（通过 `monkey` 命令触发主 Activity） |
| `-Logcat` | 启动后实时抓取应用日志（`Ctrl+C` 停止） |

**自动检测**：
- `ANDROID_NDK_HOME` / `ANDROID_NDK_ROOT` 环境变量
- 若未设置，自动探测 Visual Studio 默认安装路径 `D:\Program Files (x86)\Microsoft Visual Studio\Shared\Android\AndroidNDK\android-ndk-r27c`
- `adb` 同理从 VS Android SDK `platform-tools` 自动探测

**关键特性**：
- 设备检测内置 **3 次自动重试**，遇到 `offline` 状态自动 `adb kill-server/start-server` 恢复
- 支持 `arm64-v8a` + `armeabi-v7a` 双架构 NDK 交叉编译

**验证状态**：Native `arm64-v8a` / `armeabi-v7a` 双架构 `.so` 编译通过；APK 构建通过（~21 MB）；脚本参数解析与设备检测逻辑正确。

**新增/变更文件**：
- `test-mobile.ps1` — **新建**
- `docs/BUILD_SCRIPTS.md` — **新建**
- `Cide.Client.Android/lib/arm64-v8a/libcide_native.so` — NDK 编译产物
- `Cide.Client.Android/lib/armeabi-v7a/libcide_native.so` — NDK 编译产物
