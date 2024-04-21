# adhanapp

Adhan app for MacOS, Linux.

## Cross building
```
cargo install cross --git https://github.com/cross-rs/cross
cross build --target=x86_64-unknown-linux-gnu --release
```