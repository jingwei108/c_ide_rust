# 中等难度端到端测试补充 — 2026-05-11

## Batch 2：新增 5 个测试 + 修复 2 个 Bug

### 新增测试

在 `native/tests/end_to_end_extra_test.rs` 追加：

| 测试函数 | 覆盖场景 |
|---------|---------|
| `test_e2e_binary_search` | 有序数组二分查找：数组 + 函数 + 循环 + 多分支条件 |
| `test_e2e_string_reverse_inplace` | 原地字符串反转：数组遍历 + 双指针交换 + `char` 与 `int` 比较 |
| `test_e2e_array_stack` | 数组实现栈：`typedef struct` + 函数封装 + 指针传参 + 结构体数组成员索引赋值 |
| `test_e2e_selection_sort` | 选择排序：嵌套循环 + 条件交换 |
| `test_e2e_decimal_to_binary` | 十进制转二进制：位运算 `&` / `>>` 实际应用 + 数组暂存 |

### 修复的后端 Bug

#### Bug 1 — TypeChecker `is_comparable()` 遗漏 `Char`

**位置**：`native/src/compiler/type_checker.rs:202`

**现象**：`while (s[len] != 0)` 中 `char` 与 `int` 比较触发 **E3017** "类型不兼容，无法比较"。

**根因**：`is_comparable()` 仅接受 `Int | Float` 之间的比较，未包含 `Char`。C 语言中 `char` 会整型提升为 `int`，应允许与 `int` 比较。

**修复**：
```rust
// before
if matches!(a.kind, TypeKind::Int | TypeKind::Float) && matches!(b.kind, TypeKind::Int | TypeKind::Float) { return true; }
// after
if matches!(a.kind, TypeKind::Int | TypeKind::Char | TypeKind::Float) && matches!(b.kind, TypeKind::Int | TypeKind::Char | TypeKind::Float) { return true; }
```

#### Bug 2 — BytecodeGen 数组成员访问错误执行 `LoadMem`

**位置**：`native/src/compiler/bytecode_gen.rs:1037`

**现象**：`s->data[s->top] = x;`（`data` 为结构体内的数组成员）触发 **VM NULL 指针写入陷阱**，地址为 `0x0000`。

**根因**：`gen_expr` 处理 `Expr::Member` 时无条件执行 `LoadMem`：
- 对于普通标量成员（如 `s->top`），`LoadMem` 正确加载整数值。
- 对于数组成员（如 `s->data`），`gen_member_addr` 已生成数组基地址，此时应像数组变量一样退化为指针（不加载），但代码额外执行了 `LoadMem`，把 `data[0]` 的整数值当作地址留在栈上。后续索引计算用该整数作为基地址，极大概率得到 `0`，导致向 NULL 区域写入。

**修复**：成员类型为数组时跳过 `LoadMem`：
```rust
Expr::Member { object, member, ty, .. } => {
    self.gen_member_addr(object, member, &loc);
    if !ty.is_array() {
        self.emit(OpCode::LoadMem, 0, &loc);
    }
}
```

---

## Batch 3：新增 5 个测试（全部一次通过）

| 测试函数 | 覆盖场景 |
|---------|---------|
| `test_e2e_hanoi_recursive` | 汉诺塔：经典递归 + `char` 参数传递 + 多参数函数 |
| `test_e2e_pointer_sum_array` | 指针遍历求和：`*(arr + i)` 指针算术遍历数组 |
| `test_e2e_vowel_count` | 元音计数：字符串遍历 + `switch-case` + 函数 |
| `test_e2e_matrix_diagonal_sum` | 矩阵对角线：二维数组 + 双重索引 |
| `test_e2e_array_dedup` | 数组去重：排序 + 条件去重 + 指针修改长度 |

---

## Batch 4：新增 5 个测试 + 修复 2 个 Bug

### 新增测试

| 测试函数 | 覆盖场景 |
|---------|---------|
| `test_e2e_palindrome_string` | 回文判断：字符串字面量传参 + 双指针比较 |
| `test_e2e_matrix_add` | 矩阵相加：二维数组 + 嵌套循环 + 双重索引 |
| `test_e2e_my_strlen` | 自定义 strlen：字符串遍历 + 字符串字面量传参 |
| `test_e2e_max_subarray_sum` | 最大子数组和（Kadane）：三目运算符 + 负数数组初始化 |
| `test_e2e_simple_calc` | 简单计算器：多个小函数 + 整数运算 |

### 修复的后端 Bug

#### Bug 3 — TypeChecker `is_assignable()` 遗漏 `Array` 接受 `Pointer`

**位置**：`native/src/compiler/type_checker.rs:218`

**现象**：字符串字面量 `"hello"` 传给 `char s[]` 形参触发 **E3038** "函数参数类型不匹配"。

**根因**：Parser 将字符串字面量解析为 `Pointer(Char)`，而 `char s[]` 形参被解析为 `Array(Char, dims=[-1])`。`is_assignable()` 中：
- `target == value` 为 false（类型不同）
- `Pointer vs Array` 分支要求目标为 Pointer、值为 Array，方向相反，不匹配
- `Array vs Array` 分支要求两边都是 Array，但值是指针
- 其余分支均不匹配，最终返回 false

C 语言中函数参数 `char s[]` 等价于 `char *s`，字符串字面量退化为 `char *`，二者应兼容。

**修复**：在 `is_assignable()` 中新增 `Array` 目标接受 `Pointer` 值的分支：
```rust
if matches!(target.kind, TypeKind::Array) && matches!(value.kind, TypeKind::Pointer)
    && target.base_kind == value.base_kind {
    return true;
}
```

#### Bug 4 — BytecodeGen `flatten_init_list()` 不支持负数初始化

**位置**：`native/src/compiler/bytecode_gen.rs:1398`

**现象**：`int a[8] = {-2, 1, -3, 4, -1, 2, 1, -5};` 实际被初始化为 `{0, 1, 0, 4, 0, 2, 1, 0}`，导致最大子数组和算法输出错误值 `8` 而非 `6`。

**根因**：`flatten_init_list()` 对 `Expr::Unary { op: Neg, operand: Literal { 2 } }` 落入 `_ => result.push(0)` catch-all 分支，负数元素被替换为 `0`。

**修复**：在 `flatten_init_list()` 中显式处理 `UnaryOp::Neg`：
```rust
Expr::Unary { op: UnaryOp::Neg, operand, .. } => {
    if let Expr::Literal { value, .. } = operand.as_ref() {
        result.push(-*value);
    } else {
        result.push(0);
    }
}
```

## 回归验证

```
running 98 tests
test result: ok. 98 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
