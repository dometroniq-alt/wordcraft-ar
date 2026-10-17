@echo off
chcp 65001 >nul
cd /d "%~dp0"
where py >nul 2>nul && (py -3 server.py & goto :eof)
where python >nul 2>nul && (python server.py & goto :eof)
echo Python غير مثبت. ثبّت Python 3.10 أو أحدث من python.org ثم شغّل install_offline.bat
pause
