# Cide 项目文档

> 跨平台 C 语言教学 IDE —— 设计文档、构建指南与规范

## 文档目录

### 📁 [current/](current/) — 当前有效文档

#### 架构与设计

| 文档 | 说明 |
|------|------|
| [`current/DESIGN.md`](current/DESIGN.md) | 总体架构设计（Rust 后端 + Flutter 前端 + CideVM） |
| [`current/ROADMAP.md`](current/ROADMAP.md) | 项目路线图与开发阶段（Stage 0~8，CideVM 106 条指令；C++ 扩展 M7 Beta 已就绪） |
| [`current/C_SUBSET_SPEC.md`](current/C_SUBSET_SPEC.md) | C 语言教学子集规范（支持语法 / 排除清单） |
| [`current/CPP_SUBSET_SPEC.md`](current/CPP_SUBSET_SPEC.md) | C++14 教学子集规范（学生版） |
| [`current/MEMORY_SAFETY.md`](current/MEMORY_SAFETY.md) | 内存安全规范（Rust + Flutter） |
| [`current/RECURSIVE_TYPE_SYSTEM_REFACTOR.md`](current/RECURSIVE_TYPE_SYSTEM_REFACTOR.md) | 递归类型系统重构方案 |

#### 统一模式 / 时间旅行

| 文档 | 说明 |
|------|------|
| [`current/UNIFIED_MODE_DESIGN.md`](current/UNIFIED_MODE_DESIGN.md) | 统一模式架构设计（VM 快照 + 检查点 + Seek + 自动执行） |
| [`current/VM_EXPERIENCE_ADVANTAGE.md`](current/VM_EXPERIENCE_ADVANTAGE.md) | 自研 VM 体验优势（热力图 / 语义进度条 / 变量历史 / 异常回退） |

#### 算法与可视化

| 文档 | 说明 |
|------|------|
| [`current/ALGORITHM_DATASTRUCTURE_DESIGN.md`](current/ALGORITHM_DATASTRUCTURE_DESIGN.md) | 算法与数据结构支持设计（模式识别 / 运行时验证 / 轨迹分析） |
| [`current/DATASTRUCTURE_TEMPLATE_ROADMAP.md`](current/DATASTRUCTURE_TEMPLATE_ROADMAP.md) | 数据结构教材算法模板拓展路线图（P0/P1/P2 全景清单） |
| [`current/DATASTRUCTURE_SYNTAX_ROADMAP.md`](current/DATASTRUCTURE_SYNTAX_ROADMAP.md) | 数据结构教材语法支持路线图（编译器 C 子集补齐计划） |
| [`current/ZERO_INTRUSIVE_VISUALIZATION.md`](current/ZERO_INTRUSIVE_VISUALIZATION.md) | 零侵入可视化计划（数组排序 / 链表 / 二叉树） |
| [`current/SHADOW_VERIFICATION_FRAMEWORK.md`](current/SHADOW_VERIFICATION_FRAMEWORK.md) | Clang 影子验证框架 |
| [`current/SHADOW_VS_CI.md`](current/SHADOW_VS_CI.md) | 影子验证 vs CI 对比分析 |

#### 构建与开发

| 文档 | 说明 |
|------|------|
| [`current/BUILD.md`](current/BUILD.md) | 构建指南（环境要求 / 脚本用法 / 常见问题） |
| [`current/BUILD_SCRIPTS.md`](current/BUILD_SCRIPTS.md) | 构建脚本详细说明（`build_flutter.py` / `build_release.py` / `test_mobile.py`） |
| [`current/FLUTTER_BUILD_MANUAL.md`](current/FLUTTER_BUILD_MANUAL.md) | Flutter 手动构建指南（FRB 连接层 / 手动流程 / 故障排查） |
| [`current/FLUTTER_HOT_RELOAD_GUIDE.md`](current/FLUTTER_HOT_RELOAD_GUIDE.md) | Flutter 热重载调试指南 |
| [`current/CIDE_CLI.md`](current/CIDE_CLI.md) | `cide_cli` 命令行调试工具使用手册 |

#### Flutter 前端

| 文档 | 说明 |
|------|------|
| [`current/PANEL_DRAG_GESTURE_DESIGN.md`](current/PANEL_DRAG_GESTURE_DESIGN.md) | 面板拖拽手势设计（底部 Tab ↔ 悬浮球交换） |

> **说明**：编辑器已迁移为自研 `CideEditor`（`CideFlutter/lib/editor/`）。旧版基于 `re_editor` 的键盘、长按菜单实现文档已归档至 [`archive/`](archive/)。

#### 认知推理

| 文档 | 说明 |
|------|------|
| [`current/COGNITIVE_REASONING_ROADMAP.md`](current/COGNITIVE_REASONING_ROADMAP.md) | 认知推理系统路线图（根因分析 / 认知误区 / 知识图谱 / 意图推断） |

