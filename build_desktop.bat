@echo off
set PATH=C:\Users\fernando\.cargo\bin;%PATH%
set PATH=C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64;%PATH%
set LIBPATH=C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Tools\MSVC\14.44.35207\lib\x64;%LIBPATH%
set INCLUDE=C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Tools\MSVC\14.44.35207\include;%INCLUDE%
cd /d D:\Projects\Gitflowfy
rustup default stable-x86_64-pc-windows-msvc
cargo check --package gitflowfy-desktop