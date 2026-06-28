# Cide 模板系统规范与提示规划

> 本文档面向模板维护者与前端开发者，定义模板目录结构、占位符语法、测试防线及未来改进方向。

## 1. 目录结构

每个模板位于 `templates/<key>/` 目录下：

```
templates/
  <key>/
    meta.yaml     # 模板元数据
    source.c      # C 模板源码（二选一）
    source.cpp    # C++ 模板源码（二选一）
```

`source.c` 与 `source.cpp` 只能存在一个，由 `sync_templates.py` 自动识别并写入 `index.json` 的 `ext` 字段。

## 2. meta.yaml 字段

```yaml
key: bubble                    # 目录名，必须唯一
name: 冒泡排序                  # 展示名称
category: 排序                  # 分类，用于分组展示

params:                        # 参数定义（可选）
  n:
    label: 数组长度
    type: int                  # int | string | identifier
    default: 5                 # 默认值，用于 shadow 用例生成

tutorial:                      # 教程步骤（可选）
  steps:
    - title: 外层循环
      description: 控制排序轮数
      anchor: outer_loop       # 对应源码中的 // @tutorial-anchor: outer_loop

knowledge_nodes:               # 关联知识图谱节点（可选）
  - Algorithm
  - Sorting
```

## 3. 占位符语法

模板参数在源码中以合法 C 注释形式出现，便于默认状态下直接被 Clang/Cide 编译：

```c
int arr[/*__PARAM_n__*/ 5] = {1, 2, 3, 4, 5};
int n = /*__PARAM_n__*/ 5;
```

- `/*__PARAM_<key>__*/` 为占位符标记。
- 占位符后的第一个非空标记为默认值。
- 当前支持的默认值字符集：`[^\s\[\]();,]+`，可覆盖数组大小、变量初始化、函数参数等场景。

## 4. 同步机制

运行 `python scripts/sync_templates.py` 会：

1. 扫描 `templates/` 下所有模板。
2. 用默认参数渲染生成 `native/tests/cases_template_generated/<key>_default.c`。
3. 用 Clang 编译运行生成 golden `.out`（仅 C 模板）。
4. 生成/更新 `CideFlutter/assets/templates/index.json`。
5. 复制 `source.c/source.cpp` 到 `CideFlutter/assets/templates/<key>.c/.cpp`。

**每次修改模板源码或 meta.yaml 后，必须重新运行 `sync_templates.py`**。

## 5. 测试防线

### 5.1 静态一致性测试

```bash
python scripts/test_templates.py
```

校验内容：
- 每个模板目录包含 `meta.yaml` 和 `source.c/source.cpp`。
- `meta.yaml` 中 `key/name/category` 存在且 `key` 与目录名一致。
- `meta.yaml` 中声明的 params 与源码中的占位符一一对应。
- 每个 param 都有 `default` 值。
- `CideFlutter/assets/templates/index.json` 与 `templates/` 同步。
- assets 中源码文件与源码目录内容一致。

### 5.2 Dart 单元/Widget 测试

```bash
cd CideFlutter
flutter test test/models/template_loader_test.dart
flutter test test/widgets/template_bar_test.dart
flutter test test/widgets/ide_template_bar_test.dart
flutter test test/providers/ide_notifier_test.dart
```

覆盖点：
- `TemplateLoader` 正确加载 C/C++ 模板及 `ext` 字段。
- 源码缺失时抛出 `TemplateLoadException`。
- `CodeTemplate.buildCode` 替换占位符。
- `completeTutorial` 根据模板扩展名切换 `main.c` / `main.cpp`。

### 5.3 Shadow Verification

模板生成的 `cases_template_generated/<key>_default.c` 会被纳入 Shadow Verification，Golden 来自 Clang。

## 6. 本次修复的诚实记录

### 6.1 C++ 模板无法加载
**现象**：`CideFlutter/assets/templates/` 中存在 `cpp_*.cpp`，但 `TemplateLoader.load()` 只尝试加载 `.c`，导致 C++ 模板被静默跳过。

**修复**：
- `TemplateLoader` 读取 `index.json` 的 `ext` 字段，加载 `.c` 或 `.cpp`。
- `CodeTemplate` 增加 `ext` 字段。
- `completeTutorial` 根据 `ext` 将当前文件切换为 `main.c` 或 `main.cpp`，确保 Rust 后端按正确语言模式编译。

