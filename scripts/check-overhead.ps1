#!/usr/bin/env pwsh
# Gate G2: Measure the chosen stack's wrapper overhead
# Measures 200 invocations through GitCommand minus 200 direct git invocations

param(
    [string]$Root = (Get-Location).Path,
    [int]$Iterations = 200
)

$Cargo = "C:\Users\fernando\.cargo\bin\cargo.exe"
$CorePath = Join-Path $Root "crates\core"

# Build release binary
Write-Host "Building release binary..."
& $Cargo build --release --package gitflowfy-core 2>&1 | Out-Null

$Binary = Join-Path $Root "target\release\gitflowfy.exe"
if (-not (Test-Path $Binary)) {
    Write-Error "Binary not found at $Binary"
    exit 1
}

$Repo = Join-Path $Root "test_repo"
if (-not (Test-Path $Repo)) {
    Write-Error "Test repo not found at $Repo"
    exit 1
}

Write-Host "Measuring direct git overhead ($Iterations iterations)..."
$DirectTimes = @()
for ($i = 0; $i -lt $Iterations; $i++) {
    $Start = [System.Diagnostics.Stopwatch]::StartNew()
    & git -C $Repo rev-parse --git-dir 2>&1 | Out-Null
    $DirectTimes += $Start.Elapsed.TotalMilliseconds
}

Write-Host "Measuring wrapper overhead ($Iterations iterations)..."
$WrapperTimes = @()
for ($i = 0; $i -lt $Iterations; $i++) {
    $Start = [System.Diagnostics.Stopwatch]::StartNew()
    & gitflowfy git $Repo -- rev-parse --git-dir 2>&1 | Out-Null
    $WrapperTimes += $Start.Elapsed.TotalMilliseconds
}

# Calculate statistics
function Get-Stats($Times) {
    $Sorted = $Times | Sort-Object
    $Count = $Sorted.Count
    $Median = $Sorted[ [math]::Floor($Count / 2) ]
    $P99Index = [math]::Floor($Count * 0.99)
    $P99 = $Sorted[$P99Index]
    $Mean = ($Sorted | Measure-Object -Average).Average
    return @{ Median = $Median; P99 = $P99; Mean = $Mean }
}

$DirectStats = Get-Stats $DirectTimes
$WrapperStats = Get-Stats $WrapperTimes

# Overhead = wrapper - direct
$OverheadMedian = $WrapperStats.Median - $DirectStats.Median
$OverheadP99 = $WrapperStats.P99 - $DirectStats.P99

Write-Host "Direct git: median=$($DirectStats.Median)ms, p99=$($DirectStats.P99)ms"
Write-Host "Wrapper: median=$($WrapperStats.Median)ms, p99=$($WrapperStats.P99)ms"
Write-Host "Overhead: median=$($OverheadMedian)ms, p99=$($OverheadP99)ms"

$Pass = $OverheadMedian -le 5 -and $OverheadP99 -le 25
if ($Pass) {
    Write-Host "Gate G2 PASSED - Overhead within budget (median <= 5ms, p99 <= 25ms)"
    exit 0
} else {
    Write-Error "Gate G2 FAILED - Overhead exceeds budget (median: $OverheadMedian ms, p99: $OverheadP99 ms)"
    exit 1
}