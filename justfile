# use PowerShell instead of sh:
set shell := ["powershell.exe", "-c"]

alias   b   := build
alias   d   := develop

build:
    cargo build --target=i686-pc-windows-msvc --release
    Copy-Item -Force -Path "target\i686-pc-windows-msvc\release\*.dll" -Destination "lv_src\dragon_toml_32.dll"
    cargo build --target=x86_64-pc-windows-msvc --release
    Copy-Item -Force -Path "target\x86_64-pc-windows-msvc\release\*.dll" -Destination "lv_src\dragon_toml_64.dll"

develop:                                            
    rustup target add i686-pc-windows-msvc
    rustup target add x86_64-pc-windows-msvc
    