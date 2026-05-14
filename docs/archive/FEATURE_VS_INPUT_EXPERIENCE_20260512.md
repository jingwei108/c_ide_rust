# VS 风格输入体验增强（自动格式化 + 智能缩进 + 移动端符号栏）

**日期**：2026-05-12  
**关联需求**：类似 Visual Studio 的代码输入体验，重点解决移动端输入特殊符号繁琐的问题。

---

## 1. 背景与动机

Cide 编辑器基于 CodeMirror 6 + GaelJ.BlazorCodeMirror6，在 PC 端已有括号匹配、自动闭合、代码折叠等基础能力。但在移动端（Android MAUI WebView）存在明显痛点：

1. **缺少自动格式化**：用户输入 `int a=1;` 后不会自动变成 `int a = 1;`。
2. **缩进行为不够智能**：未显式配置缩进单位。
3. **移动端特殊符号输入极繁琐**：`{ } ( ) [ ] ; " ' # -> & *` 等高频符号在软键盘上需要多次切换面板。
4. **没有光标移动和 Undo/Redo 的快捷方式**：移动端软键盘通常没有方向键和 Ctrl+Z/Ctrl+Y。

---

## 2. 实现内容

### 2.1 回车自动格式化上一行（VS 风格）

**机制**：在 `codemirror-interop.js` 中监听 `Enter` 键，利用 `setTimeout(..., 0)` 让 CodeMirror 先完成换行与缩进，随后对上一行调用 `formatLine` 进行单行格式化，并通过 `view.dispatch` 替换原文。

**格式化规则**（`formatLine`）：
- **保留前导缩进**：只处理代码内容，不动缩进层级。
- **保护字符串与注释**：逐字符解析，遇到 `"..."` / `'...'` / `//` / `/* */` 时跳过内部处理。
- **逗号后加空格**：`foo(a,b)` → `foo(a, b)`
- **分号前去除空格**：`a = 1 ;` → `a = 1;`
- **运算符两边加空格**：
  - 双字符：`==`、`!=`、`<=`、`>=`、`&&`、`||`、`<<`、`>>`、`+=`、`-=`、`*=`、`/=`、`%=`、`&=`、`|=`、`^=`、`->`
  - 单字符：`=`、`<`、`>`、`+`、`-`、`*`、`/`、`%`、`&`、`|`、`^`、`?`、`:`、`!`、`~`
  - 注意识别一元运算符场景（如 `*p`、`&a`、`!flag`），避免在一元场景前加空格。
- **标识符与 `(` 之间加空格**：`if(` → `if (`、`foo(` → `foo (`
- **去除括号内多余空格**：`( a )` → `(a)`、`[ i ]` → `[i]`

**示例**：
```c
// 用户输入后按 Enter
int a=1;
// 自动变为
int a = 1;

// 用户输入后按 Enter
if(a==b){
// 自动变为
if (a == b) {
```

### 2.2 智能缩进

**改动**：
- `CodeMirrorEditor.razor` 中 `CodeMirror6Wrapper` 显式设置 `IndentationUnit="4"`。
- `CodeMirrorSetup` 保持 `IndentOnInput = true` + `IndentWithTab = false`。

**效果**：
- 回车后新行自动按 4 空格缩进。
- 输入 `{` 后回车，下一行自动增加一级缩进。
- 输入 `}` 后，CodeMirror Cpp 语言包会自动减少缩进。

> **注意**：`IndentationUnit` 是 `CodeMirror6Wrapper` 组件的参数（类型 `int?`），不是 `CodeMirrorSetup` 的属性。

### 2.3 移动端符号快捷栏（智能显隐）

#### 2.3.1 显隐策略

**核心需求**：符号栏只在虚拟键盘弹出时出现，平时隐藏不占空间。

