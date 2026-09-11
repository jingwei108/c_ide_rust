# 工作记录：Shadow 提速 + Phase 1 三项收口（2026-09-11）

> 范围：本轮四项工作 —— ①Shadow 验证提速（方案 A/B + release DLL 陈旧检测）②`cide_set_quarantine_budget` ③StepPayload schema v0.1 文档化 ④`cide_cli serve` JSON-lines 会话模式。
> 状态：**代码与验证全部完成**。~~未提交~~（记录时按用户要求「不提交」）；现已随 2026-09-11 后续提交合入（对应 [`CHANGELOG.md`](../../CHANGELOG.md) [Unreleased] 的 Shadow 提速 / Phase 1 出口 / schema / 隔离预算条目）。
> 关联文档：[`CHANGELOG.md`](../../CHANGELOG.md) [Unreleased]、[`docs/spec/STEP_PAYLOAD_SCHEMA_V0_1.md`](../spec/STEP_PAYLOAD_SCHEMA_V0_1.md)、[`CIDE_CLI.md`](CIDE_CLI.md) §6、[`CIDE_HEAP_QUARANTINE_DECISION.md`](CIDE_HEAP_QUARANTINE_DECISION.md) §6、主计划 §6 实现进度。

---

## 0. 结论速览

| # | 工作 | 交付 | 验证 |
|---|---|---|---|
| 1 | Shadow 提速 | `shadow_verify.py`：Clang 结果缓存 + `--jobs` 并行 + `--rebuild` 陈旧检测；CI 夜间全量重算 + 缓存 | 632 用例 ×4 次运行逐项一致；**103.6s → 1.1s（缓存命中）/ 20.4s（冷启动）** |
| 2 | 隔离预算 capi 出口 | `cide_set_quarantine_budget` / `cide_get_quarantine_budget` | 3 个集成测试（含"预算 0 → free 后立即复用"的行为证明） |
| 3 | StepPayload schema v0.1 | `docs/spec/STEP_PAYLOAD_SCHEMA_V0_1.md` + 字段冻结测试 | 5 个冻结测试（从 capi 出口 JSON 断言）；回放记录含对端待办 |
| 4 | `cide_cli serve` | `session_api.rs`（共享层）+ `cmd_serve` + CI 冒烟 | `scripts/serve_smoke.py` 26 项断言通过 |

**全局验证**：`cargo test --workspace` 全绿（exit 0）；`cargo clippy --workspace --all-targets --all-features -- -D warnings` 无警告；Shadow 门禁 0 非预期差异。

---

## 1. Shadow 提速

### 1.1 实现（方案 A + 方案 B + 顺手修）

| 项 | 实现位置 | 要点 |
|---|---|---|
| **A. Clang 结果缓存** | `shadow_verify.py`（`clang_cache_material/key/load/store`） | key = schema + platform + **clang 版本** + 编译/运行参数+超时 + 用例源码（文件用例读原文件，含同目录 `*.h` 内容哈希）+ stdin + VFS 预设文件哈希；目录 `native/tests/shadow_verification/.clang_cache/`；原子落盘（`.tmp` + `os.replace`）；损坏或 schema 不符 = 未命中 |
| `--refresh-clang` | `parse_args` + `run_case_task` | 无条件重算并覆盖写回；CI `schedule` 事件自动启用 |
| **B. multiprocessing.Pool 并行** | `execute_cases` / `_execute_cases_pool` / `worker_init` / `run_case_task` | `--jobs N`（默认 0 = `min(CPU,8)`，1 = 串行）；`imap_unordered(chunksize=1)`；结果按用例索引重排 |
| **回退路径（环境必需）** | `_execute_cases_shards` / `run_shard_worker` | 本机受限环境**禁止命名管道** → Pool 不可用（WinError 5）；自动回退到 **N 个 `python shadow_verify.py --shard-worker`** 子进程（round-robin 分片 + JSON 文件回传），`subprocess` 的匿名管道不受限 |
| `prepare_test_files` 迁移 | `worker_init` | 从"每用例写进程 cwd"改为"**每 worker 进程一次**，写进各自私有运行目录 `run_<pid>/`"；该目录同时作为 Clang 运行的 cwd（用例 fopen 写出的文件互不干扰） |
| **C. release DLL 陈旧检测** | `dll_staleness` / `warn_stale_dll` / `rebuild_release_dll` | 比对 `native/src`+`native/crates`+`Cargo.toml` 最新 mtime 与 DLL mtime（容差 2s）；陈旧 → 醒目警告；`--rebuild` → `cargo build --release` |
| 其他 | `_load_dll` | `ctypes.CDLL` + 全部签名设置从"每用例一次"改为"每进程一次" |
| 确定性修复 | `load_case_files` | glob 结果 `sorted(...)`（见 §1.4-3）；分片 payload 携带用例 **name** 并做对账 |
| 调试入口 | `--limit N` | 只跑前 N 个用例（标注"非完整门禁"，且不覆盖 latest 报告/K&R 报告，避免污染看板与 CI artifact） |

