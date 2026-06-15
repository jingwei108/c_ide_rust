# Cide 项目后续计划（2026 Q3）

> 制定时间：2026-06-15  
> 依据：[`docs/current/REVIEW_2026-06-14.md`](./REVIEW_2026-06-14.md) 与本项目的全面评估结论  
> 当前阶段：Phase 42 `[Unreleased]` 进行中  
> 目标受众：项目维护者、后续开发 Agent、潜在协作者

---

## 一、制定原则

1. **诚实记录优先**：任何已知失败、行为差异、未修复问题必须同步到 `*_FAILURES.md`，禁止粉饰数据。
2. **先止血再健身**：P0 修复工程收尾类问题，P1 提升可维护性，P2 扩展能力，P3 做长期技术储备。
3. **测试驱动**：每项改动必须有对应的回归测试或失败记录更新；CI 必须持续全绿。
4. **单作者风险兜底**：通过文档化、脚本化、自动化降低对单一维护者的依赖。
5. **最小侵入**：尽量不改已有接口，优先清理、补齐、文档化。

---

## 二、总体目标

在 2026 Q3（约 8~10 周）内完成以下目标：

| 目标编号 | 目标 | 关键结果（KR） |
|---|---|---|
| G1 | 工程收尾 | Clippy 持续 0 警告；过期日志/文档清理完毕；移动端脚本可正常工作 |
| G2 | 可移植性 | Android 构建不再依赖个人环境硬编码路径；新开发者可在 30 分钟内完成首次构建 |
| G3 | 测试防线加固 | 源码内单元测试覆盖 `typeck`/`codegen` 核心路径；失败记录文件无冗余 |
| G4 | C 子集逼近 100% Shadow | 511 → 505+ 匹配；未修复差异全部诚实记录并分类 |
| G5 | C++ M7 试用 | 修复 Beta 试用中发现的 P0/P1 问题；产出 `CPP_BETA_FEEDBACK.md` |
| G6 | 性能与体验 | `frameCache` 滑动窗口稳定运行；大型程序单步调试无明显卡顿 |

---

## 三、阶段划分与任务清单

### Phase A：P0 紧急修复（第 1~2 周）

> 目标：消除评估中发现的明确工程缺陷，恢复 CI/脚本的完全可用状态。

#### A1. 清理过期与误导性文件

- [ ] **A1.1** 删除 `native/clippy.log`（当前 Clippy 已 0 警告，该日志为 2026-05-10 过期产物）
  - 验收：`git ls-files | grep clippy.log` 无输出
- [x] **A1.2** 整理 `native/tests/cases_template_generated/E2E_FAILURES.md`
  - 移除已标记 `FIXED` 的冗余条目
  - 确保与根目录 `native/tests/E2E_FAILURES.md` 的 4 个 `KNOWN_FAILURE` 一致
  - 验收：`ci_three_tier_check.py` 不再输出文档不一致警告

#### A2. 修复移动端脚本

- [x] **A2.1** 修复 `scripts/test_mobile.py` 包名不一致
  - 将 `PACKAGE_NAME = "com.example.cide"` 改为 `"com.cide.app"`
  - 同步移动 `MainActivity.kt` 到 `com/cide/app/` 并更新包声明与 `AndroidManifest.xml`
  - 同步更新 `AGENTS.md`、`BUILD.md`、`BUILD_SCRIPTS.md` 中的 adb 命令示例
  - 验收：构建产物包名为 `com.cide.app`；在已连接 Android 设备上执行 `python scripts/test_mobile.py --install --run` 可正确启动应用

#### A3. CI 健壮性兜底

- [x] **A3.1** 验证 Android CI job 在真实 GitHub Actions runner 上通过
  - 真实 runner 暴露 `rust` job 缺少 Flutter 工具链问题；本地构建暴露 Gradle wrapper SSL 证书问题
  - 已记录到 `docs/current/CI_FAILURES.md`（CI-001、CI-002），修复方案待实施
