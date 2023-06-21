#!/bin/bash

BUILD_TYPE=release
SHARED_LIBARY_NAME=libtoml.dylib
BUILD_OUTPUT_DIR=target/$BUILD_TYPE

# build shared library with Cargo
cargo build --$BUILD_TYPE

# Create a variable with the name of the framework
FRAMEWORK_NAME=dragon_toml_64

./bundle_framework.sh
