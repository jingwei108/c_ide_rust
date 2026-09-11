# Cide 路线图

> 最后核对：2026-09-11（前端切割后重写）
>
> **核心原则**：不急着发布，不把时间浪费在"能用"上；每一分投入都指向教学场景真正需要、而通用工具链不提供的能力。
> **当前定位**：教学 C/C++ 子集参考执行引擎（白箱后端，MIT）。前端已切割给社区，原生移动端放弃。

---

## 一、当前状态（2026-09-11）

| 维度 | 状态 |
|:---|:---|
| 引擎核心 | Rust workspace（10 个子 crate），编译管线 Lexer → Parser → TypeChecker → BytecodeGen → CideVM 全链路自研 |
| 出口 1 · C ABI | ✅ 可用；第一批 13 个入口全部落地，`cide_abi_version()` = 1.1.0 |
| 出口 2 · wasm32 | ✅ 冒烟实证（零修改构建 3.75MB，C API 全链路 + E3070 工作）；绑定包与 CI 固化待 Phase 2a |
| 出口 3 · serve | ✅ `cide_cli serve` JSON-lines 落地（id 关联 / 错误帧同构 / `session.reset` / 与 capi 共用 `session_api`） |
| 协议 | ✅ StepPayload schema v0.1 发布（[`../spec/STEP_PAYLOAD_SCHEMA_V0_1.md`](../spec/STEP_PAYLOAD_SCHEMA_V0_1.md)），待对端回放校验签字 |
| 测试防线 | ✅ 五层防线全绿：C Shadow 636 用例、C++ Shadow 100 用例、`cargo test` 845 passed / 0 failed、clippy 0 warning |
| C 子集 | ✅ 教学子集 + C99/C11 扩展（VLA / `_Generic` / 复合字面量 / 变参 / VFS 文件 I/O 等） |
| C++ 子集 | 🚧 Phase 42 进行中（Stage 0~6 已完成：类与继承、模板单态化、RAII、引用、移动构造、`unique_ptr`、内置容器） |
| 统一模式 | ✅ 快照 / 检查点 / Seek / 异常回退 / 逐语义标签 / 热力图，全链路可用 |
| 前端 | ⛔ 已迁出（标签 `before-frontend-split`） |

---

## 二、已完成里程碑

### 引擎与语言（Stage 0~8）

| 阶段 | 内容 | 状态 |
|:---|:---|:---|
| Stage 0 | 基础编译器（Lexer → Parser → AST → TypeChecker） | ✅ |
| Stage 1 | 自研 CideVM（字节码定义 + 解释器 + C API 接线 + 安全加固） | ✅ |
| Stage 2 | 运行时中文诊断（符号表导出 / 越界 / 除零 / 空指针 / 死循环分析） | ✅ |
| Stage 3 | 指令级单步 + 内存视图 + 指针追踪 | ✅ |
| Stage 4 | 零侵入可视化（算法识别骨架 + VM 运行时教学事件 + 运行时验证） | ✅ |
| Stage 5 | 诊断与修复系统（三级信息架构 + 结构化自动修复 + 知识卡片） | ✅ |
| Stage 6 | ~~前端搭建（Flutter + 自研编辑器）~~ → **已随前端切割迁出** | ⛔ |
| Stage 7 | C 子集拓展（`float`/位运算/三目/指针算术/`const`/`NULL`/函数指针等） | ✅ |
| Stage 8 | 统一模式 / 时间旅行（快照 + 检查点 + Seek + 自动执行 + 异常回退） | ✅ |

### 工程化（Phase 0~42，逐项记录见 [`AGENTS.md`](../../AGENTS.md) 与 [`CHANGELOG.md`](../../CHANGELOG.md)）

