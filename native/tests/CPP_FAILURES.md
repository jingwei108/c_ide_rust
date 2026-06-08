# C++ 扩展测试失败记录

> 本文件记录 Cide C++ 扩展相关的已知失败与偏差。
>  philosophy: All in. Record don't hide. Fix real bugs, not test cases.

## 当前状态

截至 Stage 5（Dogfooding 基础设施）完成：

- `parser_cpp_unit_test.rs`: 15/15 通过
- `parser_cpp_unit_test.rs`: 17/17 通过（含 2 个新增初始化列表测试）
- `typeck_cpp_unit_test.rs`: 17/17 通过（含 Stage 4 引用 4 个）
- `bytecode_gen_cpp_unit_test.rs`: 33/33 通过（含 Stage 4 引用 4 个）
- `cpp_dogfooding_test.rs`: 7/7 通过（Stage 5 基础设施自验证 + Stage 6 `vector<int>` 首个 Dogfooding）
- **C++ 扩展合计: 74/74 通过**

**当前无已知失败。**

## 历史记录

### ~~Stage 0 容器预编译~~ → 已修复

- `vector<int>` / `vector<float>` / `string` / `list<int>` / `vector<char>` / `sort_int` 预编译通过
- 所有容器类型布局和类型映射已对齐

### ~~Stage 3 `new[]/delete[]` 元素构造析构~~ → 已修复

- `new A[n]` 在 `base[-4]` 存储元素个数，`delete[]` 逆序调用析构函数
- 修复 `get_temp_slot` 只有 3 个独立 slot 导致 `i_temp` 与 `user_ptr_temp` 冲突的 bug
- 扩展 `temp_slot0..3` 支持 4 个独立临时变量槽位
- 新增测试 `test_cpp_new_array_ctor_dtor`、`test_cpp_new_array_ctor_dtor_reverse_order`

## KNOWN_DIVERGENCE（设计决策导致的偏差）

无。

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
