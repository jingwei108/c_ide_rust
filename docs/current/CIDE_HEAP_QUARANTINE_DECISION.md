# 设计决议：堆内存改为 Bump 分配 + 永久隔离（Permanent Quarantine）

> 决议日期：2026-09-11
> 状态：**已拍板**（回应"伪 GC"PR：拒绝真 GC 方向，采纳其动机、替换其手段）
> 归属：主计划 [`CIDE_BACKEND_SPLIT_WASM_WHITEBOX_PLAN.md`](CIDE_BACKEND_SPLIT_WASM_WHITEBOX_PLAN.md) 的引擎层决策，Phase 1 内可实施（与切割无依赖）

---

## 1. 决策

Cide 堆从"free_list + 合并复用"分配器改为 **bump 分配 + 永久隔离**：`malloc` 顶指针 O(1) 推进、地址永不复用；`free` 保留全部检测语义（记录/判定/诊断）但**不归还空间**；进程（VM 实例）结束随 1MB 线性内存整体回收。

业界先例：ASAN 的 quarantine（隔离区）机制——free 的块延迟复用以保 UAF 检测窗口。本方案是**永久隔离**的教学加强版，不是对 C 语义的妥协。

## 2. 动机

| 维度 | 收益 |
|---|---|
| UAF | freed 块内容永久保留，`free(p)` 后访问 `*p` **永远可检出**（无地址复用关闭检测窗口的漏报） |
| Double-Free | 地址唯一性使 freed_logs 不被新分配污染，检测确定性最强 |
| 泄漏 | regions 表即泄漏清单（免 free_list 扣除维护），报告从推导变直读 |
| 确定性 | 内存状态单调只增：时间旅行快照（CoW，V-P1-9）脏页更少；判分重放不受 free/malloc 交错历史影响 |
| 性能/复杂度 | host_free 的 free_list 维护与 `merge_free_list` 全部删除；malloc 变 O(1)（收敛 V-P1-11 一半问题） |
| 兜底 | 无限分配撞三道墙，每道都是教学 trap：VM 1MB 上限 → `set_max_steps` 步数保险丝 → V-P1-12 无效 free 诊断 |

## 3. 语义变更

| 操作 | 旧 | 新 |
|---|---|---|
| `malloc` | free_list 查找 + 合并 | 顶指针推进；地址永不复用 |
| `free` | 标记 + 归还 free_list | **标记 + 不归还**；freed_logs / 泄漏判定 / E3027·E3061 诊断全部保留 |
| `realloc` | 原地扩展或搬移 | 恒为新块拷贝（与 glibc 常见路径一致） |
| 内存可视化 | 分配块/空闲块（含外部碎片图） | **已分配/已释放（隔离）双色**——UAF 教学画面反而更清晰 |

## 4. 已知差异与代价（诚实记录）

1. **"free 后再 malloc 得到同地址"的教学演示不再成立**——写入 `C_SUBSET_SPEC.md` 已知差异："Cide 堆采用永久隔离策略以强化 UAF 检测，free 后地址不复用（同 ASAN quarantine）"。此差异是检测器的指纹，非缺陷；
2. **外部碎片教学话题消失**——Phase 14 的碎片可视化 UI 资产弃用或改造为泄漏堆叠图，记录于 CHANGELOG；
3. **防线 3 语义对齐**——host_contract_tests（3a）、differential_stress（3c）、fuzz E 的 free 语义断言按新语义重写（这是把"分配器复用行为"从契约中除名的正规流程，非粉饰）。

## 5. 边界推导（regions 单调增长）

最坏场景 `malloc(1)` 循环：每条 `MemoryRegionData` 约几十字节 host 内存，纯记录表可达堆本身数倍。但每次 malloc 至少消耗数个 VM 步，默认 `max_steps = 100_000` 先触发——**步数保险丝天然封顶 region 数量**（约数万条 ≈ 数 MB host 内存）。契约化：`set_max_steps` 同时封顶 region 表大小，写入 API 文档。

## 6. 验收清单

- [ ] host_malloc/host_free/host_realloc 按新语义实现，free_list/merge 代码移除；
- [ ] UAF / Double-Free / 泄漏报告 / 无效 free 诊断在新模型下全部回归通过（`crash_regression_tests.rs` 补永久隔离专项用例：free 后读、free 后地址不复用断言）；
- [ ] 防线 3a/3c/fuzz E 断言重写并全绿；
- [ ] `C_SUBSET_SPEC.md` 已知差异补录（§4-1）；CHANGELOG 记录碎片可视化资产处置；
- [ ] 无限分配三道墙用例（1MB 耗尽 / max_steps / region 表封顶推导验证）；
- [ ] Shadow 门禁全绿（含 K&R/LeetCode 内存密集用例——预期地址值类断言为零，无判分风险）。

## 7. 对"伪 GC"PR 的处置

**拒绝真 GC 方向**：GC 自动回收会掩盖学生最需要学的错误——本项目核心教学资产就是 free 语义的教学（泄漏报告、UAF/Double-Free 知识卡片），GC 等于把考点删了。采纳其动机（内存管理简化），以本决议的 bump + 永久隔离替换其手段：不帮学生收拾，但把每一次没收拾的后果变成可见的教学信号。
