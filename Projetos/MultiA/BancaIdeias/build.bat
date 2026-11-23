@echo off
g++ -std=c++20 -O2 -Wall -Wextra src\main.cpp src\gemini.cpp src\tools.cpp -o banca_ideias.exe -lwinhttp -lws2_32
if %ERRORLEVEL% NEQ 0 (
    echo Build failed!
    exit /b 1
)
echo Build OK: banca_ideias.exe
