# C IDE Build Script
# Builds the Rust native backend and the Avalonia / MAUI frontend.
#
# Usage:
#   .\build.ps1                           # Desktop Debug build
#   .\build.ps1 -Configuration Release    # Desktop Release build
#   .\build.ps1 -Target Android           # Android build (.so + APK)
#   .\build.ps1 -Clean                    # Clean all build artifacts
#   .\build.ps1 -Test                     # Run cargo test/clippy before build
#   .\build.ps1 -Run                      # Build and run desktop app

param(
    [ValidateSet("Debug", "Release")]
    [string]$Configuration = "Debug",

    [ValidateSet("Desktop", "Android", "All")]
    [string]$Target = "Desktop",

    [switch]$Clean,
    [switch]$Run,
    [switch]$Test
)

$ErrorActionPreference = "Stop"
$root = $PSScriptRoot

function Write-Header($text) {
    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host "  $text" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
}

function Write-Success($text) { Write-Host $text -ForegroundColor Green }
function Write-Warn($text) { Write-Host $text -ForegroundColor Yellow }

# ============================================================================
# Output directories
# ============================================================================
$distDir = Join-Path $root "dist"
$desktopDir = Join-Path $distDir "desktop"
$androidDir = Join-Path $distDir "android"

# ============================================================================
# Clean
# ============================================================================
if ($Clean) {
    Write-Header "Cleaning build artifacts"
    $dirs = @(
        "native/target",
        "native/target/android",
        "Cide.Client/bin", "Cide.Client/obj",
        "Cide.Client.Desktop/bin", "Cide.Client.Desktop/obj",
        "Cide.Client.Maui/bin", "Cide.Client.Maui/obj",
        "Cide.Client.Shared/bin", "Cide.Client.Shared/obj",
        "Cide.Client.Tests/bin", "Cide.Client.Tests/obj",
        "dist"
    )
    foreach ($d in $dirs) {
        $p = Join-Path $root $d
        if (Test-Path $p) {
            Remove-Item -Recurse -Force $p
            Write-Host "Removed $p"
        }
    }
}

# ============================================================================
# Test & Lint (Rust)
# ============================================================================
if ($Test) {
    Write-Header "Running Rust tests and lints"

    Push-Location (Join-Path $root "native")
    try {
        Write-Host "Running cargo test..."
        & cargo test
        if ($LASTEXITCODE -ne 0) { throw "cargo test failed (exit $LASTEXITCODE)" }
        Write-Success "cargo test passed"

        Write-Host "Running cargo clippy..."
        & cargo clippy
        if ($LASTEXITCODE -ne 0) { throw "cargo clippy failed (exit $LASTEXITCODE)" }
        Write-Success "cargo clippy passed"
    }
    finally {
        Pop-Location
    }
}

