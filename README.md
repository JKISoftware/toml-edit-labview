# toml-edit-labview
A LabVIEW wrapper around the Rust [toml_edit](https://docs.rs/toml_edit/latest/toml_edit/) library (built into a DLL).

[![Rust](https://github.com/JKISoftware/toml-edit-labview/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/JKISoftware/toml-edit-labview/actions/workflows/rust.yml)
![LabVIEW Version](https://img.shields.io/badge/LabVIEW-2020%20SP1-%23E37725.svg?})

## Goals of this Project
- To provide a cross-platform, production-quality, [toml](https://toml.io/) file library in LabVIEW
  - To support/preserve comments and other user formatting in the TOML data.
  - To be as easy to use as the LabVIEW Config File VIs for very basic use cases.
  - To expose lower-level capabilities of TOML for more complex/advanced use-cases.
- To showcase (and learn) how to interoperate between LabVIEW and Rust.
  - We use the [toml_edit](https://docs.rs/toml_edit/latest/toml_edit/) Rust crate, under the hood, as cross-platform a shared library built using Rust.

## Roadmap
- [ ] Cross-platform LabVIEW Support
  - [X] Windows
    - [ ] 32-bit LabVIEW - _in progress..._
    - [X] 64-bit LabIVEW
  - [ ] MacOSX - _in progress..._
    - [ ] Intel - _in progress..._
    - [ ] ARM (M1/M2/etc.)
  - [ ] Linux Desktop - _in progress..._
  - [ ] NI Linux Real-Time (e.g. cRIO and PXI)
- [ ] TOML data types supported
  - [ ] Scalar types
    - [X] String
    - [X] Integer
    - [ ] Float
    - [ ] Boolean
    - [ ] Datetime
- [X] Aggregate types
  - [X] Tables
  - [X] Subtables
  - [X] Inline Tables
  - [ ] Arrays
- [ ] VI Package
  - [ ] Installable in the palettes
  - [ ] Published on vipm.io 

## Development System Setup

Installation details: [rust-lang.org](https://forge.rust-lang.org)

### Windows Setup

#### Install Rust

Download and Run [rustup-init.exe](https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe).

This installer will run in a terminal/console and will prompt you to install MS Visual Studio Community Edition, in order to install the MSVC++ compiler and Windows APIs.  This takes up a bit of space (~4GB) and is required for building the DLL on Windows.

> More details: [rust-lang.org](https://forge.rust-lang.org) >> [other-installation-methods](https://forge.rust-lang.org/infra/other-installation-methods.html) >> windows >> rustup-init.exe

#### Install "just" (a nice command-line tool we'll use to build)

`cargo install just`

#### Install other Dev Packages

`just develop`

#### Build DLLs

`just build`

Note: on Mac and Linux the build process will be a little bit different. You can take a look at the **[Rust Build CI/CD Action](https://github.com/JKISoftware/toml-edit-labview/actions/workflows/rust.yml)** ([.github/workflows/rust.yml](https://github.com/JKISoftware/toml-edit-labview/blob/main/.github/workflows/rust.yml)) to see how it works.
