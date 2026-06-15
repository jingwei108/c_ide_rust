# C IDE 构建指南

> [English Version](BUILD_EN.md)

本文档说明项目的构建流程、脚本用法和环境要求。

> **迁移说明**：前端已从 .NET MAUI 迁移至 Flutter。旧版 MAUI 构建文档见 [`docs/archive/ARCHIVE_MAUI_BUILD_SCRIPTS.md`](../archive/ARCHIVE_MAUI_BUILD_SCRIPTS.md)（已归档）。

---

## 环境要求

| 组件 | 版本 | 用途 |
|:---|:---|:---|
| Rust | 1.95.0+ | Native 后端（`cide_native`） |
| Cargo | 随 Rust 安装 | Rust 包管理 |
| cargo-ndk | 最新 | Android `.so` 交叉编译 |
| Flutter SDK | 3.24+ | 跨平台前端 |
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
| [`scripts/build_flutter.py`](../../scripts/build_flutter.py) | 构建 Native 后端 + Flutter 前端 | 日常开发编译 |
| [`scripts/build_release.py`](../../scripts/build_release.py) | Release 构建（Desktop + Android） | 发布打包 |
| [`scripts/test_mobile.py`](../../scripts/test_mobile.py) | 移动端完整流水线：构建 → 安装 → 启动 → 日志 | Flutter Android 真机/模拟器测试 |

> 旧版 MAUI 构建脚本已归档至 [`docs/archive/ARCHIVE_MAUI_BUILD_SCRIPTS.md`](../archive/ARCHIVE_MAUI_BUILD_SCRIPTS.md)。

---

## `scripts/build_flutter.py` — 日常构建

### 功能

1. **Native 后端（Rust）**：`cargo build [--release]` 编译 `cide_native.dll` / `.so`
2. **桌面端前端（Flutter Windows）**：`flutter build windows` + 自动复制 DLL
3. **移动端前端（Flutter Android）**：`flutter build apk`（自动集成 `.so`）
4. **FRB 代码生成**：必要时运行 `flutter_rust_bridge_codegen generate`

### 参数

| 参数 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `-c`, `--configuration` | `Debug` / `Release` | `Debug` | 构建配置 |
| `-t`, `--target` | `Desktop` / `Android` / `All` | `Desktop` | 构建目标平台 |
| `--clean` | flag | 关闭 | 清理所有构建产物 |
| `--run` | flag | 关闭 | 构建完成后运行桌面端应用（仅 Desktop） |
| `--offline` | flag | 关闭 | 离线构建（不下载 pub 依赖） |

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
```

---

## `scripts/build_release.py` — 发布构建

### 功能

- **Desktop**：Rust Release + Flutter Windows Release
- **Android**：Rust Release NDK 交叉编译 + Flutter APK Release

### 参数

| 参数 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `-t`, `--target` | `Desktop` / `Android` / `All` | `All` | 构建目标平台 |
| `--clean` | flag | 关闭 | 清理所有构建产物 |

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

### 功能

专注于 **Flutter Android** 真机/模拟器的快速测试循环：

```
Native .so 编译 → Flutter APK 打包 → 设备安装 → 应用启动 → Logcat 日志抓取
```

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

# 2. 复制 DLL 到 Flutter 项目
Copy-Item native/target/release/cide_native.dll CideFlutter/rust_builder/windows/ -Force

# 3. 前端（Flutter）
cd CideFlutter
flutter pub get --offline
flutter build windows --debug

# 4. 运行
flutter run -d windows
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

# 2. 前端（Flutter）
cd CideFlutter
flutter pub get --offline
flutter build apk --release

# 3. 安装并启动
adb install -r "build/app/outputs/flutter-apk/app-release.apk"
adb shell monkey -p com.cide.app -c android.intent.category.LAUNCHER 1

# 4. 查看日志
adb logcat --pid=$(adb shell pidof com.cide.app)
```

### 运行测试

```powershell
# Rust 后端测试
cd native
cargo test
cargo clippy

# Flutter 前端测试
cd CideFlutter
flutter test
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

### Q5: Flutter 构建报错 "Unable to find suitable Visual Studio"

确保安装了 "Desktop development with C++" 工作负载，或设置：
```powershell
$env:FLUTTER_ROOT = "C:\Your\Path\To\flutter"
```
