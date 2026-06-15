# Cide Web 部署方案 D：GitHub Pages + Gitee Pages 双部署

> 适用于：国内用户为主、零预算、计划 MIT 开源、不追求快速上线的项目阶段。
>
> 核心策略：**GitHub Pages 作为主站（海外/CI 原生），Gitee Pages 作为国内镜像。**

---

## 一、方案概览

```text
GitHub 公开仓库（MIT）
   └── GitHub Actions 构建 Flutter Web + Rust WASM
        ├── 产物 → GitHub Pages（https://你的用户名.github.io/c_ide_rust）
        └── 仓库同步 → Gitee Pages（https://你的用户名.gitee.io/c_ide_rust）
```

### 为什么选这个组合

| 维度 | GitHub Pages | Gitee Pages | 组合效果 |
|---|---|---|---|
| 费用 | 公开仓库免费 | 公开仓库免费 | 零预算 |
| 国内访问 | 一般/较慢 | 快 | 国内用户走 Gitee |
| 海外访问 | 快 | 一般 | 海外用户走 GitHub |
| CI 集成 | 原生支持 GitHub Actions | 需仓库同步后触发 | GitHub Actions 一次构建，双平台受益 |
| 源码公开 | 开源后自然公开 | 需手动导入/同步 | 与 MIT 开源策略一致 |

---

## 二、前置条件

### 2.1 项目准备

在部署之前，必须先完成以下本地验证：

```bash
# 1. 安装 WASM target
rustup target add wasm32-unknown-unknown

# 2. 安装 wasm-pack
cargo install wasm-pack

# 3. 构建 Rust WASM
# --no-opt 跳过 binaryen 优化，避免 CI/本地首次构建时从 GitHub 下载 binaryen。
cd native
wasm-pack build --target web --out-dir ../CideFlutter/web/pkg --no-opt

# 4. 构建 Flutter Web
cd ../CideFlutter
flutter build web --release

# 5. 本地预览
cd build/web
python -m http.server 8080
```

浏览器访问 `http://localhost:8080`，确认功能正常。

### 2.2 Rust 代码 WASM 兼容性改造 ✅ 已完成

在 `wasm32-unknown-unknown` 目标下，以下 API 已通过 `#[cfg(target_arch = "wasm32")]` / `#[cfg(not(target_arch = "wasm32"))]` 完成条件编译处理；桌面端与 Android 端保持原有逻辑不变。

#### 线程相关

文件：`native/src/flutter_bridge.rs`

`run_auto_steps_stream` 在桌面端继续使用 `std::thread::spawn`，在 Web 端改用 `wasm_bindgen_futures::spawn_local` 执行单线程事件循环：

```rust
pub fn run_auto_steps_stream(
    sink: crate::frb_generated::StreamSink<crate::unified::stream::StepStreamBatch>,
    batch_size: i32,
) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::spawn(move || {
            run_auto_steps_stream_loop(sink, batch_size);
        });
    }

    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(async move {
            run_auto_steps_stream_loop(sink, batch_size);
        });
    }
}
```

#### 时间相关

文件：`native/src/vm/host_funcs.rs`

`host_time` / `host_clock` 已统一使用 `current_time_millis()` 辅助函数；Web 端通过 `js_sys::Date::now()` 获取时间戳：

```rust
#[cfg(not(target_arch = "wasm32"))]
fn current_time_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(target_arch = "wasm32")]
fn current_time_millis() -> u64 {
    js_sys::Date::now() as u64
}
```

#### 异常捕获

文件：`native/src/engine/session_ops.rs`

`wasm32-unknown-unknown` 不支持 `std::panic::catch_unwind`，因此 `execute_run` 在 Web 端直接执行 VM 运行逻辑，桌面端仍保留 catch_unwind 保护。

#### 依赖

`native/Cargo.toml` 已添加 Web 专属依赖：

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
js-sys = "0.3"
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
```

> 具体行号可能随代码变化，请以实际代码为准。

### 2.3 创建 Web 入口

确保 `CideFlutter/web/index.html` 存在，并注入 WASM 加载脚本。模板见 `CideFlutter/web/index.html`。

---

## 三、目录与文件结构

```text
c_ide_rust/
├── .github/
│   └── workflows/
│       └── deploy_web.yml          # GitHub Actions 构建+部署
├── CideFlutter/
│   ├── web/
│   │   └── index.html              # WASM 加载入口
│   └── build/web/                  # 构建产物（.gitignore）
├── native/
│   └── Cargo.toml                  # 添加 wasm32 依赖
├── scripts/
│   └── build_web.sh                # 本地构建脚本
└── docs/current/
    └── WEB_DEPLOYMENT_GITHUB_AND_GITEE_PAGES.md  # 本文档
