"""
C IDE Build Scripts — Shared Utilities
提供跨脚本的通用功能：彩色输出、命令执行、ADB/NDK 自动探测、设备检测等。
"""

import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path
from typing import List, Optional


class Colors:
    """ANSI 颜色码。Windows 旧版终端会自动忽略不支持的转义序列。"""
    CYAN = "\033[36m"
    GREEN = "\033[32m"
    YELLOW = "\033[33m"
    RED = "\033[31m"
    RESET = "\033[0m"


def _supports_color() -> bool:
    """简单判断当前终端是否支持颜色输出。"""
    if sys.platform == "win32":
        # Windows 10 1909+ 默认支持 ANSI；旧版可通过 ENABLE_VIRTUAL_TERMINAL_PROCESSING 开启
        return True
    return hasattr(sys.stdout, "isatty") and sys.stdout.isatty()


_SUPPORTS_COLOR = _supports_color()


def _c(text: str, color: str) -> str:
    if _SUPPORTS_COLOR:
        return f"{color}{text}{Colors.RESET}"
    return text


def header(text: str) -> None:
    bar = "=" * 40
    print(f"\n{bar}")
    print(_c(f"  {text}", Colors.CYAN))
    print(f"{bar}")


def success(text: str) -> None:
    print(_c(text, Colors.GREEN))


def warn(text: str) -> None:
    print(_c(text, Colors.YELLOW))


def error(text: str) -> None:
    print(_c(text, Colors.RED), file=sys.stderr)


def info(text: str) -> None:
    print(_c(f"  ℹ {text}", Colors.YELLOW))


def run(
    cmd: List[str],
    cwd: Optional[Path] = None,
    check: bool = True,
    capture_output: bool = False,
    env: Optional[dict] = None,
) -> subprocess.CompletedProcess:
    """
    执行外部命令，封装 subprocess.run。

    Args:
        cmd: 命令及参数列表（同 subprocess.run 的 args）
        cwd: 工作目录
        check: 非零退出码时是否抛出 CalledProcessError
        capture_output: 是否捕获 stdout/stderr
        env: 额外的环境变量（在 os.environ 基础上更新）

    Returns:
        subprocess.CompletedProcess 实例
    """
    merged_env = None
    if env:
        merged_env = {**os.environ, **env}

    if not capture_output:
        # 直接透传输出到终端
        result = subprocess.run(cmd, cwd=cwd, check=False, env=merged_env)
    else:
        result = subprocess.run(
            cmd, cwd=cwd, check=False, capture_output=True, text=True, env=merged_env
        )

    if check and result.returncode != 0:
        raise subprocess.CalledProcessError(
            result.returncode, cmd, output=result.stdout, stderr=result.stderr
        )
    return result


def find_adb() -> Optional[Path]:
    """
    查找 adb 可执行文件路径。
    1. 先查 PATH
    2. 再探测 VS 默认 Android SDK 路径
    """
    adb = shutil.which("adb")
    if adb:
        return Path(adb)

    vs_adb = Path(
        r"D:\Program Files (x86)\Microsoft Visual Studio\Shared\Android\android-sdk\platform-tools\adb.exe"
    )
    if vs_adb.exists():
        return vs_adb

    return None


def find_ndk() -> Optional[Path]:
    """
    查找 Android NDK 路径。
    1. 先查 ANDROID_NDK_HOME / ANDROID_NDK_ROOT 环境变量
    2. 再探测 VS 默认安装路径
    """
    for env_key in ("ANDROID_NDK_HOME", "ANDROID_NDK_ROOT"):
        val = os.environ.get(env_key)
        if val and Path(val).exists():
            return Path(val)

    vs_android = Path(
        r"D:\Program Files (x86)\Microsoft Visual Studio\Shared\Android\AndroidNDK"
    )
    if vs_android.exists():
        candidates = sorted(
            [d for d in vs_android.iterdir() if d.is_dir()],
            key=lambda p: p.name,
            reverse=True,
        )
        if candidates:
            return candidates[0]

    return None


def detect_device(adb: Path, max_retries: int = 3) -> str:
    """
    检测已连接的 Android 设备，支持自动重试。

    Args:
        adb: adb 可执行文件路径
        max_retries: 最大重试次数

    Returns:
        设备序列号

    Raises:
        RuntimeError: 未找到可用设备
    """
    for retry in range(1, max_retries + 1):
        result = run([str(adb), "devices"], capture_output=True)
        lines = result.stdout.strip().splitlines()

        devices: List[str] = []
        offline: List[str] = []

        for line in lines:
            line = line.strip()
            if not line or line.startswith("List of devices"):
                continue
            parts = line.split()
            if len(parts) >= 2:
                if parts[-1] == "device":
                    devices.append(parts[0])
                elif parts[-1] == "offline":
                    offline.append(parts[0])

        if devices:
            return devices[0]

        if offline:
            warn(f"Device(s) offline. Attempting adb server restart ({retry}/{max_retries})...")
            run([str(adb), "kill-server"], check=False)
            run([str(adb), "start-server"], check=False)
            time.sleep(2)
        elif retry < max_retries:
            warn(f"No device found. Retrying in 3 seconds ({retry}/{max_retries})...")
            time.sleep(3)

    raise RuntimeError("No Android device or emulator detected.")


def get_project_root() -> Path:
    """返回项目根目录（scripts/build_utils.py 的上两级目录）。"""
    return Path(__file__).resolve().parent.parent


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


def build_rust_android(
    root: Path,
    configuration: str = "Debug",
    copy_to_native_target_android: bool = False,
) -> None:
    """手动构建 Rust Android .so（多 ABI）。

    Args:
        root: 项目根目录。
        configuration: Debug 或 Release。
        copy_to_native_target_android: 若 True，将构建产物复制到 native/target/android/<abi>/。
    """
    header("Building Rust Backend (Android)")
    ndk_home = find_ndk()
    if ndk_home is None:
        warn("ANDROID_NDK_HOME / ANDROID_NDK_ROOT not set. Skipping manual .so build.")
        warn("cargokit may still build it during Gradle phase if configured.")
        return

    abi_map = {
        "arm64-v8a": "aarch64-linux-android",
        "armeabi-v7a": "armv7-linux-androideabi",
    }
    native_dir = root / "native"
    profile = "release" if configuration == "Release" else "debug"

    for abi, rust_target in abi_map.items():
        header(f"Building libcide_native.so ({abi})")
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

        so_source = native_dir / "target" / rust_target / profile / "libcide_native.so"
        if not so_source.exists():
            warn(f"Expected .so not found: {so_source}")
            continue

        success(f"Built {so_source}")

        if copy_to_native_target_android:
            so_dest_dir = root / "native" / "target" / "android" / abi
            so_dest_dir.mkdir(parents=True, exist_ok=True)
            shutil.copy2(so_source, so_dest_dir / "libcide_native.so")
            success(f"Copied libcide_native.so ({abi}) -> {so_dest_dir}")
