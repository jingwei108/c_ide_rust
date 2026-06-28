# 指针复合赋值运算符全面拓展计划

> 记录日期：2026-06-28
> 关联模块：`cide_typeck` / `cide_codegen` / `cide_vm`（VM 无需改动）
> 关联诊断码：`E3045_CompoundAssignType`

## 1. 现状与问题

当前 Cide 的复合赋值运算符（`+=`、`-=`、`*=`、`/=`、`%=`、`&=`、`|=`、`^=`、`<<=`、`>>=`）在类型检查阶段被限制为**左右操作数必须都是标量**（`int`、`char`、`float`、`double`、`long long` 及其 `unsigned` 变体）。

典型受限代码：

```c
int a[10];
int* p = a;
p += 2;   // 当前报错：E3045 复合赋值要求两边都是标量类型
p -= 1;   // 同上
```

根因定位：

- `cide_typeck/src/expr/ops.rs:284-290`：`resolve_assign` 中 `E3045_CompoundAssignType` 判断直接调用 `is_scalar`，未区分 `AddAssign/SubAssign` 的指针语义。
- `cide_codegen/src/expr/assign.rs:42-124`：`emit_compound` 闭包只按标量选择 `Add/Sub` 等 opcode，未像 `binary.rs` 那样对指针做 `ptr_step_size` 缩放。

## 2. 学生场景覆盖调研

对现有题库（baseline 305、K&R 81、LeetCode 138、模板生成 82，共约 600+ 用例）进行扫描，结果如下：

| 用法 | 出现次数 | 学生场景热度 | 说明 |
|------|----------|--------------|------|
| `p++` / `++p` / `p--` / `--p` | 大量 | 🔥🔥🔥 | 已支持，是学生指针遍历的首选写法 |
| `p = p + n` / `p = p - n` | 少量 | 🔥🔥 | 已支持，等价于指针移动 |
| `p += n` / `p -= n` | **0** | 🔥 | 题库中未出现，但教学代码/教材示例常见 |
| `void* p; p += n;` | **0** | 🔥 | 题库未出现；教学中 `malloc` 返回 `void*` 后可能遇到 |
| `char* s += n` | **0** | 🔥 | 字符串处理中可能出现，但学生更习惯 `s++` 或 `s = s + n` |
| `p += i++` 等右侧带副作用 | **0** | 🔥 | 整数复合赋值有类似写法，指针上极少 |
| `int** pp += n` | **0** | 🧊 | 多级指针算术，教学中罕见 |
| `p *= n` / `p /= n` 等 | **0** | 🧊 | C 标准未定义，学生不会这么写 |
| `p += q`（指针+指针） | **0** | 🧊 | C 标准未定义 |

**结论**：指针复合赋值在学生代码中并非高频语法，但属于"教材会讲、学生可能模仿"的基础语法糖。由于 `p = p + n` 和 `p++` 都已支持，支持 `p += n` 的主要价值是**降低学生从教材/网络代码迁移到 Cide 时的摩擦**，而不是解锁新能力。

## 3. C/C++ 教学子集语义边界

### 3.1 推荐支持的语义（覆盖学生 99% 场景）

| 表达式 | 等价形式 | 结果类型 | 学生场景 |
|--------|----------|----------|----------|
| `p += n` | `p = p + n` | 与 `p` 相同 | 数组遍历、指针跳跃 |
| `p -= n` | `p = p - n` | 与 `p` 相同 | 反向遍历、回退指针 |
| `p += 0` | 无实际偏移 | 与 `p` 相同 | 边界占位、教学演示 |

覆盖类型：

- 普通数据指针：`int*`、`char*`、`float*`、`double*`、`struct S*`。
- 字符串指针：`char*` 步长为 1，与 `s++` 语义一致。
- 多级指针：`int** pp; pp += 1;` 按 `sizeof(int*)` 缩放（教学中较少见，但实现成本低）。
- 数组退化后的指针：函数参数 `int a[]` 退化为 `int*`，随后 `a += i` 语义上等价于指针移动。
- `void* p; p += n;`：C 标准未定义，但 GCC/Clang 扩展按 **1 字节**处理。教学场景中 `malloc` 返回 `void*` 后学生可能直接写 `p += 4`，建议**作为扩展支持**，与主流编译器行为一致，但需在文档中诚实记录这是非标准扩展。

右侧表达式：支持任意整数表达式，包括带副作用的 `i++`、函数调用返回值等，与现有整数复合赋值保持一致。

### 3.2 明确禁止并给出清晰诊断的语义

