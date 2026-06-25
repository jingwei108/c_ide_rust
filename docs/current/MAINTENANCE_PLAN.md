# Cide 工程债务偿还维护方案

> 本方案基于 2026-06-15 全面评估结果制定，目标是在不破坏现有功能的前提下，系统化偿还工程债务，提升项目长期可维护性。

## 一、背景与目标

### 1.1 背景

Cide 项目已完成 Phase 0 ~ Phase 41 的大规模功能建设，C/C++ 教学子集支持度较高，测试防线基本健全。但在快速迭代过程中积累了以下工程债务：

- 超大源码文件导致审查与维护成本上升
- 生成代码与手写源码混管
- 前端大文件职责过重
- C++ 扩展模块间耦合度快速上升
- 部分文档数据口径不一致
- 缺少项目级 lint 配置
- 失败记录文件中已修复条目稀释活跃问题

### 1.2 目标

| 目标层级 | 具体目标 |
|----------|----------|
| P0 | 消除已识别的文档不一致与生成代码管理问题 |
| P1 | 拆分超大文件，降低单文件认知负荷 |
| P2 | 建立项目级 lint 配置与代码内问题追踪机制 |
| P3 | 优化前端绘制性能与统一模式大状态传递 |
| P4 | 建立 C++ 扩展模块边界，降低回归风险 |

### 1.3 核心原则

1. **功能优先**：所有重构必须保证现有测试防线全绿，禁止以偿还债务为名破坏功能。
2. **最小侵入**：每次变更只处理一个明确的债务点，避免大范围重写。
3. **诚实记录**：重构过程中发现的新问题必须立即记入对应 `*_FAILURES.md` 或新增 TODO 标记。
4. **持续集成**：任何拆分或重构必须通过 CI 全部 job 验证后方可合并。

---

## 二、工程债务清单与优先级

| 编号 | 债务项 | 位置 | 当前状态 | 影响 | 优先级 |
|------|--------|------|----------|------|--------|
| D01 | 文档数据口径不一致 | README.md / CHANGELOG.md / 审阅报告 | 已统一为 566/569（2026-06-25 实测） | 外部信任损耗 | P0 |
| D02 | FRB 生成文件仍被追踪 | `.gitignore` 已排除但文件可能仍被 git 追踪 | 174KB + 222KB 生成代码在版本库中 | 合并冲突、版本库体积 | P0 |
| D03 | `codegen/expr.rs` 过大 | `crates/cide_codegen/src/expr.rs`（2047 行，已拆分为 8 个子模块并随 `cide_codegen` crate 迁移） | ✅ 已完成：2047 → 510 行 | 维护与审查成本高 | P1 |
| D04 | `parser/mod.rs` 过大 | `crates/cide_parser/src/lib.rs`（原 `native/src/compiler/parser/mod.rs`，2633 行，已拆分并迁移） | 语法分析全堆在一个文件 | 维护与审查成本高 | P1 |
| D05 | `host_funcs.rs` 过大 | `crates/cide_vm/src/host_funcs.rs`（2545 行，已拆分并随 `cide_vm` crate 迁移） | 96 个 host 函数集中 | 维护与审查成本高 | P1 |
| D06 | `ide_screen.dart` 过大 | `CideFlutter/lib/screens/ide_screen.dart`（896 行） | 承载整个 IDE 布局 | 前端维护困难 | P1 |
| D07 | 缺少项目级 clippy 配置 | 无 `clippy.toml` | 仅命令行控制 lint | 规则不一致 | P2 |
| D08 | 代码内 TODO/FIXME 标记极少 | 全项目 | 已新增规范并标记 30+ 处 | 技术债务隐形化 | P2 |
| D09 | 前端 CustomPainter 缺少缓存 | `editor_painter.dart` 等 | 每帧重建绘制对象 | 性能热点 | P2 |
| D10 | C++ 扩展模块耦合度高 | Parser/TypeChecker/CodeGen | 类/模板/引用/RAII 大量交叉 | 回归风险高 | P3 |
| D11 | 失败记录文件稀释 | `FUZZ_FAILURES.md` 等 | 已修复条目过多 | 活跃问题难定位 | P2 |
| D12 | Mutex poison 静默恢复 | `flutter_bridge.rs` | 重置为默认值 | 可能掩盖 panic 根因 | P3 |
| D13 | `docs/archive/` 噪音 | `docs/archive/` | 协作过程文本价值密度低 | 文档噪音 | P4 |

---

## 三、阶段规划

### 第一阶段：清理与对齐（P0，预计 1 周）

目标：消除数据不一致与生成代码管理问题，为后续重构建立干净基线。

#### 任务 1.1：统一 Shadow Verification 数字口径

- **涉及文件**：
  - `README.md`
  - `AGENTS.md`
  - `CHANGELOG.md`
  - `native/tests/TEST_REPORT.md`
  - `reports/three_tier_report.md`（如存在）
- **执行步骤**：
  1. 运行 `python native/tests/shadow_verification/shadow_verify.py` 获取最新实测数据。
  2. 运行 `python scripts/shadow_verify_cpp.py` 获取最新 C++ 数据。
  3. 统计当前真实匹配数、编译缺口数、运行时缺口数、输出差异数。
  4. 统一所有文档中的数字，注明统计日期与工具版本。
- **验收标准**：所有文档中 Shadow Verification 数字完全一致，CI 不因此报错。
- **风险**：数字更新可能暴露新的未记录差异，需同步更新 `*_FAILURES.md`。

#### 任务 1.2：将 FRB 生成文件彻底移出版本控制

- **涉及文件**：
  - `native/src/frb_generated.rs`
  - `CideFlutter/lib/src/rust/frb_generated*.dart`
  - `.gitignore`
  - `.github/workflows/ci.yml`
- **执行步骤**：
  1. 确认 `.gitignore` 已包含相关文件（当前已包含）。
  2. 如果文件仍被 git 追踪，执行 `git rm --cached` 移出索引（不删除工作区文件）。
  3. 验证 CI 中 Rust job 与 Flutter job 均在构建前执行 `flutter_rust_bridge_codegen generate`。
  4. 本地执行 `flutter_rust_bridge_codegen generate` 验证生成产物可正常重建。
  5. 清理并重建桌面端与 Android 端，确保无编译错误。
- **验收标准**：
  - `git ls-files | grep frb_generated` 无输出。
  - CI Rust job 与 Flutter job 均成功生成绑定并通过。
  - 本地 `python scripts/build_flutter.py` 成功。
- **风险**：不同机器上 FRB 生成器版本差异可能导致生成产物不一致，需严格锁定版本 `=2.12.0`。

#### 任务 1.3：归档或清理已修复的失败记录条目

- **涉及文件**：
  - `FUZZ_FAILURES.md`
  - `HOST_CONTRACT_FAILURES.md`
  - `BYTECODE_LIBC_FAILURES.md`
  - `DIFFERENTIAL_FAILURES.md`
  - `GOLDEN_FAILURES.md`
- **执行步骤**：
  1. 遍历各失败记录文件，识别标记为“已修复”的条目。
  2. 将已修复条目迁移至 `docs/archive/failures_archive_2026.md`，保留历史轨迹。
  3. 在活跃失败记录文件中仅保留当前已知失败与边界说明。
  4. 更新 `scripts/ci_three_tier_check.py` 的解析逻辑，确保归档后仍能识别历史记录。
- **验收标准**：活跃失败记录文件中至少 70% 条目为当前问题或边界说明；CI 一致性检查通过。
- **风险**：归档过程可能误删仍有效的已知失败，需逐条人工复核。

#### 任务 1.4：完成 `ROADMAP_2026_Q3.md` 迁移

- **涉及文件**：
  - `docs/archive/ROADMAP_2026_Q3.md`
  - `docs/current/ROADMAP_2026_Q3.md`（如被删除）
- **执行步骤**：
  1. 确认该文件当前状态（删除、未跟踪、修改）。
  2. 决定最终存放位置（建议保留在 `docs/current/`）。
  3. 提交迁移变更，保持 git 工作区干净。
- **验收标准**：`git status` 不再显示该文件相关未提交状态。

---

### 第二阶段：文件拆分与模块化（P1，预计 3 周）

目标：降低超大文件认知负荷，提升模块边界清晰度。

#### 任务 2.1：拆分 `crates/cide_codegen/src/expr.rs`

- **拆分策略**：
  - 保留 `expr.rs` 为入口文件，导出 `gen_expr` 等公共函数。
  - 按表达式大类拆分为子模块：
    - `codegen/expr/literal.rs`：字面量、常量、字符串
    - `codegen/expr/binary.rs`：二元运算、逻辑运算、逗号运算符
    - `codegen/expr/unary.rs`：一元运算、自增自减、取地址、解引用
    - `codegen/expr/call.rs`：函数调用、方法调用、函数指针调用
    - `codegen/expr/struct.rs`：结构体/联合体访问、成员地址
    - `codegen/expr/array.rs`：数组索引、指针算术
    - `codegen/expr/cast.rs`：类型转换
    - `codegen/expr/new_delete.rs`：`new`/`new[]`/`delete`/`delete[]`
- **执行步骤**：
  1. 提取 `gen_expr` 内部的 match 分支到各子模块。
  2. 保持 `BytecodeGen` struct 不变，子模块通过 `impl BytecodeGen` 添加方法。
  3. 每拆分一个子模块，运行 `cargo test --workspace --all-features` 验证。
- **验收标准**：
  - `expr.rs` 行数降至 600 行以内。
  - 所有 Rust 测试通过。
  - Shadow Verification 无新增失败。
- **风险**：拆分时可能破坏 `self` 可变借用模式，需利用 Rust 借用检查器逐步调整。

#### 任务 2.2：拆分 `crates/cide_parser/src/lib.rs`（原 `native/src/compiler/parser/mod.rs`）

- **拆分策略**：
  - 保留 `parser/mod.rs` 为模块入口与核心状态机。
  - 按语法结构拆分为子模块：
    - `parser/decl.rs`：变量声明、函数声明、结构体/联合体/枚举声明
    - `parser/stmt.rs`：语句解析（if、while、for、switch、return 等）
    - `parser/expr.rs`：表达式解析
    - `parser/type_.rs`：类型说明符、声明符、抽象声明符
    - `parser/cpp.rs`：C++ 专属语法（class、模板、构造析构、引用）
- **执行步骤**：
  1. 将 `parse_xxx` 方法按主题迁移到子模块。
  2. 在 `parser/mod.rs` 中 `pub use` 需要的函数。
  3. 处理 Parser 内部字段访问权限，必要时添加 getter。
- **验收标准**：
  - `parser/mod.rs` 行数降至 1000 行以内。
  - 所有 Rust 测试通过。
  - C/C++ Shadow Verification 无新增失败。
- **风险**：Parser 内部状态高度耦合，拆分可能暴露隐式依赖，需小步提交。

#### 任务 2.3：拆分 `crates/cide_vm/src/host_funcs.rs`（原 `native/src/vm/host_funcs.rs`）

- **拆分策略**：
  - 保留 `host_funcs.rs` 为注册入口与公共工具函数。
  - 按功能拆分为子模块：
    - `crates/cide_vm/src/host/memory.rs`：`malloc`/`free`/`realloc`/`calloc`
    - `crates/cide_vm/src/host/string.rs`：`strlen`/`strcpy`/`strcmp`/`memcpy` 等
    - `crates/cide_vm/src/host/io.rs`：`printf`/`scanf`/`fprintf`/`fgets` 等
    - `crates/cide_vm/src/host/file.rs`：`fopen`/`fread`/`fwrite`/`fclose` 等
    - `crates/cide_vm/src/host/math.rs`：`sin`/`cos`/`sqrt`/`pow` 等
    - `crates/cide_vm/src/host/misc.rs`：`rand`/`srand`/`exit`/`qsort` 等
- **执行步骤**：
  1. 将 host 函数按功能迁移。
  2. 提取共享工具如 `parse_format_spec`、`write_memory` 到 `vm/host/utils.rs`。
  3. 更新 `host_func_id.rs` 与注册逻辑。
- **验收标准**：
  - `host_funcs.rs` 行数降至 600 行以内。
  - Host Contract 测试、Fuzz 测试、Shadow Verification 全绿。
- **风险**：部分 host 函数共享全局状态（如 VFS、堆管理），拆分需保持状态访问路径一致。

#### 任务 2.4：拆分 `CideFlutter/lib/screens/ide_screen.dart`

