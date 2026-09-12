# StepPayload Schema v0.1（语言中立）

> 状态：**v0.1 定稿候选**（Phase 1 验收项；待 SharpTutor 三组回放场景签字后冻结为 v0.1）
> 日期：2026-09-11
> 归属：主计划 [`CIDE_BACKEND_SPLIT_WASM_WHITEBOX_PLAN.md`](../current/CIDE_BACKEND_SPLIT_WASM_WHITEBOX_PLAN.md) §3.2 协议层
> 实现锚点：`native/src/unified/types.rs`（类型定义）、`native/src/unified/collector.rs`（字段来源）、`native/src/unified/engine.rs`（窗口与 seek）、`native/src/unified/stream.rs`（差分编码）、`native/src/capi/first_batch.rs`（出口序列化）
> 消费者：capi 第一批（`cide_step_next_json` / `cide_get_step_payloads_json`）、`cide_cli serve`、wasm 绑定、任何第三方语言
> 最后核对日期：2026-09-11
> 修订说明（2026-09-11）：去前端化——§0 明示"语言中立、不依赖任何前端实现"；§7 回放输入的"Flutter frameCache"改为"原生前端 frameCache 消费序列（已切割的历史资产）"并指向 `cide_cli serve` 复现口径。字段定义与校验记录保持原样。

本文档是**协议层**定义：任何语言按此即可解析 Cide 的步数据，无需了解 Rust 内部表示。引擎内部优化（CoW 快照等）不得改变本文档的字段语义。

---

## 0. 文档约定

| 约定 | 说明 |
|---|---|
| 传输编码 | UTF-8 JSON。capi 每个入口返回**完整 JSON 字符串**（rust-alloc 所有权，须用 `cide_free_string` 释放）；`cide_cli serve` 为 NDJSON（每行一个 JSON 对象） |
| 字段命名 | `snake_case`（与 Rust serde 默认一致） |
| 枚举取值 | **字符串字面量**，大小写敏感（如 `"Valid"` / `"Read"`） |
| 步号 | `step_index` 从 **0** 开始，一步 = VM 执行一条字节码指令（含透明的 `StepEvent` 调试指令） |
| 行号 | `code_line` 为 **1 起**源码行号；`0` 表示"当前步无对应源码行"（如库函数内部） |
| 地址 | 1MB 线性内存空间内的 `u32`（`0x1000` 起为全局区、`0x5000` 起为堆区、栈区自高地址向下） |
| 可空 | JSON `null`；消费方须同时容忍**字段缺省**（只增不改前提下新字段可能不出现在旧数据里） |
| 版本化纪律 | **字段只增不改语义**；新增字段必须可空/有默认值；消费方必须忽略未知字段；废弃走双写过渡期 |
| 版本获取 | `cide_abi_version()`（capi 契约版本）+ `cide_engine_version()`（引擎版本，含构建期 git hash） |
| 语言中立 | **本 schema 语言中立，不依赖任何前端实现**：字段语义只由引擎与出口定义，任何前端（含已切割的历史前端资产）都只是消费方；回放校验以出口 JSON 为准（§7.1） |

---

## 1. 顶层对象：`StepPayload`

单个执行步的完整快照。

