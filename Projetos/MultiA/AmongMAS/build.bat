@echo off
gcc -O2 -Wall -Wextra src\main.c src\map.c src\sim.c -o among_mas.exe -lws2_32
if %ERRORLEVEL% NEQ 0 (
    echo Build failed!
    exit /b 1
)
echo Build OK: among_mas.exe
