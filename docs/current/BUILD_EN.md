# C IDE Build Guide

> [中文版](BUILD.md)

This document describes the project's build flow, script usage, and environment requirements.

> **Migration note**: The frontend has migrated from .NET MAUI to Flutter. Legacy MAUI build docs: [`docs/archive/ARCHIVE_MAUI_BUILD_SCRIPTS.md`](../archive/ARCHIVE_MAUI_BUILD_SCRIPTS.md) (archived).

---

## Requirements

| Component | Version | Purpose |
|:----------|:--------|:--------|
| Rust | 1.95.0+ | Native backend (`cide_native`) |
| Cargo | Installed with Rust | Rust package management |
| cargo-ndk | Latest | Android `.so` cross-compilation |
| Flutter SDK | 3.24+ | Cross-platform frontend |
| Android NDK | 27+ | Android Native backend cross-compilation (optional) |
| adb | Installed with Android SDK | Android device install/debug (optional) |

### Install cargo-ndk

```powershell
cargo install cargo-ndk
```

### Android NDK Environment Variables

```powershell
# Temporary (current session)
$env:ANDROID_NDK_HOME = "C:\Your\Path\To\ndk\27.0.1"

# Permanent
[Environment]::SetEnvironmentVariable("ANDROID_NDK_HOME", "C:\Your\Path\To\ndk\27.0.1", "User")
```

---

## Script List

| Script | Function | Scenario |
|:-------|:---------|:---------|
| [`scripts/build_flutter.py`](../../scripts/build_flutter.py) | Build Native backend + Flutter frontend | Daily development builds |
| [`scripts/build_release.py`](../../scripts/build_release.py) | Release build (Desktop + Android) | Release packaging |
| [`scripts/test_mobile.py`](../../scripts/test_mobile.py) | Mobile full pipeline: build → install → launch → logs | Flutter Android device/emulator testing |

> Legacy MAUI build scripts are archived at [`docs/archive/ARCHIVE_MAUI_BUILD_SCRIPTS.md`](../archive/ARCHIVE_MAUI_BUILD_SCRIPTS.md).

---

## `scripts/build_flutter.py` — Daily Build

### Function

1. **Native backend (Rust)**: `cargo build [--release]` compiles `cide_native.dll` / `.so`
2. **Desktop frontend (Flutter Windows)**: `flutter build windows` + auto DLL copy
3. **Mobile frontend (Flutter Android)**: `flutter build apk` (auto integrates `.so`)
4. **FRB code generation**: runs `flutter_rust_bridge_codegen generate` when necessary

### Parameters

| Parameter | Type | Default | Description |
|:----------|:-----|:--------|:------------|
| `-c`, `--configuration` | `Debug` / `Release` | `Debug` | Build configuration |
| `-t`, `--target` | `Desktop` / `Android` / `All` | `Desktop` | Target platform |
| `--clean` | flag | off | Clean all build artifacts |
| `--run` | flag | off | Run desktop app after build (Desktop only) |
| `--offline` | flag | off | Offline build (do not download pub dependencies) |

### Examples

```bash
# Desktop Debug build (default)
python scripts/build_flutter.py

# Desktop Release build, then run
python scripts/build_flutter.py -c Release --run

# Clean and rebuild desktop
python scripts/build_flutter.py --clean -t Desktop

# Android full build (NDK .so + APK)
python scripts/build_flutter.py -t Android

# Offline build (no network)
python scripts/build_flutter.py --offline
```

---

## `scripts/build_release.py` — Release Build

### Function

- **Desktop**: Rust Release + Flutter Windows Release
- **Android**: Rust Release NDK cross-compile + Flutter APK Release

### Parameters

| Parameter | Type | Default | Description |
|:----------|:-----|:--------|:------------|
| `-t`, `--target` | `Desktop` / `Android` / `All` | `All` | Target platform |
| `--clean` | flag | off | Clean all build artifacts |

### Examples

```bash
# Build Desktop and Android Release
python scripts/build_release.py

# Desktop only
python scripts/build_release.py -t Desktop

# Clean and build
python scripts/build_release.py --clean
```

---

## `scripts/test_mobile.py` — Mobile Test Pipeline

### Function

Focused on **Flutter Android** device/emulator quick test loop:

```
Native .so compile → Flutter APK package → device install → app launch → Logcat log capture
```

### Parameters

| Parameter | Type | Default | Description |
|:----------|:-----|:--------|:------------|
| `-c`, `--configuration` | `Debug` / `Release` | `Debug` | Build configuration |
| `--skip-native-build` | flag | off | Skip NDK `.so` compile, repackage APK only |
| `--install` | flag | off | Auto install APK after build |
| `--run` | flag | off | Auto launch app after install |
| `--logcat` | flag | off | Capture app logs in real time after launch (`Ctrl+C` to stop) |