| 字段 | 类型 | 可空 | 语义 |
|---|---|---|---|
| `step_index` | int32 | 否 | 步号（0 起） |
| `code_line` | int32 | 否 | 当前源码行号（1 起；0 = 无源码行） |
| `func_name` | string | 否 | 当前函数名（教学可读名，如 `main` / `bubble_sort`）；未知时为空串 |
| `semantic_label` | string | 否 | 教学语义标签，由源码行 + 变量值推断（如 `循环 i=0, j=1`、`交换 arr[1]↔arr[2]`、`调用 printf`、`内存分配`） |
| `algorithm_step` | object | **是** | 算法步骤语义快照（§2.6）；未命中算法模板时为 `null` |
| `local_vars` | array | 否 | 当前作用域变量快照（§2.1）；无变量时为 `[]` |
| `call_stack` | array | 否 | 调用栈（§2.2）；**自底向上**，最后一个元素为当前帧 |
| `vis_events` | array | 否 | 本步产生的可视化事件（§2.7）；**取走式**：同一步数据不会被重复投递 |
| `heatmap_line` | int32 | 否 | 热力图行号（当前实现恒等于 `code_line`） |
| `heatmap_count` | uint64 | 否 | 该行**截至本步**的累计执行次数 |
| `accessed_vars` | array | 否 | 本步读写的变量（§2.3） |
| `array_snapshots` | array | 否 | 数组快照（§2.4），供条形图/可视化 |
| `pointer_snapshots` | array | 否 | 指针快照（§2.5），四状态见 §3.1 |
| `root_cause_hint` | object | **是** | 运行时陷阱的根因提示（§2.8）；无提示时为 `null` |

### 示例（截断）

```json
{
  "step_index": 42,
  "code_line": 12,
  "func_name": "bubble_sort",
  "semantic_label": "循环 i=0, j=1",
  "algorithm_step": {
    "algorithm_name": "bubble_sort",
    "display_name": "冒泡排序",
    "phase": "比较",
    "description": "比较 arr[1] 与 arr[2]"
  },
  "local_vars": [
    { "name": "i", "addr": 1048576, "is_local": true, "ty_name": "int", "value": "0" },
    { "name": "j", "addr": 1048580, "is_local": true, "ty_name": "int", "value": "1" }
  ],
  "call_stack": [{ "func_name": "main", "return_line": 0 }, { "func_name": "bubble_sort", "return_line": 0 }],
  "vis_events": [{ "ty": 1, "line": 12, "extra0": 0, "extra1": 0, "extra2": 0, "context": "arr[j] > arr[j+1]" }],
  "heatmap_line": 12,
  "heatmap_count": 7,
  "accessed_vars": [{ "name": "j", "access_type": "Read" }, { "name": "arr", "access_type": "Read" }],
  "array_snapshots": [{ "name": "arr", "element_ty": "int", "elements": ["5", "3", "4", "1", "2"] }],
  "pointer_snapshots": [
    { "name": "p", "addr": 1048600, "ty_name": "int*", "target_addr": 20480, "target_name": "arr", "status": "Valid" }
  ],
  "root_cause_hint": null
}
```

---

## 2. 子结构

### 2.1 `ApiVariableSnapshot` — 变量快照

| 字段 | 类型 | 语义 |
|---|---|---|
| `name` | string | 变量名 |
| `addr` | uint32 | 变量自身在线性内存中的地址 |
| `is_local` | bool | 是否局部变量（`false` = 全局/静态） |
| `ty_name` | string | **C/C++ 风格类型名**（如 `int` / `unsigned int` / `const char*` / `int[5]` / `struct Node` / `Foo` / `int&`）。**自 2026-09-11 起为稳定可读名**（此前是 Rust `Type` 的 `Debug` 表示，如 `Int { is_unsigned: false, is_const: false }`，属内部结构泄漏，见 §8 #7）；指针识别见 §3.1 |
| `value` | string | 值的字符串化。`double`/`float` 按定点格式化并去掉尾随 0；整数为十进制；指针为十六进制 `0x…`；**数组为元素摘要**（如 `{5, 3, 1, 4, 2}`，超过 16 个元素截断为 `…`），不再显示首元素 |

### 2.2 `ApiFrameInfo` — 调用帧

| 字段 | 类型 | 语义 |
|---|---|---|
| `func_name` | string | 帧对应函数名 |
| `return_line` | int32 | **当前恒为 0**（MVP 简化，schema 保留字段；见 §8 已知限制） |

数组顺序**自底向上**：`call_stack[0]` 为最外层（通常 `main`），`call_stack[last]` 为当前执行帧。

### 2.3 `AccessedVar` — 本步访问的变量

| 字段 | 类型 | 语义 |
|---|---|---|
| `name` | string | 变量名 |
| `access_type` | string | `"Read"` 或 `"Write"`（§3.2） |

