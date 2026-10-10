@echo off
setlocal EnableExtensions
set "TMA_ROOT=%~dp0"
set "TMA_NODE=%TMA_ROOT%web-worker\runtime\win-x64\node.exe"
set "TMA_PYTHON=%TMA_ROOT%runtime\python-win-x64\python.exe"
set "TMA_CORE=%TMA_ROOT%bin\win-x64\tma-core.exe"
set "TMA_SUPERVISOR=%TMA_ROOT%bin\win-x64\tma-supervisor.exe"
set "TMA_OTP=%TMA_ROOT%runtime\elixir-availability-win-x64\bin\tma_availability.bat"
set "TMA_ACTION=%~1"
if not defined TMA_ACTION set "TMA_ACTION=prepare"
if not exist "%TMA_NODE%" (echo TMA_NODE_RUNTIME_MISSING& exit /b 11)
if /I "%TMA_ACTION%"=="prepare" goto prepare
if /I "%TMA_ACTION%"=="verify" goto verify
if /I "%TMA_ACTION%"=="supervisor-health" goto supervisorhealth
echo TMA_COMMAND_UNSUPPORTED=%TMA_ACTION%
exit /b 64

:prepare
"%TMA_NODE%" "%TMA_ROOT%web-worker\src\portable_bridge_cli.ts" prepare
if errorlevel 1 (echo TMA_PRODUCT_PREPARATION_FAILED& exit /b 4)
echo TMA_PORTABLE_PREPARED=TRUE
echo TMA_BROWSER_EXTENSION_PERMISSION=REQUIRES_BROWSER_APPROVAL
echo TMA_LIVE_MISSION_PROVEN=FALSE
exit /b 0

:verify
call "%~f0" prepare
if errorlevel 1 exit /b 4
if not exist "%TMA_CORE%" (echo TMA_CORE_MISSING& exit /b 11)
if not exist "%TMA_SUPERVISOR%" (echo TMA_SUPERVISOR_MISSING& exit /b 11)
if not exist "%TMA_PYTHON%" (echo TMA_PYTHON_MISSING& exit /b 11)
if not exist "%TMA_OTP%" (echo TMA_OTP_MISSING& exit /b 11)
"%TMA_CORE%" self-test
if errorlevel 1 (echo TMA_CORE_SELFTEST_FAILED& exit /b 12)
"%TMA_SUPERVISOR%" self-test
if errorlevel 1 (echo TMA_SUPERVISOR_SELFTEST_FAILED& exit /b 13)
"%TMA_PYTHON%" -I -m timed_mission_agent.cli self-test
if errorlevel 1 (echo TMA_PYTHON_SELFTEST_FAILED& exit /b 14)
call "%TMA_OTP%" version
if errorlevel 1 (echo TMA_AVAILABILITY_SELFTEST_FAILED& exit /b 15)
echo TMA_INTEGRATED_OFFLINE_BOOTSTRAP=PASS
echo TMA_BROWSER_EXTENSION_PERMISSION=REQUIRES_BROWSER_APPROVAL
echo TMA_LIVE_MISSION_PROVEN=FALSE
exit /b 0

:supervisorhealth
if not exist "%TMA_SUPERVISOR%" (echo TMA_SUPERVISOR_MISSING& exit /b 11)
"%TMA_SUPERVISOR%" serve-health 18765
exit /b %ERRORLEVEL%
