# C IDE

一款面向教学场景的移动端 C 语言子集 IDE。

## 核心技术栈

- **桌面前端**: C# Avalonia 11.x
- **移动前端**: .NET MAUI Blazor Hybrid + CodeMirror 6
- **前端渲染**: Skia / Avalonia Canvas（桌面）；SkiaSharp（移动端）
- **后端核心**: Rust 手写 C 子集编译器 → 自定义字节码 + CideVM 教学虚拟机
- **通信**: C API (`extern "C"`) + P/Invoke
- **构建**: Rust/Cargo + dotnet + PowerShell

## 运行平台

| 平台 | 优先级 | 状态 |
|------|--------|------|
| Android (MAUI Blazor Hybrid) | P0 | ✅ 核心功能可用 |
| Windows Desktop (Avalonia) | P1 | ✅ 开发中 |
| iOS | P2 | 后续考虑 |

## 项目结构

```
├── build.ps1                    # 日常构建脚本
├── build-release.ps1            # Release 发布构建脚本
├── test-mobile.ps1              # MAUI Android 测试流水线
├── native/                      # Rust 后端
│   ├── Cargo.toml
│   ├── src/
│   │   ├── compiler/            # Lexer / Parser / AST / TypeChecker / BytecodeGen
│   │   ├── vm/                  # CideVM 字节码解释器
│   │   ├── capi/                # C API (P/Invoke / JNI 接口)
│   │   └── diagnostics/         # 结构化诊断与自动修复建议
│   └── tests/                   # Rust 集成测试
├── Cide.Client/                 # Avalonia 共享库（桌面端）
│   ├── Core/
│   │   ├── NativeMethods.cs     # P/Invoke 声明
│   │   └── CompilerService.cs   # 编译器服务封装
│   ├── Views/                   # 视图（AXAML）
│   └── ViewModels/              # 视图模型（MVVM）
├── Cide.Client.Desktop/         # Avalonia 桌面入口
├── Cide.Client.Maui/            # MAUI Blazor Hybrid 移动端（当前主要开发目标）
│   ├── Components/              # Blazor 组件（CodeMirror 6、工具栏、面板）
│   ├── Core/                    # 复用后端（NativeMethods、CompilerService、Models）
│   ├── ViewModels/              # 适配后的 MainViewModel
│   └── wwwroot/                 # Blazor 静态资源（CSS、JS）
├── Cide.Client.Shared/          # 共享 ViewModel / 服务
├── Cide.Client.Tests/           # C# xUnit 单元测试
└── docs/                        # 设计文档与构建指南
```

## 快速开始

### 前置要求

- Rust 1.95.0+（安装 [rustup](https://rustup.rs/)）
- .NET SDK 10.0+（从 [dotnet.microsoft.com](https://dotnet.microsoft.com/) 下载）
- cargo-ndk（`cargo install cargo-ndk`）
- Android NDK（仅 Android 构建需要）

### 构建

```powershell
# 构建并运行桌面端
.\build.ps1 -Target Desktop -Run

# 构建并测试 MAUI Android（构建 → 安装 → 启动）
.\test-mobile.ps1 -Install -Run

# 快速重新打包（仅前端改动）
.\test-mobile.ps1 -SkipNativeBuild -Install -Run

# 构建 + 实时日志
.\test-mobile.ps1 -Install -Run -Logcat

# 清理并重新构建
.\build.ps1 -Clean -Target Desktop

# 构建前运行测试
.\build.ps1 -Test -Target Desktop
```

详见 [`docs/BUILD.md`](BUILD.md)。

## C 语言子集支持

### 数据类型
```c
int a;                // 标量类型（支持 signed/unsigned/const/long/short 修饰符，均映射为 int）
int arr[10];          // 一维/多维数组
int* p;               // 指针（支持算术运算 p++ / p+i / p-q）
int* p = malloc(4);   // 动态分配
char s[] = "hello";   // 字符串/字符数组
struct Node { int val; struct Node* next; };
enum Color { RED, GREEN, BLUE };
typedef int Integer;
```

### 语句
```c
if (cond) { } else { }
for (int i = 0; i < n; i++) { }   // C99 风格
while (cond) { }
do { } while (cond);
switch (x) { case 1: ... break; default: ... }
return expr;
```

### 表达式
```c
+ - * / % == != < <= > >= && || !
& | ^ ~ << >>                    // 位运算
= += -= *= /= %=
?:                               // 三目运算符
arr[i]  foo(a,b)  &a  *p  node.val  node->val  ++a  a++
sizeof(int)  sizeof(struct S)    // sizeof
```

## 设计文档

- [`DESIGN.md`](DESIGN.md) — 总体架构设计
- [`C_SUBSET_SPEC.md`](C_SUBSET_SPEC.md) — C 语言子集规范
- [`CUSTOM_VM_DESIGN.md`](CUSTOM_VM_DESIGN.md) — 自定义 CideVM 虚拟机设计
- [`UX_DIAGNOSTICS_DESIGN.md`](UX_DIAGNOSTICS_DESIGN.md) — 友好中文提示与智能修复
- [`ZERO_INTRUSIVE_VISUALIZATION.md`](ZERO_INTRUSIVE_VISUALIZATION.md) — 零侵入可视化
- [`ALGORITHM_DATASTRUCTURE_DESIGN.md`](ALGORITHM_DATASTRUCTURE_DESIGN.md) — 算法与数据结构支持
- [`MOBILE_TABLET_ADAPTATION.md`](MOBILE_TABLET_ADAPTATION.md) — 移动端与平板适配
- [`BUILD.md`](BUILD.md) — 构建指南
