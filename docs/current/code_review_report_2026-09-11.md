# 代码审查报告 2026-09-11（外部 PR 清单复核与修复跟踪）

> **来源**：外部审查 / PR 清单，共 12 项（6 条能力缺口 + 教学标注 P0-1~P0-4 + P1-5~P1-6 + P2-7 三项）。
> **复核方式**：不复用清单结论——逐条读源码 + 用 `cide_cli` / `cide_cli serve`（JSON-lines）独立复现，
> 探针脚本留存于 `tmp/pr_check/`（`probe_payload.py` / `probe_dup.py` / `probe_oob.py` / `probe_multi2.py`）。
> **复核结论（2026-09-11）**：**12 项全部成立**，其中 3 处表述需要修正（§2），并在复核过程中发现 2 项清单未提的同源问题（§3）。
> **修复策略**：按优先级分批实施，每批完成后跑测试与防线，结果记入 §4 与 [`CHANGELOG.md`](../../CHANGELOG.md)。

---

## 1. 逐条复核结论

| # | 项 | 判定 | 独立复现证据 | 文档记录 | 状态 |
|---|---|---|---|---|---|
| 1 | lambda 返回类型硬编码 `Type::int()`（typeck 两处） | ✅ 成立 | `crates/cide_typeck/src/expr/cpp.rs:208`（`resolve_lambda` 注册 `__call`）、`crates/cide_typeck/src/lib.rs:381`（Pass 4 提升生成的 `FuncDecl`） | `native/tests/CPP_FAILURES.md:132` ✅ | ⏳ 待修 |
| 2 | 文件作用域 lambda 变量不支持（报 E3004） | ✅ 成立 | `CPP_FAILURES.md:131` 已录复现式（`auto gf = [](int){…}` 于 `main` 外） | `CPP_FAILURES.md:131` ✅ | ⏳ 待修 |
| 3 | `scanf` 返回值未实现（视作 void） | ✅ 成立 | Cide：`E3004 无法将 'void' 赋值给 'int'`；Clang：`a=5 r=1` | `docs/current/C_SUBSET_SPEC.md:416` ✅ | **✅ 已修（批次 G）** |
| 4 | `scanf` 普通字符指令被忽略 | ✅ 成立 | `scanf("a=%d",&a)` 输入 `a=5` → Cide `a=0`、Clang `a=5` | `C_SUBSET_SPEC.md:415` ✅ | **✅ 已修（批次 G）** |
| 5 | 堆决议第三道墙（region 表封顶）无用例 | ✅ 成立 | `docs/current/CIDE_HEAP_QUARANTINE_DECISION.md:75-76` 自述"region 表封顶为**推导项**（见 §5）" | 决议文档 ✅ | **✅ 已修（批次 F，并揭出更严重的关联缺陷，见 §4）** |
| 6 | 泄漏报告输出粘连 | ✅ 成立（比描述更重） | 实测 `===== 内存泄漏检测报告 =====发现 1 处…：  • 第 4 行…💡 提示：…======` 压成一行：`append_leak_report` 的 **5 行全部无尾随 `\n`**，而 display 为零分隔顺序拼接 | 未记录 | **✅ 已修（批次 A）** |
| P0-1 | 冒泡趟数文案把概念教反 | ✅ 成立 | 实测 desc：`第 1 趟：将第 5 大的元素放到正确位置`(13×)、`第 2 趟：第 4 大`、`第 3 趟：第 3 大`、`第 4 趟：第 2 大`；根因 `crates/cide_algorithm_steps/src/sorting.rs` 的 `kth = n - i` | 未记录 | **✅ 已修（批次 A）** |
| P0-2 | 越界文案（描述不存在的比较） | ✅ 成立（数字对、模板类型需修正，见 §2.1） | 实测 5 元素冒泡含 `arr[5]` 的 desc = **24 次**（与清单一致），真实比较 10 次 | 未记录 | **✅ 已修（批次 C）** |
| P0-3 | 同一 payload 内两套标注互相矛盾 | ✅ 成立 | 实测同一 payload：`semantic_label='交换 arr[5]↔arr[6]'` 与 `description='交换 arr[0]↔arr[1]，较大的元素向右移动'` 并存；根因 `native/src/unified/collector.rs:254-256` 的 `loop_vars.first()` 取到白名单里最先出现的 `n` | 未记录 | **✅ 已修（批次 C）** |
| P0-4 | 多文件会话语义标注捏造 | ✅ 成立（根因更深，见 §3.1） | 实测两文件（helper.c 在前）：`func_name='main'` 时 `code_line = 20..25`，而 main.c 仅 13 行、helper.c 仅 16 行；根因 `collector.rs:203` 用 `compile_units.first()` 的**文件内行号**去查**跨文件全局偏移行号** | 未记录 | **✅ 已修（批次 D）** |
| P1-5 | CLI 退出码不反映编译失败 | ✅ 成立 | `cide_cli compile bad.c` 退出码 **0**（诊断照常打印）；`run bad.c` = 1 | 未记录 | **✅ 已修（批次 A）** |
| P1-6 | C++ 向上转型被误报为"数据截断" | ✅ 成立（级别/编码需修正，见 §2.2） | Cide：`[警告] 不兼容的指针类型赋值：Base* ← Derived*。` + 建议"隐式类型转换可能导致数据截断"；`clang++ -Wall -Wextra` 实测零警告、exit 0。根因 `crates/cide_typeck/src/convert.rs:309` 的 `t_pointee != v_pointee` 未考虑继承可达性，且复用标量转换码 `W3053` | **`CPP_SUBSET_SPEC.md` 与全仓 md 均无记录** ❌ 违反诚实记录纪律 | **✅ 已修（批次 E，含补记录）** |
| P2-7a | 两个 `for` 各自声明 `i` 时 `local_vars` 列出两个 `i` | ✅ 成立 | 实测 `local_vars = ['s','i','i']`：两个 `i` 地址不同（1048568 / 1048572），消费方无法区分 | 未记录 | **✅ 已修（批次 B）** |
| P2-7b | 数组在 `local_vars` 暴露内存地址，与 `array_snapshots` 重复且误导 | ✅ 成立 | 实测 `{"name":"a","addr":1048548,"value":"0"}` 与 `array_snapshots` 的 `{"name":"a","elements":[…]}` 重复 | 未记录 | **✅ 已修（批次 B）** |
| P2-7c | `ty` 字段以 Rust Debug 形式泄漏到用户输出 | ✅ 成立（字段名为 `ty_name`，见 §2.3） | 实测 `"ty_name": "Int { is_unsigned: false, is_const: false }"` | 未记录 | **✅ 已修（批次 B）** |