同名变量在同一集合中可能同时出现 `Read` 与 `Write` 两条（如 `x++`）。

### 2.4 `ArraySnapshot` — 数组快照

| 字段 | 类型 | 语义 |
|---|---|---|
| `name` | string | 数组变量名 |
| `element_ty` | string | 元素类型名 |
| `elements` | string[] | 元素值（字符串化，规则同 `value`） |

### 2.5 `PointerSnapshot` — 指针快照

| 字段 | 类型 | 语义 |
|---|---|---|
| `name` | string | 指针变量名 |
| `addr` | uint32 | 指针变量自身的地址 |
| `ty_name` | string | 指针类型名 |
| `target_addr` | uint32 | 指向的地址（`0` = NULL） |
| `target_name` | string | 被指对象的变量名（能在当前快照中匹配到地址时给出，否则空串） |
| `status` | string | 四状态枚举，见 §3.1 |

### 2.6 `AlgorithmStepSnapshot` — 算法步骤

| 字段 | 类型 | 语义 |
|---|---|---|
| `algorithm_name` | string | 算法标识（如 `bubble_sort`） |
| `display_name` | string | 教学展示名（如 `冒泡排序`） |
| `phase` | string | 阶段（如 `比较` / `交换` / `初始化`） |
| `description` | string | 一句话教学描述 |

### 2.7 `VisEvent` — 可视化事件

| 字段 | 类型 | 语义 |
|---|---|---|
| `ty` | int32 | 事件类型码，见 §3.3 |
| `line` | int32 | 触发事件的源码行 |
| `extra0` / `extra1` / `extra2` | int32 | 类型相关的附加整数（当前实现恒为 0，保留扩展） |
| `context` | string | 人类可读上下文（如 `arr[j] > arr[j+1]`） |

### 2.8 `RootCauseHint` — 陷阱根因提示

| 字段 | 类型 | 可空 | 语义 |
|---|---|---|---|
| `category` | string | 否 | 根因类别，见 §3.4 |
| `one_liner` | string | 否 | 一句话解释（面向学生） |
| `related_lines` | int32[] | 否 | 相关源码行（可点击跳转） |
| `suggested_fix_kind` | string | 否 | 修复类型，见 §3.5 |
| `suggested_fix_line` | int32 | **是** | 建议修复位置 |
| `suggested_fix_desc` | string | **是** | 修复的自然语言描述 |

---

## 3. 枚举

### 3.1 `pointer_snapshots[].status` — 指针四状态

| 取值 | 语义 |
|---|---|
| `"Valid"` | 指向有效内存（栈、全局、已分配未释放的堆块） |
| `"Freed"` | 指向**已释放**的堆内存（地址命中 `is_heap && is_freed` 的已登记区域）——UAF 教学信号 |
| `"Null"` | NULL 指针（`target_addr == 0`） |
| `"Dangling"` | 悬空/越界：地址落在 NULL 陷阱区（`< 0x1000`）或线性内存之外（≥ 1MB） |

**判定优先级（实现契约）**：`Null` → `Dangling` → `Freed` → `Valid`（顺序不可交换：`target_addr == 0` 先于一切；越界先于"是否已释放"）。

**指针变量的识别**：`ty_name` 含 `*` 或 `Pointer` 的变量进入指针快照；`value` 以 `0x…` 十六进制或十进制无符号解析失败者跳过（如函数指针的符号名）。

> 说明：`Freed` 的判定依赖引擎的**有界隔离区**（free 后地址在隔离窗口内不复用，见 [`CIDE_HEAP_QUARANTINE_DECISION.md`](../current/CIDE_HEAP_QUARANTINE_DECISION.md)）。隔离窗口外的 UAF 可能退化为 `Valid`（读到复用块），属已知差异。

### 3.2 `accessed_vars[].access_type`

| 取值 | 语义 |
|---|---|
| `"Read"` | 本步读取了该变量 |
| `"Write"` | 本步写入（含 `++`/`--`）了该变量 |