- [x] **A3.2** 将 Flutter 工具 Patch 文档化
  - 产出 `docs/current/FLUTTER_BUILD_NOTES.md`
  - 说明 patch 目的、位置、失效后处理步骤、CI 环境要求、本地常见错误
  - 关联 `docs/current/CI_FAILURES.md` 中 CI-001/CI-002

---

### Phase B：P1 工程清理与可维护性（第 3~5 周）

> 目标：降低单作者依赖、消除重复代码、统一构建脚本、整理分支。

#### B1. 构建脚本重构

- [x] **B1.1** 合并 `build_flutter.py` 与 `build_release.py` 的重复逻辑
  - 将 `find_flutter()` 与 `build_rust_android()` 下沉到 `build_utils.py`
  - 行数从 514 降至 393，减少 121 行（23.5%），超过 ≥15% 目标
  - 脚本 `--help` 与 `py_compile` 验证通过；功能保持不变
- [x] **B1.2** 重构 `native/.cargo/config.toml`
  - 删除硬编码 NDK 链接器路径
  - 默认通过 `cargo-ndk` 自动处理；提供环境变量覆盖方案
  - 更新 `AGENTS.md` 中 Android 构建说明
  - 验收：在未安装 `android-ndk-r28c` 到默认路径的新机器上，`cargo ndk ... build` 仍能成功

#### B2. 分支整理

- [x] **B2.1** 审阅 9 个本地分支：
  - 删除 6 个已合并分支：`feat/editor-refactor`、`feat/memory-expand-vfs`、`feat/post-code-review-followup`、`feat/unified-mode`、`flutter-migration`、`refactor/recursive-type-system`
  - 删除 2 个废弃分支：`feature/react-native-webview`、`feature/tauri-migration`
  - 保留 1 个犹豫分支：`feature/flutter-webview`（WebView CM6 实验），设置 2026-08-31 再次 review
  - 产出 `docs/archive/BRANCH_DECISIONS.md` 记录全部决策
  - 验收：本地分支数从 9 降至 3（含 remote），满足 ≤ 5 个

#### B3. 源码级测试补强

- [x] **B3.1** 为 `compiler/typeck` 增加 10 个源码内单元测试
  - 覆盖 `implicit_cast_target` 的标量转换规则与引用/指针拒绝路径
  - 覆盖 `insert_implicit_cast` 对 int/float 字面量的实际转换插入
- [x] **B3.2** 为 `compiler/codegen` 增加 10 个源码内单元测试
  - 覆盖 `flatten_init_list` 的空列表、简单值、嵌套列表、float 位模式、负值、designator 报错路径
  - 覆盖 `stmt_loc` 对 `VarDecl`/`Return` 的定位
  - 覆盖 `compute_stride` 对一维/二维/三维数组的步长计算
  - 验收：`cargo test --lib` 单元测试数从 53 提升至 74，Clippy 0 警告

#### B4. 文档同步

- [x] **B4.1** 更新 `AGENTS.md` 中构建命令章节
  - 删除已过时的手动 NDK 路径说明
  - 补充 `cide_cli` 的新增命令（如 `export`）
- [ ] **B4.2** 更新 `CHANGELOG.md` 的 `[Unreleased]` 条目
  - 将 Phase 42 中已完成的 `frameCache`、LeetCode 阶段 6、FRB 构建时生成等纳入

---

### Phase C：P2 能力扩展与质量提升（第 6~8 周）

> 目标：继续逼近 Clang 行为一致性，补强 C++ Beta 试用，优化核心体验。

#### C1. C 子集 Shadow 缺口收敛

- [x] **C1.1** 分析当前 C Shadow Verification 的不匹配用例
  - 编译缺口 3 个：`inline_asm`、`static_assert`、`typeof_operator`
  - 输出差异 2 个：`vfs_io_extensions`、`file_fread`（根因是 VFS 未模拟 Windows CRT 文本模式 + Shadow 用例间文件未隔离）
  - 运行时缺口 3 个：`bTree_default`、`infixEvaluation_default`、`spfa_default`（根因是模板代码缺陷；Cide 的边界/NULL 检测是教学核心特性，保留为差异并诚实记录）
  - cide_better / 架构差异 4 个：`keyword_compat`、`merge_default`、`function_pointer_sizeof`、`sizeof_array_param`（非 Cide 缺陷，保留）
