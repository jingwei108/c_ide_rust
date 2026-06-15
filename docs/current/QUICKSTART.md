# Cide 快速入门指南

> [English Version](QUICKSTART_EN.md)

本指南帮助你在 10 分钟内跑通 Cide 的最常用路径：命令行调试与桌面端 IDE。

> 如果你是贡献者或需要深度构建配置，请参阅 [`BUILD.md`](BUILD.md) 和 [`FLUTTER_BUILD_MANUAL.md`](FLUTTER_BUILD_MANUAL.md)。

---

## 环境要求

| 工具 | 版本 | 用途 |
|:---|:---|:---|
| Rust | 1.95.0+ | 编译 Rust 后端 |
| Cargo | 随 Rust 安装 | Rust 包管理 |
| Flutter | 3.24+ | 桌面端 / Android 前端 |
| Python | 3.8+ | 运行构建脚本 |
| Visual Studio 2022+ | 含 C++ 桌面开发 | Windows 桌面构建 |

快速检查：

```bash
rustc --version        # >= 1.95.0
flutter doctor         # Windows 工具链无异常
```

---

## 一分钟上手：命令行工具（无需 Flutter）

`cide_cli` 是最快的体验方式，可以直接编译、运行、单步调试 C/C++ 代码。

### 1. 构建 CLI

```bash
cd native
cargo build --release --bin cide_cli
```

Windows 产物：`native/target/release/cide_cli.exe`

### 2. 直接运行代码片段

无需创建文件，使用 `-` 从标准输入读取代码：

```bash
# 管道方式
echo '#include <stdio.h>
int main() { printf("hello, cide\n"); return 0; }' | cargo run --release --bin cide_cli -- run -
```

或使用 here-document：

```bash
cargo run --release --bin cide_cli -- run - <<'EOF'
#include <stdio.h>
int main() {
    int a, b;
    scanf("%d %d", &a, &b);
    printf("sum = %d\n", a + b);
    return 0;
}
EOF
# 然后输入两个整数并按回车
```

### 3. 单步调试一个文件

```bash
cargo run --release --bin cide_cli -- step tests/cases/baseline/hello_world.c
```

进入交互后：

| 命令 | 说明 |
|:---|:---|
| `Enter` | 执行下一步 |
| `p` | 打印当前局部变量 |
| `o` | 打印当前程序输出 |
| `r` | 全速运行到结束 |
| `q` | 退出 |

> 完整 CLI 用法见 [`CIDE_CLI.md`](CIDE_CLI.md)。

---

## 五分钟上手：桌面端 IDE

### 1. 构建并运行

在项目根目录执行：

```bash
# 桌面端 Debug 构建并运行
python scripts/build_flutter.py --run
```

首次构建会：
1. 编译 Rust 后端为 `cide_native.dll`
2. 获取 Flutter 依赖
3. 构建 Windows 桌面端
4. 启动应用

### 2. 创建并运行你的第一个程序

启动 IDE 后：

1. 新建文件或打开示例
2. 输入 C/C++ 教学子集代码，例如：

```c
#include <stdio.h>

int main() {
    int n = 5;
    int sum = 0;
    for (int i = 1; i <= n; i++) {
        sum += i;
    }
    printf("sum = %d\n", sum);
    return 0;
}
```

3. 点击**运行**按钮，查看输出
4. 点击**单步调试**按钮，观察变量 `i`、`sum` 的变化
5. 在右侧**可视化面板**查看数组、指针或算法执行过程

---

## 常用命令速查

```bash
# 桌面端 Debug 构建（默认）
python scripts/build_flutter.py

# 桌面端 Release 构建并运行
python scripts/build_flutter.py -c Release --run

# Android APK 构建
python scripts/build_flutter.py -t Android

# 清理构建产物
python scripts/build_flutter.py --clean

# 离线构建
python scripts/build_flutter.py --offline

# 移动端完整流水线：构建 + 安装 + 启动 + 日志
python scripts/test_mobile.py --install --run --logcat
```

---

## 验证项目是否健康

```bash
# Rust 后端测试（719+ 用例）
cd native && cargo test

# C Shadow Verification（与 Clang 对比 stdout）
python native/tests/shadow_verification/shadow_verify.py

# C++ Shadow Verification
python scripts/shadow_verify_cpp.py
```

---

## 常见问题

### `cargo` 命令未找到

安装 Rust 并确保 `~/.cargo/bin` 在 PATH 中：

```bash
https://rustup.rs
```

### `flutter` 命令未找到

将 Flutter SDK 的 `bin` 目录加入 PATH：

```powershell
$env:PATH += ";D:\flutter\bin"
```

### 桌面端构建报错 "Unable to find suitable Visual Studio"

打开 Visual Studio Installer，安装 **"使用 C++ 的桌面开发"** 工作负载。

### Android 构建报错 "ANDROID_NDK_HOME not set"

```powershell
$env:ANDROID_NDK_HOME = "C:\Your\Path\To\ndk\27.1.12297006"
```

或安装 `cargo-ndk`：

```bash
cargo install cargo-ndk
```

### 运行时崩溃 "Failed to load dynamic library"

通常是 `cide_native.dll` 未复制到 Flutter 输出目录。使用构建脚本 `scripts/build_flutter.py` 可自动处理；手动模式见 [`FLUTTER_BUILD_MANUAL.md`](FLUTTER_BUILD_MANUAL.md)。

---

## 下一步

- 了解支持的 C 子集：[`C_SUBSET_SPEC.md`](C_SUBSET_SPEC.md)
- 了解支持的 C++ 子集：[`CPP_SUBSET_SPEC.md`](CPP_SUBSET_SPEC.md)
- 了解测试防线：[`SHADOW_VERIFICATION_FRAMEWORK.md`](SHADOW_VERIFICATION_FRAMEWORK.md)
- 了解架构设计：[`DESIGN.md`](DESIGN.md)
- 查看 CLI 完整手册：[`CIDE_CLI.md`](CIDE_CLI.md)
