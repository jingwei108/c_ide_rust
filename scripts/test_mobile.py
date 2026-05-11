#!/usr/bin/env python3
"""
C IDE Mobile Test Script
Builds native .so libraries, packages the Android APK, and optionally
installs / runs / captures logs on a connected device or emulator.

Usage:
    python scripts/test_mobile.py                    # Full build (native + APK)
    python scripts/test_mobile.py --install --run    # Build, install APK, and launch app
    python scripts/test_mobile.py --run --logcat     # Build, install, launch, then tail logs
    python scripts/test_mobile.py --skip-native-build # Only build APK (reuse existing .so)
"""

import argparse
import shutil
import subprocess
import sys
import time
from pathlib import Path

from build_utils import (
    detect_device,
    error,
    find_adb,
    find_ndk,
    header,
    info,
    run,
    success,
    warn,
    get_project_root,
)

PACKAGE_NAME = "com.cide.app"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Mobile test pipeline for MAUI Android."
    )
    parser.add_argument(
        "-c",
        "--configuration",
        choices=["Debug", "Release"],
        default="Debug",
        help="Build configuration (default: Debug)",
    )
    parser.add_argument(
        "--skip-native-build",
        action="store_true",
        help="Skip NDK .so compilation, only re-package APK",
    )
    parser.add_argument(
        "--install", action="store_true", help="Install APK to device after build"
    )
    parser.add_argument(
        "--run", action="store_true", help="Launch app after installation"
    )
    parser.add_argument(
        "--logcat", action="store_true", help="Capture app logs after launch (Ctrl+C to stop)"
    )
    return parser.parse_args()


def build_native_so(root: Path, configuration: str) -> None:
    header("Building Native Backend (Android)")
    ndk_home = find_ndk()
    if ndk_home is None:
        error("Android NDK not found. Set ANDROID_NDK_HOME or install VS Android workload.")
        sys.exit(1)

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


def build_apk(root: Path, configuration: str) -> Path:
    header("Building MAUI Android APK")
    android_dir = root / "dist" / "android"

    # Force clean MAUI obj cache so updated .so files are re-packaged
    maui_obj_dir = root / "Cide.Client.Maui" / "obj"
    if maui_obj_dir.exists():
        warn("Cleaning MAUI build cache to ensure fresh .so packaging...")
        shutil.rmtree(maui_obj_dir)
    if android_dir.exists():
        shutil.rmtree(android_dir)

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

    # Locate the signed APK
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
    if apk is None:
        apks = list(android_dir.glob("*.apk"))
        if apks:
            apk = apks[0]
    if apk is None:
        raise FileNotFoundError(f"No APK found in {android_dir}")

    size_mb = round(apk.stat().st_size / (1024 * 1024), 2)
    success(f"APK built: {apk} ({size_mb} MB)")
    return apk


def install_apk(adb: Path, device: str, apk: Path) -> None:
    header("Installing APK")
    warn("Uninstalling old version to clear WebView cache...")
    run([str(adb), "-s", device, "uninstall", PACKAGE_NAME], check=False)
    run([str(adb), "-s", device, "install", "-d", str(apk)])
    success("APK installed successfully")


def launch_app(adb: Path, device: str) -> None:
    header("Launching C IDE (MAUI)")
    run(
        [
            str(adb),
            "-s",
            device,
            "shell",
            "monkey",
            "-p",
            PACKAGE_NAME,
            "-c",
            "android.intent.category.LAUNCHER",
            "1",
        ],
        check=False,
    )
    success("App launched on device")


def capture_logcat(adb: Path, device: str) -> None:
    header("Starting Logcat (Ctrl+C to stop)")
    run([str(adb), "-s", device, "logcat", "-c"], check=False)
    result = run(
        [str(adb), "-s", device, "shell", "pidof", PACKAGE_NAME],
        capture_output=True,
        check=False,
    )
    app_pid = result.stdout.strip().split()[0] if result.stdout else ""
    if app_pid and app_pid.isdigit():
        print(f"Filtering logcat for PID: {app_pid}")
        try:
            run([str(adb), "-s", device, "logcat", f"--pid={app_pid}"])
        except KeyboardInterrupt:
            pass
    else:
        warn(f"Could not get PID for {PACKAGE_NAME}, showing unfiltered logcat")
        result = run(
            [str(adb), "-s", device, "logcat", "-d"],
            capture_output=True,
            check=False,
        )
        lines = result.stdout.strip().splitlines()
        for line in lines[-100:]:
            print(line)


def main() -> int:
    args = parse_args()
    root = get_project_root()

    adb = find_adb()

    try:
        if not args.skip_native_build:
            build_native_so(root, args.configuration)
        else:
            warn("Skipping native .so build (--skip-native-build)")

        apk = build_apk(root, args.configuration)

        if args.install or args.run or args.logcat:
            if adb is None:
                error("adb not found. Cannot install/run on device.")
                return 1

            device = detect_device(adb)

            if args.install or args.run:
                install_apk(adb, device, apk)

            if args.run:
                launch_app(adb, device)

            if args.logcat:
                capture_logcat(adb, device)

        header("Mobile Test Complete")
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