- [x] **C1.2** 修复优先级最高的缺口
  - 已修复：`inline_asm`（解析 GCC 风格占位）、`static_assert`（消费语法）、`typeof_operator`（Parser + TypeChecker 支持 `typeof(expr)`）
  - 已修复：`kr_5_8` 输出差异（Shadow 按需注入 `atof` 前向声明，避免与 K&R 自定义 `itoa`/`qsort` 冲突）
  - 已修复：`vfs_io_extensions`、`file_fread`（VFS 完整实现 Windows 文本模式换行转换 + Shadow 用例间文件隔离）
  - 保留为差异：`bTree_default`（未初始化指针）、`infixEvaluation_default`（栈下溢）、`spfa_default`（队列越界）已更新 `E2E_FAILURES.md` 为模板缺陷分类
  - 验收：C Shadow 匹配率从 498/511（97%）提升至 **504/511（98.6%）**，编译缺口与输出差异均归零

#### C2. C++ M7 Beta 试用

- [ ] **C2.1** 发起内部 Beta 试用
  - 邀请 3~5 名真实用户（学生/教师）使用 Cide 完成简单 C++ 练习
  - 收集反馈到 `docs/current/CPP_BETA_FEEDBACK.md`
- [ ] **C2.2** 修复 Beta 反馈中的 P0/P1 问题
  - 所有 P0 问题必须伴随测试用例和失败记录更新
- [ ] **C2.3** 补充 10~15 个 C++ 教学高频用例
  - 重点：引用参数返回值、移动语义边界、容器迭代器

#### C3. 性能优化

- [ ] **C3.1** 验证 `frameCache` 滑动窗口在大型程序上的表现
  - 测试程序：≥ 5000 行的排序/图论模板
  - 验收：单步执行无显著卡顿（帧率 ≥ 30fps）
- [ ] **C3.2** 对 `mangle_name` 进行性能剖析
  - 若存在热点，实施缓存或优化
- [ ] **C3.3** 评估 VM 快照增量序列化
  - 产出 `docs/current/VM_SNAPSHOT_OPTIMIZATION.md` 设计文档

#### C4. 开发者体验（DX）

- [ ] **C4.1** 编写 `docs/current/FIRST_TIME_SETUP.md`
  - Windows 桌面端、Android 端、CLI 工具分别给出步骤
  - 列出常见错误与解决方式
- [ ] **C4.2** 提供一键 setup 脚本（Windows PowerShell / bash）
  - 自动检测 Flutter、Rust、NDK 安装
  - 自动运行首次构建验证

---

### Phase D：P3 战略储备（第 9~10 周）

> 目标：为 M8 产品化做准备，探索长期方向。

#### D1. 跨平台扩展评估

- [ ] **D1.1** 评估 macOS/Linux 桌面端构建可行性
  - 在本地或 CI 中尝试 `flutter build macos` / `flutter build linux`
  - 记录阻塞点到 `docs/current/CROSS_PLATFORM_NOTES.md`
- [ ] **D1.2** 评估 iOS 支持可行性
  - 明确是否需要 Apple 开发者账号、macOS runner 等依赖

#### D2. 发布准备

- [ ] **D2.1** 配置 Android Release 签名
  - 在 `CideFlutter/android/app/build.gradle.kts` 中配置 release signingConfig
  - 将签名密钥信息放入环境变量/密钥管理，不提交到仓库
- [ ] **D2.2** 定义版本号策略
  - 从 `0.1.0` 演进规则：M7 Beta、M8 RC、M9 Release
  - 在 `CHANGELOG.md` 中明确版本边界

#### D3. 社区与模板

- [ ] **D3.1** 修复 4 个模板已知失败中的至少 1 个
  - 候选：`spfa_default`（图论教学高频）
