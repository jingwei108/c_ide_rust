# 简单数据持久化与自动恢复方案

> 目标：保存上一次编辑的代码（覆盖正常退出、后台杀死、崩溃场景），以及 UI 设置（主题、字体等）。
> 原则：极简实现，不引入过重依赖。

---

## 核心设计

| 数据类型 | 存储方式 | 理由 |
|---------|---------|------|
| **代码文本** | 本地文件 (`autosave.c`) | 代码可能很长（>100KB），`shared_preferences` 有大小限制且性能差 |
| **UI 设置**（主题、字体等） | `shared_preferences` | 数据量小，KV 存取最方便 |
| **保存时机** | Debounce 2秒 + 生命周期 pause + 正常退出 | 覆盖崩溃、后台杀死、正常退出所有场景 |

---

## 1. 底层存储服务

```dart
// CideFlutter/lib/services/local_storage_service.dart

import 'dart:io';
import 'dart:async';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// 所有本地持久化的统一入口
class LocalStorageService {
  static final LocalStorageService _instance = LocalStorageService._internal();
  factory LocalStorageService() => _instance;
  LocalStorageService._internal();

  SharedPreferences? _prefs;
  String? _docDir;

  Future<void> init() async {
    _prefs = await SharedPreferences.getInstance();
    _docDir = (await getApplicationDocumentsDirectory()).path;
  }

  // ========== 代码文件操作 ==========

  Future<String?> loadAutoSave() async {
    if (_docDir == null) return null;
    final file = File('$_docDir/autosave.c');
    if (!await file.exists()) return null;
    return file.readAsString();
  }

  Future<void> saveAutoSave(String code) async {
    if (_docDir == null) return;
    final file = File('$_docDir/autosave.c');
    await file.writeAsString(code, flush: true); // flush=true 确保落盘
  }

  Future<void> clearAutoSave() async {
    if (_docDir == null) return;
    final file = File('$_docDir/autosave.c');
    if (await file.exists()) await file.delete();
  }

  // ========== SharedPreferences 封装 ==========

  String? getString(String key) => _prefs?.getString(key);
  int? getInt(String key) => _prefs?.getInt(key);
  bool? getBool(String key) => _prefs?.getBool(key);
  double? getDouble(String key) => _prefs?.getDouble(key);

  Future<bool> setString(String key, String value) async =>
      await _prefs?.setString(key, value) ?? false;

  Future<bool> setInt(String key, int value) async =>
      await _prefs?.setInt(key, value) ?? false;

  Future<bool> setBool(String key, bool value) async =>
      await _prefs?.setBool(key, value) ?? false;

  Future<bool> setDouble(String key, double value) async =>
      await _prefs?.setDouble(key, value) ?? false;
}
```

---

## 2. 自动保存管理器

```dart
// CideFlutter/lib/services/auto_save_service.dart

import 'dart:async';
import 'local_storage_service.dart';

/// 管理代码的 debounce 自动保存
class AutoSaveService {
  final _storage = LocalStorageService();
  Timer? _debounceTimer;
  bool _isSaving = false;

  /// 触发自动保存（debounce 2秒）
  void onCodeChanged(String code) {
    _debounceTimer?.cancel();
    _debounceTimer = Timer(const Duration(seconds: 2), () {
      _doSave(code);
    });
  }

  /// 立即保存（用于生命周期变化或用户显式保存）
  Future<void> saveNow(String code) async {
    _debounceTimer?.cancel();
    await _doSave(code);
  }

  Future<void> _doSave(String code) async {
    if (_isSaving) return;
    _isSaving = true;
    await _storage.saveAutoSave(code);
    _isSaving = false;
  }

  Future<String?> loadLastCode() async {
    return await _storage.loadAutoSave();
  }

  Future<void> clear() async {
    _debounceTimer?.cancel();
    await _storage.clearAutoSave();
  }

  void dispose() {
    _debounceTimer?.cancel();
  }
}
```

---

## 3. UI 设置状态管理（替换现有硬编码主题）

