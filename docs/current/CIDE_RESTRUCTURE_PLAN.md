# 结构重构决议与执行计划（2026-09-11）

> **决策背景**：前端切割后仓库表面积最小、无外部借用（SharpTutor 未接入、社区前端未认领）——
> 这是改内部结构零外部成本的唯一窗口。fix 历史的解剖显示脉冲式清偿（6-16 一波 14 个、
> 9 月两波 6 个）间隔在缩短，且每波根因高度集中于同一结构病：**同一概念存在多个真相来源**。
> 与其等下一波脉冲，不如按手术清单一次收口。
>
> **基线锚点**：标签 `pre-restructure`。Shadow 636 用例 0 非预期差异 / cargo test 全绿 /
> clippy 零警告 / serve 冒烟 26/26 —— **重构期间全程在线**，任何一批导致行为漂移即回滚。

## 1. 换什么、不换什么（边界声明）

**换（结构）**：
- 会话模型：全局单例（flutter_bridge 799 行，u64 map + static）→ `Session` + `session_api` 单轨
- 内存协调：全局区/堆区静态边界（HEAP_START 写死）→ 堆起点动态化
- 语义真相来源：已收口的（输出/stdin/行号/配置）保持；未收口的（教学标注双来源）单源化
- 模块边界：>800 行文件拆分、host IO 家族收敛（fmt 解析单源）

**不换（行为资产）**：
- 编译管线五段结构（Lexer → Parser → TypeChecker → CodeGen → VM）
- 字节码格式与 636 个 Shadow golden（Clang 对照的行为锚）
- 出口协议（capi ABI 1.1.0 / serve 协议 / StepPayload schema v0.1）
- VM 1MB 线性内存模型与教学检测语义（UAF/Double-Free/泄漏报告）

**明确不做**：推倒重写编译器/VM。教学引擎的价值锚在行为正确性，重写等于重新欠一遍
行为债，且无第二套防线能兜住重写期间的行为漂移。

## 2. 批次计划（每批独立提交、独立过全防线）

| 批次 | 内容 | 验收线 |
|------|------|--------|
| **R1 内存边界** | ① 堆起点动态化：`heap_offset = max(HEAP_START, align4(global_end))`——栈碰撞检查已确认用动态 `heap_offset`（`control.rs:69`），上移自动跟随；② **判据单源化**：`literal.rs`（`MEM_SIZE/16`=64KB）与 `state.rs`（`HEAP_START`=20KB）两套魔数统一到 `GLOBAL_REGION_LIMIT` 单一常量；③ **argv 编址修复**（评审后新发现）：`global_count` 恒为 0，argv 数组固定从 `GLOBAL_START` 写入，与 codegen 全局数据重叠编址（预存 bug）——改为自全局区上界**向下**分配；④ 影响面 6 个逻辑点：`session_ops.rs:68`（重置）、`memory_state.rs:104/241/252`（初始化/统计口径）、`state.rs:266`（argv）、`compile_pipeline.rs`（注入点）；⑤ `global_end > HEAP_START` 时编译 warning 提示堆可用空间 | 三方挤压用例（大全局 + malloc 失败明确 trap + 深递归栈溢出明确 trap）；大全局 + malloc 数据不损坏；lc_22/lc_977 回归；Shadow 0 非预期差异 |
| **R2 会话收口** | cide_cli 四个子命令（compile/run/step/unified）迁 `Session` + `session_api`（serve 同款）；flutter_bridge 零消费后整删；3 处过时注释更新 | CLI 行为不变（冒烟对照）；G6 闭环；全局 static 清零 |
| **R3 语义单源审计** | 以 E-P1-5 模式扫全库枚举残留「多真相」点；教学标注单源化（collector 标签与 algorithm_steps 描述共用一次推断）；审计报告进 docs | 审计清单归档；标注矛盾类缺陷结构性消除 |
| **R4 债务与防线** | decl.rs 拆分（892 行 → 子模块）；unwrap×3 收敛；engineering_health 翻新进 CI（G11）；**G1** 模板生成器恢复（剥离 Flutter 半边）；**G10 + G12 数字自动对账**（C++ 侧 74vs78 与 C 侧三套口径——AGENTS 78绿/4失败、E2E_FAILURES 79/3、代码常量 2——对账须覆盖「文档声明数 vs 代码常量数」方向，现有 three_tier 只查单方向）；**G13** 两条 C++ 活约束（模板类跨文件重复定义、`T()` 值初始化）补入 CPP_SUBSET_SPEC；**G2** wasm 冒烟进 CI | CI 新增检查全绿；<800 行规约恢复 |

### G9 独立决策（能力缺口，不属结构债，不混入 R1-R4）

`validate_algorithm()` / `ValidationResult` 的"运行时算法属性验证"随前端切割**整体消失**
（后端从未有过，原载体 `algorithm_validation.dart` 已迁出）。这不是重构能顺带解决的——
需要显式选择：

- **方案 A（默认采纳）**：列入 Phase 2a 出口演进批次——`cide_algorithm_steps` 已有 41 个
  模板元数据可作判定基础，落点 `validation.rs`（语言中立层），由 serve/capi 按需暴露；
  由 SharpTutor 的实际诉求触发排期，避免做无人消费的能力。
- **方案 B（若放弃）**：ROADMAP.md 的 G9 改标「能力已随前端迁出，待社区前端认领」，
  并从 Phase 2a 计划中移除对应条目。

二选一之前 G9 保持「待评估」状态；**不允许**默认沉默地丢掉教学能力。

顺序有依赖：R1 先修唯一正确性问题；R2 让 CLI 成为 session_api 第三活体测试，为 R3 提供三出口互证。

## 3. 升级为更大重构的判据（诚实记录）

R3 审计后若仍出现新的「多真相」类缺陷、或发生「修 A 坏 B」横切回归 ≥2 次、或
known_issue 趋势向上——重新评估结构性重写（届时 R3 的病灶地图即重写图纸）。

## 4. 已知风险与回滚

- R1 触碰 `MemoryState` 初始化时机（快照/时间旅行兼容）与泄漏报告的 `HEAP_START` 假设（约 5 处引用）
- R2 触碰 CLI 全部子命令；Shadow 驱动走 capi 不受影响
- 回滚单位 = 批次提交；`pre-restructure` 标签保底整轮回退
