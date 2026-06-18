#!/usr/bin/env python3
"""
Cide Flutter Build Script
Builds the Flutter frontend with Rust backend integration.

Supports both manual Rust builds (for environments without Developer Mode)
and automatic builds via cargokit.

Usage:
    python scripts/build_flutter.py                    # Desktop Debug build
    python scripts/build_flutter.py -c Release         # Desktop Release build
    python scripts/build_flutter.py -t Android         # Android APK build
    python scripts/build_flutter.py -t All             # Build Desktop + Android
    python scripts/build_flutter.py --clean            # Clean build artifacts
    python scripts/build_flutter.py --run              # Build and run desktop app
    python scripts/build_flutter.py --offline          # Use offline pub get
    python scripts/build_flutter.py --skip-rust        # Skip Rust build (cargokit handles it)
"""

import argparse
import shutil
import subprocess
import sys
from pathlib import Path

from build_utils import (
    error,
    find_flutter,
    header,
    info,
    run,
    success,
    warn,
    get_project_root,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build the Cide Flutter frontend with Rust backend."
    )
    parser.add_argument(
        "-c",
        "--configuration",
        choices=["Debug", "Release"],
        default="Debug",
        help="Build configuration (default: Debug)",
    )
    parser.add_argument(
        "-t",
        "--target",
        choices=["Desktop", "Android", "All"],
        default="Desktop",
        help="Build target platform (default: Desktop)",
    )
    parser.add_argument(
        "--clean", action="store_true", help="Clean Flutter build artifacts"
    )
    parser.add_argument(
        "--run",
        action="store_true",
        help="Build and run desktop app (only for Desktop target)",
    )
    parser.add_argument(
        "--offline",
        action="store_true",
        help="Run flutter pub get --offline (for air-gapped environments)",
    )
    parser.add_argument(
        "--skip-rust",
        action="store_true",
        help="Skip manual Rust build (let cargokit handle it via flutter build)",
    )
    parser.add_argument(
        "--test",
        action="store_true",
        help="Run cargo test and clippy before build",
    )
    return parser.parse_args()


def clean_build(root: Path) -> None:
    header("Cleaning Flutter build artifacts")
    dirs = [
        root / "CideFlutter" / "build",
        root / "CideFlutter" / ".dart_tool",
    ]
    for d in dirs:
        if d.exists():
            shutil.rmtree(d)
            print(f"Removed {d}")


def build_rust_desktop(root: Path, configuration: str) -> Path:
    """手动构建 Rust Desktop DLL（适用于未启用 Windows 开发者模式的环境）。"""
    header("Building Rust Backend (Desktop)")
    native_dir = root / "native"
    cargo_args = ["cargo", "build"]
    if configuration == "Release":
        cargo_args.append("--release")
    run(cargo_args, cwd=native_dir)

    profile = "release" if configuration == "Release" else "debug"
    dll_source = native_dir / "target" / profile / "cide_native.dll"

    if not dll_source.exists():
        raise FileNotFoundError(f"Rust build succeeded but DLL not found: {dll_source}")

    success(f"Rust build OK: {dll_source}")
    return dll_source


def copy_dll_to_flutter_build(
    root: Path, dll_source: Path, configuration: str
) -> None:
    """将 DLL 复制到 Flutter Windows 构建目录，供运行时加载。"""
    profile_dir = "Release" if configuration == "Release" else "Debug"
    flutter_build_dir = (
        root / "CideFlutter" / "build" / "windows" / "x64" / "runner" / profile_dir
    )
    flutter_build_dir.mkdir(parents=True, exist_ok=True)
    shutil.copy2(dll_source, flutter_build_dir / "cide_native.dll")
    success(f"Copied cide_native.dll -> {flutter_build_dir}")