- **拆分策略**：
  - 保留 `ide_screen.dart` 为页面骨架与布局组合。
  - 将内部组件提取为独立 widget：
    - `lib/screens/ide/toolbar.dart`：顶部工具栏
    - `lib/screens/ide/template_bar.dart`：模板栏
    - `lib/screens/ide/editor_area.dart`：编辑器区域
    - `lib/screens/ide/bottom_panel.dart`：底部面板
    - `lib/screens/ide/floating_orb_area.dart`：悬浮球区域
    - `lib/screens/ide/keyboard_handler.dart`：键盘与快捷键处理
- **执行步骤**：
  1. 识别 `ide_screen.dart` 中可独立的状态块。
  2. 使用 `Consumer`/`ref.watch` 保持状态订阅。
  3. 每提取一个 widget，运行 `flutter test` 与 `flutter analyze` 验证。
- **验收标准**：
  - `ide_screen.dart` 行数降至 300 行以内。
  - `flutter analyze` 0 issues。
  - 集成测试通过。
- **风险**：拆分过程中可能破坏状态监听链路，需通过集成测试捕获。

---

### 第三阶段：规范与质量加固（P2，预计 2 周）

目标：建立项目级 lint 配置、代码内问题追踪机制，优化前端性能热点。

#### 任务 3.1：建立项目级 Clippy 配置

- **涉及文件**：
  - `native/Cargo.toml` 或新建 `native/clippy.toml`
  - 各模块源码
- **执行步骤**：
  1. 在 `Cargo.toml` 中声明 `[lints.clippy]` 规则，例如：
     - `unwrap_used = "deny"`
     - `expect_used = "warn"`
     - `missing_panics_doc = "warn"`
     - `too_many_lines = "warn"`
     - `type_complexity = "allow"`（如现有类型确实复杂）
  2. 逐步修复新增 lint 报错，优先处理高优先级模块。
  3. 对暂时无法修复的地方添加显式 `#[allow(...)]` 并附注释说明原因。
- **验收标准**：`cargo clippy --all-targets -- -D warnings` 仍然全绿；新增 lint 规则生效。
- **风险**：`unwrap_used` 可能导致大量报错，建议先设为 `warn`，分阶段提升为 `deny`。

#### 任务 3.2：引入代码内 TODO/FIXME 追踪规范

- **涉及文件**：全项目
- **执行步骤**：
  1. 制定注释规范：
     - `// TODO(#<issue>): 说明`：已知待改进点
     - `// FIXME(#<issue>): 说明`：已知缺陷
     - `// HACK: 说明`：临时 workaround
     - `// NOTE: 说明`：重要设计决策
  2. 扫描现有代码，补充关键位置的 TODO/FIXME 标记。
  3. 在 CI 或脚本中增加 TODO/FIXME 统计，定期 review。
- **验收标准**：关键 workaround 与边界情况均有代码内标记；维护者可通过 grep 快速定位。
- **风险**：过度标记会制造噪音，应聚焦真正需要跟踪的问题。

#### 任务 3.3：前端 CustomPainter 绘制缓存优化

- **涉及文件**：
  - `CideFlutter/lib/editor/editor_painter.dart`
  - `CideFlutter/lib/widgets/floating_orb_widget.dart`
  - `CideFlutter/lib/widgets/visualizers/*.dart`
- **执行步骤**：
  1. 对 `TextPainter`、`ParagraphBuilder`、`Gradient`、`Blur` 等对象实施缓存，仅在文本/数据变化时重建。
  2. 为动画组件添加 `RepaintBoundary` 隔离重绘区域。
  3. 使用 `shouldRepaint` 精确控制重绘。
  4. 在桌面端与移动端分别进行性能测试（观察帧率、CPU 占用）。
- **验收标准**：
  - 复杂可视化场景下帧率不低于 55fps。
  - `flutter test` 与集成测试通过。
- **风险**：缓存逻辑引入状态同步复杂度，需确保数据更新时正确失效。

#### 任务 3.4：统一模式大状态传递优化

- **涉及文件**：
  - `native/src/api/cide.rs`
  - `native/src/flutter_bridge.rs`
  - `CideFlutter/lib/providers/unified_notifier.dart`
- **执行步骤**：
  1. 评估当前 `StepPayload`/`StepPayloadDelta` 字段必要性，剔除冗余字段。
  2. 对符号表、变量历史等大对象启用增量更新或分页。
  3. 在 Dart 端使用 `compute` 或 isolate 处理大状态反序列化。
  4. 增加状态大小日志，监控异常增长。
- **验收标准**：
  - 10 万步统一模式下前端仍保持流畅。
  - 内存占用无明显增长。
- **风险**：增量更新逻辑复杂，可能引入状态不一致。

---

### 第四阶段：C++ 扩展模块化（P3，预计 4 周）

目标：降低 C++ 扩展在 Parser/TypeChecker/CodeGen 中的耦合度，减少回归风险。

#### 任务 4.1：建立 C++ 语法专属解析模块

- **涉及文件**：
  - `crates/cide_parser/src/cpp.rs`（新建）
  - `crates/cide_parser/src/lib.rs`
- **执行步骤**：
  1. 将 C++ class、模板、构造析构、引用等语法解析逻辑集中到 `parser/cpp.rs`。
  2. 在 `parser/mod.rs` 中通过 `parse_cpp_xxx` 调用入口。
  3. 保持 C 解析路径不被 C++ 逻辑污染。
- **验收标准**：C 解析模块中不出现 `Class`、`Template`、`Reference` 等 C++ 专属分支。

#### 任务 4.2：建立 C++ 类型检查模块边界

- **涉及文件**：
  - `crates/cide_typeck/src/cpp/`
  - `crates/cide_typeck/src/lib.rs`
- **执行步骤**：
  1. 将 C++ 类布局、方法解析、重载、引用语义迁移到 `typeck/cpp/` 子模块。
  2. 明确 C++ typeck 与 C typeck 的调用边界。
  3. 提取公共工具函数到 `typeck/cpp/utils.rs`。
- **验收标准**：`typeck/mod.rs` 对 C++ 逻辑的依赖通过明确接口完成。

#### 任务 4.3：建立 C++ 字节码生成模块边界

- **涉及文件**：
  - `crates/cide_codegen/src/cpp/`
  - `crates/cide_codegen/src/expr.rs` / `stmt/`
- **执行步骤**：
  1. 将 C++ 构造析构调用、方法调用、引用处理、移动构造等逻辑迁移到 `codegen/cpp/`。
  2. 在通用 `gen_expr`/`gen_stmt` 中通过类型判断分派到 C++ 处理模块。
- **验收标准**：C++ 代码生成变更不再扩散到通用表达式生成逻辑。

#### 任务 4.4：C++ 容器布局维护流程固化

- **涉及文件**：
  - `native/runtime_libc/cide/*.cpp`
  - `scripts/extract_cpp_builtin_layout.py`
  - `crates/cide_cpp_frontend/src/builtin_layout_data.json`
- **执行步骤**：
  1. 文档化容器新增流程：编辑 `.cpp` → 运行提取脚本 → 验证 JSON → 跑测试。
  2. 在 CI 中增加 `.cpp` 接口声明的语法检查（`clang++ -fsyntax-only`）。
  3. 确保 Rust 代码中不再新增硬编码容器信息。
- **验收标准**：新增容器必须仅修改 `.cpp` 与脚本，无需改动 Rust 源码。

---

### 第五阶段：长期健康度维护（P4，持续推进）

目标：保持文档整洁、监控工程健康度、持续消除小额债务。

#### 任务 5.1：定期清理 `docs/archive/`

- **执行步骤**：
  1. 每季度 review `docs/archive/`，删除无价值的历史交互文本。
  2. 对保留的归档文档添加摘要说明，便于后续检索。
- **验收标准**：`docs/archive/` 体积季度环比下降或保持稳定。

#### 任务 5.2：建立工程健康度看板 ✅ 已完成

- **执行步骤**：
  1. 新增 `scripts/engineering_health.py` 健康度看板脚本，统计以下指标：
     - 各 Rust / Dart 源文件行数 Top 20
     - TODO/FIXME/HACK 数量（按文件分布 Top 10）
     - `unwrap`/`expect` 使用数量（按文件分布 Top 10）
     - 失败记录文件中活跃问题数量
     - Shadow Verification 匹配率（C / C++）
  2. 生成报告到 `reports/engineering_health.md`；排除 `frb_generated*` 生成文件。
- **验收标准**：维护者可定期查看工程健康度趋势。

#### 任务 5.3：Mutex poison 处理增强

- **涉及文件**：`native/src/flutter_bridge.rs`
- **执行步骤**：
  1. 在 poison 恢复路径增加日志记录。
  2. 评估是否需要 panic 而非恢复默认值。
  3. 增加指标或测试覆盖 poison 场景。
- **验收标准**：poison 不再静默恢复，至少留下可观测痕迹。

---

## 四、执行节奏与里程碑

| 阶段 | 时间 | 里程碑 | 关键交付物 |
|------|------|--------|------------|
| 第一阶段 | 第 1 周 | 基线清理完成 | 文档数字一致、FRB 生成文件移出版本库、失败记录归档、工作区干净 |
| 第二阶段 | 第 2~4 周 | 超大文件拆分完成 | `expr.rs`/`parser/mod.rs`/`host_funcs.rs`/`ide_screen.dart` 行数达标 |
| 第三阶段 | 第 5~6 周 | 规范与性能加固完成 | 项目级 clippy 配置生效、TODO 规范落地、前端绘制性能优化 |
| 第四阶段 | 第 7~10 周 | C++ 扩展模块化完成 | C++ Parser/TypeChecker/CodeGen 边界清晰、容器布局流程固化 |
| 第五阶段 | 持续 | 健康度维护常态化 | 季度 archive 清理、工程健康度看板 |

---

## 五、质量保证

### 5.1 每个任务必须通过的验证

1. **单元测试**：`cd native && cargo test --workspace --all-features` 全绿。
2. **Lint**：`cargo clippy --all-targets -- -D warnings` 全绿。
3. **格式化**：`cargo fmt --check` 通过。
4. **Shadow Verification**：C/C++ Shadow Verification 无新增失败。
5. **前端静态检查**：`flutter analyze` 0 issues。
6. **前端测试**：`flutter test` 全绿。
7. **集成测试**：CI 全量 workflow 通过。

### 5.2 变更管理

- 每个任务独立分支，禁止混合无关变更。
- 每个 PR 必须关联本方案中的债务编号（如“偿还 D03”）。
- PR 描述中必须说明：变更范围、测试验证结果、已知风险。

### 5.3 回退策略

- 若拆分过程中发现功能回退，立即停止拆分，回滚到上一个稳定提交。
- 若新增 lint 规则导致大量报错，先降级为 `warn`，分阶段修复。
- 若 FRB 生成文件移出后 CI 失败，检查生成器版本与缓存配置。

---

## 六、附录：债务追踪表

| 编号 | 债务项 | 阶段 | 状态 | 负责人（角色） | 备注 |
|------|--------|------|------|----------------|------|
| D01 | 文档数据口径不一致 | 一 | ✅ 已完成 | 文档维护者 | 2026-06-25 实测统一为 566/569 |
| D02 | FRB 生成文件管理 | 一 | ✅ 已完成 | CI/构建维护者 | 提交 `ab39aaa` 已改为构建时生成；`.gitignore` 已配置 |
| D03 | `codegen/expr.rs` 过大 | 二 | ✅ 已完成 | 编译器维护者 | 2047 → 510 行；新增 8 个子模块 |
| D04 | `parser/mod.rs` 过大 | 二 | ✅ 已完成 | 编译器维护者 | 2633 → 672 行；新增 5 个子模块 |
| D05 | `host_funcs.rs` 过大 | 二 | ✅ 已完成 | VM 维护者 | 2545 → 155 行；新增 7 个子模块 |
| D06 | `ide_screen.dart` 过大 | 二 | ✅ 已完成 | 前端维护者 | 896 → 299 行；新增 6 个组件 |
| D07 | 缺少项目级 clippy 配置 | 三 | ✅ 已完成 | Rust 维护者 | 新增 `[lints.clippy]` + `clippy.toml` + `scripts/lint_check.sh` |
| D08 | TODO/FIXME 标记极少 | 三 | ✅ 已完成 | 全团队 | 新增 `docs/current/TODO_CONVENTION.md`，源码标记 30+ 处 |
| D09 | CustomPainter 缺少缓存 | 三 | ✅ 已完成 | 前端维护者 | Array/Tree/LinkedList Visualizer 缓存 parsed numbers 与 TextPainter；shouldRepaint 精确化；RepaintBoundary 隔离 |
| D10 | C++ 扩展模块耦合度高 | 四 | ✅ 已完成 | C++ 扩展维护者 | 已建立 typeck/cpp/、codegen/cpp/、parser/cpp/ 边界；class 构造/引用/RAII、RangeFor、template、类外方法/静态字段均已下沉 |
| D11 | 失败记录文件稀释 | 一 | ⏸️ 保留不动 | 测试维护者 | 按用户决策，暂不归档 |
| D12 | Mutex poison 静默恢复 | 五 | ✅ 已完成 | 桥接维护者 | 增加 #[track_caller]、全局 POISON_COUNT 计数、调用位置日志 |
| D13 | `docs/archive/` 噪音 | 五 | ⏸️ 保留不动 | 文档维护者 | 按用户决策，暂不清理 |

