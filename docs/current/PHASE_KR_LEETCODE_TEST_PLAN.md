# Phase 28：K&R C 经典例题 + LeetCode 简单题测试覆盖计划

> **核心原则**：测试不是为了标榜通过率，而是为了诚实地发现自己可能存在的问题。Cide 的错误会误导学生，所以我们宁可数据难看，也不扭曲代码去迎合编译器。
>
> ** motto**: *All in. Record don't hide. Fix real bugs, not test cases.*

---

## 0. 背景与动机

当前 Cide 已有 **278** 个 baseline E2E 用例（全部通过）和 **82** 个模板生成用例（74 通过，8 个已知失败并记录在案）。这套基础设施已经证明了"记录问题而非粉饰"的文化是可行的（见 `GOLDEN_FAILURES.md`、`KNOWN_TEMPLATE_FAILURES`）。

但 baseline 用例以"原子语法特性"为主，缺乏**真实的、有教学价值的完整程序**作为回归防护，且本项目为**闭源开发，缺乏用户反馈**，本开发团队也不想消耗用户时间去发现问题，甚至该过程可能出现误导情况。K&R C 是 C 语言教学的事实标准，LeetCode 是算法练习的事实标准。将这两者全部纳入测试框架，能暴露现有原子测试无法发现的：**语义组合缺陷、边界条件遗漏、标准行为偏差**。

---

## 1. 关键原则（不可妥协）

### 1.1 ALL_IN — 全部纳入，不预先筛选

- 即使某 K&R 例题或 LeetCode 题目明显使用了 Cide 不支持的语法（如 `goto`、bitfield、`argc/argv`、文件 I/O），也**必须**纳入测试。
- 不因为"明知会失败"就不放进去。失败本身就是最有价值的信息。

### 1.2 NO_CODE_DISTORTION — 不扭曲代码迎合编译器

对 K&R 原始代码和 LeetCode 标准解法，**禁止**以下修改：
- 把 `int* p = &a;` 改成 `int p = a;` 来绕过指针问题
- 把递归改成循环来绕过递归深度问题
- 把 `struct` 改成多个全局变量来绕过复合类型问题
- 把标准库调用替换成 Cide 特有的 workaround

**唯一允许的机械性修改**：
- **LeetCode 包装**：LeetCode 是函数签名，必须添加标准 `main()` 驱动。驱动代码必须简单、机械、透明（见附录 A）。
- **多文件合并**：K&R 某些例题天然多文件，允许合并到单文件，但不得改变函数实现。
- **补充 `return 0;`**：K&R 第一版原始代码可能省略 `return 0;`，这在现代 C 中本就是标准要求，补充它不算扭曲。

### 1.3 RECORD_DONT_HIDE — 记录问题，不隐藏失败

任何失败必须记录到对应的 `*_FAILURES.md`，注明：
- **失败原因**：编译错误 / 运行时错误 / 输出不匹配 / 超时 / 崩溃
- **Cide 限制？** 是否因为 Cide 不支持某语法导致
- **代码本身问题？** 是否 K&R/LeetCode 代码本身有 bug（如教材缺陷）
- **环境差异？** 是否因为沙盒 stdin/stdout 行为与原生环境不同
- **涉及语法特性**：如 `argc/argv`、`long long`、`隐式函数声明` 等
- **学生影响评级**：`P0`（会误导学生，必须尽快修复）/ `P1`（限制已知，可接受）/ `P2`（边缘场景）

### 1.4 GOLDEN_FROM_CLANG — Golden 输出以 Clang 为准

- 所有 golden `.out` 文件优先使用 Clang 生成。
- 若 Clang 和 Cide 都失败但失败原因不同，记录差异，不强行生成 golden。

---

## 2. 基础设施准备（阶段 0）

### 2.1 目录结构

