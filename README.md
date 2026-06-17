# Cide

> [English Version](README_EN.md)

一个跨平台 C/C++ 受限子集教学 IDE：Rust 后端编译器 + CideVM 自定义字节码虚拟机 + 基于 Trace 的模板 JIT 加速，Flutter 前端，算法可视化与时间旅行调试教学。

## 这是什么

Cide 最初是为教学场景设计的一款轻量级 IDE，目标是在课堂环境中让学生能够：

- 编写 C/C++ 教学子集代码
- 单步执行、观察内存与变量变化
- 通过可视化理解算法与数据结构
- 在错误发生时获得结构化诊断与学习建议

项目后端是一个完整的编译器管线（Lexer → Parser → TypeChecker → BytecodeGen → CideVM），前端基于 Flutter，支持 Android 与 Windows 桌面。

## 诚实声明：这是一个 AI 实验田

**Cide 不是由一个完整掌握每一行代码的团队从零手写而成的项目。**

它是人与 AI 协作的产物：

- 项目设计者参与了整体架构、功能方向、关键决策与部分细节调整
- 大量代码、测试、文档由 AI（包括本 README）生成、重构与维护
- 设计者无法保证能够回答社区提出的每一个问题
- `docs/archive` 中保留一部分协作交互文本

如果你在使用过程中发现：

- 某段代码的设计理由说不清
- 某个边界行为与标准不一致
- 某处文档滞后于实现
- 某些改动看起来是"为了通过测试而绕过问题"

这些都有可能是 AI 实验田的典型痕迹。我们宁可诚实记录，也不想伪装成一切尽在掌控。

## 如果你感到被浪费了时间

大可抨击我们。

批评是项目继续改进的真实动力。如果你愿意，可以通过 Issue 或 PR 指出问题；如果只想发泄，我们也接受——毕竟一个无法对全部代码负责的项目，本就配不上所有人的信任。

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | Flutter + 自研 `CideEditor` + CustomPainter 可视化 |
| 后端 | Rust 1.95.0 |
| VM | 自定义字节码解释器（106 条指令） |
| JIT | 基于 Trace 的模板超级指令加速（热点循环识别，安全 Rust 内预编译函数序列） |
| 桥接 | flutter_rust_bridge v2.12.0 |
| 构建 | Python 脚本 + Cargo + Flutter |

> **注意**：模板 JIT 并非传统机器码 JIT。由于 crate 启用 `#![forbid(unsafe_code)]`，无法动态生成机器码，因此将热点循环的字节码 trace 编译为预编译 Rust 函数指针序列（超级指令），跳过解释器 dispatch 开销，不匹配时回退到标准解释执行。

## 当前状态

- **C 子集**：覆盖教学场景常用语法，Shadow Verification 511 用例 / 505 匹配（2026-06-16 实测）
- **C++14 教学子集扩展**：M7 Beta Readiness 已就绪，83/83 C++ Shadow Verification 全绿，61 个 C++ E2E 用例全绿
- **编辑器**：已移除第三方 `re_editor`，当前为自研 `CideEditor`（`CideFlutter/lib/editor/`）（原身为re_editor，基于其进行魔改）
- **统一模式 / 时间旅行**：已实现
- **算法可视化**：数组排序、链表、二叉树等零侵入可视化已实现

## 项目结构

```
CideFlutter/          Flutter 前端（Android + Windows Desktop）
native/               Rust 后端
├── src/
│   ├── compiler/     编译器（Lexer / Parser / AST / TypeChecker / BytecodeGen）
│   ├── vm/           CideVM 字节码解释器 + JIT 模板加速
│   ├── unified/      统一模式 / 时间旅行引擎
│   ├── diagnostics/  结构化诊断、自动修复、知识图谱
│   ├── api/          flutter_rust_bridge API
│   ├── capi/         精简 C API（Shadow Verification / CLI 服务）
│   └── bin/          cide_cli 命令行调试工具
├── runtime_libc/     标准库存根与内置 C++ 容器模板
└── tests/            测试防线与用例
docs/                 设计文档、构建指南与规范
scripts/              Python 构建与测试脚本
```

## 快速开始

完整快速入门指南见 [`docs/current/QUICKSTART.md`](docs/current/QUICKSTART.md)。

常用命令：

```bash
# 命令行快速体验（无需 Flutter）
cd native
cargo build --release --bin cide_cli
cargo run --release --bin cide_cli -- run tests/cases/baseline/hello_world.c

# Windows 桌面端构建并运行
python scripts/build_flutter.py --run

# 运行全部 Rust 测试
cd native && cargo test

# C Shadow Verification
python native/tests/shadow_verification/shadow_verify.py

# C++ Shadow Verification
python scripts/shadow_verify_cpp.py

# Flutter 前端测试（单元 / Widget / 集成）
cd CideFlutter && flutter test
cd CideFlutter && flutter test -d windows integration_test/
```

Flutter 测试框架详见 [`docs/current/FLUTTER_TESTING.md`](docs/current/FLUTTER_TESTING.md)。

环境要求、完整构建命令与开发约定见 [`AGENTS.md`](AGENTS.md)。

## 测试防线

项目采用五条分层协作的测试防线，核心哲学：*测试不是为了标榜通过率，而是为了诚实地发现自己可能存在的问题*。

1. **Shadow Verification**：与 Clang/Clang++ 对比 stdout 输出
2. **K&R + LeetCode 真实程序回归**：K&R 69 题、LeetCode 48 题
3. **三层契约验证**：Host / Bytecode Libc / Differential Stress
4. **Fuzz 压力测试**
5. **CI 集成与一致性监控**

当前状态：

- C Shadow Verification：**511 用例，505 匹配**（差异均为诚实记录的已知限制；2026-06-16 实测）
- C++ Shadow Verification：**83/83 全绿，0 gap**
- C++ E2E 回归用例：**61/61 全绿**
- 全量 `cargo test`：**719 passed，0 failed**

## 许可证

[MIT](LICENSE)
