# Double (f64) 全管线支持 — 测试修复记录

> 修复日期：2026-05-17  
> 涉及范围：Lexer→Parser→TypeChecker→BytecodeGen→VM→HostFuncs  
> 测试状态：**全部通过**（128 e2e + 单元测试）

---

## 问题总览

在完成 VM 字节偏移迁移和 BytecodeGen 的 `*D` 操作码后，42 个端到端测试失败。经逐类排查，共定位并修复 **10 个根因**，涵盖参数传递约定、隐式类型转换、host 函数调用约定、数组初始化、变量对齐等。

---

## 1. `FuncMeta.param_sizes` 语义错误

### 现象
- `test_e2e_forward_decl`、`test_e2e_multi_arg_function`、`test_e2e_array_stack` 等所有带函数调用的测试报 **栈下溢**。

### 根因
`enter_function` 把参数的**字节大小**（int=4, double=8）直接 push 到 `param_sizes`：
```rust
let sz = if p.ty.is_array() { 4 } else { self.type_size(&p.ty) };
param_sizes.push(sz);   // ❌ 存的是字节
```
而 `exit_function` 中：
```rust
meta.arg_count = meta.param_sizes.iter().sum();  // 变成总字节数
```
VM `Call` 把 `arg_count` 当作总**字数**（word count）来 pop：
```rust
let total_words = meta.arg_count as u32;
for word_count in meta.param_sizes.iter().rev() { ... }
```
对于 `add(int, int)`，`param_sizes = [4, 4]`，`arg_count = 8`。VM 尝试 pop 8 次，但 caller 只 push 了 2 个值 → 栈下溢。

### 修复
```rust
let words = (sz + 3) / 4;
param_sizes.push(words);  // ✅ 存字数
```

---

## 2. VM `Call` 参数传递顺序颠倒

### 现象
- 即使 `param_sizes` 修复后，函数调用参数值互换。例如 `sub(15, 3)` 内部看到 `a=3, b=15`。

### 根因
新 `Call` 代码使用逆序遍历：
```rust
let mut word_offset = total_words;
for word_count in meta.param_sizes.iter().rev() {
    word_offset -= words;
    let addr = locals_base + word_offset * 4;
    ... pop -> store ...
}
```
caller push 顺序是逆序（最后参数先 push），栈顶是**第一个参数**的值。但上述代码把栈顶值存到了高地址（后面的参数位置）。

### 修复
改为正序遍历，`word_offset` 从 0 递增：
```rust
let mut word_offset = 0;
for word_count in meta.param_sizes.iter() {
    let words = *word_count as u32;
    let addr = locals_base + word_offset * 4;
    for w in (0..words).rev() {  // double: 先 pop high，再 pop low
        let val = self.pop() as i32;
        self.store_i32(addr + w * 4, val, &inst.loc);
    }
    word_offset += words;
}
```
这样 pop 的栈顶（第一个参数的值）填入最低的局部地址，与 `local_indices` 正序一致。

---

## 3. `printf` `%f` 无法正确读取 double

### 现象
- `test_e2e_double_basic` 输出 `0.000000` 而非 `5.500000`。

### 根因
`format_printf_string` 中 `%f` 一直读取 f32：
```rust
'f' => {
    let f = f32::from_bits(arg as u32);  // ❌ 只取低 32 位
    ...
}
```
double 参数在 value stack 上是完整的 64 位 f64 bits。取低 32 位 reinterpret 为 f32 得到几乎为 0 的值。

### 修复
```rust
'f' => {
    let f = f64::from_bits(arg);  // ✅ 读取完整 64 位
    ...
}
```

---

## 4. `printf`/`fprintf` 的 float 参数未提升为 double

### 现象
- 现有大量测试使用 `float a = 3.14f; printf("%f", a);`。如果 `%f` 改为读 f64，这些测试会失败，因为 `PushConstF` 只填充低 32 位，高 32 位为 0，reinterpret 为 f64 后值极小。

### 根因
C vararg 中 `float` 会提升为 `double`，但 BytecodeGen 对 `printf` 的 float 参数直接 push f32 bits。

### 修复
在 `gen_expr(Call)` 中，对 `printf`/`fprintf` 的 `Float` 参数自动插入 `CastF2D`：
```rust
if (name == "printf" || name == "fprintf") && arg_ty_kind == TypeKind::Float {
    self.emit(OpCode::CastF2D, 0, &loc);
}
```
这样所有传入 `printf` 的浮点参数统一为 f64 bits，`%f` 可以统一读取 f64。

---

## 5. `SplitD` 错误地用于 `CallHost`

### 现象
- `test_e2e_double_arr` 输出 `0.000000`。`bytecode` 中 `LoadMemD` 后仍有 `SplitD`。

### 根因
BytecodeGen 对所有 `Double` 参数都 emit `SplitD`：
```rust
} else if arg_ty.kind == TypeKind::Double {
    self.gen_expr(arg);
    self.emit(OpCode::SplitD, 0, &loc);  // ❌ 不区分 Call / CallHost
}
```
`SplitD` 把 f64 拆成两个 i32 stack words，这是为了配合 VM `Call` 的按-word 参数传递。但 `CallHost`（如 `printf`）由 host 函数自己 pop，它只 pop 1 个值（specs.len() 决定）。`SplitD` 后栈上变成 `[low, high, fmt_addr]`，host 只 pop `high`，导致格式错误。

