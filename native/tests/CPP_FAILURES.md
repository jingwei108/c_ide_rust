# C++ 扩展测试失败记录

> 本文件记录 Cide C++ 扩展相关的已知失败与偏差。
>  philosophy: All in. Record don't hide. Fix real bugs, not test cases.

## 当前状态

截至 Phase 42 维护计划任务 G 推进后：

- `parser_cpp_unit_test.rs`: 33/33 通过
- `typeck_cpp_unit_test.rs`: 28/28 通过
- `bytecode_gen_cpp_unit_test.rs`: 38/38 通过
- `cpp_dogfooding_test.rs`: 全部通过（含 Stage 5 基础设施自验证 + Stage 6 `vector<int>` / `list<int>` / `string` Dogfooding）
- **C++ E2E 回归：`native/tests/cases/cpp/` 74 个用例全部通过，`KNOWN_CPP_FAILURES` 为空**
- **C++ 扩展合计: 175/175 通过**（99 单元测试 + 2 个 E2E 监控测试 + 74 个 E2E 实际用例；Dogfooding 测试另行统计）

### M6 E2E 回归覆盖

新增 `native/tests/cases/cpp/` 目录，包含 61 个自包含 C++14 用例，覆盖：

| 类别 | 用例数 | 关键特性 |
|---|---|---|
| 核心语言 | 18 | class / ctor / dtor / 引用 / auto / 范围 for / 模板 / 虚函数 / this / 方法重载 / unique_ptr |
| 容器与算法 | 15 | 自实现 vector<int/float/char> / list<int> / string / 排序 / 栈 / 队列 / 链表 / 二叉树 |
| 教学/OJ 题目 | 28 | Two Sum / 去重 / 移除元素 / 二分 / 最大子数组 / 股票 / 单数 / 多数 / 旋转 / 移动零 / 回文 / 括号 / 反转链表 / 合并链表 / 树深度 / 相同树 / 翻转树 / 爬楼梯 / 帕斯卡 / 平方根 / 罗马数字 / 缺失数字 / 公共前缀 / 首个唯一字符 |

### 维护计划任务 G 补充用例（10 个）

| 类别 | 用例数 | 新增用例 |
|---|---|---|
| 核心语言 | 6 | `cpp_pair_template` / `cpp_template_func_multi` / `cpp_reference_member` / `cpp_reference_param_chain` / `cpp_ctor_init_list` / `cpp_function_overload_template` |
| 容器与算法 | 4 | `cpp_template_array` / `cpp_template_stack` / `cpp_unique_ptr_reset` / `cpp_class_array` |

### 本次新增用例

| 类别 | 用例数 | 新增用例 |
|---|---|---|
| 容器与算法 | 2 | `cpp_cide_vec_class` — 验证 `cide_vec<T>` 对 class 类型模板实参的支持（push_back / get / 自动构造析构）；`cpp_cide_list_class` — 验证 `cide_list<T>` 对 class 类型模板实参的支持（push_back / get / 自动构造析构） |
| 核心语言 | 1 | `cpp_const_reference_param` — 验证 `const T&` 参数可绑定到字面量、变量与表达式右值 |

> `cpp_cide_vec_class` / `cpp_cide_list_class` 使用 Cide 内置容器 `cide_vec<T>` / `cide_list<T>`，Clang++ 无法直接编译。影子验证脚本通过文件首行 `// category: gap` 将其标记为预期差异。

Golden 全部由 Clang++ (`-std=c++14 -O0`) 生成，Cide 输出与之逐行对比。

### Stage 6~1 Dogfooding 状态

Dogfooding 详细状态已迁移至 **`DOGFOODING_FAILURES.md`**。

| 容器/算法 | C++ 编译 | C 基线 | stdout 一致性 | 字节码等价 |
|-----------|---------|--------|--------------|-----------|
| `vector<int>` | ✅ | ✅ | ✅ | ✅ |
| `vector<float>` | ✅ | ✅ | ✅ | ✅ |
| `vector<char>` | ✅ | ✅ | ✅ | ✅ |
| `list<int>` | ✅ | ✅ | ✅ | size ✅ / get ⚠️ |
| `string` | ✅ | ✅ | ✅ | ✅ |
| `sort_int` | ✅ | ✅ | ✅ | — |

