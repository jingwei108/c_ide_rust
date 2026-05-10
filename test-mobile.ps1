# C IDE Mobile Test Script
# Builds native .so libraries, packages the Android APK, and optionally
# installs / runs / captures logs on a connected device or emulator.
#
# Usage:
#   .\test-mobile.ps1                    # Full build (native + APK)
#   .\test-mobile.ps1 -Install -Run      # Build, install APK, and launch app
#   .\test-mobile.ps1 -Run -Logcat       # Build, install, launch, then tail logs
#   .\test-mobile.ps1 -SkipNativeBuild   # Only build APK (reuse existing .so)

param(
    [ValidateSet("Debug", "Release")]
    [string]$Configuration = "Debug",

    [switch]$SkipNativeBuild,
    [switch]$Install,
    [switch]$Run,
    [switch]$Logcat
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
function Write-ErrorColored($text) { Write-Host $text -ForegroundColor Red }

# ============================================================================
# Auto-detect Android SDK / NDK
# ============================================================================
$ndkHome = $env:ANDROID_NDK_HOME
if (-not $ndkHome) { $ndkHome = $env:ANDROID_NDK_ROOT }

$adbPath = $null
if (Get-Command adb -ErrorAction SilentlyContinue) {
    $adbPath = (Get-Command adb).Source
}

# If NDK not set via env, try Visual Studio default paths
$vsAndroidBase = "D:\Program Files (x86)\Microsoft Visual Studio\Shared\Android"
if (-not $ndkHome -and (Test-Path $vsAndroidBase)) {
    $ndkCandidates = Get-ChildItem -Path "$vsAndroidBase\AndroidNDK" -Directory -ErrorAction SilentlyContinue |
        Sort-Object Name -Descending
    if ($ndkCandidates) {
        $ndkHome = $ndkCandidates[0].FullName
        Write-Warn "ANDROID_NDK_HOME not set; auto-detected NDK: $ndkHome"
    }
}

# If adb not found, try VS Android SDK platform-tools
if (-not $adbPath -and (Test-Path "$vsAndroidBase\android-sdk\platform-tools\adb.exe")) {
    $adbPath = "$vsAndroidBase\android-sdk\platform-tools\adb.exe"
    Write-Warn "adb not in PATH; auto-detected: $adbPath"
}

# ============================================================================
# Build Native .so (Rust + cargo-ndk)
# ============================================================================
if (-not $SkipNativeBuild) {
    if (-not $ndkHome) {
        Write-ErrorColored "Android NDK not found. Set ANDROID_NDK_HOME or install VS Android workload."
        exit 1
    }

    Write-Header "Building Native Backend (Android)"

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
            Write-ErrorColored "Native Android build ($abi) failed: $_"
            exit 1
        }
        finally {
            Pop-Location
        }

        $soDir = if ($Configuration -eq "Release") { "release" } else { "debug" }
        $soSource = Join-Path $root "native/target/$rustTarget/$soDir/libcide_native.so"
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
else {
    Write-Warn "Skipping native .so build (--SkipNativeBuild)"
}

# ============================================================================
# Build Android APK (MAUI)
# ============================================================================
Write-Header "Building MAUI Android APK"

$androidDir = Join-Path $root "dist/android"

# Force clean MAUI obj cache so updated .so files are re-packaged
$mauiObjDir = Join-Path $root "Cide.Client.Maui/obj"
if (Test-Path $mauiObjDir) {
    Write-Warn "Cleaning MAUI build cache to ensure fresh .so packaging..."
    Remove-Item -Recurse -Force $mauiObjDir
}
if (Test-Path $androidDir) {
    Remove-Item -Recurse -Force $androidDir
}

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
    if ($LASTEXITCODE -ne 0) { throw "APK build failed" }
}
catch {
    Write-ErrorColored "Android frontend build failed: $_"
    exit 1
}
finally {
    Pop-Location
}

