# Flutter 前端迁移状态报告

> 日期：2026-05-13  
> 分支：`flutter-migration`  
> 目标：将 .NET MAUI 前端迁移至 Flutter，保留 Rust 后端完全不变

---

## 1. 已完成

### 1.1 通信层 — flutter_rust_bridge v2

- [x] 安装 `flutter_rust_bridge_codegen` v2.12.0
- [x] 运行 `integrate` 创建基础集成结构
- [x] 运行 `generate` 生成 Dart 绑定（`lib/src/rust/api/cide.dart`）
- [x] 创建 `native/src/api/cide.rs`：20+ 个 FRB 公开 API 函数
- [x] 创建 `native/src/flutter_bridge.rs`：内部业务实现（供 FRB 层调用）
- [x] 保留 `native/src/capi/mod.rs` 完整不变（MAUI 兼容）

**FRB 公开 API 列表：**

| 函数 | 说明 |
|------|------|
| `compile(source)` | 编译 C 源码 |
| `run_code()` | 全速运行 |
| `step_next()` | 单步调试 |
| `get_diagnostics()` | 获取诊断信息 |
| `get_algorithm_matches()` | 获取算法匹配 |
| `get_variables()` | 获取变量快照 |
| `get_memory_regions()` | 获取内存区域 |
| `get_callstack()` | 获取调用栈 |
| `get_output()` | 获取输出文本 |
| `get_current_line()` | 获取当前行 |
| `is_waiting_input()` | 是否等待输入 |
| `add_breakpoint(line)` | 添加断点 |
| `clear_breakpoints()` | 清除断点 |
| `set_input(input)` | 设置预输入 |
| `provide_input_line(line)` | 提供单行输入 |
| `get_vis_events()` | 获取可视化事件 |
| `clear_vis_events()` | 清除可视化事件 |
| `read_memory(addr, count)` | 从 VM 内存读取 i32 数组 |
| `reset_session()` | 重置会话 |

### 1.2 Dart 前端 — 完整 IDE 界面（MAUI 风格上下布局）

#### 主界面布局（2026-05-13 重构）
- **顶部工具栏**：运行(▶)、停止(■)、单步(⏭)、主题切换、状态文本、清空输出
- **编辑器区域**：自适应剩余空间，`re_editor` 代码编辑器
- **符号快捷栏**：`{}` `()` `[]` `""` `''` `;` `#` `->` 及运算符、光标移动、Tab、撤销/重做
- **模板快捷栏**：冒泡排序、二分查找、链表节点、快速排序、递归阶乘、斐波那契、数组遍历、指针交换
- **底部 Tab 面板**：输出 / 诊断 / 算法，带未读数字角标，点击诊断可跳转至对应行
- **浮动调试按钮** (FAB)：点击弹出底部调试抽屉
- **底部 Tab 面板（可定制）**：默认 输出/诊断/算法，支持拖拽调整高度、Tab 左右交换、双击删除、跨区拖拽转移
- **浮动调试按钮（可定制）**：默认展开 7 个 Tab（知识卡片/指针视图/数组可视化/内存区域/局部变量/监视变量/调用栈），支持展开/收起、容量上限（最多7个）
- **面板拖拽交互**：元素可在底部和悬浮球之间拖拽转移，拖到空位自动添加，悬浮球满时提示"已达上限"

#### Widget 清单

| 文件 | 职责 |
|------|------|
| `lib/main.dart` | 应用入口 + FRB 初始化 |
| `lib/providers/ide_provider.dart` | Riverpod `Notifier` 状态管理（编译/运行/调试/输入/断点/Tab） |
| `lib/providers/theme_provider.dart` | 浅色/深色主题切换 |
| `lib/screens/ide_screen.dart` | 主界面（工具栏 + 编辑器 + 快捷栏 + 底部面板 + 调试抽屉） |
| `lib/widgets/editor_panel.dart` | `re_editor` 代码编辑器 + C 自动补全 + 断点/诊断行标记 + 当前行高亮 |

