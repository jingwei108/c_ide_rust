# Safe test runner - kills process on timeout or excessive memory
param(
    [Parameter(Mandatory=$true)]
    [string]$ExePath,
    
    [int]$TimeoutSeconds = 10,
    [int]$MaxMemoryMB = 500,
    [string[]]$Arguments = @()
)

$proc = $null
try {
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $ExePath
    $psi.Arguments = $Arguments -join " "
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.WorkingDirectory = (Get-Location)
    
    $proc = [System.Diagnostics.Process]::Start($psi)
    $startTime = Get-Date
    
    while (-not $proc.HasExited) {
        Start-Sleep -Milliseconds 200
        $elapsed = (Get-Date) - $startTime
        
        if ($elapsed.TotalSeconds -gt $TimeoutSeconds) {
            Write-Error "TIMEOUT: Process killed after $TimeoutSeconds seconds"
            $proc.Kill()
            $proc.WaitForExit(1000)
            exit 124
        }
        
        try {
            $memMB = [math]::Round($proc.WorkingSet64 / 1MB, 1)
            if ($memMB -gt $MaxMemoryMB) {
                Write-Error "MEMORY LIMIT: Process killed after using ${memMB} MB (limit: ${MaxMemoryMB} MB)"
                $proc.Kill()
                $proc.WaitForExit(1000)
                exit 137
            }
        } catch {}
    }
    
    $stdout = $proc.StandardOutput.ReadToEnd()
    $stderr = $proc.StandardError.ReadToEnd()
    if ($stdout) { Write-Host $stdout }
    if ($stderr) { Write-Host $stderr }
    exit $proc.ExitCode
    
} finally {
    if ($proc -and -not $proc.HasExited) {
        try { $proc.Kill() } catch {}
    }
}