# Locate the signed APK (prefer MAUI package)
$apk = Get-ChildItem -Path $androidDir -Filter "com.cide.mobile-Signed.apk" | Select-Object -First 1
if (-not $apk) {
    $apk = Get-ChildItem -Path $androidDir -Filter "*Signed.apk" | Select-Object -First 1
}
if (-not $apk) {
    $apk = Get-ChildItem -Path $androidDir -Filter "*.apk" | Select-Object -First 1
}
if (-not $apk) {
    Write-ErrorColored "No APK found in $androidDir"
    exit 1
}
Write-Success "APK built: $($apk.FullName) ($([math]::Round($apk.Length/1MB,2)) MB)"

# ============================================================================
# Device / Emulator detection
# ============================================================================
if ($Install -or $Run -or $Logcat) {
    if (-not $adbPath) {
        Write-ErrorColored "adb not found. Cannot install/run on device."
        exit 1
    }

    Write-Header "Detecting Android Devices"

    # Retry logic: USB connections can be flaky; auto-recover via kill-server/start-server
    $maxRetries = 3
    $devices = $null
    for ($retry = 1; $retry -le $maxRetries; $retry++) {
        $rawOutput = & $adbPath devices
        $devices = $rawOutput | Where-Object {
            $line = $_.Trim()
            $line -and $line -notmatch "List of devices" -and ($line -split "\s+")[-1] -eq "device"
        } | ForEach-Object {
            ($_ -split "\s+")[0].Trim()
        }

        if ($devices) { break }

        $offlineDevices = $rawOutput | Where-Object {
            $line = $_.Trim()
            $line -and $line -notmatch "List of devices" -and ($line -split "\s+")[-1] -eq "offline"
        }

        if ($offlineDevices) {
            Write-Warn "Device(s) offline. Attempting adb server restart ($retry/$maxRetries)..."
            & $adbPath kill-server | Out-Null
            & $adbPath start-server | Out-Null
            Start-Sleep -Seconds 2
        }
        elseif ($retry -lt $maxRetries) {
            Write-Warn "No device found. Retrying in 3 seconds ($retry/$maxRetries)..."
            Start-Sleep -Seconds 3
        }
    }

    if (-not $devices) {
        Write-ErrorColored "No Android device or emulator detected. Connect a device or start an emulator."
        exit 1
    }

    $device = $devices | Select-Object -First 1
    Write-Success "Target device: $device"
}

# ============================================================================
# Install APK
# ============================================================================
if ($Install -or $Run) {
    Write-Header "Installing APK"
    $packageName = "com.cide.mobile"
    Write-Warn "Uninstalling old version to clear WebView cache..."
    & $adbPath -s $device uninstall $packageName | Out-Null
    # Ignore uninstall exit code (app may not be installed yet)
    $installOutput = & $adbPath -s $device install -d $apk.FullName 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-ErrorColored "APK install output: $installOutput"
        throw "APK installation failed (exit code: $LASTEXITCODE)"
    }
    Write-Success "APK installed successfully"
}

# ============================================================================
# Launch App
# ============================================================================
if ($Run) {
    Write-Header "Launching C IDE (MAUI)"
    $packageName = "com.cide.mobile"
    # Use monkey to launch the main launcher activity without needing the exact Java-style activity name
    & $adbPath -s $device shell monkey -p $packageName -c android.intent.category.LAUNCHER 1 | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "App launch failed" }
    Write-Success "App launched on device"
}

# ============================================================================
# Capture Logcat
# ============================================================================
if ($Logcat) {
    Write-Header "Starting Logcat (Ctrl+C to stop)"
    $packageName = "com.cide.mobile"
    & $adbPath -s $device logcat -c
    $pidRaw = & $adbPath -s $device shell pidof $packageName
    $appPid = ($pidRaw -split '\s+')[0].Trim()
    if ($appPid -and $appPid -match '^\d+$') {
        Write-Host "Filtering logcat for PID: $appPid"
        & $adbPath -s $device logcat --pid=$appPid
    } else {
        Write-Warn "Could not get PID for $packageName, showing unfiltered logcat"
        & $adbPath -s $device logcat -d | Select-Object -Last 100
    }
}

Write-Header "Mobile Test Complete"