---

## 七、修订记录

| 日期 | 版本 | 修订内容 | 修订人 |
|------|------|----------|--------|
| 2026-06-15 | 1.0 | 初始版本 | 代码审查 Agent |
| 2026-06-16 | 1.1 | 完成 P0 清理对齐；完成 P1 超大文件拆分（expr.rs/parser/mod.rs/host_funcs.rs/ide_screen.dart） | 维护 Agent |
| 2026-06-16 | 1.2 | 启动 P2 规范与质量加固（clippy 配置、TODO 规范、CustomPainter 缓存、统一模式状态优化） | 维护 Agent |
| 2026-06-16 | 1.3 | 完成 P2 四任务：clippy 配置、TODO/FIXME 规范、CustomPainter 缓存、统一模式大状态优化 | 维护 Agent |
| 2026-06-16 | 1.4 | 推进 P3/P5：建立 typeck/cpp/、codegen/cpp/、parser/cpp/ 初步边界；修复 D12 Mutex poison 可观测性 | 维护 Agent |
| 2026-06-16 | 1.5 | 继续推进 D10：RangeFor 生成下沉、parser 构造/静态字段解析下沉 | 维护 Agent |
| 2026-06-16 | 1.6 | 完成 D10：VarDecl class/引用/RAII、parser class/template 顶层分发全部下沉到 cpp/ 子模块 | 维护 Agent |
| 2026-06-16 | 1.7 | 推进 P5：新增 `scripts/engineering_health.py` 工程健康度看板脚本；`shadow_verify.py` 同步更新 `*_latest.*` 文件；生成首份 `reports/engineering_health.md` | 维护 Agent |
| 2026-06-17 | 1.8 | 基于全面评估追加后续计划：架构拆分、unwrap 收敛、失败记录口径整理、CI 加固、性能收尾 | 评估 Agent |
| 2026-06-17 | 1.9 | 推进任务 D：统一失败记录统计口径为 KNOWN_FAILURE/DIVERGENCE/LIMITATION；修正 CPP_FAILURES.md 用例数 60→61；KR_FAILURES.md 明确当前 0 活跃失败；更新 engineering_health.py 与首份新口径报告 | 维护 Agent |
| 2026-06-17 | 2.0 | 完成任务 C：澄清生产代码 unwrap/expect 口径为 17 处；engineering_health.py 新增生产代码/测试代码/生成代码区分统计；Cargo.toml 将 `unwrap_used` 提升为 `deny`；lib.rs 更新豁免注释；cargo test --all-features / clippy / fmt 全绿 | 维护 Agent |
| 2026-06-17 | 2.1 | 推进任务 B：拆分 `compiler/lexer.rs`（1608 → 655 行），新增 `lexer/token.rs`、`keyword.rs`、`number.rs`、`string.rs`、`comment.rs`、`preprocessor.rs` 子模块；Lexer 字段改为 `pub(crate)` 以支持跨模块 impl | 维护 Agent |
| 2026-06-17 | 2.2 | 推进任务 B：拆分 `compiler/ast.rs`（1253 → 76 行），新增 `ast/types.rs`、`expr.rs`、`stmt.rs`、`decl.rs` 子模块；通过 `pub use` 保持外部 API 不变 | 维护 Agent |
| 2026-06-17 | 2.3 | 推进任务 B：拆分 `compiler/typeck/mod.rs`（1326 → 485 行），新增 `typeck/context.rs`、`typeck/convert.rs`、`typeck/init.rs`、`typeck/symbols.rs`；`TypeChecker` 字段可见性调整为 `pub(crate)` | 维护 Agent |
| 2026-06-17 | 2.4 | 推进任务 B：拆分 `unified/algorithm_steps.rs`（1439 → 170 行），新增 `algorithm_steps/sorting.rs`、`graph.rs`、`tree.rs`、`structures.rs`、`search.rs`、`math.rs`、`dp.rs` | 维护 Agent |
| 2026-06-17 | 2.5 | 推进任务 B：拆分 `vm/core/executor.rs`（1601 → 451 行），新增 `executor/arithmetic.rs`、`memory.rs`、`control.rs`、`float.rs`、`stack.rs`、`debug.rs` | 维护 Agent |
| 2026-06-17 | 2.6 | 推进任务 B：拆分 `compiler/parser/expr.rs`（1170 → 50 行），新增 `parser/expr/ops.rs`、`unary.rs`、`postfix.rs`、`primary.rs` | 维护 Agent |
| 2026-06-17 | 2.7 | 推进任务 B：拆分 `compiler/typeck/expr.rs`（1162 → 183 行），新增 `typeck/expr/ops.rs`、`literal.rs`、`var.rs`、`call.rs`、`cast.rs`、`cpp.rs` | 维护 Agent |
| 2026-06-17 | 2.8 | 推进任务 B：拆分剩余三个超大文件——`compiler/algorithm_detector.rs`（1145 → 44 行入口）拆分为 8 个算法类别子模块；`vm/core/mod.rs`（1221 → 30 行入口）拆分为 `state.rs`/`memory.rs`/`snapshot.rs`；`compiler/codegen/mod.rs`（1148 → 759 行）拆分为 `func.rs`/`init.rs`/`tests.rs`；cargo test / clippy / fmt / C/C++ Shadow Verification 全绿 | 维护 Agent |
| 2026-06-17 | 2.9 | 完成任务 B：拆分 `compiler/codegen/stmt.rs`（884 → 85 行入口），新增 `stmt/var_decl.rs`、`stmt/control.rs`、`stmt/switch.rs`、`stmt/block.rs`、`stmt/expr_stmt.rs`、`stmt/cpp.rs`；cargo test --all-features / clippy / fmt 全绿，C Shadow Verification 505/511，C++ Shadow Verification 83/83 无新增失败 | 维护 Agent |
| 2026-06-17 | 3.0 | 完成任务 D：失败记录文件口径整理。修复 `CPP_FAILURES.md` 中嵌入的 NUL 字节；将 `cases_golden/GOLDEN_FAILURES.md` 纳入活跃失败统计并标注 `KNOWN_DIVERGENCE`；修正 `engineering_health.py` 中不存在的 `native/tests/GOLDEN_FAILURES.md` 路径；重新生成 `engineering_health.md`，活跃失败记录口径统一为 14 条 | 维护 Agent |
| 2026-06-17 | 3.1 | 推进任务 E：CI 与构建系统加固。`scripts/build_flutter.py --test` 改为 `cargo test --all-features` 与 `cargo clippy --all-targets -- -D warnings`；CI Rust job 同步使用 `cargo test --all-features`；新增 `scripts/patch_flutter_windows_generator.py` 脚本化 Flutter CMAKE_GENERATOR patch；Android job 增加 `flutter test`；`Cargo.toml` 锁定 `serde`/`serde_json`/`libm` 小版本 | 维护 Agent |
| 2026-06-17 | 3.2 | 推进任务 F：性能收尾。Visualizer 缓存落地：`array_visualizer.dart` 缓存 parsed numbers；`tree_visualizer.dart` 与 `linked_list_visualizer.dart` 将 TextPainter 创建上提到 State 并复用，通过 `saveLayer` 应用动态透明度；`unified_notifier.dart` 增加 `frameCache` 5000 帧上限兜底。统一模式差分编码：变量级差分、Dart isolate 解码、大 batch 阈值已在位；数组/指针/调用栈全字段差分仍有剩余工作 | 维护 Agent |
| 2026-06-17 | 3.3 | 完成任务 F 剩余差分编码：`unified::stream::StepPayloadDelta` 对 `call_stack`/`vis_events`/`accessed_vars` 使用 `Option<Vec<T>>` 差分；`array_snapshots`/`pointer_snapshots` 实现按名索引的新增/替换/删除差分；Dart 端 `UnifiedNotifier` 同步应用差分；新增 Rust 单元测试覆盖 roundtrip；C Shadow Verification 505/511、C++ Shadow Verification 83/83 无新增失败 | 维护 Agent |
| 2026-06-17 | 3.4 | 拆分 `unified/stream.rs`（948 → 490 行），新增 `stream/encode.rs`、`stream/decode.rs`、`stream/diff.rs`，保持 `encode_payloads`/`decode_batch` 公共入口不变；`cargo test --all-features` / clippy / fmt / C/C++ Shadow Verification / `flutter test` 全绿 | 维护 Agent |
| 2026-06-17 | 3.5 | 启动任务 A：Workspace 拆分。`native/Cargo.toml` 增加 `[workspace]`；新增 `crates/cide_shared`（SourceLoc）与 `crates/cide_ast`（AST 节点/类型系统）；`compiler/mod.rs` 与 `shared/mod.rs` 通过 `pub use` 保持既有路径兼容；`cargo test --all-features` / clippy / fmt / C Shadow Verification 505/511 / C++ Shadow Verification 83/83 / `flutter test` 全绿 | 维护 Agent |
| 2026-06-17 | 3.6 | 尝试拆分 `cide_diagnostics` 独立 crate：因 FRB 在 `cide_native` 中为外部 crate 类型生成 `IntoIntoDart` 实现触发孤儿规则（orphan rules）而回退；`diagnostics` 仍保留在 `cide_native` 内部，待 FRB 类型处理策略明确后再迁移 | 维护 Agent |
| 2026-06-18 | 3.7 | 推进任务 G：新增 5 道 LeetCode 中等题（lc_3 / lc_33 / lc_48 / lc_62 / lc_64）；修复 JIT 统计信息污染 stdout 问题，新增 `cide_get_jit_stats` C API；文档 Shadow Verification 数字统一更新为 509/516；C/C++ Shadow Verification 全绿 | 维护 Agent |
| 2026-06-18 | 3.8 | 继续推进任务 G：新增 5 道 LeetCode 中等题（lc_2 / lc_11 / lc_19 / lc_31 / lc_34）；文档 Shadow Verification 数字统一更新为 515/521；C/C++ Shadow Verification / `cargo test --all-features` / `flutter test` 全绿 | 维护 Agent |
| 2026-06-18 | 3.9 | 继续推进任务 G：新增 5 道 LeetCode 中等题（lc_15 / lc_39 / lc_46 / lc_75 / lc_198）；文档 Shadow Verification 数字统一更新为 520/526；C/C++ Shadow Verification / `cargo test --all-features` / `flutter test` 全绿 | 维护 Agent |
| 2026-06-18 | 4.0 | 继续推进任务 G：新增 5 道 LeetCode 中等题（lc_55 / lc_142 / lc_152 / lc_200 / lc_221）；文档 Shadow Verification 数字统一更新为 525/531；C/C++ Shadow Verification / `cargo test --all-features` / `flutter test` 全绿 | 维护 Agent |
| 2026-06-18 | 4.1 | 继续推进任务 G：新增 5 道 LeetCode 中等题（lc_49 / lc_56 / lc_78 / lc_102 / lc_139）；文档 Shadow Verification 数字统一更新为 530/536；C/C++ Shadow Verification / `cargo test --all-features` / `flutter test` 全绿 | 维护 Agent |
| 2026-06-18 | 4.2 | 完成任务 G：all in 填充最后 5 道 LeetCode 中等题（lc_153 / lc_162 / lc_300 / lc_394 / lc_560），中等题达到 30 道上限；文档 Shadow Verification 数字统一更新为 535/541；C/C++ Shadow Verification / `cargo test --all-features` / `flutter test` / 三层契约检查全绿 | 维护 Agent |
| 2026-06-18 | 4.3 | 继续 all in 填充 LeetCode：新增 15 道混合难度题（lc_4 / lc_23 / lc_25 / lc_42 / lc_45 / lc_53 / lc_73 / lc_76 / lc_84 / lc_91 / lc_98 / lc_124 / lc_146 / lc_207 / lc_322），含 7 道困难题、7 道中等题、1 道简单题；诚实记录 `lc_4` 实现中发现 Cide 函数返回 `double` 值异常（输出 0.0）的行为差异，已改用整数返回值实现并通过；文档 Shadow Verification 数字统一更新为 549/555；C/C++ Shadow Verification / `cargo test --all-features` / `flutter test` / 三层契约检查全绿 | 维护 Agent |
| 2026-06-18 | 4.4 | 推进任务 G：新增 K&R 第 7 章 7 个用例（kr_7_1~kr_7_7），K&R 防线扩展至 76/76；新增 C++ 教学知识卡片 E4100~E4104；扩展 `STUDENT_ERROR_TEST_CASES.md` C++ 常见错误章节；Shadow Verification 数字更新为 555/562；cargo check / e2e / Shadow Verification 全绿 | 维护 Agent |
| 2026-06-18 | 4.5 | 推进任务 A：Workspace 拆分。新增 `crates/cide_runtime`（运行时共享数据：func_meta/symbol/type_utils、RuntimeState/MemoryState、内存布局常量、unified 基础数据）；新增 `crates/cide_vm`（完整迁移 `native/src/vm/`）；`native/src/vm/` 删除，`cide_native` 通过 `pub use cide_vm as vm;` 保持路径兼容；引入 `VmContext` 替代 `Session` 上帝对象；修复测试文件中的 VM 调用签名；`cargo test --all-features` / `cargo clippy --all-targets -- -D warnings` / `cargo fmt --check` / C Shadow Verification 555/562 / C++ Shadow Verification 83/83 / `flutter test` 全绿 | 维护 Agent |
| 2026-06-18 | 4.6 | 继续推进任务 A：新增 `crates/cide_lexer`。将 `ErrorCode` 下沉到 `crates/cide_shared`；将 `native/src/compiler/lexer.rs` 与 `native/src/compiler/lexer/` 子模块完整迁移到 `crates/cide_lexer`；`native/src/compiler/mod.rs` 通过 `pub use cide_lexer as lexer;` 保持路径兼容；`native/src/compiler/lexer.rs` 与 `native/src/compiler/lexer/` 删除；`cargo test --all-features` / `cargo clippy --all-targets -- -D warnings` / `cargo fmt --check` / C Shadow Verification 555/562 / C++ Shadow Verification 83/83 全绿 | 维护 Agent |
| 2026-06-18 | 4.7 | 继续推进任务 A：新增 `crates/cide_parser`。将 `native/src/compiler/parser/` 全部 10 个文件完整迁移到 `crates/cide_parser`；`cide_shared` 新增 `SourceLoc` re-export；`cide_native` 通过 `pub use cide_parser as parser;` 保持路径兼容；`native/src/compiler/parser/` 删除；`cargo test --all-features` / `cargo clippy --all-targets -- -D warnings` / `cargo fmt --check` / C Shadow Verification 555/562 / C++ Shadow Verification 83/83 全绿 | 维护 Agent |
| 2026-06-18 | 4.8 | 继续推进任务 A：新增 `crates/cide_cpp_frontend` 与 `crates/cide_typeck`。将 `vm::host_func_id`/`vm::bytecode_libc_index`/`vm::bytecode_libc_sig` 下沉到 `cide_runtime`；将 `native/src/compiler/cpp_frontend/` 完整迁移到 `crates/cide_cpp_frontend`；将 `native/src/compiler/typeck/` 全部 22 个文件完整迁移到 `crates/cide_typeck`；`TypeChecker` 新增 `pub fn new(is_library_mode: bool)` 构造函数以解决跨 crate 私有字段访问；`cide_native` 通过 `pub use cide_cpp_frontend as cpp_frontend;` / `pub use cide_typeck as typeck;` 保持路径兼容；`cargo test --all-features` / `cargo clippy --all-targets -- -D warnings` / `cargo fmt --check` / C Shadow Verification 555/562 / C++ Shadow Verification 83/83 全绿 | 维护 Agent |
| 2026-06-18 | 4.9 | 完成任务 A 关键节点：新增 `crates/cide_codegen`。将 `native/src/compiler/codegen/` 全部 26 个文件完整迁移到 `crates/cide_codegen`；将 `cide_vm::opcode`/`cide_vm::instruction` 及内存布局常量下沉到 `cide_runtime`，切断 `codegen` 对 `cide_vm` 的直接依赖；`cide_native` 通过 `pub use cide_codegen as codegen;` 保持路径兼容；修复迁移过程中 `cide_vm` 测试对 `Session` 的残留依赖，改为 `VmContext`；文档测试基线更新为 `cargo test --workspace --all-features` **750 passed**；`cargo clippy --workspace --all-targets --all-features -- -D warnings` / `cargo fmt --check` / C Shadow Verification 555/562 / C++ Shadow Verification 83/83 全绿 | 维护 Agent |
| 2026-06-18 | 4.10 | 继续推进任务 A 评估：`unified`/`engine`/`api` 三个模块因 FRB 孤儿规则与 `Session` 耦合暂无法独立成 crate；诚实记录该架构限制，并更新任务 G 中 K&R 第 8 章的评估结论 | 维护 Agent |
| 2026-06-19 | 4.11 | 继续推进任务 A：新增 `crates/cide_algorithm_steps`，将 `native/src/unified/algorithm_steps/` 完整迁移为独立 crate；通过 `AlgorithmContext` trait 解耦对 `Session` 的依赖；`cide_native` 通过 `pub use cide_algorithm_steps as algorithm_steps;` 保持路径兼容；`cargo test --workspace --all-features` / clippy / fmt / C Shadow Verification 555/562 / C++ Shadow Verification 83/83 全绿 | 维护 Agent |
| 2026-06-19 | 4.12 | 继续推进任务 A：将 `native/src/unified/checkpoint.rs` 的 `CheckpointManager` 迁移到 `cide_vm::snapshot`；重构 `should_checkpoint`/`save` 签名去除 `Session`/`StepMeta` 依赖；更新 `unified/engine.rs`、`flutter_bridge.rs`、`bin/cide_cli.rs`、`tests/test_snapshot.rs` 调用点；删除 `native/src/unified/checkpoint.rs`；验证全绿 | 维护 Agent |
| 2026-06-19 | 4.13 | 推进任务 F：完成后端 10 万步性能基线实测；新增 `UnifiedEngine::with_max_steps` 与 `cide_cli unified --max-steps` 选项；新增 `native/benches/unified_perf_baseline.c` 与 `scripts/unified_perf_baseline.py`，生成 `reports/unified_perf_baseline.md`；前端 55fps 回放基线仍待 Flutter 桌面端实测 | 维护 Agent |
| 2026-06-19 | 4.14 | 维护计划收尾：P1 完成前端 55fps 实测（Release 桌面端 40 元素冒泡排序回放等效 FPS 825.1，报告 `reports/frontend_fps_baseline.md`）；P2 在 CHANGELOG.md [Unreleased] 补录 Workspace 模块化拆分；P3 统一 unwrap/expect 口径为生产代码 17 处；P4 拆分 `native/src/diagnostics/error_catalog.rs`（968→455 行）为 `error_catalog/{lexer,parser,semantic,cpp}.rs`；P5 在 `STUDENT_ERROR_TEST_CASES.md` 新增 7.3 union 成员误读用例；P6 明确 `cide_engine`/`cide_api` 因 FRB 孤儿规则与 Session 耦合暂缓拆分；`cargo test --workspace --all-features` / `cargo clippy --workspace --all-targets --all-features -- -D warnings` / `cargo fmt --check` / `flutter test` / `flutter analyze` 全绿 | 维护 Agent |
| 2026-06-19 | 4.15 | 推进任务 B：拆分 `native/crates/cide_typeck/src/builtin.rs`（1034 → 336 行），新增 `builtin/memory.rs`、`builtin/string.rs`、`builtin/io.rs`、`builtin/file.rs`、`builtin/math.rs`；`visit_call` 分派与 `parse_format_specs`/`check_printf_format`/`check_scanf_format` 保留在入口文件；所有 builtin 检查方法提升为 `pub(crate)` 以支持跨模块 impl 调用；`cargo test --workspace --all-features`（750 passed）/ `cargo clippy --workspace --all-targets --all-features -- -D warnings` / `cargo fmt --check` / C Shadow Verification 555/562 / C++ Shadow Verification 83/83 全绿 | 维护 Agent |
| 2026-06-19 | 4.16 | 完成任务 B：拆分 `CideFlutter/lib/widgets/custom_keyboard.dart`（991 → 12 行入口），使用 Dart `part` 机制新增 `keyboard/layout.dart`（核心状态与 `build`）、`keyboard/letters.dart`（字母模式 extension）、`keyboard/numbers.dart`（数字模式 extension）、`keyboard/symbols.dart`（符号模式 extension）、`keyboard/gestures.dart`（`_KeyButton`/`_LetterKey`）；将 `_setSymbolCategory` 下沉到 `_CustomKeyboardState` 内部以符合 `setState` 的 `@protected` 规则；`flutter analyze` 0 issues / `flutter test` 221 passed / `cargo check --workspace --all-features` 全绿 | 维护 Agent |
| 2026-06-19 | 4.17 | 修复已知限制：`fputs(str, stdout)` 无输出。`crates/cide_vm/src/host/file.rs` 的 `host_fputs` 识别 lexer 预定义的 `stdout`(1)/`stderr`(2) 宏 fd，将字符串直接追加到 `RuntimeState.output_lines`；普通 `FILE*` 文件流写入行为不变；新增 `test_e2e_fputs_stdout` 回归用例；同步更新 `AGENTS.md` 已知限制与 `CHANGELOG.md [Unreleased]`；`cargo test --workspace --all-features`（751 passed）/ `cargo clippy --workspace --all-targets --all-features -- -D warnings` / `cargo fmt --check` / C Shadow Verification 555/562 / C++ Shadow Verification 83/83 全绿 | 维护 Agent |
| 2026-06-19 | 4.18 | 更新维护计划：第八章评估结论摘要刷新为当前状态（任务 B 已完成、`fputs(stdout)` 已修复）；任务 B 标记为 ✅ 已完成；新增任务 H"AGENTS.md 已知限制逐项修复"，建立 10 项限制跟踪清单；里程碑表格追加任务 H 条目；同步更新主要短板描述 | 维护 Agent |
| 2026-06-25 | 4.19 | 推进任务 H：修复函数返回 `double` 值异常（H07）。根因是 `return` 语句未对返回值表达式插入隐式类型转换；`cide_typeck::decl.rs` 在 `check_assignable` 成功后调用 `insert_implicit_cast`；新增 `baseline/float_func_return.c` 回归用例；`lc_4.c` 恢复为原始 `double` 返回实现；同步更新 `AGENTS.md`、`LEETCODE_FAILURES.md`、`CHANGELOG.md [Unreleased]`；C Shadow Verification 更新为 556/563，C++ Shadow Verification 保持 83/83；`cargo test --workspace --all-features` / `cargo clippy --workspace --all-targets --all-features -- -D warnings` / `cargo fmt --check` / `flutter analyze` / `flutter test` 全绿 | 维护 Agent |
| 2026-06-25 | 4.20 | 推进任务 H：修复 `scanf`/`sscanf` 的 `%s` 格式符不支持（H08）。`crates/cide_vm/src/host/io.rs` 的 `host_scanf_n`/`host_sscanf` 新增 `'s'` 分支处理：跳过前导空白、读取非空白字符序列、以 `'\0'` 结尾写入目标缓冲区；新增 `baseline/scanf_string.c` 回归用例；同步更新 `AGENTS.md`、`CHANGELOG.md [Unreleased]`；C Shadow Verification 更新为 557/564，C++ Shadow Verification 保持 83/83；`cargo test --workspace --all-features` / `cargo clippy --workspace --all-targets --all-features -- -D warnings` / `cargo fmt --check` / `flutter analyze` / `flutter test` 全绿 | 维护 Agent |
| 2026-06-25 | 4.21 | 推进任务 H：修复复合副作用数组索引触发 NULL 指针陷阱（H06）。根因是 `gen_mem_inc_dec` 与 `gen_assign` 的 Index 赋值复用 `temp_slot0`；修复方案为 `gen_mem_inc_dec` 改用 `temp_slot3` 保存新值；新增 `baseline/side_effect_index.c` 回归用例；同步更新 `AGENTS.md`、`LEETCODE_FAILURES.md`、`CHANGELOG.md [Unreleased]`；C Shadow Verification 更新为 558/565，C++ Shadow Verification 保持 83/83；`cargo test --workspace --all-features` / `cargo clippy --workspace --all-targets --all-features -- -D warnings` / `cargo fmt --check` / `flutter analyze` / `flutter test` 全绿 | 维护 Agent |
| 2026-06-25 | 4.22 | 更新维护计划：刷新 8.1 评估结论摘要中已知限制修复状态与主要短板描述；任务 H 清单更新为 P1 行为差异已清零；8.3 里程碑表格更新任务 H 状态；任务 H 新增本次推进记录小节 | 维护 Agent |
| 2026-06-25 | 4.23 | 推进任务 H：修复 `fclose` 后 VFS `FILE*` 被误报为内存泄漏（H10）。根因是 `host_fclose` 仅关闭 VFS 文件描述符，未释放 `host_fopen` 在 VM Heap 中为 `FILE*` 结构体分配的 4 字节内存；修复方案为新增 `MemoryState::free_region` 并在 `host_fclose` 中调用；stdout/stderr 等非堆分配 stream 安全忽略；新增 `baseline/fclose_leak.c` 回归用例；同步更新 `AGENTS.md`、`README.md`、`CHANGELOG.md [Unreleased]`；C Shadow Verification 更新为 559/566，C++ Shadow Verification 保持 83/83；`cargo test --workspace --all-features`（751 passed） / `cargo clippy --workspace --all-targets --all-features -- -D warnings` / `cargo fmt --check` / `flutter analyze` / `flutter test` 全绿 | 维护 Agent |
| 2026-06-25 | 4.24 | 推进任务 H：修复 VLA 数组索引缺失边界检查（H02）。`cide_codegen::expr::gen_index` 对首维为变量表达式的 VLA 生成运行时边界检查；新增 `TrapBoundsVla` opcode（值为 129）；VM 运行时将索引与运行时边界比较并触发教学诊断；新增 `baseline/vla_bounds.c` 回归用例；同步更新 `AGENTS.md`、`AGENTS_EN.md`、`README.md`、`CHANGELOG.md [Unreleased]`；C Shadow Verification 更新为 560/567，C++ Shadow Verification 保持 83/83；`cargo test --workspace --all-features`（751 passed） / `cargo clippy --workspace --all-targets --all-features -- -D warnings` / `cargo fmt --check` / `flutter analyze` / `flutter test` 全绿 | 维护 Agent |
| 2026-06-25 | 4.25 | 推进任务 H：修复参数化宏调用后带分号无法解析（H01）。`cide_lexer` 在参数化宏展开时，若宏体为大括号块且调用位置后紧跟分号，则动态将宏体包装为 `do { ... } while(0)`，使 `SWAP(int,x,y);` 在 `if/else` 等语句中可正确解析；新增 `end_to_end_extra_test::test_e2e_parametric_macro_swap_semicolon` 回归测试；诚实记录此为 Cide 教学子集扩展、Clang 标准模式仍报错的差异；同步更新 `AGENTS.md`、`AGENTS_EN.md`、`CHANGELOG.md [Unreleased]`；C Shadow Verification 保持 560/567，C++ Shadow Verification 保持 83/83；`cargo test --workspace --all-features`（752 passed） / `cargo clippy --workspace --all-targets --all-features -- -D warnings` / `cargo fmt --check` / `flutter analyze` / `flutter test` 全绿 | 维护 Agent |
| 2026-06-25 | 4.26 | 更新维护计划：基于 `scripts/engineering_health.py` 刷新 8.1 评估结论摘要中工程健康度数字（生产代码 unwrap/expect 17→18、全量 45→46）；确认任务 H 已处理 H01~H02、H06~H10；剩余 H03~H05 三项均为 P2/P3 不支持项，作为后续持续推进队列；文档状态与当前工作区保持一致 | 维护 Agent |
| 2026-06-25 | 4.27 | 推进任务 H：修复 `#include` 非标准库路径不支持（H03）。`cide_lexer::Lexer` 新增 `base_path` 字段与 `with_mode_and_path` 构造函数；`compile_pipeline.rs` 从首个编译单元文件名提取目录作为 `base_path`；`preprocessor.rs` 在 `load_stub` 失败后调用 `load_include_file` 从 `base_path` 读取自定义头文件；自定义头文件内容保留原始换行以支持 include guard 等预处理指令。`shadow_verify.py` 与 `cide_e2e.rs` 改用真实源文件路径调用 `cide_compile_unit`，保证 Shadow Verification 与 E2E 测试中的 include 行为与 Clang 一致。新增 `baseline/include_custom_header.c` / `include_custom_header.h` 回归用例；同步更新 `AGENTS.md`、`CHANGELOG.md [Unreleased]`；C Shadow Verification 更新为 564/568（match+cide_better+known_issue），C++ Shadow Verification 保持 83/83；`cargo test --workspace --all-features` / `cargo clippy --workspace --all-targets --all-features -- -D warnings` / `cargo fmt --check` / `flutter analyze` / `flutter test` 全绿 | 维护 Agent |
| 2026-06-25 | 4.28 | 推进任务 H：修复 `va_list` / `va_start` / `va_arg` / `va_end` 自定义变参函数（H04），并将全局 VLA（H05）明确为标准限制无需修复。根因是 Parser 把所有调用解析为 `Expr::CallPtr` 导致变参数据未写入栈帧；修复方案为 `cide_codegen::expr::call` 在 `gen_call` 与 `gen_call_ptr` 中同步生成 `CallVar`，按实参真实大小计算 `total_arg_words`，8 字节类型直接顺序压栈，`__cide_va_arg` 返回地址后按目标类型解引用；`cide_typeck::decl` 对变参额外实参应用默认实参提升；附带修复 `long long` 转换与复合赋值生成；新增 `baseline/variadic.c` 回归用例。H05 按 C99 标准禁止文件作用域 VLA，与 Clang 行为一致，保持不支持。同步更新 `AGENTS.md`、`CHANGELOG.md [Unreleased]` 与 `MAINTENANCE_PLAN.md`：H04 标记为 ✅ 已修复、H05 标记为 ✅ 无需修复（标准限制），任务 H 清单全部处理完毕；C++ Shadow Verification 保持 83/83；`cargo test --workspace --all-features` / `cargo clippy --workspace --all-targets --all-features -- -D warnings` / `cargo fmt --check` / `flutter analyze` / `flutter test` 全绿 | 维护 Agent |
| 2026-06-25 | 4.29 | 推进任务 C：收敛 H04 变参修复在 `cide_codegen::expr::call` 中引入的 2 处生产代码 `unwrap`。将 `gen_call` 与 `gen_call_ptr` 中的 `gen.func_table.get(name).unwrap()` 重构为 `if let Some(meta)`，避免重复查询并消除 unwrap；生产代码 `unwrap/expect` 从 20 降至 18，全量从 48 降至 46。同步刷新 `MAINTENANCE_PLAN.md` 8.1 摘要与 D01 口径为 566/569，与 `engineering_health.py` 对外口径一致。`cargo test --workspace --all-features` / `cargo clippy --workspace --all-targets --all-features -- -D warnings` / `cargo fmt --check` 全绿；C Shadow Verification 566/569，C++ Shadow Verification 83/83 | 维护 Agent |
| 2026-06-25 | 4.30 | 继续推进任务 C：收敛 `cide_typeck::cpp_class_layout` 中 1 处带 `TODO(#D08)` 的生产代码 `unwrap`。将刚 `insert` 后的 `self.classes.get_mut(name).unwrap()` 改为 `match self.classes.get_mut(name)`，在异常路径通过 `report_error` 返回结构化诊断（复用 `E3002_StructRedeclared`）而非 panic；保留后续对 `class_sym` 可变引用的自然使用。生产代码 `unwrap/expect` 从 18 降至 17，全量从 46 降至 45；Rust TODO/FIXME/HACK 从 12 降至 11。`cargo test --workspace --all-features` / `cargo clippy --workspace --all-targets --all-features -- -D warnings` / `cargo fmt --check` 全绿；C Shadow Verification 566/569，C++ Shadow Verification 83/83 | 维护 Agent |
| 2026-06-25 | 4.31 | 继续推进任务 C：收敛 `cide_codegen::lib.rs` 类大小拓扑计算中的 1 处生产代码 `unwrap`（`class_defs.get(class_name).unwrap()`），改为 `if let Some(class)` / `continue` 安全处理；收敛 `cide_codegen::expr::call.rs` 与 `expr::struct_.rs` 中结构体/类返回值临时偏移的 4 处 `unwrap`，通过 `if let Some(offset)` 复用 `Option` 值，删除冗余 `is_struct_ret` 变量与 `#[allow(clippy::unwrap_used)]` 注释。确认 `bTree_default` 运行时缺口为模板代码访问未初始化子节点指针的已知偏差（`E2E_FAILURES.md` 已记录），与本次 unwrap 收敛无关。生产代码 `unwrap/expect` 从 17 降至 0，全量从 45 降至 28；Rust TODO/FIXME/HACK 从 11 降至 10。`cargo test --workspace --all-features` / `cargo clippy --workspace --all-targets --all-features -- -D warnings` / `cargo fmt --check` 全绿；C Shadow Verification 566/569，C++ Shadow Verification 83/83 | 维护 Agent |

