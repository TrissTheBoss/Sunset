# Density/scalability + scenario benchmark for the raw renderer (M12 baseline).
#
# Usage:
#   powershell -File bench_dense.ps1 [-Radius N] [-Secs 8] [-Spin] [-Edit]
#
# -Radius N  generates a rolling-hill world spanning (2N+1) chunks (default:
#            the authored building-yard fixture).
# -Spin      rotates the camera continuously during measurement.
# -Edit      toggles a block periodically during measurement.
#
# Uses the release binary. Env vars are set per run and cleared between runs.
param(
    [int]$Radius = -1,
    [float]$Secs = 8,
    [switch]$Spin,
    [switch]$Edit
)

$exe = 'C:\Users\Triss\Desktop\SunsetV2\target\release\sunset.exe'
$out = 'C:\Users\Triss\Desktop\SunsetV2\bench_out.txt'

Remove-Item Env:SUNSET_WORLD_RADIUS, Env:SUNSET_BENCH_SPIN, Env:SUNSET_BENCH_EDIT -ErrorAction SilentlyContinue
if ($Radius -ge 0) { $env:SUNSET_WORLD_RADIUS = "$Radius" }
if ($Spin) { $env:SUNSET_BENCH_SPIN = "1" }
if ($Edit) { $env:SUNSET_BENCH_EDIT = "1" }
$env:SUNSET_BENCH_SECS = "$Secs"

$sw = [System.Diagnostics.Stopwatch]::StartNew()
& $exe *> $out
$sw.Stop()

Write-Output ("[total {0}s] radius={1} spin={2} edit={3}" -f [math]::Round($sw.Elapsed.TotalSeconds, 1), $Radius, [bool]$Spin, [bool]$Edit)
Select-String -Path $out -Pattern 'chunks:|BENCH_RESULT' | ForEach-Object { $_.Line }
