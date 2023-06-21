#!/bin/bash

BUILD_TYPE=release

# build shared library with Cargo
cargo build --$BUILD_TYPE

# Create a variable with the name of the framework
FRAMEWORK_NAME=dragon_toml_64

# Create the framework directory (and subdirectories)
rm -rf $FRAMEWORK_NAME.framework
mkdir -p $FRAMEWORK_NAME.framework/Versions/A

# Copy the `.dylib` file into the framework as version `A`
cp target/$BUILD_TYPE/libtoml.dylib $FRAMEWORK_NAME.framework/Versions/A/$FRAMEWORK_NAME
chmod ugo+rx $FRAMEWORK_NAME.framework/Versions/A/$FRAMEWORK_NAME

# Create symbolic links to the current version of the framework
ln -s ./A ./$FRAMEWORK_NAME.framework/Versions/Current
ln -s ./Versions/A/$FRAMEWORK_NAME ./$FRAMEWORK_NAME.framework


# Copy into LabVIEW Source Folder
rm -rf lv_src/$FRAMEWORK_NAME.framework
cp -R $FRAMEWORK_NAME.framework lv_src/