```
native/tests/
├── cases/
│   ├── baseline/               # 已有 278 个
│   ├── knr/                    # 新增：K&R 例题源码
│   └── leetcode/               # 新增：LeetCode 简单题源码
├── cases_golden/
│   ├── baseline/               # 已有
│   ├── knr/                    # 新增：K&R 期望输出
│   ├── leetcode/               # 新增：LeetCode 期望输出
│   └── gap/                    # 已有，复用
├── KR_FAILURES.md              # 新增：K&R 失败记录
├── LEETCODE_FAILURES.md        # 新增：LeetCode 失败记录
└── shadow_verification/
    └── cases/
        ├── knr/                # 影子验证用例（复用或引用 cases/knr/）
        └── leetcode/           # 影子验证用例
```

### 2.2 测试入口扩展

在 `native/tests/cide_e2e.rs` 中新增：

```rust
const KNOWN_KR_FAILURES: &[&str] = &[
    // 阶段 1~3 逐步填充
];

const KNOWN_LEETCODE_FAILURES: &[&str] = &[
    // 阶段 4~5 逐步填充
];

#[test]
fn test_cide_e2e_knr() { ... }

#[test]
fn test_cide_e2e_leetcode() { ... }

#[test]
fn test_cide_e2e_knr_known_failures() { ... }  // 监控已知失败是否意外通过

#[test]
fn test_cide_e2e_leetcode_known_failures() { ... }
```

### 2.3 影子验证扩展

修改 `shadow_verify.py`：
- 支持从 `cases/knr/` 和 `cases/leetcode/` 自动加载 `.c` 文件（而非内联在 Python 列表中）。
- 增加分类统计：`knr_compile_gap`、`knr_runtime_gap`、`leetcode_compile_gap` 等。
- 生成 `shadow_verification/reports/kr_leetcode_report.json`。

---

## 3. 分阶段实施计划

### 阶段 1：K&R 第 1-2 章（语言基础与运算符）

**目标**：约 25 道，覆盖变量、控制流、数组、函数、字符串基础、位运算。

**候选清单**（按章节）：

| 编号 | 题目 | 章节 | 关键语法 | 风险预估 |
|:---|:---|:---|:---|:---|
| kr_1_3 | 华氏-摄氏温度转换表 | 1.2 | 循环, printf | 低 |
| kr_1_4 | 摄氏-华氏温度转换表 | 1.2 | 循环, printf | 低 |
| kr_1_5 | 温度转换表（倒序） | 1.3 | for 循环 | 低 |
| kr_1_6 | 验证表达式取值 | 1.5 | 逻辑表达式 | 低 |
| kr_1_7 | 打印 EOF 值 | 1.5 | printf | 低 |
| kr_1_8 | 统计空白/制表/换行 | 1.5 | 字符输入, if | 中（getchar 行为）|
| kr_1_9 | 替换连续空格为单个 | 1.5 | 状态机 | 低 |
| kr_1_10 | 转义字符可视化 | 1.5 | switch, 转义 | 低 |
| kr_1_11 | 单词计数程序测试 | 1.5 | 单词边界 | 低 |
| kr_1_12 | 每行一个单词输出 | 1.5 | 状态机 | 低 |
| kr_1_13 | 单词长度直方图 | 1.6 | 数组 | 低 |
| kr_1_14 | 字符频率直方图 | 1.6 | 数组 | 低 |
| kr_1_15 | 温度转换（函数版） | 1.7 | 函数 | 低 |
| kr_1_16 | 最长行 | 1.9 | 数组, 字符串 | 低 |
| kr_1_17 | 打印长度大于 80 的行 | 1.9 | 字符串 | 低 |
| kr_1_18 | 删除行尾空格/制表/空行 | 1.9 | 字符串处理 | 低 |
| kr_1_19 | 字符串反转（reverse 函数） | 1.9 | 字符串, 函数 | 低 |
| kr_2_3 | htoi（十六进制转 int） | 2.3 | 循环, 字符处理 | 低 |
| kr_2_4 | squeeze（删除字符） | 2.8 | 字符串 | 低 |
| kr_2_5 | any（查找字符） | 2.8 | 字符串 | 低 |
| kr_2_6 | setbits（设置位字段） | 2.9 | 位运算 | 低 |
| kr_2_7 | invert（位反转） | 2.9 | 位运算 | 低 |
| kr_2_8 | rightrot（循环右移） | 2.9 | 位运算 | 低 |
| kr_2_9 | bitcount（统计 1 的位数） | 2.9 | 位运算 | 低 |
| kr_2_10 | lower（条件表达式版） | 2.11 | 三目运算 | 低 |

