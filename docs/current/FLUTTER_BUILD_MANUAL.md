# Cide Flutter 手动构建手册

> 本文档面向需要从零开始搭建 Flutter 前端构建环境，或需要深入理解连接层与手动构建流程的开发者。
> 
> 日期：2026-05-13  
> 关联脚本：`scripts/build_flutter.py`

---

## 目录

1. [环境准备](#1-环境准备)
2. [连接层（flutter_rust_bridge）](#2-连接层flutter_rust_bridge)
3. [手动构建流程](#3-手动构建流程)
4. [构建脚本自动化](#4-构建脚本自动化)
5. [故障排查](#5-故障排查)

---

## 1. 环境准备

### 1.1 必需工具链

| 工具 | 版本要求 | 用途 | 验证命令 |
|------|---------|------|---------|
| Flutter SDK | `>=3.29.0` | Dart 前端 + 跨平台构建 | `flutter doctor` |
| Rust toolchain | `>=1.95.0` | `cide_native` 后端 | `rustc --version` |
| cargo-ndk | 最新 | Android `.so` 交叉编译 | `cargo ndk --version` |
| Android SDK | API 34+ | Android 构建基础 | `sdkmanager --list` |
| Android NDK | `27.1.12297006`（推荐） | Rust 交叉编译链接器 | `$ANDROID_NDK_HOME/ndk-build --version` |
| Visual Studio 2022+ | 含 C++ 桌面开发 | Windows CMake 构建 | `cmake --version` |

### 1.2 Flutter 环境检查

```bash
flutter doctor
```

必须全部通过（或仅有可接受的警告）：
- ✅ Flutter (Channel stable, 3.29.0+)
- ✅ Windows toolchain (Visual Studio / CMake)
- ✅ Android toolchain (SDK + NDK)
- ✅ Connected device（如需真机调试）

### 1.3 Rust 环境检查

```bash
rustc --version        # >= 1.95.0
cargo --version
cargo ndk --version    # 如缺失：cargo install cargo-ndk
```

### 1.4 环境变量（Windows PowerShell 示例）

```powershell
# Flutter
$env:PATH += ";D:\flutter\bin"

# Android
$env:ANDROID_HOME = "C:\Users\<用户名>\AppData\Local\Android\Sdk"
$env:ANDROID_NDK_HOME = "$env:ANDROID_HOME\ndk\27.1.12297006"
$env:PATH += ";$env:ANDROID_HOME\platform-tools"

# Rust（rustup 默认已加入 PATH）
```

---

## 2. 连接层（flutter_rust_bridge）

### 2.1 架构位置

```
Flutter 前端 (Dart)
  │
  ├─ lib/main.dart                     # 入口，初始化 FRB
  ├─ lib/src/rust/api/cide.dart        # FRB 自动生成的 Dart 绑定
  ├─ lib/src/rust/frb_generated.dart   # FRB 生成的序列化/反序列化代码
  │
  ▼
flutter_rust_bridge (Dart 侧运行时)
  │
  ▼
dart:ffi → DynamicLibrary.open("cide_native.dll")
  │
  ▼
Rust 后端 (cide_native)
  ├─ native/src/api/cide.rs            # FRB 公开 API（#[frb] 标记）
  ├─ native/src/api/mod.rs             # API 模块入口
  ├─ native/src/flutter_bridge.rs      # 业务包装层（Session 管理）
  ├─ native/src/capi/mod.rs            # C API（MAUI / P/Invoke 兼容层）
  └─ native/src/lib.rs                 # crate 入口
```

### 2.2 关键设计原则

**不改动核心 Rust 逻辑**：FRB 连接层只做**类型转换**和**包装**，所有编译器/VM 代码保持原样。

**Session 全局单例**：`flutter_bridge.rs` 使用 `lazy_static! + Mutex<Session>` 维护全局会话状态，避免 Dart 侧管理 Rust 对象生命周期。

**双接口共存**：
- `capi/mod.rs` → C API（`extern "C"`）→ MAUI 通过 P/Invoke 调用
- `api/cide.rs` → FRB API（`#[frb]`）→ Flutter 通过 dart:ffi 调用

两者底层共享同一个 `Session` 和 `CideVM` 实例。

### 2.3 重新生成绑定（Rust API 变更后）

```bash
export PATH="$PATH:/d/flutter/bin"
cd CideFlutter
flutter_rust_bridge_codegen generate
```

**触发条件**：
- 修改了 `native/src/api/cide.rs` 中的 `#[frb]` 函数签名
- 新增了 `#[frb]` 公开函数
- 修改了 `#[frb]` 结构体字段

**不需要重新生成的场景**：
- 修改 `native/src/flutter_bridge.rs` 的内部实现
- 修改 `native/src/compiler/` / `native/src/vm/` 等核心模块
- 纯 Dart 侧代码修改

### 2.4 连接层关键文件速查

| 文件 | 职责 | 修改频率 |
|------|------|---------|
| `native/src/api/cide.rs` | FRB 公开 API 定义 | 低（仅新增功能时） |
| `native/src/api/mod.rs` | API 模块注册 | 低 |
| `native/src/flutter_bridge.rs` | 业务包装 + Session 管理 | 中 |
| `native/flutter_rust_bridge.yaml` | FRB 生成配置 | 极低 |
| `CideFlutter/lib/src/rust/api/cide.dart` | **自动生成**，勿手动修改 | 自动 |
| `CideFlutter/lib/src/rust/frb_generated.dart` | **自动生成**，勿手动修改 | 自动 |
| `CideFlutter/rust_builder/` | cargokit 跨平台构建插件 | 极低 |

---

## 3. 手动构建流程

### 3.1 Windows 桌面端（手动模式）

**适用场景**：
- Windows **未启用开发者模式**（cargokit 无法创建符号链接）
- 需要精确控制 Rust 编译配置
- 调试 Rust 后端时

**步骤 1：构建 Rust DLL**

```bash
cd native
cargo build --release
# 输出：native/target/release/cide_native.dll
```

Debug 模式：
```bash
cargo build
# 输出：native/target/debug/cide_native.dll
```

**步骤 2：复制 DLL 到 Flutter 构建目录**

```bash
# Release
cp native/target/release/cide_native.dll CideFlutter/build/windows/x64/runner/Release/

# Debug
cp native/target/release/cide_native.dll CideFlutter/build/windows/x64/runner/Debug/
```

> 注意：`flutter build windows` 可能会重新创建输出目录，建议在 `flutter build` **之后**再复制一次，或使用构建脚本自动处理。

**步骤 3：获取 Flutter 依赖**

```bash
cd CideFlutter
flutter pub get
# 离线环境：flutter pub get --offline
```

**步骤 4：构建 Flutter Windows**

```bash
# Debug
flutter build windows --debug

# Release
flutter build windows --release
```

**步骤 5：运行（可选）**

```bash
flutter run -d windows
```

**输出位置**：
- Debug: `CideFlutter/build/windows/x64/runner/Debug/`
- Release: `CideFlutter/build/windows/x64/runner/Release/`

### 3.2 Windows 桌面端（自动模式 / cargokit）

**适用场景**：
- Windows **已启用开发者模式**（`ms-settings:developers`）
- 希望 `flutter build` 一键完成 Rust + Flutter 编译

**启用开发者模式后**：

```bash
cd CideFlutter
flutter pub get
flutter build windows --release
```

`rust_builder` 插件会通过 `cargokit` 自动：
1. 调用 `cargo build` 生成 DLL
2. 将 DLL 复制到正确的输出目录
3. 完成 Flutter 构建

### 3.3 Android 移动端（手动模式）

**适用场景**：
- NDK 配置特殊，需要手动控制 ABI
- cargokit 自动构建失败时的 fallback

**步骤 1：构建 Rust .so（多 ABI）**

```bash
cd native

# arm64-v8a
cargo ndk --target aarch64-linux-android --platform 21 build --release

# armeabi-v7a
cargo ndk --target armv7-linux-androideabi --platform 21 build --release

# x86_64（可选，模拟器）
cargo ndk --target x86_64-linux-android --platform 21 build --release
```

**步骤 2：复制 .so 到 Flutter Android 项目**

```bash
# arm64-v8a
cp native/target/aarch64-linux-android/release/libcide_native.so \
   CideFlutter/android/app/src/main/jniLibs/arm64-v8a/

# armeabi-v7a
cp native/target/armv7-linux-androideabi/release/libcide_native.so \
   CideFlutter/android/app/src/main/jniLibs/armeabi-v7a/
```

**步骤 3：构建 Flutter APK**

```bash
cd CideFlutter
flutter pub get
flutter build apk --release
```

**输出位置**：
`CideFlutter/build/app/outputs/flutter-apk/app-release.apk`

### 3.4 Android 移动端（自动模式 / cargokit + Gradle）

**适用场景**：标准开发流程，NDK 和 Gradle 已正确配置。

当前项目已配置：
- 阿里云 Maven 镜像（`settings.gradle.kts` / `build.gradle.kts`）
- 本地 Gradle wrapper（`gradle-8.13-bin.zip`）
- NDK 版本 `27.1.12297006`

```bash
cd CideFlutter
flutter pub get
flutter build apk --release
```

Gradle 构建阶段，`rust_builder` 会自动调用 cargokit 编译 Rust .so 并打包进 APK。

---

## 4. 构建脚本自动化

### 4.1 推荐用法

日常开发使用 `scripts/build_flutter.py`，它封装了上述手动流程：

```bash
# 桌面端 Debug（自动 cargo build + 复制 DLL + flutter build）
python scripts/build_flutter.py

# 桌面端 Release + 运行
python scripts/build_flutter.py -c Release --run

# Android APK（离线环境）
python scripts/build_flutter.py -t Android --offline

# 清理 + 重新构建
python scripts/build_flutter.py --clean --offline

# 开发者模式已启用时（跳过手动 Rust 构建，让 cargokit 处理）
python scripts/build_flutter.py --skip-rust
```

### 4.2 脚本与手动构建的映射关系

| 手动步骤 | 脚本对应逻辑 |
|---------|------------|
| `cargo build --release` | `build_rust_desktop()` |
| `cp cide_native.dll .../runner/Release/` | `copy_dll_to_flutter_build()` |
| `flutter pub get --offline` | `run([flutter, "pub", "get", "--offline"])` |
| `flutter build windows --release` | `run([flutter, "build", "windows", "--release"])` |
| `cargo ndk --target ... build --release` | `build_rust_android()` |
| `flutter build apk --release` | `build_flutter_android()` |

### 4.3 构建脚本关系

| 脚本 | 用途 | 目标前端 |
|------|------|---------|
| `scripts/build_flutter.py` | Flutter Desktop / Android 日常构建 | Flutter |
| `scripts/build_release.py` | Flutter Release 发布构建（Desktop + Android） | Flutter |
| `scripts/test_mobile.py` | Flutter Android 移动端完整流水线 | Flutter |

---

## 5. 故障排查

### 5.1 `flutter doctor` 报 Windows 工具链缺失

**现象**：`Visual Studio - develop for Windows` 打叉。

**解决**：
1. 打开 Visual Studio Installer
2. 修改 → 工作负荷 → 勾选 **"使用 C++ 的桌面开发"**
3. 单个组件 → 确保 **"C++ CMake tools for Windows"** 和 **"Windows 10/11 SDK"** 已安装

### 5.2 `cargo ndk` 命令找不到

**现象**：`cargo: error: no such subcommand: ndk`

**解决**：
```bash
cargo install cargo-ndk
```

### 5.3 Windows 构建报错：`Cannot create symlink`

**现象**：
```
CMake Error: failed to create symbolic link ...
```

**原因**：未启用 Windows 开发者模式，`cargokit` 需要创建符号链接 `.plugin_symlinks`。

**方案 A（推荐）**：启用开发者模式
```powershell
# 以管理员运行 PowerShell
reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock" /t REG_DWORD /f /v "AllowDevelopmentWithoutDevLicense" /d "1"
# 或图形界面：设置 → 隐私和安全性 → 开发者模式 → 开
```

**方案 B（fallback）**：使用手动构建模式（见 3.1），绕过 cargokit 的符号链接需求。

### 5.4 `flutter build apk` 报错：Gradle 下载失败

**现象**：SSL 握手错误，或 `gradle-8.13-bin.zip` 下载超时。

**解决**：项目已配置本地 Gradle，检查：
```properties
# CideFlutter/android/gradle/wrapper/gradle-wrapper.properties
distributionUrl=file:///C:/.../gradle-8.13-bin.zip
```

确保路径指向有效的本地 zip 文件。

### 5.5 `flutter pub get --offline` 报包缺失

**现象**：`Could not find package xxx in cache`

**解决**：
1. 确保该包在 `~/AppData/Local/Pub/Cache/hosted/pub.dev/`
2. 目录名格式必须为 `包名-版本号`（如 `re_editor-0.8.0`）
3. 如果包不在缓存中，需在有网络的机器上先 `flutter pub get`，然后将缓存目录复制过来

### 5.6 FRB 绑定生成失败

**现象**：`flutter_rust_bridge_codegen generate` 报错。

**常见原因**：
- `native/src/api/cide.rs` 中使用了 FRB 不支持的 Rust 类型（如 `HashMap<K, V>` 需改为 `Vec<(K, V)>`）
- 同名类型冲突（如 `SourceLoc` 在多个模块定义）

**解决**：
1. 检查 `native/flutter_rust_bridge.yaml` 的 `rust_input` 配置
2. 确保所有 `#[frb]` 类型均为 FRB 支持的基础类型（`String`, `Vec<T>`, `i32`, `bool`, 自定义 struct）
3. 同名类型通过 `#[frb(name = "...")]` 重命名

### 5.7 运行时崩溃：`Failed to load dynamic library`

**现象**：Flutter 启动后立即崩溃，日志显示找不到 `cide_native.dll`。

**原因**：DLL 未复制到 Flutter 可执行文件的同级目录。

**解决**：
```bash
# 确认 DLL 存在
ls CideFlutter/build/windows/x64/runner/Release/cide_native.dll

# 如缺失，手动复制
cp native/target/release/cide_native.dll CideFlutter/build/windows/x64/runner/Release/
```

---

## 附录：快速参考卡

```bash
# === 一次性环境检查 ===
flutter doctor
rustc --version
cargo ndk --version

# === Flutter 桌面端（手动） ===
cd native && cargo build --release
cp target/release/cide_native.dll ../CideFlutter/build/windows/x64/runner/Release/
cd ../CideFlutter && flutter run -d windows

# === Flutter Android（手动） ===
cd native
cargo ndk --target aarch64-linux-android --platform 21 build --release
cp target/aarch64-linux-android/release/libcide_native.so \
   ../CideFlutter/android/app/src/main/jniLibs/arm64-v8a/
cd ../CideFlutter && flutter build apk --release

# === Flutter 脚本构建 ===
python scripts/build_flutter.py                    # Desktop Debug
python scripts/build_flutter.py -c Release --run   # Desktop Release + 运行
python scripts/build_flutter.py -t Android         # Android APK
python scripts/build_flutter.py --offline          # 离线模式
```