### 1.2 实测数据（2026-09-11，632 用例）

| 运行 | 命令 | 耗时 | 门禁结论 |
|---|---|---|---|
| ① 优化前基线（串行 + 全量重算） | `--jobs 1 --refresh-clang` | **103.6s** | match 613 / known_issue 3 / cide_better 16，**0 非预期差异** |
| ② 优化后（并行 + 缓存全命中） | `--jobs 8` | **1.1s**（632/632 命中） | 与①**逐项一致** |
| ③ 冷启动（并行 + 全量重算） | `--jobs 8 --refresh-clang` | **20.4s** | 与①**逐项一致** |
| ④ capi 重构后重建 DLL 再跑 | `--jobs 8 --rebuild` | 1~2s + 重建 14s | 与①**逐项一致** |

> 一致性口径：比较三次运行的 `(用例名, diff_type, expected_category)` 序列，`IDENTICAL`（脚本比对，非人工目测）。
> 证据文件（临时）：`tmp/shadow_pre_opt.json`（①）、`tmp/shadow_post_opt.json`（②）、`tmp/shadow_cold.json`（③）、`tmp/shadow_rebuilt.json`（④）。
> 结论：**提速没有改变任何一条判定** —— 门禁结论不变是硬要求，已机械核对。

日常预期：改动用例后只有被改用例 miss，其余命中 → 介于 1.1s 与 20.4s 之间；冷启动（首次或 clang 升级）20s 级。

### 1.3 CI 集成（方案 A 的"夜间全量重算"）

- `.github/workflows/ci.yml` 新增 `schedule: cron '0 18 * * *'`（UTC 18:00 = 北京 02:00）。
- 新增 `Clang version (for cache key)` + `actions/cache@v4`（path `.clang_cache`，key 含 `runner.os` + clang 版本串 + `shadow_verify.py` 哈希，`restore-keys` 允许同 clang 版本跨脚本改动复用 —— 缓存项内部还有 schema 校验兜底）。
- Shadow 步骤在 `schedule` 事件下自动追加 `--refresh-clang`（**夜间全量重算**，防 Golden 随 clang 漂移）。
- 防漂移是双闸：**cache key 含 clang 版本**（升级即失效）+ **夜间强制重算**（发现漂移即门禁失败）。

### 1.4 踩坑记录（诚实记录，全部为本轮实际发生）

1. **`tempfile` 在本机环境下不可用**：`tempfile.mkdtemp()`/`TemporaryDirectory` 内部用 `os.mkdir(p, 0o700)` 建目录，实测该目录**创建后不可写、不可删**（`PermissionError(13)`），shadow 在第一个用例即崩溃（0.5s）。
   定位过程：`os.mkdir(p)` 正常 / `os.mkdir(p, 0o700)` 失败 / `os.makedirs(p)` 正常。
   处置：全脚本弃用 `tempfile`，改自管 `.shadow_tmp/`（已加 `.gitignore`）。**这也顺带解决了并行目录隔离需求。**
