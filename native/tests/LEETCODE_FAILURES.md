# LeetCode 简单题失败记录

> 记录原则：诚实记录，不隐藏失败。
> 格式参见 `docs/current/PHASE_KR_LEETCODE_TEST_PLAN.md` 附录 B。

## 当前状态

**LeetCode 防线当前尚未实施。**

- `native/tests/cases/leetcode/` 目录不存在
- `native/tests/cases_golden/leetcode/` 目录不存在
- 本文件为占位模板，无具体失败条目

## 为什么还没做

LeetCode 防线是 `docs/current/PHASE_KR_LEETCODE_TEST_PLAN.md` 中规划的内容，原计划作为 Phase 28 的一部分推进。但在实际迭代过程中，项目优先级调整为**集中攻克 C++ 子集扩展**（Phase 31+）。C++ 扩展涉及 Lexer、Parser、AST、TypeChecker、BytecodeGen、VM 全管线的重大改造，对学生教学的长期价值更高，因此 LeetCode 防线**暂时搁置**。

## 什么时候做

LeetCode 防线仍是项目计划的一部分。待 C++ 子集进入稳定维护期后，将按 `PHASE_KR_LEETCODE_TEST_PLAN.md` 重新启动：

1. 创建 `native/tests/cases/leetcode/` 目录
2. 填充 LeetCode 简单题源码（数组/字符串、链表/树/栈等）
3. 使用 Clang 生成 `native/tests/cases_golden/leetcode/` 期望输出
4. 在 `native/tests/cide_e2e.rs` 中填充 `KNOWN_LEETCODE_FAILURES`
5. 将本文件更新为真实的失败记录

## 诚实记录声明

本文件明确记录 LeetCode 防线当前不在，以避免"已有"描述造成误解。这不是数据粉饰，而是项目优先级调整的真实状态。K&R 防线（69 个用例）已全部跑通，当前真实程序回归防线仅由 K&R 构成。

## 统计摘要

| 阶段 | 总数 | 通过 | 失败 | 记录时间 |
|:-----|:-----|:-----|:-----|:---------|
| LeetCode 数组/字符串 | 0 | 0 | 0 | - |
| LeetCode 链表/树/栈 | 0 | 0 | 0 | - |

## 已知失败详情

<!-- 待 LeetCode 防线启动后按以下模板逐条追加 -->

<!--
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
