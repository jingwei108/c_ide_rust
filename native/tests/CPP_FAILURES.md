# C++ 扩展测试失败记录

> 本文件记录 Cide C++ 扩展相关的已知失败与偏差。
>  philosophy: All in. Record don't hide. Fix real bugs, not test cases.

## 当前状态

截至 Stage 3（`new[]/delete[]` 元素构造析构）完成：

- `parser_cpp_unit_test.rs`: 15/15 通过
- `typeck_cpp_unit_test.rs`: 13/13 通过
- `bytecode_gen_cpp_unit_test.rs`: 29/29 通过
- **C++ 扩展合计: 57/57 通过**

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

## 待观察项

- `list_int` 无 `clear` 方法（C 实现未提供，不影响当前测试）
- `sort_int` 为自由函数，非容器方法，不经过 `cpp_container.rs` 降级路径
