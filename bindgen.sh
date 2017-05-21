#!/bin/sh
# Runs the appropriate bindgen command for openexr-sys
bindgen openexr-sys/c_wrapper/cexr.h > openexr-sys/src/bindings.rs

