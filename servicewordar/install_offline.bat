@echo off
chcp 65001 >nul
cd /d "%~dp0"
echo تثبيت المكتبة المطلوبة (lxml) من مجلد wheels بلا إنترنت...
where py >nul 2>nul && (py -3 -m pip install --no-index --find-links wheels lxml uharfbuzz fonttools pypdf reportlab pillow & goto :done)
python -m pip install --no-index --find-links wheels lxml uharfbuzz fonttools pypdf reportlab pillow
:done
echo.
echo للتحقق:  py -3 tests\run_tests.py
pause
