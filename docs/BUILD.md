# C IDE 构建指南

本文档说明项目的构建流程、脚本用法和环境要求。

> **迁移说明**：后端已从 C++/CMake 迁移至 Rust/Cargo。旧版 CMake 构建文档见 `BUILD_SCRIPTS.md`（已废弃）。

---

## 环境要求

| 组件 | 版本 | 用途 |
|:---|:---|:---|
| Rust | 1.95.0+ | Native 后端（`cide_native`） |
| Cargo | 随 Rust 安装 | Rust 包管理 |
| cargo-ndk | 最新 | Android `.so` 交叉编译 |
| .NET SDK | 10.0+ | C# 前端编译 |
| Android NDK | 27+ | Android Native 后端交叉编译（可选） |
| adb | 随 Android SDK | Android 设备安装调试（可选） |

### 安装 cargo-ndk

```powershell
cargo install cargo-ndk
```

### Android NDK 环境变量

```powershell
# 临时设置（当前会话）
$env:ANDROID_NDK_HOME = "C:\Your\Path\To\ndk\27.0.1"

# 永久设置
[Environment]::SetEnvironmentVariable("ANDROID_NDK_HOME", "C:\Your\Path\To\ndk\27.0.1", "User")
```

---

## 脚本清单

| 脚本 | 功能 | 适用场景 |
|:---|:---|:---|
| [`build.ps1`](../build.ps1) | 构建 Native 后端 + Avalonia/Maui 前端 | 日常开发编译 |
| [`build-release.ps1`](../build-release.ps1) | Release 构建（Desktop AOT + Android Trim） | 发布打包 |
| [`test-mobile.ps1`](../test-mobile.ps1) | 移动端完整流水线：构建 → 安装 → 启动 → 日志 | MAUI Android 真机/模拟器测试 |

---

## `build.ps1` — 日常构建

### 功能

1. **Native 后端（Rust）**：`cargo build [--release]` 编译 `cide_native.dll` / `.so`
2. **桌面端前端（Avalonia）**：`dotnet publish` 打包 `Cide.Client.Desktop`
3. **移动端前端（MAUI）**：`dotnet publish` 打包 `Cide.Client.Maui` APK
4. 自动将 Native DLL/.so 复制到正确位置

### 参数

| 参数 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `-Configuration` | `Debug` / `Release` | `Debug` | 构建配置 |
| `-Target` | `Desktop` / `Android` / `All` | `Desktop` | 构建目标平台 |
| `-Clean` | Switch | 关闭 | 清理所有构建产物 |
| `-Run` | Switch | 关闭 | 构建完成后运行桌面端应用（仅 Desktop） |
| `-Test` | Switch | 关闭 | 构建前运行 `cargo test` 和 `cargo clippy` |

### 使用示例

```powershell
# 桌面端 Debug 构建（默认）
.\build.ps1

# 桌面端 Release 构建，构建完成后直接运行
.\build.ps1 -Configuration Release -Run

# 清理并重新构建桌面端
.\build.ps1 -Clean -Target Desktop

# 构建前运行测试和 clippy
.\build.ps1 -Test -Target Desktop

# Android 端完整构建（NDK .so + APK）
.\build.ps1 -Target Android

# 同时构建桌面端和 Android 端
.\build.ps1 -Target All
```

---

## `build-release.ps1` — 发布构建

### 功能

- **Desktop**：Rust Release + Avalonia Native AOT（单文件自包含）
- **Android**：Rust Release NDK 交叉编译 + MAUI AOT + Trim + r8

### 参数

| 参数 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `-Target` | `Desktop` / `Android` / `All` | `All` | 构建目标平台 |
| `-Clean` | Switch | 关闭 | 清理所有构建产物 |

### 使用示例

```powershell
# 构建桌面端和 Android 端 Release
.\build-release.ps1

# 仅构建桌面端
.\build-release.ps1 -Target Desktop

# 清理后构建
.\build-release.ps1 -Clean
```

---