### 修复
只对普通函数调用 `Call` emit `SplitD`：
```rust
} else if arg_ty.kind == TypeKind::Double {
    self.gen_expr(arg);
    if self.func_index.get(name).is_some() {  // ✅ 仅普通函数
        self.emit(OpCode::SplitD, 0, &loc);
    }
}
```

---

## 6. TypeChecker 缺少赋值/初始化隐式 cast

### 现象
- `double a = 3.5;` 中 `3.5` 的 `FloatLiteral` 保持 `ty = Float`，BytecodeGen 生成 `PushConstF`。

### 根因
`insert_implicit_cast` 只在函数调用参数中被调用，`VarDecl`、`Assign`、`InitList` 元素中均未调用。

### 修复
在三处补充隐式转换：
- `Stmt::VarDecl` 的 `init_expr`
- `Stmt::VarDecl` 的 `extra_vars` init
- `Expr::Assign` 的 `right`
- `check_array_initializer` / `validate_nested_init_list` 的每个元素

---

## 7. double 数组初始化精度丢失

### 现象
- `double arr[3] = {1.1, 2.2, 3.3}; printf("%f", arr[1]);` 输出 `1074580685.000000`（即 `2.2f` 的 f32 bits 被当作 f64 值）。

### 根因
`flatten_init_list` 对 `FloatLiteral` 执行：
```rust
Expr::FloatLiteral { value, .. } => result.push((*value as f32).to_bits() as i32)
```
得到的是 f32 的 bits（`i32`）。数组初始化代码把它当作 f64 值写入：
```rust
let idx = self.push_f64_constant(val as f64);  // ❌ val 是 i32，val as f64 = 1066192077.0
```

### 修复
- **局部 double 数组**：循环内直接 `gen_expr(elem)`，让 TypeChecker 已经转换好的 `FloatLiteral`（`ty = Double`）生成正确的 `PushConstD`。
- **全局 double 数组**：不再通过 `flatten_init_list`，而是直接匹配 `elements[i]` 的 `FloatLiteral`/`Literal`/`Unary Neg`，取原始 `f64` value 的 `to_bits()`。

---

## 8. `current_func_arg_bytes` 未设置

### 现象
- `test_e2e_2d_array_func_arg` 输出垃圾值如 `65508 2 0`。

### 根因
`gen_expr(Identifier)` 中对数组参数和局部数组的区分逻辑：
```rust
if ty.is_array() {
    if local_offset < self.current_func_arg_bytes {
        // Array parameter decayed to pointer → LoadLocal (读指针值)
    } else {
        // Local array → GetFrameBase + offset (取地址)
    }
}
```
但 `enter_function` 中**从未设置** `current_func_arg_bytes`，它一直是 0。导致所有数组（包括参数）都走 `else` 分支，生成的是参数的**地址** `&m`，而非参数存储的**指针值** `m`。

### 修复
在 `enter_function` 末尾：
```rust
self.current_func_arg_bytes = offset;
```

---

## 9. 局部变量/参数未按 4 字节对齐

### 现象
- `test_e2e_hanoi_recursive` 报 **Call: 栈溢出（栈与堆发生碰撞）**。

### 根因
`enter_function` 和 `gen_stmt(VarDecl)` 使用原始字节大小分配：
```rust
offset += sz;   // char: sz=1
```
导致 `hanoi(int n, char from, char to, char aux)` 的 `local_count = 4+1+1+1 = 7`。

但 VM `Call` 的存储逻辑按 4 字节 word 存储参数：
```rust
let addr = locals_base + word_offset * 4;
```
第四个参数（aux）存到 `locals_base + 12`，远超 `local_count=7` 的栈帧边界。后续 `store_i32` 越界写入栈外内存，虽未直接 trap，但 `printf` 内部读取 `from`/`to`/`aux` 时得到垃圾值，更严重的是栈帧布局混乱导致递归时 `heap_limit` 检查误判碰撞。

### 修复
所有局部变量和参数分配按 4 字节对齐：
```rust
let aligned_sz = (sz + 3) & !3;  // 向上取整到 4
offset += aligned_sz;
```
这样 `char` 也占 4 字节，与 `LoadLocal`/`StoreLocal` 的 4 字节存取宽度一致。

---

## 修改文件清单

| 文件 | 修改内容 |
|------|----------|
| `native/src/compiler/bytecode_gen.rs` | `param_sizes` 存字数；`current_func_arg_bytes` 赋值；局部变量 4 字节对齐；`printf` float→double cast；`SplitD` 仅用于 `Call`；局部/全局 double 数组初始化直接 `gen_expr` |
| `native/src/compiler/type_checker.rs` | `VarDecl`/`Assign`/`InitList` 补充 `insert_implicit_cast` |
| `native/src/vm/vm.rs` | `Call` 正序遍历 `param_sizes`，`word_offset` 从 0 递增 |
| `native/src/vm/host_funcs.rs` | `format_printf_string` 的 `%f` 读取 `f64::from_bits(arg)` |

---

## 测试验证

```bash
cd native && cargo test
# 结果：全部通过
#   end_to_end_extra_test: 128 passed
#   其他单元测试: 全部 passed
```