---

## 2. 对清单的 3 处修正

### 2.1 P0-2：24 次越界描述落在"交换"模板，不是"比较"模板

清单写"产生 24 步 **比较** arr[4] 与 arr[5]"。独立复现（`probe_oob.py`，5 元素冒泡）：

```
含 'arr[5]' 的 desc 步数            = 24   ← 与清单数字一致
含 '比较 arr[4] 与 arr[5]' 的步数   = 0
含 '比较 arr[' 的 desc 步数         = 235
24× '交换 arr[4]↔arr[5]，较大的元素向右移动'
```

即越界描述全部出现在**交换**模板；比较模板采样时 `j` 只取到 0..3，越界发生在 `temp = arr[j]` 行采样时
`j` 已自增到退出值。**结论不变**（确有 24 步越界描述、真实比较仅 10 次），但定位修复点时应落在交换/比较两个模板共同的"采样时机"判据上。

### 2.2 P1-6：Cide 输出的是警告 `W3053`，不是 `E3053`

Cide 实际产出 `ErrorCode::W3053_ImplicitScalarConversion`（warning）。`(E3053)` 是 `cide_cli` 统一按 `E` 前缀
打印 `error_code` 造成的**显示问题**（诊断 severity 本身仍是 warning）。"教学上把向上转型讲成数据截断"的判断成立。

### 2.3 P2-7c：字段名是 `ty_name`（字符串），不是 `ty`

`ApiVariableSnapshot.ty_name` 由 `format!("{:?}", v.ty)` 生成（`collector.rs:25`），因此是 `Type` 枚举的 Rust Debug 形式。

