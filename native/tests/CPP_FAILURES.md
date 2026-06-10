# C++ 扩展测试失败记录

> 本文件记录 Cide C++ 扩展相关的已知失败与偏差。
>  philosophy: All in. Record don't hide. Fix real bugs, not test cases.

## 当前状态

截至 Stage 6（Dogfooding 运行一致性验证）完成：

- `parser_cpp_unit_test.rs`: 30/30 通过
- `typeck_cpp_unit_test.rs`: 26/26 通过
- `bytecode_gen_cpp_unit_test.rs`: 36/36 通过
- `cpp_dogfooding_test.rs`: 全部通过（含 Stage 5 基础设施自验证 + Stage 6 `vector<int>` / `list<int>` / `string` Dogfooding）
- **C++ 扩展合计: 92/92 通过**

### Stage 6 Dogfooding 状态

| 容器 | C++ 编译 | C 基线 | stdout 一致性 | 字节码比较 |
|------|---------|--------|--------------|-----------|
| `vector<int>` | ✅ | ✅ | ✅ | ⚠️ 因架构限制不可比较（见下文 KNOWN_DIVERGENCE） |
| `list<int>` | ✅ | ✅ | ✅ | ⚠️ 因架构限制不可比较 |
| `string` | ✅ | ✅ | ✅ | ⚠️ 因架构限制不可比较 |

**当前无运行失败。**

## 历史记录

### ~~`cpp_dogfooding_test.rs` mangled 函数名错误导致字节码比较测试永远 SKIP~~ → 已修复（2026-06-10）

- **位置**：`native/tests/cpp_dogfooding_test.rs` 第 190 行
- **问题**：`let cpp_get_name = "get__vector__int";`（错误）
- **正确值**：`"vector__int__get"`（mangling 规则为 `format!("{}__{}", class_name, method_name)`，`vector<int>` 实例化后类名为 `vector__int`）
- **后果**：`func_table.contains_key("get__vector__int")` 恒为 `false`，测试永远走 `SKIP` 分支，从未真正执行过字节码比较
- **修复**：修正为 `"vector__int__get"`，重跑测试确认通过

### ~~Stage 0 容器预编译~~ → 已修复

- `vector<int>` / `vector<float>` / `string` / `list<int>` / `vector<char>` / `sort_int` 预编译通过
- 所有容器类型布局和类型映射已对齐

### ~~Stage 3 `new[]/delete[]` 元素构造析构~~ → 已修复

- `new A[n]` 在 `base[-4]` 存储元素个数，`delete[]` 逆序调用析构函数
- 修复 `get_temp_slot` 只有 3 个独立 slot 导致 `i_temp` 与 `user_ptr_temp` 冲突的 bug
- 扩展 `temp_slot0..3` 支持 4 个独立临时变量槽位
- 新增测试 `test_cpp_new_array_ctor_dtor`、`test_cpp_new_array_ctor_dtor_reverse_order`

## KNOWN_DIVERGENCE（设计决策导致的偏差）

### 跨编译单元预编译函数字节码不可比较（架构限制）

- **影响范围**：`test_cpp_vector_int_get_bytecode_comparison` 及同类字节码比较测试
- **根因**：C 基线容器（`cide_vec_get_int` 等）为预编译 Bytecode Libc 函数，存储在固定索引段中；`compile_cpp_bytecode` 仅返回当前编译单元的 `CompileOutput`，预编译函数的指令序列不在 `func_table` 中
- **后果**：C++ 与 C 版本的逐指令一致验证在现有测试架构下不可行
- **缓解**：以运行 stdout 语义等价作为首要验收标准；字节码比较仅对同一编译单元内的函数有意义

### C++ `vector<int>` 与 C `cide_vec_int` 的 `push_back` 字节码差异（算法差异）

- **根因**：C++ 版使用 `new[]/delete[]` + 循环复制，C 版使用 `realloc`
- **判定**：实现方式不同而非编译器缺陷，以运行 stdout 一致性为验收标准

## Dogfooding 过程中发现的语言子集边界

- ~~**构造函数成员初始化列表 `Class() : field(val) {}` 暂不支持**~~ → **已修复（2026-06-08）**
  - Parser 在两个构造函数分支（无返回类型 + 有返回类型）的 `RParen` 后增加 `parse_ctor_init_list()`
  - 初始化列表被降解为构造函数体内 `this->field = expr;` 赋值语句，插入到 `Block` 开头
  - 新增白盒测试 `test_parser_cpp_ctor_init_list`、`test_parser_cpp_ctor_init_list_with_body`
  - Dogfooding `vector<int>` 已恢复为标准初始化列表写法

## 待观察项

- `list_int` 无 `clear` 方法（C 实现未提供，不影响当前测试）
- `sort_int` 为自由函数，非容器方法，不经过 `cpp_container.rs` 降级路径
- C++ `vector<int>` 与 C `cide_vec_int` 的 `push_back` 字节码差异：C++ 版使用 `new[]/delete[]` + 循环复制，C 版使用 `realloc`。算法差异导致字节码不一致，但这属于实现方式不同而非编译器缺陷。以运行 stdout 一致性为首要验收标准。
