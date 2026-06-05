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

---

## 附录：4 个高级函数指针 E2E 测试修复（后续追加）

**日期**: 2026-05-18（同日追加）  
**关联提交**: `90f546a`（Type 递归化 + DeclaratorGuard + 27 个回归测试修复）之后  
**修复后状态**: `cargo test --test end_to_end_extra_test` 147 passed; 0 failed

### 测试列表

| 测试名 | 失败现象 |
|--------|---------|
| `test_e2e_pointer_to_function_pointer` | 运行时 `CallPtr: 未知函数索引 0`；TypeChecker 将 `int (**fp)(int)` 推导为"返回 `int*` 的函数指针" |
| `test_e2e_function_pointer_returning_pointer` | Parser 不支持 `static` 局部变量；`static int arr[3]` 在函数体内被当作表达式语句解析，报"预期 ';'" |
| `test_e2e_sizeof_function_pointer_types` | `sizeof(int (*)(int))` 中 `int (*)(int)` 被当作表达式解析，产生大量语法错误（"预期表达式"、"预期 ')'"） |
| `test_e2e_typedef_function_pointer_array` | `typedef int (*Op)(int, int);` 报错"typedef 后预期标识符名称"；`parse_typedef` 不支持带声明符的复杂类型 |

### 根因分析

1. **`interpret_declarator_node` 的 `Function` 分支对多级指针处理错误**
   - 原始代码：`Function(Pointer(ptr_inner), params)` 中把 `ptr_inner` 的推导结果当作**函数返回类型**。
   - 例：`int (**pp)(int)` → node = `Function(Pointer(Pointer(Base)), [int])` → `ptr_inner = Pointer(Base)` → `return_ty = interpret(Pointer(Base), Int) = Pointer { pointee: Int }` → 被当作返回类型，得到 `Pointer { pointee: Function { return_type: Pointer { pointee: Int }, ... } }`（即 `int *(*)(int)`）。
   - 正确语义：`ptr_inner` 应被递归解释为"以函数指针为基础类型的声明符"，得到 `Pointer { pointee: Pointer { pointee: Function { return_type: Int, ... } } }`。

2. **Parser 不支持 `static` 存储类说明符**
   - `static` 未被加入 `TokenType` 枚举，Lexer 将其作为普通 `Identifier` 输出。
   - `parse_statement` 的 `is_type_token()` 分支无法匹配 `static`，导致进入 `parse_expr_stmt`，`static` 被当作变量名处理。

3. **`parse_sizeof` 只支持基础类型 + 单级指针**
   - 遇到 `sizeof(int (*)(int))` 时，`parse_base_type()` 消费 `int` 后，`match_token(Star)` 失败（当前是 `(`），且 `check(RParen)` 失败，于是回退到 `sizeof(expr)` 路径。
   - `int (*)(int)` 作为表达式完全不合法，导致级联错误。

4. **`parse_typedef` 使用 `parse_type_only()`（仅 base type + 可选单 `*`）**
   - 遇到 `typedef int (*Op)(int, int);` 时，`parse_type_only()` 只消费 `int`，遇到 `(` 停止。
   - 随后 `consume(Identifier)` 发现当前 token 是 `(`，报错。

### 修复方案

#### 1. `interpret_declarator_node` 的 `Function` 分支递归化

```rust
DeclaratorNode::Function(inner, params) => {
    match inner.as_ref() {
        DeclaratorNode::Pointer(ptr_inner) => {
            match ptr_inner.as_ref() {
                DeclaratorNode::Array(array_inner, size) => {
                    // (*fp[N])(params) → function pointer array
                    let elem_ty = Self::interpret_declarator_node(array_inner, base_type);
                    Type::Array { element: ..., array_size: *size, ... }
                }
                _ => {
                    let func_ptr_type = Type::Pointer {
                        pointee: Box::new(Type::Function {
                            return_type: Box::new(base_type.clone()),
                            param_types: params.iter().map(|p| p.ty.clone()).collect(),
                            is_const: false,
                        }),
                        is_const: false,
                    };
                    Self::interpret_declarator_node(ptr_inner, &func_ptr_type)
                }
            }
        }
        _ => {
            let inner_ty = Self::interpret_declarator_node(inner, base_type);
            Type::Pointer {
                pointee: Box::new(Type::Function {
                    return_type: Box::new(inner_ty),
                    param_types: params.iter().map(|p| p.ty.clone()).collect(),
                    is_const: false,
                }),
                is_const: false,
            }
        }
    }
}
```

核心改动：`Pointer` case 的 `_` 子分支不再把 `ptr_inner` 当作返回类型，而是构造一个 `func_ptr_type`（函数指针基础类型），然后递归调用 `interpret_declarator_node(ptr_inner, &func_ptr_type)`。这样：
- `ptr_inner = Base` → 返回 `func_ptr_type`（普通函数指针）
- `ptr_inner = Pointer(Base)` → 返回 `Pointer { pointee: func_ptr_type }`（指向函数指针的指针）
- `ptr_inner = Array(Base, N)` → 走 `Pointer` 分支的 `Array` case → `Array { element: Pointer { pointee: func_ptr_type }, size: N }`（函数指针数组）