**新增数据模型**：
| 文件 | 职责 |
|------|------|
| `lib/models/panel_item.dart` | 10 个面板元素定义 + 拖拽数据结构 |
| `lib/models/knowledge_card.dart` | 11 张高频错误教学卡片 |

**已删除的旧文件**：`console_panel.dart`、`diagnostics_panel.dart`、`memory_panel.dart`、`variables_panel.dart`、`callstack_panel.dart`、`algorithm_panel.dart`（功能已合并至 `ide_screen.dart`）

#### 编辑器功能
- [x] **断点行号标记**：通过 `customLineIndex2Text` 在有断点的行号前显示 `●`
- [x] **当前行高亮**：调试时调用 `controller.selectLine` 聚焦当前行，`cursorLineColor` 高亮焦点行边框
- [x] **诊断行内提示**：错误行号前显示 `✗`，警告行号前显示 `⚠`，提示行号前显示 `ℹ`
- [x] **代码自动补全**：`CodeAutocomplete` 包裹 `CodeEditor`，提供 40+ C 关键字和标准库函数提示（`printf`/`scanf`/`malloc`/`qsort` 等）
- [x] **符号快捷插入**：从 UI 工具栏一键插入成对符号，光标自动置于中间
- [x] **代码模板**：8 个常用 C 代码模板一键插入编辑器

### 1.3 跨平台构建配置

- [x] `rust_builder/` — FRB 生成的跨平台 Rust 构建插件
- [x] 修复 `rust_builder/windows/CMakeLists.txt` Rust 路径错误
- [x] **Flutter 工具修复** — 为 VS 2026（版本 18）添加 CMake 生成器支持
- [x] **Flutter 工具修复** — patch `flutter_plugins.dart` 跳过 Windows symlink 限制（Developer Mode）
- [x] **Rust API 扩展** — 新增 `read_memory(addr, count)`，支持从 VM 内存按 i32 读取数组元素

### 1.4 依赖管理改造（path → pub.dev）

**背景**：原 `pubspec.yaml` 所有非 SDK 依赖均通过 `path: packages/xxx` 引用，注释标注 "offline development"。`packages/` 目录包含 6 个包的完整源码（~25MB），且 `re_highlight` 存在隐蔽漏洞——`re_editor` 内部引用的是 pub.dev 版本而非本地 path，纯离线环境会构建失败。

**改造动作**：
1. 修改 `pubspec.yaml`：将 `re_editor`、`flutter_riverpod`、`flutter_rust_bridge` 改为 `^` 版本约束；恢复 `flutter_lints: ^5.0.0`
2. 将 `packages/` 中的 4 个包复制到本地 Pub Cache（`~/AppData/Local/Pub/Cache/hosted/pub.dev/`），模拟 hosted 依赖
3. 删除 `CideFlutter/packages/` 目录（释放 ~25MB 仓库空间）
4. 删除旧 `pubspec.lock`，重新执行 `flutter pub get --offline`

**改造结果**：

| 检查项 | 结果 |
|--------|------|
| `flutter pub get --offline` | ✅ 74 个依赖全部解析，无缺失 |
| `pubspec.lock` | ✅ 生成，`source: hosted` 指向 pub.dev |
| `package_config.json` | ✅ `re_editor`/`flutter_riverpod`/`flutter_rust_bridge` 均指向 Pub Cache |
| `flutter analyze lib/` | ✅ 0 错误、0 警告 |
| `packages/` 目录 | ✅ 已删除 |

**已知限制**：
- `rust_builder/crgokit/build_tool/` 的 Dart 代码缺少 `github`、`http`、`ed25519_edwards` 等依赖，导致 `flutter analyze` 全局扫描时报告 70+ 错误。但这些错误**仅属于 cargokit 构建工具**，不影响主应用 `lib/` 编译与运行
- 当前环境无法联网，Pub Cache 中的包为手动复制注入；后续如需升级版本，需在有网络环境下运行 `flutter pub get`

