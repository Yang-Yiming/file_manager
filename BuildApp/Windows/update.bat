@echo off
REM 文件管理器 Windows 智能更新脚本
REM 支持快速模式、版本管理等功能

setlocal enabledelayedexpansion

REM 确定项目根目录
set "SCRIPT_DIR=%~dp0"
pushd "%SCRIPT_DIR%..\.."
set "PROJECT_ROOT=%CD%"
popd

cd /d "%PROJECT_ROOT%"

REM 默认参数
set "QUICK_MODE=0"
set "NEW_VERSION="
set "NO_BACKUP=0"
set "SHOW_HELP=0"

REM 解析命令行参数
:parse_args
if "%~1"=="" goto :args_done
if /i "%~1"=="-q" set "QUICK_MODE=1"
if /i "%~1"=="--quick" set "QUICK_MODE=1"
if /i "%~1"=="-v" (
    set "NEW_VERSION=%~2"
    shift
)
if /i "%~1"=="--version" (
    set "NEW_VERSION=%~2"
    shift
)
if /i "%~1"=="-b" set "NO_BACKUP=1"
if /i "%~1"=="--no-backup" set "NO_BACKUP=1"
if /i "%~1"=="-h" set "SHOW_HELP=1"
if /i "%~1"=="--help" set "SHOW_HELP=1"
shift
goto :parse_args

:args_done

REM 显示帮助
if "%SHOW_HELP%"=="1" (
    echo.
    echo 文件管理器 Windows 智能更新工具
    echo.
    echo 用法: update.bat [选项]
    echo.
    echo 选项:
    echo   -q, --quick         快速模式（仅重新编译）
    echo   -v, --version VER   更新版本号到 VER
    echo   -b, --no-backup     跳过备份
    echo   -h, --help          显示此帮助信息
    echo.
    echo 示例:
    echo   update.bat -q                快速重新编译
    echo   update.bat -v 1.0.0          更新到版本 1.0.0
    echo   update.bat -v 1.0.0 -b       更新版本且跳过备份
    echo.
    goto :end
)

echo 🚀 文件管理器智能更新工具 (Windows)
echo ===================================

REM 检查环境
if not exist "Cargo.toml" (
    echo ❌ 无法找到项目根目录（Cargo.toml 不存在）
    exit /b 1
)

where cargo >nul 2>&1
if errorlevel 1 (
    echo ❌ 未找到 Rust/Cargo，请先安装
    exit /b 1
)

REM 获取当前版本号
for /f "tokens=3 delims== " %%a in ('findstr /r "^version" Cargo.toml') do (
    set "CURRENT_VERSION=%%a"
    set "CURRENT_VERSION=!CURRENT_VERSION:"=!"
)

REM 更新版本号（如果需要）
if not "%NEW_VERSION%"=="" (
    echo ℹ️  更新版本号从 !CURRENT_VERSION! 到 %NEW_VERSION%...
    powershell -Command "(Get-Content Cargo.toml) -replace '^version = \".*\"', 'version = \"%NEW_VERSION%\"' | Set-Content Cargo.toml"
    set "CURRENT_VERSION=%NEW_VERSION%"
)

echo ℹ️  当前版本: !CURRENT_VERSION!

REM 备份现有版本（如果存在且未禁用备份）
if "%NO_BACKUP%"=="0" (
    if exist "FileManager" (
        echo ℹ️  备份当前版本...
        set "BACKUP_NAME=FileManager_backup_!CURRENT_VERSION!_%DATE:~0,4%%DATE:~5,2%%DATE:~8,2%_%TIME:~0,2%%TIME:~3,2%%TIME:~6,2%"
        set "BACKUP_NAME=!BACKUP_NAME: =0!"
        if exist "!BACKUP_NAME!" rmdir /s /q "!BACKUP_NAME!"
        move "FileManager" "!BACKUP_NAME!" >nul
        echo ✅ 备份到 !BACKUP_NAME!
    )
)

REM 快速模式或完整构建
if "%QUICK_MODE%"=="1" (
    echo ⚡ 快速模式：仅重新编译...
    cargo build --release --quiet
    if errorlevel 1 (
        echo ❌ 编译失败
        exit /b 1
    )
    
    REM 创建最小目录结构
    if not exist "FileManager" mkdir "FileManager"
    copy "target\release\file_manager.exe" "FileManager\" >nul
    
    echo ✅ 快速更新完成！
) else (
    echo ℹ️  完整构建模式...
    call "%SCRIPT_DIR%build.bat"
    if errorlevel 1 (
        echo ❌ 构建失败
        exit /b 1
    )
)

REM 验证结果
if not exist "FileManager\file_manager.exe" (
    echo ❌ 更新失败：可执行文件不存在
    exit /b 1
)

echo.
echo ✅ 更新完成！
echo   版本: !CURRENT_VERSION!
echo   模式: %QUICK_MODE%
if "%QUICK_MODE%"=="1" (
    echo   (快速模式)
) else (
    echo   (完整构建)
)
echo   位置: %CD%\FileManager
echo.
echo ℹ️  提示: 可以直接运行 FileManager\file_manager.exe

:end
if "%QUICK_MODE%"=="0" pause