# C 语言子集规范（教学场景专用）

> 核心问题：对于一个教学场景，C 子集应该支持到什么程度？

---

## 1. 设计原则

### 1.1 最小必要集（Minimum Viable Subset）

**目标**：用最少的语法，覆盖 C 语言最核心的教学价值。

| 教学价值 | 需要的语法 | 是否必须 |
|:---|:---|:---|
| 程序的基本结构 | 变量、表达式、语句 | ✅ 必须 |
| 算法思维 | if/else、循环、函数、递归 | ✅ 必须 |
| C 语言的灵魂 | 指针（&、*） | ✅ 必须 |
| 复合数据 | 数组、struct | ✅ 必须 |
| 内存管理 | malloc/free | ✅ 必须 |
| 底层原理 | 内存布局、栈/堆/指针关系 | ✅ 必须 |

### 1.2 排除原则

**排除标准**：
1. 会分散初学者注意力的细节（如 printf 的格式化字符串）
2. 增加编译器复杂度但教学价值低（如 double 精度问题）
3. 可以用现有语法等价表达的（如 break/continue 可用 return 替代）
4. 在 CideVM 沙盒中无意义的（如 #include、文件 I/O）

---

## 2. 支持的语法（Phase 1 MVP）

### 2.1 数据类型

```c
// 标量类型：int（32位有符号整数）、char（8位字符，按 i32 存储）
int a;
int a = 5;
char c = 'A';
char c = 65;   // char 与 int 可隐式转换（带警告）

// 无符号整数（语义上与 int 相同，教学子集不区分有/无符号）
unsigned u = 5;
unsigned int v = 10;

// 一维数组（大小必须是编译期常量或省略）
int arr[10];
int arr[] = {1, 2, 3, 4, 5};  // 自动推断大小为 5
char s[] = "hello";           // 字符串初始化 char 数组，自动推断大小为 6（含 '\0'）

// 多维数组（支持嵌套初始化列表和函数参数传递）
int mat[3][3] = { {1,2,3}, {4,5,6}, {7,8,9} };
void foo(int m[][3]) { m[0][0] = 1; }

// 指针
int* p;
int* p = &a;      // 取地址
int* p = malloc(4);  // 动态分配（4 = sizeof(int)）
char* str = "hello"; // 字符串字面量退化为 char*（用于 printf/scanf）

// 结构体
struct Node {
    int val;
    struct Node* next;
};
struct Node node;     // 值语义（简化：不需要理解 struct 拷贝）
struct Node* np;      // 指针语义

// 结构体初始化（新增）
struct Node n = {10, 0};      // 完整初始化
struct Node m = {5};          // 部分初始化（剩余字段自动为 0）
struct Node a = {1, 0};
struct Node b = {2, &a};      // 初始化列表中可使用取地址表达式

// 枚举（编译期常量，底层为 int）
enum Color { Red, Green, Blue };
enum Color { Red, Green = 2, Blue };  // 可显式指定值

// 类型别名
typedef int MyInt;
typedef int* IntPtr;
```

**设计决策**：
- **int 为主，char 为辅**：char 用于字符串教学；char 本质是小整数，按 i32 存储
- **一维数组**：足够演示排序、搜索等算法
- **多维数组**：支持二维数组声明、嵌套初始化列表、索引访问和函数参数传递（如 `int[][3]`）
- **数组/字符串初始化**：支持 `{1,2,3}` 和 `"hello"` 两种初始化方式，自动推断大小
- **基本指针**：&（取地址）、*（解引用）是 C 的灵魂，必须支持
- **struct**：链表、树等数据结构的基础；统一为引用语义（不需要理解值拷贝 vs 引用）
- **enum**：编译期计算常量值，生成 CideVM 全局常量，便于教学演示状态机
- **typedef**：简化复杂类型声明，提升代码可读性

### 2.2 语句

```c
// 变量声明（支持每行多个变量）
int a;
int a = 5;
int arr[10];
int a = 1, b = 2, c = 3;  // 多变量声明

// 赋值语句
a = 10;
a += 5;   // 复合赋值
a++;      // 后缀自增
++a;      // 前缀自增

// 表达式语句
foo(a, b);

// 块作用域
{
    int b = 20;  // b 只在这个块内可见
}

// if/else
if (a > 5) {
    // ...
} else {
    // ...
}

// while 循环
while (i < n) {
    // ...
}

// do...while 循环
do {
    // ...
} while (i < n);

// for 循环（C99 风格：可在初始化中声明变量）
for (int i = 0; i < n; i++) {
    // ...
}

// switch / case / default
switch (x) {
    case 1:
        // ...
        break;
    case 2:
        // ...
        break;
    default:
        // ...
        break;
}

// break / continue
for (int i = 0; i < n; i++) {
    if (arr[i] == target) {
        found = i;
        break;      // 跳出循环
    }
    if (arr[i] == 0) {
        continue;   // 跳过本次循环剩余代码
    }
}

// return
return a;
return;       // 等价于 return 0;
```