大小写敏感；无第三种取值（读写同时发生时给出两条记录）。

### 3.3 `vis_events[].ty`

| 取值 | 语义 | 当前状态 |
|---|---|---|
| `1` | 比较（`compare`）——源码中数组元素参与的比较表达式 | **当前唯一产出值** |
| 其余 | 保留 | 预留给交换/移动/递归/指针变更等可视化事件 |

### 3.4 `root_cause_hint.category`

`OffByOne` / `UseAfterFree` / `UninitializedIndex` / `WrongStartIndex` / `SizeMismatch` / `NullDeref` / `DivZero` / `DoubleFree`

### 3.5 `root_cause_hint.suggested_fix_kind`

`ChangeLeToLt` / `AddNullCheck` / `InitVariable` / `FixLoopStart` / `FixArraySize` / `SetNullAfterFree` / `AvoidDivZero` / `None`

---

## 4. frameCache 窗口语义（重要一致性契约）

引擎持有滑动窗口 `frame_cache`（`UnifiedEngine`），窗口内保存最近的 `StepPayload`。

| 参数 | 值 | 说明 |
|---|---|---|
| 窗口大小 | **2000 帧** | 超过即触发裁剪 |
| 裁剪比例 | **20%** | 裁剪时丢弃**最早**的 `ceil(len × 0.2)` 帧（至少 1 帧） |
| `frame_cache_start_step` | 步号 | `frame_cache[0]` 对应的真实步号；窗口滑动时同步前移；`reset()` 后为 0 |
| `max_collected_step` | 步号 | 窗口内已收集的最大步号；窗口为空时为 `frame_cache_start_step - 1` |
| 检查点间隔 | **20 步** | `CheckpointManager::new(20)`；seek 的重放粒度 |

### 4.1 查询 `payload.get(start, end)`

- 语义为**左闭右开** `[start, end)`，按**真实步号**索引；
- 请求区间被裁剪到当前窗口内：窗口外的部分**静默丢弃**，返回数组可能为空；
- 返回数组的第一个元素不一定对应 `start`（若 `start` 早于 `frame_cache_start_step`）。

### 4.2 越窗 seek（懒重算）

目标步不在窗口内时的行为（顺序契约）：

1. 取**最近的检查点**（`checkpoints.nearest(target)`）；无可用检查点 → `success: false` + `error`；
2. 恢复 VM 到该检查点（增量快照在此重建为全量）；
3. **正向重放**至目标步：逐步执行并逐帧收集 payload；重放途中每跨过一个检查点间隔（20 步）保存新检查点（加速后续 seek）；
4. 重放中遇到 `trap` → `success: false`；遇到 `waiting_input` / `finished` → 以该步为终点返回 `success: true`；
5. 重放结束后执行**窗口重置**：`frame_cache_start_step = max(0, target - 2000 + 1)`，并**截断 target 之后的帧**——即 seek 回退后窗口是"目标步及其之前 1999 帧"，不含未来帧。

### 4.3 seek 后各视图的一致性

seek 到第 N 步后，`local_vars` / `call_stack` / `array_snapshots` / `pointer_snapshots` / `heatmap_count` 均以**第 N 步的快照**为准（而非"当前最新状态"）。消费方据此重绘所有视图，不需要自行回退状态。

`cide_get_step_payloads_json(start, end)` 的响应携带 `cache_start_step` 与 `max_collected_step`，消费方据此判断"我要的区间是否还在窗口内"。

---

## 5. 差分编码：`StepStreamBatch` / `StepPayloadDelta`

用于批量传输（省带宽）：一个 batch = 1 个完整基准快照 + N 个差分。

### 5.1 `StepStreamBatch`

| 字段 | 类型 | 语义 |
|---|---|---|
| `symbol_table` | string[] | 全局去重字符串池；**索引 0 恒为空串**，其余按首次出现顺序分配 |
| `base_payloads` | object[] | 每 batch 的第 1 个完整快照（`StepPayloadRef`） |
| `deltas` | object[] | 后续步的差分（`StepPayloadDelta`） |
| `finished` / `trapped` / `waiting_input` / `paused` | bool | 批次结束状态 |
| `current_line` | int32 | 批次结束时的源码行 |
| `trap_message` | string\|null | 陷阱信息 |
| `cache_start_step` | int32 | 窗口起始步号（与 §4 同义） |