**当前无运行失败。**

## 历史记录

### ~~`cpp_dogfooding_test.rs` mangled 函数名错误导致字节码比较测试永远 SKIP~~ → 已修复（2026-06-10）

详细记录见 `DOGFOODING_FAILURES.md`。

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

C++ Dogfooding 的详细 KNOWN_DIVERGENCE 记录已迁移至 **`DOGFOODING_FAILURES.md`**。

### C++ `vector<int>` 与 C `cide_vec_int` 的 `push_back` 字节码差异（算法差异）

- **根因**：C++ 版使用 `new[]/delete[]` + 循环复制，C 版使用 `realloc`
- **判定**：实现方式不同而非编译器缺陷，以运行 stdout 一致性为验收标准

## Dogfooding 过程中发现的语言子集边界

- ~~**构造函数成员初始化列表 `Class() : field(val) {}` 暂不支持**~~ → **已修复（2026-06-08）**
  - Parser 在两个构造函数分支（无返回类型 + 有返回类型）的 `RParen` 后增加 `parse_ctor_init_list()`
  - 初始化列表被降解为构造函数体内 `this->field = expr;` 赋值语句，插入到 `Block` 开头
  - 新增白盒测试 `test_parser_cpp_ctor_init_list`、`test_parser_cpp_ctor_init_list_with_body`
  - Dogfooding `vector<int>` 已恢复为标准初始化列表写法

## M6 过程中识别的 Cide C++ 子集边界（已全部消除）

M6 阶段记录的 10 项 C++ 子集边界已全部在后续迭代中修复：

| # | 边界 | 修复要点 |
|---|---|---|
| 1 | 类字段逗号多声明 | `parse_class_decl_inner` 支持循环解析多个声明符 |
| 2 | 多维数组 `{0}` 初始化 | `validate_nested_init_list` 特殊处理单元素扁平初始化列表 |
| 3 | 指针逻辑运算 `&&` / `||` | `typeck/expr.rs` 的 `And`/`Or` 分支接受指针/数组类型 |
| 4 | 模板类 `Class<T>&` 自引用参数 | Parser 将类模板名加入 `template_names`，单态化替换裸 `Type::Class` |
| 5 | 引用参数访问同类私有成员 | `Member`/`MemberCall` 解引用 `Reference`/`RValueRef` 基底 |
| 6/7 | 方法返回引用作为左值 / 返回 `*this` | `gen_assign` 支持引用返回的调用，类型系统识别引用返回左值 |
| 8 | 类内方法重载与递归调用 | `ClassSymbol::methods` 改为 `HashMap<String, Vec<MethodSig>>`，实现重载决议与 `Class__method__N` mangling |
| 9 | `printf("%.1f")` 浮点精度 | VM 格式解析已支持精度，`cpp_vector_float.cpp` 使用标准写法 |
| 10 | 字符字面量 `'\0'` | Lexer 已支持转义，`cpp_string_basic.cpp` 使用标准写法 |

> 现在 `native/tests/cases/cpp/` 的 61 个用例全部使用标准 C++14 语法编写，无需为 Cide 做额外规避。`KNOWN_CPP_FAILURES` 仍为空。

## 待观察项

- `list_int` 无 `clear` 方法（C 实现未提供，不影响当前测试）
- `sort_int` 为自由函数，非容器方法，不经过 `cpp_container.rs` 降级路径
- C++ `vector<int>` 与 C `cide_vec_int` 的 `push_back` 字节码差异：C++ 版使用 `new[]/delete[]` + 循环复制，C 版使用 `realloc`。算法差异导致字节码不一致，但这属于实现方式不同而非编译器缺陷。以运行 stdout 一致性为首要验收标准。
