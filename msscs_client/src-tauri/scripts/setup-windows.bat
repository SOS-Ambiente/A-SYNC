@echo off
REM Windows build environment setup script

echo Setting up MSSCS Windows build environment...

REM Check for Visual Studio Build Tools
where cl >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Visual Studio Build Tools not found. Please install with "C++ build tools" workload.
    echo Visit: https://visualstudio.microsoft.com/visual-cpp-build-tools/
    exit /b 1
)

REM Install Rust target
echo Installing Windows target...
rustup target add x86_64-pc-windows-msvc

REM Check for Node.js and pnpm
where node >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Node.js not found. Please install Node.js.
    exit /b 1
)

where pnpm >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: pnpm not found. Please install pnpm.
    echo Run: npm install -g pnpm
    exit /b 1
)

echo Windows build environment setup complete!
echo Run: pnpm build:windows