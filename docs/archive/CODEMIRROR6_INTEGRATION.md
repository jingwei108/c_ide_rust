# CodeMirror 6 集成记录

## 背景

Monaco Editor v0.52 在 MAUI BlazorWebView Android 环境中语法高亮完全失效（详见 [ARCHIVE_MONACO_SYNTAX_HIGHLIGHT_WEBVIEW_FALLBACK.md](./ARCHIVE_MONACO_SYNTAX_HIGHLIGHT_WEBVIEW_FALLBACK.md)）。经过评估，决定移除 Monaco，集成 CodeMirror 6。

## 方案选择

使用 `GaelJ.BlazorCodeMirror6` NuGet 包装器（v0.10.0），原因：
- 支持 .NET 6/7/8/9，含 MAUI Hybrid 目标
- C/C++ 语法高亮内置（`CodeMirrorLanguage.Cpp`）
- 行号、括号匹配、代码折叠、主题切换均通过参数配置
- 社区确认有 MAUI Hybrid 使用案例

## 集成步骤

### 1. 安装 NuGet 包
```bash
dotnet add package GaelJ.BlazorCodeMirror6 --version 0.10.0
```

### 2. 添加命名空间引用
`Components/_Imports.razor`：
```razor
@using GaelJ.BlazorCodeMirror6
@using GaelJ.BlazorCodeMirror6.Models
```

### 3. 创建编辑器组件
`Components/Editor/CodeMirrorEditor.razor`：
- 使用 `<CodeMirror6Wrapper>` 组件
- 配置 `Language="CodeMirrorLanguage.Cpp"`
- 启用 `LineNumbers="true"`、`HighlightActiveLine="true"`、`FoldGutter=true`
- 支持 `ThemeMirrorTheme.OneDark` / `GithubLight` 主题切换

### 4. 替换 Home.razor
移除 `MonacoEditor` 和 `SimpleEditor`，统一使用 `CodeMirrorEditor`。

### 5. 清理 Monaco
删除以下文件/目录：
- `wwwroot/monaco/`（11.24 MB）
- `wwwroot/js/monaco-interop.js`
- `wwwroot/css/monaco.css`
- `Components/Editor/MonacoEditor.razor`
- `Components/Editor/SimpleEditor.razor`

---

## 遇到的坑

### 坑 1：MAUI Hybrid CSS Bundle 路径问题（GitHub Issue #210）

**现象**：CM6 包装器在运行时尝试加载：
```
_content/GaelJ.BlazorCodeMirror6/GaelJ.BlazorCodeMirror6.bundle.scp.css
```

但 MAUI Blazor Hybrid 的静态资源打包会为文件名追加 content hash，实际文件为：
```
_content/GaelJ.BlazorCodeMirror6/GaelJ.BlazorCodeMirror6.ewb2sj01iq.bundle.scp.css
```

导致 CSS 加载 404，编辑器样式缺失。

**解决**：双重保障
1. 在项目 `wwwroot/_content/GaelJ.BlazorCodeMirror6/` 下放一份不带 hash 的 CSS 副本
2. 在 `.csproj` 中显式添加 `MauiAsset` 项确保文件被打包进 APK：
```xml
<MauiAsset Include="wwwroot\_content\GaelJ.BlazorCodeMirror6\GaelJ.BlazorCodeMirror6.bundle.scp.css"
           LogicalName="wwwroot/_content/GaelJ.BlazorCodeMirror6/GaelJ.BlazorCodeMirror6.bundle.scp.css" />
```

验证：APK 中同时包含带 hash 和不带 hash 的 CSS 文件。

---

### 坑 2：Blazor `@` 绑定符号遗漏

**现象**：编辑器永远显示字符串字面量 `_internalCode`，不显示实际代码内容。

**截图特征**：第 1 行显示 `_internalCode`，有语法高亮（说明 CM6 工作），但内容错误。

**根因**：Blazor 属性绑定漏写 `@` 符号。

```razor
<!-- 错误：传递字符串字面量 "_internalCode" -->
Doc="_internalCode"

<!-- 正确：绑定 C# 变量 _internalCode 的值 -->
Doc="@_internalCode"
```

没有 `@` 时，Blazor 将属性值视为纯文本字符串，而不是 C# 表达式。

**修复**：补上 `@` 符号。

---

### 坑 3：双向绑定循环导致输入重置

**现象**：用户输入代码到一定数量后，编辑器内容突然重置为旧值，后续输入无效。

**根因**：Blazor 渲染循环自我覆盖。

1. 用户输入 → CM6 触发 `DocChanged` → `OnDocChanged` 更新 `_internalCode` → `CodeChanged.InvokeAsync`
2. `Home.razor` 收到变化 → `VM.SourceCode = code` → `PropertyChanged` → `StateHasChanged()`
3. `Home.razor` 重新渲染 → 传入 `Code` 参数 → `CodeMirrorEditor.OnParametersSet`
4. `OnParametersSet` 发现 `_internalCode != Code` → 强制覆盖 `_internalCode` → CM6 的 `Doc` 被重置
5. 编辑器内容被强制恢复为旧值

**修复**：添加 `_isInternalChange` 标志位，区分"内部编辑触发"和"外部参数传入"。

```csharp
private bool _isInternalChange;

protected override void OnParametersSet()
{
    // 只有外部传入的新代码才更新，编辑自身触发的 re-render 跳过
    if (!_isInternalChange && !string.Equals(_internalCode, Code))
    {
        _internalCode = Code;
    }
    _isInternalChange = false;
}

private async Task OnDocChanged(string? newCode)
{
    if (newCode != null && !string.Equals(newCode, _internalCode))
    {
        _isInternalChange = true;  // 标记变化来自编辑器内部
        _internalCode = newCode;
        await CodeChanged.InvokeAsync(newCode);
    }
}
```

---

## 当前状态

| 功能 | 状态 |
|------|------|
| C/C++ 语法高亮 | ✅ 正常（Lezer 解析器） |
| 行号显示 | ✅ 正常 |
| 括号匹配 | ✅ 正常 |
| 代码折叠 | ✅ 正常 |
| 主题切换（dark/light） | ✅ 正常 |
| 代码编辑 | ✅ 正常（@绑定 + 循环修复后） |
| 断点 gutter | ⏳ TODO（包装器不直接暴露自定义 gutter API） |
| 错误行/当前行高亮 | ⏳ TODO（需通过 JS interop 操作 EditorView） |

## 相关文件

| 文件 | 说明 |
|------|------|
| `Components/Editor/CodeMirrorEditor.razor` | CM6 编辑器封装组件 |
| `Components/Pages/Home.razor` | 使用 CodeMirrorEditor 替代 MonacoEditor |
| `Components/_Imports.razor` | 添加 GaelJ.BlazorCodeMirror6 命名空间 |
| `Cide.Client.Maui.csproj` | 添加 NuGet 包引用 + MauiAsset workaround |
| `wwwroot/_content/GaelJ.BlazorCodeMirror6/GaelJ.BlazorCodeMirror6.bundle.scp.css` | CSS bundle 路径 workaround |