**设计决策**：
- **多变量声明**：`int a = 1, b = 2;` 支持同一类型多个变量同时声明
- **支持 for 循环**：这是算法教学的核心语法（排序、遍历等）
- **支持块作用域**：让学生理解变量的生命周期
- **break/continue**：循环控制的核心语法，搜索/过滤算法必备
- **switch/case**：多分支选择的经典语法，支持 fallthrough（不写 break 自然落入下一 case）
- **do...while**：至少执行一次的循环，与 while 形成互补教学

### 2.3 表达式

```c
// 算术运算（整数）
a + b
a - b
a * b
a / b      // 整数除法
a % b      // 取模

// 比较运算
a == b
a != b
a < b
a <= b
a > b
a >= b

// 逻辑运算
a && b     // 短路求值
a || b     // 短路求值
!a

// 赋值
a = b
a += b
a -= b
a *= b
a /= b
a %= b

// 数组索引
arr[i]
arr[0] = 10;

// 函数调用
foo(a, b)

// 取地址
&a

// 解引用（带空指针检查）
*p
*p = 10;

// 结构体访问（-> 和 . 行为一致，简化教学）
node.val
node->val
np->val

// 自增自减
++a
a++
--a
a--

// sizeof（编译期常量，教学子集中所有标量和指针均为 4 字节）
sizeof(int)      // 4
sizeof(char)     // 4（按 i32 存储）
sizeof(a)        // 4
sizeof(p)        // 4
```

**设计决策**：
- **整数除法**：`5 / 2 = 2`，让学生理解整数运算的特点
- **短路求值**：`&&` 和 `||` 必须支持短路，这是重要的概念
- **-> 和 . 行为一致**：struct 统一为引用语义，学生不需要理解 `(*p).val` 的转换
- **sizeof**：编译期计算，帮助学生理解类型大小和内存布局

### 2.4 函数

```c
// 函数定义
int add(int a, int b) {
    return a + b;
}

// 无参数函数
void hello() {
    // ...
}

// 递归函数
int factorial(int n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}

// main 函数作为入口
int main() {
    // ...
    return 0;
}
```

**设计决策**：
- **支持递归**：这是算法教学的核心（阶乘、斐波那契、树遍历）
- **void 返回类型**：简化无返回值函数的定义
- **main 作为入口**：符合 C 语言惯例

### 2.5 内存管理（简化版）

```c
// 动态分配（参数为字节数）
int* arr = malloc(10 * 4);   // 分配 10 个 int（每个 4 字节）

// 释放
free(arr);

// 使用分配的内存
arr[0] = 1;
arr[1] = 2;
```

**设计决策**：
- **参数是字节数**：`malloc(10 * 4)` 或 `malloc(10 * sizeof(int))`
  - `sizeof(int)` 和 `sizeof(struct S)` 已支持，帮助学生理解类型大小
- **宿主管理堆分配**：`malloc` / `realloc` / `free` 是宿主导入函数，宿主记录分配元数据（用于内存泄漏检测）
- **`realloc` 已支持**：完整支持扩容/缩容、NULL ptr（等价 malloc）、size 0（等价 free）

---

## 3. 明确不支持的语法

### 3.1 排除清单

