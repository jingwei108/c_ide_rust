# Cide

跨平台 C 语言教学 IDE（Flutter 前端 + Rust 后端 + 自研 CideVM）。

## 功能特性

- **手写 C 子集编译器**：Lexer → Parser → TypeChecker → BytecodeGen → CideVM
- **自研 106 条指令栈式虚拟机**：1MB 线性内存，支持单步调试、断点、Trap 系统
- **统一模式 / 时间旅行**：自动逐语句执行、任意历史步回退、异常自动回退
- **中文诊断系统**：56+ 错误码 + 结构化自动修复建议 + 11 张知识卡片
- **算法可视化**：数组排序动画、链表可视化、二叉树可视化、内存映射 Canvas
- **跨平台**：Android + Desktop Windows

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | Flutter + `re_editor` + `flutter_riverpod` |
| 后端 | Rust 1.95.0 + `flutter_rust_bridge` v2 |
| VM | 自定义字节码解释器 |

## 构建

```bash
# 桌面端 Debug
python ../scripts/build_flutter.py

# Android 完整构建
python ../scripts/build_flutter.py -t Android
```

详见项目根目录 `AGENTS.md`。
