# 错误提示增强、一键修复与输出复制功能

> 日期：2026-05-10  
> 涉及平台：Desktop (Avalonia) + Maui (Android) + Rust Native

---

## 1. 错误提示增强

### 问题
- 诊断面板始终显示"无诊断信息"（Rust 后端 `diagnostics` Vec 从未被填充）
- 错误位置不准确：
  - 拼接编译单元时多追加了一个 `\n`，导致行号偏移 +1
  - Parser `consume` 在缺失分号时，报错位置是下一个 token（如下一行的 `return`），而不是分号应该出现的位置

### 修复

#### Rust 后端 (`native/src/capi/mod.rs`)

**1. 填充结构化诊断数据**

`cide_compile_all` 在编译出错时，只把格式化字符串写入了 `session.compile.errors`，但 `session.compile.diagnostics` 被 `clear()` 后从未重新填充。

- 新增 `CompileError` trait，统一抽象 `LexerError` / `ParseError` / `TypeError`
- 修改 `push_diagnostics`，在生成错误字符串的同时，将结构化诊断 push 到 `session.compile.diagnostics`

```rust
fn push_diagnostics<T: CompileError>(session: &mut Session, errors: &[T], source: &str)
```

**2. 修复行号偏移**

拼接编译单元时，之前无条件追加 `\n`：

```rust
full_source.push_str(&unit.source);
full_source.push('\n');   // 如果 source 已有换行，会多出一行
```

修复后，只在不以换行符结尾时才追加：

```rust
full_source.push_str(&unit.source);
if !unit.source.ends_with('\n') {
    full_source.push('\n');
}
```

**3. 修复分号缺失的报错位置**

Parser 的 `consume` 方法在期望 `Semicolon` 但失败时，原实现使用 `self.current()`（即下一个 token）的位置：

```rust
line: self.current().line,
column: self.current().column,
```

修复后，当缺失分号时，使用 `self.previous()` token 的位置（表达式实际结束位置）：

```rust
let (err_line, err_column) = if ty == TokenType::Semicolon && self.pos > 0 {
    (self.previous().line, self.previous().column)
} else {
    (self.current().line, self.current().column)
};
```

#### Desktop 端 (`Cide.Client`)

**1. 诊断卡片点击导航**

- `MainViewModel.cs`：新增 `JumpToDiagnosticCommand`，设置 `HighlightedLine = diag.Line`
- `MainView.axaml`（两处）：诊断卡片 `Border` 添加 `Cursor="Hand"`、`PointerPressed="OnDiagnosticPressed"`、`ToolTip.Tip="点击跳转到错误位置"`、悬停透明度效果
- `MainView.axaml.cs`：新增 `OnDiagnosticPressed`，将诊断项转发给 ViewModel

**2. 编辑器错误行红色高亮**

之前只有行号区有红色背景，编辑器内容区没有。

- `CodeEditor.axaml.cs`：
  - 新增 `HighlightedLine` StyledProperty，变化时自动滚动到对应行并聚焦编辑器
  - 新增 `ErrorLineBackgroundRenderer`（实现 `IBackgroundRenderer`），在编辑器内容区绘制红色半透明背景（`#22FF4444`）
  - 注册到 `Editor.TextArea.TextView.BackgroundRenderers`

**3. 编辑器滚动到高亮行**

`HighlightedLine` 变化时：
- 设置 `CaretOffset` 到目标行首
- 调用 `BringCaretToView()` 滚动到视口
- 调用 `Editor.Focus()` 聚焦编辑器，方便直接修改

#### Maui 端 (`Cide.Client.Maui`)

**1. 诊断卡片点击导航**

- `Home.razor`：诊断卡片 `<div>` 添加 `@onclick="() => OnDiagnosticClick(diag)"`
- `Home.razor` (`@code`)：新增 `OnDiagnosticClick` 方法，设置 `VM.HighlightedLine` 并调用 `_editor.ScrollToLine()`
- `CodeMirrorEditor.razor`：新增 `ScrollToLine(int line)`，调用 JS interop
- `codemirror-interop.js`：新增 `scrollToLine(id, line)`，设置选区到目标行首，计算滚动位置使该行居中，并 `focus()` 编辑器

