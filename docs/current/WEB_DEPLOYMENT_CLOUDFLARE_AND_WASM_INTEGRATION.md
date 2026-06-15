# Cide Web 部署方案 C：Cloudflare Pages + Rust WASM 引入方案

> 本文档说明如何将 Cide 的 Flutter Web 前端部署到 Cloudflare Pages，以及如何在 Web 环境中引入 Rust 后端 WASM。

---

## 一、方案 C：Cloudflare Pages 部署

Cloudflare Pages 是静态站点托管服务，全球 CDN 加速、自动 HTTPS、支持 Git 集成和 CLI 部署，适合 Cide Web 的对外发布。

### 1.1 前置条件

- Cloudflare 账号
- 已安装 Node.js（用于 Wrangler CLI）
- 项目已完成 Flutter Web + Rust WASM 构建（详见第二节）

### 1.2 方式一：Wrangler CLI 部署（推荐首次快速验证）

#### 步骤 1：安装 Wrangler

```bash
npm i -g wrangler
```

#### 步骤 2：登录 Cloudflare

```bash
wrangler login
```

浏览器会弹出授权页面，确认即可。

#### 步骤 3：构建 Flutter Web + WASM

```bash
cd CideFlutter
flutter build web --release
```

确保产物位于 `CideFlutter/build/web/`。

#### 步骤 4：部署

```bash
cd CideFlutter/build/web
wrangler pages deploy . --project-name=cide-web
```

首次部署会提示创建项目，选择后完成。

部署成功后，Wrangler 会输出访问地址，例如：

```text
https://cide-web.pages.dev
```

#### 后续更新

每次更新只需重新执行构建和部署：

```bash
cd CideFlutter
flutter build web --release
cd build/web
wrangler pages deploy . --project-name=cide-web
```

---

### 1.3 方式二：Git 集成部署（推荐长期维护）

#### 步骤 1：在 Cloudflare Dashboard 创建 Pages 项目