2. **`multiprocessing.Pool` 在受限环境不可用**：Pool 的 `SimpleQueue` 基于**命名管道**，被拒（WinError 5）。`subprocess` 使用的匿名管道不受影响（Clang 调用一直正常）。
   处置：`_multiprocessing_pool_usable()` 探测（建一个空队列），失败自动回退分片 subprocess。**两条路径共用同一 `run_case_task` 与缓存层，语义一致。**
3. **glob 顺序导致用例错配（严重，已修）**：`Path.glob("*.c")` 顺序依赖底层 `scandir`，**跨进程不保证一致** → 分片 worker 按索引取到的是**另一个用例**，结果是"用 A 的结果标注 B 的名字"，同时缓存 key 漂移（一次运行写了 24 个缓存文件而只有 12 个用例）。
   处置：`sorted(...)` 固定顺序 + payload 传 name 并做索引↔名字对账（不一致直接 fail）。
4. **分片 worker 漏调 `worker_init`（严重，已修）**：重写 `run_shard_worker` 时丢失初始化调用 → `clang_version` 为空串（缓存 key 全部漂移，命中率 0）且**VFS 预设文件未写入**（文件 IO 用例的 Clang 侧行为失真）。
   处置：恢复调用并加注释说明"漏掉会同时导致缓存 key 漂移与文件 IO 用例缺预设文件"。
   > 这两个坑都不是"性能问题"，而是**并行化引入的静默正确性风险** —— 之所以被抓到，是因为每次都做了"逐项比对"而不是只看门禁是否绿。
5. **`--limit` 的副作用**：调试用截断运行会覆盖 `reports/shadow_report_latest.md`、`shadow_data_latest.json`、`kr_leetcode_report.json`（CI artifact）。
   处置：`--limit` 时跳过 latest 与专项报告的更新，并打印提示。
6. **0o700 目录残留（未清理，环境限制）**：探测阶段用 `tempfile` 建的两个目录 `tmp/probe_ws/`、`tmp/probe_ws2/` **删不掉** ——
   其 ACL 连 `takeown /F /R` 与 `icacls /grant` 都被拒绝（`Access is denied`），`shutil.rmtree` 亦失败。
   这是 §1.4-1 陷阱的实物证据：**该环境下 `os.mkdir(p, 0o700)` 产生的目录对本进程实际不可访问**（不可读写、不可删除）。
   残留不影响 git（不可读 → git 不跟踪）与后续运行；如环境权限放宽，可用管理员权限删除这两个目录。

---

## 2. 堆隔离预算的 capi 出口（决议 §1 完整性缺口）

- **缺口**：堆决议 §1 明确"隔离预算可调，写进会话配置"，引擎字段 `MemoryState::quarantine_budget` 已就绪，但 capi 没有 setter —— 外部（SharpTutor / serve）无法调整。
- **交付**：`native/src/capi/first_batch.rs`
  - `cide_set_quarantine_budget(session, bytes) -> c_int`：默认 256KB；`0` = 关闭隔离（free 后立即可复用，教学对照）；超大值**裁剪到堆上限 1MB**（语义安全：等价"整个堆都是隔离区"）；负值 → -1。
  - `cide_get_quarantine_budget(session) -> c_int`：读回（判分/回放场景需要确认两端配置一致）。
- **测试**（`native/tests/capi_first_batch_tests.rs`，该文件 15→18 用例）：
  - `test_set_quarantine_budget_controls_address_reuse`：同一程序 `p=malloc(4); free(p); q=malloc(4); printf("[%d]", p==q);` ——
    默认预算输出 `[0]`（隔离窗口内不复用），预算 0 输出 `[1]`（立即复用）。**用行为证明，而不是只断言字段值。**
  - `test_quarantine_budget_is_clamped_to_heap_limit`、`test_quarantine_budget_setter_rejects_null_session`。
- **文档同步**：堆决议 §6 验收清单新增该条；serve 的 `config.get/set` 暴露同一字段（`scripts/serve_smoke.py` 断言默认值 262144 与 set 生效）。

---

## 3. StepPayload Schema v0.1 文档化（Phase 1 验收项）

