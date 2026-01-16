@echo off
set DATABASE_URL=postgres://rustpress:rustpress@localhost:5433/rustpress
cd /d "C:\Users\Software Engineering\Desktop\rustpress"
target\release\rustpress.exe
