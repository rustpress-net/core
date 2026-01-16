@echo off
call "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat"
cd /d "C:\Users\Software Engineering\Desktop\rustpress"
cargo check -p rustpress-database