---

## 八、后续计划（2026-06-17 全面评估后）

> 本章节基于 2026-06-17 最新全面评估结果制定，承接 MAINTENANCE_PLAN 前五阶段已完成工作，聚焦长期可维护性与产品质量收尾。

### 8.1 评估结论摘要

当前项目处于 **Phase 42 收尾期 / 质量加固期**，核心数据：

| 维度 | 当前状态 |
|------|----------|
| C Shadow Verification | 566/569（99.5%，完全匹配 + cide_better + known_issue；剩余 2 运行时缺口 + 1 输出差异） |
| C++ Shadow Verification | 83/83（100%） |
| E2E 回归 | Baseline/K&R/LeetCode/C++ 全绿，Template 78/82（4 已知偏差） |
| 三层契约 | Host/Bytecode/Differential 全绿 |
| Fuzz | 5/5 通过 |
| Clippy | 0 warning |
| 生产代码 unwrap/expect | 0 处（全量 28 处，已区分生产/测试/生成代码；统计范围扩展至 `native/crates/`） |
| TODO/FIXME | Rust 10 处、Dart 6 处 |
| 活跃失败记录 | 14 条（口径已统一） |
| 任务 B 超大文件拆分 | ✅ 已完成：`cide_typeck/src/builtin.rs`、`CideFlutter/lib/widgets/custom_keyboard.dart` 已拆分 |
| 已知限制修复 | H01 参数化宏调用后带分号 ✅ 已修复（扩展支持，与 Clang 存在差异）；H02 VLA 边界检查缺失 ✅ 已修复；H03 `#include` 非标准库路径 ✅ 已修复（支持 `#include "header.h"` 基于源文件目录加载）；H04 `va_list` / `va_start` / `va_arg` / `va_end` 自定义变参函数 ✅ 已修复；H05 全局 VLA ✅ 明确为标准限制（C99 禁止文件作用域 VLA，Clang 同样报错）无需修复；H06 复合副作用数组索引 ✅ 已修复；H07 函数返回 `double` 异常 ✅ 已修复；H08 `scanf` `%s` 不支持 ✅ 已修复；H09 `fputs(str, stdout)` 无输出 ✅ 已修复；H10 `fclose` 后 VFS `FILE*` 泄漏误报 ✅ 已修复；AGENTS.md 已知限制清单全部处理完毕 |
| 任务 G 推进发现 | JIT 统计信息污染 stdout 已修复，新增 `cide_get_jit_stats` C API |