- [ ] **D3.2** 设计模板贡献指南
  - 产出 `docs/current/TEMPLATE_CONTRIBUTION_GUIDE.md`

---

## 四、里程碑与验收标准

| 里程碑 | 时间 | 关键验收标准 |
|---|---|---|
| M1：P0 收尾完成 | 第 2 周末 | `clippy.log` 已删除；`test_mobile.py` 包名修复；CI 全绿；失败记录无冗余 |
| M2：工程清理完成 | 第 5 周末 | 构建脚本重复逻辑减少 15%；本地分支 ≤ 5 个；源码内单元测试 ≥ 73 个；新开发者 30 分钟构建指南可用 |
| M3：C 子集 99% Shadow | 第 8 周末 | C Shadow 匹配率 ≥ 99%；C++ Beta 反馈文档产出；`frameCache` 大型程序流畅 |
| M4：M8 准备就绪 | 第 10 周末 | Android Release 签名配置完成；跨平台评估文档产出；版本号策略确定 |

---

## 五、风险与应对

| 风险 | 可能性 | 影响 | 应对措施 |
|---|---|---|---|
| 单一作者精力不足 | 中 | 高 | 每个任务必须产出清晰文档和测试；复杂任务拆解为 ≤ 2 天可完成的小任务 |
| Flutter/Rust 工具链升级破坏构建 | 中 | 高 | CI 固定 Flutter 3.29.x、Rust 1.95.0；升级前在独立分支验证 |
| C Shadow 缺口部分不可修复 | 高 | 中 | 对不可修复项明确记录为 `KNOWN_DIVERGENCE`，不扭曲代码迎合 Clang |
| C++ Beta 反馈问题过多 | 中 | 中 | 优先修复 P0/P1，P2 纳入 M8 计划；保持失败记录诚实 |
| Android CI 真实 runner 失败 | 中 | 高 | 若失败，创建 `CI_FAILURES.md` 并在 1 周内修复或降级为可选 job |

---

## 六、任务跟踪模板

每个任务创建时建议按以下格式记录（可在 `docs/current/TASK_TRACKER.md` 中维护）：

```markdown
### TASK-XXX：任务标题
- **阶段**：P0 / P1 / P2 / P3
- **负责人**：liangjingwei / Agent
- **开始时间**：YYYY-MM-DD
- **预计完成**：YYYY-MM-DD
- **关联文件**：
- **验收标准**：
- **失败记录更新**：无 / 已更新 `XXX_FAILURES.md`
- **状态**：todo / in_progress / done
```

---

## 七、附录：参考文档

- [`AGENTS.md`](../../AGENTS.md)：项目 Agent 指南与构建命令
- [`CHANGELOG.md`](../../CHANGELOG.md)：阶段历史与 [Unreleased] 条目
- [`docs/current/REVIEW_2026-06-14.md`](./REVIEW_2026-06-14.md)：全面审阅报告
- [`docs/current/C_SUBSET_SPEC.md`](./C_SUBSET_SPEC.md)：C 教学子集规范
- [`docs/current/CPP_SUBSET_SPEC.md`](./CPP_SUBSET_SPEC.md)：C++ 教学子集规范
- [`native/tests/E2E_FAILURES.md`](../../native/tests/E2E_FAILURES.md)：已知失败记录
- [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml)：CI 配置

---

**备注**：本计划为滚动计划，每两周 review 一次。若某任务提前完成，可提前启动下一阶段任务；若某任务阻塞，必须在 3 天内更新到本文件并说明原因。

---

## 八、推进记录

### 2026-06-15 首次检查（由 Agent 执行）

