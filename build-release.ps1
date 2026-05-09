# C IDE Release Build Script
# Builds both Desktop (Native AOT) and Android (AOT + Trim) in Release configuration.
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
        "native/build",
        "native/build-android-arm64-v8a",
        "native/build-android-armeabi-v7a",
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

    # Native backend
    $buildDir = Join-Path $root "native/build"
    New-Item -ItemType Directory -Path $buildDir -Force | Out-Null
    Push-Location $buildDir
    try {
        $hasNinja = Get-Command "ninja" -ErrorAction SilentlyContinue
        $hasVS = $env:VisualStudioVersion -or (Get-Command "msbuild" -ErrorAction SilentlyContinue)
        $cmakeArgs = @("..")
        if ($hasNinja) {
            $cmakeArgs += @("-G", "Ninja")
        } elseif (-not $hasVS) {
            $cmakeArgs += @("-G", "MinGW Makefiles")
        }
        $cmakeArgs += "-DCMAKE_BUILD_TYPE=$Configuration"
        & cmake @cmakeArgs
        if ($LASTEXITCODE -ne 0) { throw "CMake configuration failed" }
        cmake --build . --config $Configuration --parallel
        if ($LASTEXITCODE -ne 0) { throw "Build failed" }
    }
    finally { Pop-Location }

    # Copy native DLL
    $dllCandidates = @(
        (Join-Path $root "native/build/bin/$Configuration/cide_native.dll"),
        (Join-Path $root "native/build/bin/cide_native.dll"),
        (Join-Path $root "native/build/libcide_native.dll")
    )
    $dllSource = $null
    foreach ($c in $dllCandidates) {
        if (Test-Path $c) { $dllSource = $c; break }
    }
    if ($dllSource) {
        New-Item -ItemType Directory -Path $desktopDir -Force | Out-Null
        Copy-Item $dllSource (Join-Path $desktopDir "cide_native.dll") -Force
        Write-Success "Copied cide_native.dll -> dist/desktop/"
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
        $abis = @("arm64-v8a", "armeabi-v7a")
        foreach ($abi in $abis) {
            $buildDir = Join-Path $root "native/build-android-$abi"
            New-Item -ItemType Directory -Path $buildDir -Force | Out-Null
            Push-Location $buildDir
            try {
                $toolchain = Join-Path $ndkHome "build/cmake/android.toolchain.cmake"
                $cmakeArgs = @(
                    "..", "-G", "Ninja",
                    "-DCMAKE_TOOLCHAIN_FILE=$toolchain",
                    "-DANDROID_ABI=$abi",
                    "-DANDROID_PLATFORM=android-21",
                    "-DCMAKE_BUILD_TYPE=$Configuration",
                    "-DCIDE_BUILD_TESTS=OFF"
                )
                & cmake @cmakeArgs
                if ($LASTEXITCODE -ne 0) { throw "CMake failed for $abi" }
                cmake --build . --config $Configuration --parallel
                if ($LASTEXITCODE -ne 0) { throw "Build failed for $abi" }
            }
            finally { Pop-Location }
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
