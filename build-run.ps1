$env:TMP = "D:\cargo-tmp"
$env:TEMP = "D:\cargo-tmp"
$env:CARGO_TARGET_DIR = "D:\cargo-build"
$env:CARGO_HTTP_CHECK_REVOKE = "false"

Set-Location "$PSScriptRoot\src-tauri"
Write-Host "Building..." -ForegroundColor Cyan
cargo tauri build 2>&1

if ($LASTEXITCODE -eq 0) {
    Write-Host "`nBuild OK! Launching..." -ForegroundColor Green
    Start-Process "D:\cargo-build\release\yoshunko-admin.exe"
} else {
    Write-Host "`nBuild failed!" -ForegroundColor Red
}
Write-Host "`nPress any key to exit..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