| 表达式 | 原因 | 期望行为 |
|--------|------|----------|
| `p *= n` / `p /= n` / `p %= n` / `p &= n` / `p \|= n` / `p ^= n` / `p <<= n` / `p >>= n` | C 标准未定义指针的这些运算 | 保持 `E3045` 报错，提示"指针不支持此复合赋值" |
| `p += q` / `p -= q`（`p`、`q` 均为指针） | 复合赋值的左侧必须可写，右侧若为指针则无意义 | 报错，提示"指针只能与整数进行加减" |
| 函数指针 `void (*fp)(); fp += 1;` | C 标准禁止函数指针算术 | 报错 |

### 3.3 两种实施策略建议

#### 策略 A：最小可行方案（推荐，ROI 最高）

- 仅支持 `AddAssign` / `SubAssign`。
- 左侧为完整对象类型指针（含 `void*` 按 1 字节）。
- 右侧为整数表达式（`char`/`int`/`long long`/`unsigned` 等）。
- 不支持函数指针、不支持指针 ± 指针。
- 足够覆盖学生从教材/网络代码迁移过来的需求。

#### 策略 B：完整标准方案

- 在策略 A 基础上，对 `void*` 严格按 C 标准报错（与 Clang 标准模式一致）。
- 更精细的错误码分类，如 `E3045A_CompoundAssignPointerType`。
- 增加对 `const` 指针 `+=` 的诊断增强。
- 适合需要严格对标 Clang 标准模式的场景，但会与学生常见写法产生摩擦。

**建议采用策略 A**，因为学生代码与教材示例普遍依赖 GCC/Clang 扩展行为，且 `void*` 算术实现成本极低。

## 4. 全链路改动方案

### 4.1 TypeChecker（`cide_typeck/src/expr/ops.rs`）

改造 `resolve_assign` 中 `E3045` 判断前的逻辑：

1. 新增局部辅助判断：
   - `left_is_ptr = left_type.is_pointer()`
   - `right_is_int = self.is_int(&right_type) || right_type.kind() == TypeKind::LongLong`
   - `left_is_void_ptr = ...`
   - `left_is_func_ptr = ...`

2. 对 `AddAssign | SubAssign` 单独分支（策略 A）：
   - 若左侧为指针且右侧为整数：通过。
   - 若左侧为 `void*` 且右侧为整数：通过（按 1 字节扩展，与 GCC/Clang 一致）。
   - 若左侧为函数指针：报错。
   - 若左侧为指针且右侧不是整数：报错，使用更精确的错误码（可复用 `E3016_ArithmeticTypeError` 或新增 `E3045A_CompoundAssignPointerType`）。

3. 对其余复合赋值运算符：保持标量限制，但错误信息应区分"指针不支持此复合赋值"。

4. 类型转换：指针复合赋值不需要 `insert_implicit_cast` 对右侧做普通类型转换，但应确保右侧整数类型可被接受（`char`、`int`、`long long`、`unsigned` 等）。可在类型检查阶段将右侧表达式结果视为整型，codegen 阶段按需处理。

5. 右侧副作用：`resolve_expr_type(right)` 已经会完整解析右侧表达式，因此 `p += i++`、`p += foo()` 等带副作用表达式天然支持，无需额外处理。但需注意 `gen_assign` 中临时槽位不要与右侧表达式使用的槽位冲突（参考 H06 修复经验）。

6. 赋值兼容性：`check_assignable(&left_type, &right_type, loc)` 对 `p += n` 会返回 `false`（指针与整数不可赋值），因此需要在调用 `check_assignable` 前对指针 `+=/-=` 场景短路，避免误报 `E3044`。

### 4.2 BytecodeGen（`cide_codegen/src/expr/assign.rs`）

在 `gen_assign` 的复合赋值路径中，识别左侧为指针类型时，替换 `emit_compound` 的标量分支。

#### 4.2.1 指针 `+=` 字节码模板

以局部变量指针 `int* p` 为例，左侧形态不同但栈上操作一致：

```text
; 读旧指针值到栈顶
LoadLocal p

; 生成右侧整数表达式（结果在栈顶为 int）
gen_expr(n)

; 计算步长 = sizeof(int)
PushConst step
Mul

; 相加
Add

; 写回变量 p
StoreLocal p

; 返回值：复合赋值表达式返回左值类型，Cide 当前行为是返回写入后的值
; 若调用方需要栈顶保留返回值，应在 StoreLocal 后再 LoadLocal p
```

#### 4.2.2 指针 `-=` 字节码模板

```text
LoadLocal p
gen_expr(n)
PushConst step
Mul
Sub
StoreLocal p
; （按需 LoadLocal p 返回新值）
```

#### 4.2.3 左值形态处理

当前 `gen_assign` 已支持多种左值形态，指针复合赋值需复用这些路径：

