# toml-edit-labview
A LabVIEW wrapper around the rust `toml_edit` library (built into a DLL).

[![Rust](https://github.com/JKISoftware/toml-edit-labview/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/JKISoftware/toml-edit-labview/actions/workflows/rust.yml)
![LabVIEW Version](https://img.shields.io/badge/LabVIEW-2020%20SP1-%23E37725.svg?})

## Development

### Install Rust
[rust-lang.org](https://forge.rust-lang.org) >> [other-installation-methods](https://forge.rust-lang.org/infra/other-installation-methods.html) >> windows >> [rustup-init.exe](https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe)

### Install "just" (a nice command-line tool we'll use to build)
`cargo install just`

### Install other Dev Packages
`just develop`

### Build DLLs
`just build`
