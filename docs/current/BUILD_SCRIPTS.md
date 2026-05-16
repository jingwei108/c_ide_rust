# C IDE 构建与测试脚本指南

本文档说明项目中所有 Python 构建脚本的使用方法、参数和常见问题排查。

> **技术栈**：后端 Rust/Cargo，前端 Flutter，构建脚本 Python 3。
> 
> 旧版 MAUI 构建脚本已归档至 [`docs/archive/ARCHIVE_MAUI_BUILD_SCRIPTS.md`](../archive/ARCHIVE_MAUI_BUILD_SCRIPTS.md)。

---

## 脚本清单

| 脚本 | 功能 | 适用场景 |
|:---|:---|:---|
| [`scripts/build_flutter.py`](../scripts/build_flutter.py) | 构建 Native 后端 + Flutter 前端（Desktop / Android） | 日常开发编译、打包 |
| [`scripts/build_release.py`](../scripts/build_release.py) | Release 构建（Desktop + Android） | 发布打包 |
| [`scripts/test_mobile.py`](../scripts/test_mobile.py) | 移动端完整测试流水线：构建 → 安装 → 启动 → 日志 | Flutter Android 真机/模拟器测试 |

所有脚本共用 [`scripts/build_utils.py`](../scripts/build_utils.py) 中的通用工具（彩色输出、命令执行、ADB/NDK 自动探测、设备检测等）。

---

## `scripts/build_flutter.py` — 日常构建

### 功能概述

统一构建 Rust Native 后端和 Flutter 前端，支持桌面端（Windows）和 Android 端。

**构建流程**（桌面端）：
1. Native 后端：`cargo build` 编译 `cide_native.dll`
2. 前端：`flutter build windows` 打包 Flutter Windows 应用
3. 将 DLL 复制到 Flutter 构建输出目录

**构建流程**（Android 端）：
1. Native 后端：`cargo ndk` 交叉编译 `arm64-v8a` + `armeabi-v7a` 的 `libcide_native.so`
2. 前端：`flutter build apk` 打包 Flutter APK

### 参数

| 参数 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `-c`, `--configuration` | `Debug` / `Release` | `Debug` | 构建配置 |
| `-t`, `--target` | `Desktop` / `Android` / `All` | `Desktop` | 构建目标平台 |
| `--clean` | flag | 关闭 | 清理 Flutter 构建产物 |
| `--run` | flag | 关闭 | 构建完成后运行桌面端应用（仅 `-t Desktop` 有效） |
| `--offline` | flag | 关闭 | 离线构建（`flutter pub get --offline`） |
| `--skip-rust` | flag | 关闭 | 跳过手动 Rust 构建（由 cargokit 自动处理） |
| `--test` | flag | 关闭 | 构建前运行 `cargo test` 和 `cargo clippy` |

### 使用示例

```bash
# 桌面端 Debug 构建（默认）
python scripts/build_flutter.py

# 桌面端 Release 构建，构建完成后直接运行
python scripts/build_flutter.py -c Release --run

# 清理并重新构建桌面端
python scripts/build_flutter.py --clean -t Desktop

# Android 端完整构建（NDK .so + APK）
python scripts/build_flutter.py -t Android

# 离线构建（无网络环境）
python scripts/build_flutter.py --offline

# 构建前运行测试和 lint
python scripts/build_flutter.py --test
```

### 环境变量

| 变量 | 说明 |
|:---|:---|
| `ANDROID_NDK_HOME` / `ANDROID_NDK_ROOT` | Android NDK 路径，用于 `-t Android` |

---

## `scripts/build_release.py` — 发布构建

### 功能概述

- **Desktop**：Rust Release + Flutter Windows Release
- **Android**：Rust Release NDK 交叉编译 + Flutter APK Release

### 参数

| 参数 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `-t`, `--target` | `Desktop` / `Android` / `All` | `All` | 构建目标平台 |
| `--clean` | flag | 关闭 | 清理构建产物 |

### 使用示例

```bash
# 构建桌面端和 Android 端 Release
python scripts/build_release.py

# 仅构建桌面端
python scripts/build_release.py -t Desktop

# 清理后构建
python scripts/build_release.py --clean
```

---

## `scripts/test_mobile.py` — 移动端测试流水线

### 功能概述

专注于 **Flutter Android** 真机/模拟器的快速测试循环：

```
Native .so 编译 → Flutter APK 打包 → 设备安装 → 应用启动 → Logcat 日志抓取
```

