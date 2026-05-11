#!/usr/bin/env python3
"""
C IDE Build Script
Builds the Rust native backend and the MAUI frontend (Android + Desktop).

Usage:
    python scripts/build.py                           # Desktop Debug build
    python scripts/build.py -c Release                # Desktop Release build
    python scripts/build.py -t Android                # Android build (.so + APK)
    python scripts/build.py --clean                   # Clean all build artifacts
    python scripts/build.py --test                    # Run cargo test/clippy before build
    python scripts/build.py --run                     # Build and run desktop app
"""

import argparse
import shutil
import sys
from pathlib import Path

from build_utils import (
    Colors,
    error,
    find_ndk,
    header,
    run,
    success,
    warn,
    get_project_root,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build the Rust native backend and the MAUI frontend."
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
        "--clean", action="store_true", help="Clean all build artifacts"
    )
    parser.add_argument(
        "--run", action="store_true", help="Build and run desktop app (only for Desktop target)"
    )
    parser.add_argument(
        "--test", action="store_true", help="Run cargo test and clippy before build"
    )
    return parser.parse_args()


def clean_build(root: Path) -> None:
    header("Cleaning build artifacts")
    dirs = [
        root / "native" / "target",
        root / "native" / "target" / "android",
        root / "Cide.Client.Maui" / "bin",
        root / "Cide.Client.Maui" / "obj",
        root / "Cide.Client.Shared" / "bin",
        root / "Cide.Client.Shared" / "obj",
        root / "Cide.Client.Tests" / "bin",
        root / "Cide.Client.Tests" / "obj",
        root / "dist",
    ]
    for d in dirs:
        if d.exists():
            shutil.rmtree(d)
            print(f"Removed {d}")


def run_rust_tests(root: Path) -> None:
    header("Running Rust tests and lints")
    native_dir = root / "native"
    print("Running cargo test...")
    run(["cargo", "test"], cwd=native_dir)
    success("cargo test passed")

    print("Running cargo clippy...")
    run(["cargo", "clippy"], cwd=native_dir)
    success("cargo clippy passed")


def build_desktop(root: Path, configuration: str) -> None:
    header("Building Native Backend (Desktop)")
    native_dir = root / "native"
    cargo_args = ["cargo", "build"]
    if configuration == "Release":
        cargo_args.append("--release")
    run(cargo_args, cwd=native_dir)

    dll_name = "cide_native.dll"
    profile = "release" if configuration == "Release" else "debug"
    dll_source = root / "native" / "target" / profile / dll_name
    desktop_dir = root / "dist" / "desktop"

    if dll_source.exists():
        desktop_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy2(dll_source, desktop_dir / dll_name)
        success(f"Copied {dll_name} -> dist/desktop/")
    else:
        warn(f"{dll_name} not found at {dll_source}")

    header("Building MAUI Desktop Frontend")
    run(["dotnet", "restore", "Cide.slnx"], cwd=root)
    run(
        [
            "dotnet",
            "publish",
            "Cide.Client.Maui/Cide.Client.Maui.csproj",
            "-f",
            "net10.0-windows10.0.19041.0",
            "-c",
            configuration,
            "-o",
            str(desktop_dir),
            "--self-contained",
            "false",
        ],
        cwd=root,
    )
    success(f"Desktop artifacts collected in: {desktop_dir}")


def run_desktop(root: Path) -> None:
    header("Running Desktop Application")
    exe = root / "dist" / "desktop" / "Cide.Client.Maui.exe"
    if exe.exists():
        run([str(exe)], cwd=root, check=False)
    else:
        raise FileNotFoundError(f"Executable not found: {exe}")


def build_android(root: Path, configuration: str) -> None:
    header("Building Native Backend (Android)")
    ndk_home = find_ndk()

    if ndk_home is None:
        warn("ANDROID_NDK_HOME or ANDROID_NDK_ROOT not set. Skipping native .so build.")
        warn("Set it to your Android NDK path, e.g.: $env:ANDROID_NDK_HOME = 'C:\\Android\\ndk\\27.0.1'")
    else:
        abi_map = {
            "arm64-v8a": "aarch64-linux-android",
            "armeabi-v7a": "armv7-linux-androideabi",
        }
        for abi, rust_target in abi_map.items():
            header(f"Building Native Backend (Android {abi})")
            native_dir = root / "native"
            cargo_args = [
                "cargo",
                "ndk",
                "--target",
                rust_target,
                "--platform",
                "21",
                "build",
            ]
            if configuration == "Release":
                cargo_args.append("--release")
            run(cargo_args, cwd=native_dir)

            profile = "release" if configuration == "Release" else "debug"
            so_source = root / "native" / "target" / rust_target / profile / "libcide_native.so"
            so_copied = False

            if so_source.exists():
                so_dest_dir = root / "native" / "target" / "android" / abi
                so_dest_dir.mkdir(parents=True, exist_ok=True)
                shutil.copy2(so_source, so_dest_dir / "libcide_native.so")
                success(f"Copied libcide_native.so ({abi}) -> native/target/android/{abi}/")

                maui_lib_dir = root / "Cide.Client.Maui" / "lib" / abi
                maui_lib_dir.mkdir(parents=True, exist_ok=True)
                shutil.copy2(so_source, maui_lib_dir / "libcide_native.so")
                success(f"Copied libcide_native.so ({abi}) -> Cide.Client.Maui/lib/{abi}/")
                so_copied = True

            if not so_copied:
                warn(f"libcide_native.so not found for {abi} at {so_source}")

    header("Building MAUI Android Frontend")
    android_dir = root / "dist" / "android"
    run(["dotnet", "restore", "Cide.slnx"], cwd=root)
    run(
        [
            "dotnet",
            "publish",
            "Cide.Client.Maui/Cide.Client.Maui.csproj",
            "-f",
            "net10.0-android",
            "-c",
            configuration,
            "-p:AndroidPackageFormat=apk",
            "-o",
            str(android_dir),
            "--self-contained",
            "false",
        ],
        cwd=root,
    )
    success(f"Android artifacts collected in: {android_dir}")


def main() -> int:
    args = parse_args()
    root = get_project_root()

    try:
        if args.clean:
            clean_build(root)

        if args.test:
            run_rust_tests(root)

        if args.target in ("Desktop", "All"):
            build_desktop(root, args.configuration)
            if args.run:
                run_desktop(root)

        if args.target in ("Android", "All"):
            build_android(root, args.configuration)

        header("Build Complete")
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