**预期**：90%+ 应通过。潜在失败：
- `getchar()` 相关题目（1.8, 1.9 系列）：Cide 沙盒中 stdin 为空，可能需要预置输入字符串替换 `getchar()`，或记录为"环境差异"。**注意**：若替换 `getchar` 为从字符串读取，需在 `*_FAILURES.md` 中注明"因沙盒无 stdin，使用字符串模拟输入"。
- `INT_MIN` 边界：若 K&R 练习涉及 `-n` 当 `n == INT_MIN` 时的溢出行为，记录为"实现限制"。

---

### 阶段 2：K&R 第 3-4 章（控制流、函数、程序结构）

**目标**：约 20 道，覆盖二分查找、排序、递归、宏、静态变量、作用域。

**候选清单**：

| 编号 | 题目 | 章节 | 关键语法 | 风险预估 |
|:---|:---|:---|:---|:---|
| kr_3_1 | 折半查找（binsearch） | 3.3 | while, 数组 | 低 |
| kr_3_2 | escape（转义函数） | 3.4 | switch, 字符串 | 低 |
| kr_3_3 | 展开连续范围（如 0-9） | 3.3 | 循环, 数组 | 低 |
| kr_3_4 | itoa（处理最大负数） | 3.6 | 字符串, 溢出 | 中（INT_MIN）|
| kr_3_5 | itob（整数转任意进制） | 3.5 | 字符串, 数学 | 低 |
| kr_3_6 | itoa（指定宽度） | 3.6 | 字符串, 填充 | 低 |
| kr_3_7 | trim（删除尾部空白） | 3.5 | 字符串 | 低 |
| kr_3_8 | 快速 reverse（一次循环） | 3.5 | 字符串 | 低 |
| kr_4_1 | strrindex（最右出现） | 4.1 | 字符串 | 低 |
| kr_4_2 | atof（科学计数法） | 4.2 | 字符串, float | 中（atof 精度）|
| kr_4_3 | 栈计算器（expr, 基础版） | 4.3 | 栈, switch, static | 中 |
| kr_4_4 | 栈计算器（增加运算符） | 4.4 | 栈, static | 中 |
| kr_4_5 | 栈计算器（增加数学函数） | 4.5 | 函数指针? | 高（若涉及 math.h）|
| kr_4_6 | 栈计算器（处理变量） | 4.6 | 外部变量 | 中 |
| kr_4_7 | ungets（回退整个字符串） | 4.7 | static 数组, 指针 | 低 |
| kr_4_8 | 递归打印十进制数 | 4.10 | 递归 | 低 |
| kr_4_9 | 递归快速排序 | 4.10 | 递归, 指针 | 低 |
| kr_4_10 | 递归 itoa | 4.10 | 递归, 字符串 | 低 |
| kr_4_11 | 汉诺塔 | 4.10 | 递归 | 低 |
| kr_4_12 | 递归 printd | 4.10 | 递归 | 低 |
| kr_4_13 | 递归 reverse（字符串） | 4.10 | 递归, 指针 | 低 |
| kr_4_14 | 宏 swap(t,x,y) | 4.11 | 参数化宏 | 低 |

**预期**：80%+ 应通过。潜在失败：
- `static` 局部变量：Cide Phase 27 已完成支持，应该通过。
- `expr` 计算器：依赖 `getch`/`ungetch` 和 `getchar`。需要预置输入字符串，或记录为"沙盒 stdin 限制"。
- `atof` / 科学计数法：若涉及 `double` 精度问题，记录为"精度限制"。
- 涉及 `math.h`（如 `sin`, `pow`）的题目：Cide 可能不支持。记录为"缺少数学库"。

---

### 阶段 3：K&R 第 5-6 章（指针与结构体）

**目标**：约 20 道，覆盖指针版字符串函数、指针数组、命令行参数、结构体、表查找。