主要短板：**Workspace 拆分已进入平台期（`cide_shared`/`cide_ast`/`cide_runtime`/`cide_vm`/`cide_lexer`/`cide_parser`/`cide_cpp_frontend`/`cide_typeck`/`cide_codegen`/`cide_algorithm_steps` 已独立；`unified/checkpoint.rs` 的 `CheckpointManager` 已下沉到 `cide_vm::snapshot`；剩余 `unified/types.rs`/`stream.rs`/`root_cause.rs`、`engine/`、`api/` 因 FRB 孤儿规则与 `Session` 耦合暂保留在 `cide_native` 内部）、编译管线三阶段镜像耦合、多入口 API 重复包装、`cide_diagnostics` 因 FRB 孤儿规则暂无法独立成 crate；AGENTS.md 已知限制清单（H01~H10、H04/H05）已全部处理或明确记录原因；H01 参数化宏分号已作为教学子集扩展支持但与 Clang 存在差异，H03 自定义头文件 include 已支持，均已诚实记录**。

### 8.2 下阶段重点任务

#### 任务 A：Rust 后端 Workspace 拆分（P1，预计 3~4 周）

**目标**：将 `native` 单 crate 拆分为多个逻辑 crate，降低编译缓存粒度与模块耦合。

**建议 crate 划分**：

