# [已归档] Android 启动闪退修复记录

> **状态**: 已归档。本文档记录的是旧 Avalonia.Android 项目（Cide.Client.Android/）的启动闪退问题。
> 该项目已于 2026-05-04 清理移除，移动端现由 Cide.Client.Maui（MAUI Blazor Hybrid）接管。
> 本文档保留作为历史事故排查参考。

## 问题描述

C IDE Android 应用在物理设备上启动后立即闪退：
- 启动画面（Splash Screen）显示后应用直接崩溃
- 无任何 .NET 侧日志输出到 logcat
- 增量构建异常快速（~0.5s），怀疑构建缓存导致修改未生效

## 诊断过程

### 阶段 1：排除应用层问题

为定位崩溃点，进行了以下隔离测试：

1. **主题修复**：将 `styles.xml` 的 parent 从 `Theme.Material.Light.NoActionBar` 改为 `Theme.Material.Light.NoActionBar`（后续发现仍需改为 AppCompat）
2. **NullReference 防护**：在 `MainViewModel` 构造函数中初始化 `CurrentKnowledgeCard`
3. **TextMate 硬编码**：在 `CodeEditor` 的 TextMate 初始化外加 `try/catch`
4. **全局异常处理**：添加 `TaskScheduler.UnobservedTaskException` 和 `AndroidEnvironment.UnobservedExceptionRaiser`
5. **最小视图测试**：在 `App.axaml.cs` 中将 `MainView` 替换为纯 `TextBlock`，排除 XAML/用户控件问题
6. **禁用 CompiledBindings**：将 `AvaloniaUseCompiledBindingsByDefault` 改为 `false`

**结果**：以上修改均未解决闪退，且无任何 logcat 输出，说明崩溃发生在 .NET 托管代码执行之前。

### 阶段 2：对比正常项目

与同一设备上可正常运行的 **Avalonia 2048 项目** 进行横向对比，发现 7 个关键差异：

| 差异项 | 2048 项目 (✅ 正常) | C IDE 项目 (❌ 崩溃) |
|--------|---------------------|----------------------|
| Avalonia 版本 | `12.0.1` | `11.3.0` |
| CompiledBindings | 未设置（默认） | Android csproj 中 `=true` |
| AOT 编译 | 显式禁用 | 未设置（可能默认启用） |
| MainActivity | `AvaloniaMainActivity` (非泛型) | `AvaloniaMainActivity<App>` |
| SplashScreen 包 | 有 `Xamarin.AndroidX.Core.SplashScreen` | 无 |
| OS Platform 版本 | `23` | `21` |
| **Fast Deployment** | **未明确设置** | **默认启用（Debug）** |

### 阶段 3：抓取原生崩溃日志

执行完整清理重建后，通过 `adb logcat` 捕获到关键崩溃信息：

```
Abort message: 'No assemblies found in 
'/data/user/0/com.cide.app/files/.__override__/arm64-v8a' 
or '<unavailable>'. Assuming this is part of Fast Deployment. Exiting...'
```

**诊断结论**：.NET Android 的 **Fast Deployment** 机制在 Debug 配置下默认启用，将程序集部署到设备的 `/sdcard/Android/data/...` 或应用私有目录中。由于权限问题或部署失败，Mono runtime 在启动时找不到这些外部程序集，直接调用 `abort()` 终止进程。

### 阶段 4：修复后暴露的新问题

禁用 Fast Deployment（`EmbedAssembliesIntoApk=true`）后，APK 大小从 12.85MB 增至 70.69MB，确认程序集已嵌入。但应用再次崩溃，新错误为：

```
java.lang.IllegalStateException: 
You need to use a Theme.AppCompat theme (or descendant) with this activity.
```

**诊断结论**：Avalonia Android 的 `MainActivity` 底层依赖 AndroidX AppCompat，必须使用 `Theme.AppCompat` 派生主题。原项目的 `Theme.Material.Light.NoActionBar` 不兼容。

---

## 根因分析

本次启动闪退由 **三个叠加问题** 导致：

