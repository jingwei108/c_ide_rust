# Cide 项目文档

> 跨平台 C 语言教学 IDE —— 设计文档、构建指南与规范

## 文档目录

### 📁 [current/](current/) — 当前有效文档

| 文档 | 说明 |
|------|------|
| [`current/DESIGN.md`](current/DESIGN.md) | 总体架构设计（Rust 后端 + Flutter 前端） |
| [`current/ROADMAP.md`](current/ROADMAP.md) | 项目路线图与开发阶段 |
| [`current/BUILD.md`](current/BUILD.md) | 构建指南（Rust + Flutter） |
| [`current/BUILD_SCRIPTS.md`](current/BUILD_SCRIPTS.md) | 构建脚本详细说明 |
| [`current/C_SUBSET_SPEC.md`](current/C_SUBSET_SPEC.md) | C 语言教学子集规范 |
| [`current/MEMORY_SAFETY.md`](current/MEMORY_SAFETY.md) | 内存安全规范 |
| [`current/FLUTTER_MIGRATION_PLAN.md`](current/FLUTTER_MIGRATION_PLAN.md) | Flutter 前端迁移计划 |
| [`current/FLUTTER_MIGRATION_STATUS.md`](current/FLUTTER_MIGRATION_STATUS.md) | Flutter 迁移当前状态 |
| [`current/FLUTTER_BUILD_MANUAL.md`](current/FLUTTER_BUILD_MANUAL.md) | Flutter 手动构建指南 |
| [`current/FLUTTER_HOT_RELOAD_GUIDE.md`](current/FLUTTER_HOT_RELOAD_GUIDE.md) | Flutter 热重载指南 |
| [`current/ALGORITHM_DATASTRUCTURE_DESIGN.md`](current/ALGORITHM_DATASTRUCTURE_DESIGN.md) | 算法与数据结构支持设计 |
| [`current/STUDENT_ERROR_TEST_CASES.md`](current/STUDENT_ERROR_TEST_CASES.md) | 学生常见错误测试用例 |
| [`current/UI_ISSUES_AND_PLAN.md`](current/UI_ISSUES_AND_PLAN.md) | UI 问题与改进计划 |
| [`current/UI_FLOATING_BUTTON_DESIGN.md`](current/UI_FLOATING_BUTTON_DESIGN.md) | 悬浮按钮交互设计 |
| [`current/PANEL_DRAG_GESTURE_DESIGN.md`](current/PANEL_DRAG_GESTURE_DESIGN.md) | 面板拖拽手势设计 |
| [`current/CUSTOM_KEYBOARD.md`](current/CUSTOM_KEYBOARD.md) | 自定义键盘设计 |
| [`current/EDITOR_LONG_PRESS_MENU.md`](current/EDITOR_LONG_PRESS_MENU.md) | 编辑器长按菜单设计 |
| [`current/MOBILE_FUNCTIONAL_TEST_PLAN.md`](current/MOBILE_FUNCTIONAL_TEST_PLAN.md) | 移动端功能测试计划 |
| [`current/CODE_REVIEW_5_16.md`](current/CODE_REVIEW_5_16.md) | 全面代码审查报告（2026-05-16） |
| [`current/SHADOW_VERIFICATION_FRAMEWORK.md`](current/SHADOW_VERIFICATION_FRAMEWORK.md) | Clang 影子验证框架 |
| [`current/DOUBLE_TYPE_SUPPORT_PLAN.md`](current/DOUBLE_TYPE_SUPPORT_PLAN.md) | `double` 类型支持计划 |

### 📁 [archive/](archive/) — 历史归档文档

存放已完成、废弃或不再维护的历史文档，包括：
- 各阶段迁移计划（MAUI → Flutter、C++ → Rust、WASM → 自定义 VM）
- 历史代码审查报告与事故复盘
- 已废弃技术方案（WASM、CodeMirror6、OCR 导入等）
- 一次性修复记录与进度报告

> ⚠️ **archive/ 中的文档仅供追溯参考，内容可能已严重过时。**