### 6.2 教程无法退出 / 运行状态混乱
**现象**：点击模板进入教程后，点击“跳过”或“运行代码”，底部教程面板理论上应消失，但顶部的 `ExecutionControlPanel` 仍显示上一段运行的进度条/覆盖率，造成“还在模板里”的错觉。

**根因**：
- `IdeState.copyWith` 无法通过 `activeTutorial: null` 清除教程状态（被 `?? this.activeTutorial` 覆盖）。
- 教程模式下未隐藏 `ExecutionControlPanel`，上一段运行的残留状态会与新界面叠加。

**修复**：
- `IdeState.copyWith` 增加 `clearActiveTutorial` 标志。
- `completeTutorial` 使用 `clearActiveTutorial: true` 正确退出教程。
- `ide_screen.dart` 在 `activeTutorial != null` 时隐藏 `ExecutionControlPanel`。

### 6.3 覆盖率显示超过 100%
**现象**：执行控制面板显示“覆盖率 633.3%”。

**根因**：VM 执行热力图会记录标准库/预编译字节码的行号，导致 `lineCounts` 中出现远超当前源码行号的条目。

**修复**：
- 给 `cide_shared::SourceLoc` 增加 `file_id: i32` 字段（0 表示用户主文件，非 0 表示外部文件/标准库），旧产物通过 `#[serde(default)]` 保持兼容。
- Bytecode Libc 加载器 (`cide_vm::bytecode_libc_loader`) 在加载预编译产物后，将所有 libc 指令的 `loc.file_id` 置为 1。
- VM 执行器 (`cide_vm::core::executor`) 记录 heatmap 时只统计 `file_id == 0` 且 `line > 0` 的指令，从源头避免外部行号混入。
- `ExecutionControlPanel._buildCoverageText` 移除前端按 `totalLines` 过滤的绕过逻辑，heatmap 数据本身已只包含用户源码行号。

**诚实记录**：`SourceLoc` 字面量构造点众多（Parser/CodeGen/测试等），本次通过批量脚本补全 `file_id: 0` 完成迁移；后续新增构造应优先使用 `SourceLoc::new(line, column)` 或 `SourceLoc::with_file(line, column, file_id)`。

### 6.4 模板源码自身缺陷
| 模板 | 问题 | 修复 |
|------|------|------|
| `factorial` | `meta.yaml` 声明 `n` 参数，但 `source.c` 未使用占位符 | 在 `factorial()` 调用处加入 `/*__PARAM_n__*/ 5` |
| `fib` | 同上 | 在 `fibonacci()` 调用处加入 `/*__PARAM_n__*/ 7` |
| `merge` | `merge()` 函数定义在 `mergeSort()` 之后，Clang 报隐式声明 | 调整函数顺序 |
| `threadedBinaryTree` | 线索化遍历缺少终止条件，导致无限循环 | 引入标准头节点法遍历 |

## 7. 未来改进规划

### 7.1 模板提示增强（P1）
- **参数输入提示**：在 `TemplateParam` 中增加 `hint` 字段，参数对话框展示填写示例与取值范围。
- **模板选择提示**：`TemplateBar` 支持长按/悬停显示模板简短说明。
- **搜索与分类**：模板数量增加后，增加分类下拉与关键字搜索。

### 7.2 占位符能力扩展（P2）
- 支持多行默认值（如初始化列表 `/*__PARAM_arr__*/ {1,2,3}`）。
- 支持同一参数在源码中出现多次（当前已支持，正则会全部替换）。
- 支持条件占位符，如 `/*__PARAM_lang__==cpp*/ ... /*__PARAM_END__*/`。

### 7.3 教程系统升级（P2）
- `meta.yaml` 支持 `explanations` 完整写入，不再依赖 Dart 硬编码。
- 教程步骤支持高亮多行与变量提示。
- 教程结束时自动打开 Output 面板，避免学生看不到运行结果。

### 7.4 后端热图过滤（P3）
- 当前覆盖率过滤在前端完成；长期应在 Rust VM 记录 heatmap 时区分用户代码与标准库行号，只记录用户源码范围。

## 8. 相关命令速查

```bash
# 校验模板静态一致性
python scripts/test_templates.py

# 同步模板到 Flutter assets 并生成 golden
python scripts/sync_templates.py

# 运行模板相关 Dart 测试
cd CideFlutter
flutter test test/models/template_loader_test.dart test/widgets/ide_template_bar_test.dart test/providers/ide_notifier_test.dart
```
