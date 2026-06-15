# C++ 扩展 Stage 6 进入评估报告

> **评估日期**：2026-06-10  
> **评估人**：Kimi Code CLI  
> **对照文档**：`CPLUSPLUS_EXTENSION_PLAN.md`（v2.5，2026-06-08）、`tempest-green-lantern-superman.md`  
> **当前状态**：Stage 6 运行一致性验证已全部完成（vector<int> / list<int> / string）

---

## 一、评估结论

**Stage 6 运行一致性验证已全部通过，可以进入 Stage 2 替换决策流程。但存在 3 项技术债务，需在决策前同步补齐。**

---

## 二、Stage 6 完成标准逐项核对

依据 `CPLUSPLUS_EXTENSION_PLAN.md` + `tempest-green-lantern-superman.md` 的双轨标准：

| 验收项 | 标准来源 | 要求 | 实际状态 | 判定 |
|--------|---------|------|---------|------|
| `vector<int>` C++ 编译运行 | Stage 6 Week 1 | C++ 版编译通过，stdout 与 C 基线一致 | ✅ `test_cpp_vector_int_dogfooding_runs` 绿；stdout `3\n1\n4\n` | **通过** |
| `vector<int>` C 基线运行 | Stage 6 Week 1 | C 基线编译通过，stdout 一致 | ✅ `test_c_vector_int_baseline_runs` 绿 | **通过** |
| `list<int>` C++ 编译运行 | Stage 6 Week 2 | C++ 版编译通过，stdout 与 C 基线一致 | ✅ `test_cpp_list_int_dogfooding_runs` 绿；stdout `3\n0\n1\n2\n` | **通过** |
| `list<int>` C 基线运行 | Stage 6 Week 2 | C 基线编译通过，stdout 一致 | ✅ `test_c_list_int_baseline_runs` 绿 | **通过** |
| `string` C++ 编译运行 | Stage 6 Week 2 | C++ 版编译通过，stdout 与 C 基线一致 | ✅ `test_cpp_string_dogfooding_runs` 绿；stdout `5\nhello\n` | **通过** |
| `string` C 基线运行 | Stage 6 Week 2 | C 基线编译通过，stdout 一致 | ✅ `test_c_string_baseline_runs` 绿 | **通过** |
| 字节码语义等价 | `tempest-green-lantern-superman.md` | C++ 与 C 版本语义等价（运行 stdout 一致即接受） | ✅ 6/6 Dogfooding 运行测试全部 stdout 一致 | **通过** |
| 字节码逐指令一致 | `CPLUSPLUS_EXTENSION_PLAN.md` M9 | 字节码 C ≡ B（逐指令一致） | ⚠️ 未达成。算法差异（C++ `new[]/delete[]` + 循环复制 vs C `realloc`）导致 `push_back` 字节码天然不同；且当前测试框架无法比较跨编译单元（C 容器为预编译 libc 函数，不在 `func_table` 中） | **记录为已知偏差** |
| C++ 单元测试 | CI 三 tier | Parser/TypeChecker/BytecodeGen 全绿 | ✅ 30 + 26 + 36 = 92 个测试全绿 | **通过** |
| Dogfooding 基础设施 | Stage 5 | `compile_cpp_bytecode` + `assert_bytecode_equivalent` + harness | ✅ 4 个工具自验证测试全绿 | **通过** |
| 全量回归 | `AGENTS.md` 防线 | `cargo test` 全量无回归 | ✅ 607+ 测试通过；`bytecode_libc_consistency` 12 绿；`differential_stress` 18 绿 | **通过** |
| CI 三 tier | `ci_three_tier_check.py` | C++ 专项 tier 通过 | ✅ 全部通过 | **通过** |
| 失败记录诚实性 | `CPP_FAILURES.md` | 无未记录失败，禁止粉饰 | ⚠️ 文档记录不完整（见下文缺陷 1、2） | **需补齐** |

---

## 三、发现的技术债务（诚实记录）

### 缺陷 1：`cpp_dogfooding_test.rs` 中 C++ mangled 函数名错误，导致字节码比较测试永远 SKIP

