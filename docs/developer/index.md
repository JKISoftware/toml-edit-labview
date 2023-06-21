# Developer Documentation

## Interoperating Between LabVIEW and Rust

### Theory of Operation

#### Thin Shared Library
LabVIEW calls the [toml_edit](https://docs.rs/toml_edit/latest/toml_edit/) Rust crate (library), by way of shared library (.dll, .so, etc.) that is a very thin wrapper around `toml_edit`.

The source for this shared library is in the [src/lib.rs](https://github.com/JKISoftware/toml-edit-labview/blob/main/src/lib.rs) file.

#### TOML object references
The shared library exposes an API for creating and closing references to the core objects/types in the `toml_edit` library (e.g. [toml_edit::Document](https://docs.rs/toml_edit/latest/toml_edit/struct.Document.html), [toml_edit::Table](https://docs.rs/toml_edit/latest/toml_edit/struct.Table.html), [toml_edit::Item](https://docs.rs/toml_edit/latest/toml_edit/enum.Item.html), [toml_edit::Value](https://docs.rs/toml_edit/latest/toml_edit/enum.Item.html#variant.Value), [toml_edit::InlineTable](https://docs.rs/toml_edit/latest/toml_edit/struct.InlineTable.html), [toml_edit::Array](https://docs.rs/toml_edit/latest/toml_edit/struct.Array.html), etc.)

#### 32-bit and 64-bit Support
Support for 32-bit and 64-bit builds of the shared library is achieved by configuring the LabVIEW call library function nodes to pass references as Unsigned Pointer-sized Integer (USZ) values (see #1)

#### Passing String data from Rust to LabVIEW
Passing string data from Rust to LabVIEW is done by...

  - (1) Rust passes a cstring pointer to LabVIEW as the return value of the shared library function,
  - (2) LabVIEW code uses the LabVIEW memory manager API to read the string data (byte-by-byte until a null character 0x00), and finally
  - (3) LabVIEW call an exported funtion in the Rust library to close the reference to the string data (deallocate/free the memory for the string).

#### Passing Array of String data from Rust to LabVIEW
Passing array of string data (e.g. a list of keynames in a table) from Rust to LabVIEW can be done by treating the array as a multi-line string.

> Note: This approach assumes (correctly) that keynames (e.g. of table elements) cannot have end-of line (EOL) characters in them.

So, we can simply...

  - (1) convert (in Rust) the array of strings to a multi-line string (so it can be sent as a scalar string),
  - (2) pass the multi-line string to LabVIEW (using the same technique as for scalar strings)
  - (3) converting (in LabVIEW) the multi-line string to an array of strings.

#### Passing Strings from LabVIEW to Rust
Passing strings from LabVIEW to Rust is done in a very simple way -- as a cstring pointer, which we would do for a typical C++ DLL.

## Cross-Plaform Support
It's not too tricky to build the shared library for other platforms -- we simply add the target using cargo, and then specify the target when we do the build.

We do the builds on github runners for the specific platform: `windows-latest`, `macos-latest`, and `ubuntu-latest`.

### Mac .Framework Bundle
LabVIEW for Mac can call `.framework` bundles via the Call Library Function Node -- we can't call the `.dylib` file (that Rust outputs during the build on mac) directly from LabVIEW.

So, we need to create a `.framework` bundle from the `.dylib` file, which just involves: creating some folders, renaming the `.dylib` file, and making some symbolic links (it's only a bit complex due to the lack of [available documentation](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPFrameworks/Concepts/FrameworkAnatomy.html) on how to do this).  You can see the [.github/workflows/rust.yml](https://github.com/JKISoftware/toml-edit-labview/blob/main/.github/workflows/rust.yml) workflow file for details.0