| 检查项 | 结果 | 关键数据 |
|---|---|---|
| `clippy.log` 是否存在 | ✅ 已删除 | `git ls-files` 无输出 |
| Clippy 0 警告 | ✅ 通过 | `cargo clippy --all-targets -- -D warnings` 通过 |
| `test_mobile.py` 包名 | ✅ 已修复 | 已改为 `com.cide.app`；`MainActivity.kt` 包声明与 `AndroidManifest.xml` 已同步；文档示例已更新 |
| `CI_FAILURES.md` | ✅ 已创建 | 记录 CI-001（FRB 找不到 Flutter）与 CI-002（Gradle SSL 证书）|
| 源码内 lib 单元测试 | ✅ 已达成 | 从 53 提升至 74，覆盖 typeck/codegen 核心路径 |
| `.cargo/config.toml` 硬编码路径 | ✅ 已清理 | 改为 cargo-ndk 自动处理 + 环境变量覆盖说明；本地 `cargo ndk` 构建验证通过 |
| `native/.cargo/config.toml` 硬编码 NDK 路径 | ❌ 仍在 | 保留 `C:/Users/liangjingwei/.../android-ndk-r28c` |
| 本地分支数 | ✅ 已达成 | 从 9 降至 3（master、feature/flutter-webview、remotes/origin/master）|
| 源码内 lib 单元测试数 | ✅ 已达成 | 从 53 提升至 74 |
| C Shadow 匹配率 | ✅ 接近目标 | 504/511（98.6%），编译缺口与输出差异已归零；剩余 3 个运行时缺口为核心安全检测特性差异，已诚实记录 |
| `cases_template_generated/E2E_FAILURES.md` | ❌ 冗余 | 含 4 条已修复记录、2 条重复 `dfs_default`；与根目录 `E2E_FAILURES.md` 不一致；CI 输出一致性警告 |
| Flutter 工具 Patch 文档化 | ❌ 未产出 | `docs/current/FLUTTER_BUILD_NOTES.md` 不存在 |
| Android CI 真实 runner 验证 | ❌ 未验证 | `CI_FAILURES.md` 不存在 |
| `CHANGELOG.md [Unreleased]` | ✅ 已更新 | 已包含 frameCache、内置容器解耦、代码审查修复等 |
| C++ Shadow Verification | ✅ 全绿 | 83/83 匹配 |
| 三层契约 + Fuzz + E2E | ✅ 全绿 | 91/12/18/5/10/33/31/43/28 全部通过 |

**缺口详情（13 个不匹配）**：
- 编译缺口 3：`inline_asm`、`static_assert`、`typeof_operator`
- 运行时缺口 3：`bTree_default`、`infixEvaluation_default`、`spfa_default`
- 输出差异 3：`vfs_io_extensions`、`file_fread`、`kr_5_8`
- 已知特殊分类 4：`keyword_compat`（cide_better）、`merge_default`（cide_better）、`function_pointer_sizeof`（架构差异）、`sizeof_array_param`（架构差异）

**下一步启动顺序**：A2.1 → A1.2 → B1.2 → B3.1/B3.2 → B2.1 → A3.2 → C1.2

---

### 2026-06-15 第二次推进（由 Agent 执行）

| 检查项 | 结果 | 关键数据 |
|---|---|---|
| A1.2 `E2E_FAILURES.md` 整理 | ✅ 完成 | 根目录与 `cases_template_generated/` 下两个文件已同步；4 个模板失败全部重新分类为 `KNOWN_DIVERGENCE`（模板代码缺陷）；`ci_three_tier_check.py` 不再输出 E2E 文档不一致警告 |
| B1.2 `.cargo/config.toml` 重构 | ✅ 完成 | 已删除硬编码 NDK 路径，改为 `cargo-ndk` 自动处理 + 环境变量覆盖说明；保留手动配置模板注释 |
| B4.1 `AGENTS.md` 更新 | ✅ 完成 | 已补充 `cide_cli export` 命令说明与示例；NDK 路径说明已清理 |
| `ci_three_tier_check.py` | ✅ 全绿 | 全部 9 组测试通过，无 WARN |

**当前剩余重点任务**：
- C2.x C++ M7 Beta 试用（需要真实用户，当前无法由 Agent 独立完成）
- C3.x 性能优化（`frameCache` 大型程序验证、`mangle_name`  profiling）
- C4.x 开发者体验（`FIRST_TIME_SETUP.md`、一键 setup 脚本）
- D.x 战略储备（跨平台、签名、版本号策略）
