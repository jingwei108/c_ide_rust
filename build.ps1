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
# Native Backend (C++)
# ============================================================================
if ($Target -eq "Desktop" -or $Target -eq "All") {
    Write-Header "Building Native Backend (Desktop)"

    $buildDir = Join-Path $root "native/build"
    New-Item -ItemType Directory -Path $buildDir -Force | Out-Null

    Push-Location $buildDir
    try {
        $clangRoot = "C:/Clang/clang+llvm-22.1.4-x86_64-pc-windows-msvc"
        $clangBin = "$clangRoot/bin"

        $cmakeArgs = @("..")

        # Generator selection
        $hasNinja = Get-Command "ninja" -ErrorAction SilentlyContinue
        $hasVS = $env:VisualStudioVersion -or (Get-Command "msbuild" -ErrorAction SilentlyContinue)

        switch ($Compiler) {
            "Clang" {
                if (-not $hasNinja) {
                    Write-Warning "Ninja not found. Falling back to MinGW Makefiles for Clang."
                }
                $cmakeArgs += @("-G", "Ninja")
                $cmakeArgs += "-DCMAKE_C_COMPILER=$clangBin/clang.exe"
                $cmakeArgs += "-DCMAKE_CXX_COMPILER=$clangBin/clang++.exe"
                $cmakeArgs += "-DCMAKE_RC_COMPILER=$clangBin/llvm-rc.exe"
            }
            "ClangCL" {
                if (-not $hasNinja) {
                    Write-Warning "Ninja not found. Falling back to MinGW Makefiles for ClangCL."
                }
                $cmakeArgs += @("-G", "Ninja")
                $cmakeArgs += "-DCMAKE_C_COMPILER=$clangBin/clang-cl.exe"
                $cmakeArgs += "-DCMAKE_CXX_COMPILER=$clangBin/clang-cl.exe"
            }
            "MSVC" {
                if (-not $hasVS) {
                    throw "MSVC selected but Visual Studio / MSBuild not found."
                }
                # Let CMake auto-select Visual Studio generator
            }
            "MinGW" {
                $cmakeArgs += @("-G", "MinGW Makefiles")
            }
            default {
                if ($hasNinja) {
                    $cmakeArgs += @("-G", "Ninja")
                } elseif (-not $hasVS) {
                    $cmakeArgs += @("-G", "MinGW Makefiles")
                }
            }
        }

        $cmakeArgs += "-DCMAKE_BUILD_TYPE=$Configuration"
        & cmake @cmakeArgs
        if ($LASTEXITCODE -ne 0) { throw "CMake configuration failed" }
        cmake --build . --config $Configuration --parallel
        if ($LASTEXITCODE -ne 0) { throw "Build failed" }
    }
    catch {
        Write-Error "Native backend build failed: $_"
    }
    finally {
        Pop-Location
    }

    # Copy native DLL to unified dist directory
    # Try multiple possible output paths (MSVC multi-config vs MinGW single-config)
    $dllCandidates = @(
        (Join-Path $root "native/build/bin/$Configuration/cide_native.dll"),
        (Join-Path $root "native/build/bin/cide_native.dll"),
        (Join-Path $root "native/build/libcide_native.dll")
    )
    $dllSource = $null
    foreach ($c in $dllCandidates) {
        if (Test-Path $c) {
            $dllSource = $c
            break
        }
    }
    if ($dllSource) {
        New-Item -ItemType Directory -Path $desktopDir -Force | Out-Null
        Copy-Item $dllSource (Join-Path $desktopDir "cide_native.dll") -Force
        Write-Host "Copied cide_native.dll -> dist/desktop/" -ForegroundColor Green
    } else {
        Write-Warning "cide_native.dll not found in any expected location: $dllCandidates"
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
        $abis = @("arm64-v8a", "armeabi-v7a")
        foreach ($abi in $abis) {
            $buildDir = Join-Path $root "native/build-android-$abi"
            New-Item -ItemType Directory -Path $buildDir -Force | Out-Null

            Push-Location $buildDir
            try {
                $toolchain = Join-Path $ndkHome "build/cmake/android.toolchain.cmake"
                if (-not (Test-Path $toolchain)) {
                    throw "Android toolchain not found: $toolchain"
                }

                $cmakeArgs = @(
                    "..",
                    "-G", "Ninja",
                    "-DCMAKE_TOOLCHAIN_FILE=$toolchain",
                    "-DANDROID_ABI=$abi",
                    "-DANDROID_PLATFORM=android-21",
                    "-DCMAKE_BUILD_TYPE=$Configuration",
                    "-DCIDE_BUILD_TESTS=OFF"
                )
                & cmake @cmakeArgs
                if ($LASTEXITCODE -ne 0) { throw "CMake configuration failed for Android $abi" }

                cmake --build . --config $Configuration --parallel
                if ($LASTEXITCODE -ne 0) { throw "Build failed for Android $abi" }

                # Copy .so into Android project for packaging
                $soSource = Join-Path $buildDir "lib/libcide_native.so"
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
            catch {
                Write-Error "Native Android build ($abi) failed: $_"
            }
            finally {
                Pop-Location
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
