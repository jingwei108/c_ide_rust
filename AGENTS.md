# Cide 项目 Agent 指南

> [English Version](AGENTS_EN.md)

## 项目概览

Cide 是一个跨平台 C 语言 IDE，包含：

- **前端**：Flutter (Android + Desktop Windows) — 使用自研 `CideEditor` 编辑器 + `flutter_riverpod` 状态管理
- **后端**：共享 Rust native 编译器/VM (`cide_native`)
- **编译管线**：Lexer → Parser → TypeChecker → BytecodeGen → CideVM
- **桥接**：flutter_rust_bridge v2 (`native/src/api/cide.rs` → `CideFlutter/lib/src/rust`)
- **必须中文输出思考以及回答问题**
- **未经允许禁止git提交**
- **诚实记录**：本项目作为教学c/cpp子集，以clang为标准，任何本项目与标准不符合的，都要进行记录
## 技术栈

| 层级 | 技术 |
|------|------|
| Android | Flutter + 自研 `CideEditor` + CustomPainter 可视化 |
| Desktop | Flutter + 自研 `CideEditor` + CustomPainter 可视化 |
| Native | **Rust 1.95.0**, Cargo, cdylib/staticlib/rlib |
| VM | 自定义字节码解释器，1MB 线性内存 |
| Bridge | flutter_rust_bridge v2.12.0 (SSE codec) |

## 关键目录

