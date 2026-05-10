# Cide 项目 Agent 指南

## 项目概览

Cide 是一个跨平台 C 语言 IDE，包含：

- **前端**：.NET MAUI (Android) + Avalonia (Desktop)
- **后端**：共享 Rust native 编译器/VM (`cide_native`)
- **编译管线**：Lexer → Parser → TypeChecker → BytecodeGen → CideVM

## 技术栈

| 层级 | 技术 |
|------|------|
| Android | .NET 10 MAUI BlazorWebView + CodeMirror6 |
| Desktop | Avalonia 11.3.0 + .NET 10 |
| Native | **Rust 1.95.0**, Cargo, cdylib/staticlib/rlib |
| VM | 自定义字节码解释器，256KB 线性内存 |

## 关键目录

```
native/src/compiler/    Lexer, Parser, TypeChecker, BytecodeGen, AST (Rust)
native/src/vm/          CideVM 字节码解释器 (Rust)
native/src/capi/        C API (P/Invoke / JNI 接口) (Rust)
native/src/diagnostics/ 结构化诊断、自动修复建议 (Rust)
Cide.Client/            Avalonia 桌面端共享代码
Cide.Client.Desktop/    Avalonia 桌面端入口
Cide.Client.Maui/       MAUI 移动端
Cide.Client.Shared/     共享 ViewModel / 服务
docs/                   设计文档、事故报告
```

## Rust 迁移进度（已完成 ✅）

| 阶段 | 模块 | 状态 |
|------|------|------|
| Phase 0 | Rust 骨架 + C API 桩 + Session 类型 | ✅ 完成 |
| Phase 1 | VM 迁移 (CideVM + host funcs) | ✅ 完成 |
| Phase 2a | Lexer | ✅ 完成 |
| Phase 2b | AST | ✅ 完成 |
| Phase 2c | Parser | ✅ 完成 |
| Phase 2d | TypeChecker | ✅ 完成 |
| Phase 2e | BytecodeGen | ✅ 完成 |
| Phase 2f | C API `cide_compile_all` 接线 | ✅ 完成 |
| Phase 3 | C# 前端端到端测试（Desktop / Android 编译通过） | ✅ 完成 |
| Phase 4 | Android 目标构建（cargo-ndk） | ✅ 完成 |
| Phase 5 | 清理遗留 C++ / CMake 文件 | ✅ 完成 |
| Phase 6 | 全面审查：编译警告清理 + 安全加固 + 测试覆盖拓展 | ✅ 完成 |
| Phase 7 | Desktop 内存泄漏修复 + Maui scanf 输入 + sizeof/scanf 子集拓展 | ✅ 完成 |

## 编码约定

### Rust (native)
- AST 使用 enum 替代 C++ 多态类层次：`Expr` / `Stmt` 枚举 + `Box<Expr>` / `Vec<Box<Expr>>`
- `SourceLoc` 已添加 `Copy` derive（两个 `i32`，值传递无开销）
- Parser 零进度保护：`if pos_ == checkpoint { self.advance(); }`
- 错误处理：不 panic，收集到 `Vec<Error>` 后统一返回
- Borrow checker 冲突解决模式：先 clone 数据再调用需要 `&mut self` 的方法

### C# (frontend)
- ViewModel 使用 `ObservableObject` / `INotifyPropertyChanged`
- Native 调用通过 P/Invoke：`[DllImport("cide_native")]`
- UI 更新必须在主线程：`MainThread.InvokeOnMainThreadAsync()`
- 取消令牌必须正确 Dispose：`cts?.Cancel(); cts?.Dispose(); cts = null;`

## 已知限制

### 当前不支持
（暂无）

### 已支持的关键特性
- **逗号分隔的多变量声明** — `int a = 1, b = 2;`
- **多维数组**（`int arr[3][3]`）— 声明、嵌套初始化列表 `{ {1,2}, {3,4} }`、索引访问 `arr[i][j]`、函数参数传递 `void f(int[][3])`
- **`#define` 宏** — 简单常量替换（如 `#define N 100`）
- **printf 可变参数** — 支持任意数量参数（如 `printf("%d %d %d", a, b, c)`）
- **局部 `char` 数组字符串初始化** — `char s[6] = "hello"; printf("%s", s);`
- **`enum` 局部/全局变量声明** — `enum Color c = GREEN;`（需先声明 enum 类型）
- **`typedef`** — `typedef int Integer; Integer a = 42;`
- **`sizeof` 运算符** — `sizeof(int)`、`sizeof(char)`、`sizeof(struct S)`、`sizeof(arr)`、`sizeof(ptr)`
- **`scanf` 多参数** — `scanf("%d %d %d", &a, &b, &c)`
- **指针算术** — `p++` / `p--` / `p + i` / `p - i` / `p - q`，自动按 pointee 类型大小缩放（`int*` 步长 4，`char*` 步长 1，`struct*` 步长为结构体大小）

