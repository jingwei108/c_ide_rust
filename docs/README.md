# Cide 项目文档

> 教学 C/C++ 子集参考执行引擎（白箱后端）——架构设计、语言子集规范、协议与测试防线
>
> 最后核对：2026-09-11（前端切割后重新整理：归档 17 份旧文档、删除英文文档、重写核心文档）

## 文档目录

### 📁 [current/](current/) — 当前有效文档

#### 定位、路线与架构

| 文档 | 说明 |
|------|------|
| [`current/CIDE_BACKEND_SPLIT_WASM_WHITEBOX_PLAN.md`](current/CIDE_BACKEND_SPLIT_WASM_WHITEBOX_PLAN.md) | **后端定位主计划**：前端切割决策、三出口一核心架构、协议先行、Phase 0~3 路线 |
| [`current/DESIGN.md`](current/DESIGN.md) | 架构总纲（编译器管线 / CideVM / 内存模型 / 时间旅行 / 诊断 / 协议 / 关键决策） |
| [`current/ROADMAP.md`](current/ROADMAP.md) | 路线图：当前状态、已完成里程碑、下一步、已知缺口（诚实记录） |
| [`current/MAINTENANCE_PLAN.md`](current/MAINTENANCE_PLAN.md) | 工程债务偿还与长期维护方案（`#DXX` 债务编号体系的事实源） |
| [`current/MEMORY_SAFETY.md`](current/MEMORY_SAFETY.md) | 内存安全规范（Rust 边界、线性内存、堆隔离与检查清单） |

#### 构建与上手

| 文档 | 说明 |
|------|------|
| [`current/QUICKSTART.md`](current/QUICKSTART.md) | 快速入门：命令行 / JSON-lines 会话 / wasm32 三条主路径 |
| [`current/BUILD.md`](current/BUILD.md) | 构建指南：引擎、CLI、wasm32、测试防线、脚本清单与排障 |
| [`current/CIDE_CLI.md`](current/CIDE_CLI.md) | `cide_cli` 使用手册（含 `serve` JSON-lines 协议契约与方法一览） |

#### 语言子集规范（行为契约）

| 文档 | 说明 |
|------|------|
| [`current/C_SUBSET_SPEC.md`](current/C_SUBSET_SPEC.md) | C 教学子集规范（支持语法 / 排除清单 / 与 Clang 的已记录差异） |
| [`current/CPP_SUBSET_SPEC.md`](current/CPP_SUBSET_SPEC.md) | C++14 教学子集规范（面向学生/教师，含 Honest Subset 边界） |
| [`current/CPLUSPLUS_EXTENSION_PLAN.md`](current/CPLUSPLUS_EXTENSION_PLAN.md) | C++ 子集拓展实施计划（Stage 0~6 与后续 Phase 全景） |
| [`current/STAGE2B_CPP_CONTAINER_TEMPLATE_NOTES.md`](current/STAGE2B_CPP_CONTAINER_TEMPLATE_NOTES.md) | 内置 C++ 容器模板化迁移笔记与编译器约束 |

#### 标准库与测试防线

| 文档 | 说明 |
|------|------|
| [`current/SUPPORTED_LIBC.md`](current/SUPPORTED_LIBC.md) | 标准库支持矩阵（头文件 × 函数 × 实现层 × 验证状态） |
| [`current/STDLIB_AND_TEST_DESIGN.md`](current/STDLIB_AND_TEST_DESIGN.md) | 标准库四层架构（VM Builtin / Rust Host / Bytecode Libc）与测试设计 |
| [`current/BYTECODE_LIBC_PRODUCTIZATION.md`](current/BYTECODE_LIBC_PRODUCTIZATION.md) | Bytecode Libc 产品化（构建期预编译 + 固定索引段） |
| [`current/SHADOW_VERIFICATION_FRAMEWORK.md`](current/SHADOW_VERIFICATION_FRAMEWORK.md) | 影子验证框架（Clang 对照、门禁语义、提速设施、已知限制） |
| [`current/STUDENT_ERROR_TEST_CASES.md`](current/STUDENT_ERROR_TEST_CASES.md) | 学生常见错误测试用例集（错误代码 + 预期诊断） |
| [`current/TODO_CONVENTION.md`](current/TODO_CONVENTION.md) | 代码内 TODO/FIXME/HACK/SAFETY 标签与 `#DXX` 编号约定 |

