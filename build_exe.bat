@echo off
cd /d %~dp0

echo [1/2] Build frontend Svelte...
pushd frontend-svelte
call npm run build
if errorlevel 1 ( echo GAGAL di frontend & popd & pause & exit /b 1 )
popd

echo [2/2] Build EXE...
cd src-tauri
cargo build --release
if errorlevel 1 ( echo GAGAL di cargo & pause & exit /b 1 )

echo.
echo SUKSES: %~dp0src-tauri\target\x86_64-pc-windows-gnu\release\checkmat.exe
pause
