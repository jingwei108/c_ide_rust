# C IDE Build Script
# Builds both the C++ native backend and the Avalonia frontend.

param(
    [ValidateSet("Debug", "Release")]
    [string]$Configuration = "Debug",

    [ValidateSet("Desktop", "Android", "All")]
    [string]$Target = "Desktop",

    [switch]$Clean,
    [switch]$Run,

    [ValidateSet("Default", "Clang", "ClangCL", "MSVC", "MinGW")]
    [string]$Compiler = "Default"
)

$ErrorActionPreference = "Stop"
$root = $PSScriptRoot

function Write-Header($text) {
    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host "  $text" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
}

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

# ============================================================================
# Native Backend (Rust)
# ============================================================================
if ($Target -eq "Desktop" -or $Target -eq "All") {
    Write-Header "Building Native Backend (Desktop)"

    Push-Location (Join-Path $root "native")
    try {
        $cargoArgs = @("build", "--release")
        if ($Configuration -eq "Debug") {
            $cargoArgs = @("build")
        }
        $oldEAP = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        & cargo @cargoArgs 2>$null
        $cargoExit = $LASTEXITCODE
        $ErrorActionPreference = $oldEAP
        if ($cargoExit -ne 0) { throw "Cargo build failed (exit $cargoExit)" }
    }
    catch {
        Write-Error "Native backend build failed: $_"
    }
    finally {
        Pop-Location
    }

    $dllSource = Join-Path $root "native/target/release/cide_native.dll"
    if ($Configuration -eq "Debug") {
        $dllSource = Join-Path $root "native/target/debug/cide_native.dll"
    }
    if (Test-Path $dllSource) {
        New-Item -ItemType Directory -Path $desktopDir -Force | Out-Null
        Copy-Item $dllSource (Join-Path $desktopDir "cide_native.dll") -Force
        Write-Host "Copied cide_native.dll -> dist/desktop/" -ForegroundColor Green
    } else {
        Write-Warning "cide_native.dll not found at $dllSource"
    }
}

# ============================================================================
# Avalonia Frontend (.NET)
# ============================================================================
if ($Target -eq "Desktop" -or $Target -eq "All") {
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

    Write-Host "Desktop artifacts collected in: $desktopDir" -ForegroundColor Green
}

if ($Target -eq "Android" -or $Target -eq "All") {
    Write-Header "Building Native Backend (Android)"

    # Detect Android NDK
    $ndkHome = $env:ANDROID_NDK_HOME
    if (-not $ndkHome) { $ndkHome = $env:ANDROID_NDK_ROOT }
    if (-not $ndkHome) {
        Write-Warning "ANDROID_NDK_HOME or ANDROID_NDK_ROOT not set. Skipping native .so build."
        Write-Warning "Set it to your Android NDK path, e.g.: `$env:ANDROID_NDK_HOME = 'C:\Android\ndk\27.0.1'"
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
                $cargoArgs = @(
                    "ndk",
                    "--target", $rustTarget,
                    "--platform", "21",
                    "build"
                )
                if ($Configuration -eq "Release") {
                    $cargoArgs += "--release"
                }
                $oldEAP = $ErrorActionPreference
                $ErrorActionPreference = "Continue"
                & cargo @cargoArgs 2>$null
                $cargoExit = $LASTEXITCODE
                $ErrorActionPreference = $oldEAP
                if ($cargoExit -ne 0) { throw "Cargo NDK build failed for $abi (exit $cargoExit)" }
            }
            catch {
                Write-Error "Native Android build ($abi) failed: $_"
            }
            finally {
                Pop-Location
            }

            $soDir = if ($Configuration -eq "Release") { "release" } else { "debug" }
            $soSource = Join-Path $root "native/target/$rustTarget/$soDir/libcide_native.so"
            if (Test-Path $soSource) {
                $soDestDir = Join-Path $root "Cide.Client.Maui/lib/$abi"
                New-Item -ItemType Directory -Path $soDestDir -Force | Out-Null
                Copy-Item $soSource (Join-Path $soDestDir "libcide_native.so") -Force
                Write-Host "Copied libcide_native.so ($abi) -> Cide.Client.Maui/lib/$abi/" -ForegroundColor Green
            }
            else {
                Write-Warning "libcide_native.so not found for $abi at $soSource"
            }
        }
    }

    Write-Header "Building MAUI Android Frontend"

    Push-Location $root
    try {
        dotnet restore Cide.slnx
        if ($LASTEXITCODE -ne 0) { throw "dotnet restore failed" }
        dotnet publish Cide.Client.Maui/Cide.Client.Maui.csproj `
            -f net10.0-android `
            -c $Configuration `
            -o $androidDir `
            --self-contained false
        if ($LASTEXITCODE -ne 0) { throw "Android frontend build failed" }
    }
    finally {
        Pop-Location
    }

    Write-Host "Android artifacts collected in: $androidDir" -ForegroundColor Green

    Write-Header "Building MAUI Android Frontend (New)"

    Push-Location $root
    try {
        dotnet restore Cide.slnx
        if ($LASTEXITCODE -ne 0) { throw "dotnet restore failed" }
        dotnet publish Cide.Client.Maui/Cide.Client.Maui.csproj `
            -f net10.0-android `
            -c $Configuration `
            -p:AndroidPackageFormat=apk `
            -o "$androidDir/maui"
        if ($LASTEXITCODE -ne 0) { throw "MAUI Android frontend build failed" }
    }
    finally {
        Pop-Location
    }

    Write-Host "MAUI Android artifacts collected in: $androidDir/maui" -ForegroundColor Green
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
        Write-Error "Executable not found: $exe"
    }
}

Write-Header "Build Complete"
