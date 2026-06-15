# Cide Quick Start Guide

> [中文版](QUICKSTART.md)

This guide helps you get the most common Cide paths running within 10 minutes: command-line debugging and the desktop IDE.

> If you are a contributor or need in-depth build configuration, see [`BUILD_EN.md`](BUILD_EN.md) and [`FLUTTER_BUILD_MANUAL.md`](FLUTTER_BUILD_MANUAL.md).

---

## Requirements

| Tool | Version | Purpose |
|:-----|:--------|:--------|
| Rust | 1.95.0+ | Compile Rust backend |
| Cargo | Installed with Rust | Rust package management |
| Flutter | 3.24+ | Desktop / Android frontend |
| Python | 3.8+ | Run build scripts |
| Visual Studio 2022+ | With C++ desktop development | Windows desktop build |

Quick checks:

```bash
rustc --version        # >= 1.95.0
flutter doctor         # Windows toolchain OK
```

---

## Get Running in One Minute: Command-Line Tool (No Flutter Required)

`cide_cli` is the fastest way to experience Cide: compile, run, and step-debug C/C++ code directly.

### 1. Build the CLI

```bash
cd native
cargo build --release --bin cide_cli
```

Windows artifact: `native/target/release/cide_cli.exe`

### 2. Run a Code Snippet Directly

No file creation needed; use `-` to read source from standard input:

```bash
# Pipe mode
echo '#include <stdio.h>
int main() { printf("hello, cide\n"); return 0; }' | cargo run --release --bin cide_cli -- run -
```

Or use a here-document:

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
# Then type two integers and press Enter
```

### 3. Step-Debug a File

```bash
cargo run --release --bin cide_cli -- step tests/cases/baseline/hello_world.c
```

Interactive commands:

| Command | Description |
|:--------|:------------|
| `Enter` | Execute next step |
| `p` | Print current local variables |
| `o` | Print current program output |
| `r` | Run to completion |
| `q` | Quit |

> For full CLI usage, see [`CIDE_CLI_EN.md`](CIDE_CLI_EN.md).

---

## Get Running in Five Minutes: Desktop IDE

### 1. Build and Run

Run from the project root:

```bash
# Desktop Debug build and run
python scripts/build_flutter.py --run
```

The first build will:
1. Compile the Rust backend into `cide_native.dll`
2. Fetch Flutter dependencies
3. Build the Windows desktop app
4. Launch the app

### 2. Create and Run Your First Program

After the IDE launches:

1. Create a new file or open an example
2. Enter C/C++ teaching-subset code, for example:

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

3. Click the **Run** button to see output
4. Click the **Step** button to observe changes to variables `i` and `sum`
5. Use the **Visualization panel** on the right to view arrays, pointers, or algorithm execution

---

## Common Commands Cheat Sheet

```bash
# Desktop Debug build (default)
python scripts/build_flutter.py

# Desktop Release build and run
python scripts/build_flutter.py -c Release --run

# Android APK build
python scripts/build_flutter.py -t Android

# Clean build artifacts
python scripts/build_flutter.py --clean

# Offline build
python scripts/build_flutter.py --offline

# Mobile full pipeline: build + install + run + logs
python scripts/test_mobile.py --install --run --logcat
```

---

## Verify Project Health

```bash
# Rust backend tests (719+ cases)
cd native && cargo test

# C Shadow Verification (compare stdout with Clang)
python native/tests/shadow_verification/shadow_verify.py

# C++ Shadow Verification
python scripts/shadow_verify_cpp.py
```

---

## Frequently Asked Questions

### `cargo` command not found

Install Rust and make sure `~/.cargo/bin` is on PATH:

```bash
https://rustup.rs
```

### `flutter` command not found

Add the Flutter SDK `bin` directory to PATH:

```powershell
$env:PATH += ";D:\flutter\bin"
```

### Desktop build error: "Unable to find suitable Visual Studio"

Open Visual Studio Installer and install the **"Desktop development with C++"** workload.

### Android build error: "ANDROID_NDK_HOME not set"

```powershell
$env:ANDROID_NDK_HOME = "C:\Your\Path\To\ndk\27.1.12297006"
```

Or install `cargo-ndk`:

```bash
cargo install cargo-ndk
```

### Runtime crash: "Failed to load dynamic library"

Usually `cide_native.dll` was not copied to the Flutter output directory. The build script `scripts/build_flutter.py` handles this automatically; for manual mode see [`FLUTTER_BUILD_MANUAL.md`](FLUTTER_BUILD_MANUAL.md).

---

## Next Steps

- Learn the supported C subset: [`C_SUBSET_SPEC.md`](C_SUBSET_SPEC.md)
- Learn the supported C++ subset: [`CPP_SUBSET_SPEC.md`](CPP_SUBSET_SPEC.md)
- Learn about test defenses: [`SHADOW_VERIFICATION_FRAMEWORK.md`](SHADOW_VERIFICATION_FRAMEWORK.md)
- Learn about architecture: [`DESIGN.md`](DESIGN.md)
- Full CLI manual: [`CIDE_CLI_EN.md`](CIDE_CLI_EN.md)