def build_flutter_desktop(
    root: Path, configuration: str, run_app: bool, skip_rust: bool
) -> None:
    dll_source: Path | None = None

    if not skip_rust:
        dll_source = build_rust_desktop(root, configuration)

    header("Building Flutter Desktop (Windows)")
    flutter_dir = root / "CideFlutter"
    flutter_exe = find_flutter()

    flutter_args = [flutter_exe, "build", "windows"]
    if configuration == "Release":
        flutter_args.append("--release")
        flutter_args.extend(["--obfuscate", "--split-debug-info=symbols/"])
    else:
        flutter_args.append("--debug")

    run(flutter_args, cwd=flutter_dir)
    success("Flutter Windows build completed")

    # 如果手动构建了 Rust，在 flutter build 完成后再次复制 DLL
    # 因为 flutter build 可能会清理输出目录
    if dll_source and dll_source.exists():
        copy_dll_to_flutter_build(root, dll_source, configuration)

    if run_app:
        header("Running Flutter Desktop App")
        # 如果手动复制了 DLL，flutter run 前确保它在正确的位置
        if dll_source and dll_source.exists():
            copy_dll_to_flutter_build(root, dll_source, configuration)
        run([flutter_exe, "run", "-d", "windows"], cwd=flutter_dir, check=False)


def build_flutter_android(
    root: Path, configuration: str, skip_rust: bool
) -> None:
    if not skip_rust:
        build_rust_android(root, configuration)

    header("Building Flutter Android APK")
    flutter_dir = root / "CideFlutter"
    flutter_exe = find_flutter()

    flutter_args = [flutter_exe, "build", "apk"]
    if configuration == "Release":
        flutter_args.append("--release")
        flutter_args.extend(["--obfuscate", "--split-debug-info=symbols/"])
    else:
        flutter_args.append("--debug")

    run(flutter_args, cwd=flutter_dir)
    success("Flutter Android build completed")

    # 提示 APK 输出路径
    apk_dir = flutter_dir / "build" / "app" / "outputs" / "flutter-apk"
    if apk_dir.exists():
        apks = list(apk_dir.glob("*.apk"))
        if apks:
            info(f"APK output: {apks[0]}")


def main() -> int:
    args = parse_args()
    root = get_project_root()

    try:
        if args.clean:
            clean_build(root)

        if args.test:
            header("Running Rust tests and lints")
            native_dir = root / "native"
            print("Running cargo test --all-features...")
            run(["cargo", "test", "--all-features"], cwd=native_dir)
            success("cargo test passed")
            print("Running cargo clippy --all-targets -- -D warnings...")
            run(["cargo", "clippy", "--all-targets", "--", "-D", "warnings"], cwd=native_dir)
            success("cargo clippy passed")

        flutter_exe = find_flutter()
        success(f"Flutter found: {flutter_exe}")

        # 依赖解析
        header("Getting Flutter dependencies")
        flutter_dir = root / "CideFlutter"
        pub_args = [flutter_exe, "pub", "get"]
        if args.offline:
            pub_args.append("--offline")
            warn("Using offline mode — ensure all packages are in pub cache.")
        run(pub_args, cwd=flutter_dir)
        success("Dependencies resolved")

        # 生成 flutter_rust_bridge 绑定（Rust + Dart）
        header("Generating flutter_rust_bridge bindings")
        run(["flutter_rust_bridge_codegen", "generate"], cwd=flutter_dir)
        success("FRB bindings generated")

        # 构建
        if args.target in ("Desktop", "All"):
            build_flutter_desktop(
                root, args.configuration, args.run, args.skip_rust
            )

        if args.target in ("Android", "All"):
            build_flutter_android(root, args.configuration, args.skip_rust)

        header("Flutter Build Complete")
        return 0

    except subprocess.CalledProcessError as e:
        error(f"Command failed: {' '.join(e.cmd)}")
        if e.stderr:
            error(e.stderr)
        return e.returncode
    except Exception as e:
        error(str(e))
        return 1


if __name__ == "__main__":
    sys.exit(main())
