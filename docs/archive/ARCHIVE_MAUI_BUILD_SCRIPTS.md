# 已废弃：MAUI 构建脚本说明

> ⚠️ **本文档已归档，内容严重过时**。MAUI 前端已于 Phase 9 下线，当前项目使用 Flutter 前端。
> 最新构建指南请参见 [`docs/current/BUILD.md`](../current/BUILD.md) 和 [`docs/current/BUILD_SCRIPTS.md`](../current/BUILD_SCRIPTS.md)。

---

## 历史脚本清单

以下脚本曾在 MAUI 时代使用，现已从 `scripts/` 目录移除：

| 脚本 | 原功能 | 替代方案 |
|:---|:---|:---|
| `scripts/build.py` | 构建 Native 后端 + MAUI 前端（Desktop / Android） | `scripts/build_flutter.py` |
| `scripts/build_release.py` | Release 构建（MAUI Windows 自包含 + Android AOT/Trim） | `scripts/build_release.py`（已重写为 Flutter 版本） |
| `scripts/test_mobile.py` | MAUI Android 完整测试流水线 | `scripts/test_mobile.py`（已重写为 Flutter 版本） |
| `scripts/test_full_chain.py` | MAUI 全链验证（安装 → 启动 → smoke test） | 已删除，功能由 `scripts/test_mobile.py` 覆盖 |

---

## `scripts/build.py`（MAUI 版本）

### 功能概述

统一构建 Rust Native 后端和 C# MAUI 前端，支持桌面端（Windows）和 Android 端。

**构建流程**（桌面端）：
1. Native 后端：`cargo build` 编译 `cide_native.dll`
2. 前端：`dotnet publish` 打包 `Cide.Client.Maui` (Windows)
3. 将 DLL 复制到 `dist/desktop/`

**构建流程**（Android 端）：
1. Native 后端：`cargo ndk` 交叉编译 `arm64-v8a` + `armeabi-v7a` 的 `libcide_native.so`
2. 前端：`dotnet publish` 打包 MAUI APK 到 `dist/android/`

### 参数

| 参数 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `-c`, `--configuration` | `Debug` / `Release` | `Debug` | 构建配置 |
| `-t`, `--target` | `Desktop` / `Android` / `All` | `Desktop` | 构建目标平台 |
| `--clean` | flag | 关闭 | 清理所有构建产物 |
| `--run` | flag | 关闭 | 构建完成后运行桌面端应用 |
| `--test` | flag | 关闭 | 构建前运行 `cargo test` 和 `cargo clippy` |

### 使用示例

```bash
# 桌面端 Debug 构建（默认）
python scripts/build.py

# 桌面端 Release 构建，构建完成后直接运行
python scripts/build.py -c Release --run

# Android 端完整构建（NDK .so + APK）
python scripts/build.py -t Android
```

---

## `scripts/build_release.py`（MAUI 版本）

### 功能概述

- **Desktop**：Rust Release + MAUI Windows 单文件自包含发布
- **Android**：Rust Release NDK 交叉编译 + MAUI AOT + Trim + r8

### 参数

| 参数 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `-t`, `--target` | `Desktop` / `Android` / `All` | `All` | 构建目标平台 |
| `--clean` | flag | 关闭 | 清理所有构建产物 |

---

## `scripts/test_mobile.py`（MAUI 版本）

### 功能概述

专注于 **MAUI Android** 真机/模拟器的快速测试循环：

```
Native .so 编译 → APK 打包 → 设备安装 → 应用启动 → Logcat 日志抓取
```

### 参数

| 参数 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `-c`, `--configuration` | `Debug` / `Release` | `Debug` | 构建配置 |
| `--skip-native-build` | flag | 关闭 | 跳过 NDK `.so` 编译，仅重新打包 APK |
| `--install` | flag | 关闭 | APK 构建完成后自动安装到设备 |
| `--run` | flag | 关闭 | 安装后自动启动应用 |
| `--logcat` | flag | 关闭 | 启动后实时抓取应用日志 |

---

## `scripts/test_full_chain.py`（MAUI 版本）

### 功能概述

在已连接设备上验证完整链路：APK 安装 → 应用启动 → WebView 加载检测。

### 参数

| 参数 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `--device` | string | 自动探测 | 指定设备序列号 |
| `--apk-path` | string | `dist\android\com.cide.app-Signed.apk` | APK 路径 |
| `--skip-install` | flag | 关闭 | 跳过 APK 安装 |

---

## 手动应急命令（MAUI 时代）

### 桌面端手动构建

```bash
# Native 后端
cd native
cargo build --release

# 前端
dotnet publish Cide.Client.Maui/Cide.Client.Maui.csproj \
    -f net10.0-windows10.0.19041.0 \
    -c Release -o dist/desktop --self-contained false

# 运行
dist/desktop/Cide.Client.Maui.exe
```

### Android 手动构建与安装

```bash
# 1. NDK 交叉编译 arm64-v8a
cd native
cargo ndk -t aarch64-linux-android --platform 21 build --release

# 2. 复制 .so
mkdir -p Cide.Client.Maui/lib/arm64-v8a
cp native/target/aarch64-linux-android/release/libcide_native.so Cide.Client.Maui/lib/arm64-v8a/

# 3. 构建 APK
dotnet publish Cide.Client.Maui/Cide.Client.Maui.csproj \
    -f net10.0-android -c Release \
    -p:AndroidPackageFormat=apk -o dist/android

# 4. 安装并启动
adb install -r "dist/android/com.cide.app-Signed.apk"
adb shell monkey -p com.cide.app -c android.intent.category.LAUNCHER 1
```

---

> 归档时间：2026-05-14
> 下线原因：MAUI 前端已全面迁移至 Flutter，构建系统随之更新。