- **位置**：`native/tests/cpp_dogfooding_test.rs` 第 190 行
- **问题**：`let cpp_get_name = "get__vector__int";`
- **实际 mangling 规则**（`typeck/mod.rs:275`）：`format!("{}__{}", c.name, name)`
- **正确值**：模板类 `vector<int>` 实例化后类名为 `vector__int`，方法 `get` 的 mangled 名应为 **`vector__int__get`**
- **后果**：`func_table.contains_key("get__vector__int")` 恒为 `false`，测试永远走 `SKIP` 分支，**从未真正执行过字节码比较**
- **修复动作**：修正 mangled 名，并验证 `get_function_instructions` 能正确提取指令切片

### 缺陷 2：`CPP_FAILURES.md` 未同步记录 `list<int>` / `string` 的 Dogfooding 状态

- **位置**：`native/tests/CPP_FAILURES.md`
- **问题**：文件当前仅记录 `vector<int>` 字节码差异，未提及 `list<int>` 和 `string` 的 Dogfooding 已通过
- **也未记录**：`test_cpp_vector_int_get_bytecode_comparison` 因 mangled 名错误而 SKIP 的事实
- **后果**：与 "All in. Record don't hide." 哲学存在偏差
- **修复动作**：更新 `CPP_FAILURES.md`，补充上述记录

### 缺陷 3：C 预编译容器函数不在 `func_table` 中，字节码比较架构受限

- **位置**：`native/tests/test_utils.rs` + `native/tests/cpp_dogfooding_test.rs`
- **问题**：C 基线使用 `cide_vec_get_int` 等预编译 libc 函数，`compile_cpp_bytecode` 只返回当前编译单元的 `CompileOutput`，预编译函数的指令序列不在其中
- **后果**：跨编译单元的字节码一致验证在现有架构下不可行
- **修复动作**：在 `CPP_FAILURES.md` 中明确标注此架构限制，并评估是否需要在 `test_utils.rs` 中加载预编译字节码数据以支持完整比较

---

## 四、"不因缺陷扭曲代码"原则遵循情况

| 铁律 | 遵循度 | 说明 |
|------|--------|------|
| **零改动 VM** | ✅ 100% | VM 未做任何修改 |
| **不复用 Clang** | ✅ 100% | 完全自研 |
| **BytecodeGen 线性扩展** | ✅ 95% | 新增 C++ 节点各有专用文件 |
| **诊断体系保护** | ✅ 100% | E4001-E4999 预留，C 错误码无影响 |
| **狗吃自己狗粮** | ✅ 85% | vector/list/string 均用 Cide C++ 编译器编译，未为迎合编译器而改写算法 |
| **诚实记录** | ⚠️ 75% | `CPP_FAILURES.md` 未及时同步；字节码比较测试以 SKIP 隐藏 mangled 名错误 |

---

## 五、Go / No-Go 决策建议

### 建议：Go（有条件通过）

Stage 6 的核心验收标准——**运行一致性**——已全部达成。`vector<int>` / `list<int>` / `string` 三个容器的 C++ 版本均能用 Cide C++ 编译器正确编译，且 stdout 与 C 基线逐字节一致。这证明编译器已具备编译真实 C++ 容器代码的能力，满足进入 Stage 2 决策的最低门槛。

### Stage 2 决策会议前必须先完成的修补（预计 1 人日）

1. **修正 `cpp_dogfooding_test.rs` 第 190 行**：将 `"get__vector__int"` 改为 `"vector__int__get"`，使字节码比较测试真正运行
2. **更新 `CPP_FAILURES.md`**：
   - 记录 `list<int>` / `string` Dogfooding 已通过
   - 记录 `vector<int>` `get` 方法字节码比较因 mangled 名错误而 SKIP（修正后更新状态）
   - 明确记录"跨编译单元预编译函数字节码不可比较"的架构限制
3. **重跑 `cargo test --test cpp_dogfooding_test`**，确认修正后测试行为符合预期

### Stage 2 决策会议议题

- 是否接受"语义等价（stdout 一致）"替代"逐指令一致"作为 Dogfooding 最终验收标准？
- 若接受，制定 C 容器退役时间表（建议分阶段：先 `vector<int>`，后 `list` / `string`）

---

> **评估依据**：`CPLUSPLUS_EXTENSION_PLAN.md` v2.5、`tempest-green-lantern-superman.md`、现有 `CPP_EXTENSION_ENTRY_S6_AUDIT.md`、`CPP_FAILURES.md`、以及 `native/tests/*.rs` 代码事实。
