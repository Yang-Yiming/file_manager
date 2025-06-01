@echo off
REM 文件管理器 Windows 应用构建脚本
REM 简洁高效的构建工具

setlocal enabledelayedexpansion

REM 确定项目根目录
set "SCRIPT_DIR=%~dp0"
pushd "%SCRIPT_DIR%..\.."
set "PROJECT_ROOT=%CD%"
popd

cd /d "%PROJECT_ROOT%"

echo 🚀 文件管理器构建工具 (Windows)
echo ===============================

REM 检查环境
if not exist "Cargo.toml" (
    echo ❌ 无法找到项目根目录（Cargo.toml 不存在）
    echo 当前目录: %CD%
    exit /b 1
)

where cargo >nul 2>&1
if errorlevel 1 (
    echo ❌ 未找到 Rust/Cargo，请先安装
    exit /b 1
)

REM 获取版本号
for /f "tokens=3 delims== " %%a in ('findstr /r "^version" Cargo.toml') do (
    set "VERSION=%%a"
    set "VERSION=!VERSION:"=!"
)

echo ℹ️  构建文件管理器 v!VERSION!...

REM 清理并构建
echo ℹ️  编译 Release 版本...
cargo build --release --quiet
if errorlevel 1 (
    echo ❌ 编译失败
    exit /b 1
)

REM 创建发布目录
echo ℹ️  创建应用程序包...
if exist "FileManager" rmdir /s /q "FileManager"
mkdir "FileManager"
mkdir "FileManager\data"

REM 复制可执行文件
copy "target\release\file_manager.exe" "FileManager\" >nul
if errorlevel 1 (
    echo ❌ 复制可执行文件失败
    exit /b 1
)

REM 复制资源文件（如果存在）
if exist "res\icon.ico" copy "res\icon.ico" "FileManager\" >nul

REM 创建启动脚本
echo @echo off > "FileManager\FileManager.bat"
echo cd /d "%%~dp0" >> "FileManager\FileManager.bat"
echo start "" "file_manager.exe" >> "FileManager\FileManager.bat"

REM 创建说明文件
echo 文件快速访问管理器 v!VERSION! > "FileManager\README.txt"
echo. >> "FileManager\README.txt"
echo 使用方法： >> "FileManager\README.txt"
echo 1. 双击 file_manager.exe 直接运行 >> "FileManager\README.txt"
echo 2. 或双击 FileManager.bat 启动 >> "FileManager\README.txt"
echo. >> "FileManager\README.txt"
echo 数据文件将保存在用户主目录下的 AppData\Roaming\file_manager\ 文件夹中 >> "FileManager\README.txt"

REM 验证构建结果
if not exist "FileManager\file_manager.exe" (
    echo ❌ 构建失败：可执行文件不存在
    exit /b 1
)

REM 获取文件大小
for %%F in ("FileManager") do set "SIZE=%%~zF"
set /a "SIZE_MB=!SIZE! / 1024 / 1024"

echo ✅ 构建完成！
echo   版本: !VERSION!
echo   位置: %CD%\FileManager
echo.
echo ℹ️  使用方法：
echo   • 双击 FileManager\file_manager.exe 运行
echo   • 或双击 FileManager\FileManager.bat 启动
echo   • 可将整个 FileManager 文件夹复制到任意位置使用

echo.
echo ✅ 全部完成！
pause