**候选清单**：

| 编号 | 题目 | 章节 | 关键语法 | 风险预估 |
|:---|:---|:---|:---|:---|
| kr_5_1 | getint（读取整数，返回状态） | 5.1 | 指针参数 | 低 |
| kr_5_2 | getfloat（读取浮点数） | 5.2 | 指针参数, float | 中 |
| kr_5_3 | 指针版 strcat | 5.3 | 指针算术 | 低 |
| kr_5_4 | 指针版 strend | 5.4 | 指针算术 | 低 |
| kr_5_5 | 指针版 strncpy/strncat/strncmp | 5.5 | 指针算术 | 低 |
| kr_5_6 | 指针版 atoi, itoa, reverse, strindex | 5.6 | 指针 | 低 |
| kr_5_7 | 指针版 readlines（排序前奏） | 5.6 | 指针数组, malloc | 中 |
| kr_5_8 | 快速排序（指针数组版） | 5.6 | 指针数组, 函数指针, qsort | 低 |
| kr_5_9 | 快速排序（递减序） | 5.11 | 函数指针 | 低 |
| kr_5_10 | echo（命令行参数） | 5.10 | argc/argv | **高** |
| kr_5_11 | 打印月份名（指针数组） | 5.9 | 指针数组 | 低 |
| kr_5_12 | 处理负宽度（entab/detab） | 5.11 | 字符串 | 低 |
| kr_5_13 | tail（打印最后 n 行） | 5.6 | 指针数组, malloc | 中 |
| kr_5_14 | 排序（增加字段选项） | 5.11 | 函数指针, 结构体 | 中 |
| kr_5_15 | 查找（增加选项） | 5.11 | 函数指针 | 中 |
| kr_5_16 | dcl/undcl（复杂声明解析） | 5.12 | 复杂语法 | **高**（Parser 可能不支持）|
| kr_6_1 | getword（提取关键字） | 6.3 | 结构体数组 | 低 |
| kr_6_2 | 统计 C 关键字（二叉树） | 6.5 | 结构体, 树, 递归 | 低 |
| kr_6_3 | 交叉引用（单词+行号） | 6.3 | 结构体, 链表 | 低 |
| kr_6_4 | 统计单词频率 | 6.4 | 结构体, 树 | 低 |
| kr_6_5 | 哈希表查找 | 6.6 | 结构体, 哈希 | 低 |
| kr_6_6 | 表查找（define/undef） | 6.6 | 结构体, 链表 | 低 |

**预期**：70%+ 应通过。关键风险点：
- **`argc/argv`**：Cide VM 目前可能不支持命令行参数传入。若不支持，**所有涉及 `argc/argv` 的题目（echo、find 等）将编译或运行失败**。这是本次测试最可能暴露的 P0 级缺失特性，必须诚实记录。
- **`dcl`/`undcl`**：涉及 `int (*(*x[3])())[5]` 这种复杂声明。Cide Parser 可能不支持函数返回指针数组、数组的数组等复杂组合。若失败，记录 Parser 限制。
- `getint`/`getfloat`：依赖 `getchar`，stdin 问题同上。

---

### 阶段 4：LeetCode 简单题 - 数组与字符串

**目标**：约 25 道，覆盖数组操作、字符串处理、简单数学、二分查找。

**候选清单**（从 Top Interview 150 Easy 筛选，要求可用 Cide C 子集表达）：

