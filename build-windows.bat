@echo off
chcp 65001 >nul
rem WordCraft AR — build on Windows (one-time setup: install Rust from https://rustup.rs, MSVC Build Tools
rem are installed by rustup's prompt). Output: target\release\wordcraft.exe
setlocal
cd /d "%~dp0"
where cargo >nul 2>nul || (
  echo [WordCraft AR] Rust is not installed. Install it from https://rustup.rs then run this file again.
  echo [WordCraft AR] راست غير مثبّت. ثبّته من https://rustup.rs ثم شغّل هذا الملف مرة أخرى.
  pause & exit /b 1
)
echo [WordCraft AR] building the release binary (first build takes 10-25 minutes)...
cargo build --release -p wordcraft || (echo build failed & pause & exit /b 1)
if not exist dist mkdir dist
copy /y target\release\wordcraft.exe "dist\WordCraft AR.exe" >nul
if exist ..\servicewordar xcopy /e /i /y ..\servicewordar dist\servicewordar >nul
echo.
echo [WordCraft AR] done: dist\WordCraft AR.exe
echo [WordCraft AR] تم: dist\WordCraft AR.exe
pause
