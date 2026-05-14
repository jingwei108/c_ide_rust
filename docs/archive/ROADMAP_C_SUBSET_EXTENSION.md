# C IDE 子集拓展路线图

> 基于 2026-05-10 的编译器能力边界全面评估

## 一、当前能力边界速览

| 层级 | 已支持 | 未支持/限制 |
|:---|:---|:---|
| **Lexer** | 48 种 token，整数/十六进制/浮点/字符/字符串字面量，`//` 和 `/* */` 注释，`#define` 常量宏 | 无八进制，无宽字符 |
| **Parser** | 变量/数组/指针/struct/enum/typedef，if/while/do-while/for/switch，全部表达式优先级（含三元、cast、复合赋值、位运算） | 无 `union`，无 `goto`，无函数指针，无 VLA，无 `static`/`extern` |
| **TypeChecker** | int/char/float/void/指针/数组/struct/enum，`const` 语义（直接变量），`unsigned`/`long` 等修饰符被解析但**映射为 int** | 无 `unsigned` 语义 |
| **BytecodeGen** | 63 条指令（含 float 算术/比较/转换），栈机模型，数组越界检查，指针步长缩放 | — |
| **VM** | 256KB 线性内存，NULL 陷阱区，malloc/free（first-fit），printf/scanf（可变参数，含 `%f`），strlen/strcpy/strcmp | 无 `realloc`/`qsort`/`stderr`，无文件 IO |
| **Diagnostics** | 56+ 错误码中文元数据，结构化自动修复（分号/括号/引号/`\|→\|\| `/`&→&&`/`<=→<`/`=→==`） | — |

---

## 二、拓展优先级矩阵

评估维度：**教学价值**（学生代码中多常见）× **实现难度**（需要改多少模块）

### P0 — 立即做（1~2 天，教学价值极高）

这些特性学生代码中**高频出现**，实现只需在现有架构上“打补丁”，不改变核心数据流。

| # | 特性 | 教学价值 | 实现难度 | 需改模块 | 说明 |
|:---|:---|:---|:---|:---|:---|
| 1 | **`NULL` 关键字/宏** | ⭐⭐⭐ | 🟢 极低 | Lexer + Parser | `int *p = NULL;` 目前报未声明变量。让 Lexer 识别 `NULL` 为 `TokenType::Null`，Parser 中将其解析为整数 `0` 即可。 |
| 2 | **`getchar()` / `putchar(int)`** | ⭐⭐⭐ | 🟢 极低 | VM host_funcs + BytecodeGen + TypeChecker | 最基础的字符 IO。`getchar` 从输入缓冲区读一个字符；`putchar` 输出一个字符到输出缓冲区。 |
| 3 | **`rand()` / `srand(int)`** | ⭐⭐⭐ | 🟢 极低 | VM host_funcs + BytecodeGen + TypeChecker | 算法教学必备（生成随机数组、随机数猜测等）。VM 内用 `rand::Rng` 或 `std::hash` 实现。 |
| 4 | **`memset(ptr, value, size)`** | ⭐⭐⭐ | 🟢 低 | VM host_funcs + BytecodeGen + TypeChecker | 数组批量初始化教学常用（`memset(arr, 0, sizeof(arr))`）。VM 内循环 `store_i8`。 |
| 5 | **`exit(int)`** | ⭐⭐ | 🟢 极低 | VM host_funcs + BytecodeGen + TypeChecker | 提前终止程序。VM 中设置 `running = false` 并记录退出码。 |
| 6 | **`strcat(dest, src)`** | ⭐⭐ | 🟢 低 | VM host_funcs + BytecodeGen + TypeChecker | 字符串拼接。找到 dest 末尾 `\0`，追加 src。注意边界。 |
| 7 | **`atoi(str)`** | ⭐⭐ | 🟢 低 | VM host_funcs + BytecodeGen + TypeChecker | 字符串转整数。教学常用（如命令行参数解析的简化版）。 |

**P0 实施路径**：
1. 每个新宿主函数只需在 **3 个文件** 中各增加 ~15 行代码：
   - `vm/host_funcs.rs`：实现函数逻辑
   - `compiler/bytecode_gen.rs`：`visit_call` 中映射 `name → host_id`
   - `compiler/type_checker.rs`：`visit_call` 中添加参数个数/类型检查
2. 分配新的 host ID（当前用到 32，P0 可占用 33~39）。
3. `NULL` 是唯一的例外——只需在 Lexer 关键字表和 Parser 的因子解析中加一行。