### 5.2 `StepPayloadDelta`（字段级差分）

| 字段 | 类型 | 语义 |
|---|---|---|
| `step_index` / `code_line` / `func_name_idx` / `semantic_label_idx` | 标量 | 同 `StepPayload`，字符串改为符号表索引 |
| `algorithm_step` | object\|null | 同上（`null` = 无；注意与"未变"的区分见下） |
| `var_deltas` | object[] | **值发生变化**的变量 `{name_idx, value}` |
| `new_vars` | object[] | 新出现的变量（完整快照） |
| `removed_var_name_indices` | int32[] | 消失的变量名索引 |
| `call_stack` | array\|**null** | `null` = 调用栈无变化；否则为**完整新调用栈** |
| `vis_events` | array\|**null** | `null` = 无事件；否则为**当前步完整事件列表**（空数组表示"本步确实没有事件"） |
| `accessed_vars` | array\|**null** | `null` = 无变化；否则完整新集合 |
| `array_snapshots` | array\|**null** | `null` = 无变化；否则为新增/替换的数组快照 |
| `removed_array_name_indices` | int32[] | 被删除的数组名索引 |
| `pointer_snapshots` | array\|**null** | `null` = 无变化；否则为新增/替换的指针快照 |
| `removed_pointer_name_indices` | int32[] | 被删除的指针名索引 |
| `root_cause_hint` | object\|null | 同 `StepPayload` |

**`null` 与 `[]` 的区别是契约的一部分**：`null` 表示"与上一步相同（沿用）"，`[]` 表示"本步为空"。消费方必须区分。

### 5.3 解码不变量

对同一 batch：`base_payloads[i]` 依次应用后续 `deltas` 后，必须重建出与逐个 `StepPayload` **逐字段等价**的对象。该不变量已有回归测试覆盖（`stream.rs::test_accessed_vars_and_vis_events_delta` 等）。

---

## 6. 出口形状（capi 与 serve 共用同一语义）

### 6.1 `cide_step_next_json(session)`

```json
{
  "payloads": [ /* StepPayload[]，本次推进产生的步 */ ],
  "finished": false,
  "trapped": false,
  "waiting_input": false,
  "paused": false,
  "current_line": 12,
  "trap_message": null,
  "cache_start_step": 0
}
```

`paused: true` 表示命中断点（断点在 VM 层判定）。`payloads` 在本入口固定为 1 个元素（单步推进），批量推进属第三批 `run_auto_steps`。

### 6.2 `cide_get_step_payloads_json(session, start, end)`

```json
{ "payloads": [ /* StepPayload[]，已裁剪到窗口内 */ ], "cache_start_step": 0, "max_collected_step": 137 }
```

### 6.3 `cide_cli serve` 帧

NDJSON；请求带 `id`，响应回填同一 `id`；错误帧与成功帧**同构**（`{"id":n,"ok":false,"error":{"code":…,"message":…}}`）。方法名与 capi 入口一一对应，见 [`CIDE_CLI.md`](../current/CIDE_CLI.md)。

---

## 7. 回放场景校验记录

> 校验输入分两类：**我方（Cide）现有消费序列**（原生前端 frameCache 消费序列（**已切割的历史资产**，同形口径现由 `cide_cli serve` 复现）/ StepStreamBatch 的真实调用序列）与**对端（SharpTutor）三组场景**（防抖编译流 / fixtures 判分流 / 单步+seek+内存查询交错流，见 `CIDE_CAPI_REVIEW_RESPONSE.md` §2 与 §8）。

