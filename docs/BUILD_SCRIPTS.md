# C IDE 构建与测试脚本指南

本文档说明项目中所有 PowerShell 脚本的使用方法、参数和常见问题排查。

---

## 脚本清单

| 脚本 | 功能 | 适用场景 |
|:---|:---|:---|
| [`build.ps1`](../build.ps1) | 构建 Native 后端 + Avalonia 前端（桌面） | 日常开发编译、打包、发布 |
| [`test-mobile.ps1`](../test-mobile.ps1) | 移动端完整测试流水线：构建 → 安装 → 启动 → 日志 | MAUI Android 真机/模拟器测试 |

---

## `build.ps1` — 主构建脚本

### 功能概述

统一构建 C++ Native 后端和 C# 前端，支持桌面端（Windows）和 Android 端。

**构建流程**（桌面端）：
1. Native 后端：CMake + Ninja/MSVC/MinGW 编译 `cide_native.dll`
2. 前端：`.NET publish` 打包 `Cide.Client.Desktop`
3. 将 DLL 复制到 `dist/desktop/`

**构建流程**（Android 端，由 `test-mobile.ps1` 处理）：
1. Native 后端：NDK 交叉编译 `arm64-v8a` + `armeabi-v7a` 的 `libcide_native.so`
2. 前端：`.NET publish` 打包 MAUI APK 到 `dist/android/`

### 参数

| 参数 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `-Configuration` | `Debug` / `Release` | `Debug` | 构建配置 |
| `-Target` | `Desktop` / `Android` / `All` | `Desktop` | 构建目标平台 |
| `-Clean` | Switch | 关闭 | 清理所有构建产物（`native/build*`、`bin/obj`、`dist`） |
| `-Run` | Switch | 关闭 | 构建完成后运行桌面端应用（仅 `-Target Desktop` 有效） |
| `-Compiler` | `Default` / `Clang` / `ClangCL` / `MSVC` / `MinGW` | `Default` | 桌面端 C++ 编译器选择 |

### 使用示例

```powershell
# 桌面端 Debug 构建（默认）
.\build.ps1

# 桌面端 Release 构建，构建完成后直接运行
.\build.ps1 -Configuration Release -Run

# 清理所有构建产物
.\build.ps1 -Clean

# 使用 Clang 编译桌面端 Native 后端
.\build.ps1 -Compiler Clang

# Android 端完整构建（NDK .so + APK）
.\build.ps1 -Target Android

# 同时构建桌面端和 Android 端
.\build.ps1 -Target All
```

### 桌面端编译器选择

| 编译器 | 要求 | 说明 |
|:---|:---|:---|
| `Default` | Ninja 或 Visual Studio | 自动探测：优先 Ninja，其次 VS，最后 MinGW |
| `Clang` | Ninja + `C:/Clang/clang+llvm-22.1.4-x86_64-pc-windows-msvc` | 使用 Clang/Clang++ |
| `ClangCL` | Ninja + 同上 | 使用 clang-cl（MSVC 兼容模式） |
| `MSVC` | Visual Studio | 使用 Visual C++ 编译器 |
| `MinGW` | MinGW | 使用 MinGW Makefiles 生成器 |

### 环境变量

| 变量 | 说明 |
|:---|:---|
| `ANDROID_NDK_HOME` / `ANDROID_NDK_ROOT` | Android NDK 路径，用于 `-Target Android` |

---

## `test-mobile.ps1` — 移动端测试流水线

### 功能概述

专注于 **MAUI Android** 真机/模拟器的**快速测试循环**，提供从 Native `.so` 编译 → APK 打包 → 设备安装 → 应用启动 → Logcat 日志抓取的完整自动化流水线。

与 `build.ps1 -Target Android` 的区别：
- `build.ps1` 构建 **Avalonia Desktop**（`Cide.Client.Desktop`）
- `test-mobile.ps1` 构建 **MAUI Android**（`Cide.Client.Maui`），并自动完成安装、启动、日志抓取

### 参数

| 参数 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `-Configuration` | `Debug` / `Release` | `Debug` | 构建配置 |
| `-SkipNativeBuild` | Switch | 关闭 | 跳过 NDK 原生 `.so` 编译，仅重新打包 APK |
| `-Install` | Switch | 关闭 | APK 构建完成后自动安装到设备 |
| `-Run` | Switch | 关闭 | 安装后自动启动应用 |
| `-Logcat` | Switch | 关闭 | 启动后实时抓取应用日志（`Ctrl+C` 停止） |

### 使用示例

```powershell
# 仅构建 APK（含 Native .so）
.\test-mobile.ps1

# 快速重新打包（前端代码改动后，跳过 .so 编译）
.\test-mobile.ps1 -SkipNativeBuild -Install -Run

# 构建 + 安装 + 启动 + 实时日志（完整测试流水线）
.\test-mobile.ps1 -Install -Run -Logcat

# Release 模式构建并安装
.\test-mobile.ps1 -Configuration Release -Install -Run
```

### 自动检测

| 组件 | 检测逻辑 |
|:---|:---|
| Android NDK | 先查 `ANDROID_NDK_HOME` / `ANDROID_NDK_ROOT`，再探测 VS 默认路径 |
| adb | 先查 PATH 中的 `adb`，再探测 VS Android SDK `platform-tools` |

VS 默认探测路径：
```
D:\Program Files (x86)\Microsoft Visual Studio\Shared\Android\AndroidNDK\android-ndk-r27c
D:\Program Files (x86)\Microsoft Visual Studio\Shared\Android\android-sdk\platform-tools\adb.exe
```

### 设备连接稳定性