#### 代码审查与测试

| 文档 | 说明 |
|------|------|
| [`current/REVIEW_2026-06-14.md`](current/REVIEW_2026-06-14.md) | Cide 项目全面审阅报告（2026-06-14，最新） |
| [`current/code_review_report.md`](current/code_review_report.md) | Cide 项目全面代码审阅报告（2026-06-13/14 修订版） |
| [`current/STUDENT_ERROR_TEST_CASES.md`](current/STUDENT_ERROR_TEST_CASES.md) | 学生常见错误测试用例 |

#### C++ 扩展

| 文档 | 说明 |
|------|------|
| [`current/CPLUSPLUS_EXTENSION_PLAN.md`](current/CPLUSPLUS_EXTENSION_PLAN.md) | C++14 教学子集拓展实施计划（M7 Beta Readiness 已就绪） |
| [`current/M7_BETA_READINESS.md`](current/M7_BETA_READINESS.md) | M7 Beta Readiness 检查清单 |
| [`current/CPP_BUILTIN_LAYOUT_DECOUPLING_PLAN.md`](current/CPP_BUILTIN_LAYOUT_DECOUPLING_PLAN.md) | 内置容器布局解耦计划（已完成） |
| [`current/STAGE2B_CPP_CONTAINER_TEMPLATE_NOTES.md`](current/STAGE2B_CPP_CONTAINER_TEMPLATE_NOTES.md) | Stage 2b 内置 C++ 容器模板化迁移笔记 |
| [`current/TEMPLATE_AND_VERIFICATION_DECOUPLING.md`](current/TEMPLATE_AND_VERIFICATION_DECOUPLING.md) | 算法模板与验证解耦方案 |

#### 标准库与测试防线

| 文档 | 说明 |
|------|------|
| [`current/STDLIB_AND_TEST_DESIGN.md`](current/STDLIB_AND_TEST_DESIGN.md) | 标准库拓展与测试防线设计 |
| [`current/SUPPORTED_LIBC.md`](current/SUPPORTED_LIBC.md) | Cide 标准库支持矩阵 |
| [`current/PHASE_KR_LEETCODE_TEST_PLAN.md`](current/PHASE_KR_LEETCODE_TEST_PLAN.md) | K&R C 经典例题 + LeetCode 简单题测试覆盖计划 |
| [`current/S6_READINESS_ASSESSMENT.md`](current/S6_READINESS_ASSESSMENT.md) | S6 Dogfooding readiness 评估报告 |
| [`current/SHADOW_VERIFICATION_FRAMEWORK.md`](current/SHADOW_VERIFICATION_FRAMEWORK.md) | 影子验证框架 |

#### 计划与方案（部分待实现）

| 文档 | 说明 |
|------|------|
| [`current/LOCAL_PERSISTENCE_PLAN.md`](current/LOCAL_PERSISTENCE_PLAN.md) | 简单数据持久化与自动恢复方案（部分实现：`learning_progress_service.dart` 已使用 `shared_preferences`） |
| [`current/IMAGE_INPUT_INTEGRATION_PLAN.md`](current/IMAGE_INPUT_INTEGRATION_PLAN.md) | 拍照与本地图片输入集成方案 |

---

### 📁 [archive/](archive/) — 历史归档文档

存放已完成、废弃或不再维护的历史文档，包括：
- 各阶段迁移计划（MAUI → Flutter、C++ → Rust、WASM → 自定义 VM）
- 历史代码审查报告与事故复盘
- 已废弃技术方案（WASM、CodeMirror6、`re_editor` 编辑器、OCR 导入等）
- 已完成的实现计划（double 支持 / 函数指针 / 多文件编译 / 内存扩容 / C++ S6 进入计划等）
- 一次性修复记录与进度报告

**最近归档**（2026-06-15）：
- `ARCHIVE_CIDE_EDITOR_REFACTOR_PLAN.md`：编辑器重构草案（已落地为自研 `CideEditor`）
- `ARCHIVE_CUSTOM_KEYBOARD.md` / `ARCHIVE_EDITOR_LONG_PRESS_MENU.md`：基于已移除 `re_editor` 的实现记录
- `ARCHIVE_CODE_REVIEW_REPORT_2026_06_05.md` / `ARCHIVE_REVIEW_2026-06-08.md`：被 6-14 审阅报告替代
- `ARCHIVE_tempest-green-lantern-superman.md`：早期 C++ 扩展计划，已合并入 `CPLUSPLUS_EXTENSION_PLAN.md`
- `ARCHIVE_CPP_EXTENSION_ENTRY_S6_AUDIT.md`：Stage 6 进入评估报告（已完成，被后续 M7 Readiness 报告替代）
- `ARCHIVE_test2.md`：临时测试文件

> ⚠️ **archive/ 中的文档仅供追溯参考，内容可能已严重过时。**