```dart
// CideFlutter/lib/providers/settings_provider.dart

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../services/local_storage_service.dart';

class AppSettings {
  final ThemeMode themeMode;
  final double fontSize;
  final bool showLineNumbers;
  final bool wordWrap;
  final bool autoSaveEnabled;

  const AppSettings({
    this.themeMode = ThemeMode.dark,
    this.fontSize = 14.0,
    this.showLineNumbers = true,
    this.wordWrap = false,
    this.autoSaveEnabled = true,
  });

  AppSettings copyWith({
    ThemeMode? themeMode,
    double? fontSize,
    bool? showLineNumbers,
    bool? wordWrap,
    bool? autoSaveEnabled,
  }) {
    return AppSettings(
      themeMode: themeMode ?? this.themeMode,
      fontSize: fontSize ?? this.fontSize,
      showLineNumbers: showLineNumbers ?? this.showLineNumbers,
      wordWrap: wordWrap ?? this.wordWrap,
      autoSaveEnabled: autoSaveEnabled ?? this.autoSaveEnabled,
    );
  }
}

class SettingsNotifier extends Notifier<AppSettings> {
  final _storage = LocalStorageService();

  @override
  AppSettings build() {
    // 启动时从本地读取
    return _loadFromStorage();
  }

  AppSettings _loadFromStorage() {
    final themeIndex = _storage.getInt('settings.themeMode') ?? 0; // 0=dark, 1=light, 2=system
    return AppSettings(
      themeMode: ThemeMode.values[themeIndex.clamp(0, 2)],
      fontSize: _storage.getDouble('settings.fontSize') ?? 14.0,
      showLineNumbers: _storage.getBool('settings.showLineNumbers') ?? true,
      wordWrap: _storage.getBool('settings.wordWrap') ?? false,
      autoSaveEnabled: _storage.getBool('settings.autoSaveEnabled') ?? true,
    );
  }

  Future<void> _persist() async {
    await _storage.setInt('settings.themeMode', state.themeMode.index);
    await _storage.setDouble('settings.fontSize', state.fontSize);
    await _storage.setBool('settings.showLineNumbers', state.showLineNumbers);
    await _storage.setBool('settings.wordWrap', state.wordWrap);
    await _storage.setBool('settings.autoSaveEnabled', state.autoSaveEnabled);
  }

  void setThemeMode(ThemeMode mode) {
    state = state.copyWith(themeMode: mode);
    _persist();
  }

  void toggleTheme() {
    final next = state.themeMode == ThemeMode.dark ? ThemeMode.light : ThemeMode.dark;
    state = state.copyWith(themeMode: next);
    _persist();
  }

  void setFontSize(double size) {
    state = state.copyWith(fontSize: size);
    _persist();
  }

  void toggleLineNumbers() {
    state = state.copyWith(showLineNumbers: !state.showLineNumbers);
    _persist();
  }

  void toggleWordWrap() {
    state = state.copyWith(wordWrap: !state.wordWrap);
    _persist();
  }

  void toggleAutoSave() {
    state = state.copyWith(autoSaveEnabled: !state.autoSaveEnabled);
    _persist();
  }
}

final settingsProvider = NotifierProvider<SettingsNotifier, AppSettings>(
  SettingsNotifier.new,
);
```

---

## 4. 生命周期监听（覆盖后台杀死/崩溃）

```dart
// CideFlutter/lib/services/app_lifecycle_observer.dart

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../providers/ide_provider.dart';
import 'auto_save_service.dart';

/// 监听 App 生命周期，在后台/退出前强制保存
class AppLifecycleObserver extends WidgetsBindingObserver {
  final AutoSaveService _autoSave;
  final Ref _ref;

  AppLifecycleObserver(this._autoSave, this._ref);

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    switch (state) {
      case AppLifecycleState.paused:
      case AppLifecycleState.detached:
      case AppLifecycleState.inactive:
        // App 进入后台或被杀死前，强制保存当前代码
        final code = _ref.read(ideProvider).source;
        _autoSave.saveNow(code);
        break;
      case AppLifecycleState.resumed:
      case AppLifecycleState.hidden:
        // 不需要处理
        break;
    }
  }
}
```

---

## 5. 现有文件修改点

### 5.1 `main.dart`：初始化存储服务

```dart
// CideFlutter/lib/main.dart

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  
  // 初始化本地存储（必须在 runApp 前完成）
  await LocalStorageService().init();
  
  runApp(const ProviderScope(child: MyApp()));
}
```

### 5.2 `theme_provider.dart`：废弃硬编码，接入 settingsProvider

直接删除 `theme_provider.dart` 或改为导出：

```dart
// CideFlutter/lib/providers/theme_provider.dart
// 改为从 settings_provider 导出，保持兼容
export 'settings_provider.dart' show settingsProvider;
```

所有原来用 `ref.watch(themeProvider)` 的地方改为：
```dart
final themeMode = ref.watch(settingsProvider).themeMode;
```

### 5.3 `editor_panel.dart`：启动加载 + 编辑时触发保存