---

## 3. 复核中发现的清单未提问题（同源）

1. **`code_line` 是跨文件全局偏移行号，与文件内行号不一致**（P0-4 根因的更深一层）：
   实测两文件会话中 `func_name='main'` 的 `code_line` 达到 20..25，而 main.c 只有 13 行 —— 单文件时
   "全局行号 == 文件内行号"因此从未暴露；多文件时 `compile_units.first()` 查表必然错配。修复需同时
   给出"行号 → 编译单元"的映射，而不只是把 `first()` 换成"按文件查找"。
2. **同名变量会连带污染语义标签**：`semantic_label` 实测为 `循环 n=0, i=0, j=0, i=0, j=0`，重复的 `i/j`
   来自与 P2-7a 同源的变量快照，说明该缺陷不止影响 `local_vars` 字段。

---

## 4. 分批修复记录

### 批次 A（2026-09-11）：P0-1 + P1-5 + 条目 6

| 项 | 改动 | 实测验证 |
|---|---|---|
| P0-1 | `crates/cide_algorithm_steps/src/sorting.rs`：`kth = n - i` → `i + 1`（并加 `i + 1 <= n` 有效性判据），附注释说明"趟数 = 排名" | desc 变为 `第 1 趟：将第 1 大的元素放到正确位置`、`第 2 趟：第 2 大`、`第 3 趟：第 3 大`、`第 4 趟：第 4 大` |
| P1-5 | `native/src/bin/cide_cli.rs::cmd_compile` 接住 `compile_file` 的返回值，失败时 `exit(1)`（与 `cmd_run` 一致） | `compile bad.c` → **exit 1**；`compile` 正常文件 → exit 0 |
| 条目 6 | `crates/cide_runtime/src/runtime_state.rs::push_note` 统一为附注补齐尾随 `\n`（**仅 note 通道**，stdout/stderr 逐字节保真不动）；`engine/session_ops.rs::append_leak_report` 注释同步 | 泄漏报告恢复逐行：`===== 内存泄漏检测报告 =====` / `发现 1 处…` / `  • 第 4 行的 malloc…` / `💡 提示：…` / `==============================` |

**回归**：`cargo test --test end_to_end_extra_test --test fuzz_stress_test --test host_contract_tests
--test capi_first_batch_tests --test step_payload_schema_v0_1_test --test crash_regression_tests`
→ **366 passed / 0 failed**。

### 批次 B（2026-09-11）：P2-7（变量快照可见性与类型名）

| 项 | 改动 | 实测验证 |
|---|---|---|
| P2-7a | `Symbol::decl_line` 新增（codegen 参数/局部/静态/全局四处 + `compile_pipeline` 三处转换 + 2 个测试构造点）；`CideVM::get_variable_snapshot` 按 (函数归属, 声明行) 过滤 + 同名去重；`code_line == 0` 时不输出局部变量 | 两个 `for` 的 `i` 按执行位置正确切换（地址序列恰为 2 个且有序）；同一 payload 不再出现同名变量 |
| 同源缺陷（清单未提） | 函数内符号此前不过滤函数归属 —— `helper` 的局部变量以 `main` 的 `locals_base` 读出（地址错位 + 污染 `semantic_label`） | `main` 的 payload 不再含 `hn`；`helper` 的 payload 不再含 `mv` |
| P2-7b | `local_vars` 中数组的 `value` 改为元素摘要（`{5, 3, 1, 4, 2}`，>16 元素截断 `…`），`addr` 保留 | 数组条目 value 由首元素 `0` 变为元素摘要 |
| P2-7c | 新增 `cide_runtime::type_display_name`（C 风格可读名，单一来源，同时接管 `array_snapshots[].element_ty`） | `ty_name` 由 `Int { is_unsigned: false, is_const: false }` 变为 `int` / `int[5]` |

**回归**：新增 `native/tests/step_payload_vars_test.rs`；`cargo test` 全量 0 failed。

### 批次 C（2026-09-11）：P0-2 + P0-3（算法标注正确性）