| 左值形态 | 示例 | 处理要点 |
|----------|------|----------|
| 局部变量 | `int* p; p += n;` | `LoadLocal` / `StoreLocal` |
| 全局变量 | `int* gp; gp += n;` | `LoadGlobal` / `StoreGlobal` |
| 静态局部变量 | `static int* sp; sp += n;` | `LoadStaticLocal` / `StoreStaticLocal` |
| 解引用 | `*pp += n;` | 先 `gen_addr` 取地址，再 `LoadMem` / `StoreMem`；注意这里 `*pp` 的 `left_type` 是 `int`，不是指针，因此不会进入指针复合赋值分支 |
| 数组索引 | `arr[i] += n;` | 索引结果类型是元素类型（如 `int`），走标量分支 |
| 成员访问 | `s.p += n;` | 若成员为指针类型，按指针分支处理 |
| C++ 引用返回 | `f().p += n;` | 当前引用返回的复合赋值已报错，可保持暂不支持 |

关键修改点：

- 在 `emit_compound` 闭包中，不要只根据 `left_is_double/float/long_long/unsigned` 判断，而是先判断 `left.ty().is_pointer()`。
- 对指针 `AddAssign/SubAssign` 使用新的闭包 `emit_ptr_compound`，按 `ptr_step_size(left.ty())` 缩放。
- `void*` 特殊处理：`compute_type_size(void)` 当前返回 0，因此 `ptr_step_size(void*)` 会返回 0。需要在该分支中判断 pointee 为 `void` 时强制使用步长 1，与 GCC/Clang 扩展行为一致。
- 注意 `left.ty()` 在数组参数退化后应为 `Pointer` 类型；若仍为 `Array`，需要参考 `binary.rs` 中 `left_is_ptrlike` 的处理，将数组转换为指向首元素的指针。
- 右侧带副作用：复合赋值模板"读旧值 → gen_expr(右) → 缩放 → 运算 → 写回"本身不会破坏副作用顺序，但需确保临时槽位不冲突。

### 4.3 VM（`cide_vm`）

**无需新增 opcode**，复用现有 `PushConst`、`Mul`、`Add`、`Sub`、`LoadLocal`、`StoreLocal`、`LoadMem`、`StoreMem` 等指令即可。

需要验证的风险点：

- `Add` / `Sub` 对有符号整数有溢出 trap。指针运算可能出现"地址相加后超过 32 位有符号整数范围"的情况，但 Cide VM 使用 32 位地址空间，教学场景下不会触发。
- 若右侧整数为负数，`Mul` 得到负步长，`Add` 会正确执行减法效果。

## 5. 测试防线

### 5.1 Baseline 回归用例（`native/tests/cases/baseline/`）

新增用例：

1. `pointer_add_assign.c`
   ```c
   #include <stdio.h>
   int main() {
       int a[5] = {10, 20, 30, 40, 50};
       int* p = a;
       p += 2;
       printf("%d\n", *p);
       p += 0;
       printf("%d\n", *p);
       p -= 1;
       printf("%d\n", *p);
       return 0;
   }
   ```
   期望输出：`30`、`30`、`20`。

2. `pointer_add_assign_char.c`
   验证 `char*` 步长为 1。

3. `pointer_add_assign_double.c`
   验证 `double*` 步长为 8。

4. `pointer_add_assign_struct.c`
   验证 `struct S*` 按结构体总大小缩放。

5. `pointer_add_assign_multi_level.c`
   验证 `int** pp; pp += 1;` 按 `sizeof(int*)` 缩放。

6. `pointer_add_assign_negative.c`
   验证右侧为负整数时的减法效果。

7. `pointer_add_assign_member.c`
   验证结构体成员为指针时的复合赋值。

8. `pointer_add_assign_void.c`
   验证 `void*` 按 1 字节扩展支持，与 GCC/Clang 扩展行为一致。
   ```c
   #include <stdio.h>
   int main() {
       char buf[10] = {0,1,2,3,4,5,6,7,8,9};
       void* p = buf;
       p += 3;
       printf("%d\n", *(char*)p);
       p -= 2;
       printf("%d\n", *(char*)p);
       return 0;
   }
   ```
   期望输出：`3`、`1`。

9. `pointer_add_assign_side_effect.c`
   验证右侧带副作用表达式：`p += i++`。
   ```c
   #include <stdio.h>
   int main() {
       int a[5] = {10,20,30,40,50};
       int* p = a;
       int i = 1;
       p += i++;
       printf("%d %d\n", *p, i);
       return 0;
   }
   ```
   期望输出：`20 2`。

### 5.2 错误诊断用例（`native/tests/cases/error/` 或 `STUDENT_ERROR_TEST_CASES.md`）