```dart
// CideFlutter/lib/widgets/editor_panel.dart

import '../services/auto_save_service.dart';

class EditorPanelState extends ConsumerState<EditorPanel> {
  late CodeLineEditingController _controller;
  final _autoSave = AutoSaveService();
  final _focusNode = FocusNode();

  @override
  void initState() {
    super.initState();

    // ========== 启动时加载自动保存的代码 ==========
    _loadAutoSave();

    // 延迟添加 listener
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        _controller.addListener(_onChanged);
      }
    });
  }

  /// 加载上次保存的代码
  Future<void> _loadAutoSave() async {
    final lastCode = await _autoSave.loadLastCode();
    if (lastCode != null && lastCode.isNotEmpty) {
      // 恢复代码到编辑器
      _controller = CodeLineEditingController.fromText(
        lastCode,
        const CodeLineOptions(indentSize: 4),
      );
      // 同步到 provider
      ref.read(ideProvider.notifier).updateSource(lastCode);
      setState(() {});
    } else {
      // 没有自动保存，使用默认/模板代码
      final source = ref.read(ideProvider).source;
      _controller = CodeLineEditingController.fromText(
        source,
        const CodeLineOptions(indentSize: 4),
      );
    }
    _lastLineCount = _controller.lineCount;
  }

  void _onChanged() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        final text = _controller.text;
        ref.read(ideProvider.notifier).updateSource(text);
        
        // ========== 触发自动保存 ==========
        final autoSaveEnabled = ref.read(settingsProvider).autoSaveEnabled;
        if (autoSaveEnabled) {
          _autoSave.onCodeChanged(text);
        }
      }
    });
    // ... 原有换行补分号逻辑不变
  }

  @override
  void dispose() {
    // 退出前强制保存一次
    _autoSave.saveNow(_controller.text);
    _autoSave.dispose();
    _controller.dispose();
    _focusNode.dispose();
    super.dispose();
  }
}
```

### 5.4 `ide_screen.dart`：绑定生命周期监听

```dart
class _IdeScreenState extends ConsumerState<IdeScreen>
    with SingleTickerProviderStateMixin, WidgetsBindingObserver {
  
  final _autoSave = AutoSaveService();

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this); // ← 新增
    // ... 原有代码
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this); // ← 新增
    _autoSave.dispose();
    // ... 原有 dispose
    super.dispose();
  }

  // ========== 新增生命周期回调 ==========
  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    switch (state) {
      case AppLifecycleState.paused:
      case AppLifecycleState.detached:
      case AppLifecycleState.inactive:
        final code = ref.read(ideProvider).source;
        _autoSave.saveNow(code);
        break;
      default:
        break;
    }
  }
}
```

---

## 6. 新增/修改的文件清单

| 文件 | 动作 | 说明 |
|------|------|------|
| `lib/services/local_storage_service.dart` | **新建** | 文件 + SP 统一封装 |
| `lib/services/auto_save_service.dart` | **新建** | Debounce 自动保存 |
| `lib/providers/settings_provider.dart` | **新建** | 替代 `theme_provider`，管理所有 UI 设置 |
| `lib/services/app_lifecycle_observer.dart` | **新建** | 生命周期监听（可选，也可直接写在 IdeScreen） |
| `lib/main.dart` | **修改** | `runApp` 前 `await LocalStorageService().init()` |
| `lib/providers/theme_provider.dart` | **修改** | 改为导出 `settings_provider` 或删除 |
| `lib/widgets/editor_panel.dart` | **修改** | 启动时 `_loadAutoSave()`，编辑时 `_autoSave.onCodeChanged()` |
| `lib/screens/ide_screen.dart` | **修改** | `WidgetsBindingObserver` + `didChangeAppLifecycleState` |
| `pubspec.yaml` | **修改** | 加 `path_provider: ^2.1.5`（如果之前图片方案没加的话） |

---

## 7. 异常场景覆盖验证

| 场景 | 是否覆盖 | 机制 |
|------|---------|------|
| 用户正常点击退出 | ✅ | `EditorPanel.dispose()` 中 `saveNow()` |
| App 切后台（Home 键） | ✅ | `didChangeAppLifecycleState(paused)` |
| 系统内存不足杀死 App | ✅ | `paused` 时已保存，debounce 的 2 秒也大概率已触发 |
| App 崩溃 | ⚠️ 部分 | 崩溃前 2 秒内的修改可能丢失，这是可接受的 |
| 手机重启 | ✅ | 文件持久化在磁盘，启动时 `_loadAutoSave()` 读取 |
| 卸载重装 | ❌ | 应用数据被系统清除，无法恢复，这是预期行为 |

---

## 8. 最小可行版本（如果只想最快落地）

如果只想先实现**"退出后代码不丢"**这一个功能，最短路径是：

```dart
// 在 editor_panel.dart 的 initState 和 dispose 中各加 3 行：

void initState() {
  super.initState();
  // ... 原有代码 ...
  _loadLastCode(); // 启动时读
}

Future<void> _loadLastCode() async {
  final dir = await getApplicationDocumentsDirectory();
  final file = File('${dir.path}/autosave.c');
  if (await file.exists()) {
    final code = await file.readAsString();
    _controller = CodeLineEditingController.fromText(code);
    ref.read(ideProvider.notifier).updateSource(code);
  }
}

@override
void dispose() {
  // 退出时写
  getApplicationDocumentsDirectory().then((dir) {
    File('${dir.path}/autosave.c').writeAsString(_controller.text);
  });
  _controller.dispose();
  super.dispose();
}
```

不需要 `shared_preferences`，不需要 `path_provider` 依赖以外的任何东西，5 分钟就能跑通。