# ============================================================================
# Native Backend (Rust) — Desktop
# ============================================================================
if ($Target -eq "Desktop" -or $Target -eq "All") {
    Write-Header "Building Native Backend (Desktop)"

    Push-Location (Join-Path $root "native")
    try {
        $cargoArgs = if ($Configuration -eq "Release") { @("build", "--release") } else { @("build") }
        & cargo $cargoArgs
        if ($LASTEXITCODE -ne 0) { throw "cargo build failed (exit $LASTEXITCODE)" }
    }
    finally {
        Pop-Location
    }

    $dllSource = if ($Configuration -eq "Release") {
        Join-Path $root "native/target/release/cide_native.dll"
    } else {
        Join-Path $root "native/target/debug/cide_native.dll"
    }

    if (Test-Path $dllSource) {
        New-Item -ItemType Directory -Path $desktopDir -Force | Out-Null
        Copy-Item $dllSource (Join-Path $desktopDir "cide_native.dll") -Force
        Write-Success "Copied cide_native.dll -> dist/desktop/"
    } else {
        Write-Warn "cide_native.dll not found at $dllSource"
    }

    # ============================================================================
    # Avalonia Frontend (.NET)
    # ============================================================================
    Write-Header "Building Avalonia Desktop Frontend"

    Push-Location $root
    try {
        dotnet restore Cide.slnx
        if ($LASTEXITCODE -ne 0) { throw "dotnet restore failed" }

        dotnet publish Cide.Client.Desktop/Cide.Client.Desktop.csproj `
            -c $Configuration `
            -o $desktopDir `
            --self-contained false
        if ($LASTEXITCODE -ne 0) { throw "Desktop frontend build failed" }
    }
    finally {
        Pop-Location
    }

    Write-Success "Desktop artifacts collected in: $desktopDir"
}

# ============================================================================
# Native Backend (Rust) — Android
# ============================================================================
if ($Target -eq "Android" -or $Target -eq "All") {
    Write-Header "Building Native Backend (Android)"

    $ndkHome = $env:ANDROID_NDK_HOME
    if (-not $ndkHome) { $ndkHome = $env:ANDROID_NDK_ROOT }

    if (-not $ndkHome) {
        Write-Warn "ANDROID_NDK_HOME or ANDROID_NDK_ROOT not set. Skipping native .so build."
        Write-Warn "Set it to your Android NDK path, e.g.: `$env:ANDROID_NDK_HOME = 'C:\Android\ndk\27.0.1'"
    }
    else {
        $abiMap = @{
            "arm64-v8a"   = "aarch64-linux-android"
            "armeabi-v7a" = "armv7-linux-androideabi"
        }
        foreach ($abi in $abiMap.Keys) {
            $rustTarget = $abiMap[$abi]
            Write-Header "Building Native Backend (Android $abi)"

            Push-Location (Join-Path $root "native")
            try {
                $cargoArgs = @("ndk", "--target", $rustTarget, "--platform", "21", "build")
                if ($Configuration -eq "Release") {
                    $cargoArgs += "--release"
                }
                & cargo $cargoArgs
                if ($LASTEXITCODE -ne 0) { throw "cargo ndk build failed for $abi (exit $LASTEXITCODE)" }
            }
            finally {
                Pop-Location
            }

            $soDir = if ($Configuration -eq "Release") { "release" } else { "debug" }
            $soSource = Join-Path $root "native/target/$rustTarget/$soDir/libcide_native.so"
            $soCopied = $false
            if (Test-Path $soSource) {
                # Copy to csproj-referenced path (flatten release/debug)
                $soDestDir = Join-Path $root "native/target/android/$abi"
                New-Item -ItemType Directory -Path $soDestDir -Force | Out-Null
                Copy-Item $soSource (Join-Path $soDestDir "libcide_native.so") -Force
                Write-Success "Copied libcide_native.so ($abi) -> native/target/android/$abi/"

                # Also copy to legacy Maui/lib path for compatibility
                $mauiLibDir = Join-Path $root "Cide.Client.Maui/lib/$abi"
                New-Item -ItemType Directory -Path $mauiLibDir -Force | Out-Null
                Copy-Item $soSource (Join-Path $mauiLibDir "libcide_native.so") -Force
                Write-Success "Copied libcide_native.so ($abi) -> Cide.Client.Maui/lib/$abi/"
                $soCopied = $true
            }
            if (-not $soCopied) {
                Write-Warn "libcide_native.so not found for $abi at $soSource"
            }
        }
    }

    # ============================================================================
    # MAUI Android Frontend
    # ============================================================================
    Write-Header "Building MAUI Android Frontend"

    Push-Location $root
    try {
        dotnet restore Cide.slnx
        if ($LASTEXITCODE -ne 0) { throw "dotnet restore failed" }

        dotnet publish Cide.Client.Maui/Cide.Client.Maui.csproj `
            -f net10.0-android `
            -c $Configuration `
            -p:AndroidPackageFormat=apk `
            -o $androidDir `
            --self-contained false
        if ($LASTEXITCODE -ne 0) { throw "Android frontend build failed" }
    }
    finally {
        Pop-Location
    }

    Write-Success "Android artifacts collected in: $androidDir"
}

# ============================================================================
# Run
# ============================================================================
if ($Run -and $Target -eq "Desktop") {
    Write-Header "Running Desktop Application"
    $exe = Join-Path $desktopDir "Cide.Client.Desktop.exe"
    if (Test-Path $exe) {
        & $exe
    }
    else {
        throw "Executable not found: $exe"
    }
}

Write-Header "Build Complete"