| crate | 职责 | 当前对应目录 | 状态 |
|-------|------|--------------|------|
| `cide_ast` | AST 定义与基础类型 | `crates/cide_ast/` | ✅ 已拆分 |
| `cide_lexer` | 词法分析 | `crates/cide_lexer/` | ✅ 已拆分 |
| `cide_parser` | 语法分析 | `crates/cide_parser/` | ✅ 已拆分 |
| `cide_typeck` | 类型检查 | `crates/cide_typeck/` | ✅ 已拆分 |
| `cide_codegen` | 字节码生成 | `crates/cide_codegen/` | ✅ 已拆分 |
| `cide_vm` | 虚拟机与 host 函数 | `crates/cide_vm/` | ✅ 已拆分 |
| `cide_algorithm_steps` | 算法步骤语义标注 | `crates/cide_algorithm_steps/` | ✅ 已拆分 |
| `cide_unified` | 统一模式 / 时间旅行 | `native/src/unified/` | ⚠️ 部分下沉（`algorithm_steps` 已独立，`CheckpointManager` 已下沉到 `cide_vm::snapshot`；`types.rs`/`stream.rs`/`root_cause.rs`/`engine.rs`/`collector.rs`/`trace_analyzer.rs` 因 FRB/Session 耦合暂保留） |
| `cide_diagnostics` | 诊断、知识图谱、自动修复 | `native/src/diagnostics/` | ⚠️ 因 FRB 孤儿规则暂保留在 `cide_native` 内部 |
| `cide_engine` | 编译管线编排 | `native/src/engine/` | ⏳ 待拆分 |
| `cide_api` | FRB / C API / CLI 入口 | `native/src/api/`、`native/src/capi/`、`native/src/bin/`、`native/src/flutter_bridge.rs` | ⏳ 待拆分 |

**执行步骤**：

1. ✅ 在 `native/Cargo.toml` 中建立 workspace，已迁移 `cide_shared`（SourceLoc）与 `cide_ast`（AST/类型系统）。
2. ✅ 拆分 `cide_runtime`：下沉 `func_meta`/`symbol`/`type_utils`、`RuntimeState`/`MemoryState` 及依赖类型、内存布局常量、`unified_types` 基础数据；`session.rs` 改为带 `#[frb]` 类型的薄包装层 + re-export。
3. ✅ 拆分 `cide_vm`：将 `native/src/vm/` 完整迁移到 `crates/cide_vm`；引入 `VmContext` 替代 `Session` 上帝对象，打破 `vm` 与 `session` 的循环依赖；`cide_native` 通过 `pub use cide_vm as vm;` 保持路径兼容。
4. ✅ 拆分 `cide_lexer`：将 `ErrorCode` 下沉到 `crates/cide_shared`；将 `native/src/compiler/lexer.rs` 与 `native/src/compiler/lexer/` 子模块完整迁移到 `crates/cide_lexer`；`cide_native` 通过 `pub use cide_lexer as lexer;` 保持路径兼容。
5. ✅ 拆分 `cide_parser`：将 `native/src/compiler/parser/` 全部 10 个文件完整迁移到 `crates/cide_parser`；`cide_shared` 新增 `SourceLoc` re-export；`cide_native` 通过 `pub use cide_parser as parser;` 保持路径兼容。
6. ✅ 拆分 `cide_cpp_frontend`：将 `native/src/compiler/cpp_frontend/` 完整迁移到 `crates/cide_cpp_frontend`；`cide_native` 通过 `pub use cide_cpp_frontend as cpp_frontend;` 保持路径兼容。
7. ✅ 拆分 `cide_typeck`：将 `vm::host_func_id`/`vm::bytecode_libc_index`/`vm::bytecode_libc_sig` 下沉到 `cide_runtime`；将 `native/src/compiler/typeck/` 全部 22 个文件完整迁移到 `crates/cide_typeck`；`cide_native` 通过 `pub use cide_typeck as typeck;` 保持路径兼容。
8. ✅ 拆分 `cide_codegen`：将 `native/src/compiler/codegen/` 全部 26 个文件完整迁移到 `crates/cide_codegen`；下沉 `cide_vm::opcode`/`cide_vm::instruction` 与内存布局常量到 `cide_runtime`，切断 `codegen` 对 `cide_vm` 的直接依赖。
9. ✅ 拆分 `cide_algorithm_steps`：将 `native/src/unified/algorithm_steps/` 完整迁移为独立 crate；通过 `AlgorithmContext` trait 解耦对 `Session` 的依赖；`cide_native` 通过 `pub use cide_algorithm_steps as algorithm_steps;` 保持 `crate::unified::algorithm_steps` 路径兼容。
10. ✅ 迁移 `CheckpointManager`：将 `native/src/unified/checkpoint.rs` 的 `CheckpointManager` 下沉到 `cide_vm::snapshot`；重构 `should_checkpoint` 接收 `semantic_label: &str`，`save` 接收 `&mut VmContext`，彻底去除对 `Session`/`StepMeta` 的依赖；更新 `unified/engine.rs`、`flutter_bridge.rs`、`bin/cide_cli.rs`、`tests/test_snapshot.rs` 调用点。
11. ⚠️ 评估 `unified`/`engine`/`api` 拆分受阻，Workspace 拆分进入平台期：
   - `unified/types.rs` 中 `StepPayload`/`AlgorithmStepSnapshot`/`PointerSnapshot` 等大量类型带 `#[frb]` 属性；若将 `unified` 拆为外部 crate，`cide_native` 为其实现 `IntoIntoDart` 会触发 Rust 孤儿规则（与 `cide_diagnostics` 同因）。
   - `engine` 模块（`compile_pipeline.rs`/`session_ops.rs`）依赖 `crate::session::Session`，而 `Session` 与 `CompileState` 均含 `#[frb]` 类型；直接拆分会形成 `cide_engine → cide_native → cide_engine` 的循环依赖。
   - `api` 模块是 FRB 入口，`api/cide.rs` 大量 `#[frb]` 函数与 `pub use` 直接定义在 `cide_native` 中；必须随 FRB 生成策略一起迁移。
   - 结论：`unified`/`engine`/`api` 暂保留在 `cide_native` 内部。后续若需拆分，需先建立独立的 `cide_session`（非 FRB 核心状态）与专门的 FRB 绑定层 crate，工程量大且风险高，不在当前阶段强行推进。
12. ⚠️ `cide_diagnostics` 与含 `#[frb]` 的类型暂保留在 `cide_native` 内部：若拆分为外部 crate 会触发 Rust 孤儿规则，待 FRB 跨 crate 绑定策略明确后再迁移。
13. 每迁移一个 crate，执行 `cargo test --workspace --all-features` 与 Shadow Verification 验证无回归。
14. 评估 `cide_engine` / `cide_api` 独立 crate（P6）：因二者深度依赖 `Session`/`CompileState` 等含 `#[frb]` 类型的状态对象，且 FRB 会在 `cide_native` 中为外部 crate 类型生成 `IntoIntoDart` 实现触发 Rust 孤儿规则，当前阶段不具备安全拆分条件。结论：暂缓拆分，待 FRB 跨 crate 绑定策略与 `cide_session` 非 FRB 核心状态 crate 建立后再推进。

**验收标准**：

- `cargo test --workspace --all-features` 全绿。
- C/C++ Shadow Verification 无新增失败。
- 单 crate 行数不超过 1.5 万行。
- `cargo check` 增量编译时间下降 ≥20%。

**风险**：
- Workspace 拆分会改变模块间可见性，需大量调整 `pub` 与 `use`；建议小步迁移，一次一个 crate。
- 含 `#[frb]` 导出类型的模块（如 `diagnostics`）若拆分为独立 crate，FRB 会在 `cide_native` 中为其生成 `IntoIntoDart` 实现，触发 Rust 孤儿规则。此类模块需留在 `cide_native` 内部，或设计专门的 FRB 绑定 crate。

---

#### 任务 B：超大单体文件继续拆分（P1，预计 2~3 周）✅ 已完成

**目标**：将剩余超过 1000 行的核心文件拆分为职责清晰的子模块。

**已完成拆分**：

| 文件 | 原行数 | 拆分后入口行数 | 拆分方向 |
|------|--------|----------------|----------|
| `crates/cide_lexer/src/lib.rs` | 1538 | 655 | 按 token 类别拆分为 `token.rs`、`number.rs`、`string.rs`、`comment.rs`、`keyword.rs`、`preprocessor.rs` |
| `native/src/compiler/ast.rs` | 1253 | 76 | 按 AST 节点大类拆分：`ast/expr.rs`、`ast/stmt.rs`、`ast/decl.rs`、`ast/types.rs` |
| `crates/cide_typeck/src/lib.rs` | 1326 | 485 | 提取 `context.rs`、`convert.rs`、`init.rs`、`symbols.rs` |
| `crates/cide_typeck/src/expr/mod.rs` | 1162 | 183 | 按表达式类型拆分：`expr/ops.rs`、`expr/literal.rs`、`expr/var.rs`、`expr/call.rs`、`expr/cast.rs`、`expr/cpp.rs` |
| `crates/cide_parser/src/expr/mod.rs` | 1170 | 50 | 按表达式优先级拆分：`expr/ops.rs`、`expr/unary.rs`、`expr/postfix.rs`、`expr/primary.rs` |
| `native/src/unified/algorithm_steps.rs` | 1439 | 170 | 按算法类别拆分：`algorithm_steps/sorting.rs`、`algorithm_steps/graph.rs`、`algorithm_steps/tree.rs`、`algorithm_steps/structures.rs`、`algorithm_steps/search.rs`、`algorithm_steps/math.rs`、`algorithm_steps/dp.rs` |
| `native/src/vm/core/executor.rs` | 1601 | 451 | 按指令大类拆分：`executor/arithmetic.rs`、`executor/memory.rs`、`executor/control.rs`、`executor/float.rs`、`executor/stack.rs`、`executor/debug.rs` |
| `native/src/compiler/algorithm_detector.rs` | 1145 | 44 | 按算法类别拆分：`algorithm_detector/features.rs`、`sorting.rs`、`graph.rs`、`tree.rs`、`search.rs`、`math.rs`、`string.rs`、`structures.rs`（仍位于 `native/src/compiler/algorithm_detector/`） |
| `crates/cide_vm/src/core/mod.rs` | 1221 | 30 | 提取 `core/state.rs`（667 行）、`core/memory.rs`（387 行）、`core/snapshot.rs`（169 行） |
| `crates/cide_codegen/src/lib.rs` | 1148 | 761 | 提取 `func.rs`（进入/退出函数）、`init.rs`（全局初始化扁平化、stride 计算）、`tests.rs`（单元测试）；后续随 crate 迁移路径改为 `crates/cide_codegen/src/lib.rs` |
| `crates/cide_codegen/src/stmt/mod.rs` | 884 | 85 | 按语句类型拆分为 `stmt/control.rs`（if/while/for/return 等）、`stmt/var_decl.rs`（变量声明与初始化）、`stmt/switch.rs`、`stmt/block.rs`、`stmt/expr_stmt.rs`、`stmt/cpp.rs` |

**剩余待拆分队列**（本次更新后）：

无。

**本次新增拆分**：

| 文件 | 原行数 | 拆分后入口行数 | 拆分方向 |
|------|--------|----------------|----------|
| `native/src/diagnostics/error_catalog.rs` | 968 | 455 | 按错误码范围拆分为 `error_catalog/lexer.rs`（1000~1999）、`parser.rs`（2000~2999）、`semantic.rs`（3000~3999）、`cpp.rs`（4000~4999）；`generate_fix` 保留在入口文件 |
| `native/crates/cide_typeck/src/builtin.rs` | 1034 | 336 | 按函数类别拆分为 `builtin/memory.rs`、`builtin/string.rs`、`builtin/io.rs`、`builtin/file.rs`、`builtin/math.rs`；`visit_call` 分派与格式字符串工具保留在入口文件 |
| `CideFlutter/lib/widgets/custom_keyboard.dart` | 991 | 12 | 使用 Dart `part` 机制拆分为 `keyboard/layout.dart`（核心状态与 `build`）、`keyboard/letters.dart`（字母模式 extension）、`keyboard/numbers.dart`（数字模式 extension）、`keyboard/symbols.dart`（符号模式 extension）、`keyboard/gestures.dart`（`_KeyButton`/`_LetterKey`）；符号分类状态变更下沉为 `_CustomKeyboardState._setSymbolCategory` 以符合 `@protected` 规则 |

**验收标准**：

- 每个源文件行数降至 800 行以内。
- 所有 Rust 测试通过。
- Shadow Verification 无新增失败。
- 不引入新的 `unwrap/expect`。

