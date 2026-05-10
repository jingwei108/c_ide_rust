# Float 类型支持与函数调用参数隐式转换

> 日期：2026-05-10
> 涉及平台：Rust Native (Lexer → Parser → TypeChecker → BytecodeGen → VM)
> 状态：✅ 已完成

---

## 1. Float 类型支持（P1 拓展）

### 背景
学生代码中最常见的挫败源之一是写 `float x = 3.14;` 或 `printf("%f", x);` 时编译器无法识别浮点数。本次实现为 Cide 子集添加了完整的 `float`（32 位单精度浮点）支持。

### 实现范围

| 模块 | 改动 |
|:---|:---|
| **Lexer** | 新增 `TokenType::FloatLiteral`，`number()` 识别 `3.14` 浮点字面量；`keyword_type()` 添加 `float` |
| **AST** | `TypeKind::Float`；`Expr::FloatLiteral { value: f64 }` |
| **Parser** | `parse_base_type()` 支持 `float`；cast/参数/返回类型等场景全部生效 |
| **TypeChecker** | scalar 类型体系（int/char/float），支持 float 的算术/比较/赋值/复合赋值/自增自减/条件/一元运算 |
| **BytecodeGen** | `type_size(Float) => 4`；新增 `CastI2F`/`CastF2I` 隐式转换，以及 `AddF`/`SubF`/`MulF`/`DivF`/`NegF`/`EqF`/`NeF`/`LtF`/`LeF`/`GtF`/`GeF`/`PushConstF` |
| **VM Opcode** | `opcode.rs` 新增 13 条 float 指令（编号 44~63） |
| **VM 执行** | `vm.rs` 实现所有 float 指令，使用 `f32::from_bits`/`to_bits` 复用现有 `i32` 值栈 |
| **Host 函数** | `printf`（0/1/2/n 参数版本）和 `scanf` 添加 `%f` 格式符支持 |

### VM 浮点指令设计

采用 **f32 单栈槽位模式**（而非双精度或双栈槽）：

- 一个 `float` 占 4 字节 = 一个 `i32` 栈槽
- `LoadLocal`/`StoreLocal`/`LoadMem`/`StoreMem` 可直接复用（仅读写 4 字节 bit pattern）
- 运算时通过 `f32::from_bits(pop() as u32)` 转换为浮点数运算，再 `to_bits() as i32` 压栈
- 优点：最小化 VM 改动，不需要新增内存加载/存储指令

---

## 2. 函数调用参数自动隐式转换

### 背景
在 `float` 支持完成后，函数调用参数仍然不会自动转换。例如：
```c
void foo(float x) {}
foo(5);       // 需要自动把 int 转为 float
void bar(int x) {}
bar(3.7);     // 需要自动把 float 截断为 int（并提示）
putchar(65.0); // host 函数也需要支持
```

### 实现方案

**TypeChecker 层插入隐式 Cast 节点**：

```rust
fn insert_implicit_cast(expr: &mut Expr, target: &Type) {
    // int/char -> float：插入 Expr::Cast { target_type: float }
    // float -> int/char：插入 Expr::Cast { target_type: int/char }
}
```

在 `visit_call` 中，对每个参数执行：
1. `resolve_expr_type(arg)` 解析实参类型
2. `is_assignable(expected, arg_type)` 检查兼容性（已会发出 `W3053` 警告）
3. `insert_implicit_cast(arg, expected)` 在 AST 中自动包装 `Cast` 节点

**BytecodeGen 层无需修改**：已有的 `Expr::Cast` 生成逻辑会自动输出 `CastI2F` / `CastF2I` 指令。

### 覆盖范围

| 场景 | 是否支持 | 说明 |
|:---|:---|:---|
| 普通函数调用 | ✅ | `void foo(float); foo(5);` |
| Host 函数（int 参数）| ✅ | `putchar`、`malloc`、`srand`、`memset` 后两参数、`exit` |
| `printf` 格式字符串参数 | ❌ | 参数类型由 `%d`/`%f` 动态决定，当前不解析格式字符串做逐参数检查 |

---

## 3. 测试覆盖

### Float 相关（7 个端到端测试）

| 测试名 | 验证内容 |
|:---|:---|
| `test_e2e_float_basic` | `float` 变量声明、加法、printf `%f` |
| `test_e2e_float_arithmetic` | `+ - * /` 四则运算 |
| `test_e2e_float_compare` | `> < == !=` 比较运算 |
| `test_e2e_float_mixed_int` | `float + int` 混合运算（结果提升为 float） |
| `test_e2e_float_cast` | `(float)a` / `(int)b` 显式强制转换 |
| `test_e2e_float_assign_int` | `float x = 5;` 隐式 int→float 转换 |
| `test_e2e_float_compound_assign` | `x += 3.0;` 复合赋值 |

### 函数调用隐式转换（3 个端到端测试）

| 测试名 | 验证内容 |
|:---|:---|
| `test_e2e_float_func_arg_implicit_cast` | `foo(5)` → 形参 `float` 自动转换 |
| `test_e2e_int_func_arg_implicit_cast_from_float` | `bar(3.7)` → 形参 `int` 自动截断 |
| `test_e2e_host_func_arg_implicit_cast` | `putchar(65.0)` → host 函数参数自动转换 |

---

## 4. 质量指标

- `cargo test`：**97/97 通过**（compile_pipeline 12 + end_to_end_extra 68 + end_to_end 17）
- `cargo clippy`：**0 警告**
- `dotnet build` (Desktop)：**0 错误 / 0 警告**

---

## 5. 已知限制

1. **不支持 `double`** — 仅支持 32 位 `float`
2. **`-0.0` 逻辑边界** — `&&`/`||` 对 float 按 bit pattern 判断，`-0.0`（0x80000000）会被视为 true（极罕见）
3. **`printf` 格式字符串参数不自动转换** — `printf("%f", 5)` 不会把 `5` 自动转为 float，建议写 `printf("%f", (float)5)` 或 `printf("%f", 5.0)`