---

### P1 — 短期做（3~7 天，教学价值高）

这些特性会**显著改善学生体验**，但需要跨模块协作或新增指令。

| # | 特性 | 教学价值 | 实现难度 | 需改模块 | 说明 |
|:---|:---|:---|:---|:---|:---|:---|
| 8 | **`const` 语义** | ⭐⭐⭐ | 🟡 中 | Parser + AST + TypeChecker | `const int MAX = 100;` 目前被忽略。需要：① Parser 保留 `is_const` 标记到 `VarDecl`；② TypeChecker 在符号表记录 `is_const`；③ 赋值检查时拒绝 `const` 左值。 |
| 9 | **`float` / `double` 基础支持** | ⭐⭐⭐ | 🔴 高 | Lexer + AST + TypeChecker + BytecodeGen + VM | ✅ **已完成（P1）**。支持 `float` 类型（32 位，`double` 未支持），浮点字面量 `3.14`，算术/比较/隐式转换，`printf("%f")`/`scanf("%f")`，`(int)`/`(float)` 强制转换。 |
| 10 | **`sprintf` / `snprintf`** | ⭐⭐⭐ | 🟡 中 | VM host_funcs + BytecodeGen + TypeChecker | 字符串格式化是教学高频需求。可复用 `host_printf_n` 的格式化逻辑，将结果写入目标缓冲区而非输出。`snprintf` 需额外长度限制。 |
| 11 | **函数式宏 `#define MAX(a,b) ((a)>(b)?(a):(b))`** | ⭐⭐⭐ | 🟡 中高 | Lexer（预处理器） | 教学中极其常见。实现最小子集：① 宏定义带括号参数列表；② 调用时按位置文本替换；③ 不支持 `##`/`#`/可变参数。难点：替换时需要处理括号嵌套和优先级保护。 |
| 12 | **`assert(cond)`** | ⭐⭐ | 🟡 中 | VM host_funcs 或 预处理器宏 | 如果条件为假，输出错误信息并终止。可当作内置函数实现，也可当作宏展开为 `if (!(cond)) { printf(...); exit(1); }`。 |
| 13 | **`static` 局部变量** | ⭐⭐ | 🟡 中高 | Parser + TypeChecker + BytecodeGen | `static int count = 0;` 在函数内。需要在全局数据区分配，但作用域限制在函数内。BytecodeGen 中 `LoadGlobal`/`StoreGlobal` 访问，但符号名需要 mangling（如 `func_name::var_name`）。 |
| 14 | **`realloc(ptr, size)`** | ⭐⭐⭐ | 🟡 中 | VM host_funcs | 调整已分配内存大小。实现逻辑：`malloc` 新块 → `memcpy` 旧数据 → `free` 旧块。注意 `ptr==NULL` 时退化为 `malloc`。 |
| 15 | **`calloc(n, size)`** | ⭐⭐ | 🟢 低 | VM host_funcs | `malloc(n*size)` + `memset(0)`。注意 `n*size` 溢出检查。 |

**P1 关键决策**：
- **`float` 支持** 是 P1 中最重的工作，也是学生痛点最集中的地方。如果资源允许，建议优先做。
- **`const` 语义** 改动面相对集中（TypeChecker 为主），性价比极高。
- **函数式宏** 如果太复杂，可以先支持最常见的单参数宏（如 `#define SQR(x) ((x)*(x))`）。

---

### P2 — 中期做（2~4 周，特定场景有价值）

| # | 特性 | 教学价值 | 实现难度 | 说明 |
|:---|:---|:---|:---|:---|
| 16 | **真正的 `unsigned` 语义** | ⭐⭐⭐ | 🔴 高 | 当前 `unsigned int` 只是 `int` 别名。需要：① Type 中区分 `is_unsigned`；② 新增无符号比较/除法/移位 VM 指令；③ printf `%u`/`%x` 格式符。 |
| 17 | **`long long` / 64 位整数** | ⭐⭐ | 🔴 高 | 需要 VM 支持 64 位寄存器或双字操作，C API 也要扩展。教学价值不如 `float`。 |
| 18 | **`union`** | ⭐⭐ | 🟡 中 | Parser + AST + TypeChecker + BytecodeGen（共享内存布局）。教学中使用频率较低。 |
| 19 | **文件 IO `fopen`/`fread`/`fwrite`/`fclose`** | ⭐⭐ | 🔴 高 | VM 需要维护文件句柄表，前端需要文件系统沙箱。在 IDE 环境中使用场景有限（学生通常只读写标准 IO）。 |
| 20 | **`sizeof` 数组形参退化保护** | ⭐⭐ | 🟢 低 | TypeChecker 发出警告：当 `arr` 作为函数参数退化后，`sizeof(arr)` 实际是 `sizeof(int*)`。 |

