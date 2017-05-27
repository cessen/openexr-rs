#!/bin/sh
# Runs the appropriate bindgen command for openexr-sys
bindgen \
    --whitelist-function ".*CEXR.*" \
    --whitelist-type ".*CEXR.*" \
    --whitelist-var ".*CEXR.*" \
    openexr-sys/c_wrapper/cexr.h > openexr-sys/src/bindings.rs