- `int* p; int* q; p += q;` → 报错：指针不能参与 `+=` 运算。
- `int* p; p *= 2;` → 报错：指针不支持 `*=`。
- `void (*fp)(); fp += 1;` → 报错：函数指针不能算术。

### 5.3 Shadow Verification

- 普通数据指针用例：Golden 由 Clang 标准模式生成，输出应完全一致。
- `void*` 用例：由于 C 标准未定义 `void*` 算术，需使用 Clang 的默认扩展模式（`-std=gnu17` 或不加 `-pedantic`）生成 Golden，并在文档中记录这是 GCC/Clang 扩展行为。

### 5.4 单元测试

- `cide_typeck` 单元测试：验证 `p += n` 类型检查通过、`p *= n` 报错。
- `cide_codegen` 单元测试：验证生成字节码包含 `PushConst step`、`Mul`、`Add`/`Sub`。

## 6. 文档与诚实记录

### 6.1 需更新的文档

- `docs/current/C_SUBSET_SPEC.md`：在"赋值语句"章节补充指针 `+=/-=` 示例，并说明仅支持 `+/-` 复合赋值。
- `docs/current/CPP_SUBSET_SPEC.md`：同步更新 C++ 子集说明。
- `AGENTS.md` / `AGENTS_EN.md`：更新"C 教学子集支持概览"中复合赋值的描述。
- `CHANGELOG.md`：在 `[Unreleased]` 中记录新增支持。

### 6.2 诚实记录项（与 Clang 的差异）

- `void*` 算术：Cide 按 **1 字节步长**支持 `void* p; p += n;`，这与 GCC/Clang 默认扩展行为一致，但**不符合严格的 C 标准**（C99/C11 未定义 `void*` 指针算术）。教学中应引导学生优先写 `char* p = malloc(...); p += n;`。
- 函数指针算术：明确不支持并记录。
- 指针 `-=` 整数结果为指针：与普通二元 `-` 的"指针 - 指针 = ptrdiff_t"区分，避免学生混淆。
- 指针复合赋值返回值：Cide 返回右值指针值，C 标准中返回左值；教学场景通常不依赖此差异。

## 7. 实施顺序与里程碑

| 阶段 | 任务 | 预计文件 | 验收标准 |
|------|------|----------|----------|
| P0 | TypeChecker 支持 `p += n` / `p -= n` 类型检查 | `cide_typeck/src/expr/ops.rs` | `int* p; p += 2;` 不再报 E3045；错误场景仍正确报错 |
| P1 | BytecodeGen 生成指针步长缩放字节码 | `cide_codegen/src/expr/assign.rs` | 输出与 Clang 一致 |
| P2 | Baseline 回归用例 | `native/tests/cases/baseline/` | Shadow Verification 通过 |
| P3 | 错误诊断用例与单元测试 | `native/tests/` | 错误码与提示准确 |
| P4 | 文档同步 | `C_SUBSET_SPEC.md` / `AGENTS.md` / `CHANGELOG.md` | 文档与实现一致 |
| P5 | CI 全量回归 | `.github/workflows/ci.yml` | `cargo test` / `shadow_verify.py` / `flutter test` 全绿 |

## 8. 风险与回退策略

1. **`gen_assign` 已很复杂（600+ 行）**：修改时优先提取指针复合赋值的独立闭包或子函数，避免在 `emit_compound` 中增加过多分支。
2. **与普通二元指针运算不一致**：所有缩放逻辑必须复用 `ptr_step_size`，避免硬编码步长。
3. **返回值语义**：C 标准中 `p += n` 返回的是左值（可再赋值），Cide 当前复合赋值表达式返回的是右值标量。指针复合赋值可继续返回右值指针值，与现有行为保持一致，并在文档中诚实记录。
4. **数组退化边界**：若左侧仍为 `Array` 类型（非常见场景），需要转换为 `Pointer` 处理。

## 9. 结论

本计划通过**类型检查放宽 + 字节码步长缩放**两阶段改造，使 Cide 支持指针与整数的 `+=`/`-=` 复合赋值，覆盖学生代码迁移和教材示例中的常见场景：

- 普通数据指针、`char*` 字符串指针、`void*`（按 1 字节扩展）、多级指针均支持。
- 右侧支持任意整数表达式，包括带副作用的 `i++`、函数调用返回值。
- 函数指针、指针与指针的 `+=`/`-=`、以及其他复合赋值运算符保持禁止并给出清晰诊断。

全链路不需要新增 VM opcode，测试与文档同步后可纳入 CI。对学生而言，此改造主要价值是**降低从教材/网络代码迁移到 Cide 的语法摩擦**，而非解锁新能力——因为 `p = p + n` 和 `p++` 已经支持。