```

---

## 四、GitHub Actions 工作流

详见 `.github/workflows/deploy_web.yml`。

主要流程：

1. 检出代码
2. 安装 Flutter
3. 安装 Rust + wasm32 target
4. 安装 wasm-pack
5. 构建 Rust WASM 到 `CideFlutter/web/pkg/`
6. 构建 Flutter Web
7. 生成 SPA 回退页：复制 `index.html` → `404.html`
8. 校验产物：`index.html`、`404.html`、`pkg/cide_native_bg.wasm`、`pkg/cide_native.js`
9. 部署到 GitHub Pages
10. （可选）若配置了 `GITEE_PRIVATE_KEY`，自动同步到 Gitee

---

## 五、Gitee Pages 镜像配置

### 5.1 创建 Gitee 仓库

1. 注册/登录 [Gitee](https://gitee.com)
2. 创建仓库 `c_ide_rust`
3. 选择 **公开仓库**
4. 完成实名认证（Gitee Pages 强制要求）

### 5.2 自动同步 GitHub → Gitee

`.github/workflows/deploy_web.yml` 已内置可选的 Gitee 同步任务：

- 当仓库 **未配置** `GITEE_PRIVATE_KEY` 时，同步任务自动跳过，不影响 GitHub Pages 部署。
- 当仓库 **已配置** `GITEE_PRIVATE_KEY` 时，每次部署完成后自动镜像到 Gitee。

配置步骤：

1. 在 GitHub 仓库 **Settings → Secrets → Actions** 中添加 `GITEE_PRIVATE_KEY`（Gitee 账号的 SSH 私钥）。
2. 确保 Gitee 仓库名为 `c_ide_rust`，且与 GitHub 仓库 `owner` 一致；若不一致，请手动修改 workflow 中的 `destination-repo`。
3. 将对应的公钥添加到 Gitee 账号的 **SSH 公钥** 中。

```yaml
# deploy_web.yml 中相关片段（已启用）
sync-to-gitee:
  runs-on: ubuntu-latest
  needs: check-gitee-secret
  if: needs.check-gitee-secret.outputs.has_key == 'true'
  steps:
    - name: Sync to Gitee
      uses: wearerequired/git-mirror-action@v1
      env:
        SSH_PRIVATE_KEY: ${{ secrets.GITEE_PRIVATE_KEY }}
      with:
        source-repo: git@github.com:${{ github.repository }}.git
        destination-repo: git@gitee.com:${{ github.repository_owner }}/c_ide_rust.git
```

### 5.3 开启 Gitee Pages

1. 进入 Gitee 仓库 → **服务 → Gitee Pages**
2. 选择部署分支（默认 `master` 或 `main`）
3. 选择部署目录：`CideFlutter/build/web`
4. 点击 **启动**

> 注意：Gitee Pages 部署目录不支持选择子目录为 build 输出，因此需要把构建产物推送到仓库根目录或配置特殊分支。实际操作中，建议：
> - 方案 A：使用 `gh-pages` 分支专门存放构建产物，Gitee Pages 也指向该分支。
> - 方案 B：在 Gitee 仓库单独配置一个 `gitee-pages` 分支，仅含 `index.html` 和 `assets/`。

### 5.4 验证 Gitee Pages 对 WASM 的支持

部署后，检查浏览器 Network 面板：

- `.wasm` 文件请求是否 200
- Response Headers 中 `Content-Type` 是否为 `application/wasm`

如果 Gitee Pages 返回 `application/octet-stream` 或其他类型，浏览器可能无法编译 WASM，需要联系 Gitee 支持或改用其他国内方案。

---

## 六、本地快速构建脚本

使用 `scripts/build_web.sh` 可以在本地一键构建，便于调试：

```bash
bash scripts/build_web.sh
```

构建完成后产物在 `CideFlutter/build/web/`，包含：

- `index.html` / `404.html`：主入口与 SPA 回退页
- `pkg/`：Rust WASM 输出（`cide_native_bg.wasm`、`cide_native.js` 等）
- `assets/`：模板资源、字体、图片等

本地预览：

```bash
cd CideFlutter/build/web
python -m http.server 8080
```

浏览器访问 `http://localhost:8080` 即可。

> 注意：本地构建默认使用 `<base href="/">`，线上 GitHub Pages 项目站点使用 `<base href="/c_ide_rust/">`，由 `deploy_web.yml` 中的 `--base-href` 参数控制。

Windows 用户可以使用 Git Bash 或 WSL 执行。

---

## 七、部署后验证清单

```markdown
□ GitHub Pages 能正常访问 https://你的用户名.github.io/c_ide_rust
□ Gitee Pages 能正常访问 https://你的用户名.gitee.io/c_ide_rust
□ 浏览器控制台无 WASM 加载错误
□ .wasm 文件 Content-Type 为 application/wasm
□ 模板资源 assets/templates/ 能正常加载
□ build/web 目录包含 404.html（SPA 回退）
□ 单页路由刷新不 404（GitHub/Gitee Pages 会返回 404.html）
□ 编译/运行/单步调试核心功能正常
```

---

## 八、常见问题

### Q1：为什么不用 Cloudflare Pages？

Cloudflare Pages 免费版在国内访问质量不稳定，对国内用户不友好。如果未来用户扩展到海外，可以再增加 Cloudflare Pages 作为第三镜像。

### Q2：GitHub Pages 有流量限制吗？

GitHub Pages 对公开仓库有每月 100GB 带宽限制，对教学/个人项目足够。超出后可考虑迁移到 CDN。

### Q3：Gitee Pages 支持自定义域名吗？

支持，但需要域名备案。无预算阶段可以先使用 `*.gitee.io` 二级域名。

### Q4：WASM 文件太大怎么办？

- 使用 `flutter build web --release` 已启用压缩
- 开启 GitHub Pages / Gitee Pages 的 gzip/brotli（平台自动处理）
- 未来可考虑 `--wasm` 渲染器减少 CanvasKit 体积

---

## 九、后续演进路线

```text
阶段 1（当前）：GitHub Pages + Gitee Pages 双部署
阶段 2（有预算后）：国内对象存储 + CDN，绑定备案域名
阶段 3（国际化后）：Cloudflare Pages 作为海外镜像
阶段 4（性能优化）：Flutter --wasm 渲染器、WASM 代码分割
```

---

## 十、相关文件

- 构建脚本：`scripts/build_web.sh`
- CI 工作流：`.github/workflows/deploy_web.yml`
- Web 入口：`CideFlutter/web/index.html`
- 原始 Cloudflare 方案：`docs/current/WEB_DEPLOYMENT_CLOUDFLARE_AND_WASM_INTEGRATION.md`