| # | 场景 | 输入序列 | 期望（schema 断言） | 状态 |
|---|---|---|---|---|
| C1 | 原生前端 frameCache 消费序列（**已切割的历史资产**；现由 `cide_cli serve` 同形口径复现，见 C4） | `compile` → `step_begin` → `step_next` ×N → `get_step_payloads_json(窗口)` → 断点暂停 → 继续 | 顶层 14 字段齐全；`call_stack` 自底向上；`cache_start_step` 单调不减；窗口裁剪后 `payloads` 非空且步号连续 | ✅ 已实测（§7.2，由 `step_payload_schema_v0_1_test` 冻结） |
| C2 | 差分往返 | 同一步序列的 `StepPayload[]` → `encode_payloads` → `decode` | 解码结果与原始 payload 逐字段等价；`null` 与 `[]` 语义区分正确 | ✅ 已有回归测试（`stream.rs::test_accessed_vars_and_vis_events_delta` 等） |
| C3 | 窗口滑动与越窗 seek | 连续执行 >2000 步 → 查询窗口 → seek 回退到窗口外 → 再查询 | 窗口 2000 帧、丢最早 20%；越窗 seek 触发检查点恢复 + 正向重放；seek 后窗口为 `[target-1999, target]` | ✅ 已实测（`step_payload_schema_v0_1_test` + `unified_engine_window_test`） |
| C4 | serve 出口形状一致性（新增） | `cide_cli serve`：`compile` → `run` → `output.delta` → `step.begin` → `step.next` → `payload.get` → `seek` → `session.reset` | 与 capi 同形：`payloads` 字段、`cache_start_step`、`status` 枚举、iso 帧（`id`/`ok`） | ✅ 已实测（`scripts/serve_smoke.py`，26 项断言） |
| S1 | 防抖编译流（对端） | 高频 `compile_unit` + `compile_json`，期间夹杂 `step_next` | 诊断 JSON 稳定；`payloads` 不因重编译而串步 | ⏳ 待对端执行（Cide 侧接口已就绪） |
| S2 | fixtures 判分流（对端） | 固定输入程序批量判分：`compile` → `run_json` → `get_output_delta` | `status`/`return_value`/`steps_executed` 稳定可复现（配 `cide_set_deterministic`） | ⏳ 待对端执行 |
| S3 | 单步 + seek + 内存查询交错流（对端） | `step_next` / `seek` / `memory.regions` 交错 | 三视图一致：指针四状态与内存区域状态不矛盾；`accessed_vars` 枚举值合法 | ⏳ 待对端执行（`memory.regions` 属 capi 第二批；serve 已有过渡形态可先回放） |

### 7.1 校验方法

- Cide 侧回放以 **capi 入口**（而非内部 Rust API）执行——协议契约必须在出口处成立；
- 断言对象是 **JSON 的字段与取值**，不是 Rust 结构体；
- 每次 schema 变更（v0.1 → v0.2）必须重跑 C1–C3，并在本表追加一行历史记录。

### 7.2 本次实测记录（2026-09-11）

| 项 | 载体（可复现命令） | 结果 |
|---|---|---|
| C1 顶层 14 字段 + 子结构字段冻结 | `cargo test --test step_payload_schema_v0_1_test` → `test_step_payload_top_level_fields_frozen` / `test_substructure_fields_frozen` | ✅ 5 passed（含指针四状态 / accessed_vars 枚举字面量 / 一步全字段序列化） |
| C1 出口形状（capi） | `cargo test --test capi_first_batch_tests` → `test_step_next_and_payload_schema_fields` | ✅ 18 passed（同批含隔离预算、断点、游标等用例） |
| C2 差分往返（`null` vs `[]`） | `cargo test --workspace`（`unified::stream` 单测） | ✅ 全绿（exit 0） |
| C3 窗口 2000 帧 + 越窗行为 | `test_frame_cache_window_2000_frames_with_20pct_trim` + `cargo test --test unified_engine_window_test` | ✅ 窗口上限与 `cache_start_step` 前移断言通过 |
| C4 serve 出口一致性 | `python scripts/serve_smoke.py` | ✅ 26 项断言通过（id 关联 / 帧同构 / 生命周期 / 与 capi 同形的 payload 字段） |
| 静态检查 | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | ✅ 无警告（exit 0） |
| S1–S3 | — | ⏳ **未执行**：SharpTutor 场景由对端提供，本轮无对端输入。已在表内标注为待办，避免"文档宣称已校验"的失真 |

