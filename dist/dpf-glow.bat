@echo off
REM Run Digital Product Factory with the OpenGL (Glow) renderer.
REM
REM Use this if dpf.exe shows a BLACK window. The two back ends fail on
REM different graphics drivers, so if one is black, the other usually works.
REM
REM Whatever happens, dpf.exe writes dpf-startup.log next to itself - send that
REM file to support and it will say exactly which back end failed and why.
cd /d "%~dp0"
echo Starting Digital Product Factory with the OpenGL (Glow) renderer...
echo.
dpf.exe --renderer glow
if errorlevel 1 (
  echo.
  echo The app exited with an error. See dpf-startup.log in this folder.
  pause
)
