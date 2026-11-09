# Density/scalability benchmark for the raw renderer.
#
# Generates a deterministic rolling-hill world spanning `(2*Radius+1)` chunks on
# a side (via `SUNSET_WORLD_RADIUS`) and measures the renderer for `Secs`
# seconds after a 2 s warm-up (via `SUNSET_BENCH_SECS`), then exits.
#
# Usage:
#   powershell -File bench_dense.ps1 -Radius 32 [-Secs 8]
#
# Uses the release binary (faster single-threaded meshing + representative FPS).
param([int]$Radius, [float]$Secs = 8)

$exe = 'C:\Users\Triss\Desktop\SunsetV2\target\release\sunset.exe'
$out = 'C:\Users\Triss\Desktop\SunsetV2\bench_out.txt'

$env:SUNSET_WORLD_RADIUS = "$Radius"
$env:SUNSET_BENCH_SECS = "$Secs"

$sw = [System.Diagnostics.Stopwatch]::StartNew()
& $exe *> $out
$sw.Stop()

Write-Output ("[total {0}s]" -f [math]::Round($sw.Elapsed.TotalSeconds, 1))
Select-String -Path $out -Pattern 'chunks:|BENCH_RESULT' | ForEach-Object { $_.Line }
