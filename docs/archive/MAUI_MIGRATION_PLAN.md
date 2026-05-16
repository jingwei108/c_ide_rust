# C IDE .NET MAUI 迁移方案

> **状态**: ✅ POC 验证通过，核心功能已可用  
> **目标**: 将移动端前端从 Avalonia 迁移至 .NET MAUI Blazor Hybrid，解决 Android 虚拟键盘、语法高亮及长期移动端维护性问题。桌面端保留 Avalonia，形成双前端架构。

---

## 一、目标架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           共享后端 (C++ Native)                          │
│                    libcide_native.so / .dll / .dylib                    │
│                         纯净 C API (cide_capi.h)                         │
└─────────────────────────────────────────────────────────────────────────┘
                                    ▲
                                    │ P/Invoke / JNI / FFI
        ┌───────────────────────────┼───────────────────────────┐
        │                           │                           │
   ┌────┴────┐                 ┌────┴────┐                 ┌────┴────┐
   │ Avalonia │                 │  MAUI   │                 │ 未来 iOS │
   │ Desktop  │                 │ Mobile  │                 │ (可选)  │
   └────┬────┘                 └────┬────┘                 └────┬────┘
        │                           │                           │
   ┌────┴───────────────────────────┴───────────────────────────┴────┐
   │                    共享 C# 中间层 (可复制)                        │
   │  NativeMethods.cs / CompilerService.cs / Models / ViewModels    │
   └─────────────────────────────────────────────────────────────────┘
```

### 关键设计决策

| 决策 | 方案 | 理由 |
|:---|:---|:---|
| **代码编辑器** | Blazor Hybrid + Monaco Editor | VS Code 核心，键盘/IME/高亮全原生支持 |
| **内存/指针可视化** | SkiaSharp (`SKCanvasView`) | 跨平台 2D 渲染，与现有绘制逻辑兼容 |
| **UI 框架** | MAUI Blazor Hybrid | 复用 C# 代码，原生移动端体验 |
| **桌面端** | 保留 Avalonia | 不破坏现有桌面体验，后端共享 |
| **数据绑定** | MVVM (CommunityToolkit.Mvvm) | 与现有 ViewModel 逻辑兼容 |

---

## 二、实际项目结构

```
Cide.Client.Maui/                          # MAUI 项目（已实际创建）
├── App.xaml / App.xaml.cs                 # MAUI 应用入口
├── MainPage.xaml / MainPage.xaml.cs       # 主页面容器（承载 BlazorWebView）
├── MauiProgram.cs                         # MAUI 程序注册
│
├── Components/                             # Blazor 组件
│   ├── Main.razor                         # Blazor 根组件（Router）
│   ├── Layout/
│   │   └── MainLayout.razor               # Blazor 布局
│   ├── Pages/
│   │   └── Home.razor                     # 主页面（编辑器+工具栏+面板）
│   └── Editor/
│       ├── MonacoEditor.razor             # Monaco 编辑器封装（C#↔JS 互操作）
│       └── SimpleEditor.razor             # Monaco 失败时降级方案（textarea）
│
├── Core/                                   # 从原项目复用
│   ├── NativeMethods.cs                   # P/Invoke 声明
│   ├── CompilerService.cs                 # 编译器服务封装
│   └── Models/                            # 数据模型（Diagnostic, VariableSnapshot 等）
│
├── ViewModels/                             # 适配后的视图模型
│   └── MainViewModel.cs                   # 从 Avalonia 适配，移除 UI 依赖
│
├── wwwroot/                                # Blazor 静态资源
│   ├── index.html                         # Blazor 宿主页面
│   ├── css/
│   │   ├── app.css                        # 全局样式
│   │   └── monaco.css                     # Monaco 编辑器样式
│   └── js/
│       ├── monaco-interop.js              # C# ↔ JS 互操作桥
│       └── canvas-interop.js              # Canvas 互操作
│   └── monaco/                            # Monaco Editor 本地资源 (~11MB 精简版)
│       └── vs/
│           ├── loader.js
│           ├── editor/
│           └── basic-languages/c/
│
└── Cide.Client.Maui.csproj
```

---

## 三、实际实现记录

### 3.1 创建与基础配置

- 基于 `maui-blazor` 模板创建 `Cide.Client.Maui`
- TargetFramework: `net10.0-android`
- 包名: `com.cide.mobile`
- 最小 API 级别: 24

### 3.2 核心后端复用

| 文件 | 来源 | 修改内容 |
|:---|:---|:---|
| `NativeMethods.cs` | `Cide.Client/Core/` | 仅更新命名空间，DllImport 库名保持 `cide_native` |
| `CompilerService.cs` | `Cide.Client/Core/` | 仅更新命名空间，纯 C# 无 UI 依赖 |
| `Diagnostic.cs` 等模型 | `Cide.Client/ViewModels/` | 仅更新命名空间 |
| `MainViewModel.cs` | `Cide.Client/ViewModels/` | 移除 Avalonia 依赖，`[RelayCommand]` 方法改为 `public`，使用 `CommunityToolkit.Mvvm` |
| `KnowledgeCardLoader` | 嵌入资源加载器 | 从 Avalonia 嵌入资源改为 `Assembly.GetManifestResourceStream` 加载 JSON |

### 3.3 Monaco Editor 集成

- 本地嵌入 Monaco 精简版（仅 C 语言支持，~11MB）
- `index.html` 中通过 `script` 标签加载 `monaco/vs/loader.js`
- C# ↔ JS 互操作：
  - `monacoInterop.createEditor` — 初始化编辑器
  - `OnCodeChanged` / `OnBreakpointToggled` — JSInvokable 回调
  - `SetTheme` / `SetBreakpoints` / `SetErrorLines` — C# 调用 JS

**关键问题**：Android WebView `file://` 协议不支持 Web Worker，Monaco 的 worker 会失败。
**解决方案**：添加 dummy worker 降级脚本，Monaco 在 worker 不可用时回退到主线程执行。