与 `build_flutter.py -t Android` 的区别：
- `build_flutter.py` 仅构建输出到 `CideFlutter/build/`
- `test_mobile.py` 额外完成安装、启动、日志抓取

### 参数

| 参数 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `-c`, `--configuration` | `Debug` / `Release` | `Debug` | 构建配置 |
| `--skip-native-build` | flag | 关闭 | 跳过 NDK `.so` 编译，仅重新打包 APK |
| `--install` | flag | 关闭 | APK 构建完成后自动安装到设备 |
| `--run` | flag | 关闭 | 安装后自动启动应用 |
| `--logcat` | flag | 关闭 | 启动后实时抓取应用日志（`Ctrl+C` 停止） |

### 使用示例

```bash
# 仅构建 APK（含 Native .so）
python scripts/test_mobile.py

# 快速重新打包（前端代码改动后，跳过 .so 编译）
python scripts/test_mobile.py --skip-native-build --install --run

# 构建 + 安装 + 启动 + 实时日志（完整测试流水线）
python scripts/test_mobile.py --install --run --logcat

# Release 模式构建并安装
python scripts/test_mobile.py -c Release --install --run
```

### 自动检测

| 组件 | 检测逻辑 |
|:---|:---|
| Android NDK | 先查 `ANDROID_NDK_HOME` / `ANDROID_NDK_ROOT`，再探测 VS 默认路径 |
| adb | 先查 PATH 中的 `adb`，再探测 VS Android SDK `platform-tools` |
| flutter | 先查 PATH 中的 `flutter`，再探测常见 Windows 安装路径 |

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

### Q1: 脚本报错 "cargo ndk command not found"

**解决**：
```bash
cargo install cargo-ndk
```

---

### Q2: Android 端报错 "ANDROID_NDK_HOME not set"

**解决**：
```bash
# 临时设置（当前会话有效）
export ANDROID_NDK_HOME="/path/to/ndk/27.0.1"
# Windows PowerShell:
# $env:ANDROID_NDK_HOME = "C:\Your\Path\To\ndk\27.0.1"
```

如果通过 Visual Studio 安装了 Android 工作负载，脚本通常能自动探测到默认路径。

---

### Q3: `adb devices` 检测不到设备（空白 / `offline` / `unauthorized`）

**诊断步骤**：

```bash
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

### Q5: `test_mobile.py` 设备频繁掉线

1. 手机屏幕保持亮屏（开发者选项中开启"不锁定屏幕"）
2. 关闭 USB 调试的自动关闭功能（部分 MIUI/ColorOS 有）
3. 换 USB 口 / 换数据线
4. 如果脚本检测失败，直接使用手动命令：
   ```bash
   adb install -r "CideFlutter/build/app/outputs/flutter-apk/app-release.apk"
   adb shell monkey -p com.example.cide -c android.intent.category.LAUNCHER 1
   ```

---

## 手动应急命令

如果脚本因环境问题无法使用，可手动执行：

### 桌面端手动构建

```bash
# Native 后端
cd native
cargo build --release          # Release
cargo build                    # Debug

# DLL 输出路径
# Release: native/target/release/cide_native.dll
# Debug:   native/target/debug/cide_native.dll

# 前端
cd CideFlutter
flutter pub get --offline
flutter build windows --debug   # Debug
flutter build windows --release # Release

# 运行
flutter run -d windows
```

### Android 手动构建与安装

```bash
# 1. NDK 交叉编译 arm64-v8a
cd native
cargo ndk -t aarch64-linux-android --platform 21 build --release

# 2. 复制 .so
mkdir -p native/target/android/arm64-v8a
cp native/target/aarch64-linux-android/release/libcide_native.so native/target/android/arm64-v8a/

# 3. 构建 APK
cd CideFlutter
flutter build apk --release

# 4. 安装并启动
adb install -r "build/app/outputs/flutter-apk/app-release.apk"
adb shell monkey -p com.example.cide -c android.intent.category.LAUNCHER 1

# 5. 查看日志
adb logcat --pid=$(adb shell pidof com.example.cide)
```

---

## 相关文档

- [`BUILD.md`](BUILD.md) — 构建指南与环境要求
- [`AGENTS.md`](../AGENTS.md) — Agent 快速参考（构建命令速查）
- [`docs/archive/ARCHIVE_MAUI_BUILD_SCRIPTS.md`](../archive/ARCHIVE_MAUI_BUILD_SCRIPTS.md) — 已归档的 MAUI 构建脚本说明
