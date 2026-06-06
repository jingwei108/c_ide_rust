# Flutter 模板运行时加载状态记录

> **状态：Phase 4 已完成。** 旧 Dart 硬编码模板框架已完全移除，运行时加载成为唯一途径。

## 已完成

- `pubspec.yaml` 注册 `assets/templates/` 和 `assets/templates/index.json`
- `TemplateLoader` 异步加载 `index.json` + `.c` 文件构建 `CodeTemplate`
- `ide_screen.dart` / `learning_path_panel.dart` 已接入异步加载
- `CodeTemplate.buildCode` 仅支持新占位符语法 `/*__PARAM_key__*/ defaultValue`
- `CideFlutter/lib/models/templates/*.dart` 已删除（11 个硬编码文件）
- `template_registry.dart` 已清理：移除 `allTemplates` fallback 及全部旧 import

## 历史问题与解决状态

### 1. `explanations`（逐行代码解释） ✅ 已解决

- **解决方式**（2026-06-06）:
  1. `sync_templates.py` 新增 `extract_all_dart_tutorials()`，从 Dart 硬编码模板中完整提取 `focusLines` + `explanations`
  2. 生成 `index.json` 时优先使用 Dart 数据，fallback 到 `meta.yaml`
  3. `TemplateLoader` 解析 `explanations` 数组并构建 `LineExplanation` 列表

### 2. `focusLines` 精度不足 ✅ 已解决

- **解决方式**（2026-06-06）: 同 explanations，从 Dart 硬编码模板中提取完整的 `focusLines` 列表写入 `index.json`。

### 3. 硬编码模板 fallback ✅ 已移除（2026-06-06）

- ~~`template_registry.dart` 中的 `allTemplates` 已标记 `@deprecated`~~
- ~~保留作为 `getDynamicTemplates()` 加载失败时的 fallback~~
- `allTemplates` 及全部 `templates/*.dart` 硬编码文件已彻底删除
- `getDynamicTemplates()` 现在直接返回 `TemplateLoader.load()`，无 fallback

## 数据完整性验证

- 2026-06-06: 82 个模板全部通过对比验证
- `index.json` 中的 `tutorialSteps`（含 `focusLines` + `explanations`）与 Dart 硬编码模板完全一致
- 0 个 mismatches
