# RuMorph

The rust compiler can only be accessed from nightly.
```bash
rustup toolchain install nightly
rustup default nightly         // change to nightly channel
rustup component add --toolchain nightly rust-src rustc-dev llvm-tools-preview
```

## Build up my clippy lints
```bash
cd rust-clippy
cargo build --release --bin cargo-clippy --bin clippy-driver -Zunstable-options --out-dir "$(rustc --print=sysroot)/bin"
cd /path/to/crate
cargo +nightly-2023-06-02 clippy -- -Wclippy::transmute_statistics
```
