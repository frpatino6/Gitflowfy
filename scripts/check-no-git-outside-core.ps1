#!/usr/bin/env pwsh
# Gate G4: No git spawn outside crates/core

param(
    [string]$Root = (Get-Location).Path
)

$CorePath = Join-Path $Root "crates\core"
$Errors = @()

function Add-Error {
    param($File, $LineNum, $Msg)
    $Errors += ("RUST: {0}:{1}: {2}" -f $File.FullName, $LineNum, $Msg)
}

# Check Rust files outside crates/core
$RustFiles = Get-ChildItem -Path $Root -Recurse -Filter "*.rs" -File | Where-Object {
    $_.FullName -notlike "*\crates\core\*"
}

foreach ($File in $RustFiles) {
    $Content = Get-Content $File.FullName -Raw
    $LineNum = 0
    foreach ($Line in $Content -split "`n") {
        $LineNum++
        if ($Line -match 'Command::new\s*\(\s*["'']git["'']') {
            $Errors += ("RUST: {0}:{1}: git spawn via Command::new" -f $File.FullName, $LineNum)
        }
        if ($Line -match 'process\.spawn\(') {
            $Errors += ("RUST: {0}:{1}: process.spawn" -f $File.FullName, $LineNum)
        }
        if ($Line -match 'exec\s*\(\s*["'']git["'']') {
            $Errors += ("RUST: {0}:{1}: git exec" -f $File.FullName, $LineNum)
        }
    }
}

# Check JS/TS files
$JsFiles = Get-ChildItem -Path $Root -Recurse -Include "*.js","*.ts","*.mjs" -File | Where-Object {
    $_.FullName -notlike "*\crates\core\*" -and $_.FullName -notlike "*\node_modules\*" -and $_.FullName -notlike "*\spike\*"
}

foreach ($File in $JsFiles) {
    $Content = Get-Content $File.FullName -Raw
    $LineNum = 0
    foreach ($Line in $Content -split "`n") {
        $LineNum++
        if ($Line -match 'spawn\s*\(\s*["'']git["'']') {
            $Errors += ("JS: {0}:{1}: git spawn" -f $File.FullName, $LineNum)
        }
        if ($Line -match 'exec\s*\(\s*["'']git["'']') {
            $Errors += ("JS: {0}:{1}: git exec" -f $File.FullName, $LineNum)
        }
    }
}

if ($Errors.Count -gt 0) {
    Write-Error "Gate G4 FAILED - Git spawns found outside crates/core:"
    $Errors | ForEach-Object { Write-Error $_ }
    exit 1
} else {
    Write-Host "Gate G4 PASSED - No git spawns outside crates/core"
    exit 0
}