脚本内置 **3 次自动重试** 机制：
1. 检测到 `offline` 设备 → 自动 `adb kill-server` / `start-server` → 重试
2. 未检测到设备 → 等待 3 秒 → 重试
3. 第 3 次仍失败 → 报错退出

---

## 常见问题排查（FAQ）

### Q1: `build.ps1` 桌面端编译报错 "CMake configuration failed"

**排查**：
```powershell
# 确认 CMake 和 Ninja 已安装
Get-Command cmake
Get-Command ninja

# 确认 Visual Studio 已安装（如用 MSVC）
Get-Command msbuild
```

**解决**：
- 未安装 CMake：从 [cmake.org](https://cmake.org/download/) 下载安装
- 未安装 Ninja：`choco install ninja` 或从 [GitHub Releases](https://github.com/ninja-build/ninja/releases) 下载

---

### Q2: `build.ps1` / `test-mobile.ps1` Android 端报错 "ANDROID_NDK_HOME not set"

**解决**：
```powershell
# 临时设置（当前会话有效）
$env:ANDROID_NDK_HOME = "C:\Your\Path\To\Android\Sdk\ndk\27.0.1"

# 永久设置（用户级）
[Environment]::SetEnvironmentVariable("ANDROID_NDK_HOME", "C:\Your\Path\To\Android\Sdk\ndk\27.0.1", "User")
```

如果通过 Visual Studio 安装了 Android 工作负载，脚本通常能自动探测到默认路径。

---

### Q3: `adb devices` 检测不到设备（空白 / `offline` / `unauthorized`）

**诊断步骤**：

```powershell
adb devices
```

| 输出 | 状态 | 解决 |
|:---|:---|:---|
| `xxxxxxxx    device` | ✅ 正常 | 可直接运行脚本 |
| `xxxxxxxx    offline` | ⚠️ 掉线 | `adb kill-server && adb start-server`，保持手机亮屏 |
| `xxxxxxxx    unauthorized` | ❌ 未授权 | 手机屏幕点击"允许 USB 调试"，或重新插拔 |
| 空白 | ❌ 未识别 | 换数据线、换 USB 口、开启开发者选项和 USB 调试 |

**常见原因**：
- **数据线仅支持充电**：换一根确定能传数据的线
- **USB 口供电不足**：插到机箱**后置 USB 口**
- **手机锁屏/休眠**：保持屏幕亮屏，关闭"USB 调试自动关闭"
- **弹窗未处理**：手机屏幕会有"允许 USB 调试吗？"弹窗，必须点击**确定**

---

### Q4: 安装 APK 时手机提示"禁止安装未知来源应用"

各厂商设置路径：
- **小米/Redmi**：设置 → 隐私保护 → 特殊权限设置 → 安装未知应用 → 允许"文件管理"
- **华为/荣耀**：设置 → 安全 → 更多安全设置 → 外部来源应用下载 → 开启
- **OPPO/一加/realme**：设置 → 密码与安全 → 系统安全 → 外部来源应用 → 允许
- **vivo/iQOO**：设置 → 安全与隐私 → 更多安全设置 → 安装未知应用 → 允许

---

### Q5: `test-mobile.ps1` 设备频繁掉线

1. 手机屏幕保持亮屏（开发者选项中开启"不锁定屏幕"）
2. 关闭 USB 调试的自动关闭功能（部分 MIUI/ColorOS 有）
3. 换 USB 口 / 换数据线
4. 如果脚本检测失败，直接使用手动命令：
   ```powershell
   # MAUI 版本（当前默认）
   adb install -r "dist/android/com.cide.mobile-Signed.apk"
   adb shell monkey -p com.cide.mobile -c android.intent.category.LAUNCHER 1
   
   # 旧版 Avalonia Android（已冻结）
   adb install -r "dist/android/com.cide.app-Signed.apk"
   adb shell monkey -p com.cide.app -c android.intent.category.LAUNCHER 1
   ```

---

## 手动应急命令

如果脚本因环境问题无法使用，可手动执行：

### 桌面端手动构建

```powershell
# Native 后端
cd native/build
cmake .. -G Ninja -DCMAKE_BUILD_TYPE=Debug
cmake --build . --parallel

# 前端
dotnet publish Cide.Client.Desktop/Cide.Client.Desktop.csproj -c Debug -o dist/desktop

# 运行
dist/desktop/Cide.Client.Desktop.exe
```

### Android 手动构建与安装

```powershell
# 1. NDK 交叉编译 arm64-v8a
$ndk = $env:ANDROID_NDK_HOME
$toolchain = "$ndk/build/cmake/android.toolchain.cmake"
cd native/build-android-arm64-v8a
cmake .. -G Ninja -DCMAKE_TOOLCHAIN_FILE="$toolchain" -DANDROID_ABI=arm64-v8a -DANDROID_PLATFORM=android-21 -DCMAKE_BUILD_TYPE=Debug -DCIDE_BUILD_TESTS=OFF
cmake --build . --parallel
cd ../..

# 2. 复制 .so
Copy-Item "native/build-android-arm64-v8a/lib/libcide_native.so" "Cide.Client.Maui/lib/arm64-v8a/" -Force

# 3. 构建 APK
dotnet publish Cide.Client.Maui/Cide.Client.Maui.csproj -c Debug -o dist/android

# 4. 安装并启动
adb install -r "dist/android/com.cide.app-Signed.apk"
adb shell monkey -p com.cide.app -c android.intent.category.LAUNCHER 1

# 5. 查看日志
adb logcat --pid=$(adb shell pidof com.cide.app)
```

---

## 相关文档

- [`OPTIMIZATION_AND_BUG_ANALYSIS_20260427.md`](OPTIMIZATION_AND_BUG_ANALYSIS_20260427.md) — 项目整体优化与 Bug 分析
