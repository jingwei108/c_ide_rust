# 本地分支决策记录

> 制定时间：2026-06-15
> 依据：`docs/current/ROADMAP_2026_Q3.md` B2.1 分支整理任务
> 目标：本地分支数 ≤ 5 个

---

## 决策结果汇总

| 分支名 | 决策 | 理由 | 操作 |
|---|---|---|---|
| `feat/editor-refactor` | 删除 | 已全部合并到 `master`，无独有提交 | `git branch -D` |
| `feat/memory-expand-vfs` | 删除 | 已全部合并到 `master`，无独有提交 | `git branch -D` |
| `feat/post-code-review-followup` | 删除 | 已全部合并到 `master`，无独有提交 | `git branch -D` |
| `feat/unified-mode` | 删除 | 已全部合并到 `master`，无独有提交 | `git branch -D` |
| `flutter-migration` | 删除 | 已全部合并到 `master`，无独有提交 | `git branch -D` |
| `refactor/recursive-type-system` | 删除 | 已全部合并到 `master`，无独有提交 | `git branch -D` |
| `feature/react-native-webview` | 删除 | 废弃技术方向；含已标记 `DEPRECATED` 的 Tauri 提交 | `git branch -D` |
| `feature/tauri-migration` | 删除 | 废弃技术方向；提交已标记 `DEPRECATED` | `git branch -D` |
| `feature/flutter-webview` | 保留 | 含未合并的 WebView CM6 编辑器实验提交，作为历史方案参考；若 2026 Q3 末仍未采用则删除 | 保留 |

---

## 详细说明

### 已合并分支（6 个）

通过 `git log --oneline master..<branch>` 验证，以下分支相对于 `master` 均无独有提交，说明其内容已通过 PR/merge 进入 `master`：

- `feat/editor-refactor`
- `feat/memory-expand-vfs`
- `feat/post-code-review-followup`
- `feat/unified-mode`
- `flutter-migration`
- `refactor/recursive-type-system`

这些分支仅保留历史引用价值，当前 `master` 已包含其全部工作成果，可安全删除。

### 废弃分支（2 个）

- **`feature/react-native-webview`**：
  - 独有提交 `6fe43ca` 为 Flutter + WebView CM6 迁移计划文档。
  - 独有提交 `429f9f0` 明确标记 `DEPRECATED`，是 Tauri + TS 迁移脚手架。
  - 当前项目已选择纯 Flutter + 自研 `CideEditor` 路线，该分支无保留必要。

- **`feature/tauri-migration`**：
  - 独有提交 `429f9f0` 明确标记 `DEPRECATED`。
  - Tauri + TypeScript 方案已被放弃。

### 犹豫/保留分支（1 个）

- **`feature/flutter-webview`**：
  - 独有提交 `d883894`、`928f8de` 为 WebView CM6 编辑器前端实验。
  - 犹豫原因：当前主分支使用自研 `CideEditor`（CustomPainter），但 WebView 方案在输入法和复杂文本渲染上仍有参考价值；若未来需要重新评估编辑器实现，该分支可提供完整历史对比。
  - 决策：暂时保留，设置 2026 Q3 末（约 2026-08-31）为再次 review 时间点；届时若仍未采用则删除。

---

## 删除后分支预期

保留分支：

- `master`
- `feature/flutter-webview`
- `remotes/origin/master`

本地分支数：2 个（含 `master`），满足路线图 ≤ 5 的要求。

---

## 操作命令（供维护者核对）

```bash
# 删除已合并分支
git branch -D feat/editor-refactor
git branch -D feat/memory-expand-vfs
git branch -D feat/post-code-review-followup
git branch -D feat/unified-mode
git branch -D flutter-migration
git branch -D refactor/recursive-type-system

# 删除废弃分支
git branch -D feature/react-native-webview
git branch -D feature/tauri-migration

# 保留
git branch  # 应仅剩 master、feature/flutter-webview 及远程跟踪分支
```
