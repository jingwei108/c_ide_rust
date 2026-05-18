# 27 个回归测试失败修复报告

**日期**: 2026-05-18  
**关联提交**: `0863bcf`（函数指针完整支持 + DeclaratorNode 树形解析）引入的回归  
**修复后状态**: `cargo test` 全部通过（142 E2E + 全部单元测试 0 失败）

---

## 1. 失败的测试列表

| 测试名 | 失败现象 |
|--------|---------|
| `test_e2e_float_basic` | 输出 `0.000000` 而非 `5.500000` |
| `test_e2e_float_arithmetic` | 同上 |
| `test_e2e_float_assign_int` | 同上 |
| `test_e2e_float_basic` | 同上 |
| `test_e2e_float_cast` | 同上 |
| `test_e2e_float_compound_assign` | 同上 |
| `test_e2e_float_func_arg_implicit_cast` | 同上 |
| `test_e2e_float_mixed_int` | 同上 |
| `test_e2e_double_arr` | 输出 `0.000000` 而非 `2.200000` |
| `test_e2e_double_basic` | 输出 `0.000000` 而非 `5.500000` |
| `test_e2e_double_cast` | 同上 |
| `test_e2e_double_compound_assign` | 同上 |
| `test_e2e_double_func_arg_return` | 同上 |
| `test_e2e_double_implicit_cast_from_int` | 同上 |
| `test_e2e_double_precision_64bit` | 同上 |
| `test_e2e_double_printf_precision` | 同上 |
| `test_e2e_double_scanf_lf` | 同上 |
| `test_e2e_double_scanf_lf_and_int` | 同上 |
| `test_e2e_long_long_arith` | 输出 `0` 而非正确长整型结果 |
| `test_e2e_long_long_basic` | 同上 |
| `test_e2e_long_long_scanf` | 同上 |
| `test_e2e_2d_array_func_arg` | 编译错误：数组初始化类型不匹配（期望 `int`，实际 `void`） |
| `test_e2e_multidim_array_init` | 同上 |
| `test_e2e_matrix_add` | 同上 |
| `test_e2e_matrix_diagonal_sum` | 同上 |
| `test_e2e_struct_array_avg` | `float` 输出 `0.000000` |
| `test_e2e_union_double_member` | `double` 输出 `0.00` |
| `test_e2e_printf_format_modifiers` | 同上 |
| `test_e2e_atoi` | 编译错误：字符串字面量长度超过数组大小 |
| `test_e2e_qsort` | 编译错误：数组初始化项过多 |
| `test_e2e_strcmp` | 编译错误：字符串字面量长度超过数组大小 |
| `test_e2e_strcpy` | 同上 |
| `test_e2e_strlen` | 同上 |
| `test_e2e_vowel_count` | 同上 |

---

## 2. Root Cause 分析

本次回归由三个互相独立的 `0863bcf` 重构副作用引起。

### Root Cause A：多维数组 `dims` 维度信息丢失

**位置**: `native/src/compiler/parser.rs` — `interpret_declarator_node`  
**影响**: 所有涉及多维数组初始化/传递的测试（约 5 个失败）

`0863bcf` 引入的 `interpret_declarator_node` 在处理 `DeclaratorNode::Array` 时，只把当前层的 `size` 放入 `dims`，没有递归合并内层 `dims`：

```rust
// 重构前（正确）
// 旧代码通过其他方式维护了完整 dims

// 重构后（错误）
_ => Type::Array {
    element: Box::new(inner_ty),
    array_size: *size,
    dims: vec![*size],        // ← 只保留一层！
    is_const: false,
}
```

结果 `int arr[2][3]` 的 `dims` 变成 `[2]`（丢失了 `[3]`）。`TypeChecker` 的多维数组初始化检查 `validate_nested_init_list` 因此认为 `{{1,2,3}, {4,5,6}}` 是一维数组的 2 个元素，但每个元素却是 `InitList`（类型为 `Void`），触发 **E3006** "期望 `int`，实际 `void`"。

