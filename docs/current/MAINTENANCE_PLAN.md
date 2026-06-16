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
| D01 | 文档数据口径不一致 | README.md / CHANGELOG.md / 审阅报告 | 已统一为 505/511（2026-06-16 实测） | 外部信任损耗 | P0 |
| D02 | FRB 生成文件仍被追踪 | `.gitignore` 已排除但文件可能仍被 git 追踪 | 174KB + 222KB 生成代码在版本库中 | 合并冲突、版本库体积 | P0 |
| D03 | `codegen/expr.rs` 过大 | `native/src/compiler/codegen/expr.rs`（2047 行） | 单函数超千行 | 维护与审查成本高 | P1 |
| D04 | `parser/mod.rs` 过大 | `native/src/compiler/parser/mod.rs`（2633 行） | 语法分析全堆在一个文件 | 维护与审查成本高 | P1 |
| D05 | `host_funcs.rs` 过大 | `native/src/vm/host_funcs.rs`（2545 行） | 96 个 host 函数集中 | 维护与审查成本高 | P1 |
| D06 | `ide_screen.dart` 过大 | `CideFlutter/lib/screens/ide_screen.dart`（896 行） | 承载整个 IDE 布局 | 前端维护困难 | P1 |
| D07 | 缺少项目级 clippy 配置 | 无 `clippy.toml` | 仅命令行控制 lint | 规则不一致 | P2 |
| D08 | 代码内 TODO/FIXME 标记极少 | 全项目 | 临时 workaround 无处追踪 | 技术债务隐形化 | P2 |
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

#### 任务 2.1：拆分 `native/src/compiler/codegen/expr.rs`

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
  3. 每拆分一个子模块，运行 `cargo test` 验证。
- **验收标准**：
  - `expr.rs` 行数降至 600 行以内。
  - 所有 Rust 测试通过。
  - Shadow Verification 无新增失败。
- **风险**：拆分时可能破坏 `self` 可变借用模式，需利用 Rust 借用检查器逐步调整。

#### 任务 2.2：拆分 `native/src/compiler/parser/mod.rs`

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

#### 任务 2.3：拆分 `native/src/vm/host_funcs.rs`

- **拆分策略**：
  - 保留 `host_funcs.rs` 为注册入口与公共工具函数。
  - 按功能拆分为子模块：
    - `vm/host/memory.rs`：`malloc`/`free`/`realloc`/`calloc`
    - `vm/host/string.rs`：`strlen`/`strcpy`/`strcmp`/`memcpy` 等
    - `vm/host/io.rs`：`printf`/`scanf`/`fprintf`/`fgets` 等
    - `vm/host/file.rs`：`fopen`/`fread`/`fwrite`/`fclose` 等
    - `vm/host/math.rs`：`sin`/`cos`/`sqrt`/`pow` 等
    - `vm/host/misc.rs`：`rand`/`srand`/`exit`/`qsort` 等
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
  - `native/src/compiler/parser/cpp.rs`（新建）
  - `native/src/compiler/parser/mod.rs`
- **执行步骤**：
  1. 将 C++ class、模板、构造析构、引用等语法解析逻辑集中到 `parser/cpp.rs`。
  2. 在 `parser/mod.rs` 中通过 `parse_cpp_xxx` 调用入口。
  3. 保持 C 解析路径不被 C++ 逻辑污染。
- **验收标准**：C 解析模块中不出现 `Class`、`Template`、`Reference` 等 C++ 专属分支。

#### 任务 4.2：建立 C++ 类型检查模块边界

- **涉及文件**：
  - `native/src/compiler/typeck/cpp/`
  - `native/src/compiler/typeck/mod.rs`
- **执行步骤**：
  1. 将 C++ 类布局、方法解析、重载、引用语义迁移到 `typeck/cpp/` 子模块。
  2. 明确 C++ typeck 与 C typeck 的调用边界。
  3. 提取公共工具函数到 `typeck/cpp/utils.rs`。
- **验收标准**：`typeck/mod.rs` 对 C++ 逻辑的依赖通过明确接口完成。

#### 任务 4.3：建立 C++ 字节码生成模块边界

- **涉及文件**：
  - `native/src/compiler/codegen/cpp/`
  - `native/src/compiler/codegen/expr.rs` / `stmt.rs`
- **执行步骤**：
  1. 将 C++ 构造析构调用、方法调用、引用处理、移动构造等逻辑迁移到 `codegen/cpp/`。
  2. 在通用 `gen_expr`/`gen_stmt` 中通过类型判断分派到 C++ 处理模块。
- **验收标准**：C++ 代码生成变更不再扩散到通用表达式生成逻辑。

#### 任务 4.4：C++ 容器布局维护流程固化

- **涉及文件**：
  - `native/runtime_libc/cide/*.cpp`
  - `scripts/extract_cpp_builtin_layout.py`
  - `native/src/compiler/cpp_frontend/builtin_layout_data.json`
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

1. **单元测试**：`cd native && cargo test` 全绿。
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
| D01 | 文档数据口径不一致 | 一 | ✅ 已完成 | 文档维护者 | 2026-06-16 实测统一为 505/511 |
| D02 | FRB 生成文件管理 | 一 | ✅ 已完成 | CI/构建维护者 | 提交 `ab39aaa` 已改为构建时生成；`.gitignore` 已配置 |
| D03 | `codegen/expr.rs` 过大 | 二 | ✅ 已完成 | 编译器维护者 | 2047 → 510 行；新增 8 个子模块 |
| D04 | `parser/mod.rs` 过大 | 二 | ✅ 已完成 | 编译器维护者 | 2633 → 672 行；新增 5 个子模块 |
| D05 | `host_funcs.rs` 过大 | 二 | ✅ 已完成 | VM 维护者 | 2545 → 155 行；新增 7 个子模块 |
| D06 | `ide_screen.dart` 过大 | 二 | ✅ 已完成 | 前端维护者 | 896 → 299 行；新增 6 个组件 |
| D07 | 缺少项目级 clippy 配置 | 三 | ✅ 已完成 | Rust 维护者 | 新增 `[lints.clippy]` + `clippy.toml` + `scripts/lint_check.sh` |
| D08 | TODO/FIXME 标记极少 | 三 | ✅ 已完成 | 全团队 | 新增 `docs/current/TODO_CONVENTION.md`，源码标记 30+ 处 |
| D09 | CustomPainter 缺少缓存 | 三 | ✅ 已完成 | 前端维护者 | TextPainter/Paint 缓存、shouldRepaint 精确化、RepaintBoundary 隔离 |
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

---

> 本方案应与 `AGENTS.md`、`CHANGELOG.md`、`CODE_REVIEW_REPORT.md` 共同维护，任何新发现的工程债务应及时追加到本方案中。
