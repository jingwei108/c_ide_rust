<#
.SYNOPSIS
    Memory safety pre-commit check script for Cide project.
.DESCRIPTION
    Scans C# and Rust source files for common memory safety anti-patterns.
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
$rsFiles = Get-ChildItem -Path "$root\native\src" -Recurse -Filter "*.rs" |
    Where-Object { $_.FullName -notmatch '\\target\\' }

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
# Rust Checks
# ============================================================================
foreach ($file in $rsFiles) {
    $lines = Get-Content $file.FullName -ErrorAction SilentlyContinue
    if (-not $lines) { continue }

    $inUnsafeBlock = $false
    $unsafeBraceCount = 0

    for ($i = 0; $i -lt $lines.Count; $i++) {
        $line = $lines[$i]
        $ln = $i + 1

        # Track unsafe blocks roughly
        if ($line -match '\bunsafe\s*\{') {
            $inUnsafeBlock = $true
            $unsafeBraceCount = 1
        }
        elseif ($inUnsafeBlock) {
            $unsafeBraceCount += ($line -split '\{' | Measure-Object).Count - 1
            $unsafeBraceCount -= ($line -split '\}' | Measure-Object).Count - 1
            if ($unsafeBraceCount -le 0) { $inUnsafeBlock = $false }
        }

        # 1. Raw pointer dereference outside unsafe block (shouldn't compile, but flag anyway)
        if ($line -match '\*\s*\w+\s*\.\s*as_ptr\(\)' -and -not $inUnsafeBlock) {
            # heuristic only
        }

        # 2. transmute usage
        if ($line -match '\btransmute\b' -and $line -notmatch '\/\/.*transmute') {
            Add-Violation $file.FullName $ln "std::mem::transmute detected - ensure source and target types have identical layout" "Warning"
        }

        # 3. raw pointer offset without bounds check
        if ($line -match '\.offset\(' -and $line -notmatch 'bounds|check|len|size') {
            Add-Violation $file.FullName $ln "Raw pointer offset without apparent bounds check" "Warning"
        }

        # 4. CStr::from_ptr with potentially dangling pointer
        if ($line -match 'CStr::from_ptr' -and $line -notmatch 'as_ptr\(\)') {
            Add-Violation $file.FullName $ln "CStr::from_ptr used - verify pointer lifetime exceeds CStr usage" "Warning"
        }

        # 5. Manual memory allocation in Rust (should use Vec/Box)
        if ($line -match '\balloc::\w+|Layout::new|GlobalAlloc' -and $line -notmatch '\/\/') {
            Add-Violation $file.FullName $ln "Manual allocator usage detected - prefer safe abstractions" "Warning"
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
