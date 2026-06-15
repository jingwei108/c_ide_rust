# LeetCode 简单题失败记录

> 记录原则：诚实记录，不隐藏失败。
> 格式参见 `docs/current/PHASE_KR_LEETCODE_TEST_PLAN.md` 附录 B。

## 当前状态

**LeetCode 防线已启动（阶段 4：数组/字符串简单题）。**

- `native/tests/cases/leetcode/` 已创建
- `native/tests/cases_golden/leetcode/` 已创建
- 当前已填充 **10** 道 LeetCode 简单题，全部通过
- 本文件暂无具体失败条目

## 已覆盖用例

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

## 统计摘要

| 阶段 | 总数 | 通过 | 失败 | 记录时间 |
|:-----|:-----|:-----|:-----|:---------|
| LeetCode 数组/字符串 | 10 | 10 | 0 | 2026-06-14 |
| LeetCode 链表/树/栈 | 0 | 0 | 0 | - |

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

## 后续计划

1. 继续填充阶段 4 剩余数组/字符串简单题（约 15 道）。
2. 进入阶段 5：链表、树、栈与队列简单题。
3. 将 LeetCode 用例纳入 `shadow_verify.py` 扫描范围，生成差异报告。
