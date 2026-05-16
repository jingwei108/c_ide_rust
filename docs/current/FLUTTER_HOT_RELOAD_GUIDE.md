# Flutter 热重载调试指南

> 适用于 Cide 项目（Flutter + Rust 混合工程）

---

## 环境准备

### 1. 确认设备已连接

```bash
cd CideFlutter
flutter devices
```

典型输出示例：

```
Found 3 connected devices:
  PKT110 (mobile)   • V8XSK75PMR9HYPTG • android-arm64  • Android 16 (API 36)
  Windows (desktop) • windows          • windows-x64    • Microsoft Windows [版本 10.0.26200.8246]
  Edge (web)        • edge             • web-javascript • Microsoft Edge 148.0.3967.54
```

### 2. 首次运行前构建 Rust 后端

Flutter UI 代码支持热重载，但 **Rust 后端（`native/`）需要预先编译成动态库**。

#### Windows 桌面端

```bash
# 方式一：使用项目脚本（推荐）
cd D:\code\c_ide_rust
python scripts/build_flutter.py

# 方式二：手动构建
cd native
cargo build
cd ..

# 手动复制 DLL 到 Flutter 构建目录（如果脚本未自动复制）
copy native\target\debug\cide_native.dll CideFlutter\build\windows\x64\runner\Debug\
```

> **注意**：如果启用了 **Windows 开发者模式**，`cargokit` 可能在 `flutter run` 时自动构建 Rust，无需手动复制 DLL。

#### Android 端

Android 的 `.so` 库通常由 `cargokit` 在 Gradle 构建阶段自动编译，无需手动干预。如果遇到缺失 `.so` 的错误，执行：

```bash
cd native
cargo ndk -t aarch64-linux-android --platform 21 build
cargo ndk -t armv7-linux-androideabi --platform 21 build
```

---

## 启动热重载调试

### 命令行方式（最稳定）

#### Android 真机/模拟器

```bash
cd CideFlutter
flutter run -d <设备ID>

# 示例
flutter run -d V8XSK75PMR9HYPTG
flutter run -d PKT110           # 也可以用设备名称
```

#### Windows 桌面端

```bash
cd CideFlutter
flutter run -d windows
```

成功启动后，终端会进入交互模式，显示以下提示：

```
Flutter run key commands.
r Hot reload. 🔥🔥🔥
R Hot restart.
h List all available interactive commands.
d Detach (terminate "flutter run" but leave application running).
c Clear the screen
q Quit (terminate the application on the device).
```

### VS Code 图形化方式（更方便）

#### 步骤 1：创建启动配置

在 `CideFlutter/.vscode/launch.json` 写入：

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Cide Flutter (Desktop)",
      "cwd": "CideFlutter",
      "request": "launch",
      "type": "dart",
      "deviceId": "windows",
      "args": []
    },
    {
      "name": "Cide Flutter (Android)",
      "cwd": "CideFlutter",
      "request": "launch",
      "type": "dart",
      "deviceId": "android",
      "args": []
    }
  ]
}
```

#### 步骤 2：按 F5 启动

- 修改代码 → **保存（Ctrl+S）**
- VS Code 会自动触发 **Hot Reload**
- 调试控制台会显示 `Reloaded X libraries`

---

## Hot Reload vs Hot Restart

| 操作 | 按键 | 速度 | 状态保留 | 适用场景 |
|------|------|------|----------|----------|
| **Hot Reload** | `r` | ~1 秒 | ✅ 保留 | 修改 UI 样式、布局、Dart 业务逻辑 |
| **Hot Restart** | `R` | ~3-5 秒 | ❌ 重置 | 修改 `main()`、全局状态、初始化逻辑 |

### 判断该用哪个

- **只改了 `lib/` 下的 Dart 文件（Widget、Provider、样式）** → 按 `r`
- **改了 `main()`、`MaterialApp`、Riverpod 全局 Provider** → 按 `R`
- **改了 Rust 代码（`native/src/`）** → 先重新 `cargo build`，再按 `R`

---

## 修改不同代码的处理方式

### 1. 纯 Dart/Flutter UI 代码

例如修改 `lib/screens/ide_screen.dart` 中的布局、颜色、组件：

```dart
// 修改前
return Scaffold(backgroundColor: Colors.black);

// 修改后
return Scaffold(backgroundColor: Colors.red);
```

**操作**：保存文件 → 终端按 `r`（或 VS Code 自动重载）

**效果**：界面立刻变红，应用状态（如输入的代码、滚动位置）完全保留。

### 2. Riverpod 状态管理代码

例如修改 `lib/providers/ide_notifier.dart` 中的某个方法：

**操作**：保存文件 → 按 `r`

**注意**：如果修改了 Provider 的初始化逻辑或数据结构，可能需要按 `R`。

### 3. Rust 后端代码

例如修改 `native/src/vm/` 或 `native/src/compiler/` 中的逻辑：

**操作**：

```bash
# 1. 重新构建 Rust
cd native
cargo build

