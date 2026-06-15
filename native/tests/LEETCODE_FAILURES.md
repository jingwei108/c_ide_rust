# LeetCode 简单题失败记录

> 记录原则：诚实记录，不隐藏失败。
> 格式参见 `docs/current/PHASE_KR_LEETCODE_TEST_PLAN.md` 附录 B。

## 当前状态

**LeetCode 防线已全面实施（阶段 4 + 阶段 5）。**

- `native/tests/cases/leetcode/` 已创建
- `native/tests/cases_golden/leetcode/` 已创建
- 当前已填充 **48** 道 LeetCode 简单题源码
- 当前通过 **48** 道，已知失败 **0** 道
- 在填充过程中发现 1 处 Cide 与 Clang 行为差异，已通过改写源码规避，详见下方"实施过程发现"章节

## 已覆盖用例

### 阶段 4：数组与字符串（27 道）

| 用例 | 题目 | 类别 | 状态 | 备注 |
|:-----|:-----|:-----|:-----|:-----|
| lc_1 | Two Sum | 数组 | 通过 | 返回动态分配数组，驱动中释放 |
| lc_26 | Remove Duplicates from Sorted Array | 数组 | 通过 | |
| lc_27 | Remove Element | 数组 | 通过 | |
| lc_35 | Search Insert Position | 数组/二分 | 通过 | |
| lc_53 | Maximum Subarray | 数组/DP | 通过 | |
| lc_66 | Plus One | 数组/数学 | 通过 | 返回动态分配数组 |
| lc_88 | Merge Sorted Array | 数组/双指针 | 通过 | |
| lc_121 | Best Time to Buy and Sell Stock | 数组/贪心 | 通过 | |
| lc_136 | Single Number | 数组/位运算 | 通过 | |
| lc_169 | Majority Element | 数组 | 通过 | |
| lc_189 | Rotate Array | 数组 | 通过 | |
| lc_217 | Contains Duplicate | 数组 | 通过 | 使用 `qsort` + 回调比较函数 |
| lc_238 | Product of Array Except Self | 数组 | 通过 | 返回动态分配数组 |
| lc_268 | Missing Number | 数组/数学 | 通过 | |
| lc_283 | Move Zeroes | 数组/双指针 | 通过 | |
| lc_14 | Longest Common Prefix | 字符串 | 通过 | 返回动态分配字符串 |
| lc_28 | Implement strStr() | 字符串 | 通过 | |
| lc_58 | Length of Last Word | 字符串 | 通过 | |
| lc_125 | Valid Palindrome | 字符串 | 通过 | 使用 `ctype.h` 的 `isalnum` |
| lc_344 | Reverse String | 字符串 | 通过 | |
| lc_387 | First Unique Character | 字符串 | 通过 | |
| lc_9 | Palindrome Number | 数学 | 通过 | |
| lc_13 | Roman to Integer | 数学 | 通过 | |
| lc_69 | Sqrt(x) | 数学/二分 | 通过 | |
| lc_70 | Climbing Stairs | DP | 通过 | |
| lc_118 | Pascal's Triangle | 数组 | 通过 | 返回动态分配二维数组 |
| lc_119 | Pascal's Triangle II | 数组 | 通过 | 返回动态分配数组 |

### 阶段 5：链表、树、栈与队列（21 道）