| 项 | 改动 | 实测验证 |
|---|---|---|
| P0-2 | `infer_bubble_sort` 新增内层下标有效性判据（`j < n-1-i`），j 停在退出值时返回 `None` | 含 `arr[5]` 的描述 **24 → 0** 步 |
| P0-3 | 交换标签改为**从源码行解析下标标识符**（`temp = arr[j]` → `j`），解析不到再按 `j → i → k` 回退 | 同一步的两套标注逐字一致（`交换 arr[0]↔arr[1]` vs `交换 arr[0]↔arr[1]`） |

**回归**：`step_payload_vars_test.rs` 扩至 6 用例。

### 批次 D（2026-09-11）：P0-4（多文件行号归属）

| 项 | 改动 | 实测验证 |
|---|---|---|
| P0-4 | 新增 `Session::source_line_at(global_line)`（按 `CompileState.file_ranges` 换算文件与文件内行号，单文件会话保持原语义）；四处重复实现统一到它（collector / `AlgorithmContext` / engine / trace_analyzer） | `func_name='main'` 的 `line 25` 正确映射到 main.c 第 9 行；main 的标签不再串用 helper.c 的源码行 |
| 顺带修复 | 函数定义行（`int helper(int x) {`）不再被标成"递归调用 helper" | 新增用例断言不出现该标签 |

**回归**：`step_payload_vars_test.rs` 扩至 8 用例；schema 文档 §8 新增 #9（`code_line` 为全局行号、无文件字段）与 #10（花括号换行的定义行仍可能误判）。

### 批次 E（2026-09-11）：P1-6（向上转型误报）

| 项 | 改动 | 实测验证 |
|---|---|---|
| P1-6 | 新增专用码 `W3067_PointerTypeMismatch`（error_codes + 错误目录 + 建议文案）；`TypeChecker::is_upcast` 沿单继承链判定向上转型；指针不兼容文案改为指针语义 | `Base* b = new Derived();` → **零诊断**（与 `clang++ -Wall -Wextra` 一致）；`Derived* d = b;` → `W3067` 提示需要显式转换 |
| 补记录 | `CPP_SUBSET_SPEC.md` 新增 §4.5：三场景对照表 + 修复记录 + **剩余差异**（Clang 对向下转型是 error 拒绝编译，Cide 仅警告）+ 已知显示瑕疵（警告 code 被加 `E` 前缀） | clang++ 实测向下转型 `exit=1`，Cide `exit=0`（继续编译） |

**回归**：新增 `native/tests/pointer_upcast_test.rs`（3 用例）；`type_checker_unit_test.rs` 的 B39 用例改用新码文案。

### 批次 F（2026-09-11）：条目 5 补用例时发现的关联缺陷（会话级保险丝）

补第三道墙用例时发现 **第二道墙本身是坏的**：

| 缺陷 | 根因 | 实测 | 修复 |
|---|---|---|---|
| **步数保险丝从未生效（严重）** | `compile_pipeline::setup_vm` 硬编码 `vm.set_max_steps(10_000_000)`，每次 run 抹掉会话配置（与 `CideVM::reset()` "保留会话级配置"的注释冲突） | 设 2000 步的程序跑到 **305216 步**、撞 1MB 堆墙才停；旧用例只断言"消息含步数超限"，1000 万步同样满足 → 长期掩盖 | 删除该行（默认值由 `CideVM::default()` 提供）；`test_second_wall_max_steps_fuse` 强化为回显 `（1000 步）` |
| 配置静默丢弃 | capi `cide_set_max_steps` / `cide_set_call_depth_limit` 与 serve `config.set` 写成 `if let Some(vm) { .. }` 并返回"成功" | 会话无 VM 时配置被丢弃却报告成功 | 下沉到 `Session::set_max_steps` / `set_call_depth_limit`（无 VM 时先建立承载配置的 VM），capi 与 serve 共用 |
| 配置可写不可读 | `session_api::config()` 不含 `max_steps` / `call_depth_limit` | 消费方无法确认保险丝是否落到 VM 上（正是不透明掩盖了上一条） | 补回显（VM 未创建时为 `null`）+ `CideVM::max_steps()` getter |
| 第三道墙无用例 | 决议 §6 自述为"推导项" | — | 新增 `test_third_wall_region_table_bounded_when_step_fuse_trips_first`（region 条数 ≤ 步数上限、未撞 1MB 墙） |