- **Phase 18~26**：地毯式审阅（P0 soundness 修复、VM 优化、clippy 0 警告）、UAF/Double-Free 检测、认知推理 P0~P3（轨迹根因 / 误区模式 / 知识图谱 / 意图推断）、语义补全 v2、模板 JIT
- **Phase 27~30**：数据结构语法拓展（数组退化 / `unsigned` / `const` / `extern` / VLA）、CLI 工具、Bytecode Libc 产品化、语法拓展（逗号运算符 / Designated Initializer / `offsetof`）
- **Phase 31~42**：C++ 教学子集（类/继承/模板单态化 → RAII → `new[]`·`delete[]` → 引用 → 隐式移动构造 → `unique_ptr` → 内置容器收口与布局解耦 → M6 测试防线 → **Phase 42 进行中**）
- **2026-09-06**：全面代码审阅（137 条发现）+ 四批修复；Shadow 与 CI 门禁"带牙齿"
- **2026-09-11**：**前端切割**（仓库转型纯后端）；capi 第一批；`cide_cli serve`；StepPayload schema v0.1；堆内存 bump + 有界隔离决议；Shadow 提速（103.6s → 1.1s 缓存命中）与 stdin 注入

---

## 三、进行中与下一步

阶段编排以 [`CIDE_BACKEND_SPLIT_WASM_WHITEBOX_PLAN.md`](CIDE_BACKEND_SPLIT_WASM_WHITEBOX_PLAN.md) §6 为准，此处为执行视图。

### Phase 0 · 切割准备 ✅ 已完成（2026-09-11）

仓库拆分（`CideFlutter/`、FRB 桥接、web 部署、Flutter 构建脚本迁出）、CI 收缩为纯后端、
AGENTS/README 重写、MIT 化、模板用例静态化。

### Phase 1 · 边界补全 🚧 收尾中

| 项 | 状态 |
|:---|:---|
| 语言中立层（`native/src/session_api.rs`） | ✅ capi 与 serve 共用同一入口语义 |
| capi 第一批 13 项（版本/JSON 编译运行/断点/单步/输出游标/保险丝配置/`cide_free_string`） | ✅ 全部落地 |
| `cide_cli serve`（id 关联 / 帧同构 / `session.reset`） | ✅ 落地并进 CI 冒烟（26 项断言） |
| StepPayload schema v0.1 | ✅ 文档发布 + 字段冻结测试；⏳ **对端（SharpTutor）三组回放场景待执行** |
| Issue A/B（scanf 空白指令、lambda 调用缺陷） | ✅ 已修并进回归防线 |
| 隔离预算会话配置（capi + serve 双出口） | ✅ |

### Phase 2a / 2b · 并行（下一步）

**2a · wasm 白箱**：JS/TS 绑定包、`scripts/wasm_smoke/` 固化并进 CI、浏览器最小 demo、体积优化（3.75MB → 2MB 内评估）。

**2b · capi 第二批**：`cide_get_memory_regions_json`（`kind: global|stack|heap` + `status`）、`cide_read_memory_bytes_json`、`cide_get_heap_stats_json`、`cide_get_struct_fields_json`、错误码表机器可读导出（含"超出教学子集"E4001~E4031 段与已知差异清单）。

### Phase 3 · 时间旅行完整面

capi 第三批（`run_auto_steps` / `seek_to_step` / `get_vis_events` / `get_heatmap` / `cide_demangle`）、
时间旅行 CoW（消除每步 1MB memcpy 的 O(n²)）、快照边界完整化（VFS / `local_sym_map` / step 派生伪时钟）。

### 持续项

- **Phase 42 · C++ 子集**：按教学需求继续拓展（当前边界见 [`CPP_SUBSET_SPEC.md`](CPP_SUBSET_SPEC.md)）；
- **文档同步**：实现变更必须同步 `C_SUBSET_SPEC.md` / `CPP_SUBSET_SPEC.md` / `CHANGELOG.md` / `AGENTS.md`；
- **防线维护**：`*_FAILURES.md` 双向对账（转绿未更新文档即 CI 失败）。

---