1. 登录 [Cloudflare Dashboard](https://dash.cloudflare.com/)
2. 进入 **Pages → Create a project → Connect to Git**
3. 选择 `c_ide_rust` 仓库
4. 开始配置

#### 步骤 2：配置构建设置

| 配置项 | 值 |
|---|---|
| Framework preset | None |
| Build command | `cd CideFlutter && flutter build web --release` |
| Build output directory | `/CideFlutter/build/web` |
| Root directory | `/` |

#### 步骤 3：设置环境变量

如果构建过程需要 Flutter 或 Rust 环境，Cloudflare 的构建镜像默认不包含它们。需要在 **Environment variables** 中配置，或使用自定义构建命令下载。

更简单的方式是：在仓库根目录放置一个构建脚本，让 Cloudflare 调用该脚本。

创建 `scripts/build_web_for_cloudflare.sh`：

```bash
#!/usr/bin/env bash
set -e

# 安装 Flutter
FLUTTER_VERSION="3.29.0"
curl -o flutter.tar.xz https://storage.googleapis.com/flutter_infra_release/releases/stable/linux/flutter_linux_${FLUTTER_VERSION}-stable.tar.xz
tar xf flutter.tar.xz
export PATH="$PWD/flutter/bin:$PATH"
flutter config --no-analytics
flutter doctor

# 安装 Rust
export RUSTUP_HOME="$PWD/rustup"
export CARGO_HOME="$PWD/cargo"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --target wasm32-unknown-unknown
export PATH="$CARGO_HOME/bin:$PATH"

# 安装 wasm-pack 和 flutter_rust_bridge_codegen
cargo install wasm-pack
cargo install flutter_rust_bridge_codegen --version 2.12.0

# 生成 FRB 绑定
cd CideFlutter
flutter_rust_bridge_codegen generate

# 构建 Rust WASM
cd ../native
wasm-pack build --target web --out-dir ../CideFlutter/web/pkg

# 构建 Flutter Web
cd ../CideFlutter
flutter build web --release
```

然后 Build command 改为：

```bash
bash scripts/build_web_for_cloudflare.sh
```

> 注意：此脚本每次构建都下载 Flutter 和 Rust，耗时较长。建议搭配 Cloudflare Pages 的构建缓存或改用 GitHub Actions 构建后上传。

---

### 1.4 方式三：Dashboard 手动上传

适合临时演示，不适合持续集成。

1. 在本地执行 `flutter build web --release`
2. 将 `CideFlutter/build/web/` 压缩为 zip
3. Cloudflare Dashboard → Pages → Create a project → Direct Upload
4. 上传 zip 文件
5. 部署完成

---

### 1.5 Cloudflare Pages 特有配置

在 `CideFlutter/build/web/` 目录中放置以下文件，Flutter 构建时不会覆盖它们（只要它们原本就在 `web/` 源目录中）。

#### `_headers`：设置 MIME 类型和缓存策略

```text
/*.wasm
  Content-Type: application/wasm
  Cache-Control: public, max-age=31536000, immutable

/*.js
  Cache-Control: public, max-age=31536000, immutable

/assets/templates/*
  Cache-Control: public, max-age=86400
```

#### `_redirects`：单页应用路由回退

```text
/* /index.html 200
```

> 这确保用户直接访问 `https://cide-web.pages.dev/editor` 时不会 404。

#### `_routes.json`：更精确的路由控制（可选）

```json
{
  "version": 1,
  "include": ["/*"],
  "exclude": ["/assets/*"]
}
```

---

### 1.6 绑定自定义域名

1. Cloudflare Dashboard → Pages → cide-web → Custom domains
2. 输入域名，例如 `cide.example.com`
3. 按提示添加 DNS 记录
4. Cloudflare 自动颁发 HTTPS 证书

---

## 二、Web 引入方案：Rust WASM 如何集成到 Flutter Web

Cide 的 `rust_builder` 当前只配置了 `android / ios / linux / macos / windows` 平台，没有 Web 平台。因此 Web 版本需要手动将 Rust 后端编译为 WASM 并引入。

---

### 2.1 方案一：手动 wasm-pack + 静态资源引入（当前推荐）

这是与现有项目结构最兼容的方案。

#### 步骤 1：安装工具链

```bash
# Rust WASM target
rustup target add wasm32-unknown-unknown

# wasm-pack
cargo install wasm-pack
```

#### 步骤 2：构建 Rust WASM

```bash
cd native
wasm-pack build --target web --out-dir ../CideFlutter/web/pkg
```

产物位于 `CideFlutter/web/pkg/`：

```text
pkg/
  ├─ cide_native.js       # JS glue，暴露 init 函数
  ├─ cide_native_bg.wasm  # WASM 二进制
  └─ package.json
```

#### 步骤 3：在 `web/index.html` 中加载 WASM

修改 `CideFlutter/web/index.html`，在 `<head>` 中添加：

```html
<script type="module">
  import init from './pkg/cide_native.js';
  window.cideWasmInit = init;
</script>
```

> 注意：`window.cideWasmInit` 是为了让 FRB 的 Web 绑定能够找到初始化入口。实际是否需要暴露取决于 FRB 版本和集成方式。

#### 步骤 4：配置 FRB Web 绑定

`flutter_rust_bridge` v2.12.0 已经生成了 `frb_generated.web.dart`。默认情况下，`RustLib.init()` 在 Web 平台会尝试加载 WASM。

需要确保 FRB 知道 WASM 文件的位置。常见做法是在 `web/index.html` 中通过全局变量传递：

```html
<script>
  window.FRB_WASM_MODULE_URL = './pkg/cide_native_bg.wasm';
</script>
```

#### 步骤 5：Flutter 构建

```bash
cd CideFlutter
flutter build web --release
```

构建时，`web/pkg/` 目录会被复制到 `build/web/pkg/`。

#### 优点

- 不依赖 cargokit，立刻可用
- 构建流程清晰可控

#### 缺点

- 每次 Rust 代码修改后需要手动重新执行 `wasm-pack build`
- 需要维护 `web/index.html` 的加载脚本

---

### 2.2 方案二：扩展 rust_builder 支持 Web（长期推荐）

通过扩展 `CideFlutter/rust_builder/`，让 `flutter build web` 自动调用 `wasm-pack`。

#### 需要修改的文件

1. `CideFlutter/rust_builder/pubspec.yaml`

   添加 `web` 平台声明：

   ```yaml
   flutter:
     plugin:
       platforms:
         android:
           ffiPlugin: true
         ios:
           ffiPlugin: true
         linux:
           ffiPlugin: true
         macos:
           ffiPlugin: true
         windows:
           ffiPlugin: true
         web:
   ```

2. 创建 `CideFlutter/rust_builder/web/`

   参考 [cargokit](https://github.com/irondash/cargokit) 文档，添加 Web 构建入口。通常需要：

   - `cide_native_web.podspec` 或等效文件
   - 调用 `wasm-pack build` 的构建脚本

3. 修改 `CideFlutter/pubspec.yaml`

   当前已引用 `cide_native` rust_builder，扩展后无需改动。

4. 修改 `CideFlutter/web/index.html`

   确保加载生成的 WASM 产物。

#### 优点

- 与现有构建流程一致，`flutter build web` 一键完成
- 减少手动步骤

#### 缺点

- 需要深入理解 cargokit 的构建机制
- 开发和调试周期较长

---

### 2.3 方案三：纯 Web 前端 + Vite/Webpack 桥接（未来可选）

如果未来决定不用 Flutter Web，而是改为 React/Vue + Monaco Editor，可以这样集成 Rust：

#### Vite 示例

```bash
cd native
wasm-pack build --target web --out-dir ../web-frontend/pkg
```

在 Vite 项目中：

```typescript
import init, { compile } from '../pkg/cide_native.js';

await init();
const result = compile('int main() { return 0; }');
```

#### 配置 `vite.config.ts`

```typescript
import { defineConfig } from 'vite';

export default defineConfig({
  server: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
});
```

> 目前 Cide 前端是 Flutter，此方案仅作为未来架构演进的参考。

---

## 三、推荐组合

| 阶段 | 部署平台 | Web 引入方案 | 理由 |
|---|---|---|---|
| 快速验证 | Cloudflare Pages CLI | 手动 wasm-pack | 最快看到效果 |
| 长期维护 | Cloudflare Pages Git 集成 | 扩展 rust_builder | 自动化程度高 |
| 中国大陆 | Cloudflare Pages + 国内 CDN | 手动 wasm-pack | 兼顾速度和成本 |
| 私有化 | Nginx / IIS 自建 | 手动 wasm-pack | 完全可控 |

---

## 四、部署后常见问题

### 4.1 WASM 加载失败

检查浏览器开发者工具 Network 标签：

- `.wasm` 文件请求是否 200
- Response Headers 中 `Content-Type` 是否为 `application/wasm`
- 路径是否正确（base href 设置是否匹配部署路径）

### 4.2 模板资源 404

确保 `CideFlutter/pubspec.yaml` 中的 assets 已包含：

```yaml
flutter:
  assets:
    - assets/templates/
    - assets/templates/index.json
```

并且构建后 `build/web/assets/` 下存在这些文件。

### 4.3 单页路由 404

直接刷新非根路径时如果 404，说明服务器没有回退到 `index.html`。需要配置：

- Cloudflare Pages：添加 `_redirects` 文件
- Nginx：`try_files $uri $uri/ /index.html;`
- IIS：配置 URL Rewrite 规则

### 4.4 首次加载慢

- 开启 gzip/brotli 压缩
- 对 `.wasm`、`.js` 设置长期缓存
- 考虑使用 Flutter 的 `--wasm` 渲染器减少 CanvasKit 下载体积

---

## 五、附录：完整本地验证命令

```bash
# 1. 构建 Rust WASM
cd native
wasm-pack build --target web --out-dir ../CideFlutter/web/pkg

# 2. 构建 Flutter Web
cd ../CideFlutter
flutter build web --release

# 3. 本地预览（需要 Python）
cd build/web
python -m http.server 8080

# 4. 浏览器打开 http://localhost:8080
```

如果本地预览正常，再部署到 Cloudflare Pages。
