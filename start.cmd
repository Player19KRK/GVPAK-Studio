@echo off
title GVPAK Studio
powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%~dp0start.ps1" %*
if errorlevel 1 (
  echo.
  echo GVPAK Studio could not start. See the error above.
  pause
  exit /b 1
)
