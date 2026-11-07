@echo off
"%~dp0target\release\winvideo.exe" %*
if "%~1"=="" pause