#### 统一模式、可视化与教学体验

| 文档 | 说明 |
|------|------|
| [`current/UNIFIED_MODE_DESIGN.md`](current/UNIFIED_MODE_DESIGN.md) | 统一模式 / 时间旅行设计（状态机、检查点、帧缓存、seek 契约） |
| [`current/VM_EXPERIENCE_ADVANTAGE.md`](current/VM_EXPERIENCE_ADVANTAGE.md) | 自研 VM 的体验优势（热力图 / 语义进度条 / 变量历史 / 异常回退） |
| [`current/ZERO_INTRUSIVE_VISUALIZATION.md`](current/ZERO_INTRUSIVE_VISUALIZATION.md) | 零侵入可视化设计（数组 / 链表 / 二叉树，自动识别与按需反推） |
| [`current/ALGORITHM_DATASTRUCTURE_DESIGN.md`](current/ALGORITHM_DATASTRUCTURE_DESIGN.md) | 算法与数据结构支持总设计（模式识别 / 运行时验证 / 轨迹分析） |
| [`current/COGNITIVE_REASONING_ROADMAP.md`](current/COGNITIVE_REASONING_ROADMAP.md) | 认知推理系统（根因分析 / 认知误区 / 知识图谱 / 意图推断） |
| [`current/DATASTRUCTURE_TEMPLATE_ROADMAP.md`](current/DATASTRUCTURE_TEMPLATE_ROADMAP.md) | 教材算法模板清单与覆盖现状 |
| [`current/TEMPLATE_GUIDE.md`](current/TEMPLATE_GUIDE.md) | 算法模板维护指南（目录结构、meta.yaml、占位符、验证链路） |
| [`current/TEMPLATE_AND_VERIFICATION_DECOUPLING.md`](current/TEMPLATE_AND_VERIFICATION_DECOUPLING.md) | 模板与验证解耦方案（模板即合法 C + Clang Golden） |

#### 出口、协议与引擎决议

| 文档 | 说明 |
|------|------|
| [`spec/STEP_PAYLOAD_SCHEMA_V0_1.md`](spec/STEP_PAYLOAD_SCHEMA_V0_1.md) | **StepPayload v0.1 语言中立协议 schema**（字段语义 / 窗口 / seek / 差分编码） |
| [`current/CIDE_CAPI_REVIEW_RESPONSE.md`](current/CIDE_CAPI_REVIEW_RESPONSE.md) | capi 签名评审定稿（外部消费者诉求逐条回应 + 分批实现状态） |
| [`current/CIDE_HEAP_QUARANTINE_DECISION.md`](current/CIDE_HEAP_QUARANTINE_DECISION.md) | 堆内存决议：bump 分配 + 有界隔离（三道墙） |

#### 质量、审查与工作记录

| 文档 | 说明 |
|------|------|
| [`current/code_review_report_2026-09-06.md`](current/code_review_report_2026-09-06.md) | 全面代码审阅报告（137 条发现）与四批修复追踪（**修复进度权威追踪**） |
| [`current/code_review_report_2026-09-11.md`](current/code_review_report_2026-09-11.md) | 外部 PR 清单 12 项复核与批次 A~H 修复记录 |
| [`current/WORKLOG_2026-09-11_SHADOW_SPEEDUP_AND_PHASE1.md`](current/WORKLOG_2026-09-11_SHADOW_SPEEDUP_AND_PHASE1.md) | 工作日志：Shadow 提速 / 隔离预算 / schema v0.1 / serve |

---

### 📁 [spec/](spec/) — 语言中立协议

对外承诺的 wire format 定义，与任何前端实现解耦。当前：