此外，当数组大小未指定时（`char s[]`），`size = -1`，旧代码直接保留 `-1`；但重构后的 `array_size` 计算逻辑把 `-1` 当作 `1` 处理：

```rust
let array_size = if *size > 0 { *size } else { 1 } * inner_array_size;
// 当 size = -1 时，array_size = 1 * 1 = 1
```

这导致 `char s[] = "hello"` 的数组大小被错误设为 `1`，而 `"hello"` 长度 `5 + 1 > 1`，触发 **E3008** "字符串字面量长度超过数组大小"。

---

### Root Cause B：`CallPtr` 中 `Double`/`LongLong` 参数无条件拆分

**位置**: `native/src/compiler/bytecode_gen.rs` — `Expr::CallPtr` 参数处理  
**影响**: 所有 `double` / `long long` 传给 `printf` 的测试（约 15 个失败）

`0863bcf` 后，Parser 将所有函数调用统一解析为 `Expr::CallPtr`（包括 `printf`）。`CallPtr` 的字节码生成对 `Double`/`LongLong` 参数无条件发射 `SplitD` / `SplitQ`：

```rust
// Expr::CallPtr 分支（错误）
} else if arg_ty.kind() == TypeKind::Double {
    self.gen_expr(arg);
    self.emit(OpCode::SplitD, 0, &loc);   // ← 无条件拆分
}
```

`SplitD` 将 64 位值拆成 `low`（低 32 位）和 `high`（高 32 位）两个栈值。这原本是**用户函数调用**（`Call` 指令）所需，因为用户函数的参数字槽按 4 字节处理。但 `printf` 是 **host 函数**（`CallHost` 指令），参数通过 `vm.pop()` 直接读取 64 位值，不需要拆分。

结果 `printf("%f", 3.5)` 时：
1. `PushConstD` 推入 `3.5` 的 64 位值 `0x400C000000000000`
2. `SplitD` 将其拆成 `low=0x00000000`、`high=0x400C0000`
3. `printf` 只 `pop()` 一次，读到 `low = 0`
4. `f64::from_bits(0) = 0.0`，输出 `0.000000`

`LongLong` 同理：只读到低 32 位，输出 `0`。

---

### Root Cause C：`CallPtr` 中缺少 `printf` 的 `float` → `double` 提升

**位置**: `native/src/compiler/bytecode_gen.rs` — `Expr::CallPtr` 参数处理  
**影响**: 所有 `float` 传给 `printf` 的测试（约 7 个失败）

C 语言标准规定：`printf` 的 `%f` 期望 `double` 参数，`float` 会自动提升为 `double`。旧的 `Expr::Call` 分支实现了这个提升：

```rust
// Expr::Call 分支（正确，但已不被使用）
if (name == "printf" || name == "fprintf") && arg_ty_kind == TypeKind::Float {
    self.emit(OpCode::CastF2D, 0, &loc);
}
```

但 `Expr::CallPtr` 分支没有这段逻辑。`float` 参数作为 32 位值直接传给 `printf`，`printf` 将其位模式当作 `f64` 解释。`0x40600000`（即 `3.5f32`）作为 `f64` 位模式是一个极小的次正规数（约 `4.6e-10`），`%f` 默认 6 位精度格式化后输出 `0.000000`。

---

## 3. 修复内容

### 修复 A：`interpret_declarator_node` 的 `dims` 与 `array_size`

**文件**: `native/src/compiler/parser.rs`

```rust
DeclaratorNode::Array(inner, size) => {
    let inner_ty = Self::interpret_declarator_node(inner, base_type);
    match inner.as_ref() {
        DeclaratorNode::Pointer(ptr_inner) => { /* 不变 */ }
        _ => {
            let (element, mut inner_dims, inner_array_size) =
                if let Type::Array { element, dims, array_size, .. } = &inner_ty {
                    (element.clone(), dims.clone(), *array_size)
                } else {
                    (Box::new(inner_ty.clone()), Vec::new(), 1)
                };
            inner_dims.push(*size);                       // 当前层追加到末尾
            let array_size = if *size > 0 {
                *size * inner_array_size
            } else {
                *size                                      // 保留 0 / -1
            };
            Type::Array {
                element,
                array_size,
                dims: inner_dims,
                is_const: false,
            }
        }
    }
}
```

