@echo off
chcp 65001 >nul
cd /d "%~dp0"

set "EXE=%~dp0src-tauri\target\release\novel-reader-core.exe"

if exist "%EXE%" (
  start "" "%EXE%"
  exit /b 0
)

echo 尚未找到已打包程序，正在尝试开发模式启动...
echo （需要先: npm install，且会打开开发者窗口）
echo.

where npm >nul 2>&1
if errorlevel 1 (
  echo [提示] 请先双击「一键打包.bat」生成 exe，或安装 Node.js 后使用开发模式。
  pause
  exit /b 1
)

call npm run tauri dev