# 2. 确保 DLL 已复制到 Flutter 构建目录
# （Windows）
copy target\debug\cide_native.dll ..\CideFlutter\build\windows\x64\runner\Debug\

# 3. 回到 Flutter 终端，按 R（Hot Restart）
```

> Rust 代码 **不支持** Flutter Hot Reload，必须重新编译 native 库后 Hot Restart。

### 4. 依赖变更（`pubspec.yaml`）

增删 Flutter 依赖包后：

```bash
flutter pub get
```

然后**停止当前运行**，重新执行 `flutter run`。

---

## 常见问题排查

### 问题 1：`flutter run` 报错 `No pubspec.yaml file found`

**原因**：当前目录不在 `CideFlutter/` 下。

**解决**：

```bash
cd D:\code\c_ide_rust\CideFlutter
flutter run -d <设备>
```

### 问题 2：按 `r` 后提示 `Reloaded 0 libraries`

**原因**：文件未保存，或修改的是 `const` Widget。

**解决**：
- 确保已按 **Ctrl+S** 保存
- VS Code 用户建议开启 **Auto Save**（文件 → 自动保存）
- 如果 Widget 声明为 `const`，Flutter 可能缓存了旧状态，尝试去掉 `const` 或按 `R`

### 问题 3：修改后界面没变化

**解决**：
1. 按 `R`（Hot Restart）强制刷新
2. 检查修改的文件是否正在被使用（是否 import 到了入口文件 `main.dart`）
3. 如果修改了主题色，确认 `MaterialApp` 的 `theme` 配置已更新

### 问题 4：`flutter run` 报错找不到 `cide_native.dll`

**原因**：Rust 后端未构建或 DLL 未复制到正确位置。

**解决**：

```bash
cd D:\code\c_ide_rust
python scripts/build_flutter.py
```

或手动：

```bash
cd native
cargo build
copy target\debug\cide_native.dll ..\CideFlutter\build\windows\x64\runner\Debug\
```

### 问题 5：Android 构建报错 `assembleDebug failed`

**排查步骤**：

1. 确认 Rust Android `.so` 已构建：
   ```bash
   cd native
   cargo ndk -t aarch64-linux-android --platform 21 build
   ```

2. 清理 Flutter 构建缓存后重试：
   ```bash
   cd CideFlutter
   flutter clean
   flutter pub get
   flutter run -d <设备>
   ```

### 问题 6：终端被占用，无法输入 `r`

**解决**：
- 按 **Enter** 键确认终端焦点，再输入 `r`
- 或在新终端窗口执行 `flutter attach` 附加到正在运行的应用

---

## 高效调试技巧

### 技巧 1：同时调试多个设备

开两个终端窗口，分别运行：

```bash
# 终端 1：桌面端
flutter run -d windows 

# 终端 2：Android 真机
flutter run -d PKT110
```

修改代码保存后，两个终端都可以独立按 `r` 热重载。

### 技巧 2：使用 `--hot` 和 `--track-widget-builds`

```bash
flutter run -d windows --track-widget-builds
```

开启 **Widget 构建追踪**，在 DevTools 中可以看到每个 Widget 的重建次数，帮助定位性能问题。

### 技巧 3：Detach 模式

按 `d` 让 `flutter run` 断开连接，但应用继续运行。之后可以用 `flutter attach` 重新连接：

```bash
flutter attach -d windows
```

### 技巧 4：Dart DevTools

```bash
flutter pub global activate devtools
flutter pub global run devtools
```

浏览器打开 `http://127.0.0.1:9100`，可以查看：
- Widget Inspector（检查 UI 树）
- Performance（性能分析）
- Network（网络请求）
- Logging（日志输出）

---

## 速查表

```bash
# 查看设备
flutter devices

# 启动调试（桌面端）
cd CideFlutter && flutter run -d windows

# 启动调试（Android 真机）
cd CideFlutter && flutter run -d <设备ID>

# 构建 Release APK
cd CideFlutter && flutter build apk --release

# 清理构建缓存
cd CideFlutter && flutter clean

# 获取依赖
cd CideFlutter && flutter pub get

# Rust 桌面端构建
cd native && cargo build

# Rust Android 构建
cd native && cargo ndk -t aarch64-linux-android --platform 21 build
```

---

## 参考

- [Flutter Hot Reload 官方文档](https://docs.flutter.dev/tools/hot-reload)
- [项目构建手册](BUILD.md)
- [Flutter 构建脚本说明](BUILD_SCRIPTS.md)