**回归**：新增 `native/tests/session_config_test.rs`（4 用例：配置跨 run 存活并生效、调用深度上限存活、`config()` 回显、capi 编译前设置生效）。

### 批次 G（2026-09-11）：条目 3 + 条目 4（scanf 族）

| 项 | 改动 | 实测验证 |
|---|---|---|
| 条目 3 | `scanf` 按 C11 7.21.6.2 返回成功匹配并赋值的项数：typeck 由 `void` 改 `int`，`host_scanf_n` 统计并压栈（与早已正确的 `sscanf` 对齐） | `int r = scanf("%d", &a);` → Cide `a=5 r=1` = Clang |
| 条目 4 | 新增 `ScanfItem::Literal(u8)`：普通字符指令与输入流精确比较，不匹配即**停止解析**；`%%` 展开为字面 `%` 参与匹配；`sscanf` 同族同修 | `scanf("a=%d", &x)` 读 `a=42` → Cide `x=42` = Clang；读 `b=42` → `x=-1 r=0` = Clang |

**同批（由防线扩容暴露的真实缺陷）**：
- **标准输入换行口径统一**：capi `cide_set_input` / FRB `set_input` / serve `run.input` / CLI `-i` 各自用
  `str::lines()` 拆分，**丢掉行尾 `'\n'`**（E2E 防线用 `split_inclusive('\n')`）→ `getchar()` 永远读不到换行。
  统一到 `RuntimeState::split_stdin` / `set_stdin`。
- **Shadow 防线支持 `.in` 注入**：此前不喂 stdin，K&R 目录 29 个 `.in` 从未使用（两侧"都无输入"的虚假 match）。
  启用后首轮暴露 19 例 `output_gap`（`kr_1_8` 换行计数恒 0、`kr_4_3` 无输出等），修复换行口径后全部转绿。

**回归**：新增 `baseline/scanf_return_value.c` / `scanf_literal_match.c` / `scanf_literal_mismatch.c`（含负向）。

### 批次 H（2026-09-11）：条目 1 + 条目 2（lambda）

| 项 | 改动 | 实测验证 |
|---|---|---|
| 条目 1 | 新增 `TypeChecker::infer_lambda_return_type`（首个 `return` 的轻量推断），存入 `LambdaInfo::return_type`；`resolve_lambda` 的 `__call` 签名与 Pass 4 的 `FuncDecl` 共用 | `[](double x){ return x * 2.0; }` → `d=3.00` = Clang++（此前被当 int → E3062） |
| 条目 2 | Pass 2.5 改为**先定型再登记**：全局 `auto`/`typeof` 先解析初始化器并替换 `g.ty` 再 `declare_var`；解析结果缓存复用 | `auto gf = [](int x){ return x + 7; };` → `gg=8 6` = Clang++（此前 E3064→E3066） |

**回归**：新增 `native/tests/cpp_lambda_test.rs`（3 用例）；`CPP_FAILURES.md` 两项标记已修复 + 剩余限制
（多 `return` 合并、尾置返回类型 `-> T`）。

---

## 5. 状态汇总

| 批次 | 项 | 状态 |
|---|---|---|
| A | P0-1 / P1-5 / 条目 6 | ✅ 已修 |
| B | P2-7a / P2-7b / P2-7c + 1 项同源缺陷 | ✅ 已修 |
| C | P0-2 / P0-3 | ✅ 已修 |
| D | P0-4 + 1 项同源误判 | ✅ 已修 |
| E | P1-6 + 补 `CPP_SUBSET_SPEC` 记录 | ✅ 已修 |
| F | 条目 5（第三道墙用例）+ 3 项关联缺陷（步数保险丝失效 / 配置静默丢弃 / 无回显） | ✅ 已修 |
| G | 条目 3（scanf 返回值）/ 条目 4（scanf 普通字符指令）+ 标准输入换行口径统一 + Shadow `.in` 注入 | ✅ 已修 |
| H | 条目 1（lambda 返回类型推断）/ 2（文件作用域 lambda 变量） | ✅ 已修 |