**2. CSS 交互反馈**

- `app.css`：`.diag-item` 添加 `cursor: pointer`、`:active` 时的透明度/微缩放反馈动画

---

## 2. 一键修复功能

### 问题
Rust 后端生成错误时，`fix_suggestion` / `fix_kind` / `replace_*` / `replacement_text` 字段全部为空，导致 C# 前端的"应用修复"按钮永远不会显示。

### 修复

#### Rust 后端 (`native/src/capi/mod.rs`)

在 `push_diagnostics` 中，根据错误码为常见错误自动生成修复建议和结构化替换坐标：

| 错误码 | 场景 | FixSuggestion | FixKind | 结构化操作 |
|--------|------|---------------|---------|-----------|
| **E2005** | 缺少分号 | "语句末尾缺少分号，建议添加 ';'" | ReplaceText(1) | 在行末插入 `;`（`start == end`） |
| **E3023** | 变量未声明 | "变量未声明，建议先声明变量再使用" | ManualHint(4) | 仅提示 |
| **E3015** | 条件表达式不合法 | "建议检查是否误用 '=' 代替 '=='" | ManualHint(4) | 仅提示 |

```rust
2005 => {
    let line_idx = (e.line() as usize).saturating_sub(1);
    let line_text = source_lines.get(line_idx).unwrap_or(&"");
    let trimmed_len = line_text.trim_end().len() as i32;
    ("语句末尾缺少分号，建议添加 ';'".to_string(), 1,
     e.line(), trimmed_len, e.line(), trimmed_len, ";".to_string())
}
```

#### C# 端 (`Cide.Client.Shared/Core/CodeFixService.cs`)

`ApplyStructuredReplace` 原先拒绝 `startCol >= endCol`（认为是无效范围），现在改为：

```csharp
// startCol == endCol means insert; startCol > endCol is invalid
if (startCol < 0 || endCol > line.Length || startCol > endCol)
    return new CodeFixResult(false, null, string.Empty);
```

支持 `start == end` 的**插入操作**，完美适配"在行末添加分号"的场景。

#### 效果

编译出错后，诊断卡片现在显示：
- 💡 修复建议文案
- **"应用修复"按钮**（`FixSuggestion` 非空时显示）

点击按钮 → `CodeFixService.TryApplyFix` → 结构化替换 → 代码自动更新 → 编辑器内容刷新。

---

## 3. 输出框可复制功能

### Desktop (Avalonia)

| 改动 | 文件 |
|------|------|
| `TextBlock` → `TextBox`（只读） | `MainView.axaml`（两处：右侧面板 + 底部面板） |
| 支持鼠标选中 + `Ctrl+C` 复制 | `TextBox` 默认行为 |
| 添加 📋 复制按钮（浮动右上角） | `MainView.axaml` + `MainView.axaml.cs` |
| 点击一键复制全部输出到剪贴板 | `OnCopyOutputClick` 通过 `TopLevel.GetTopLevel(this)?.Clipboard` 写入 |

### Maui (Blazor WebView)

| 改动 | 文件 |
|------|------|
| `user-select: text` / `-webkit-user-select: text` | `app.css`（`.console-output`） |
| 确保移动端长按可选中复制 | CSS 属性 |
| 添加 📋 复制按钮（绝对定位右上角） | `Home.razor` + `app.css` |
| 点击一键复制全部输出 | `CopyOutputAsync` 使用 `Microsoft.Maui.ApplicationModel.DataTransfer.Clipboard.Default.SetTextAsync()` |

---

## 验证结果

| 项目 | 结果 |
|------|------|
| Rust 测试 | ✅ 75/75 通过 |
| C# 单元测试 | ✅ 3/3 通过 |
| Desktop Debug 构建 | ✅ 0 警告 0 错误 |
| Maui Debug 构建 | ✅ 0 警告 0 错误 |
| Android `.so` (arm64 Release) | ✅ 构建成功 |
| Android `.so` (armv7 Release) | ✅ 构建成功 |

完整 APK 构建命令：

```powershell
.\test-mobile.ps1 -Configuration Release -Install -Run
```
