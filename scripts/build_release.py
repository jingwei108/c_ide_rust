#!/usr/bin/env python3
"""
Cide Release Build Script
Builds Flutter frontend with Rust backend in Release configuration.

Usage:
    python scripts/build_release.py                    # Build both Desktop and Android
    python scripts/build_release.py -t Desktop         # Build Desktop only
    python scripts/build_release.py -t Android         # Build Android only
    python scripts/build_release.py --clean            # Clean before build
"""

import argparse
import shutil
import subprocess
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
        description="Release build for Flutter Desktop and/or Android."
    )
    parser.add_argument(
        "-t",
        "--target",
        choices=["Desktop", "Android", "All"],
        default="All",
        help="Build target platform (default: All)",
    )
    parser.add_argument(
        "--clean", action="store_true", help="Clean build artifacts"
    )
    return parser.parse_args()


def clean_build(root: Path) -> None:
    header("Cleaning build artifacts")
    dirs = [
        root / "native" / "target" / "android",
        root / "CideFlutter" / "build",
        root / "dist",
    ]
    for d in dirs:
        if d.exists():
            shutil.rmtree(d)
            print(f"Removed {d}")


def find_flutter() -> str:
    """查找 flutter 可执行文件，支持常见安装路径。"""
    flutter = shutil.which("flutter")
    if flutter:
        return flutter

    candidates = [
        Path(r"D:\flutter\bin\flutter.bat"),
        Path(r"C:\flutter\bin\flutter.bat"),
        Path(r"D:\tools\flutter\bin\flutter.bat"),
    ]
    for c in candidates:
        if c.exists():
            return str(c)

    raise FileNotFoundError(
        "flutter command not found. Please add Flutter to your PATH.\n"
        "Common location: D:\\flutter\\bin"
    )


def build_desktop(root: Path) -> None:
    header("Building Desktop Release (Flutter Windows)")
    flutter_dir = root / "CideFlutter"
    flutter_exe = find_flutter()

    # Build Rust backend in release
    native_dir = root / "native"
    run(["cargo", "build", "--release"], cwd=native_dir)

    # Copy DLL to dist
    dll_source = root / "native" / "target" / "release" / "cide_native.dll"
    desktop_dir = root / "dist" / "desktop"
    if dll_source.exists():
        desktop_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy2(dll_source, desktop_dir / "cide_native.dll")
        success("Copied cide_native.dll -> dist/desktop/")
    else:
        warn(f"cide_native.dll not found at {dll_source}")

    # Build Flutter Windows release
    run([flutter_exe, "build", "windows", "--release"], cwd=flutter_dir)
    success("Flutter Windows release build completed")

    # Report output size
    build_dir = flutter_dir / "build" / "windows" / "x64" / "runner" / "Release"
    if build_dir.exists():
        exes = list(build_dir.glob("*.exe"))
        if exes:
            size_mb = round(exes[0].stat().st_size / (1024 * 1024), 2)
            success(f"Desktop EXE: {exes[0].name} ({size_mb} MB)")
        total = sum(f.stat().st_size for f in build_dir.rglob("*") if f.is_file())
        success(f"Desktop publish total: {round(total / (1024 * 1024), 2)} MB")


def build_android(root: Path) -> None:
    header("Building Android Release (Flutter APK)")
    flutter_dir = root / "CideFlutter"
    flutter_exe = find_flutter()

    # Build Rust Android .so if NDK available
    ndk_home = find_ndk()
    if ndk_home is None:
        warn("ANDROID_NDK_HOME not set. Skipping manual .so build.")
        warn("cargokit may still build it during Gradle phase if configured.")
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

            so_source = (
                root / "native" / "target" / rust_target / "release" / "libcide_native.so"
            )
            if so_source.exists():
                so_dest_dir = root / "native" / "target" / "android" / abi
                so_dest_dir.mkdir(parents=True, exist_ok=True)
                shutil.copy2(so_source, so_dest_dir / "libcide_native.so")
                success(
                    f"Copied libcide_native.so ({abi}) -> native/target/android/{abi}/"
                )
            else:
                warn(f"libcide_native.so not found for {abi} at {so_source}")

    # Build Flutter APK release
    run([flutter_exe, "build", "apk", "--release"], cwd=flutter_dir)
    success("Flutter Android release build completed")

    # Report APK size
    apk_dir = flutter_dir / "build" / "app" / "outputs" / "flutter-apk"
    if apk_dir.exists():
        apks = list(apk_dir.glob("*.apk"))
        if apks:
            apk = apks[0]
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
