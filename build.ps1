$env:CARGO_TARGET_DIR = "D:\cargo-build"
$env:CARGO_HTTP_CHECK_REVOKE = "false"
$env:TMP = "D:\cargo-tmp"
$env:TEMP = "D:\cargo-tmp"

Write-Host "=== 构建前端 ===" -ForegroundColor Cyan
Set-Location "$PSScriptRoot"
npm run build 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "前端构建失败！" -ForegroundColor Red
    Read-Host "按任意键退出"
    exit 1
}

Write-Host "`n=== 打包 Tauri ===" -ForegroundColor Cyan
Set-Location "$PSScriptRoot\src-tauri"
cargo tauri build 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "`n=== 打包完成 ===" -ForegroundColor Green
    $exe = Get-ChildItem -Path "D:\cargo-build\release\yoshunko-admin.exe" -ErrorAction SilentlyContinue
    if (-not $exe) {
        $exe = Get-ChildItem -Path "$PSScriptRoot\src-tauri\target\release\yoshunko-admin.exe" -ErrorAction SilentlyContinue
    }
    if ($exe) {
        Write-Host "exe: $($exe.FullName)" -ForegroundColor Green
        Write-Host "`n启动应用..." -ForegroundColor Cyan
        Start-Process $exe.FullName
    }
} else {
    Write-Host "`n打包失败！" -ForegroundColor Red
}
Write-Host "`n按任意键退出..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
