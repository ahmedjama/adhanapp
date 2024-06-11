#!/bin/sh

set -e


# Add targets for cross-compilation
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-unknown-linux-gnu

cargo install cross --git https://github.com/cross-rs/cross
cross build --target=x86_64-unknown-linux-gnu --release
cross build --target=aarch64-unknown-linux-gnu --release