---

### P3 — 长期/低优先级（教学价值低或实现极复杂）

| # | 特性 | 教学价值 | 实现难度 | 不推荐原因 |
|:---|:---|:---|:---|:---|
| 21 | `goto` | ⭐ 极低 | 🟡 中 | 教学中明确不鼓励使用 |
| 22 | 函数指针 `int (*fp)(int)` | ⭐⭐ | 🔴 高 | 初学者极少用到；`qsort` 回调依赖此特性 |
| 23 | 位域 `int x:4` | ⭐ 低 | 🟡 中高 | 嵌入式专用，教学不常见 |
| 24 | 完整预处理器（`#ifdef`/`#if`/`#endif`） | ⭐⭐ | 🔴 极高 | 工程价值高，但对教学 IDE 场景价值有限 |
| 25 | `volatile` / `restrict` 语义 | ⭐ 极低 | 🔴 高 | 编译器优化相关，初学者不涉及 |
| 26 | 宽字符 / Unicode | ⭐⭐ | 🔴 高 | 当前 UTF-8 注释已支持，宽字符字面量需求不大 |

---

## 三、下一阶段建议（如果现在开始）

**推荐组合：P0 全做 + P1 选 2~3 项**

理由：
1. **P0 全部（1~2 天）**：`NULL`、`getchar`/`putchar`、`rand`/`srand`、`memset`、`exit`、`strcat`、`atoi` —— 这些都是“学生写了就报错”的高频场景，改一行就少一个挫败感。
2. **`const` 语义（1~2 天）**：教学代码中 `const int MAX = 100;` 极其常见，目前被静默忽略。加上语义检查后，能防止学生写出 `MAX = 200;` 的困惑代码。
3. **`float`/`double` 支持（3~5 天）**：这是当前子集与“真实 C”差距最大的地方。学生第一次写 `double a = 3.14;` 就编译失败，体验很差。最小可行方案（仅支持 `double` 一种浮点类型，用 `f64` 位模式存储在 `i32` 对齐的内存中）可以控制工程量。

**不要现在做的事**：
- `union`、`goto`、`volatile` / `restrict` —— 教学价值太低
- 完整预处理器 —— 工程复杂度过高
- `long long` 64 位 —— 在 32 位 VM 中收益不大

---

## 四、实现检查清单模板

若新增一个**宿主函数**（如 `getchar`），按此清单操作：

- [ ] `native/src/vm/host_funcs.rs`：实现 `host_getchar(vm, session)` 函数
- [ ] `native/src/vm/host_funcs.rs`：`execute_host_func` match 中分配新 ID（如 `40 => host_getchar`）
- [ ] `native/src/compiler/bytecode_gen.rs`：`visit_call` 中 `host_name` 映射 + `host_id` 映射
- [ ] `native/src/compiler/type_checker.rs`：`visit_call` 中添加参数个数/类型检查 + 返回类型
- [ ] `native/src/diagnostics/error_codes.rs`：如需要，新增错误码（通常可复用 `E3028_BuiltInArgCount` / `E3029_BuiltInArgType`）
- [ ] 端到端测试：`native/tests/end_to_end_extra_test.rs` 添加用例
- [ ] 知识卡片（可选）：如该函数有常见错误模式，新增 JSON 卡片

---

## 五、风险与注意事项

1. **Host ID 冲突**：当前 host ID 用到 32，建议 P0 使用 33~39，P1 使用 40~49，预留空间。
2. **`float` 与现有类型的交互**：如果只做 `double`（64 位）而不做 `float`（32 位），可以简化（只有一种浮点类型）。但 VM 值栈元素是 `i32`，需要决定是扩展值栈为 `i64` 还是用两个 `i32` 拼接存储 `f64`。
3. **函数式宏的陷阱**：文本替换宏最容易引入优先级 bug（如 `#define SQR(x) x*x` 在 `SQR(1+2)` 中出错）。实现时必须强制外层括号保护：`((x)*(x))`。
4. **`static` 局部变量的 mangling**：如果两个函数都有 `static int x;`，BytecodeGen 需要用 mangled 名区分，否则全局符号表冲突。