### 3.4 构建脚本适配

`test-mobile.ps1` 已更新：
- 构建目标从 `Cide.Client.Android` 改为 `Cide.Client.Maui`
- 包名从 `com.cide.app` 改为 `com.cide.mobile`
- APK 查找优先 `com.cide.mobile-Signed.apk`
- 启动命令使用 `monkey -p com.cide.mobile`

---

## 四、关键问题与修复

### 问题 1：Fast Deployment 导致启动闪退

**现象**：应用启动后立即崩溃，logcat 显示：
```
No assemblies found in '/data/user/0/com.cide.mobile/files/.__override__/arm64-v8a'
```

**原因**：.NET Android Debug 配置默认启用 Fast Deployment，将程序集部署到设备外部目录。部署失败时 Mono runtime 直接 abort。

**修复**：在 `.csproj` 中禁用 Fast Deployment：
```xml
<EmbedAssembliesIntoApk>true</EmbedAssembliesIntoApk>
```

**影响**：APK 体积从 ~12MB 增至 ~300MB（Debug 配置，含全部程序集），Release 配置可大幅缩小。

### 问题 2：Native .so 压缩导致 monodroid 崩溃

**现象**：禁用 Fast Deployment 后，应用再次崩溃：
```
java.lang.UnsatisfiedLinkError: ... libcide_native.so: error loading library
ALL entries in APK named lib/arm64-v8a/ MUST be STORED
```

**原因**：APK 中的 `.so` 文件被压缩，Android 系统要求 native 库必须以 STORED（不压缩）方式打包。

**修复**：在 `.csproj` 中配置不压缩 `.so`：
```xml
<AndroidStoreUncompressedFileExtensions>.so</AndroidStoreUncompressedFileExtensions>
```

### 问题 3：Home.razor 白屏

**现象**：应用启动后 Blazor WebView 区域白屏，`Blazor.start()` 已调用但根组件渲染失败。

**诊断**：
1. 最小化测试（仅 `<div>Hello</div>`）证明基础渲染正常
2. 使用 `ErrorBoundary` 包裹内容逐个引入组件测试
3. 最终发现是 Fast Deployment 和 .so 压缩两个问题的叠加效应——组件渲染需要完整程序集和 native 库同时可用