> 字段冻结的工程意义：`step_payload_schema_v0_1_test.rs` 断言的是 **capi 出口 JSON 的键集合与枚举字面量**。任何字段改名/增删都会让该测试失败，从而强制走版本化流程（而不是让 schema 文档悄悄过期）。

### 7.3 对端接入前的预备结论

- 协议层字段已全部 `serde::Serialize` 落链（`cide_step_next_json` 直接输出），不存在"文档有、出口无"的字段；
- 三组对端场景所需的入口中，**`memory.regions` 属 capi 第二批**（尚未落地）——S3 场景需在第二批完成后才能完整回放，这是**已知的前置依赖**，不是 schema 缺口；
- S1/S2 所需入口（`compile_json` / `run_json` / `get_output_delta` / `set_deterministic`）均已就绪。

---

## 7.x 预留位：异常域字段（CSHARP_EXTENSION_PLAN.md §6-A，随 v0.1 冻结）

以下四个字段为 **C# 前端异常教学**（CS3b 激活）预留，v0.1 消费方**必须容忍其
不存在**（字段可选）；激活前不出现在任何 payload 中：

| 字段 | 类型 | 语义 |
|------|------|------|
| `handler_depth` | int | 当前受几层 try 保护 |
| `unwinding` | bool | 展开态显式标记 |
| `unwind_frames_left` | int | 剩余待展开帧数（展开动画驱动字段） |
| `current_exception` | `{type_name, message, addr, origin_line} \| null` | 当前异常寄存器；`addr` 联动内存面板 region 高亮（ARC 教学闭环）；**`origin_line` 为原始抛点行号**——`throw;` 保留、`throw e;` 改写为当前点（CSHARP_EXTENSION_PLAN §4.4 轨迹考点：知识卡片"原始抛点在第 X 行"直读本字段，**不解析 trap message 文本**；评审补充 2026-09-12） |

语义细则见 `CSHARP_EXTENSION_PLAN.md` §6-A；词汇契约（`semantic_label` 异常域
条目）同批进附录。

---

## 7.x2 S1–S5 回放执行记录（2026-09-12，驱动 `scripts/replay/replay_s1_s5.py`）

对端签字材料（SharpTutor `docs/cide-replay/`）已采纳回放，**61/61 断言 PASS**：

| 组 | 结果 | 备注 |
|----|------|------|
| S1 防抖编译流 | PASS（A1–A10） | 诊断形状/稳定性/步数据不串全过 |
| S2 fixtures 判分流 | PASS（A1–A6 ×2 轮 ×3 fixture） | 字节级判分、deterministic 可复现 |
| S3 单步+seek+内存交错流 | PASS（A1–A16） | 修复三个引擎缺陷后全绿（见 CHANGELOG 同日 Fixed） |
| S4 异常交错流 | v0.1 阶段 = A0 现状断言（由 S5 A2 覆盖） | 激活契约随 CS3b 回放 |
| S5 预留位缺省语义 | PASS（A1–A5） | v0.1 键集合 ⊆ 14 项全集、预留字段不存在、ABI/版本锚定 |

**回放暴露并修复的引擎缺陷（修复提交见 CHANGELOG）**：
1. 越窗 seek 负下标 → 占位填充无限循环（吃满 63.6GB 内存；越窗路径现重置窗口 + try_from 防御）；
2. step 0 锚点检查点被裁剪 → 越窗 seek 永久失败（锚点永不裁剪）；
3. 重放区间排他 → 目标步自身不在窗口（`..target` → `..=target`）；
4. 断点暂停双层（VM `paused` + 引擎 `is_paused`）清断点只恢复一层；
5. 进入被调函数第一步误标"递归调用 X"（caller_line 归因修正，schema §8 #10 实锤项）。