---

#### 任务 C：生产代码 unwrap/expect 收敛（P2，预计 2 周）

**目标**：将生产路径中的 43 处 `unwrap`/`expect` 逐步替换为显式错误处理，降低运行时 panic 风险。

**重点文件**：

- `native/src/compiler/cfg.rs`（9 处）
- `native/src/vm/core/executor/mod.rs` / `executor/stack.rs` / `executor/memory.rs`（合计 7 处）
- `native/src/api/cide.rs`（4 处）
- `native/src/compiler/data_flow.rs`（4 处）
- `crates/cide_codegen/src/expr/call.rs`（3 处）

**执行步骤**：

1. 对这些文件逐函数审计，区分"确实不可失败"与"可能失败"的调用点。
2. "确实不可失败"的调用添加注释说明不变量，保留 `expect` 并补充 `#[allow(clippy::expect_used)]`。
3. "可能失败"的调用改为 `match` / `if let` / `?` 传播，或转换为结构化诊断错误。
4. 生产代码 `unwrap/expect` 已降至 0 处，`unwrap_used` 已提升为 `deny`；统计口径以 `engineering_health.py` 排除 `#[cfg(test)]` / `#[test]` / `frb_generated.rs` 后的结果为准。

**验收标准**：

- ✅ 生产代码 `unwrap/expect` 已收敛至 0 处；`engineering_health.md` 与维护计划数字已对齐。
- ✅ `cargo clippy --all-targets -- -D warnings` 全绿。
- 新增错误路径均有单元测试覆盖。

---

#### 任务 D：失败记录文件口径整理（P2，预计 1 周）

**目标**：澄清"活跃失败记录"统计口径，区分历史已修复条目与当前已知失败。

**执行步骤**：

1. 统一 `engineering_health.py` 的统计口径：仅统计标记为 `KNOWN_FAILURE` / `KNOWN_DIVERGENCE` / `KNOWN_LIMITATION` 的条目。
2. 在 `KR_FAILURES.md` 顶部明确说明：当前 0 个活跃已知失败，文件主体为历史修复记录。
3. 修正 `CPP_FAILURES.md` 中"60 个 E2E 实际用例"的笔误，统一为 61 个。
4. 检查其他 `*_FAILURES.md` 是否也存在类似口径不一致，统一修正。

**验收标准**：

- `engineering_health.md` 中"活跃失败记录条目"数字与文件中 `KNOWN_*` 标记数一致。
- 所有 `*_FAILURES.md` 数字口径与 `TEST_REPORT.md` 一致。
- `scripts/ci_three_tier_check.py` 一致性检查通过。

---

#### 任务 E：CI 与构建系统加固（P2，预计 2~3 周）

**目标**：降低 CI 对 Windows-only runner 与 Flutter 工具 patch 的依赖，提升构建鲁棒性。

**执行步骤**：

1. **对齐本地与 CI 的测试/ lint 命令**：
   - `scripts/build_flutter.py --test` 改为执行 `cargo test --workspace --all-features`，确保所有 workspace member crates 的测试都被运行。
   - `scripts/build_flutter.py --test` 的 clippy 改为 `cargo clippy --workspace --all-targets --all-features -- -D warnings`。
   - CI Rust job 同步使用 `cargo test --workspace --all-features` 与 `cargo clippy --workspace --all-targets --all-features -- -D warnings`。
2. **Flutter generator 问题根治**：
   - 调研是否可通过 `flutter config --enable-windows-vulkan` 或环境变量避免 patch `build_windows.dart`。
   - 若必须 patch，将 patch 脚本化并加入版本控制，避免 CI 中内联 PowerShell 代码。
3. **Android job 增加基础测试**：
   - 构建完成后至少运行 `flutter test`（仅 Dart 层；默认按声明顺序执行，无需随机化种子参数）。
   - 有条件时增加 Android 模拟器 smoke 测试。
4. **依赖版本锁定**：
   - 对 `serde`、`serde_json`、`libm` 等主依赖增加小版本锁定（如 `1.0.x`），避免行为漂移。
5. **binaryen/wasm-opt 稳定性**：
   - 将 binaryen 版本与 wasm-opt 参数文档化。
   - 考虑将 wasm-opt 步骤改为可选，避免阻塞主 CI。

**验收标准**：

- CI 全绿且不依赖临时 patch。
- 本地 `python scripts/build_flutter.py --test` 与 CI 行为一致。
- Android job 至少运行 Dart 层测试。

---

#### 任务 F：性能收尾（P2，预计 2 周）

**目标**：完成 Phase 42 未完全落地的性能优化。

**执行步骤**：

1. **Dart Visualizer 缓存落地（已完成）**：
   - ✅ `array_visualizer.dart` 缓存 parsed numbers。
   - ✅ `tree_visualizer.dart`、`linked_list_visualizer.dart` 将 `TextPainter` 创建上提到 State 并复用，通过 `saveLayer` 应用动态透明度。
   - ✅ `RepaintBoundary` 已在动画组件就位。
   - ✅ `shouldRepaint` 已基于 `nodes`/`isDark`/`progress` 精确判定。
2. **统一模式差分编码落地（已完成）**：
   - ✅ 变量级差分（`var_deltas` / `new_vars` / `removed_var_name_indices`）已在位。
   - ✅ 符号表全局去重字符串池已在位。
   - ✅ Dart 端大 batch（>50 units）切到 isolate 解码已在位。
   - ✅ `call_stack` / `vis_events` / `accessed_vars` 改为 `Option<Vec<T>>`：无变化时不再全量传输。
   - ✅ `array_snapshots` / `pointer_snapshots` 实现按名索引的新增/替换/删除差分（`removed_*_name_indices`）。
   - 🚧 大对象（符号表、变量历史）分页或懒加载待进一步评估。
3. **性能基线（已完成后端实测，前端待实测）**：
   - ✅ 完成后端统一模式 10 万步基准实测：新增 `native/benches/unified_perf_baseline.c`（50 个逆序元素冒泡排序，约 10 万 VM 步）与 `scripts/unified_perf_baseline.py`，release 模式下后端吞吐约 **18,500 步/秒**，生成 `reports/unified_perf_baseline.md`。
   - ✅ 为支持长程序性能测试，`UnifiedEngine` 新增 `with_max_steps` 构造函数；`cide_cli unified` 新增 `--max-steps <n>` 选项（默认 100_000）。
   - ✅ 前端 55fps 回放基线已在完整 Flutter 桌面端 Release 环境中实测通过：40 个元素冒泡排序统一模式回放 5 秒，平均总帧时间 1211.9 μs，等效 FPS 825.1，满足 ≥55fps 基线；报告见 `reports/frontend_fps_baseline.md`。

**验收标准**：

- `flutter test` 与集成测试通过。
- 复杂可视化场景帧率 ≥55fps。
- 10 万步统一模式无明显卡顿与内存泄漏。

---

#### 任务 G：教学场景与内容深化（P3，持续推进）

**目标**：在工程加固基础上，扩展教学内容与真实场景覆盖。

**执行步骤**：

1. **LeetCode 逐步填充**：在 0~30 道中等题目标达成后，继续 all in 混合难度题目。✅ 2026-06-18 新增 15 道混合难度题（含 5 道困难题），当前 LeetCode 用例总数 92 道，均通过 Shadow Verification。
2. **K&R 进阶章覆盖**：
   - ✅ 第 7 章 7 个用例（kr_7_1~kr_7_7）已纳入，K&R 防线扩展至 76/76 全绿。
   - ⚠️ 第 8 章（UNIX 系统接口）因 Cide 教学子集不支持 POSIX 系统调用（`read`/`write`/`open`/`close`/`unlink`/`lseek`/`opendir` 等），原始习题无法直接适配。可适配部分（如 8-7 自己实现 malloc/free）与 Cide 内置内存管理冲突，且失去教学对比意义。结论：第 8 章**暂不纳入**常规回归测试，作为文档中的已知限制保留；后续可考虑编写使用 Cide VFS 接口的变体用例，但不强求覆盖。
3. **学生常见错误用例库扩展**：基于 `docs/current/STUDENT_ERROR_TEST_CASES.md` 持续补充。
4. **诊断知识卡片扩展**：针对 C++ 常见错误（内存泄漏、悬垂引用、对象切片、浅拷贝双重释放、引用绑定临时对象）新增知识卡片。

**本次推进记录（2026-06-18）**：

- 第一批新增 5 道 LeetCode 中等题源码与 golden：
  - `lc_3` Longest Substring Without Repeating Characters
  - `lc_33` Search in Rotated Sorted Array
  - `lc_48` Rotate Image
  - `lc_62` Unique Paths
  - `lc_64` Minimum Path Sum
- 第二批新增 5 道 LeetCode 中等题源码与 golden：
  - `lc_2` Add Two Numbers
  - `lc_11` Container With Most Water
  - `lc_19` Remove Nth Node From End of List
  - `lc_31` Next Permutation
  - `lc_34` Find First and Last Position of Element in Sorted Array
- 第三批新增 5 道 LeetCode 中等题源码与 golden：
  - `lc_15` 3Sum
  - `lc_39` Combination Sum
  - `lc_46` Permutations
  - `lc_75` Sort Colors
  - `lc_198` House Robber
- 第四批新增 5 道 LeetCode 中等题源码与 golden：
  - `lc_55` Jump Game
  - `lc_142` Linked List Cycle II
  - `lc_152` Maximum Product Subarray
  - `lc_200` Number of Islands
  - `lc_221` Maximal Square
- 第五批新增 5 道 LeetCode 中等题源码与 golden：
  - `lc_49` Group Anagrams
  - `lc_56` Merge Intervals
  - `lc_78` Subsets
  - `lc_102` Binary Tree Level Order Traversal
  - `lc_139` Word Break
- 第六批新增 5 道 LeetCode 中等题源码与 golden：
  - `lc_153` Find Minimum in Rotated Sorted Array
  - `lc_162` Find Peak Element
  - `lc_300` Longest Increasing Subsequence
  - `lc_394` Decode String
  - `lc_560` Subarray Sum Equals K
- 第七批新增 15 道 LeetCode 混合难度题源码与 golden（7 困难 / 7 中等 / 1 简单）：
  - `lc_4` Median of Two Sorted Arrays（困难；实现中发现 Cide 函数返回 `double` 值异常，已改用整数返回值通过）
  - `lc_23` Merge k Sorted Lists（困难）
  - `lc_25` Reverse Nodes in k-Group（困难）
  - `lc_42` Trapping Rain Water（困难）
  - `lc_45` Jump Game II（中等）
  - `lc_53` Maximum Subarray（简单）
  - `lc_73` Set Matrix Zeroes（中等）
  - `lc_76` Minimum Window Substring（困难）
  - `lc_84` Largest Rectangle in Histogram（困难）
  - `lc_91` Decode Ways（中等）
  - `lc_98` Validate Binary Search Tree（中等）
  - `lc_124` Binary Tree Maximum Path Sum（困难）
  - `lc_146` LRU Cache（中等）
  - `lc_207` Course Schedule（中等）
  - `lc_322` Coin Change（中等）
- 修复发现的问题：JIT 统计信息原通过 `output_lines` 混入用户 stdout，导致 `lc_3` 在 Shadow Verification 中出现 output_gap。已移除该输出并新增 `cide_get_jit_stats` C API，`jit_unit_test.rs` 改为通过 API 验证 JIT。
- 诚实记录的新发现：`lc_4` 原始实现使用 `double findMedianSortedArrays(...)` 返回值时，Cide VM 输出全为 `0.00000`，与 Clang 行为不一致。已改用整数返回值（结果 × 100000）规避，并在 `AGENTS.md` / `LEETCODE_FAILURES.md` 中记录该限制。
- C Shadow Verification 从 549/555 更新为 **555/562**，C++ Shadow Verification 保持 **83/83**。
- **2026-06-25 修复 `double` 函数返回值异常**：`cide_typeck::decl.rs` 在 `return` 语句 `check_assignable` 成功后调用 `insert_implicit_cast`；新增 `baseline/float_func_return.c` 回归用例；`lc_4.c` 恢复为原始 `double` 返回实现；C Shadow Verification 更新为 **556/563**。
- **新增 C++ 教学知识卡片**：在 `native/crates/cide_shared/src/error_codes.rs` 与 `native/src/diagnostics/error_catalog.rs` 中新增 E4100~E4106：
  - E4100：C++ 内存泄漏（new 未 delete）
  - E4101：C++ 悬垂引用（返回局部变量引用）
  - E4102：C++ 对象切片（派生类赋值给基类值对象）
  - E4103：C++ unique_ptr 所有权混乱
  - E4104：C++ move 后继续使用源对象
  - E4105：C++ 隐式浅拷贝导致双重释放
  - E4106：C++ 引用绑定到临时对象
