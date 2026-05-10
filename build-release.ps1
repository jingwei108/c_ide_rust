# C IDE Release Build Script
# Builds both Desktop (Native AOT) and Android (AOT + Trim) in Release configuration.
# Native backend is built with Rust / cargo / cargo-ndk.
#
# Usage:
#   .\build-release.ps1                    # Build both Desktop and Android
#   .\build-release.ps1 -Target Desktop    # Build Desktop only
#   .\build-release.ps1 -Target Android    # Build Android only
#   .\build-release.ps1 -Clean             # Clean before build

param(
    [ValidateSet("Desktop", "Android", "All")]
    [string]$Target = "All",

    [switch]$Clean
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
# Clean
# ============================================================================
if ($Clean) {
    Write-Header "Cleaning build artifacts"
    $dirs = @(
        "native/target/android",
        "Cide.Client/bin", "Cide.Client/obj",
        "Cide.Client.Desktop/bin", "Cide.Client.Desktop/obj",
        "Cide.Client.Maui/bin", "Cide.Client.Maui/obj",
        "Cide.Client.Shared/bin", "Cide.Client.Shared/obj",
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

$Configuration = "Release"
$distDir = Join-Path $root "dist"
$desktopDir = Join-Path $distDir "desktop"
$androidDir = Join-Path $distDir "android"

# ============================================================================
# Desktop (Native AOT)
# ============================================================================
if ($Target -eq "Desktop" -or $Target -eq "All") {
    Write-Header "Building Desktop (Native AOT + Trim)"

    # Native backend (Rust)
    Push-Location (Join-Path $root "native")
    try {
        $oldEAP = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        & cargo build --release 2>$null
        $cargoExit = $LASTEXITCODE
        $ErrorActionPreference = $oldEAP
        if ($cargoExit -ne 0) { throw "Cargo build failed (exit $cargoExit)" }
    }
    finally { Pop-Location }

    # Copy native DLL
    $dllSource = Join-Path $root "native/target/release/cide_native.dll"
    if (Test-Path $dllSource) {
        New-Item -ItemType Directory -Path $desktopDir -Force | Out-Null
        Copy-Item $dllSource (Join-Path $desktopDir "cide_native.dll") -Force
        Write-Success "Copied cide_native.dll -> dist/desktop/"
    } else {
        Write-Warn "cide_native.dll not found at $dllSource"
    }

    # Publish Desktop with Native AOT
    Push-Location $root
    try {
        dotnet restore Cide.slnx
        dotnet publish Cide.Client.Desktop/Cide.Client.Desktop.csproj `
            -c $Configuration `
            -r win-x64 `
            --self-contained true `
            -o $desktopDir
    }
    catch { Write-Error "Desktop publish failed: $_" }
    finally { Pop-Location }

    # Report size
    $exe = Join-Path $desktopDir "Cide.Client.Desktop.exe"
    if (Test-Path $exe) {
        $sizeMB = [math]::Round((Get-Item $exe).Length / 1MB, 2)
        Write-Success "Desktop EXE: $sizeMB MB"
    }
    $total = (Get-ChildItem $desktopDir -Recurse | Measure-Object -Property Length -Sum).Sum
    Write-Success "Desktop publish total: $([math]::Round($total/1MB,2)) MB"
}

# ============================================================================
# Android (AOT + Trim + r8)
# ============================================================================
if ($Target -eq "Android" -or $Target -eq "All") {
    Write-Header "Building Android (AOT + Trim + r8)"

    $ndkHome = $env:ANDROID_NDK_HOME
    if (-not $ndkHome) { $ndkHome = $env:ANDROID_NDK_ROOT }

    if (-not $ndkHome) {
        Write-Warn "ANDROID_NDK_HOME not set. Skipping native .so build."
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
                $oldEAP = $ErrorActionPreference
                $ErrorActionPreference = "Continue"
                & cargo ndk --target $rustTarget --platform 21 build --release 2>$null
                $cargoExit = $LASTEXITCODE
                $ErrorActionPreference = $oldEAP
                if ($cargoExit -ne 0) { throw "Cargo NDK build failed for $abi (exit $cargoExit)" }
            }
            catch {
                Write-Error "Native Android build ($abi) failed: $_"
            }
            finally { Pop-Location }

            $soSource = Join-Path $root "native/target/$rustTarget/release/libcide_native.so"
            if (Test-Path $soSource) {
                $soDestDir = Join-Path $root "native/target/android/$abi"
                New-Item -ItemType Directory -Path $soDestDir -Force | Out-Null
                Copy-Item $soSource (Join-Path $soDestDir "libcide_native.so") -Force
                Write-Success "Copied libcide_native.so ($abi) -> native/target/android/$abi/"

                $mauiLibDir = Join-Path $root "Cide.Client.Maui/lib/$abi"
                New-Item -ItemType Directory -Path $mauiLibDir -Force | Out-Null
                Copy-Item $soSource (Join-Path $mauiLibDir "libcide_native.so") -Force
                Write-Success "Copied libcide_native.so ($abi) -> Cide.Client.Maui/lib/$abi/"
            }
            else {
                Write-Warn "libcide_native.so not found for $abi at $soSource"
            }
        }
    }

    # Publish MAUI Android APK
    Push-Location $root
    try {
        dotnet restore Cide.slnx
        dotnet publish Cide.Client.Maui/Cide.Client.Maui.csproj `
            -f net10.0-android `
            -c $Configuration `
            -p:AndroidPackageFormat=apk `
            -o "$androidDir"
    }
    catch { Write-Error "Android publish failed: $_" }
    finally { Pop-Location }

    # Report APK size
    $apk = Get-ChildItem -Path $androidDir -Filter "com.cide.mobile-Signed.apk" | Select-Object -First 1
    if (-not $apk) { $apk = Get-ChildItem -Path $androidDir -Filter "*Signed.apk" | Select-Object -First 1 }
    if ($apk) {
        $sizeMB = [math]::Round($apk.Length / 1MB, 2)
        Write-Success "APK: $($apk.Name) ($sizeMB MB)"
    }
}

Write-Header "Release Build Complete"
