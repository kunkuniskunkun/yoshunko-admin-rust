$ErrorActionPreference = "Stop"
$exe = "D:\cargo-build\release\yoshunko-admin.exe"
$tmpDir = "D:\cargo-tmp"
$buildDir = "D:\cargo-build"

# Kill existing process to avoid "exe locked" build error
$running = Get-Process -Name "yoshunko-admin" -ErrorAction SilentlyContinue
if ($running) {
    Write-Host "Stopping yoshunko-admin..." -ForegroundColor Yellow
    $running | Stop-Process -Force
    Start-Sleep -Milliseconds 500
}

# Ensure directories exist
foreach ($d in @($tmpDir, $buildDir)) {
    if (!(Test-Path $d)) { New-Item -ItemType Directory -Path $d -Force | Out-Null }
}

$env:TMP = $tmpDir
$env:TEMP = $tmpDir
$env:CARGO_TARGET_DIR = $buildDir
$env:CARGO_HTTP_CHECK_REVOKE = "false"

$version = (Get-Content "$PSScriptRoot\src-tauri\tauri.conf.json" | ConvertFrom-Json).version
Write-Host "Building Yoshunko Admin v$version..." -ForegroundColor Cyan

Set-Location "$PSScriptRoot\src-tauri"
$sw = [System.Diagnostics.Stopwatch]::StartNew()
cargo tauri build 2>&1
$sw.Stop()

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build OK! (${0:N1}s)" -f $sw.Elapsed.TotalSeconds -ForegroundColor Green
    if (Test-Path $exe) {
        Start-Process $exe
    } else {
        Write-Host "exe not found: $exe" -ForegroundColor Red
    }
} else {
    Write-Host "Build FAILED! (${0:N1}s)" -f $sw.Elapsed.TotalSeconds -ForegroundColor Red
}

Write-Host "Press any key to exit..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