## 四、已知缺口（诚实记录）

| # | 缺口 | 影响 | 处置 |
|:---|:---|:---|:---|
| G1 | **模板 → 用例生成器缺失**：`scripts/sync_templates.py` 随前端切割移除，`native/tests/cases_template_generated/` 83 个用例目前是静态留存（仅被 `cide_e2e.rs`、`shadow_verify.py`、`extract_shadow_cases.py` 读取） | 改 `templates/` 后无法再生成用例，链路断裂 | 待认领：随 wasm 出口或社区前端一并恢复生成器（此前未被 AGENTS.md 记录，本次如实补记） |
| G2 | wasm 冒烟脚本未固化（临时目录） | wasm 出口缺 CI 守护 | Phase 2a：`scripts/wasm_smoke/` |
| G3 | wasm 下统一模式无 `catch_unwind`（panic → abort） | 与原生出口行为有差异 | 已记录，评估中 |
| G4 | `time()` / `clock()` 使用真实墙钟，破坏重放确定性 | 时间旅行回放可能不一致 | Phase 3 伪时钟 |
| G5 | `fprintf` 到自定义 `FILE*` 不落盘（写入被当作 stdout） | 与 Clang 不一致 | 已记录；教学场景请用 `fputs`/`fwrite`/`fputc` |
| G6 | `native/src/flutter_bridge.rs` 命名与定位残留 | 历史包装层，命名易误解 | 待重构收敛（与语言中立层合并或改名） |
| G7 | StepPayload schema v0.1 尚未完成对端回放校验 | 协议冻结未闭环 | Phase 1 收尾项 |
| G8 | 通用解释器路径仍有性能优化空间（时间旅行每步全量快照） | 10 万步级程序 seek 延迟 | Phase 3 CoW |
| G9 | **算法属性验证能力在后端从未落地**：`validate_algorithm()` / `ValidationResult` 在 `native/` 全目录零命中（原 `native/src/engine/algorithm_validator.rs` 不存在），原载体 `CideFlutter/lib/models/algorithm_validation.dart` 已随前端迁出；`LearningProgress` 进度追踪同理（后端仅有 `learning_path.rs` 的 `LearningPath`）。**澄清**：`AlgorithmMatch`（算法检测结果结构体）在 `native/src/session.rs` 确实存在，缺的是"运行时验证"环节本身 | 零侵入可视化的"运行时验证"维度缺后端支撑 | 待评估重建（2026-09-11 翻新时由文档核对发现，详见 [`ALGORITHM_DATASTRUCTURE_DESIGN.md`](ALGORITHM_DATASTRUCTURE_DESIGN.md) §7） |
| G10 | **C++ E2E 用例口径不一致**：`native/tests/CPP_FAILURES.md` 记 74 个，而 `native/tests/cases/cpp/` 实际 78 个 `.cpp`（`cide_e2e.rs::load_cpp_cases` 全量加载） | 文档数字与实际断言不一致 | 待对账（2026-09-11 翻新时发现，已记入 [`MAINTENANCE_PLAN.md`](MAINTENANCE_PLAN.md) 任务 D） |
| G11 | **工程债务回升**：生产代码 `unwrap/expect` 3 处（`crates/cide_typeck/src/decl.rs`，即 D14）、`scripts/engineering_health.py` 仍带前端时代口径且未进 CI（D15）、`decl.rs` 871 非空行超标（D16） | 与"生产代码 0 unwrap"的历史验收标准不符 | 见 [`MAINTENANCE_PLAN.md`](MAINTENANCE_PLAN.md) §二 债务清单（D14~D16） |
| G12 | **模板失败计数存在三套口径**：`AGENTS.md` 记"82 个，78 绿，**4 已知失败**"；`native/tests/cases_template_generated/E2E_FAILURES.md` 列 3 条（含 `infixEvaluation_default`）；`cide_e2e.rs::KNOWN_TEMPLATE_FAILURES` 与 `shadow_verify.py::KNOWN_FAILURE_CASES` **只列 2 条** —— `infixEvaluation_default` 未进入任一常量 | CI 双向对账存在盲点（该用例失败不会被门禁拦下） | 待防线负责人统一口径（2026-09-11 文档翻新时发现，见 [`TEMPLATE_GUIDE.md`](TEMPLATE_GUIDE.md)） |
| G13 | **两条 C++ 活约束未记入 `CPP_SUBSET_SPEC.md`**：同一模板类不可跨文件重复定义、`T()` 值初始化不支持（实测该规范中检索不到） | 学生可见的语言/工具链边界缺少权威记录 | 待转交该规范负责人（见 [`STAGE2B_CPP_CONTAINER_TEMPLATE_NOTES.md`](STAGE2B_CPP_CONTAINER_TEMPLATE_NOTES.md) §2.1/§2.2） |

