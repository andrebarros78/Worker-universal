@echo off
setlocal EnableExtensions
rem Portable product launcher: derive source path from this file, never a fixed drive.
set "TMA_NODE="
if exist "%~dp0..\runtime\win-x64\node.exe" set "TMA_NODE=%~dp0..\runtime\win-x64\node.exe"
if not defined TMA_NODE if exist "%~dp0runtime\win-x64\node.exe" set "TMA_NODE=%~dp0runtime\win-x64\node.exe"
if not defined TMA_NODE (
  where node.exe >nul 2>&1
  if errorlevel 1 (
    echo TMA_PORTABLE_RUNTIME_UNAVAILABLE: package Node 24 runtime for offline distribution.
    exit /b 11
  )
  set "TMA_NODE=node.exe"
)
"%TMA_NODE%" "%~dp0src\portable_bridge_cli.ts" prepare
set "TMA_EXIT=%ERRORLEVEL%"
if not "%TMA_EXIT%"=="0" (
  echo TMA_PORTABLE_PREPARE_FAILED: %TMA_EXIT%
  exit /b %TMA_EXIT%
)
echo TMA_PORTABLE_BROWSER_PREPARED=true
echo TMA_BROWSER_PERMISSION_STAGE=REQUIRES_EDGE_EXTENSION_APPROVAL
exit /b 0