1. **Fast Deployment 失败**（首要原因）
   - .NET Android Debug 配置默认启用 Fast Deployment
   - `dotnet publish` 配合 `--self-contained false` 时，程序集未正确推送到设备
   - Mono runtime 启动时找不到程序集，直接 abort

2. **主题不兼容**（次要原因）
   - `AvaloniaMainActivity` 继承自 AppCompatActivity
   - 必须使用 `Theme.AppCompat.*` 主题
   - `Theme.Material.Light.NoActionBar` 会导致 `IllegalStateException`

3. **CompiledBindings 配置冲突**（潜在风险）
   - `Cide.Client.Android.csproj` 中显式设置 `AvaloniaUseCompiledBindingsByDefault=true`
   - 与 `Cide.Client.csproj` 中的 `false` 冲突
   - Avalonia 11.3 的 CompiledBindings 在 Android 上存在 AOT codegen 问题

---

## 修复步骤

### 1. 禁用 Fast Deployment（关键修复）

在 `Cide.Client.Android.csproj` 中添加：

```xml
<EmbedAssembliesIntoApk>true</EmbedAssembliesIntoApk>
```

作用：将所有 .NET 程序集打包进 APK，不再依赖外部 Fast Deployment 目录。

### 2. 使用 AppCompat 主题（关键修复）

修改 `Cide.Client.Android/Resources/values/styles.xml`：

```xml
<?xml version="1.0" encoding="utf-8" ?>
<resources>
  <style name="MyTheme">
  </style>
  <style name="MyTheme.NoActionBar" parent="@style/Theme.AppCompat.DayNight.NoActionBar">
    <item name="android:windowActionBar">false</item>
    <item name="android:windowNoTitle">true</item>
  </style>
</resources>
```

### 3. 禁用 AOT 编译（防御性修复）

在 `Cide.Client.Android.csproj` 中添加：

```xml
<AndroidEnableProfiledAot>false</AndroidEnableProfiledAot>
<RunAOTCompilation>false</RunAOTCompilation>
```

### 4. 统一 CompiledBindings 设置

确保 `Cide.Client.Android.csproj` 和 `Cide.Client.csproj` 一致：

```xml
<AvaloniaUseCompiledBindingsByDefault>false</AvaloniaUseCompiledBindingsByDefault>
```

### 5. 清理重建

每次修改配置后必须执行深度清理：

```powershell
dotnet clean Cide.Client.Android/Cide.Client.Android.csproj -c Debug
Remove-Item -Recurse -Force Cide.Client.Android\obj\Debug
Remove-Item -Recurse -Force Cide.Client\obj\Debug
```

---

## 修改文件清单

| 文件 | 修改内容 |
|------|----------|
| `Cide.Client.Android/Cide.Client.Android.csproj` | 添加 `EmbedAssembliesIntoApk=true`、`AndroidEnableProfiledAot=false`、`RunAOTCompilation=false`、`AvaloniaUseCompiledBindingsByDefault=false` |
| `Cide.Client.Android/Resources/values/styles.xml` | 主题 parent 改为 `Theme.AppCompat.DayNight.NoActionBar` |
| `Cide.Client/App.axaml.cs` | 恢复使用 `MainView` + `MainViewModel` |

---

## 验证结果

修复后应用在 Android 物理设备上：
- ✅ 正常启动，无闪退
- ✅ `MainView` 完整加载（编辑器、调用栈、变量、内存等面板）
- ✅ Avalonia 渲染管线正常工作（Skia/OpenGL）
- ✅ Native backend (`libcide_native.so`) 正确打包进 APK

---

## 后续建议

1. **响应式布局验证**：当前界面加载的是桌面布局，需验证 `ResponsiveLayoutViewModel` 在手机竖屏下是否正确切换为手机布局
2. **代码编辑测试**：验证 `CodeEditor`（AvaloniaEdit）在 Android 上的输入、滚动、语法高亮功能
3. **Native 功能测试**：编译并运行 C 代码，验证 native backend 的编译/执行/调试流程
4. **考虑升级 Avalonia**：2048 项目使用 Avalonia 12.0.1，对 Android 有更完善的官方支持

---

*修复时间: 2026-04-29*  
*状态: ✅ 已修复（Avalonia Android 版本）*