- **交付**：[`docs/spec/STEP_PAYLOAD_SCHEMA_V0_1.md`](../spec/STEP_PAYLOAD_SCHEMA_V0_1.md)（语言中立，任何语言可据此解析）
  - 顶层 14 字段语义表 + 真实字段示例；子结构 8 个（含可空性）；
  - **指针四状态枚举**（Valid/Freed/Null/Dangling）与**判定优先级**（Null → Dangling → Freed → Valid）；
  - **`accessed_vars.access_type` 读写枚举**（大小写敏感，同名可同时出现 Read/Write）；
  - `vis_events.ty` 编码（当前仅 `1 = compare`，如实记录）；
  - **frameCache 窗口语义**：2000 帧 / 丢弃最早 `ceil(len×0.2)`（至少 1）/ `cache_start_step` / `max_collected_step` /
    **越窗 seek = 最近检查点恢复 + 正向重放**（检查点间隔 20 步）/ seek 后窗口重置为 `[target-1999, target]` 并截断未来帧；
  - 差分编码 `StepStreamBatch`/`StepPayloadDelta`（**`null` = 未变 与 `[]` = 空 的区分是契约的一部分**）；
  - 出口形状（capi 三函数 + serve 帧）；版本化纪律（字段只增不改语义、消费方忽略未知字段）；
  - **回放场景校验记录**：我方 C1–C4 已实测（含命令）；对端 S1–S3（防抖编译流 / fixtures 判分流 / 单步+seek+内存查询交错流）**如实标注为"待 SharpTutor 执行"**；
  - §8 已知限制 7 条（`return_line` 恒 0、**无 mangled 名字段**（计划要求的 display/mangled 双字段尚未落地，需 v0.2）、`vis_events.ty` 单一、`ty_name` 为 Debug 表示等）。
- **可执行锚点**：`native/tests/step_payload_schema_v0_1_test.rs`（5 用例）——从 **capi 出口 JSON** 冻结
  顶层键集合 / 子结构键集合 / 枚举字面量 / 窗口 2000 帧上限与 `cache_start_step` 前移。
  schema 一旦被无意改动即测试失败，强制走版本化流程（而不是让文档悄悄过期）。

---

## 4. `cide_cli serve` JSON-lines 会话模式（Phase 1 出口 3）

- **分层**：新增 `native/src/session_api.rs`（语言中立会话层）；capi 第一批的 6 处 JSON 结果构造全部下沉，
  `capi` 与 `serve` **共用同一实现**（纪律 §2.2-2 的实质落地，而非"看起来共享"）。capi 出口 JSON 形状不变（18 个测试全绿佐证）。
- **协议**：NDJSON；**id 关联**；**错误帧与成功帧同构**（`{"id","ok","result"}` / `{"id","ok","error":{kind,message}}`，`kind ∈ protocol|state|internal`）；
  `session.reset` 保留会话级配置（隔离预算、deterministic、argv）而清空编译/运行状态。
- **方法**：`ping` / `compile`（覆盖式）/ `run` / `output.delta` / `step.begin` / `step.next` / `payload.get` /
  `seek` / `breakpoints.set` / `memory.regions` / `config.get|set` / `session.create|reset|destroy` / `shutdown`。
- **防线**：`scripts/serve_smoke.py`（26 项断言）已进 CI；文档 `CIDE_CLI.md` §6（示例输出为**实跑粘贴**，非手写）。
- **边界说明**：`memory.regions` 为**过渡形态**（现有字段 + 隔离区统计），capi 第二批的 `kind` 三段式/字节读取仍未做（本轮明确不做）。

---

## 5. 本轮验证矩阵

| 项 | 命令 | 结果 |
|---|---|---|
| Rust 全量测试 | `cargo test --workspace` | ✅ 全绿（exit 0） |
| capi 第一批 | `cargo test --test capi_first_batch_tests` | ✅ 18 passed（15 + 新增 3） |
| schema 冻结 | `cargo test --test step_payload_schema_v0_1_test` | ✅ 5 passed |
| 静态检查 | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | ✅ 无警告 |
| Shadow 门禁（4 次） | 见 §1.2 | ✅ 0 非预期差异，且四次逐项一致 |
| serve 协议 | `python scripts/serve_smoke.py` | ✅ 26/26 断言通过 |
| DLL 陈旧检测 | 改源码后直接跑 shadow（不重建） | ✅ 正确报出"DLL 17:23:02 vs 源码 17:35:16"并给修复指引 |
| `--rebuild` | `python ... --rebuild` | ✅ 自动重建，重建后门禁结论与基线逐项一致 |

