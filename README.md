# toml-edit-labview
A LabVIEW wrapper around the Rust [toml_edit](https://docs.rs/toml_edit/latest/toml_edit/) library (built into a DLL).

[![Rust](https://github.com/JKISoftware/toml-edit-labview/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/JKISoftware/toml-edit-labview/actions/workflows/rust.yml)
![LabVIEW Version](https://img.shields.io/badge/LabVIEW-2020%20SP1-%23E37725.svg?})

At present, there are 32-bit and 64-bit Windows DLLs in the lv_src folder that are kept up-to-date. So, you should be able to clone the git repo and run the code in LabVIEW 2020 or greater on Windows.  On Mac or Linux, you can download the latest shared libraries from [the Releases](https://github.com/JKISoftware/toml-edit-labview/releases/tag/v0.0.1) or the [Actions artifacts](https://github.com/JKISoftware/toml-edit-labview/actions).  There's no VI Package yet, but that will happen in time.

## Current Status --> Alpha / Experimental
There's a lot that's in flux and/or not currently working:
- The shared library runs in the user interface thread (i.e. it's not multi-thread safe), but we're looking into it here -> [#2](https://github.com/JKISoftware/toml-edit-labview/issues/2)
- It might crash LabVIEW -- we're currently tightening up all the code to ensure it won't crash LabVIEW, buy LabVIEW does crash often when we're working on the code and tweaking things.
- It might have memory leaks -- we haven't tested that much yet.
- The names of the VIs (and shared library/functions) are in flux and likely to change.

## Key Features
Not all of these are available right now, but will be coming over time (see Roadmap, below).

- Built on top of a production-quality toml library ([toml_edit](https://docs.rs/toml_edit/latest/toml_edit/)) that's written in [Rust](https://www.rust-lang.org/).
- Provides a simple-to-use API that feels a bit like LabVIEW's config file VIs
- Provides a low-level API that exposes all the features of TOML that you might need for reading and writing to toml files.
- Cross-platform support for LabVIEW 32-bit and 64-bit on Windows, Mac, Linux, and NI-Linux Real-time.

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

See our [Developer Docs](docs/developer/index.md) for lots more info.

Rust Installation details: [rust-lang.org](https://forge.rust-lang.org)

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
