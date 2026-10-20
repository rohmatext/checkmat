@echo off
cd /d %~dp0

echo [1/3] Build frontend Svelte...
pushd frontend-svelte
call npm run build
if errorlevel 1 ( echo GAGAL di frontend & popd & pause & exit /b 1 )
popd

echo [2/3] Generate ikon...
py gen_icons.py
if errorlevel 1 ( echo GAGAL di gen_icons & pause & exit /b 1 )

echo [3/3] Build EXE...
cd src-tauri
cargo build --release
if errorlevel 1 ( echo GAGAL di cargo & pause & exit /b 1 )

echo.
echo SUKSES: %~dp0src-tauri\target\x86_64-pc-windows-gnu\release\taskmini.exe
pause
