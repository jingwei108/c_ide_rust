# Release APK CodeMirror 6 加载失败修复

## 问题现象

Release APK（AOT + Trim + LLVM）安装到真机后：
- CodeMirror 6 编辑器显示骨架屏，持续不加载
- 所有按钮点击无反应（仅有系统触控反馈）
- Debug 模式下完全正常

## 排查过程

### 阶段 1：排除资源缺失

检查 APK 内容，确认 `GaelJ.BlazorCodeMirror6` 的静态资源全部打包：
- JS chunks（含 `index.js` 入口、`lib.module.js`）✅
- CSS bundle（`GaelJ.BlazorCodeMirror6.bundle.scp.css`）✅
- `_framework/blazor.webview.js` ✅

**结论**：不是资源缺失问题。

### 阶段 2：诊断 JS 运行时

在 `index.html` 注入诊断脚本，真机测试发现关键错误：

```
CM module import FAILED: Failed to resolve module specifier
'_content/GaelJ.BlazorCodeMirror6/index.js'
```

同时：
- `fetch('_content/.../index.js')` → HTTP 200 ✅
- `import('_content/.../index.js')` → 失败 ❌

**核心发现**：`fetch` 能成功，但 ES 模块 `import()` 无法解析相对路径。

### 阶段 3：定位根因

进一步诊断显示 WebView 的 `document.baseURI` 为：

```
https://0.0.0.1/
```

MAUI Blazor Hybrid 的 WebView 使用虚拟主机 `https://0.0.0.1/` 提供资源，而非 `file://` 协议。在此环境下，相对路径 `_content/...` 无法被 ES 模块加载器正确解析。

Debug 模式下之所以正常，是因为 `AddBlazorWebViewDeveloperTools()` 改变了资源提供方式（或开发者工具放宽了模块加载限制）。

## 修复方案

### 修复 1：codemirror-interop.js 使用绝对 URL

```javascript
// 修改前（相对路径，Release 下失败）
gaeljModulePromise = import('_content/GaelJ.BlazorCodeMirror6/index.js');

// 修改后（绝对 URL，Release 下成功）
var moduleUrl = new URL('_content/GaelJ.BlazorCodeMirror6/index.js', document.baseURI).href;
gaeljModulePromise = import(moduleUrl);
```

### 修复 2：TrimmerRootAssembly 保留 CodeMirror 类型

在 `Cide.Client.Maui.csproj` 中添加：

```xml
<ItemGroup>
    <TrimmerRootAssembly Include="GaelJ.BlazorCodeMirror6" RootMode="All" />
</ItemGroup>
```

防止 AOT trimming 裁剪 `GaelJ.BlazorCodeMirror6` 中的反射类型或 JS interop 方法。

### 修复 3：CSS workaround 文件复制

NuGet 包中只有带 hash 的 CSS 文件：`GaelJ.BlazorCodeMirror6.ewb2sj01iq.bundle.scp.css`

`MauiAsset` workaround 需要不带 hash 的版本作为源文件。从 NuGet 缓存复制并重命名：

```powershell
Copy-Item "...\GaelJ.BlazorCodeMirror6.ewb2sj01iq.bundle.scp.css" \
    "wwwroot\_content\GaelJ.BlazorCodeMirror6\GaelJ.BlazorCodeMirror6.bundle.scp.css"
```

## 验证结果

| 检查项 | 结果 |
|--------|------|
| 编辑器渲染 | ✅ 正常显示，无骨架屏 |
| 语法高亮 | ✅ C 语言关键字/字符串高亮正确 |
| 行号显示 | ✅ 1-8 行正常 |
| 工具栏按钮 | ✅ 运行/单步/停止/主题切换显示正常 |
| 代码编译执行 | ✅ `1+2+3+4+5=15` 输出正确 |
| APK 大小 | 81.02 MB（AOT+LLVM，105 程序集） |

## 关键教训

1. **WebView `import()` 与 `fetch()` 行为不同**：`fetch` 可通过 WebView 资源拦截器解析，但 ES 模块 `import()` 依赖浏览器模块解析器，在虚拟主机环境下相对路径可能失败。始终使用绝对 URL。
2. **Debug ≠ Release**：Debug 模式下的开发者工具可能隐式修复某些 WebView 行为差异，Release 测试不可或缺。
3. **NuGet RCL 资源文件名**：MAUI 对静态 web assets 使用 hash 文件名，但库内部可能引用 unhashed 文件名。`MauiAsset` workaround 需要本地存在对应的源文件。
4. **Trimmer 兼容性**：第三方 Blazor RCL（尤其是使用 JS interop 的库）在 Release AOT 下需要显式保留，避免类型被裁剪。

## 相关文件

- `Cide.Client.Maui/wwwroot/js/codemirror-interop.js`
- `Cide.Client.Maui/Cide.Client.Maui.csproj`
- `Cide.Client.Maui/wwwroot/_content/GaelJ.BlazorCodeMirror6/GaelJ.BlazorCodeMirror6.bundle.scp.css`
