# C IDE 项目设计文档

> 一款面向教学场景的移动端 C 语言子集 IDE
> 核心技术：C# Avalonia 前端 + C++ 后端（手写 C 子集编译器 → 自定义字节码 + CideVM 教学虚拟机）

---

## 目录

- [1. 项目概述](#1-项目概述)
- [2. 核心架构](#2-核心架构)
- [3. C 语言子集](#3-c-语言子集)
- [4. 后端设计](#4-后端设计)
- [5. 前端设计](#5-前端设计)
- [6. 诊断与修复系统](#6-诊断与修复系统)
- [7. 算法与数据结构支持](#7-算法与数据结构支持)
- [8. 零侵入可视化](#8-零侵入可视化)
- [9. 移动端与平板适配](#9-移动端与平板适配)
- [10. ~~OCR 照片导入~~（已移除）](#10-ocr-照片导入)
- [11. 开发阶段](#11-开发阶段)

---

## 1. 项目概述

### 1.1 目标

构建一款面向移动端的 **C 语言子集 IDE**，核心特点：

- **友好中文提示**：所有编译错误、运行时异常、诊断信息均为中文，附带行号列号。
- **一键修复**：根据错误类型自动生成代码修复建议。
- **关联知识卡片**：遇到错误时弹出相关知识卡片（概念讲解 + 代码示例）。
- **内存视图**：可视化展示变量、数组、指针在虚拟内存中的布局。
- **指针/错误视图**：图形化展示指针指向关系，代码中标注错误位置。
- **零侵入可视化**：写纯 C 代码，系统自动识别算法并展示动画。
- **后续扩展**：子集渐进式解锁、知识图谱系统。

### 1.2 运行平台

| 平台 | 优先级 | 技术方案 |
|------|--------|---------|
| Android (手机/平板) | P0 | .NET MAUI Blazor Hybrid + C++ `.so` + P/Invoke |
| Windows Desktop | P1 | Avalonia + 调试与开发主力平台 |
| iOS | P2 | 后续考虑 |

### 1.3 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| 前端 UI | Avalonia 11.x + C# 12（桌面）；MAUI Blazor Hybrid（移动） | 跨平台，参考 2048 验证 Android 可行性 |
| 前端渲染 | Skia / Avalonia Canvas（桌面）；SkiaSharp（移动） | **放弃 OpenGL**，避免移动端闪退 |
| 后端核心 | C++20 | 手写 C 子集编译器 → 自定义字节码 + CideVM 教学虚拟机 |
| 通信 | C API (`extern "C"`) | 统一 P/Invoke（Android/Desktop） |
| 构建 | CMake + dotnet + PowerShell | 与参考项目保持一致 |

### 1.4 参考项目经验

| 来源 | 关键经验 | 本项目应用 |
|------|---------|-----------|
| **VisualBinaryTree.Desktop** | C 子集解释器（Lexer/Parser/AST/TypeChecker/VM）、C API 边界设计、双模式执行（编译/解释） | 参考其 C 子集范围和编译器分层设计；后端最终采用自研 CideVM |
| **2048** | Avalonia Android + 浏览器入口、Canvas + DispatcherTimer 动画、CancelAll+Snap 防闪退模式、Android 启动画面 workaround | **放弃 OpenGL** 用 Canvas；动画并发处理；触控手势；移动端适配 |

---

## 2. 核心架构

### 2.1 架构总览

```
+-----------------------------------------------------------------------------+
|                     C# Avalonia 前端 (Android / Desktop)                     |
|  +-------------+  +-------------+  +-------------------------------------+  |
|  | CodeEditor  |  | MemoryView  |  | KnowledgeCard / QuickFixPanel       |  |
|  |  代码编辑器  |  |  内存视图    |  | 知识卡片 / 一键修复面板               |  |
|  +-------------+  +-------------+  +-------------------------------------+  |
|  +-------------+  +-------------+  +-------------------------------------+  |
|  | PointerView |  | ErrorPanel  |  | ConsoleOutput / AlgoCanvas          |  |
|  |  指针视图    |  |  错误面板    |  | 输出控制台 / 算法动画画布             |  |
|  +-------------+  +-------------+  +-------------------------------------+  |
+-----------------------------------------------------------------------------+
                                    |
                                    v P/Invoke (统一 C API)
+-----------------------------------------------------------------------------+
|                        C++ 后端 (Native DLL / .so)                          |
|                                                                             |
|  +---------------------------------------------------------------------+    |
|  | ① C 子集编译器                                                       |    |
|  |   用户 C 代码 → Lexer → Parser → AST → TypeChecker → BytecodeGen    |    |
|  |   输出：自定义字节码指令序列 + 符号表 + 字符串数据段                   |    |
|  +---------------------------------------------------------------------+    |
|                                    |                                        |
|  +---------------------------------------------------------------------+    |
|  | ② CideVM 教学虚拟机（自研）                                          |    |
|  |   加载字节码 → 解释执行 → 捕获 trap → StepEvent 单步暂停             |    |
|  |   提供内存视图、指针追踪、执行步进、中文错误映射                        |    |
|  +---------------------------------------------------------------------+    |
|                                    |                                        |
|  +---------------------------------------------------------------------+    |
|  | ③ 诊断与可视化引擎                                                   |    |
|  |   源码位置映射 / 内存布局元数据 / 指针追踪表 / 中文错误消息             |    |
|  |   算法模式识别 / 运行时验证 / 执行轨迹分析                             |    |
|  +---------------------------------------------------------------------+    |
+-----------------------------------------------------------------------------+
```

### 2.2 关键技术定位

**CideVM 在本项目中的角色**：
- **教学专用执行引擎**：为 C 子集量身定制的轻量级虚拟机，不是通用 WASM 解释器
- 用户代码编译为扁平字节码，在 **CideVM** 中逐条解释执行
- 利用线性内存隔离和指令级边界检查保证安全
- 前端统一使用 **P/Invoke**，Android 和 Desktop 完全一致

**从 wasm3 到 CideVM 的演进**：

项目初期采用 **wasm3** 作为执行引擎（~50KB 纯 C，WASM 解释器），在 Phase 2 完成了编译器到 WASM 的生成。随着 Phase 3 深入，发现 wasm3 作为通用 WASM 解释器存在以下教学场景的瓶颈：

| 能力 | wasm3 现状 | CideVM 改进 |
|:---|:---|:---|
| **单步调试** | 无法暂停/恢复，只能阻塞宿主函数 | 每条指令后可检查 `paused` 标志，同步单步 |
| **运行时中文诊断** | 只能翻译英文 trap 字符串 | 在除零/越界现场直接读取变量值，生成 "当 i=5 时，arr[10] 越界" |
| **内存可视化** | `m3_GetMemory` 读原始字节，不知道变量名 | VM 自带符号表，知道 `0x1020` 是 `arr[2]` |
| **零侵入可视化** | 需注入 `__cide_step` 等 host call | VM 层直接发射 `StepEvent`，无需修改用户代码 |
| **执行步数限制** | 需 patch `m3_Yield` | 原生支持，更精确 |
| **安全隔离** | 自动内存隔离 | 自己检查边界，同等安全 |

**CideVM 核心优势**：
- 完全可控的指令集（~30 条指令），只实现教学子集真正用到的特性
- 局部变量映射到线性内存，支持 `&x` 取地址（这是 wasm3 架构下难以实现的）
- 函数调用栈帧在 `memory_` 中分配，指针/数组/结构体语义与真实 C 完全一致
- 零线程：单步在主线程同步执行，彻底消除线程泄漏风险

### 2.3 目录结构

```
c-ide/
├── build.ps1                          # 一键构建脚本
├── README.md
├── native/                            # C++ 后端
│   ├── CMakeLists.txt
│   ├── include/
│   │   └── cide_capi.h               # C API 头文件
│   ├── src/
│   │   ├── compiler/                  # C 子集 → 字节码编译器
│   │   │   ├── Lexer.cpp / .hpp
│   │   │   ├── Parser.cpp / .hpp
│   │   │   ├── Ast.hpp
│   │   │   ├── TypeChecker.cpp / .hpp
│   │   │   ├── BytecodeGen.cpp / .hpp    # AST → CideVM 字节码
│   │   │   └── SourceMap.cpp / .hpp
│   │   ├── vm/                        # CideVM 教学虚拟机
│   │   │   ├── CideVM.cpp / .hpp
│   │   │   ├── OpCode.hpp
│   │   │   └── Instruction.hpp
│   │   ├── diagnostics/               # 诊断系统
│   │   │   ├── ErrorCodes.hpp
│   │   │   ├── DiagnosticEngine.cpp / .hpp
│   │   │   ├── QuickFixGenerator.cpp / .hpp
│   │   │   └── AlgorithmMatcher.cpp / .hpp
│   │   └── capi/
│   │       └── cide_capi.cpp         # C API 桥接层
│   └── tests/                         # 测试套件
│       ├── Phase2RegressionTest.cpp
│       ├── Phase3Batch1Test.cpp      # 数组/指针/结构体
│       ├── Phase3Batch2Test.cpp      # 运行时错误
│       ├── Phase3Batch3Test.cpp      # 内存视图
│       ├── Phase3Batch4Test.cpp      # printf/scanf
│       └── NewFeaturesTest.cpp
├── Cide.Client/                       # Avalonia 共享库
│   ├── Core/
│   │   ├── CompilerService.cs
│   │   ├── NativeMethods.cs
│   │   ├── Diagnostics/
│   │   │   ├── ErrorCatalog.cs
│   │   │   ├── KnowledgeBase.cs
│   │   │   └── QuickFixEngine.cs
│   │   ├── OcrCorrection/
│   │   │   ├── OcrConfusionMap.cs
│   │   │   └── CompilerDrivenCorrector.cs
│   │   └── Responsive/
│   │       └── Breakpoints.cs
│   ├── Views/
│   │   ├── MainWindow.axaml
│   │   ├── CodeEditor.axaml
│   │   ├── MemoryCanvas.axaml.cs
│   │   ├── PointerCanvas.axaml.cs
│   │   ├── ErrorPanel.axaml
│   │   ├── KnowledgeCard.axaml
│   │   ├── ConsoleOutput.axaml
│   │   └── layouts/                   # 响应式布局
│   │       ├── PhonePortraitLayout.axaml
│   │       ├── PhoneLandscapeLayout.axaml
│   │       ├── TabletPortraitLayout.axaml
│   │       └── TabletLandscapeLayout.axaml
│   └── ViewModels/
│       ├── MainViewModel.cs
│       └── ResponsiveLayoutViewModel.cs
├── Cide.Client.Desktop/
└── docs/
    ├── DESIGN.md
    ├── C_SUBSET_SPEC.md
    ├── CUSTOM_VM_DESIGN.md
    ├── UX_DIAGNOSTICS_DESIGN.md
    ├── MOBILE_TABLET_ADAPTATION.md
    ├── ARCHIVE_OCR_IMPORT_DESIGN.md
    ├── ALGORITHM_DATASTRUCTURE_DESIGN.md
    ├── ZERO_INTRUSIVE_VISUALIZATION.md
    └── INCIDENT_*.md                  # 事故复盘记录
```

---

## 3. C 语言子集

### 3.1 Phase 1 MVP 子集

```c
// 数据类型
int a;                // 唯一标量类型
int a = 5;
int arr[10];          // 一维数组
int arr[] = {1,2,3};
int* p;               // 指针
int* p = &a;
int* p = malloc(4);   // 动态分配（4 = sizeof(int)）
struct Node {          // 结构体
    int val;
    struct Node* next;
};

// 语句
if (cond) { } else { }
for (int i = 0; i < n; i++) { }   // C99 风格
while (cond) { }
return expr;
expr;
{ stmt... }           // 块作用域

// 表达式
+ - * / % == != < <= > >= && || !
= += -= *= /= %=
arr[i]                // 数组索引
foo(a, b)             // 函数调用
&a                    // 取地址
*p                    // 解引用
node.val / node->val  // 结构体访问（行为一致）
++a / a++             // 自增自减
```

### 3.2 明确不支持

| 特性 | 遇到时的中文提示 |
|------|---------------|
| `break` / `continue` | "暂不支持 break，请改用 return 或调整循环条件" |
| `float` / `double` / `char` | "教学子集暂只支持 int 类型" |
| 指针运算 (`p++`) | "教学子集不支持指针运算，请使用数组索引 arr[i] 代替" |
| 预处理 (`#include`) | "解释器模式下无需 #include，直接编写代码即可" |
| 标准库 (`printf`) | "未定义函数 'printf'，本 IDE 暂不支持标准库函数" |
| 多维数组 | "暂不支持多维数组，请使用一维数组模拟" |

> 详细规范见 `C_SUBSET_SPEC.md`

---

## 4. 后端设计

### 4.1 编译器流程

```
源代码字符串
    |
    v
Lexer::Tokenize() -> vector<Token>
    |
    v
Parser::Parse() -> unique_ptr<ProgramNode> (AST)
    |
    v
TypeChecker::Check()
    |
    v
BytecodeGen::Generate() -> vector<Instruction> (CideVM 字节码)
    |
    v
SourceMap 生成 + 字符串数据段收集
```

**BytecodeGen 与旧 WasmCodeGen 的区别**：
- 输出从 `vector<uint8_t>` (WASM 二进制) 改为 `vector<Instruction>` (扁平指令序列)
- 指令集从 WASM 的 ~100 条压缩到教学子集实际需要的 ~30 条
- 函数调用从 WASM 的间接调用表改为直接索引调用
- 新增 `StepEvent` 指令，天然支持单步调试，无需注入 host function

### 4.2 CideVM 执行模型

CideVM 是栈式虚拟机，核心循环逐条解释执行 `Instruction`：

```cpp
enum class OpCode : uint8_t {
    PushConst, LoadLocal, StoreLocal, LoadGlobal, StoreGlobal,
    LoadMem, StoreMem, LoadMemByte, StoreMemByte,
    Add, Sub, Mul, Div, Mod, Neg,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or, Not,
    Jump, JumpIfZero, JumpIfNotZero,
    Call, CallHost, Ret, RetVoid,
    StepEvent, GetFrameBase
};

struct Instruction {
    OpCode op;
    int32_t operand;
    SourceLoc loc;    // 源码位置，用于错误映射
};
```

**执行示例**：
```cpp
// 用户代码
int main() {
    int a = 5;
    int* p = &a;
    *p = 10;
    return a;
}

// 生成的字节码（简化）
PushConst 5
StoreLocal 0          // a = 5
GetFrameBase
PushConst 0
Add                   // &a
StoreLocal 1          // p = &a
LoadLocal 1           // p
PushConst 10
StoreMem              // *p = 10
LoadLocal 0           // a
Ret
```

### 4.3 内存布局（CideVM Linear Memory）

CideVM 使用 256KB 线性内存，划分如下：

```
地址空间
|- 0x0000~0x0FFF: 保留（NULL 指针陷阱区，load/store 触发 trap）
|- 0x1000~0x4FFF: 字符串字面量区 + 全局变量区
|- 0x5000~0x0FFFF: 堆区（malloc 管理）
|- 0x10000~0x3FFFF: 栈区（函数调用帧，向下增长）
```

**关键设计：局部变量在内存中**

与 wasm3 时代不同，CideVM 将局部变量也存储在线性内存的栈区域中：

```cpp
// Call 指令：在 memory_ 中分配栈帧
uint32_t frameSize = localCount * 4;
memStackTop_ -= frameSize;           // 从高地址向下增长
// 参数从表达式栈 pop 到 memory_ 帧中
// 剩余局部变量零初始化

// LoadLocal：从 memory_ 读取
uint32_t addr = frame.localsBase + localIndex * 4;
Push(ReadI32LE(memory_.data() + addr));

// StoreLocal：向 memory_ 写入
uint32_t addr = frame.localsBase + localIndex * 4;
WriteI32LE(memory_.data() + addr, Pop());
```

这样做的好处：
- `&x` 直接返回 `localsBase + index * 4`，是真实的内存地址
- `scanf("%d", &x)` 可以直接写入
- 数组和指针语义与真实 C 完全一致

### 4.4 运行时诊断与错误处理案例

CideVM 在指令级捕获所有运行时错误，并生成中文诊断信息：

**案例 1：除零错误**
```cpp
int a = 15, b = 0;
int c = a / b;   // 😵 除零错误：15 / 0。请检查除数是否可能为零。
```

**案例 2：NULL 指针解引用**
```cpp
int* p = 0;
*p = 10;         // 访问了 NULL 指针区域（地址 0x0000）。NULL 指针不能解引用。
                 // 请确认指针已被正确初始化。
```

**案例 3：数组越界**
```cpp
int arr[5];
arr[10] = 1;     // 内存访问越界：地址 0x8028，有效范围 0x0000~0x10000。
                 // 当 i=10 时，arr[10] 越界了。数组大小是 5。
```

**案例 4：栈溢出**
```cpp
// 无限递归
int f() { return f(); }
// Call: 栈溢出。函数调用层数超过限制。
```

**案例 5：无限循环熔断**
```cpp
while (1) {}     // 程序执行步数超过限制（10000000步），可能包含无限循环。
```

### 4.5 C API 接口

```cpp
// 会话管理
CideSession* cide_session_create();
void cide_session_destroy(CideSession* s);

// 编译
int cide_compile(CideSession* s, const char* source);
const char* cide_get_compile_errors(CideSession* s);

// 执行
int cide_run(CideSession* s);
int cide_step_next(CideSession* s);   // 单步执行（同步，无线程）
const char* cide_get_runtime_error(CideSession* s);

// 输出
int cide_get_output_length(CideSession* s);
void cide_get_output(CideSession* s, char* buf, int max_len);

// 内存视图
int cide_memory_region_count(CideSession* s);
void cide_memory_region_get(CideSession* s, int index,
    uint32_t* addr, int* size, char* name, int name_size,
    char* type, int type_size, int* is_heap, int* is_freed);
int cide_memory_get_value(CideSession* s, uint32_t addr, int* out_val);
int cide_memory_get_pointer_target(CideSession* s, uint32_t addr, uint32_t* out_target);

// 诊断与修复
int cide_diagnostic_count(CideSession* s);
void cide_diagnostic_get(CideSession* s, int index,
    int* line, int* column, int* error_code,
    char* message, int msg_size, char* fix_suggestion, int fix_size);

// 执行轨迹（用于算法分析）
int cide_trace_count(CideSession* s);
void cide_trace_get(CideSession* s, int index, int* line, char* operation, int op_size);
```

> 详细 VM 设计见 `CUSTOM_VM_DESIGN.md`

---

## 5. 前端设计

### 5.1 响应式布局

```csharp
// 断点定义
public enum LayoutBreakpoint {
    Compact,     // < 600px  -> 手机
    Medium,      // 600~1024px -> 小平板
    Expanded,    // 1024~1280px -> 大平板
    Wide         // > 1280px -> 桌面
}
```

| 设备 | 布局 |
|------|------|
| **手机竖屏** | 底部导航 Tab（4 个：代码/运行/内存/错误）+ 全屏页面 |
| **手机横屏** | 左右分栏：代码 + 输出 |
| **平板竖屏** | 编辑器全宽 + 底部可视化 Tab |
| **平板横屏** | **三栏：文件(240px) | 编辑器(自适应) | 可视化(300px)** |
| **桌面** | 三栏固定 + 最高信息密度 |

### 5.2 触控优化

- 最小触控区域：**48dp**
- 代码编辑器：增大行高和字体（手机 16px），底部符号工具栏
- 手势：滑动切换 Tab、长按上下文菜单、双击选中单词
- 虚拟键盘：弹出时自动滚动到光标、隐藏底部导航

> 详细设计见 `MOBILE_ADAPTATION.md`

---

## 6. 诊断与修复系统

### 6.1 三级信息架构

| 级别 | 呈现方式 | 内容 |
|------|---------|------|
| **L1 感知** | 代码行内弹窗 | 表情 + 一句话 + 修复按钮 |
| **L2 理解** | 底部面板展开 | 代码片段 + 通俗解释 + 对比 + 生活类比 |
| **L3 原理** | 知识卡片弹窗 | 内存动画 + 概念详解 + 练习题 |

### 6.2 运行时诊断优势

利用 CideVM 符号表读取实际运行时值：

| 错误 | 静态分析只能说 | 运行时诊断能说 |
|------|-------------|---------------|
| 数组越界 | "索引可能越界" | "当 i=10 时越界了。数组大小是 5。当前 n=10。" |
| 空指针 | "p 可能未初始化" | "p 的值是 0x00000000。声明于第 3 行，之后无赋值。" |
| 无限循环 | "循环条件可能恒真" | "循环已执行 100,000 步。i 始终是 1。你注释掉了 i++。" |

### 6.3 修复分级与结构化自动修复

| 级别 | 类型 | 示例 | 自动？ |
|------|------|------|--------|
| **L1 语法修复** | 语法错误 | 补分号 `;`、补括号 `}`/`)`/`]`、`\|`→`\|\|`、`&`→`&&` | ✅ 全自动（后端结构化修复） |
| **L2 语义修复** | 常见逻辑错误 | 改 `<=` 为 `<`、加初始化 | ✅ 全自动（后端结构化修复） |
| **L3 逻辑建议** | 隐藏逻辑错误 | `=` vs `==`、死代码 | 预览确认 |
| **L4 教学引导** | 算法设计错误 | 递归边界、排序逻辑 | 仅建议 |

**后端结构化修复架构**：

```
Lexer/Parser/TypeChecker 报错
    |
    v
MakeDiagnostic(source) ──→ PopulateStructuredFix(d, source)
    |                          ├── SplitSourceLines(source)
    |                          ├── 按 errorCode 选择修复策略
    |                          └── 填充 fixKind / replaceRange / replacementText
    v
CideDiagnostic (含结构化 fix 数据)
    |
    v  P/Invoke
C# Diagnostic (FixKind, ReplaceStartLine/Column, ReplaceEndLine/Column, ReplacementText)
    |
    v
CodeFixService.TryApplyFix()
    ├── FixKind.ReplaceText → ApplyStructuredReplace()（精确字符级替换）
    ├── FixKind.InsertText  → ApplyStructuredReplace()（精确字符级插入）
    ├── FixKind.ManualHint  → 显示修复提示，不自动修改
    └── fallback → ApplyLegacyFix()（字符串匹配）
```

**已实现的结构化修复**：

| 错误码 | 触发场景 | fixKind | replacementText |
|:---|:---|:---|:---|
| `E2005_ExpectedSemicolon` | 缺少 `;` | `InsertText` | `;` |
| `E2006_ExpectedClosingBrace` | 缺少 `}` | `InsertText` | `}` |
| `E2007_ExpectedClosingParen` | 缺少 `)` | `InsertText` | `)` |
| `E2008_ExpectedClosingBracket` | 缺少 `]` | `InsertText` | `]` |
| `E1004_UnsupportedOp` | `\|`/`&` 单目误用 | `ReplaceText` | `\|\|` / `&&` |

> 详细实现见 `ARCHIVE_STRUCTURED_AUTO_FIX_20260505.md`

### 6.4 ~~OCR 导入纠错~~（已移除）

> OCR 相关代码已清理，该功能不再在路线图内。历史设计见归档文档 `ARCHIVE_OCR_IMPORT_DESIGN.md`。

---

## 7. 算法与数据结构支持

### 7.1 算法修复（不是代写代码，是智能诊断 + 引导）

| 层级 | 策略 | 方式 |
|------|------|------|
| **L1 模式识别** | AST 结构匹配 | "识别出你在写冒泡排序，外层循环应该是 i < n-1" |
| **L2 运行时验证** | 自动生成测试用例 | "测试 [5,3,8,1,2] 后元素 8 丢失了" |
| **L3 轨迹分析** | 记录比较/交换/递归调用 | "第 3 趟没有比较 arr[2] 和 arr[3]" |

**核心原则**：算法修复的目的是**教懂学生算法逻辑**，不是**代写代码**。

### 7.2 数据结构支持

当前子集（int + 指针 + struct + malloc）已支持：

| 数据结构 | 实现方式 | 可视化 |
|---------|---------|--------|
| 数组 / 动态数组 | 原生 int[] / malloc | vis_array() |
| **单链表** | struct Node { int val; Node* next; } | vis_list() |
| **双链表** | struct DNode { int val; DNode *prev, *next; } | vis_list() |
| **栈 / 队列** | 数组 + 索引 或 链表 | vis_stack() / vis_queue() |
| **二叉树** | struct TreeNode { int val; TreeNode *left, *right; } | vis_tree() |

### 7.3 子集扩展路线图

```
Phase 1（默认开放）: 变量、数组、指针、struct、if/for/while、函数、malloc
       |
       v 完成「数组排序」练习
Phase 2 解锁: break/continue、sizeof、字符串字面量、vis_* 可视化
       |
       v 完成「链表基础」练习
Phase 3 解锁: 多维数组、typedef、枚举、函数指针
       |
       v 完成「二叉树遍历」练习
Phase 4 解锁: 字符串操作、文件 I/O、标准库子集
```

> 详细设计见 `ALGORITHM_DATASTRUCTURE_DESIGN.md`

---

## 8. 零侵入可视化

### 8.1 核心设计

> **初学者写纯 C 代码，编译器自动识别算法模式，自动注入可视化指令。**

```c
// 用户写的代码（纯净的 C）
void bubbleSort(int arr[], int n) {
    for (int i = 0; i < n - 1; i++) {
        for (int j = 0; j < n - i - 1; j++) {
            if (arr[j] > arr[j + 1]) {
                int temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
        }
    }
}
```

**系统自动**：
- 检测到双重循环 + 相邻比较 + 交换 -> 识别为「冒泡排序」
- 自动在字节码中注入 VisEvent（compare/swap/update）指令
- 用户看不到任何可视化代码

### 8.2 三种模式

| 模式 | 用户代码 | 适用人群 |
|------|---------|---------|
| **自动**（默认） | 纯 C，无任何额外代码 | 初学者 |
| **引导** | 纯 C + // @vis: 注释 | 进阶学习者 |
| **手动** | C + vis_*() 函数 | 教师/高级用户 |

> 详细设计见 `ZERO_INTRUSIVE_VISUALIZATION.md`

---

## 9. 移动端与平板适配

### 9.1 设备布局

```
平板横屏（主力学习场景）
+--------------+--------------------------+---------------+
|  文件         | 代码编辑器                |  [内存视图]   |
| main.c       | （自适应宽度）            |  [指针视图]   |
|  模板         |                          |  [变量面板]   |
+--------------+--------------------------+---------------+
| 运行  | 输出: 排序完成 [1,2,3,5,8]                      |
+---------------------------------------------------------+

手机竖屏（碎片化学习）
+-----------------+
| 代码编辑器       |
+-----------------+
| [运行]          |
+-----------------+
| 底部导航 Tab    |
+-----------------+
```

### 9.2 性能优化

- 移动端动画降频至 30fps（省电）
- 内存视图最多显示 64 个格子（手机）
- 快速切换时 CancelAllAnimations() + SnapToFinalState()（参考 2048 防闪退）

> 详细设计见 `MOBILE_TABLET_ADAPTATION.md`

---

## 10. ~~OCR 照片导入~~（已移除）

> OCR 相关代码已于 2026-05-04 清理移除。历史设计见归档文档 `ARCHIVE_OCR_IMPORT_DESIGN.md`。

---

## 11. 开发阶段

### Phase 1: 基础架构（✅ 已完成）
- [x] 项目脚手架：build.ps1, CMake, .csproj, 目录结构
- [x] C API 接口定义：cide_capi.h
- [x] Avalonia 共享项目 + Android 入口 + 响应式布局框架
- [x] 代码编辑器基础（行号 + 基础高亮 + 触控优化）
- [x] **最小 wasm3 原型**：加载简单 WASM 模块，P/Invoke 验证全链路

### Phase 2: C 子集编译器（✅ 已完成）
- [x] Lexer + Parser + AST + TypeChecker
- [x] ~~WASM CodeGen~~ → **BytecodeGen**（已迁移到 CideVM 扁平字节码）
- [x] CideVM 核心实现（~30 条指令解释器）
- [x] 虚拟内存管理 + 指针追踪（局部变量映射到线性内存）
- [x] Source Map 生成 + `StepEvent` 单步指令

> 详细完成报告见 `PHASE2_COMPLETION.md`
> wasm3 → CideVM 迁移记录见 `CUSTOM_VM_DESIGN.md`

### Phase 3: 诊断与可视化（✅ 基础框架已完成，Stage 2 精确诊断增强中）
- [x] 中文错误消息系统（L1/L2/L3 基础）
- [x] QuickFix 引擎（语法/语义自动修复）
- [x] 知识卡片系统（Markdown + 内存 Canvas 图）
- [x] 零侵入可视化注入引擎（5 种核心算法规则）
- [x] 内存视图 Canvas + 指针视图 Canvas
- [x] **单步调试**：`StepEvent` 指令级暂停，同步执行，零线程风险
- [ ] **运行时诊断增强**：基础除零/越界/NULL 诊断已实现，精确到变量值的诊断正在 Stage 2 中实现

### Phase 4: 算法与数据结构（3-4 周）
- [ ] 算法模式识别系统
- [ ] 运行时验证（Property-based Testing）
- [ ] 数据结构专用诊断（断链检测、泄漏检测、递归深度）
- [ ] 内置可视化函数（vis_array, vis_list, vis_tree）
- [ ] Starter Code 模板库

### Phase 5: 移动端优化（2-3 周）
- [ ] Android 触控手势 + 虚拟键盘适配
- [ ] 横竖屏切换状态保持
- [ ] 性能优化（降帧率、简化渲染）
- [ ] 动画稳定性（CancelAll + Snap 模式）

### Phase 6: ~~OCR 导入~~（已移除）
> OCR 相关代码已清理，该功能不再在路线图内。

### Phase 7: 扩展（后续）
- [ ] 子集渐进式解锁（break/continue、多维数组、字符串）
- [ ] 知识图谱系统
- [ ] 社区贡献算法模板

---

## 关键设计决策总结

| 决策点 | 选择 | 理由 |
|--------|------|------|
| 执行引擎 | **自研 CideVM（替代 wasm3）** | 教学专用：完全可控的单步/诊断/内存可视化；局部变量映射到线性内存支持 `&x` |
| 编译目标 | **自定义扁平字节码（替代 WASM）** | 只实现教学子集需要的 ~30 条指令；简化编译器和 VM 的耦合 |
| 可视化方式 | **零侵入自动注入** | 初学者写纯 C，系统自动识别算法 |
| 渲染引擎 | **Avalonia Canvas（放弃 OpenGL）** | 参考 2048，避免移动端闪退 |
| 动画稳定性 | **CancelAll + SnapToFinalState** | 参考 2048 修复经验 |
| 中文支持 | **三级信息 + 运行时值注入** | L1 感知/L2 理解/L3 原理 |
| 算法修复 | **诊断 + 引导，不代写代码** | 保护学习过程 |
| 子集扩展 | **渐进式解锁** | 按需开放，降低认知负担 |
| ~~OCR 纠错~~ | ~~编译器驱动反馈循环~~ | ~~形式语法验证比 NLP 猜测更可靠~~ |
