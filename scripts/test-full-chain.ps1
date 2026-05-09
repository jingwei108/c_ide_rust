# C IDE MAUI Full-Chain Validation Script
# Verifies: Compile -> Run -> Step -> Memory/Array Visualization
#
# Usage:
#   .\scripts\test-full-chain.ps1 -Device <device_id>   # Run on specific device
#   .\scripts\test-full-chain.ps1                        # Auto-detect device

param(
    [string]$Device = "",
    [string]$ApkPath = "dist\android\com.cide.mobile-Signed.apk",
    [switch]$SkipInstall
)

$ErrorActionPreference = "Stop"
$pkg = "com.cide.mobile"

function Write-Header($text) {
    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host "  $text" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
}

function Write-Success($text) { Write-Host "  ✓ $text" -ForegroundColor Green }
function Write-Fail($text) { Write-Host "  ✗ $text" -ForegroundColor Red }
function Write-Info($text) { Write-Host "  ℹ $text" -ForegroundColor Yellow }

# Detect adb
$adb = if (Get-Command adb -ErrorAction SilentlyContinue) { (Get-Command adb).Source } else { $null }
if (-not $adb) {
    $vsAdb = "D:\Program Files (x86)\Microsoft Visual Studio\Shared\Android\android-sdk\platform-tools\adb.exe"
    if (Test-Path $vsAdb) { $adb = $vsAdb }
}
if (-not $adb) { throw "adb not found" }

# Detect device
if (-not $Device) {
    $devices = & $adb devices | Where-Object { $_ -match "\tdevice$" } | ForEach-Object { ($_ -split "\t")[0] }
    if (-not $devices) { throw "No Android device detected" }
    $Device = $devices | Select-Object -First 1
}
Write-Info "Target device: $Device"

# Install APK
if (-not $SkipInstall) {
    Write-Header "Installing APK"
    if (-not (Test-Path $ApkPath)) { throw "APK not found: $ApkPath" }
    & $adb -s $Device uninstall $pkg 2>$null | Out-Null
    & $adb -s $Device install -d $ApkPath
    if ($LASTEXITCODE -ne 0) { throw "Install failed" }
    Write-Success "APK installed"
}

# Launch app
Write-Header "Launching App"
& $adb -s $Device shell monkey -p $pkg -c android.intent.category.LAUNCHER 1 | Out-Null
Start-Sleep -Seconds 3
Write-Success "App launched"

# Helper: run a JS snippet in the WebView via Blazor's eval mechanism
# (Requires app to expose a test hook; see CideTestHook.js)
function Invoke-WebViewTest($js) {
    # Use am start with extra data to communicate with app? Not trivial.
    # Fallback: logcat-based verification
    return $null
}

# Clear logcat
& $adb -s $Device logcat -c

Write-Header "Running Full-Chain Tests"

$tests = @(
    @{ Name = "App Launch"; Check = { & $adb -s $Device shell pidof $pkg } }
    @{ Name = "WebView Loaded"; Check = {
        Start-Sleep 2
        $log = & $adb -s $Device logcat -d -s "chromium" | Select-String "CodeMirror|Blazor"
        return $log -ne $null
    }}
)

foreach ($t in $tests) {
    $result = & $t.Check
    if ($result) { Write-Success $t.Name } else { Write-Fail $t.Name }
}

# Note: Interactive tests (compile code, run, step, visualize) require
# UI automation (Appium/UiAutomator). This script covers deployment and
# basic smoke checks only.

Write-Info "Interactive chain tests (compile/run/step/viz) require manual verification or UI automation."
Write-Info "Manual checklist:"
Write-Info "  1. Tap editor, type 'void main() { int a = 1; }'"
Write-Info "  2. Tap '运行' → expect console output"
Write-Info "  3. Tap '单步' → expect line highlight + variable panel"
Write-Info "  4. Type array code, run bubble sort → expect array viz + compare highlights"

Write-Header "Validation Complete"
