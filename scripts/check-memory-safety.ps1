<#
.SYNOPSIS
    Memory safety pre-commit check script for Cide project.
.DESCRIPTION
    Scans C# and C++ source files for common memory safety anti-patterns.
    Returns exit code 0 if clean, 1 if violations found.
    Run manually: .\scripts\check-memory-safety.ps1
    Or install as git hook:
        copy scripts\check-memory-safety.ps1 .git\hooks\pre-commit.ps1
#>

$ErrorActionPreference = 'Stop'
$violations = @()
$root = Split-Path $PSScriptRoot -Parent

function Add-Violation($file, $line, $message, $severity) {
    $script:violations += New-Object PSObject -Property @{
        File     = $file.Substring($root.Length + 1)
        Line     = $line
        Message  = $message
        Severity = $severity
    }
}

# ============================================================================
# C# Checks
# ============================================================================
$csFiles = Get-ChildItem -Path "$root\Cide.Client" -Recurse -Filter "*.cs" |
    Where-Object { $_.FullName -notmatch '\\obj\\' -and $_.FullName -notmatch '\\bin\\' }

foreach ($file in $csFiles) {
    $lines = Get-Content $file.FullName -ErrorAction SilentlyContinue
    if (-not $lines) { continue }

    for ($i = 0; $i -lt $lines.Count; $i++) {
        $line = $lines[$i]
        $ln = $i + 1

        # 1. Event subscription without corresponding -= (heuristic)
        if ($line -match '\+\=\s*\(?\s*(\w+|\(s\s*,\s*e\))' -and
            $line -notmatch 'Loaded\s*\+\=|Unloaded\s*\+\=' -and
            $line -notmatch '\/\/.*\+\=') {
            # 简单启发式：如果文件中没有对应的 -=，报 warning
            $eventName = if ($line -match '(\w+)\s*\+\=') { $matches[1] } else { "unknown" }
            $content = $lines -join "`n"
            if (-not ($content -match [regex]::Escape($eventName) + '\s*\-\=')) {
                Add-Violation $file.FullName $ln "Event '$eventName' subscribed with += but no -= found in file" "Warning"
            }
        }

        # 2. async void (fire-and-forget without cancellation)
        if ($line -match 'async\s+void\s+\w+') {
            Add-Violation $file.FullName $ln "async void detected - ensure CancellationToken is used to prevent UAF after disposal" "Warning"
        }

        # 3. new SolidColorBrush / new Pen in non-static context (hot path)
        if ($line -match 'new\s+SolidColorBrush|new\s+Pen\s*\(' -and
            $line -notmatch 'static\s+readonly|private\s+static|CreateBrush') {
            Add-Violation $file.FullName $ln "new SolidColorBrush/Pen in instance method - consider static caching to reduce GC pressure" "Suggestion"
        }

        # 4. IntPtr field without IDisposable
        if ($line -match 'private\s+IntPtr\s+\w+') {
            $content = $lines -join "`n"
            if (-not ($content -match 'IDisposable')) {
                Add-Violation $file.FullName $ln "IntPtr field detected but class does not implement IDisposable" "Warning"
            }
        }

        # 5. Task.Delay without CancellationToken
        if ($line -match 'Task\.Delay\s*\([^,]+\)' -and $line -notmatch 'CancellationToken') {
            Add-Violation $file.FullName $ln "Task.Delay without CancellationToken - may access disposed resources after delay" "Warning"
        }
    }
}

# ============================================================================
# C++ Checks
# ============================================================================
$cppFiles = Get-ChildItem -Path "$root\native\src" -Recurse -Filter "*.cpp" |
    ForEach-Object { $_ }
$hppFiles = Get-ChildItem -Path "$root\native\src" -Recurse -Filter "*.hpp" |
    ForEach-Object { $_ }

$allCpp = $cppFiles + $hppFiles

foreach ($file in $allCpp) {
    $lines = Get-Content $file.FullName -ErrorAction SilentlyContinue
    if (-not $lines) { continue }

    for ($i = 0; $i -lt $lines.Count; $i++) {
        $line = $lines[$i]
        $ln = $i + 1

        # 1. c_str() returned from C API function
        if ($line -match 'extern\s+"C".*\bchar\s*\*\b.*\w+\s*\(' -and
            $line -notmatch '_buf\s*\(') {
            # 检查函数体内是否有 c_str() return
            $funcStart = $i
            $braceCount = 0
            $foundOpen = $false
            for ($j = $funcStart; $j -lt $lines.Count -and $j -lt $funcStart + 50; $j++) {
                $l = $lines[$j]
                if ($l -match '\{') { $foundOpen = $true; $braceCount += ($l -split '\{' | Measure-Object).Count - 1 }
                if ($l -match '\}') { $braceCount -= ($l -split '\}' | Measure-Object).Count - 1 }
                if ($foundOpen -and $l -match 'return.*\.c_str\(\)') {
                    Add-Violation $file.FullName ($j + 1) "C API returns c_str() - dangling pointer risk. Use copy-out buffer pattern instead." "Error"
                    break
                }
                if ($foundOpen -and $braceCount -le 0) { break }
            }
        }

        # 2. malloc/new without corresponding free/delete in same file
        if ($line -match '\bnew\s+\w+' -and $line -notmatch '\/\/.*new') {
            $type = if ($line -match 'new\s+(\w+)') { $matches[1] } else { "unknown" }
            $content = $lines -join "`n"
            if (-not ($content -match 'delete\s+' + [regex]::Escape($type)) -and
                -not ($content -match 'delete\s+\w+')) {
                Add-Violation $file.FullName $ln "new $type without matching delete in same file" "Warning"
            }
        }

        # 3. raw pointer arithmetic on memory buffer
        if ($line -match 'mem\[.*\]\s*=' -or $line -match '=\s*mem\[') {
            Add-Violation $file.FullName $ln "Raw array access on memory buffer - use LoadI32/StoreI32/StoreI8 wrappers instead" "Warning"
        }
    }
}

# ============================================================================
# Report
# ============================================================================
if ($violations.Count -eq 0) {
    Write-Host "[PASS] No memory safety violations detected." -ForegroundColor Green
    exit 0
}

$errors = $violations | Where-Object Severity -eq 'Error'
$warnings = $violations | Where-Object Severity -eq 'Warning'
$suggestions = $violations | Where-Object Severity -eq 'Suggestion'

Write-Host ""
Write-Host "Memory Safety Check Results" -ForegroundColor Yellow
Write-Host "===========================" -ForegroundColor Yellow
Write-Host ""

foreach ($v in $violations) {
    $color = switch ($v.Severity) {
        'Error'       { 'Red' }
        'Warning'     { 'Yellow' }
        'Suggestion'  { 'Cyan' }
        default       { 'White' }
    }
    Write-Host "$($v.Severity): $($v.File):$($v.Line)" -ForegroundColor $color -NoNewline
    Write-Host " -> $($v.Message)"
}

Write-Host ""
Write-Host "Summary: $($errors.Count) Error(s), $($warnings.Count) Warning(s), $($suggestions.Count) Suggestion(s)" -ForegroundColor Yellow

if ($errors.Count -gt 0) {
    Write-Host ""
    Write-Host "Commit blocked due to Error-level violations. Fix them or use --no-verify to bypass (not recommended)." -ForegroundColor Red
    exit 1
}

exit 0