**修复**：同时解决上述两个问题后，白屏自动消失。完整界面（工具栏、Monaco 编辑器、底部面板）正常渲染。

### 问题 4：Monaco Web Worker 在 Android WebView 中不可用

**现象**：Monaco 编辑器初始化时尝试加载 Web Worker，在 `file://` 协议下失败。

**修复**：在 `index.html` 中注入 dummy worker 脚本，覆盖 `window.Worker` 构造函数，返回一个模拟 Worker 对象，使 Monaco 回退到主线程模式。

**状态**：⚠️ 已添加降级方案，实机 Monaco 编辑功能正常，但 worker 模式性能待验证。

---

## 五、验证结果

### POC 验收（2026-04-30）

在 Android 物理设备上验证通过：

| 检查项 | 状态 | 备注 |
|:---|:---|:---|
| 应用正常启动 | ✅ | 无闪退，无崩溃 |
| CodeMirror 6 加载 | ✅ | C 代码语法高亮正常，代码可编辑（已切换为 CodeMirror 6） |
| 工具栏渲染 | ✅ | 运行/单步/停止/主题切换按钮正常显示 |
| 底部面板 | ✅ | 输出/诊断/算法三标签页正常切换 |
| Native 库加载 | ✅ | `libcide_native.so` 正确打包并加载 |
| 编译执行（运行按钮） | ✅ | 验证通过 |
| 单步调试 | ✅ | 验证通过 |
| 主题切换 | ✅ | 明暗主题可切换 |
| 悬浮球 FAB | ✅ | 拖拽、贴边吸附、扇形展开、调试 Modal 打开均正常 |
| 内存/指针可视化 | ⏳ | SkiaSharp 画布待集成 |

### UI 重制验收（2026-05-08）

参考图一完成移动端 UI 风格统一：

| 检查项 | 状态 | 备注 |
|:---|:---|:---|
| 悬浮球扇形光晕 | ✅ | 展开时扩散蓝色径向渐变背景 + 弧线装饰 |
| 悬浮球 3D 质感 | ✅ | 多层阴影 + 顶部高光 + 底部内阴影 + 按压反馈 |
| 扇形菜单 stagger 动画 | ✅ | 8 个按钮逐个延迟 25ms 弹性弹出 |
| 工具栏样式统一 | ✅ | 蓝播放/灰下一步/滑块/状态框/主题按钮 |
| 底部标签指示条 | ✅ | 选中项底部蓝色 `#0A84FF` 指示条 |
| 模板快捷栏 | ✅ | 编辑器下方横向滚动标签栏，点击插入代码模板 |
| 模态面板质感 | ✅ | 8px 毛玻璃遮罩 + 20px 圆角 + 拖动手柄 + 下滑关闭手势 |
| 诊断/算法卡片 | ✅ | 8px 圆角 + 微阴影 + 半透明背景层次 |
| 底部面板压缩 | ✅ | 移动端 140px，释放编辑器垂直空间 |

---

## 六、待办事项

### 高优先级
- [x] 验证「运行」按钮：C 代码编译执行，输出到控制台面板
- [x] 验证「单步」按钮：断点调试、调用栈、变量面板
- [x] 验证主题切换：CodeMirror 6 + MAUI 同步切换明暗主题
- [x] CodeMirror 6 编辑体验：代码编辑、光标移动、IME 输入
- [x] 悬浮球扇形展开与质感：背景光晕、弧线、stagger 动画、3D 球体质感
- [x] 工具栏重制：参考图一统一按钮样式、滑块、状态框
- [ ] 集成 SkiaSharp 画布：内存可视化、指针图、链表图

### 中优先级
- [x] 响应式布局：手机竖屏/横屏/平板适配
- [x] 知识卡片系统：从嵌入 JSON 加载并显示
- [ ] Release 配置优化：trim、AOT、包体积压缩

### 低优先级
- [ ] iOS 平台扩展：MAUI 天然支持，未来可扩展
- [ ] 16KB page size 适配：Android 16 将要求 16KB 页面大小的 `.so`
- [ ] Monaco 按需加载：仅加载 C 语言包，进一步缩减体积

---

