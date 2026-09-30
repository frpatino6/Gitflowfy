#!/usr/bin/env pwsh
# Gate G5: No embedded Git implementation
# cargo tree --all-features must contain no libgit2, git2, gix, or jgit

param(
    [string]$Root = (Get-Location).Path
)

$Cargo = "C:\Users\fernando\.cargo\bin\cargo.exe"
$Output = & $Cargo tree --all-features --prefix=none 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Error "cargo tree failed: $Output"
    exit 1
}

$Forbidden = @("libgit2", "git2", "gix", "jgit")
$Found = @()

foreach ($Line in $Output -split "`n") {
    foreach ($F in $Forbidden) {
        if ($Line -like "*$F*") {
            $Found += "Found forbidden crate: $Line"
        }
    }
}

if ($Found.Count -gt 0) {
    Write-Error "Gate G5 FAILED - Forbidden Git crates found:"
    $Found | ForEach-Object { Write-Error $_ }
    exit 1
} else {
    Write-Host "Gate G5 PASSED - No embedded Git crates"
    exit 0
}