@echo off
chcp 65001 >nul
cd /d "%~dp0"

echo ========================================
echo   Novel Reader Core - Release Build
echo ========================================
echo.

where npm >nul 2>&1
if errorlevel 1 (
  echo [错误] 未找到 npm，请先安装 Node.js 18+
  pause
  exit /b 1
)

where cargo >nul 2>&1
if errorlevel 1 (
  echo [错误] 未找到 cargo，请先安装 Rust: https://rustup.rs
  pause
  exit /b 1
)

if not exist "node_modules\" (
  echo [1/2] 安装前端依赖...
  call npm install
  if errorlevel 1 goto :fail
) else (
  echo [1/2] 依赖已存在，跳过 npm install
)

echo.
echo [2/2] 正在打包（首次约 5~15 分钟，请耐心等待）...
call npm run tauri build
if errorlevel 1 goto :fail

echo.
echo ========================================
echo   打包完成
echo ========================================
echo.
echo 免安装直接运行:
echo   src-tauri\target\release\novel-reader-core.exe
echo.
echo Windows 安装包:
echo   src-tauri\target\release\bundle\nsis\*-setup.exe
echo.
echo 可选: 双击「创建桌面快捷方式.bat」为便携 exe 创建桌面入口。
echo.
echo 双击「一键运行.bat」即可启动已打包版本。
echo.
pause
exit /b 0

:fail
echo.
echo [失败] 打包未完成，请检查上方报错。
pause
exit /b 1
