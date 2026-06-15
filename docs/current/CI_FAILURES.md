# CI 失败记录

> 记录原则：诚实记录 CI 运行中的失败，不粉饰数据。
> 覆盖范围：GitHub Actions、本地构建脚本、CI 相关工具链问题。

---

## 当前记录

### CI-001：GitHub Actions `rust` job — `Generate FRB bindings` 步骤找不到 Flutter 工具链

- **发现时间**：2026-06-15
- **涉及 workflow**：`.github/workflows/ci.yml` → `rust` job → `Generate FRB bindings`
- **失败现象**：
  ```
  Error: Dart/Flutter toolchain not available
  command="powershell" "-noprofile" "-command" "& flutter --version"
  The term 'flutter' is not recognized as the name of a cmdlet, function, script file, or operable program.
  ```
- **根因分析**：
  - `rust` job 未安装 Flutter，但 `flutter_rust_bridge_codegen generate` 需要调用 `flutter --version` 来检测 Dart/Flutter 工具链。
  - 尽管 `flutter_rust_bridge_codegen` 可能仅需要 Dart 部分，但它在启动时会尝试定位 Flutter SDK；在缺少 `flutter` 命令的 runner 上直接失败。
- **影响范围**：阻塞 `rust` job 后续所有步骤（Build native、Run Rust tests、Three Tier Check、Shadow Verification 等）。
- **是否代码缺陷**：否，是 CI 环境配置缺陷。
- **建议修复方案**：
  1. 在 `rust` job 的 "Install Rust" 步骤后增加 `subosito/flutter-action@v2` 安装 Flutter 3.29.x（与 `flutter` job 保持一致）。
  2. 或在 `rust` job 中仅安装 Dart SDK（若 FRB 不需要完整 Flutter），并设置 `FLUTTER_ROOT` / `DART_SDK` 环境变量。
  3. 评估是否可将 FRB 生成移至 `flutter` job，避免 `rust` job 依赖 Flutter。
- **状态**：待修复
- **关联任务**：`ROADMAP_2026_Q3.md` A3.1

---

### CI-002：本地/部分环境 Android 构建 — Gradle wrapper SSL 证书验证失败

- **发现时间**：2026-06-15
- **涉及脚本/命令**：`flutter build apk --debug`（也影响 `scripts/test_mobile.py`）
- **失败现象**：
  ```
  Exception in thread "main" javax.net.ssl.SSLHandshakeException:
  PKIX path building failed: sun.security.provider.certpath.SunCertPathBuilderException:
  unable to find valid certification path to requested target
  ```
- **根因分析**：
  - 当前 Java 运行时的证书信任库缺少 Gradle 分发服务器（`services.gradle.org`）的 CA 证书，或本地网络中间人证书未被信任。
  - 属于本地环境/网络问题，非项目代码问题。
- **影响范围**：影响新开发者首次 Android 构建；GitHub Actions 上若使用相同 JDK 版本也可能触发。
- **是否代码缺陷**：否。
- **建议修复方案**：
  1. 在 `FIRST_TIME_SETUP.md` 中增加常见排查项：检查 `JAVA_HOME` 对应 JDK 证书库、公司网络代理证书、尝试 `--no-daemon` 或预先下载 Gradle 分发包。
  2. 评估将 `gradle-wrapper.properties` 中 `distributionUrl` 改为企业镜像或国内镜像的可行性（需权衡可移植性）。
  3. 在 CI 中固定使用已知可用的 JDK 版本并缓存 Gradle wrapper。
- **状态**：待修复（文档化 + 环境兜底）
- **关联任务**：`ROADMAP_2026_Q3.md` A3.1、C4.1

---

## 记录规范

新增 CI 失败时按以下格式追加到本文件顶部（保持时间倒序）：

```markdown
### CI-XXX：标题
- **发现时间**：YYYY-MM-DD
- **涉及 workflow**：
- **失败现象**：
- **根因分析**：
- **影响范围**：
- **是否代码缺陷**：
- **建议修复方案**：
- **状态**：todo / in_progress / done
- **关联任务**：
```