### 已修复的关键 Bug
- **Parser 死循环（2026-04-27）**：`struct*` 返回类型误识别为 struct 声明 → `ParseStructDecl` 零进度保护
- **Parser 死循环（2026-05-09）**：`ParseBlock()` 遇到无法解析的 token 时不前进 → 添加 `pos_ == checkpoint` 保护
- **Parser 死循环（2026-05-10）**：`parse_case_stmt` 的 while 循环缺少零进度保护；`advance()` 空 token 列表 usize 下溢 panic；`synchronize()` 从未被调用 → 全面修复
- **VM 安全加固（2026-05-10）**：`addr+4` u32 溢出、`step_count` i32 溢出、`host_malloc` u32 溢出、Jump 目标越界、值栈无上限 → 全部修复
- **TypeChecker 警告代码勘误（2026-05-10）**：`W3050`/`W3051` 被滥用于不相关场景 → 新增 `W3052`~`W3055`
- **BytecodeGen char 数组初始化（2026-05-10）**：`char s[] = "hello"` 使用 `StoreLocal`（i32）导致字符间隔 3 字节零 → 改用 `StoreMemByte` 连续存储
- **移动端内存泄漏**：JS interop 监听器未清理、CTS 未 Dispose、ConsoleOutput 无上限
- **clippy 警告清零（2026-05-10）**：`Type::to_string` 改为 `Display`、`SourceLoc` clone 清理、`if_same_then_else`、`module_inception` 等 → `cargo clippy` 0 警告
- **C# 前端单元测试（2026-05-10）**：新建 `Cide.Client.Tests` xUnit 项目，覆盖 Session 创建/编译成功/编译失败路径 → 4 测试通过
- **Maui VM 释放（2026-05-10）**：`MainViewModel.Dispose()` 添加 `_disposed` 幂等保护；`Home.razor` 页面销毁时调用 `VM.Dispose()`
- **unsigned 类型提示（2026-05-10）**：Parser 保留 `is_unsigned` 标记；TypeChecker 遇到 `unsigned int x;` 时报告 `W3056` 提示"被映射为 int，暂不支持无符号语义"
- **C 子集 P0 拓展（2026-05-10）**：字符字面量 `'a'`、块注释 `/* */`、十六进制 `0xFF`、类型修饰符 `long/short/signed/const`、更多转义序列 `\r\a\b\f\v\xHH` → Lexer + Parser 全部支持，新增 5 个 E2E 测试
- **C 子集 P1 拓展（2026-05-10）**：复合赋值扩展到数组索引/指针解引用/结构体成员（`a[i]+=1`、`*p+=1`、`s.mem+=1`）、取地址扩展到复杂左值（`&a[i]`、`&s.mem`）、全局结构体变量成员访问、自增/自减扩展到复杂左值（`a[i]++`、`*p++`、`s.mem++`）→ BytecodeGen 全部支持，新增 7 个 E2E 测试
- **C 子集 P2 拓展（2026-05-10）**：位运算符 `& | ^ ~ << >>` 全管线支持（Lexer→Parser→TypeChecker→BytecodeGen→VM），新增 2 个 E2E 测试；三目运算符 `? :` 全管线支持，新增 1 个 E2E 测试

## 构建命令

```powershell
# 构建 native DLL (Release Desktop)
cd native
cargo build --release
# 输出: native/target/release/cide_native.dll

# 构建 Android .so (arm64-v8a + armeabi-v7a)
cd native
cargo ndk -t aarch64-linux-android -o target/android build --release
cargo ndk -t armv7-linux-androideabi -o target/android build --release

# 构建并运行桌面端
dotnet run --project Cide.Client.Desktop/Cide.Client.Desktop.csproj --configuration Debug

# 构建移动端 (需要 Android SDK)
dotnet build Cide.Client.Maui/Cide.Client.Maui.csproj --framework net10.0-android
```

## 调试技巧

### Native 层调试 (Rust)
1. 项目属性 → 调试 → **启用本机代码调试**
2. 在 `native/src/capi/mod.rs` 的 `cide_compile_all` / `cide_run` 打断点
3. PDB 警告（`apphost.pdb` 缺失）可以安全忽略

### 内存泄漏定位
- 托管 vs 本机：VS 内存分析器看"托管内存"，如果增长很小但任务管理器内存很大 → 泄漏在 native heap
- Parser 死循环特征：内存缓慢持续增长（~100MB/秒），AST 节点或错误消息不断累积