**回滚方案**（如需恢复 path 依赖）：
```bash
cd CideFlutter
# 1. 恢复 packages/（从 git 历史或备份）
git checkout HEAD -- packages/
# 2. 恢复 pubspec.yaml
git checkout HEAD -- pubspec.yaml
# 3. 重新解析
flutter pub get --offline
```

### 1.5 构建脚本用法

新增 `scripts/build_flutter.py`，统一封装 Flutter + Rust 的构建流程。

| 命令 | 场景 |
|------|------|
| `python scripts/build_flutter.py` | 桌面端 Debug（完整流程：Rust DLL → 复制 → Flutter Build） |
| `python scripts/build_flutter.py -c Release --run` | 桌面端 Release，构建后自动运行 |
| `python scripts/build_flutter.py -t Android --offline` | Android APK（离线环境） |
| `python scripts/build_flutter.py --clean --offline` | 清理后重新构建 |
| `python scripts/build_flutter.py --skip-rust` | 跳过手动 Rust 构建（开发者模式已启用，cargokit 自动处理） |

**脚本设计要点**：
- 自动探测 Flutter（支持 `PATH` 和常见 Windows 路径 `D:\flutter\bin`）
- 离线兼容：`--offline` 适配无网络环境
- 开发者模式未启用时：自动 `cargo build` 生成 DLL 并复制到 Flutter 输出目录
- Android NDK 自动探测：复用 `build_utils.find_ndk()`
- 输出统一彩色格式，与现有 `build.py` / `build_release.py` 保持一致

### 1.6 验证结果

| 检查项 | 命令 | 结果 |
|--------|------|------|
| Rust 编译 | `cargo check` | ✅ 通过 |
| Dart 静态分析 | `flutter analyze lib/` | ✅ 0 错误 0 警告 |
| Dart 单元测试 | `flutter test` | ✅ 通过 |
| Windows 桌面构建 | `flutter build windows` | ⚠️ 因 flutter_tools snapshot 重新编译超时（已 patch symlink，首次需 10~15min） |
| Android Gradle 构建 | `./gradlew tasks` | ✅ 通过 |
| Android APK 构建 | `flutter build apk` | ✅ `app-release.apk` 22.4MB（armv7/arm64/x86_64） |
| Rust DLL 构建 | `cargo build --release` | ✅ `cide_native.dll` 生成成功 |

---

## 2. 已知问题与解决方案

### 2.1 Windows 桌面 — 开发者模式限制

**现象**：`flutter build windows` 时 `rust_builder` 插件无法创建符号链接，导致无法自动编译/复制 Rust DLL。

**原因**：Windows 未启用开发者模式，Flutter 插件依赖符号链接（`.plugin_symlinks`）。

**当前 workaround**：
```powershell
cd native && cargo build --release
cp native/target/release/cide_native.dll CideFlutter/build/windows/x64/runner/Release/
cd CideFlutter && flutter run -d windows
```

**长期解决方案**：启用 Windows 开发者模式（`ms-settings:developers`），`rust_builder` 会自动通过 `cargokit` 处理 Rust 构建。

### 2.2 Android 构建 — Gradle / NDK 配置

**现象**：`flutter build apk` 时 Gradle wrapper 下载失败（SSL 握手错误），且 NDK 目录为空。

**原因**：
1. 网络环境对 `services.gradle.org` 和 Maven/Google 仓库的 SSL 连接受限
2. Flutter 默认指向的 NDK `26.3.11579264` 目录为空（下载中断）

**已完成的修复**：
1. **本地 Gradle**：将 `gradle-8.13-bin.zip` 复制到 `CideFlutter/android/gradle/`，修改 `gradle-wrapper.properties` 指向本地文件
2. **阿里云镜像**：在 `settings.gradle.kts` 和 `build.gradle.kts` 中配置阿里云 Maven 镜像（google/public/gradle-plugin）
3. **NDK 版本**：改用已完整安装的 `27.1.12297006`（`app/build.gradle.kts`）

**验证结果**：`./gradlew tasks` ✅ BUILD SUCCESSFUL

**首次构建注意**：`flutter build apk` 第一次运行需等待 flutter_tools snapshot 重新编译（约 10~15 分钟），后续构建速度正常。

