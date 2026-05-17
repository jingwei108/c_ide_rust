#!/usr/bin/env python3
"""
Cide Build Script (Flutter)
Builds the Flutter frontend with Rust backend.

This script delegates to build_flutter.py for all operations.

Usage:
    python scripts/build.py                           # Desktop Debug build
    python scripts/build.py -c Release                # Desktop Release build
    python scripts/build.py -t Android                # Android build (.so + APK)
    python scripts/build.py --clean                   # Clean all build artifacts
    python scripts/build.py --test                    # Run cargo test/clippy before build
    python scripts/build.py --run                     # Build and run desktop app
"""

import subprocess
import sys
from pathlib import Path


def main() -> int:
    """Delegate all operations to build_flutter.py."""
    script_dir = Path(__file__).parent
    flutter_script = script_dir / "build_flutter.py"
    
    if not flutter_script.exists():
        print(f"Error: {flutter_script} not found.", file=sys.stderr)
        return 1
    
    # Pass through all command-line arguments
    result = subprocess.run([sys.executable, str(flutter_script)] + sys.argv[1:])
    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