| 特性 | 排除理由 | 遇到时的错误提示 |
|:---|:---|:---|
| `double` | ✅ **已支持**：完整 64 位精度浮点运算、数组、函数参数/返回值、printf `%lf` / scanf `%lf` | — |
| `char` / `char*` / 字符串 | ✅ **已支持**：char 按 i32 存储，字符串通过 Data Segment 注入；支持 `strlen`/`strcpy`/`strcmp`/`strcat` | — |
| `break` / `continue` | ✅ **已支持**：循环控制的核心语法 | — |
| `goto` | 教学上不鼓励使用；增加控制流图复杂度 | "暂不支持 `goto`" |
| `do...while` | ✅ **已支持**：至少执行一次的循环 | — |
| `switch` / `case` / `default` | ✅ **已支持**：多分支选择，支持 fallthrough | — |
| 预处理 (`#include`) | CideVM 沙盒中无意义 | "解释器模式下无需 `#include`，直接编写代码即可" |
| `union` | ✅ **已支持**：全管线支持（声明、`sizeof(union U)`、成员访问、`p->i`），内存布局为所有字段 offset=0、size=max(fields) | — |
| `bitfield` | 进阶特性，初学者不需要 | "暂不支持该特性" |
| 多维数组 | ✅ **已支持**：二维数组声明、嵌套初始化、索引访问、函数参数传递 | — |
| `sizeof` | ✅ **已支持**：编译期常量，所有标量/指针返回 4 | — |
| 逗号分隔的多变量声明 (`int a, b;`) | ✅ **已支持**：`int a = 1, b = 2;` | — |
| 标准库函数 (`printf` / `scanf` / `malloc` 除外) | ✅ **已支持**：printf / scanf / malloc / free 为宿主导入函数 | — |
| `typedef` | ✅ **已支持**：类型别名，提升代码可读性 | — |
| `enum` | ✅ **已支持**：编译期常量，底层为 int | — |
| `extern` / `static` / `volatile` / `restrict` | 存储类和类型修饰符，增加复杂度 | "暂不支持存储类修饰符" |
| `const` | ✅ **已支持**：直接变量 `const` 语义，阻止赋值和自增/自减 | — |

### 3.2 隐式转换与编译器警告

教学子集允许部分隐式转换（不阻断编译），但会发出警告，帮助学生理解类型系统：

| 转换方向 | 是否允许 | 警告信息 |
|:---|:---|:---|
| `int` → `char` | ✅ | "int 被隐式转换为 char。不同类型的标量之间赋值可能会丢失精度。" |
| `char` → `int` | ✅ | "char 被隐式转换为 int。不同类型的标量之间赋值可能会丢失精度。" |
| `int` → `pointer` | ✅ | "整数被隐式转换为指针。建议确保这是有意义的地址值（如 NULL = 0）。" |
| `array` → `pointer` | ✅ | "数组隐式转换为指针。数组名在表达式中会自动退化为指向首元素的指针。" |
| `void*` → 具体指针 | ✅ | "void* 指针被隐式转换为具体类型的指针。请确保内存布局正确。" |

**设计决策**：
- 教学场景下，隐式转换不应该卡死学生（如 `char c = 65;` 是常见写法）
- 通过警告而非错误的方式，既保证代码能运行，又提醒学生注意类型安全

---

## 4. 与教学功能的映射

### 4.1 语法支持 → 教学能力

| 教学场景 | 需要的语法 | 本项目支持？ |
|:---|:---|:---|
| Hello World（变量与输出） | 变量声明、赋值、内置输出函数 | ✅ |
| 冒泡排序 | 数组、for、if、函数 | ✅ |
| 二分查找 | 数组、while、if/else、函数 | ✅ |
| 矩阵运算 | 多维数组、嵌套循环、函数 | ✅ |
| 链表操作 | struct、指针、malloc/free | ✅ |
| 二叉树遍历 | struct、指针、递归 | ✅ |
| 阶乘/斐波那契 | 递归、if | ✅ |
| 指针基础教学 | &、*、指针作为参数 | ✅ |
| 内存布局教学 | 变量、数组、指针、malloc | ✅ |
| 字符串操作 | char、char*、字符串字面量、printf/scanf | ✅ |
| 文件读写 | fopen/fread/fwrite | ❌（沙盒中不支持） |
| 浮点运算 | float/double | ✅ |
| 枚举与状态机 | enum | ✅ |
| 类型抽象 | typedef | ✅ |

### 4.2 内存视图能展示什么

基于支持的语法，内存视图可以展示：

```c
int main() {
    int a = 10;                  // 栈变量
    int arr[5] = {1,2,3,4,5};   // 栈数组
    char s[] = "hello";          // 栈字符数组
    int* p = &a;                 // 栈指针 → 栈变量
    int* heap = malloc(3 * 4);   // 堆数组
    heap[0] = 100;
    
    struct Node node;            // 栈结构体
    node.val = 1;
    
    struct Node* np = malloc(4); // 堆结构体
    np->val = 2;
    np->next = NULL;
    
    enum Color c = Green;        // 枚举变量（底层为 int）
}
```

内存视图可以展示：
- ✅ 栈变量（绿色）
- ✅ 栈数组（绿色块）
- ✅ 指针变量及其指向关系（黄色 → 箭头）
- ✅ 堆分配（蓝色）
- ✅ 结构体内存布局（多个字段并排）
- ✅ 悬垂指针检测（红色）
- ✅ 内存泄漏检测（程序结束时未 free 的堆内存）

