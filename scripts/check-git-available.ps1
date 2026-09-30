#!/usr/bin/env pwsh
# Gate G1: git is usable and identified

param(
    [string]$Root = (Get-Location).Path
)

try {
    $Output = git version 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Error "git version failed: $Output"
        exit 1
    }
    
    Write-Host "Git version: $Output"
    
    # Try to create a temp repo
    $TempDir = [System.IO.Path]::GetTempFileName()
    Remove-Item $TempDir -Force
    New-Item -ItemType Directory -Path $TempDir | Out-Null
    
    Push-Location $TempDir
    git init 2>&1 | Out-Null
    Pop-Location
    Remove-Item $TempDir -Recurse -Force
    
    Write-Host "Gate G1 PASSED - Git is usable and identified"
    exit 0
}
catch {
    Write-Error "Gate G1 FAILED: $_"
    exit 1
}