| 编号 | 题目 | 类别 | 关键语法 | 风险 |
|:---|:---|:---|:---|:---|
| lc_1 | Two Sum | 数组 | 数组, 循环 | 低 |
| lc_26 | Remove Duplicates from Sorted Array | 数组 | 数组, 双指针 | 低 |
| lc_27 | Remove Element | 数组 | 数组, 双指针 | 低 |
| lc_35 | Search Insert Position | 数组 | 数组, 二分 | 低 |
| lc_53 | Maximum Subarray | 数组 | 数组, 贪心/DP | 低 |
| lc_66 | Plus One | 数组 | 数组, 进位 | 低 |
| lc_88 | Merge Sorted Array | 数组 | 数组, 双指针 | 低 |
| lc_121 | Best Time to Buy and Sell Stock | 数组 | 数组, 贪心 | 低 |
| lc_136 | Single Number | 数组 | 数组, 位运算 | 低 |
| lc_169 | Majority Element | 数组 | 数组 | 低 |
| lc_189 | Rotate Array | 数组 | 数组, 反转 | 低 |
| lc_217 | Contains Duplicate | 数组 | 数组, 哈希? | 中（若无哈希可用排序）|
| lc_238 | Product of Array Except Self | 数组 | 数组, 前缀积 | 低 |
| lc_268 | Missing Number | 数组 | 数组, 数学 | 低 |
| lc_283 | Move Zeroes | 数组 | 数组, 双指针 | 低 |
| lc_14 | Longest Common Prefix | 字符串 | 字符串, 循环 | 低 |
| lc_28 | Implement strStr() | 字符串 | 字符串, KMP/暴力 | 低 |
| lc_58 | Length of Last Word | 字符串 | 字符串 | 低 |
| lc_125 | Valid Palindrome | 字符串 | 字符串, 双指针 | 低 |
| lc_344 | Reverse String | 字符串 | 字符串, 双指针 | 低 |
| lc_387 | First Unique Character | 字符串 | 字符串, 数组计数 | 低 |
| lc_9 | Palindrome Number | 数学 | 数学, 反转 | 低 |
| lc_13 | Roman to Integer | 数学 | 字符串, 映射 | 低 |
| lc_69 | Sqrt(x) | 数学 | 二分 | 低 |
| lc_70 | Climbing Stairs | DP | 数组/变量, DP | 低 |
| lc_118 | Pascal's Triangle | 数组 | 数组, 二维 | 低 |
| lc_119 | Pascal's Triangle II | 数组 | 数组, 一维 DP | 低 |

**预期**：85%+ 应通过。潜在失败：
- 涉及 `long long` 的题目（如大数运算）：Cide 对 `long` / `long long` 的支持程度待验证。
- 涉及 `INT_MIN` 取绝对值溢出的题目：记录为"实现限制"。
- 涉及 `qsort` 的排序比较函数：Cide 已支持 `qsort`，但需验证回调函数调用是否正确。

---

### 阶段 5：LeetCode 简单题 - 链表、树、栈与队列

**目标**：约 20 道，覆盖链表操作、树遍历、简单递归、栈模拟。

**候选清单**：

| 编号 | 题目 | 类别 | 关键语法 | 风险 |
|:---|:---|:---|:---|:---|
| lc_20 | Valid Parentheses | 栈 | 数组模拟栈 | 低 |
| lc_155 | Min Stack | 栈 | 结构体, 数组 | 低 |
| lc_225 | Implement Stack using Queues | 栈 | 数组模拟 | 低 |
| lc_232 | Implement Queue using Stacks | 队列 | 数组模拟 | 低 |
| lc_206 | Reverse Linked List | 链表 | 指针, 递归/迭代 | 低 |
| lc_21 | Merge Two Sorted Lists | 链表 | 指针, 递归 | 低 |
| lc_141 | Linked List Cycle | 链表 | 指针, 双指针 | 低 |
| lc_160 | Intersection of Two Linked Lists | 链表 | 指针 | 低 |
| lc_203 | Remove Linked List Elements | 链表 | 指针 | 低 |
| lc_234 | Palindrome Linked List | 链表 | 指针, 递归/反转 | 低 |
| lc_237 | Delete Node in a Linked List | 链表 | 指针 | 低 |
| lc_876 | Middle of the Linked List | 链表 | 指针, 双指针 | 低 |
| lc_104 | Maximum Depth of Binary Tree | 树 | 递归 | 低 |
| lc_100 | Same Tree | 树 | 递归 | 低 |
| lc_101 | Symmetric Tree | 树 | 递归 | 低 |
| lc_108 | Convert Sorted Array to BST | 树 | 递归, 数组 | 低 |
| lc_110 | Balanced Binary Tree | 树 | 递归 | 低 |
| lc_111 | Minimum Depth of Binary Tree | 树 | 递归 | 低 |
| lc_112 | Path Sum | 树 | 递归 | 低 |
| lc_226 | Invert Binary Tree | 树 | 递归 | 低 |
| lc_572 | Subtree of Another Tree | 树 | 递归, 字符串匹配 | 低 |