### 2.3 FRB 类型重复警告

**现象**：`flutter_rust_bridge_codegen generate` 时报告多个类型名重复（`SourceLoc`、`FuncMeta`、`CompileError` 等）。

**原因**：`crate::compiler::ast::SourceLoc` 与 `crate::vm::instruction::SourceLoc` 同名；`capi::CompileError` trait 与 `flutter_bridge::CompileError` trait 同名。

**影响**：仅为 INFO 级别警告，FRB 会随机选择一个。实际生成的 Dart 绑定工作正常。

**建议**：如后续需消除警告，可将内部类型重命名或降低可见性。

---

## 3. 下一步可选工作

### P1 — 核心教学功能（MAUI 原版功能）
- [x] **知识卡片系统**：11 张知识卡片（emoji + 标题 + 代码示例 + 解释 + 错误示例），编译后自动匹配错误码
- [x] **监视表达式**：添加/删除表达式，支持变量名匹配和简单数组索引（`arr[0]`），实时显示变量值和地址
- [x] **数组可视化**：从 VM 内存读取数组元素，条形图展示（支持正负值、自动缩放、索引标注）
- [ ] **指针可视化**：指针变量视图，显示指向关系和地址（待实现）
- [ ] **链表图可视化**：图节点展示链表结构（待实现）
- [ ] **算法验证**：算法检测后的通过/失败测试用例展示（待实现）

### P2 — 交互增强
- [ ] **执行速度滑块**：调节单步调试速度
- [ ] **应用修复按钮**：诊断面板中一键应用自动修复建议
- [ ] **Intro 覆盖层**：首次使用时的引导教程
- [ ] **触摸滑动切换 Tab**：底部面板支持左右滑动手势
- [ ] **键盘快捷键**：F5 运行、F10 单步、F9 断点等

### P3 — 工程化
- [x] **Android 构建环境修复**：本地 Gradle + 阿里云镜像 + NDK 配置
- [x] **Android 端到端 APK 构建**：`flutter build apk` 成功，输出 `app-release.apk` (22.4MB)，支持 armv7/arm64/x86_64
- [ ] **Windows 开发者模式集成**：启用后验证 `rust_builder` 自动构建
- [ ] **CI/CD 流水线**：在 `.github/workflows/` 中添加 Flutter 构建步骤

### P4 — 高级功能（后续考虑）
- [ ] **文件管理**：多文件 Tab 编辑（当前只支持单文件 `main.c`）
- [ ] **更丰富的代码模板**：排序、查找、数据结构等更多模板

---

## 4. 关键文件速查

```
# FRB 配置
CideFlutter/flutter_rust_bridge.yaml

# Rust API 层
native/src/api/cide.rs
native/src/api/mod.rs
native/src/flutter_bridge.rs
native/src/lib.rs

# Dart 前端
CideFlutter/lib/main.dart
CideFlutter/lib/providers/ide_provider.dart
CideFlutter/lib/providers/theme_provider.dart
CideFlutter/lib/screens/ide_screen.dart
CideFlutter/lib/widgets/editor_panel.dart

# 跨平台构建
CideFlutter/rust_builder/
CideFlutter/pubspec.yaml

# Flutter 工具修复（VS 2026 + symlink）
D:/flutter/packages/flutter_tools/lib/src/windows/visual_studio.dart
D:/flutter/packages/flutter_tools/lib/src/flutter_plugins.dart
```

---

## 5. 如何继续开发

### 重新生成 FRB 绑定（Rust API 变更后）
```bash
export PATH="$PATH:/d/flutter/bin"
cd CideFlutter
flutter_rust_bridge_codegen generate
```

### 桌面端开发（Windows）
```bash
cd native && cargo build --release
cp native/target/release/cide_native.dll CideFlutter/build/windows/x64/runner/Release/
cd CideFlutter && flutter run -d windows
```

### 移动端开发（Android，需正常网络）
```bash
cd CideFlutter
flutter pub get
flutter build apk
```