---

## 5. 与 VisualBinaryTree 的对比

| 特性 | VisualBinaryTree Algo-C Subset | 本项目 Cide-C Subset |
|:---|:---|:---|
| int | ✅ | ✅ |
| 数组 | ✅（一维） | ✅（一维 + 多维） |
| struct | ✅ | ✅ |
| 指针 | ⚠️ 有限（不支持 & 和 *） | ✅ 完整支持（&、*、作为参数） |
| malloc/free | ❌ | ✅（简化版） |
| if/else | ✅ | ✅ |
| for | ❌ | ✅ |
| while | ✅ | ✅ |
| return | ✅ | ✅ |
| 函数/递归 | ✅ | ✅ |
| break/continue | ❌ | ✅ |
| do...while | ❌ | ✅ |
| switch/case/default | ❌ | ✅ |
| char / 字符串字面量 | ❌ | ✅ |
| sizeof | ❌ | ✅ |
| typedef | ❌ | ✅ |
| enum | ❌ | ✅ |
| printf / scanf | ❌ | ✅（printf 支持可变参数） |
| float/double | ❌ | ✅ |
| 预处理 | ❌ | ❌ |
| 标准库（除 printf/scanf/malloc/free） | ❌ | ❌ |
| 指针运算 | ❌ | ❌ |

**本项目的扩展**：
- **新增 for 循环**：算法教学的核心语法
- **新增完整指针**（&、*）：C 语言教学的灵魂，内存视图和指针视图的基础
- **新增 malloc/free**：动态内存教学的基础，内存泄漏检测的前提
- **新增 break/continue**：循环控制的核心语法
- **新增 do...while / switch/case**：控制流教学完整性
- **新增 char / 字符串字面量**：字符串操作教学的基础
- **新增 sizeof / typedef / enum**：类型系统教学的基础
- **新增 printf / scanf**：格式化输入输出教学的基础

---

## 6. 编译器实现工作量评估

基于 Rust + CideVM 自定义字节码架构：

### 6.1 各模块代码量估算

| 模块 | 代码量 | 复杂度 | 说明 |
|:---|:---|:---|:---|
| Lexer | ~300 行 | 🟢 低 | 关键字、标识符、数字、运算符、字符串 |
| Parser（递归下降） | ~600 行 | 🟡 中 | 表达式优先级、语句解析、函数定义 |
| AST 节点定义 | ~200 行 | 🟢 低 | ~20 种 AST 节点类型 |
| TypeChecker | ~400 行 | 🟡 中 | 类型推导、类型兼容性检查 |
| **BytecodeGen** | **~1200 行** | **🔴 高** | **栈机代码生成、内存布局、控制流、指针步长、float 指令** |
| Source Map | ~100 行 | 🟢 低 | 指令偏移 → 源码位置映射 |
| 内置函数（print_int 等） | ~50 行 | 🟢 低 | 宿主导入的辅助函数 |
| **合计** | **~4000 行** | | |

### 6.2 降低风险的策略

**风险**：BytecodeGen 是编译器中最复杂的部分（~1200 行）。

**缓解方案**（已全部验证有效）：

| 策略 | 说明 | 效果 |
|:---|:---|:---|
| **Phase 1 缩小子集** | 先实现变量+数组+函数+指针+if/while/for | 减少 ~30% CodeGen 工作量 ✅ |
| **Rust 枚举 AST** | 用 enum 替代 C++ 多态类层次 | 减少内存管理错误，Borrow Checker 保障安全 ✅ |
| **端到端测试驱动** | 每增加一个语法特性，立即添加 E2E 测试 | 早发现错误，防止回归 ✅ |

---

## 7. 推荐实施方案

### 7.1 Phase 1：核心子集（已完成）

支持：
```c
int a = 5;
int arr[10];
int arr[] = {1, 2, 3};
int* p = &a;

if (a > 5) { }
while (a < 10) { }
for (int i = 0; i < n; i++) { }

int foo(int x) { return x + 1; }
int main() { return 0; }
```

**教学能力**：变量、数组、基本指针、控制流、函数、递归。

### 7.2 Phase 2：扩展子集（已完成）

新增：
- struct、malloc/free（简化版）
- 内置输出函数（`print_int`、`__cide_output`）
- 内存视图与内存泄漏检测

**教学能力**：链表、树、动态内存、内存泄漏检测。

