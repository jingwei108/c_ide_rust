# C IDE

一款面向教学场景的移动端 C 语言子集 IDE。

## 核心技术栈

- **桌面前端**: C# Avalonia 11.x
- **移动前端**: .NET MAUI Blazor Hybrid + CodeMirror 6
- **前端渲染**: Skia / Avalonia Canvas（桌面）；SkiaSharp（移动端）
- **后端核心**: C++20 手写 C 子集编译器 → 自定义字节码 + CideVM 教学虚拟机
- **通信**: C API (`extern "C"`) + P/Invoke
- **构建**: CMake + dotnet + PowerShell

## 运行平台

| 平台 | 优先级 | 状态 |
|------|--------|------|
| Android (MAUI Blazor Hybrid) | P0 | ✅ 核心功能可用 |
| Windows Desktop (Avalonia) | P1 | ✅ 开发中 |
| iOS | P2 | 后续考虑 |

## 项目结构

```
├── build.ps1                    # 一键构建脚本
├── test-mobile.ps1              # MAUI Android 测试流水线
├── native/                      # C++ 后端
│   ├── CMakeLists.txt
│   ├── include/
│   │   └── cide_capi.h         # C API 头文件
│   ├── third_party/
│   │   └── wasm3/               # ✅ 已集成
│   └── src/
│       ├── compiler/            # C 子集 → WASM 编译器（Phase 2）
│       ├── runtime/             # wasm3 宿主封装（Phase 2）
│       ├── diagnostics/         # 诊断系统（Phase 3）
│       └── capi/
│           └── cide_capi.cpp    # C API 实现
├── Cide.Client/                 # Avalonia 共享库（桌面端）
│   ├── Core/
│   │   ├── NativeMethods.cs     # P/Invoke 声明
│   │   ├── CompilerService.cs   # 编译器服务封装
│   │   └── Responsive/          # 响应式布局
│   ├── Views/                   # 视图（AXAML）
│   └── ViewModels/              # 视图模型（MVVM）
├── Cide.Client.Desktop/         # Avalonia 桌面入口
└── Cide.Client.Maui/            # MAUI Blazor Hybrid 移动端（当前主要开发目标）
    ├── Components/              # Blazor 组件（Monaco Editor、工具栏、面板）
    ├── Core/                    # 复用后端（NativeMethods、CompilerService、Models）
    ├── ViewModels/              # 适配后的 MainViewModel
    └── wwwroot/                 # Blazor 静态资源（Monaco、CSS、JS）
```

## 快速开始

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
```

### 开发阶段

- **Phase 1**: 基础架构（项目脚手架、C API、Avalonia 响应式布局、最小 wasm3 原型）
- **Phase 2**: C 子集编译器（Lexer/Parser/AST/TypeChecker/WASM CodeGen）
- **Phase 3**: 诊断与可视化（中文错误消息、QuickFix、零侵入可视化注入）
- **Phase 4**: 算法与数据结构（算法模式识别、运行时验证、数据结构诊断）
- **Phase 5**: 移动端优化（触控手势、横竖屏切换、性能优化）
- **Phase 6**: ~~OCR 导入~~（已移除，相关代码已清理）

## C 语言子集（Phase 1 MVP）

```c
// 数据类型
int a;                // 唯一标量类型
int arr[10];          // 一维数组
int* p;               // 指针
int* p = malloc(4);   // 动态分配
struct Node { int val; struct Node* next; };

// 语句
if (cond) { } else { }
for (int i = 0; i < n; i++) { }   // C99 风格
while (cond) { }
return expr;

// 表达式
+ - * / % == != < <= > >= && || !
= += -= *= /= %=
arr[i]  foo(a,b)  &a  *p  node.val  node->val  ++a  a++
```

## 设计文档

详见项目根目录下的 Markdown 文件：

- `DESIGN.md` — 总体架构设计
- `C_SUBSET_SPEC.md` — C 语言子集规范
- `CUSTOM_VM_DESIGN.md` — 自定义 CideVM 虚拟机设计
- `UX_DIAGNOSTICS_DESIGN.md` — 友好中文提示与智能修复
- `ZERO_INTRUSIVE_VISUALIZATION.md` — 零侵入可视化
- `ALGORITHM_DATASTRUCTURE_DESIGN.md` — 算法与数据结构支持
- `MOBILE_TABLET_ADAPTATION.md` — 移动端与平板适配
