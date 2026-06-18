#!/usr/bin/env python3
"""
Patch Flutter tools to honor the CMAKE_GENERATOR environment variable on Windows.

Flutter 3.29 on Windows always passes an explicit -G <generator> based on the
 detected Visual Studio version, which overrides the CMAKE_GENERATOR environment
 variable set in CI. This patch makes the Flutter tool fall back to the
 environment variable first, so CI can force a generator that actually exists on
 the runner (e.g. "Visual Studio 17 2022").

Usage:
    python scripts/patch_flutter_windows_generator.py
"""

import shutil
import subprocess
import sys
from pathlib import Path


def find_flutter_root() -> Path | None:
    """Return the Flutter SDK root directory."""
    flutter_exe = shutil.which("flutter")
    if not flutter_exe:
        return None
    # flutter executable is at <sdk>/bin/flutter (.bat on Windows)
    return Path(flutter_exe).resolve().parent.parent


def patch_build_windows_dart(sdk_root: Path) -> bool:
    """Apply the CMAKE_GENERATOR patch to build_windows.dart."""
    target = (
        sdk_root
        / "packages"
        / "flutter_tools"
        / "lib"
        / "src"
        / "windows"
        / "build_windows.dart"
    )
    if not target.exists():
        print(f"ERROR: {target} not found", file=sys.stderr)
        return False

    original = target.read_text(encoding="utf-8")
    old_line = "final String? cmakeGenerator = visualStudio.cmakeGenerator;"
    new_line = (
        "final String? cmakeGenerator = "
        "globals.platform.environment['CMAKE_GENERATOR'] ?? visualStudio.cmakeGenerator;"
    )

    if new_line in original:
        print(f"Already patched: {target}")
        return True

    if old_line not in original:
        print(
            f"WARNING: Expected line not found in {target}; patch may be unnecessary "
            f"or Flutter version changed.",
            file=sys.stderr,
        )
        return False

    patched = original.replace(old_line, new_line, 1)
    target.write_text(patched, encoding="utf-8")
    print(f"Patched: {target}")
    return True


def clear_flutter_tools_cache(sdk_root: Path) -> None:
    """Remove cached flutter_tools snapshot so the patch is picked up."""
    cache_dir = sdk_root / "bin" / "cache"
    if not cache_dir.exists():
        return
    for entry in cache_dir.iterdir():
        if entry.name.startswith("flutter_tools."):
            if entry.is_file() or entry.is_symlink():
                entry.unlink()
                print(f"Removed cache: {entry}")
            elif entry.is_dir():
                shutil.rmtree(entry)
                print(f"Removed cache dir: {entry}")


def main() -> int:
    sdk_root = find_flutter_root()
    if sdk_root is None:
        print("ERROR: flutter executable not found in PATH", file=sys.stderr)
        return 1

    print(f"Flutter SDK root: {sdk_root}")
    if not patch_build_windows_dart(sdk_root):
        return 1
    clear_flutter_tools_cache(sdk_root)
    print("Flutter Windows generator patch applied successfully.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