> 其它已知语言子集差异见 [`C_SUBSET_SPEC.md`](C_SUBSET_SPEC.md) / [`CPP_SUBSET_SPEC.md`](CPP_SUBSET_SPEC.md)，
> 测试差异见 `native/tests/*_FAILURES.md`。

---

## 五、差异化能力（相对通用工具链）

1. **运行时中文教学诊断**：GDB 只说 `SIGSEGV`，OnlineGDB 只说 `Runtime Error`；Cide 说"你访问了 `arr[5]`，但数组只有 5 个元素，有效索引 0~4；发生在第 3 行；当前 `i = 5`"，并给出一键修复。
2. **零侵入算法可视化**：写纯 C 冒泡排序，引擎自动识别并给出每一步的比较/交换语义与数组快照，无需任何可视化 API。
3. **内存与指针白箱**：值级变量快照、指针四状态（Valid / Freed / Null / Dangling）、堆区域与泄漏报告、隔离区可视化。
4. **时间旅行回放**：任意回退、异常自动回退到上一步并给根因卡片——这是"程序不是黑箱"的直接教学载体。
5. **协议化交付**：三出口 + 语言中立 schema，任何前端/IDE/判分服务都能接入，而不必绑定某一种 UI 技术栈。

> 单步/回放体验的设计论证见 [`VM_EXPERIENCE_ADVANTAGE.md`](VM_EXPERIENCE_ADVANTAGE.md)。

---

## 六、文档地图

| 想知道什么 | 看哪里 |
|:---|:---|
| 后端定位与切割决策 | [`CIDE_BACKEND_SPLIT_WASM_WHITEBOX_PLAN.md`](CIDE_BACKEND_SPLIT_WASM_WHITEBOX_PLAN.md) |
| 架构总纲 | [`DESIGN.md`](DESIGN.md) |
| 构建与测试防线 | [`BUILD.md`](BUILD.md)、[`QUICKSTART.md`](QUICKSTART.md) |
| CLI 命令 | [`CIDE_CLI.md`](CIDE_CLI.md) |
| 语言子集契约 | [`C_SUBSET_SPEC.md`](C_SUBSET_SPEC.md)、[`CPP_SUBSET_SPEC.md`](CPP_SUBSET_SPEC.md) |
| 标准库支持矩阵 | [`SUPPORTED_LIBC.md`](SUPPORTED_LIBC.md) |
| 协议 schema | [`../spec/STEP_PAYLOAD_SCHEMA_V0_1.md`](../spec/STEP_PAYLOAD_SCHEMA_V0_1.md) |
| 审查与修复追踪 | [`code_review_report_2026-09-06.md`](code_review_report_2026-09-06.md)、[`code_review_report_2026-09-11.md`](code_review_report_2026-09-11.md) |
| 工程维护 | [`MAINTENANCE_PLAN.md`](MAINTENANCE_PLAN.md) |
| 历史文档 | [`../archive/`](../archive/)（仅供追溯，可能严重过时） |
