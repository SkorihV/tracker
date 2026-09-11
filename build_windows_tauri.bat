@echo off
setlocal
chcp 65001 >nul
cd /d "%~dp0"

where node >nul 2>nul || (echo [ERROR] Node.js 18+ not found & exit /b 1)
where cargo >nul 2>nul || (echo [ERROR] Rust (MSVC) not found: install https://rustup.rs & exit /b 1)
where npm >nul 2>nul || (echo [ERROR] npm not found & exit /b 1)

echo [1/4] npm install
call npm install || exit /b 1

echo [2/4] frontend build
call npm run build || exit /b 1

echo [3/4] tauri build (needs VS Build Tools + WebView2)
call npx tauri build || exit /b 1

echo [4/4] DONE: src-tauri\target\release\tracker.exe
pause