**预期**：85%+ 应通过。潜在失败：
- 递归深度过大的树题（如退化为链表的树）：可能栈溢出，记录为"VM 栈限制"。
- 涉及 `NULL` 与空树边界：Cide 的 `NULL` 处理已成熟，但需验证。

---

### 阶段 6：影子验证全覆盖与差异基线

**任务**：
1. 将所有 `cases/knr/` 和 `cases/leetcode/` 的用例加入 `shadow_verify.py` 的扫描范围。
2. 运行完整影子验证，生成结构化报告：
   ```json
   {
     "summary": {
       "knr": { "total": 65, "match": 55, "compile_gap": 5, "runtime_gap": 3, "output_gap": 2 },
       "leetcode": { "total": 45, "match": 40, "compile_gap": 2, "runtime_gap": 2, "output_gap": 1 }
     },
     "gaps": [
       { "case": "kr_5_10_echo", "type": "compile_gap", "reason": "argc/argv not supported" },
       ...
     ]
   }
   ```
3. 按差异类型分类统计，输出到 `shadow_verification/reports/kr_leetcode_report.md`。
4. 更新 `docs/current/C_SUBSET_SPEC.md`，补充本次测试暴露的所有新限制。

---

### 阶段 7：自动化流水线与持续监控

**任务**：
1. **CI 工作流**：在 `.github/workflows/` 中确保 `cargo test --test cide_e2e` 覆盖 knr 和 leetcode。
2. **影子验证 CI**：新增步骤运行 `python native/tests/shadow_verification/shadow_verify.py --categories knr,leetcode`，上传报告为 artifact。
3. **报告生成**：扩展 `TEST_REPORT.md` 的生成逻辑，新增 K&R 和 LeetCode 汇总表格。
4. **已知失败监控**：
   - `KNOWN_KR_FAILURES` 和 `KNOWN_LEETCODE_FAILURES` 列表必须存在。
   - `test_cide_e2e_knr_known_failures` 测试确保：若已知失败意外通过，CI 失败提醒移除记录。
5. **文档更新**：
   - 更新 `AGENTS.md` 的"已知限制"章节，引用 `KR_FAILURES.md` 和 `LEETCODE_FAILURES.md`。
   - 在 `CHANGELOG.md` 中记录 Phase 28 的测试覆盖增量。

---

## 4. 预期成果与风险

### 4.1 预期新增用例数

| 类别 | 预计用例数 | 预计通过 | 预计失败（诚实记录）|
|:---|:---|:---|:---|
| K&R 第 1-2 章 | ~25 | ~23 | ~2 |
| K&R 第 3-4 章 | ~20 | ~16 | ~4 |
| K&R 第 5-6 章 | ~20 | ~14 | ~6 |
| LeetCode 数组/字符串 | ~25 | ~22 | ~3 |
| LeetCode 链表/树/栈 | ~20 | ~17 | ~3 |
| **合计** | **~110** | **~92** | **~18** |

> 注意："预计失败"不是目标，而是诚实的估计。实际运行后可能更多也可能更少。**无论多少，全部记录。**

### 4.2 高风险暴露点（可能发现 P0 缺陷）

以下特性若测试证明 Cide 与 Clang 行为不一致，应标记为 **P0**（误导学生）：
1. **`argc/argv` 缺失或行为错误**：学生写 `int main(int argc, char* argv[])` 是标准 C 的入门内容，若不支持必须明确告知。
2. **`long` / `long long` 语义偏差**：若 `long long` 被静默截断为 `int`，会导致学生写出在 Cide 上运行正常、在标准编译器上溢出的代码。
3. **隐式函数声明的处理**：C89 允许，C99 禁止。Cide 当前策略未知，需明确。
4. **`double` 精度或格式化输出偏差**：`printf("%.2f", 2.675)` 的舍入行为若与标准不同，会误导学生。
5. **指针算术或数组越界检查的遗漏**：若某些边界情况未触发 `TrapBounds`，可能掩盖真实 bug。