**实现方式**：
- **默认状态**：`.symbol-bar` 的 `max-height: 0`、`opacity: 0`，完全不可见。
- **显示触发**：在 `ensureObserver` 中监听 `.cm-content`（CodeMirror 的 contenteditable 区域）的 `focus` 事件，添加 `.visible` class（`max-height: 52px`、`opacity: 1`）。
- **隐藏触发**：监听 `.cm-content` 的 `blur` 事件，延迟 150ms 检查 `:focus`；若编辑器确实失焦（如用户点击底部面板），移除 `.visible` class。
- **防焦点偷走**：所有符号按钮设置 `tabindex="-1"`，点击时不会抢走编辑器焦点，因此符号栏不会意外收起。

#### 2.3.2 符号按钮布局

横向可滚动容器，分组如下：

| 类别 | 按钮 | 行为 |
|------|------|------|
| **自动配对** | `{ }` `( )` `[ ]` `" "` `' '` | 调用 `insertPair(open, close)`，插入配对符号并将光标置于中间 |
| **语句/预处理** | `;` `#` | 直接插入单个符号 |
| **指针/地址** | `->` `&` `*` | 直接插入 |
| **赋值/比较** | `=` `==` `!=` `<` `>` | 直接插入 |
| **算术** | `+` `-` `/` `%` | 直接插入 |
| **逻辑/位运算** | `&&` `||` `!` `|` `^` `~` | 直接插入 |
| **其他** | `,` `.` | 直接插入 |
| **编辑辅助** | `←` `→` `Tab` `↩ Undo` `↪ Redo` | 光标移动、插入 4 空格、撤销、重做 |

#### 2.3.3 CSS 样式要点

- `.symbol-bar`：暗色/亮色主题适配，过渡动画 `transition: max-height 0.2s ease, opacity 0.2s ease`。
- `.symbol-btn`：最小宽度 32px，高度 32px，圆角 6px，`user-select: none`，`:active` 缩放 + 变蓝反馈。
- `.symbol-pair`：加粗，略宽（40px），突出显示。
- `.symbol-action`：深色背景区分，表示功能键而非输入符号。
- `.symbol-divider`：竖线分隔不同功能组。

### 2.4 配套 JS / C# API 扩展

**`codemirror-interop.js` 新增公开 API**：

| API | 参数 | 说明 |
|-----|------|------|
| `insertPair(id, open, close)` | editorId, string, string | 插入配对符号，光标居中 |
| `moveCursor(id, offset)` | editorId, int | 相对当前光标移动指定偏移 |
| `undo(id)` | editorId | 调用 GaelJ.BlazorCodeMirror6 内部 `dispatchCommand(id, 'Undo')` |
| `redo(id)` | editorId | 调用 GaelJ.BlazorCodeMirror6 内部 `dispatchCommand(id, 'Redo')` |

**`CodeMirrorEditor.razor` 新增方法**：
- `InsertPair(string open, string close)`
- `MoveCursor(int offset)`
- `Undo()`
- `Redo()`

**`Home.razor` 新增方法**：
- `InsertPair(string open, string close)` — 透传给 `_editor`
- `MoveCursor(int offset)` — 透传给 `_editor`
- `Undo()` / `Redo()` — 透传给 `_editor`
- `InsertQuotePair()` — 辅助方法，解决 Razor 单引号属性值解析问题

---

## 3. 文件修改清单

| 文件 | 变更类型 | 说明 |
|------|----------|------|
| `Cide.Client.Maui/Components/Editor/CodeMirrorEditor.razor` | 修改 | 添加 `IndentationUnit="4"`；新增 `InsertPair`/`MoveCursor`/`Undo`/`Redo` |
| `Cide.Client.Maui/Components/Pages/Home.razor` | 修改 | 删除实验性格式化按钮；新增符号快捷栏 HTML 及对应事件方法 |
| `Cide.Client.Maui/wwwroot/app.css` | 修改 | 新增 `.symbol-bar`、`.symbol-scroll`、`.symbol-btn`、`.symbol-pair`、`.symbol-action`、`.symbol-divider` 样式 |
| `Cide.Client.Maui/wwwroot/js/codemirror-interop.js` | 修改 | 新增 `formatLine`、Enter 自动格式化、符号栏显隐控制、`insertPair`/`moveCursor`/`undo`/`redo` API |