#### 2. `parse_statement` 支持 `static` 局部声明

```rust
fn is_static_token(&self) -> bool {
    self.check(TokenType::Identifier) && self.current().text == "static"
}

// parse_statement:
_ if self.is_type_token() || self.is_static_token() => {
    if self.is_static_token() {
        self.advance(); // skip 'static'
    }
    self.parse_var_decl_stmt()
}
```

`static` 不改变类型语义，仅作为存储类说明符被跳过。

#### 3. 新增 `parse_abstract_declarator()`

抽象声明符 = 声明符去掉标识符名。用于 `sizeof(type)` 和强制类型转换 `(type)expr`。

```rust
fn parse_abstract_declarator(&mut self) -> Option<DeclaratorNode> {
    let mut ptr_prefixes = 0;
    while self.match_token(TokenType::Star) { ptr_prefixes += 1; }

    let mut node = if self.match_token(TokenType::LParen) {
        if let Some(inner) = self.parse_abstract_declarator() {
            self.consume(TokenType::RParen, "预期 ')'");
            inner
        } else {
            self.consume(TokenType::RParen, "预期 ')'");
            DeclaratorNode::Base
        }
    } else { DeclaratorNode::Base };

    // 收集后缀 [] / ()
    let mut suffixes = Vec::new();
    loop {
        if self.match_token(TokenType::LBracket) {
            let size = ...;
            self.consume(TokenType::RBracket, "预期 ']'");
            suffixes.push(DeclaratorSuffix::Array(size));
        } else if self.match_token(TokenType::LParen) {
            let params = self.parse_param_list();
            self.consume(TokenType::RParen, "预期 ')'");
            suffixes.push(DeclaratorSuffix::Function(params));
        } else { break; }
    }

    // 应用后缀 → 前缀
    for suffix in suffixes { ... }
    for _ in 0..ptr_prefixes { node = DeclaratorNode::Pointer(Box::new(node)); }

    if ptr_prefixes == 0 && matches!(node, DeclaratorNode::Base) { None } else { Some(node) }
}
```

`parse_sizeof` 修改：
```rust
if self.is_type_token() {
    t = self.parse_base_type();
    if let Some(node) = self.parse_abstract_declarator() {
        t = Self::interpret_declarator_node(&node, &t);
    }
    if self.check(TokenType::RParen) { is_type = true; }
}
```

#### 4. `parse_typedef` 改用完整声明符解析

```rust
fn parse_typedef(&mut self) {
    self.advance(); // consume 'typedef'
    let base_type = self.parse_base_type();
    let (ty, name) = self.parse_declarator(&base_type);
    self.consume(TokenType::Semicolon, "typedef 后预期 ';'");
    self.typedef_names.insert(name, ty);
}
```

### 回归修复

在修复 `bytecode_gen.rs` 以支持函数指针数组初始化（`int (*fp[2])(int) = {f1, f2};`）时，引入了一个多维数组初始化 bug：

**原始代码**（`elem_size != 8` 分支）：
```rust
let val = values.get(i).copied().unwrap_or(0);
self.emit(OpCode::PushConst, val, loc);
self.emit(OpCode::StoreMem, 0, loc);
```

**错误重构**：
```rust
if let Some(elem) = elements.get_mut(i) {
    // ... 根据 elem 类型生成代码
} else {
    self.emit(OpCode::PushConst, 0, loc);  // ❌ 错误！
    self.emit(OpCode::StoreMem, 0, loc);
}
```

对于 `int arr[2][3] = {{1,2,3},{4,5,6}};`：
- `elements` = `[InitList([1,2,3]), InitList([4,5,6])]`（长度为 2）
- `count` = 6（总元素数）
- `values` = `[1, 2, 3, 4, 5, 6]`（扁平化后）

当 `i >= 2` 时，`elements.get_mut(i)` 为 `None`，走 `else` 分支 push `0`，但正确值应为 `values[i]`。

**修复**：`else` 分支改为 `let val = values.get(i).copied().unwrap_or(0);`

### 验证结果

```bash
cd native && cargo test --test end_to_end_extra_test
```

```
running 147 tests
test result: ok. 147 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

全部测试套件：
```
test result: ok.   0 passed (empty)
test result: ok.  10 passed (bytecode_gen_unit_test)
test result: ok.  13 passed (compile_pipeline_test)
test result: ok. 147 passed (end_to_end_extra_test)
test result: ok.  23 passed (end_to_end_test)
test result: ok.  10 passed (lexer_unit_test)
test result: ok.  12 passed (parser_unit_test)
test result: ok.  12 passed (type_checker_unit_test)
test result: ok.   3 passed (test_snapshot)
test result: ok.   7 passed (vm_memory_safety_test)
```

总计 **237 passed; 0 failed**。