## 七、桌面端保留策略

迁移期间，桌面端 **完全保留现有 Avalonia 项目**，形成双前端：

```
Cide.slnx
├── Cide.Client/              # Avalonia 桌面端（保留，不动）
├── Cide.Client.Desktop/      # Avalonia 桌面启动器（保留，不动）
├── Cide.Client.Android/      # Avalonia Android（已冻结，不再维护）
└── Cide.Client.Maui/         # MAUI 移动端（当前主要开发目标）
    └── Platforms/Android/    # 替代原 Avalonia Android
```

### 构建脚本

`test-mobile.ps1` 当前构建目标：

```powershell
dotnet publish Cide.Client.Maui/Cide.Client.Maui.csproj `
    -f net10.0-android `
    -c Debug `
    -p:AndroidPackageFormat=apk `
    -o dist/android `
    --self-contained false
```

---

## 八、风险与缓解

| 风险 | 影响 | 状态 | 缓解措施 |
|:---|:---|:---|:---|
| Monaco Editor 在低端 Android 设备卡顿 | 高 | 🟡 观察中 | 已添加 `SimpleEditor` 降级方案（textarea）|
| MAUI Blazor Hybrid 启动慢 | 中 | 🟢 可接受 | Debug 配置 302MB（含全部 assemblies），Release 可大幅缩小 |
| SkiaSharp 在 MAUI 中的性能 | 中 | ⏳ 未验证 | 计划使用 `SKGLView`（OpenGL 加速）替代 `SKCanvasView` |
| 现有 `MainViewModel` 与 Blazor 绑定不兼容 | 低 | ✅ 已解决 | `[RelayCommand]` 方法改为 `public`，事件通知通过 `PropertyChanged` |
| iOS 未来扩展成本 | 中 | ⏳ 未开始 | MAUI 天然支持 iOS，扩展成本极低 |
| 包体积过大 | 中 | 🟡 观察中 | Debug 302MB，Release + AOT + Trim 可降至 ~50MB 以下 |
| Android 16 16KB page size | 低 | 🟡 已知 | native 库需重新编译，添加 `-Wl,-z,max-page-size=16384` |

---

## 九、关键决策检查点

| 检查点 | 时间 | 决策内容 | 结果 |
|:---|:---|:---|:---|
| **POC 验证** | 2026-04-30 | Monaco Editor 在 MAUI Android 实机上是否正常加载、键盘是否正常 | ✅ **通过** — 编辑器正常加载，界面完整渲染 |
| **编辑器验收** | — | Monaco 的断点、主题、语法高亮是否满足需求 | ⏳ 待验证 — 语法高亮正常，断点/主题待测试 |
| **性能验收** | — | 内存/指针视图的 SkiaSharp 渲染在实机上是否流畅（目标 60fps）| ⏳ 未开始 — 画布组件待集成 |
| **发布决策** | — | 是否冻结 Avalonia Android，全面切换至 MAUI | ⏳ 待定 — 需完成核心功能验证后决策 |

---

## 十、备选编辑器方案

| 编辑器 | 技术 | 优点 | 缺点 | 当前状态 |
|:---|:---|:---|:---|:---|
| **Monaco Editor** | Web/JS | 功能最全，VS Code 核心 | 体积大，Web Worker 需降级 | ✅ **已采用** |
| **CodeMirror 6** | Web/JS | 更轻量，移动端优化好 | 功能略少于 Monaco | ⏳ 备选 |
| **Ace Editor** | Web/JS | 老牌，稳定 | 移动端体验一般 | ⏳ 备选 |
| **MAUI Editor** | 原生 | 零问题，性能最好 | 无语法高亮，需自研 | ❌ 不适用 |
| **sora-editor** | Android 原生 | 专为 Android 设计，性能极佳 | 需平台特定嵌入 | ❌ 不适用 |

> **建议**：Monaco 当前工作正常，继续作为主编辑器。若低端机出现性能问题，可热切换到 `SimpleEditor` 降级方案。

---

*方案制定时间: 2026-04-29*  
*POC 验证时间: 2026-04-30*  
*状态: ✅ POC 通过，核心功能开发中*