| 文档 | 说明 |
|------|------|
| [`spec/STEP_PAYLOAD_SCHEMA_V0_1.md`](spec/STEP_PAYLOAD_SCHEMA_V0_1.md) | StepPayload v0.1（v0.1 定稿候选，待对端回放校验后冻结） |

---

### 📁 [archive/](archive/) — 历史归档文档

存放**已完成、已废弃或对象已不在本仓库**的历史文档，仅供追溯：

> 命名约定：2026-09-11 起新归档统一加 `ARCHIVE_` 前缀并在标题下写入归档横幅（含归档原因与日期）；
> 更早期的归档文件保留原名（如 `FLUTTER_MIGRATION_PLAN.md`、`REVIEW_2026-06-14.md`）。

- 前端时代的迁移与构建（MAUI → Flutter、Flutter 构建脚本、web 部署、前端 UI 设计）
- 历史代码审查报告与事故复盘
- 已完成的实现计划（double / 函数指针 / 多文件编译 / 内存扩容 / 递归类型重构 / 指针复合赋值等）
- 一次性评估报告与工作记录

**2026-09-11 本次归档**（前端切割后）：

| 归档文件 | 原因 |
|------|------|
| `ARCHIVE_BUILD_SCRIPTS.md` | 所描述的 Flutter 构建脚本已全部移除 |
| `ARCHIVE_CI_FAILURES.md` | FRB / Android CI 故障载体已随 CI 收缩消失 |
| `ARCHIVE_code_review_report_2026-06-13.md` | 审阅范围含前端，已被 09-06 / 09-11 报告取代 |
| `ARCHIVE_CIDE_MOBILE_TEACHING_THREE_LANGUAGE_PLAN.md` | "移动端优先"定位已被后端主计划取代 |
| `ARCHIVE_CPP_BUILTIN_LAYOUT_DECOUPLING_PLAN.md` | 布局解耦已完成（Phase 41） |
| `ARCHIVE_DATASTRUCTURE_SYNTAX_ROADMAP.md` | 语法拓展已完成（Phase 27） |
| `ARCHIVE_IMAGE_INPUT_INTEGRATION_PLAN.md` | 依赖已移除的前端与 OCR 能力 |
| `ARCHIVE_LOCAL_PERSISTENCE_PLAN.md` | 方案载体（Dart 运行时）已迁出 |
| `ARCHIVE_M7_BETA_READINESS.md` | 里程碑评估已被 Phase 34~42 超越 |
| `ARCHIVE_PANEL_DRAG_GESTURE_DESIGN.md` | 前端交互设计，宿主已迁出 |
| `ARCHIVE_PHASE_KR_LEETCODE_TEST_PLAN.md` | 计划已达成（K&R 69 绿 / LeetCode 138 通过） |
| `ARCHIVE_POINTER_COMPOUND_ASSIGN_PLAN.md` | 已全链路支持（2026-06-28） |
| `ARCHIVE_RECURSIVE_TYPE_SYSTEM_REFACTOR.md` | 重构已落地于 `cide_ast` |
| `ARCHIVE_S6_READINESS_ASSESSMENT.md` | 阶段评估已被后续里程碑覆盖 |
| `ARCHIVE_SHADOW_VS_CI.md` | 立论前提（特性缺失期）已消失 |
| `ARCHIVE_WEB_DEPLOYMENT_CLOUDFLARE_AND_WASM_INTEGRATION.md` | Flutter Web 部署路径作废 |
| `ARCHIVE_WEB_DEPLOYMENT_GITHUB_AND_GITEE_PAGES.md` | 双 Pages 部署围绕已删除产物构建 |

> ⚠️ **archive/ 中的文档仅供追溯参考，内容可能已严重过时，且不再维护。**
> 英文文档（`README_EN.md` / `BUILD_EN.md` / `CIDE_CLI_EN.md` / `QUICKSTART_EN.md` 等）已于 2026-09-11 删除，
> 仓库中仅保留 [`AGENTS_EN.md`](../AGENTS_EN.md)；翻译工作后续再议。
