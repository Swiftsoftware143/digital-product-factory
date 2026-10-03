@echo off
REM Run Digital Product Factory with the DirectX (wgpu) renderer - the default.
REM
REM Use this if dpf-glow.bat showed a black window and you want to switch back.
REM
REM Whatever happens, dpf.exe writes dpf-startup.log next to itself - send that
REM file to support and it will say exactly which back end failed and why.
cd /d "%~dp0"
echo Starting Digital Product Factory with the DirectX (wgpu) renderer...
echo.
dpf.exe --renderer wgpu
if errorlevel 1 (
  echo.
  echo The app exited with an error. See dpf-startup.log in this folder.
  pause
)
