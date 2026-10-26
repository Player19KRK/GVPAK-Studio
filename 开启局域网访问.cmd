@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0enable-team-lan.ps1" -NetworkAlias "WLAN" -TrustedNetworkName "KL-508-5G 2"
if errorlevel 1 pause
