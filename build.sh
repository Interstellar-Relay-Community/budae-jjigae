#!/bin/bash

ARCH="$1"
echo "Target arch: $ARCH"

# Rust statically links to libc, but alpine does not like it
# (especially in multistage build)
# See https://github.com/sfackler/rust-native-tls/issues/190

export RUSTFLAGS="-Ctarget-feature=-crt-static"

if [ "$ARCH" = "linux/amd64" ]; then
    export RUSTFLAGS="${RUSTFLAGS} -Ctarget-feature=+avx,+avx2,+fma"
elif [ "$ARCH" = "linux/arm64" ]; then
    export RUSTFLAGS="${RUSTFLAGS} -Ctarget-feature=+neon"
else
    echo "Bad arch: $ARCH"
    exit 1
fi

cargo build --release
