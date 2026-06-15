# Flutter 构建注意事项

> 制定时间：2026-06-15
> 依据：`docs/current/ROADMAP_2026_Q3.md` A3.2
> 适用范围：Windows 桌面端 CI/本地构建、Flutter 3.29.x

---

## 1. Flutter 工具 Patch（Windows 桌面端）

### 1.1 问题背景

Flutter 3.29 在 Windows 上构建时会根据检测到的 Visual Studio 版本显式传入 `-G <generator>` 参数，这会覆盖 `CMAKE_GENERATOR` 环境变量。当 CI runner 或本地机器上存在多个 VS 版本时，Flutter 工具可能回退到不存在的 generator（例如 `Visual Studio 16 2019`），导致 `flutter build windows` 失败。

### 1.2 Patch 目的

让 Flutter 工具优先读取 `CMAKE_GENERATOR` 环境变量，使 CI 和本地开发者可以显式控制 generator，避免 Flutter 自动探测失败。

### 1.3 Patch 位置与内容

**修改文件**（Flutter SDK 内部）：

```text
<flutter-sdk>/packages/flutter_tools/lib/src/windows/build_windows.dart
```

**原始代码**（约第 1xx 行，视 Flutter 版本而定）：

```dart
final String? cmakeGenerator = visualStudio.cmakeGenerator;
```

**Patch 后代码**：

```dart
final String? cmakeGenerator = globals.platform.environment['CMAKE_GENERATOR'] ?? visualStudio.cmakeGenerator;
```

**清除缓存**（必须执行，否则 Dart 代码不会被重新编译）：

```powershell
Remove-Item "$flutterRoot\bin\cache\flutter_tools.*" -Force -ErrorAction SilentlyContinue
```

### 1.4 项目中的自动化 Patch

本项目的 `.github/workflows/ci.yml` 在 `flutter` job 中已包含自动 Patch 步骤：

```yaml
- name: Patch Flutter tools to honor CMAKE_GENERATOR
  run: |
    $flutterRoot = (Get-Command flutter).Source | Split-Path -Parent | Split-Path -Parent
    $buildWindows = Join-Path $flutterRoot "packages\flutter_tools\lib\src\windows\build_windows.dart"
    (Get-Content $buildWindows) `
      -replace 'final String\? cmakeGenerator = visualStudio\.cmakeGenerator;', 'final String? cmakeGenerator = globals.platform.environment[''CMAKE_GENERATOR''] ?? visualStudio.cmakeGenerator;' |
      Set-Content $buildWindows
    Remove-Item "$flutterRoot\bin\cache\flutter_tools.*" -Force -ErrorAction SilentlyContinue
  shell: pwsh
```

### 1.5 Patch 失效后的处理步骤

若 CI 或本地构建突然失败并提示找不到 generator，按以下顺序排查：

1. **确认 Flutter 版本**：本 Patch 针对 Flutter 3.29.x。若升级到 3.30+，Flutter 可能已修复此问题或变更了代码位置，需重新评估是否需要 Patch。
2. **检查正则是否仍匹配**：
   ```powershell
   $buildWindows = "<flutter-sdk>\packages\flutter_tools\lib\src\windows\build_windows.dart"
   Select-String -Path $buildWindows -Pattern "cmakeGenerator = visualStudio\.cmakeGenerator"
   ```
   若无输出，说明 Flutter 源码已变化，需重新定位赋值语句。
3. **手动设置环境变量作为兜底**：
   ```powershell
   $env:CMAKE_GENERATOR = "Visual Studio 17 2022"
   flutter build windows --debug
   ```
4. **临时禁用 Patch**：若 Flutter 已原生支持 `CMAKE_GENERATOR`，可从 CI 中移除 Patch 步骤，并更新本文档。

---

## 2. CI 环境要求

### 2.1 `rust` job 需要 Flutter 工具链

当前 `.github/workflows/ci.yml` 的 `rust` job 在 "Generate FRB bindings" 步骤中调用 `flutter_rust_bridge_codegen generate`，该工具需要定位 `flutter` 命令。若 runner 未安装 Flutter，此步骤会失败：

```text
Error: Dart/Flutter toolchain not available
The term 'flutter' is not recognized as the name of a cmdlet
```

**解决方案**（选择一个）：

- 在 `rust` job 中增加 `subosito/flutter-action@v2` 安装 Flutter 3.29.x。
- 或在 `rust` job 中仅安装 Dart SDK，并设置 `FLUTTER_ROOT` / `DART_SDK` 环境变量。
- 或将 FRB 生成步骤合并到 `flutter` job，让 `rust` job 不再依赖 Flutter。

详见 `docs/current/CI_FAILURES.md` 中 **CI-001**。

---

## 3. 本地常见错误

### 3.1 Gradle wrapper SSL 证书验证失败

**现象**：

```text
javax.net.ssl.SSLHandshakeException: PKIX path building failed
unable to find valid certification path to requested target
```

**根因**：本地 JDK 证书信任库缺少 Gradle 分发服务器的 CA 证书，或网络代理证书未被信任。

**解决**：

1. 检查 `JAVA_HOME` 指向的 JDK 是否为可信发行版。
2. 若在公司代理网络中，将代理根证书导入 JDK 的 `cacerts`。
3. 临时使用已下载的 Gradle 分发包，或修改 `gradle-wrapper.properties` 使用国内镜像（仅限个人环境，不要提交）。

详见 `docs/current/CI_FAILURES.md` 中 **CI-002**。

---

## 4. 相关文档

- `.github/workflows/ci.yml`：CI 中 Flutter Patch 的自动化实现
- `docs/current/CI_FAILURES.md`：CI 失败记录与修复方案
- `AGENTS.md`：项目构建命令速查
