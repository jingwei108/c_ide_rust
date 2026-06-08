# C++ 扩展测试失败记录

> 本文件记录 Cide C++ 扩展相关的已知失败与偏差。
>  philosophy: All in. Record don't hide. Fix real bugs, not test cases.

## 当前状态

截至 Stage 0.5（Phase 3 收口）：

- `bytecode_gen_cpp_unit_test.rs`: 17/17 通过
- `parser_cpp_unit_test.rs`: 全量通过
- `typeck_cpp_unit_test.rs`: 全量通过

**当前无已知失败。**

## 历史记录

### ~~Stage 0 容器预编译~~ → 已修复

- `vector<int>` / `vector<float>` / `string` / `list<int>` / `vector<char>` / `sort_int` 预编译通过
- 所有容器类型布局和类型映射已对齐

## KNOWN_DIVERGENCE（设计决策导致的偏差）

无。

## 待观察项

- `list_int` 无 `clear` 方法（C 实现未提供，不影响当前测试）
- `sort_int` 为自由函数，非容器方法，不经过 `cpp_container.rs` 降级路径
