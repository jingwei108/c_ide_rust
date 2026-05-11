#!/usr/bin/env python3
"""
C IDE Release Build Script
Builds both Desktop (MAUI Windows) and Android (AOT + Trim) in Release configuration.
Native backend is built with Rust / cargo / cargo-ndk.

Usage:
    python scripts/build_release.py                    # Build both Desktop and Android
    python scripts/build_release.py -t Desktop         # Build Desktop only
    python scripts/build_release.py -t Android         # Build Android only
    python scripts/build_release.py --clean            # Clean before build
"""

import argparse
import shutil
import sys
from pathlib import Path

from build_utils import (
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
        description="Release build for Desktop and/or Android."
    )
    parser.add_argument(
        "-t",
        "--target",
        choices=["Desktop", "Android", "All"],
        default="All",
        help="Build target platform (default: All)",
    )
    parser.add_argument(
        "--clean", action="store_true", help="Clean all build artifacts"
    )
    return parser.parse_args()


def clean_build(root: Path) -> None:
    header("Cleaning build artifacts")
    dirs = [
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


def build_desktop(root: Path) -> None:
    header("Building Desktop (MAUI Windows)")
    configuration = "Release"

    # Native backend (Rust)
    native_dir = root / "native"
    run(["cargo", "build", "--release"], cwd=native_dir)

    # Copy native DLL
    dll_source = root / "native" / "target" / "release" / "cide_native.dll"
    desktop_dir = root / "dist" / "desktop"
    if dll_source.exists():
        desktop_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy2(dll_source, desktop_dir / "cide_native.dll")
        success("Copied cide_native.dll -> dist/desktop/")
    else:
        warn(f"cide_native.dll not found at {dll_source}")

    # Publish Desktop (MAUI Windows)
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
            "-r",
            "win-x64",
            "--self-contained",
            "true",
            "-o",
            str(desktop_dir),
        ],
        cwd=root,
    )

    # Report size
    exe = desktop_dir / "Cide.Client.Maui.exe"
    if exe.exists():
        size_mb = round(exe.stat().st_size / (1024 * 1024), 2)
        success(f"Desktop EXE: {size_mb} MB")

    total = sum(f.stat().st_size for f in desktop_dir.rglob("*") if f.is_file())
    success(f"Desktop publish total: {round(total / (1024 * 1024), 2)} MB")


def build_android(root: Path) -> None:
    header("Building Android (AOT + Trim + r8)")
    configuration = "Release"

    ndk_home = find_ndk()
    if ndk_home is None:
        warn("ANDROID_NDK_HOME not set. Skipping native .so build.")
    else:
        abi_map = {
            "arm64-v8a": "aarch64-linux-android",
            "armeabi-v7a": "armv7-linux-androideabi",
        }
        for abi, rust_target in abi_map.items():
            header(f"Building Native Backend (Android {abi})")
            native_dir = root / "native"
            try:
                run(
                    [
                        "cargo",
                        "ndk",
                        "--target",
                        rust_target,
                        "--platform",
                        "21",
                        "build",
                        "--release",
                    ],
                    cwd=native_dir,
                )
            except Exception as e:
                error(f"Native Android build ({abi}) failed: {e}")
                raise

            so_source = root / "native" / "target" / rust_target / "release" / "libcide_native.so"
            if so_source.exists():
                so_dest_dir = root / "native" / "target" / "android" / abi
                so_dest_dir.mkdir(parents=True, exist_ok=True)
                shutil.copy2(so_source, so_dest_dir / "libcide_native.so")
                success(f"Copied libcide_native.so ({abi}) -> native/target/android/{abi}/")

                maui_lib_dir = root / "Cide.Client.Maui" / "lib" / abi
                maui_lib_dir.mkdir(parents=True, exist_ok=True)
                shutil.copy2(so_source, maui_lib_dir / "libcide_native.so")
                success(f"Copied libcide_native.so ({abi}) -> Cide.Client.Maui/lib/{abi}/")
            else:
                warn(f"libcide_native.so not found for {abi} at {so_source}")

    # Publish MAUI Android APK
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
        ],
        cwd=root,
    )

    # Report APK size
    apk_candidates = [
        android_dir / "com.cide.app-Signed.apk",
    ]
    apk = None
    for c in apk_candidates:
        if c.exists():
            apk = c
            break
    if apk is None:
        signed_apks = list(android_dir.glob("*Signed.apk"))
        if signed_apks:
            apk = signed_apks[0]
    if apk and apk.exists():
        size_mb = round(apk.stat().st_size / (1024 * 1024), 2)
        success(f"APK: {apk.name} ({size_mb} MB)")


def main() -> int:
    args = parse_args()
    root = get_project_root()

    try:
        if args.clean:
            clean_build(root)

        if args.target in ("Desktop", "All"):
            build_desktop(root)

        if args.target in ("Android", "All"):
            build_android(root)

        header("Release Build Complete")
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