---

## 6. 未做 / 遗留（本轮范围之外）

- **属 Phase 2a/2b/3**：wasm 绑定包、capi 第二批内存 API（`kind` 三段式 / 字节读取）、错误码表机器可读导出、时间旅行 CoW —— 均未动。
- **依赖对端**：SharpTutor 15 用例冒烟集；schema 回放场景 **S1/S2/S3**（本轮只完成我方 C1–C4）。
- **Shadow 相关遗留**：
  - `E-P1-5`（stdout 清洗规则三处重复：`shadow_verify.py` / `shadow_verify_cpp.py` / `cide_e2e.rs`）**仍未收敛**。
    本轮 serve 的输出走 VM 原始通道 + 游标增量（**没有引入第四份清洗规则**），但三处重复本身未动 —— 属独立重构项。
    - ✅ **已收敛（2026-09-11，E-P1-5 修复轮次）**：实测不止"三处"而是**十余处**（另含 `bytecode_libc_consistency.rs`、`test_utils.rs`、
      `bytecode_gen_cpp_unit_test.rs`、`end_to_end_extra_test.rs`、`qsort_test.rs`、`test_more.py`、`test_massive.py`）。根因不在正则而在引擎输出通道：
      `RuntimeState::output_lines` 同时承载程序 stdout / stderr / 引擎附注，且附注可在流中间插入。现按 `OutputKind{Stdout,Stderr,Note}` 分离，
      capi 新增 `cide_get_program_output*` / `cide_get_engine_notes*` / `cide_get_program_output_delta`（**ABI 1.1.0**），
      十余处清洗规则全部删除并共用 `cide_output.py` 单一读取入口。详见 `MAINTENANCE_PLAN.md` 维护日志 4.41。
  - `shadow_verify_cpp.py` 仍使用 `tempfile`（同样会在受限环境失败），本轮只修了 C shadow；如需同等待遇需另开一轮。
    - ✅ **已修复（2026-09-11）**：改用自管工作目录 `.shadow_cpp_tmp/`（已 gitignore），C++ 防线在受限环境下恢复可跑
      （实测 100 用例 / 98 一致 / 2 个已记录 `clang_compile_fail` / 0 非预期差异）。
- **known_issue 数字与 AGENTS.md 的出入（如实记录）**：本轮实测 `known_issue = 3`（`function_pointer_sizeof` / `sizeof_array_param` / `spfa_default`），
  而 `AGENTS.md` 防线 1 描述为 4（2 个存量 bug 分类 + 2 个模板已知偏差）—— `bTree_default` 当前判定为 `match`（不再命中 `KNOWN_FAILURE_CASES` 的失败路径）。
  门禁不因此失败（0 非预期差异），但文档口径需要一次对齐（未在本轮改动 `AGENTS.md` 的该段统计）。
- **schema v0.2 待办**：`func_display_name` + `func_mangled_name` 双字段、`vis_events.ty` 扩充、`return_line` 补全（见 spec §8）。

---

## 7. 复现指南

```bash
# 1) 构建 release DLL（shadow 用）
cd native && cargo build --release && cd ..

# 2) Shadow：日常（并行 + 缓存）
python native/tests/shadow_verification/shadow_verify.py --jobs 8

# 3) Shadow：冷启动 / 夜间全量重算
python native/tests/shadow_verification/shadow_verify.py --jobs 8 --refresh-clang

# 4) Shadow：DLL 陈旧时自动重建
python native/tests/shadow_verification/shadow_verify.py --rebuild

# 5) 本轮的定向验证
cargo test --manifest-path native/Cargo.toml --test capi_first_batch_tests
cargo test --manifest-path native/Cargo.toml --test step_payload_schema_v0_1_test
python scripts/serve_smoke.py      # 需先 cargo build --bin cide_cli
```