### 7.3 Phase 3：核心语法扩展（已完成）

新增：
- `do...while`、`break` / `continue`
- `switch` / `case` / `default`（支持 fallthrough）
- `char` 类型与字符串字面量（Data Segment 注入）
- `sizeof`（编译期常量，返回 4）
- `typedef`（类型别名）
- `enum`（编译期常量）
- `unsigned` / `signed`（语义上与 int 相同）
- 数组/字符串初始化列表（`int a[] = {1,2,3};` / `char s[] = "hello";`）
- `printf` / `scanf`（宿主导入函数）
- 隐式转换警告机制（不阻断编译，提示类型安全问题）

**教学能力**：完整的控制流、字符串操作、类型系统、格式化 I/O。

### 7.4 Phase 4：可选增强（根据反馈）

- [x] **多维数组** — 已支持声明、嵌套初始化列表、索引访问、函数参数传递（如 `int[][3]`）
- [x] **结构体初始化**（`struct Node n = {10, &a};`）— 已支持完整/部分初始化，含指针字段
- [x] **函数前向声明** — 已支持 `int foo(int);` 原型声明，实现可放在调用者之后
- [x] **字符串库函数** — 已支持 `strlen` / `strcpy` / `strcmp` / `strcat`（宿主导入函数）
- [x] **显式类型转换（Cast）** — 已支持 `(int*)p`、`(char*)arr`、`(float)a` 等标量/指针间转换
- [x] **预处理器（宏定义）** — 已支持 `#define` 简单常量替换
- [x] **位运算** — 已支持 `& | ^ ~ << >>`
- [x] **三目运算符** — 已支持 `? :`
- [x] **指针算术** — 已支持 `p++` / `p+i` / `p-q`，自动按 pointee 大小缩放
- [x] **`const` 语义** — 已支持直接变量 `const`，阻止赋值和自增/自减
- [x] **`NULL` 关键字** — 已支持，`NULL` 被解析为 `(void*)0`
- [x] **新增宿主函数** — `getchar`/`putchar`/`rand`/`srand`/`memset`/`exit`/`strcat`/`atoi`
- [x] **`fprintf`/`realloc`/`qsort`** — 已支持
- [x] **函数指针基础** — 已支持函数名作为参数传递（如 `qsort(..., cmp)`）
- [ ] 函数指针完整支持（声明变量、赋值）
- [x] **`double` 类型** — 已支持完整 64 位精度

---

## 8. 结论

### 对于一个教学场景，多少合适？

**答案**：

> **足够演示 C 语言的核心概念（变量、控制流、函数、指针、内存），但排除会分散注意力的进阶特性和实现复杂度高的语法。**

### 黄金法则

1. **如果去掉这个特性，学生还能理解 C 的灵魂吗？**
   - 指针（&、*）→ **不能去掉**
   - break/continue → **教学价值高，已支持**（循环控制必备）

2. **这个特性会增加多少编译器复杂度？**
   - for 循环 → 复杂度中等，但教学价值极高 → **保留**
   - float/double → 复杂度中等，教学价值中等（数值计算入门）→ **保留** ✅ 已实现

3. **学生第一次接触这个特性时会困惑吗？**
   - `int a, b;` → 可能困惑（为什么可以一行两个？）→ **已支持** ✅（`int a = 1, b = 2;`）
   - `p++` vs `arr[i++]` → 需要理解步长缩放，但已支持并带教学提示 → **保留**

### 最终推荐的 Cide-C 子集（Phase 1 ~ 3 完整版）

```
数据类型：int、char、float、double、unsigned、int*、char*、float*、double*、int[]、char[]、double[]、struct、enum
类型系统：typedef、sizeof、const
语句：变量声明、赋值、if/else、while、do...while、for、switch/case/default、
       break、continue、return、块作用域
表达式：算术、比较、逻辑、位运算、赋值、三目运算符、数组索引、函数调用、&、*、
        struct访问、++/--、字符串字面量、sizeof、显式类型转换
函数：定义/调用/递归/前向声明
内存：malloc/free/realloc
I/O：printf、scanf、fprintf、getchar、putchar
字符串：strlen、strcpy、strcmp、strcat、atoi
其他：rand/srand/memset/exit/qsort

不支持：bitfield、goto、文件I/O、extern/static/volatile/restrict、
       完整预处理器（仅 #define 常量宏）
```

这个范围覆盖了 C 语言的核心教学价值（变量、控制流、函数、指针、内存、字符串、类型系统），
同时保持了编译器实现的可控性（~3500 行代码）。