## `test-mobile.ps1` — 移动端测试流水线

### 功能

专注于 **MAUI Android** 真机/模拟器的快速测试循环：

```
Native .so 编译 → APK 打包 → 设备安装 → 应用启动 → Logcat 日志抓取
```

### 参数

| 参数 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `-Configuration` | `Debug` / `Release` | `Debug` | 构建配置 |
| `-SkipNativeBuild` | Switch | 关闭 | 跳过 NDK `.so` 编译，仅重新打包 APK |
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

## 手动构建

如果脚本因环境问题无法使用，可手动执行：

### 桌面端

```powershell
# 1. Native 后端（Rust）
cd native
cargo build --release          # Release
cargo build                    # Debug

# DLL 输出路径
# Release: native/target/release/cide_native.dll
# Debug:   native/target/debug/cide_native.dll

# 2. 前端
dotnet publish Cide.Client.Desktop/Cide.Client.Desktop.csproj `
    -c Release -o dist/desktop --self-contained false

# 3. 运行
dist/desktop/Cide.Client.Desktop.exe
```

### Android 端

```powershell
# 1. Native 后端（Rust NDK 交叉编译）
cd native

# arm64-v8a
cargo ndk -t aarch64-linux-android -o target/android build --release

# armeabi-v7a
cargo ndk -t armv7-linux-androideabi -o target/android build --release

# .so 输出路径
# native/target/android/arm64-v8a/libcide_native.so
# native/target/android/armeabi-v7a/libcide_native.so

# 2. 复制 .so 到 Maui 项目
Copy-Item native/target/android/arm64-v8a/libcide_native.so   Cide.Client.Maui/lib/arm64-v8a/   -Force
Copy-Item native/target/android/armeabi-v7a/libcide_native.so Cide.Client.Maui/lib/armeabi-v7a/ -Force

# 3. 构建 APK
dotnet publish Cide.Client.Maui/Cide.Client.Maui.csproj `
    -f net10.0-android -c Release `
    -p:AndroidPackageFormat=apk -o dist/android

# 4. 安装并启动
adb install -r "dist/android/com.cide.mobile-Signed.apk"
adb shell monkey -p com.cide.mobile -c android.intent.category.LAUNCHER 1

# 5. 查看日志
adb logcat --pid=$(adb shell pidof com.cide.mobile)
```

### 运行测试

```powershell
# Rust 后端测试
cd native
cargo test
cargo clippy

# C# 前端测试
dotnet test
```

---

## 常见问题

### Q1: `cargo ndk` 命令未找到

```powershell
cargo install cargo-ndk
```

### Q2: Android 构建报错 "ANDROID_NDK_HOME not set"

```powershell
$env:ANDROID_NDK_HOME = "C:\Your\Path\To\ndk\27.0.1"
```

或通过 Visual Studio 安装器添加 "Android 开发" 工作负载，脚本会自动探测默认路径。

### Q3: `adb devices` 检测不到设备

```powershell
adb devices
```

| 输出 | 状态 | 解决 |
|:---|:---|:---|
| `xxxxxxxx    device` | ✅ 正常 | 可直接运行脚本 |
| `xxxxxxxx    offline` | ⚠️ 掉线 | `adb kill-server && adb start-server`，保持手机亮屏 |
| `xxxxxxxx    unauthorized` | ❌ 未授权 | 手机屏幕点击"允许 USB 调试" |
| 空白 | ❌ 未识别 | 换数据线、换 USB 口、开启开发者选项和 USB 调试 |

### Q4: 安装 APK 时提示"禁止安装未知来源应用"

各厂商设置路径：
- **小米/Redmi**：设置 → 隐私保护 → 特殊权限设置 → 安装未知应用
- **华为/荣耀**：设置 → 安全 → 更多安全设置 → 外部来源应用下载
- **OPPO/一加/realme**：设置 → 密码与安全 → 系统安全 → 外部来源应用
- **vivo/iQOO**：设置 → 安全与隐私 → 更多安全设置 → 安装未知应用