**语义演进（对端文档同步）**：seek 语义在锚点固化后更新——step 0 检查点恒存在，
任何 >=0 的 seek 都可成功；"success:false" 仅在 0 步场景成立（S3 §6 观测 #5 的
实测基于无锚点固化的旧版，下游文档已注明）。

---

## 8. 已知限制与遗留（诚实记录）

| # | 限制 | 影响 | 计划 |
|---|---|---|---|
| 1 | `ApiFrameInfo.return_line` 恒为 0 | 调用栈视图无法显示返回行 | MVP 简化遗留；补全需在 VM 帧结构记录返回行 |
| 2 | 无 **mangled 名**字段 | 计划 §3.2 要求"函数 display_name 与 mangled 双字段"；当前只有单一 `func_name`（教学可读名）。C++ 场景下 `__ctor__Vec` 之类的内部名与源码名不同，消费方无法同时拿到两者 | v0.2 增加 `func_display_name` + `func_mangled_name`（**只增不改**：保留 `func_name` 作为 display 语义） |
| 3 | `vis_events[].ty` 仅 `1`（compare） | 交换/移动等事件无类型码 | 与算法步骤模板一起扩充 |
| 4 | `root_cause_hint` 仅陷阱路径填充 | 常规步恒为 `null` | 按认知推理层需要扩展 |
| 5 | 精确 `end_line`/`end_column` | 属诊断 schema（`compile_json`），不在本 schema 内；当前为"起点 + 1"退化值 | 按诊断类别分批补（高价值跨度优先） |
| 6 | 窗口外的历史 payload 不可查询 | 消费方须自行落地持久化（或依赖 seek 重放）；`payload.get` 对越窗区间静默返回子集 | 设计如此（内存有界）；消费方契约已在 §4.1 写明 |
| 7 | ~~`ty_name` 为 Rust `Debug` 表示~~ | 拼写随内部重构变化，且把内部枚举结构（`Int { is_unsigned: false, … }`）泄漏到教学输出 | **✅ 已修复（2026-09-11）**：改为 C 风格稳定可读名（单一来源 `cide_runtime::type_display_name`），消费方可直接显示。指针识别规则不变（含 `*`） |
| 8 | `local_vars` 曾含**跨函数**变量与同名重复 | 消费方看到 `helper` 的局部变量出现在 `main` 的 payload（且用错 `locals_base` 读出垃圾值），两个 `for` 各声明一个 `i` 时无法区分 | **✅ 已修复（2026-09-11）**：按函数归属 + 声明行（新增 `Symbol::decl_line`）过滤，同名取"已进入作用域且最晚声明"者；无有效执行位置（`code_line == 0`）时不输出局部变量 |
| 9 | `code_line` 是**合并源码的全局行号**，payload 未携带文件名 | 多文件会话中消费方无法自行把 `code_line` 映射回"哪个文件的第几行"（引擎内部已按 `file_ranges` 正确映射，语义标注不再串文件） | v0.2 增加 `code_file` 字段（**只增不改**：`code_line` 保持全局行号语义，避免破坏既有断点/heatmap 口径） |
| 10 | 函数定义行判定为递归调用 | 仅当左花括号与函数签名**同行**时被排除；`int f(...)` 换行写 `{` 时仍可能把定义行标成"递归调用 f" | 需要多行签名识别（教学子集内少见）；已知限制 |

---

## 附录 A：字段与不变式速查

- `step_index` 严格递增（同一会话内连续步）；重放/seek 后仍以真实步号为准，不回退编号。
- `heatmap_count` 单调不减（同一行同一会话内累计）。
- `local_vars` 为**当前作用域**快照，不包含已出作用域的变量（差分层用 `removed_var_name_indices` 表达消失）。
- `pointer_snapshots` 只包含**指针类型**变量；`status` 判定优先级见 §3.1。
- 一个 `StepPayload` 对应**一条字节码指令**，不是"一行源码"；同一源码行可产生多步。
- `vis_events` 是取走式的：同一步不会重复投递（重放时会重新生成）。