```
native/src/compiler/    Lexer, Parser, TypeChecker, BytecodeGen, AST, CFG, DataFlow, IntentInference (Rust)
native/src/vm/          CideVM 字节码解释器 (Rust)
native/src/unified/     统一模式 / 时间旅行引擎 (Rust)
native/src/engine/      编译管线与工具 (Rust)
native/src/capi/        C API (MAUI 兼容层) (Rust)
native/src/api/         FRB API (flutter_rust_bridge) (Rust)
native/src/diagnostics/ 结构化诊断、自动修复建议、知识图谱、教学推理 (Rust)
CideFlutter/            Flutter 跨平台前端 (Android + Desktop Windows)
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
| Phase 3 | ~~C# 前端~~ → Flutter 前端端到端测试 | ✅ 完成 |
| Phase 4 | Android 目标构建（cargo-ndk） | ✅ 完成 |
| Phase 5 | 清理遗留 C++ / CMake 文件 | ✅ 完成 |
| Phase 6 | 全面审查：编译警告清理 + 安全加固 + 测试覆盖拓展 | ✅ 完成 |
| Phase 7 | Desktop 内存泄漏修复 + sizeof/scanf 子集拓展 | ✅ 完成 |
| Phase 8 | `float` 类型全管线支持（Lexer→Parser→TypeChecker→BytecodeGen→VM）+ 诊断系统拓展 | ✅ 完成 |
| Phase 9 | Flutter 前端从零搭建：IDE 界面 + 编辑器 + 调试面板 + 算法可视化 | ✅ 完成 |
| Phase 10 | 内存映射 Canvas + 算法可视化事件 FRB 集成 + 交互增强 | ✅ 完成 |
| Phase 11 | 代码审查修复 + 工程规范（`rustfmt.toml`/`CHANGELOG.md`）+ Flutter 前端全面模块化拆分 | ✅ 完成 |
| Phase 12 | `union` 类型全管线支持（Lexer→Parser→TypeChecker→BytecodeGen→VM）+ `sizeof(union U)` | ✅ 完成 |
| Phase 13 | 统一模式 / 时间旅行：VM 快照/恢复、检查点管理器、Seek 进度条、异常回退 | ✅ 完成 |
| Phase 14 | 堆内存可视化增强：malloc 行号追踪、外部碎片可视化、泄漏检测报告 | ✅ 完成 |
| Phase 15 | 指针追踪动画：`PointerSnapshot` 四状态（Valid/Freed/Null/Dangling）实时箭头绘制 | ✅ 完成 |
| Phase 16 | 算法步骤语义标注：27 种算法预定义步骤模板，运行时推断教学描述 | ✅ 完成 |
| Phase 17 | 代码模板参数化 + 交互式教程：参数占位符、`TemplateTutorialPanel` 逐行引导 | ✅ 完成 |
| Phase 18 | 6-04 地毯式审阅：P0 soundness 修复、VM 优化、DRY 重构、clippy 0 警告 | ✅ 完成 |
| Phase 19 | Use-After-Free / Double-Free 运行时检测：`freed_logs` 指令层检查、知识卡片 E3060/E3061 | ✅ 完成 |
| Phase 20 | 认知推理 P0：`TraceAnalyzer` 轨迹切片 + 5 类 Trap 根因推断、`RootCauseHint` | ✅ 完成 |
| Phase 21 | 认知推理 P1：`MisconceptionPattern` 6 种模式检测 + `LearningPath` 推荐引擎 | ✅ 完成 |
| Phase 22 | 认知推理 P2：`KnowledgeGraph` 24 概念节点 + 30+ 关系边、`ConceptGraphView` | ✅ 完成 |
| Phase 23 | 认知推理 P3：`ControlFlowGraph` + `DataFlow` + `IntentInference` 代码意图推断 | ✅ 完成 |
| Phase 24 | 语义智能补全 v2：`CompletionEngine` 五种上下文感知补全 | ✅ 完成 |
| Phase 25 | 模板 JIT（Trace-based Loop Accelerator）：热点循环 trace 录制 + 预优化函数指针序列 | ✅ 完成 |
| Phase 26 | Flutter Bridge 通信优化：Stream 模式、差分编码 `StepPayloadDelta`、符号表 dedup | ✅ 完成 |
| Phase 27 | 数据结构语法拓展 P0+P1：数组退化、`unsigned` 全链路、`const`、`extern`、VLA 全管线 | ✅ 完成 |
| Phase 28 | CLI 调试工具 `cide_cli`：`compile`/`run`/`step`/`unified`，支持 stdin 管道快速测试 | ✅ 完成 |
| Phase 29 | Bytecode Libc 产品化：构建期预编译 + 固定索引段 + ctype/abs 走 Bytecode 路径 | ✅ 完成 |
| Phase 30 | P0 语法拓展：通用逗号运算符、Designated Initializer、offsetof + 回归修复 | ✅ 完成 |
| Phase 31 | C++ 扩展 P0：Lexer/Parser/AST 关键字与节点扩展 | ✅ 完成 |
| Phase 32 | C++ 扩展 P1：TypeChecker 类/继承/模板单态化 | ✅ 完成 |
| Phase 33 | C++ 扩展 P2：BytecodeGen 虚函数/this 指针/构造析构 | ✅ 完成 |
| Phase 34 | C++ 扩展 Stage 0.5：容器收口（list<int>/vector<char>/sort_int）、C++ 三 tier 纳入 CI、CPP_FAILURES.md | ✅ 完成 |
| Phase 35 | C++ 扩展 Stage 2：栈对象 RAII（默认构造函数自动调用 / scope exit / return / break / continue 自动析构） | ✅ 完成 |
| Phase 36 | C++ 扩展 Stage 3：`new[]/delete[]` 元素构造析构（base[-4] 存 count、逆序 dtor、temp slot 扩展至 4 个） | ✅ 完成 |
| Phase 37 | C++ 扩展 Stage 4：引用声明与基本语义（`int& r = x` 全链路；`T&` 参数/返回值；引用自动解引用；隐式取地址；返回引用左值识别） | ✅ 完成 |
| Phase 38 | C++ 扩展 Stage 5：隐式移动构造函数自动生成（类含指针/资源字段时自动生成 `__ctor__{Class}__move`；`std::move` 初始化调用移动构造；源指针字段置空防双重释放） | ✅ 完成 |
| Phase 39 | C++ 扩展 Stage 6：`unique_ptr<T>` 简化版 dogfooding + 构造函数初始化语法 `Type name(args);` + 构造函数重载/隐式默认构造 | ✅ 完成 |
| Phase 40 | C++ 扩展 M6：测试防线收尾 — 新增 59 个 C++ E2E 回归用例（核心语言 / 容器算法 / 教学 OJ），`test_cide_e2e_cpp` 纳入 CI，Golden 由 Clang++ 生成 | ✅ 完成 |
| Phase 41 | C++ 内置容器布局解耦：`.cpp` 接口声明作为唯一真相来源 + JSON 加载器，零 Rust 硬编码 | ✅ 完成 |
| Phase 42 | P0 语法/标准库拓展 + 代码审查报告推进 + 性能优化（[Unreleased]） | 🚧 进行中 |

## 测试防线

Cide 采用**五条分层协作的测试防线**，核心哲学：*测试不是为了标榜通过率，而是为了诚实地发现自己可能存在的问题*。任何失败必须如实记录，禁止通过修改测试预期值来粉饰数据。

```
┌────────────────────────────────────────────────────────────────┐
│  防线 5：CI 集成与一致性监控（Phase F）                          │
│  └─ PR 时自动跑全部防线，*_FAILURES.md 与测试结果交叉验证        │
├────────────────────────────────────────────────────────────────┤
│  防线 4：Fuzz 压力测试（Phase E）                                │
│  └─ 随机内存状态 + 随机标准库调用序列，验证安全检测不泄漏         │
├────────────────────────────────────────────────────────────────┤
│  防线 3：三层契约验证（Phase A~C）                               │
│  ├─ 3a Host Contract：Rust 单元测试直接验证 Host Func 边界行为   │
│  ├─ 3b Bytecode Self-Consistency：C 源码 → Clang vs Cide 自举   │
│  └─ 3c Differential Stress：同一功能多实现交叉对比               │
├────────────────────────────────────────────────────────────────┤
│  防线 2：K&R 真实程序回归（已有）+ LeetCode（已启动，阶段 4）      │
│  └─ K&R 验证"真实世界代码能不能跑"；LeetCode 简单题逐步填充中     │
├────────────────────────────────────────────────────────────────┤
│  防线 1：Shadow Verification 影子验证（已有）                     │
│  └─ 验证"与 Clang 行为是否一致"                                  │
└────────────────────────────────────────────────────────────────┘
```

### 防线 1：Shadow Verification

将同一 C 源码同时交给 **Clang** 与 **Cide** 编译执行，对比 stdout 输出是否完全一致。Golden 只能来自 Clang，不能来自 Cide 自己。

- **覆盖**：298 个 Baseline 用例 + 82 个模板生成用例 + 69 个 K&R 用例 + 48 个 LeetCode 简单题（C Shadow Verification 合计 511 个用例，505 匹配）；83 个 C++ 用例（C++ Shadow Verification，83/83 全绿；2026-06-16 实测）
- **驱动**：`python native/tests/shadow_verification/shadow_verify.py`、`python scripts/shadow_verify_cpp.py`
- **报告**：`native/tests/shadow_verification/reports/`

### 防线 2：K&R 真实程序回归（已有）+ LeetCode（计划中）

收集真实教学/竞赛代码作为端到端回归用例，验证"真实世界代码能不能跑"。

- **Baseline**：`native/tests/cases/baseline/`（298 个，全绿）
- **K&R**：《C程序设计语言》课后习题（69 个，69 绿，0 已知失败）
- **Template Generated**：算法模板批量生成（82 个，78 绿，4 已知失败）
- **LeetCode**：已全面实施阶段 4 + 阶段 5，当前 48 个简单题全部通过，详见 `native/tests/LEETCODE_FAILURES.md`
- **报告**：`native/tests/TEST_REPORT.md`、`KR_FAILURES.md`、`E2E_FAILURES.md`、`LEETCODE_FAILURES.md`

### 防线 3：三层契约验证

同一功能可能同时存在 VM Builtin、Rust Host、Bytecode Libc 三种实现，需要独立验证它们之间的一致性。

| 子层 | 目标 | 关键文件 |
|------|------|----------|
| **3a Host Contract** | 验证 Layer B Host Func 的边界条件、安全注入（UAF/Double-Free/Buffer Overflow）| `native/tests/host_contract_tests.rs` |
| **3b Bytecode Self-Consistency** | Cide 编译器 + VM 能否正确编译并运行"自己的标准库" | `native/tests/bytecode_libc_consistency.rs` + `bytecode_libc_consistency/src/*.c` |
| **3c Differential Stress** | 同一功能的 Host 版与 Bytecode 版交叉验证，结果必须永远一致 | `native/tests/differential_stress.rs` |

- **失败记录**：`HOST_CONTRACT_FAILURES.md`、`BYTECODE_LIBC_FAILURES.md`、`DIFFERENTIAL_FAILURES.md`

### 防线 4：Fuzz 压力测试

使用**确定性 RNG** 生成随机内存状态与随机标准库调用序列，验证安全检测不泄漏、不崩溃。

| 场景 | 覆盖内容 |
|------|----------|
| **Fuzz A** | malloc/free/realloc 随机序列 + UAF/Double-Free 检测验证 |
| **Fuzz B** | strcpy/strcat/strncpy/memcpy/memmove + Buffer Overflow (E3070) |
| **Fuzz C** | printf/scanf/getchar/putchar 随机格式与输入 |
| **Fuzz D** | 混合恶意序列（内存/字符串/IO/rand 交叉） |
| **Fuzz E** | 随机分配 + 部分释放，验证泄漏报告准确性 |

- **驱动**：`cargo test --test fuzz_stress_test`
- **记录**：`native/tests/FUZZ_FAILURES.md`

### 防线 5：CI 集成与一致性监控

`.github/workflows/ci.yml` 在每次 Push/PR 时自动运行以上全部防线，并执行 `scripts/ci_three_tier_check.py` 进行一致性检查：

- 若 `*_FAILURES.md` 中标记为 `KNOWN_FAILURE` 的测试现在通过了 → **报错提示更新文档**
- 若测试失败了但文档中没有对应记录 → **报错提示添加记录**
- 生成 `reports/three_tier_report.md` 作为 CI artifact 上传

---

## 编码约定

### Rust (native)
- AST 使用 enum 替代 C++ 多态类层次：`Expr` / `Stmt` 枚举 + `Box<Expr>` / `Vec<Box<Expr>>`
- `SourceLoc` 已添加 `Copy` derive（两个 `i32`，值传递无开销）
- Parser 零进度保护：`if pos_ == checkpoint { self.advance(); }`
- 错误处理：不 panic，收集到 `Vec<Error>` 后统一返回
- Borrow checker 冲突解决模式：先 clone 数据再调用需要 `&mut self` 的方法

### Dart / Flutter (frontend)
- 状态管理：`flutter_riverpod` (`StateNotifier` + `StateNotifierProvider`)
- 编辑器：自研 `CideEditor`（`EditableText` + `CustomPaint` 实现），非 CodeMirror / 非 `re_editor`
- Rust 调用通过 `flutter_rust_bridge`：`rust.compile()` / `rust.stepNext()` 等
- UI 线程：`Future.delayed` / `async-await`，无需显式主线程切换
- 自定义组件：算法验证、内存映射、链表可视化、教程引导等均为 CustomPainter / Widget 实现

## C 教学子集支持概览

本项目支持的 C 语言教学子集覆盖 **Phase 1 ~ Phase 5+** 能力（含逗号运算符、Designated Initializer、`offsetof`、VFS 文件 I/O 等），详细规范见 [`docs/current/C_SUBSET_SPEC.md`](docs/current/C_SUBSET_SPEC.md)。核心支持包括：

**数据类型**：`int`、`char`、`float`、`double`、`unsigned`、`_Bool`/`bool`、`int*`、`char*`、`float*`、`double*`、`int[]`、`char[]`、`double[]`、`struct`（含按值返回）、`union`、`enum`、`typedef`

**数组**：固定大小数组（一维/多维）、**VLA 变长数组**（`int a[n]` / `int a[n][3]`，局部作用域，运行时栈分配）、数组/字符串初始化列表、数组参数退化语义

**指针**：取地址 `&`、解引用 `*`、指针算术（步长自动缩放）、**多级指针**（`int**`、`struct Node**`）、显式类型转换 `(int*)p`、函数指针（含间接调用、结构体成员、typedef）

**语句**：变量声明（含多变量、块作用域）、`if/else`、`while`、`do...while`、`for`（C99 风格变量声明）、`switch/case/default`、`break`、`continue`、`return`

**表达式**：算术、比较、逻辑（短路求值）、位运算 `& | ^ ~ << >>`、赋值（含复合赋值）、三目运算符 `?:`、数组索引、函数调用、`&`、`*`、结构体访问 `.` / `->`、`++` / `--`、`sizeof`

**函数**：定义/调用/递归/前向声明、**函数按值返回结构体**（Hidden Return Pointer ABI）

**内存**：`malloc`/`free`/`realloc`

**I/O**：`printf`/`scanf`/`sprintf`/`snprintf`/`sscanf`/`fprintf`/`fgets`/`fputs`/`puts`/`getchar`/`putchar`/`ungetc`；VFS 沙盒文件 I/O：`fopen`/`fread`/`fwrite`/`fclose`/`feof`/`fgetc`/`fputc`/`fseek`/`ftell`/`rewind`

**字符串**：`strlen`、`strcpy`、`strncpy`、`strcmp`、`strncmp`、`strcat`、`strncat`、`memcpy`、`memmove`、`memcmp`、`strchr`、`strrchr`、`strstr`、`memchr`、`strdup`、`atoi`

**数学**：`sin`/`cos`/`tan`/`sqrt`/`pow`/`atan`/`log`/`log10`/`exp`/`fabs`/`abs`/`ceil`/`floor`/`round`/`fmod`（通过 `libm`，`double` 精度）

**类型系统**：`typedef`、`sizeof`、`const`、`static`（局部+全局+函数）、`extern`、`volatile`、`restrict`、`inline`、`register`、`auto`

**头文件**：`#include <stdio.h>` / `<stdlib.h>` / `<ctype.h>` / `<math.h>` / `<string.h>` 加载存根声明

**其他**：`rand`/`srand`、`memset`、`exit`、`qsort`、`calloc`、`bsearch`、`atof`/`atol`、`#define` 宏（对象宏/参数化宏/嵌套调用）

**字符分类**：`isdigit`/`isalpha`/`islower`/`isupper`/`isalnum`/`isspace`/`isprint`/`iscntrl`/`isxdigit`/`tolower`/`toupper`（`ctype.h`，部分走 Bytecode Libc 路径）

**C++ 类与模板（Phase 31+）**：`class`、成员访问控制、`this` 指针、虚函数、模板类单态化、栈对象 RAII（自动构造/析构）、构造函数初始化语法 `Type name(args);`、隐式默认构造/移动构造、`std::move`、简化版 `unique_ptr<T>` dogfooding（构造/`get`/`release`/`reset`/析构/所有权转移）

**明确不支持**：bitfield、`va_list` 变参、全局 VLA、完整预处理器（仅 `#define` 常量宏 + `#include` 标准库存根）

## 已知限制

### 当前不支持
- **参数化宏调用后带分号** — 形如 `SWAP(int,x,y);` 的参数化宏调用，若宏体本身已包含大括号 `{ ... }`，展开后形成 `{ ... };`（复合语句 + 空语句），当前 Parser 无法正确解析。workaround：宏调用后不加额外分号，或使用 `do { ... } while(0)` 模式
- **VLA 边界检查** — VLA 数组索引暂不支持编译期/运行时的 `TrapBounds` 边界检查（`bound_size` 为 0，检查被跳过）
- **`#include` 非标准库路径** — 仅支持 `<stdio.h>` / `<stdlib.h>` / `<ctype.h>` / `<math.h>` / `<string.h>` 的存根加载；自定义头文件或 `"header.h"` 形式暂不支持
- **`va_list` / `va_start` / `va_arg` / `va_end`** — 自定义变参函数暂不支持（`printf`/`scanf` 已内置支持）
- **全局 VLA** — 全局/静态作用域的变长数组暂不支持
- **VFS 文本模式换行转换（已修复）** — 2026-06-15 已完整实现 Windows 文本模式换行转换：`"r"`/`"w"` 模式下写入时将 `\n` 展开为 `\r\n`，读取时将 `\r\n` 压缩为 `\n`；`fseek`/`ftell` 区分逻辑/物理光标以匹配 Windows CRT 行为。`vfs_io_extensions.c` 与 `file_fread.c` 已恢复匹配。

### 已知 Cide 与 Clang 的行为差异（诚实记录）

在 LeetCode 防线填充过程中发现以下 Cide 与 Clang 行为不一致：

- **复合副作用数组索引** — 形如 `a[++i] = b[j--]` 的数组索引表达式（同时包含对两个不同对象的 `++`/`--` 副作用）在 Clang/GCC 下行为正确，但在 Cide VM 中可能错误触发"访问 NULL 指针"陷阱（见 `native/tests/LEETCODE_FAILURES.md` 中 `lc_232` 记录）。workaround：将自增/自减拆分为独立语句。

> 历史特性详情和 Bug 修复记录见 [`CHANGELOG.md`](CHANGELOG.md) 和 [`docs/current/C_SUBSET_SPEC.md`](docs/current/C_SUBSET_SPEC.md)。

## 构建命令

```bash
# 日常构建（桌面端 Debug）
python scripts/build_flutter.py

# 构建并运行桌面端 Release
python scripts/build_flutter.py -c Release --run

# Android 完整构建（.so + APK）
python scripts/build_flutter.py -t Android

# 构建 + 安装 + 启动 + 日志（移动端完整流水线）
python scripts/test_mobile.py --install --run --logcat

# Release 发布构建
python scripts/build_release.py

# 构建前运行测试和 lint
python scripts/build_flutter.py --test

# Flutter 离线构建（无网络环境）
python scripts/build_flutter.py --offline

# Flutter 清理构建产物
python scripts/build_flutter.py --clean

# --- 手动命令（脚本不可用时的备选） ---

# 构建 native DLL (Release Desktop)
cd native && cargo build --release
# 输出: native/target/release/cide_native.dll

# 构建 Android .so (arm64-v8a + armeabi-v7a)
cd native
cargo ndk -t aarch64-linux-android --platform 21 build --release
cargo ndk -t armv7-linux-androideabi --platform 21 build --release

# 构建并运行 Flutter 桌面端（手动命令）
cd CideFlutter
flutter pub get --offline
flutter build windows --debug
flutter run -d windows

# 构建 Flutter Android APK（手动命令）
cd CideFlutter
flutter build apk --release

# 安装并启动（手动命令）
adb install -r "build/app/outputs/flutter-apk/app-release.apk"
adb shell monkey -p com.cide.app -c android.intent.category.LAUNCHER 1
```

## 调试技巧

### Native 层调试 (Rust)
1. 项目属性 → 调试 → **启用本机代码调试**
2. 在 `native/src/capi/mod.rs` 的 `cide_compile_all` / `cide_run` 打断点
3. PDB 警告（`apphost.pdb` 缺失）可以安全忽略

### 内存泄漏定位
- 托管 vs 本机：VS 内存分析器看"托管内存"，如果增长很小但任务管理器内存很大 → 泄漏在 native heap
- Parser 死循环特征：内存缓慢持续增长（~100MB/秒），AST 节点或错误消息不断累积

## CLI 调试工具

项目提供独立的命令行调试工具 `cide_cli`，无需启动 Flutter 前端即可直接操作 Rust 后端编译器/VM。

### 构建

```bash
cd native && cargo build --release --bin cide_cli
```

### 命令

| 命令 | 说明 |
|------|------|
| `compile <file>` | 编译并显示诊断信息（错误码 + 修复建议） |
| `run <file>` | 编译并全速运行 |
| `step <file>` | 交互式单步调试（支持 `p` 打印变量、`o` 打印输出、`r` 运行到结束、`q` 退出） |
| `unified <file>` | 统一模式（时间旅行引擎）批量执行并输出摘要 |
| `export <file1> [file2 ...] -o <out.json>` | 预编译为字节码产物（多文件 + `--builtin-libc` 选项） |

### 选项与特殊文件名

- `-i <file>`：从文件读取标准输入（多行输入供 `scanf`/`fgets` 使用）
- `-`：从标准输入读取源代码，便于快速测试代码片段

### 快速测试示例

```bash
# 管道直接运行
echo '#include <stdio.h>
int main() { printf("hello\n"); return 0; }' | cide_cli run -

# here-document 编译
cide_cli compile - <<'EOF'
#include <stdio.h>
int main() {
    int a = 10, b = 20;
    printf("%d\n", a + b);
    return 0;
}
EOF

# 带输入文件运行
cide_cli run sum.c -i input.txt

# 统一模式执行
cide_cli unified hello.c

# 预编译字节码产物（含 Bytecode Libc）
cide_cli export main.c libc_helper.c -o bundle.json --builtin-libc
```

完整文档见 [`docs/current/CIDE_CLI.md`](docs/current/CIDE_CLI.md)。