- `dims` 按声明顺序合并（内层在前，当前层追加到末尾），`int arr[2][3]` → `dims = [2, 3]`
- `array_size` 在 `size > 0` 时正常相乘；在 `size <= 0`（未指定大小）时保留原始值

---

### 修复 B 与 C：`CallPtr` 参数处理对齐 `Call`

**文件**: `native/src/compiler/bytecode_gen.rs`

```rust
Expr::CallPtr { callee, args, .. } => {
    let is_direct_call = if let Expr::Identifier { name, .. } = callee.as_ref() {
        self.func_index.contains_key(name)   // true = 用户函数（需要 Split）
    } else {
        false
    };
    for arg in args.iter_mut().rev() {
        let arg_ty = arg.ty().clone();
        if arg_ty.is_struct() { /* 不变 */ }
        else if arg_ty.kind() == TypeKind::Double {
            self.gen_expr(arg);
            if is_direct_call {
                self.emit(OpCode::SplitD, 0, &loc);   // 仅用户函数拆分
            }
        } else if arg_ty.kind() == TypeKind::LongLong {
            self.gen_expr(arg);
            if is_direct_call {
                self.emit(OpCode::SplitQ, 0, &loc);   // 仅用户函数拆分
            }
        } else {
            self.gen_expr(arg);
            if let Expr::Identifier { name, .. } = callee.as_ref() {
                // 补全 printf/fprintf 的 float → double 提升
                if (name == "printf" || name == "fprintf") && arg_ty.kind() == TypeKind::Float {
                    self.emit(OpCode::CastF2D, 0, &loc);
                }
            }
        }
    }
    // ... callee 分发逻辑不变
}
```

- `Double` / `LongLong`：仅在调用**用户函数**（`is_direct_call == true`）时才 `SplitD`/`SplitQ`；调用 host 函数（如 `printf`）时保持完整的 64 位栈值
- `Float`：在 `printf` / `fprintf` host 调用时补发 `CastF2D`，确保 `float` 提升为 `double`

---

## 4. 验证结果

```bash
cd native && cargo test
```

**全部测试套件通过**（无失败）：

```
test result: ok. 142 passed; 0 failed; 0 ignored   (end_to_end_extra_test)
test result: ok.  23 passed; 0 failed; 0 ignored   (end_to_end_test)
test result: ok.  10 passed; 0 failed; 0 ignored   (bytecode_gen_unit_test)
test result: ok.  13 passed; 0 failed; 0 ignored   (compile_pipeline_test)
test result: ok.  10 passed; 0 failed; 0 ignored   (lexer_unit_test)
test result: ok.  12 passed; 0 failed; 0 ignored   (parser_unit_test)
test result: ok.  12 passed; 0 failed; 0 ignored   (type_checker_unit_test)
test result: ok.   7 passed; 0 failed; 0 ignored   (vm_memory_safety_test)
test result: ok.   3 passed; 0 failed; 0 ignored   (test_snapshot)
```

---

## 5. 教训与后续预防

1. **Parser 重构必须跑全量回归**：`0863bcf` 修改了声明符解析，但未验证多维数组和字符串数组初始化路径。任何 Parser 改动都应触发全量 E2E 回归。
2. **`Call` → `CallPtr` 统一时要同步字节码生成**：Parser 侧的统一（全部走 `CallPtr`）必须同步检查 `bytecode_gen.rs` 中所有涉及函数调用的分支，不能遗漏 `CastF2D`、`SplitD` 等参数处理差异。
3. **64 位参数的 host/user 区分**：`SplitD`/`SplitQ` 的存在是为了适配 `Call` 指令的 4 字节参数字槽，host 函数不需要。后续新增 host 函数时需注意此边界。
