# Cide 代码内问题追踪规范

## 目的

将隐形技术债务显式化，使维护者能快速定位已知待改进点、缺陷与临时 workaround。

## 注释标签

| 标签 | 含义 | 使用场景 | 示例 |
|------|------|----------|------|
| `TODO(#<issue>):` | 已知待改进点 | 有明确改进方向但当前未实现 | `// TODO(#D08): 将此处 unwrap 替换为 Result 传播` |
| `FIXME(#<issue>):` | 已知缺陷 | 代码能运行但行为/边界有问题 | `// FIXME(#D09): shouldRepaint 始终返回 true，导致每帧重绘` |
| `HACK:` | 临时 workaround | 为赶工期或绕过阻塞而采用的非理想方案 | `// HACK: 通过空字符串占位避免 None 分支，后续应改 Option` |
| `NOTE:` | 重要设计决策 | 非显而易见的实现选择，需要解释原因 | `// NOTE: 这里故意不用递归，避免深层嵌套栈溢出` |
| `SAFETY:` | unwrap/expect 合理性说明 | 对 clippy `unwrap_used`/`expect_used` 的豁免依据 | `// SAFETY: 前面已检查非空` |

## 编号约定

- 优先使用现有债务编号（如 `#D07`、`#D08`）或 GitHub issue 编号。
- 若无现成编号，可临时使用 `#DXX` 并在 `MAINTENANCE_PLAN.md` 中追加。

## 禁止行为

- 禁止用 TODO/FIXME 标记掩盖真实缺陷而不跟踪。
- 禁止过度标记（如每行都加 NOTE），应聚焦真正需要沟通的设计点。
- 禁止删除他人的 TODO/FIXME 而不说明原因；修复后应在提交信息中引用。

## 统计与 review

`scripts/lint_check.sh` 已包含 TODO/FIXME/HACK 数量统计，维护者可定期查看趋势。