### 4.3 低风险已知限制（P1/P2）

- `goto`：教学子集明确不支持，失败预期之内。
- bitfield：明确不支持。
- 文件 I/O（`fopen`/`fread` 等）：沙盒环境限制，可接受。
- `volatile` / `restrict`：教学场景极少使用。

---

## 5. 单轮对话工作量估计

本项目严格遵守"一次对话完成一个可交付增量"。各阶段建议的每轮对话产出：

| 轮次 | 建议产出 |
|:---|:---|
| 第 1 轮 | **阶段 0**：基础设施（目录、测试入口、失败文档模板、LeetCode 驱动模板）。
| 第 2 轮 | **阶段 1**：K&R 第 1-2 章 25 道用例 + golden + 失败记录。
| 第 3 轮 | **阶段 2**：K&R 第 3-4 章 20 道用例 + golden + 失败记录。
| 第 4 轮 | **阶段 3**：K&R 第 5-6 章 20 道用例 + golden + 失败记录。
| 第 5 轮 | **阶段 4**：LeetCode 数组/字符串 25 道 + golden + 失败记录。
| 第 6 轮 | **阶段 5**：LeetCode 链表/树/栈 20 道 + golden + 失败记录。
| 第 7 轮 | **阶段 6**：影子验证全覆盖、差异报告生成。
| 第 8 轮 | **阶段 7**：CI 集成、TEST_REPORT 扩展、AGENTS.md 更新、CHANGELOG 记录。|

> 可根据实际进度合并或拆分。关键是每轮结束时有可验证的产出（测试能跑、失败有记录）。

---

## 附录 A：LeetCode 驱动模板规范

LeetCode 用例必须保留原始函数签名和实现**一字不改**，仅在其后追加机械化的 `main()` 驱动。

**模板示例（Two Sum）**：

```c
// ====== LeetCode 原始函数（禁止修改）======
int* twoSum(int* nums, int numsSize, int target, int* returnSize) {
    // ... 原始实现 ...
}

// ====== 测试驱动（机械化追加）======
int main() {
    int nums[] = {2, 7, 11, 15};
    int target = 9;
    int returnSize = 0;
    int* result = twoSum(nums, 4, target, &returnSize);
    
    for (int i = 0; i < returnSize; i++) {
        if (i > 0) printf(" ");
        printf("%d", result[i]);
    }
    printf("\n");
    
    free(result);  // 若原始函数用 malloc 分配返回数组
    return 0;
}
```

**驱动代码规则**：
1. 输入数据硬编码在 `main` 中（LeetCode 标准测试用例）。
2. 输出格式：数字用空格分隔，每行一个测试用例的输出。
3. 若函数返回链表/树，驱动中应遍历并打印节点值（如 `1 -> 2 -> 3` 打印为 `1 2 3`）。
4. 若函数是 `void`，驱动中检查副作用（如数组内容变化）并打印。
5. 所有驱动函数应尽可能相似，避免为单个题目写特殊逻辑。

---

## 附录 B：失败记录模板

新增失败时，在对应的 `*_FAILURES.md` 中按以下格式追加：

```markdown
### <case_name>

- **来源**: K&R 第 X 章 / LeetCode XXX
- **失败原因**: <编译错误 / 运行时错误 / 输出不匹配 / 超时>
- **最小复现**: <一行描述或关键代码片段>
- **是否 Cide 限制**: 是/否
- **是否代码本身问题**: 是/否
- **是否环境差异**: 是/否
- **涉及语法特性**: <如 argc/argv, long long, 隐式函数声明>
- **学生影响评级**: P0 / P1 / P2
- **建议**: <修复方向 / 记录为已知限制 / 待进一步分析>
```

---

*计划制定时间: 2026-06-06*
*下次启动建议: 从阶段 0（基础设施）开始，建立目录和测试入口。*