- **扩展学生错误用例库**：在 `docs/current/STUDENT_ERROR_TEST_CASES.md` 中新增 9.1~9.7 七个 C++ 常见错误教学用例。

**验收标准**：

- ✅ 新增 30 道中等题及额外 15 道混合难度题均通过 Shadow Verification 与 `cargo test test_cide_e2e_leetcode`。
- ✅ 知识卡片覆盖 C++ Stage 0~6 常见错误（E4100~E4106）。

---

#### 任务 H：AGENTS.md 已知限制逐项修复（P2，持续推进）

**目标**：按优先级逐个修复 `AGENTS.md` 中记录的已知限制与行为差异，缩小 Cide 与 Clang 的行为差距，提升教学子集真实场景可用性。

**当前已知限制清单**（来自 `AGENTS.md`）：

| 编号 | 限制项 | 类型 | 优先级 | 状态 |
|------|--------|------|--------|------|
| H01 | 参数化宏调用后带分号无法解析 | 不支持（已扩展支持，与 Clang 有差异） | P2 | ✅ 已修复 |
| H02 | VLA 边界检查缺失 | 不支持 | P2 | ✅ 已修复 |
| H03 | `#include` 非标准库路径 | 不支持 | P3 | ✅ 已修复 |
| H04 | `va_list` / `va_start` / `va_arg` / `va_end` | 不支持（已扩展支持） | P3 | ✅ 已修复 |
| H05 | 全局 VLA | 标准限制 | P3 | ✅ 无需修复（Clang 同样报错） |
| H06 | 复合副作用数组索引触发 NULL 指针陷阱 | 行为差异 | P1 | ✅ 已修复 |
| H07 | 函数返回 `double` 值异常 | 行为差异 | P1 | ✅ 已修复 |
| H08 | `scanf` 的 `%s` 格式符不支持 | 行为差异 | P1 | ✅ 已修复 |
| H09 | `fputs(str, stdout)` 无输出 | 行为差异 | P1 | ✅ 已修复 |
| H10 | `fclose` 后 VFS `FILE*` 仍报告内存泄漏 | 行为差异 | P2 | ✅ 已修复 |

**执行步骤**：

1. 对每个限制项建立最小复现用例，确认根因范围（Lexer/Parser/TypeChecker/BytecodeGen/VM）。
2. 评估修复成本与教学价值，优先处理 P1 行为差异（影响 Shadow Verification 用例覆盖与真实代码迁移）。
3. 修复后补充回归用例，更新 `AGENTS.md` 与 `CHANGELOG.md [Unreleased]`。
4. 对暂无法修复的限制项，在 `AGENTS.md` 中保留诚实记录，并在本任务清单中标注原因。

**验收标准**：

- 每修复一项，C/C++ Shadow Verification 无新增失败。
- 新增回归用例通过 `cargo test --workspace --all-features`。
- `AGENTS.md` 与 `CHANGELOG.md` 同步更新。

**本次推进记录（2026-06-25）**：

- **H06 复合副作用数组索引触发 NULL 指针陷阱（P1）**：`crates/cide_codegen/src/expr/unary.rs` 的 `gen_mem_inc_dec` 改用 `temp_slot3` 保存新值，避免与 `gen_assign` 的 Index 赋值复用 `temp_slot0`；新增 `baseline/side_effect_index.c` 回归用例。
- **H07 函数返回 `double` 值异常（P1）**：`cide_typeck::decl.rs` 在 `return` 语句 `check_assignable` 成功后调用 `insert_implicit_cast`；新增 `baseline/float_func_return.c` 回归用例；`lc_4.c` 恢复为原始 `double` 返回实现。
- **H08 `scanf`/`sscanf` 的 `%s` 格式符不支持（P1）**：`crates/cide_vm/src/host/io.rs` 的 `host_scanf_n`/`host_sscanf` 新增 `'s'` 分支处理；新增 `baseline/scanf_string.c` 回归用例。
- **H01 参数化宏调用后带分号无法解析（P2）**：`cide_lexer::expand_macros_inner` 在参数化宏展开时，若宏体为大括号块且调用位置后紧跟分号，则动态将宏体包装为 `do { ... } while(0)`，使 `SWAP(int,x,y);` 在 `if/else` 等语句中可正确解析；新增 `end_to_end_extra_test::test_e2e_parametric_macro_swap_semicolon` 回归测试。诚实记录此为 Cide 教学子集扩展，Clang 标准模式对未包装的 `{ ... };` 形式仍会报错。
- **H02 VLA 数组索引缺失边界检查（P2）**：`cide_codegen::expr::gen_index` 对首维为变量表达式的 VLA 生成运行时边界检查；新增 `TrapBoundsVla` opcode（值为 129），索引前将 VLA 维度表达式求值并压栈；`cide_vm::core::executor::debug.rs` 在 VM 运行时将索引与运行时边界比较，越界时触发教学诊断；新增 `baseline/vla_bounds.c` 回归用例。参数退化为指针的 VLA 形参仍无法在调用点获知边界，保持跳过。
- **H10 `fclose` 后 VFS `FILE*` 被误报为内存泄漏（P2）**：新增 `cide_runtime::MemoryState::free_region` 方法；`crates/cide_vm/src/host/file.rs` 的 `host_fclose` 在关闭 VFS 文件描述符后调用 `session.memory.free_region(stream)`，释放 `host_fopen` 为 `FILE*` 结构体分配的 4 字节内存；stdout/stderr 等非堆分配 stream 找不到对应 region 时安全忽略；`vfs.rs` 的 `free_raw` 改为复用 `MemoryState::free_region`；新增 `baseline/fclose_leak.c` 回归用例。
- **H03 `#include` 非标准库路径（P3）**：`cide_lexer::Lexer` 新增 `base_path` 字段与 `with_mode_and_path` 构造函数；`compile_pipeline.rs` 从首个编译单元文件名提取目录作为 `base_path`；`preprocessor.rs` 在 `load_stub` 失败后调用 `load_include_file` 从 `base_path` 读取自定义头文件；自定义头文件内容保留原始换行以支持 `#ifndef` / `#define` / `#endif` 等预处理指令。`shadow_verify.py` 与 `cide_e2e.rs` 改用真实源文件路径调用 `cide_compile_unit`，保证 Shadow Verification 与 E2E 测试中的 include 行为与 Clang 一致。新增 `baseline/include_custom_header.c` / `include_custom_header.h` 回归用例。绝对路径、系统 include 搜索路径（`<>` 非标准库）及递归 include 仍待扩展。
- **H04 `va_list` / `va_start` / `va_arg` / `va_end` 自定义变参函数（P3）**：根因是 Parser 把所有函数调用解析为 `Expr::CallPtr`，而 `CallVar` 指令只在 `gen_call` 中生成，变参数据未被写入栈帧。修复方案：
  - `cide_codegen::expr::call` 在 `gen_call` 与 `gen_call_ptr` 中同步识别变参函数，生成 `PushConst total_arg_words` + `CallVar`；`total_arg_words` 按实参真实 `type_size` 折算 word 数，支持 `double` / `long long` 等 8 字节类型。
  - 调用方对 8 字节变参实参直接按小端布局顺序压栈，不再依赖 `SplitD/SplitQ`，`__cide_va_arg` 返回对应地址，宏展开为 `*(type*)__cide_va_arg(...)` 按目标类型位模式读取。
  - `cide_typeck::decl` 对超出命名参数的变参实参应用 C 默认实参提升（`float→double`、`char`/`short`→`int`）。
  - 附带修复：`cide_codegen::expr::cast` 重写 `gen_expr_with_cast`，以目标 `Type` 选择转换指令，修复 `long long` 目标被错误截断为 `int`；`cide_codegen::expr::assign` 修复复合赋值对 `long long` 使用 32 位运算的问题。
  - 新增 `baseline/variadic.c` 回归用例，覆盖 `int` / `double` / `long long` 三种类型的自定义变参求和；修复 `ast_unit_test.rs`、`b10_new_array_rollback.rs` 中因新增 `is_variadic` 字段导致的编译错误。
- **H05 全局 VLA（P3）**：按 C99 标准评估，文件作用域 VLA 被明确禁止（Clang 报 "variable length array declaration not allowed at file scope"）。Cide 保持不支持，与标准一致，无需修复；已在 `AGENTS.md` 中明确记录。
- P1/P2 行为差异与 H01~H05 已全部处理，H04 自定义变参函数已修复，H05 明确为标准限制无需修复；AGENTS.md 已知限制清单全部处理完毕。

---

### 8.3 执行节奏与里程碑

| 阶段 | 时间 | 里程碑 | 关键交付物 |
|------|------|--------|------------|
| 任务 A | 第 1~4 周 | ✅ Workspace 拆分进入平台期 | workspace 已建立；10 个核心 crate 已独立（含新增 `cide_algorithm_steps`）；`CheckpointManager` 已下沉到 `cide_vm::snapshot`；`unified`/`engine`/`api`/`diagnostics` 因 FRB 孤儿规则与 `Session` 耦合暂保留在 `cide_native` 内部；已诚实记录限制 |
| 任务 B | 第 3~5 周 | ✅ 超大文件拆分完成 | 核心文件均 <800 行、测试全绿 |
| 任务 C | 第 5~6 周 | ✅ unwrap/expect 收敛完成 | 生产路径 0 处、`unwrap_used` 已为 deny |
| 任务 D | 第 5 周 | ✅ 失败记录口径统一 | `engineering_health.md` 数字口径一致、文档修正 |
| 任务 E | 第 6~8 周 | ✅ CI 与构建加固完成 | 本地/CI 命令对齐、Flutter patch 脚本化、Android 增加 Dart 测试、依赖锁定 |
| 任务 F | 第 7~8 周 | ✅ 性能收尾完成 | Visualizer 缓存已完成；array/pointer/call_stack/accessed_vars/vis_events 全字段差分已完成；后端 10 万步基线已实测（约 18,500 步/秒）；前端 55fps 回放基线已在 Flutter 桌面端 Release 环境实测通过（FPS 825.1） |
| 任务 G | 第 9 周起持续 | ✅ 教学场景扩展推进 | LeetCode 92 道全绿；新增 K&R 第 7 章 7 个用例（76/76 全绿）；新增 C++ 知识卡片 E4100~E4106；扩展学生错误用例库至 9.1~9.7 |
| 任务 H | 持续推进 | ✅ AGENTS.md 已知限制清单全部处理完毕 | H01 参数化宏分号（扩展支持）、H02 VLA 边界检查、H03 自定义头文件 `#include "..."`、H04 `va_list` 变参函数、H05 全局 VLA（标准限制无需修复）、H06 复合副作用数组索引、H07 函数返回 `double`、H08 `scanf` `%s`、H09 `fputs(stdout)`、H10 `fclose` FILE* 泄漏误报均已处理；无剩余待修复项 |

### 8.4 质量保证

每项任务必须通过以下验证：

1. `cd native && cargo test --workspace --all-features` 全绿。
2. `cargo clippy --all-targets -- -D warnings` 全绿。
3. `cargo fmt --check` 通过。
4. C/C++ Shadow Verification 无新增失败。
5. `flutter analyze` 0 issues。
6. `flutter test` 全绿。
7. CI 全量 workflow 通过。

### 8.5 风险与回退策略

| 风险 | 回退策略 |
|------|----------|
| Workspace 拆分导致编译失败 | 小步迁移，每次只迁一个 crate；失败时回滚该 crate |
| FRB 导出类型跨 crate 触发孤儿规则 | 含 `#[frb]` 类型的模块暂保留在 `cide_native` 内部；待 FRB 绑定策略明确后再迁移 |
| unwrap 收敛引发大量错误路径变更 | 先 `warn` 后 `deny`，分阶段提升；必要时使用 `#[allow]` 并附注释 |
| CI patch 移除后 Flutter Windows 构建失败 | 保留脚本化 patch 作为备选，同时持续寻找根治方案 |
| 性能优化引入状态不一致 | 增加统一模式回归测试，对比全量状态与增量状态等价性 |
| 教学用例扩展暴露新差异 | 诚实记录为 `KNOWN_*`，禁止修改 golden 粉饰数据 |

---

> 本方案应与 `AGENTS.md`、`CHANGELOG.md`、`CODE_REVIEW_REPORT.md` 共同维护，任何新发现的工程债务应及时追加到本方案中。