---

## 4. 技术细节与踩坑记录

### 4.1 `IndentationUnit` 属性位置
`CodeMirrorSetup` 类（`GaelJ.BlazorCodeMirror6.Models`）**没有** `IndentationUnit` 属性；该属性位于 `CodeMirror6Wrapper` 组件上。早期尝试在 `CodeMirrorSetup` 中设置导致编译错误 `CS0117`。最终通过直接在组件标签上写 `IndentationUnit="4"` 解决。

### 4.2 Razor 属性值中的引号嵌套
符号栏按钮的 `@onclick` 中需要传递 `"` 或 `'` 字符串给 `InsertPair`，直接写双引号包裹会导致 Razor HTML 解析器将内部 `"` 视为属性值结束。

**解决方案**：
- 绝大多数按钮：使用单引号包裹整个属性值，如 `@onclick='() => InsertTemplate(";")'`。
- 单引号按钮：由于属性值内部需要 `"'"`，且 HTML 单引号包裹时内部 `'` 仍会被 Razor C# 解析器识别为 `char` 字面量，额外添加了辅助方法 `InsertQuotePair()`，Razor 中直接写 `@onclick="InsertQuotePair"`。

### 4.3 `undo` / `redo` 的正确调用路径
最初尝试动态 `import` GaelJ.BlazorCodeMirror6 的 `index.js` 并解构 `undo` / `redo`，但该 bundle 并未导出这两个函数（它们来自 `@codemirror/commands`，被内部使用但未重新导出）。

通过阅读 bundle 源码发现 GaelJ.BlazorCodeMirror6 提供了 `dispatchCommand(id, functionName)` 接口，其中支持 `'Undo'` 和 `'Redo'` case。最终改为：
```js
if (mod?.dispatchCommand) {
    mod.dispatchCommand(id, 'Undo');  // 或 'Redo'
}
```

### 4.4 符号栏按钮点击导致编辑器失焦
若符号按钮可聚焦（默认 `tabindex="0"`），点击后焦点会离开 `.cm-content`，触发 `blur` 导致符号栏瞬间收起。

**解决方案**：所有符号按钮显式设置 `tabindex="-1"`，使其不可聚焦，点击时不会偷走编辑器焦点。配合 `blur` 的 150ms 延迟检查，确保正常点击符号栏时栏不会意外隐藏。

---

## 5. 已知限制与后续优化方向

1. **自动格式化仅作用于单行**：当前 `formatLine` 只处理单行文本，不处理跨行场景。若后续需要整文档格式化，需引入更完整的 C 代码格式化器。
2. **无物理键盘的强制补全触发**：CodeMirror 6 的 `autocompletion` 默认在输入时自动弹出，但尚无强制触发按钮。后续若需 IntelliSense，可在符号栏增加补全按钮。
3. **符号栏在桌面端也会显示**：当前实现仅基于编辑器 `focus`/`blur`，不区分设备类型。桌面端若点击编辑器，符号栏同样会滑出。如需桌面端始终隐藏，可在 JS 中增加 `window.matchMedia('(hover: none)')` 或屏幕宽度判断。
4. **Undo/Redo 依赖封装层**：`dispatchCommand` 走的是 GaelJ.BlazorCodeMirror6 封装层，若未来升级 NuGet 包导致接口变化，需同步调整。

---

## 6. 编译验证

```bash
dotnet build Cide.Client.Maui/Cide.Client.Maui.csproj \
  --framework net10.0-windows10.0.19041.0 \
  --configuration Debug
```

**结果**：0 错误，0 警告（仅既有 MVVMTK0045 AOT 兼容警告，与本功能无关）。
