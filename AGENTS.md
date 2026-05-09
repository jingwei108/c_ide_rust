# Cide 项目 Agent 指南

## 项目概览

Cide 是一个跨平台 C 语言 IDE，包含：

- **前端**：.NET MAUI (Android) + Avalonia (Desktop)
- **后端**：共享 C++ native 编译器/VM (`cide_native`)
- **编译管线**：Lexer → Parser → TypeChecker → BytecodeGen → CideVM

## 技术栈

| 层级 | 技术 |
|------|------|
| Android | .NET 10 MAUI BlazorWebView + CodeMirror6 |
| Desktop | Avalonia 11.3.0 + .NET 10 |
| Native | C++17, CMake, Ninja |
| VM | 自定义字节码解释器，256KB 线性内存 |

## 关键目录

```
native/src/compiler/    Lexer, Parser, TypeChecker, BytecodeGen, AST
native/src/vm/          CideVM 字节码解释器
native/src/capi/        C API (P/Invoke / JNI 接口)
native/src/diagnostics/ 结构化诊断、自动修复建议
Cide.Client/            Avalonia 桌面端共享代码
Cide.Client.Desktop/    Avalonia 桌面端入口
Cide.Client.Maui/       MAUI 移动端
Cide.Client.Shared/     共享 ViewModel / 服务
docs/                   设计文档、事故报告
```

## 编码约定

### C++ (native)
- 使用 `std::make_unique` 管理 AST 节点
- 错误处理：`Consume()` 失败时不抛异常，记录错误并返回当前 token（不消费）
- Parser 循环必须检测零进度：`if (pos_ == checkpoint) Advance();`
- 类型系统：`Type` 结构体使用 `TypeKind` + `baseKind` 表示指针/数组元素类型

### C# (frontend)
- ViewModel 使用 `ObservableObject` / `INotifyPropertyChanged`
- Native 调用通过 P/Invoke：`[DllImport("cide_native")]`
- UI 更新必须在主线程：`MainThread.InvokeOnMainThreadAsync()`
- 取消令牌必须正确 Dispose：`cts?.Cancel(); cts?.Dispose(); cts = null;`

## 已知限制

### 当前不支持
- **指针算术** — 有限支持（`p++`、`p+i` 等不支持）
- **逗号分隔的多变量声明** — 已支持（`int a = 1, b = 2;`）

### 已支持的关键特性
- **多维数组**（`int arr[3][3]`）— 声明、嵌套初始化列表 `{ {1,2}, {3,4} }`、索引访问 `arr[i][j]`、函数参数传递 `void f(int[][3])`
- **`#define` 宏** — 简单常量替换（如 `#define N 100`）
- **printf 可变参数** — 支持任意数量参数（如 `printf("%d %d %d", a, b, c)`）

### 已支持的关键特性
- **多维数组**（`int arr[3][3]`）— 声明、嵌套初始化列表 `{ {1,2}, {3,4} }`、索引访问 `arr[i][j]`、函数参数传递 `void f(int[][3])`

### 已修复的关键 Bug
- **Parser 死循环（2026-04-27）**：`struct*` 返回类型误识别为 struct 声明 → `ParseStructDecl` 零进度保护
- **Parser 死循环（2026-05-09）**：`ParseBlock()` 遇到无法解析的 token 时不前进 → 添加 `pos_ == checkpoint` 保护
- **移动端内存泄漏**：JS interop 监听器未清理、CTS 未 Dispose、ConsoleOutput 无上限

## 构建命令

```powershell
# 构建 native DLL (Debug)
cd native/build
cmake -S .. -B . -G Ninja -DCMAKE_BUILD_TYPE=Debug
cmake --build . --parallel

# 构建并运行桌面端
dotnet run --project Cide.Client.Desktop/Cide.Client.Desktop.csproj --configuration Debug

# 构建移动端 (需要 Android SDK)
.uild.ps1 -Target Android -Configuration Release
```

## 调试技巧

### Native 层调试
1. 项目属性 → 调试 → **启用本机代码调试**
2. 在 `native/src/capi/cide_capi.cpp` 的 `cide_compile_all` / `cide_run` 打断点
3. PDB 警告（`apphost.pdb` 缺失）可以安全忽略

### 内存泄漏定位
- 托管 vs 本机：VS 内存分析器看"托管内存"，如果增长很小但任务管理器内存很大 → 泄漏在 native heap
- Parser 死循环特征：内存缓慢持续增长（~100MB/秒），AST 节点或错误消息不断累积