### Examples

```bash
# Build APK only (with Native .so)
python scripts/test_mobile.py

# Quick repackage (after frontend code changes, skip .so compile)
python scripts/test_mobile.py --skip-native-build --install --run

# Build + install + launch + real-time logs (full pipeline)
python scripts/test_mobile.py --install --run --logcat

# Release build and install
python scripts/test_mobile.py -c Release --install --run
```

### Auto Detection

| Component | Detection logic |
|:----------|:----------------|
| Android NDK | First check `ANDROID_NDK_HOME` / `ANDROID_NDK_ROOT`, then probe VS default path |
| adb | First check `adb` in PATH, then probe VS Android SDK `platform-tools` |

VS default probe paths:
```
D:\Program Files (x86)\Microsoft Visual Studio\Shared\Android\AndroidNDK\android-ndk-r27c
D:\Program Files (x86)\Microsoft Visual Studio\Shared\Android\android-sdk\platform-tools\adb.exe
```

### Device Connection Stability

The script has a **3-time auto-retry** mechanism:
1. Detects `offline` device → auto `adb kill-server` / `start-server` → retry
2. No device detected → wait 3 seconds → retry
3. Third failure → error exit

---

## Manual Build

If scripts cannot be used due to environment issues, run manually:

### Desktop

```powershell
# 1. Native backend (Rust)
cd native
cargo build --release          # Release
cargo build                    # Debug

# DLL output paths
# Release: native/target/release/cide_native.dll
# Debug:   native/target/debug/cide_native.dll

# 2. Copy DLL to Flutter project
Copy-Item native/target/release/cide_native.dll CideFlutter/rust_builder/windows/ -Force

# 3. Frontend (Flutter)
cd CideFlutter
flutter pub get --offline
flutter build windows --debug

# 4. Run
flutter run -d windows
```

### Android

```powershell
# 1. Native backend (Rust NDK cross-compile)
cd native

# arm64-v8a
cargo ndk -t aarch64-linux-android -o target/android build --release

# armeabi-v7a
cargo ndk -t armv7-linux-androideabi -o target/android build --release

# .so output paths
# native/target/android/arm64-v8a/libcide_native.so
# native/target/android/armeabi-v7a/libcide_native.so

# 2. Frontend (Flutter)
cd CideFlutter
flutter pub get --offline
flutter build apk --release

# 3. Install and launch
adb install -r "build/app/outputs/flutter-apk/app-release.apk"
adb shell monkey -p com.cide.app -c android.intent.category.LAUNCHER 1

# 4. View logs
adb logcat --pid=$(adb shell pidof com.cide.app)
```

### Run Tests

```powershell
# Rust backend tests
cd native
cargo test
cargo clippy

# Flutter frontend tests
cd CideFlutter
flutter test
```

---

## FAQ

### Q1: `cargo ndk` command not found

```powershell
cargo install cargo-ndk
```

### Q2: Android build error "ANDROID_NDK_HOME not set"

```powershell
$env:ANDROID_NDK_HOME = "C:\Your\Path\To\ndk\27.0.1"
```

Or add the "Android development" workload through Visual Studio Installer; the script will auto-probe the default path.

### Q3: `adb devices` cannot detect device

```powershell
adb devices
```

| Output | Status | Fix |
|:-------|:-------|:----|
| `xxxxxxxx    device` | ✅ Normal | Run scripts directly |
| `xxxxxxxx    offline` | ⚠️ Offline | `adb kill-server && adb start-server`, keep phone screen on |
| `xxxxxxxx    unauthorized` | ❌ Unauthorized | Tap "Allow USB debugging" on phone screen |
| Blank | ❌ Unrecognized | Change data cable, USB port, enable developer options and USB debugging |

### Q4: APK install prompt "Install apps from unknown sources blocked"

Manufacturer settings paths:
- **Xiaomi/Redmi**: Settings → Privacy → Special permissions → Install unknown apps
- **Huawei/Honor**: Settings → Security → More security settings → External source apps download
- **OPPO/OnePlus/realme**: Settings → Password & Security → System security → External source apps
- **vivo/iQOO**: Settings → Security & Privacy → More security settings → Install unknown apps

### Q5: Flutter build error "Unable to find suitable Visual Studio"

Ensure "Desktop development with C++" workload is installed, or set:
```powershell
$env:FLUTTER_ROOT = "C:\Your\Path\To\flutter"
```