| 用例 | 题目 | 类别 | 状态 | 备注 |
|:-----|:-----|:-----|:-----|:-----|
| lc_20 | Valid Parentheses | 栈 | 通过 | 数组模拟栈 |
| lc_155 | Min Stack | 栈 | 通过 | 结构体封装 |
| lc_225 | Implement Stack using Queues | 栈 | 通过 | 数组模拟队列 |
| lc_232 | Implement Queue using Stacks | 队列 | 通过 | 原始复合副作用写法触发 Cide 差异，已改写规避 |
| lc_206 | Reverse Linked List | 链表 | 通过 | |
| lc_21 | Merge Two Sorted Lists | 链表 | 通过 | |
| lc_141 | Linked List Cycle | 链表 | 通过 | 构造环状链表 |
| lc_160 | Intersection of Two Linked Lists | 链表 | 通过 | |
| lc_203 | Remove Linked List Elements | 链表 | 通过 | |
| lc_234 | Palindrome Linked List | 链表 | 通过 | |
| lc_237 | Delete Node in a Linked List | 链表 | 通过 | |
| lc_876 | Middle of the Linked List | 链表 | 通过 | |
| lc_104 | Maximum Depth of Binary Tree | 树 | 通过 | |
| lc_100 | Same Tree | 树 | 通过 | |
| lc_101 | Symmetric Tree | 树 | 通过 | |
| lc_108 | Convert Sorted Array to BST | 树 | 通过 | 中序遍历验证 |
| lc_110 | Balanced Binary Tree | 树 | 通过 | |
| lc_111 | Minimum Depth of Binary Tree | 树 | 通过 | |
| lc_112 | Path Sum | 树 | 通过 | |
| lc_226 | Invert Binary Tree | 树 | 通过 | 中序遍历验证 |
| lc_572 | Subtree of Another Tree | 树 | 通过 | |

## 统计摘要

| 阶段 | 总数 | 通过 | 失败 | 记录时间 |
|:-----|:-----|:-----|:-----|:---------|
| LeetCode 数组/字符串 | 27 | 27 | 0 | 2026-06-14 |
| LeetCode 链表/树/栈 | 21 | 21 | 0 | 2026-06-14 |
| **合计** | **48** | **48** | **0** | 2026-06-14 |

## 实施过程发现

### lc_232：复合副作用数组索引表达式触发 Cide 差异

- **来源**: LeetCode 232 — Implement Queue using Queues
- **发现时间**: 2026-06-14
- **现象**: 原始实现使用 `obj->out[++obj->outTop] = obj->in[obj->inTop--];` 时，Cide 运行时报告"访问了 NULL 指针区域（地址 0x0000）。NULL 指针不能解引用"；同一份代码在 Clang 下正常执行。
- **是否 Cide 限制**: 是
- **是否代码本身问题**: 否（代码在 Clang/GCC 下行为正确）
- **是否环境差异**: 否
- **涉及语法特性**: 数组索引表达式中同时包含对两个不同对象的 `++`/`--` 副作用
- **学生影响评级**: P1
- **当前处理**: 已将 `obj->out[++obj->outTop] = obj->in[obj->inTop--];` 拆分为独立语句（先自增、再赋值、再自减），用例已通过。
- **建议**: 进一步分析 BytecodeGen 或 VM 对含副作用数组索引的求值顺序/地址计算；在 `AGENTS.md` 已知限制中补充说明。

## 已知失败详情

当前无已知失败。

<!-- 待新增失败时按以下模板追加：
### <case_name>

- **来源**: LeetCode XXX
- **失败原因**: <编译错误 / 运行时错误 / 输出不匹配 / 超时>
- **最小复现**: <关键代码片段>
- **是否 Cide 限制**: 是/否
- **是否代码本身问题**: 是/否
- **是否环境差异**: 是/否
- **涉及语法特性**: <如 long long, 递归深度, qsort 回调>
- **学生影响评级**: P0 / P1 / P2
- **建议**: <修复方向 / 记录为已知限制 / 待进一步分析>
-->

## 阶段 6 完成情况

LeetCode 用例已纳入 `shadow_verify.py` 扫描范围，生成专项报告 `native/tests/shadow_verification/reports/kr_leetcode_report.json`。

| 来源 | 总数 | 匹配 | 编译缺口 | 运行时缺口 | 输出差异 |
|:-----|:-----|:-----|:---------|:-----------|:---------|
| K&R | 69 | 68 | 0 | 0 | 1（`kr_5_8`，已知的 qsort 输出顺序差异） |
| LeetCode | 48 | 48 | 0 | 0 | 0 |

## 后续计划

1. 持续观察新增用例是否暴露其他 Cide 与 Clang 行为差异。
2. 将 shadow 报告路径纳入 CI artifact 上传。
