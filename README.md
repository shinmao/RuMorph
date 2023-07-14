# RuMorph
Follow the steps if you want to install RuMorph on your localhost.

## First time to set up environment
Set up the folder and path for rumorph:
```
python3 setup_rumorph_home.py path/to/your/rumorph_home
```
Set up rustc version, toolchains, and env variables:
```bash
rustup install nightly-2023-06-02
rustup default nightly-2023-06-02         // change to nightly channel which is rustc 1.72.0-nightly
rustup component add rust-src rustc-dev llvm-tools-preview miri

// env var setup
export RUMORPH_RUST_CHANNEL=nightly-2023-06-02
export RUMORPH_RUNNER_HOME="/home/RuMorph/rumorph-home"
export RUSTFLAGS="-L $HOME/.rustup/toolchains/nightly-2023-06-02-x86_64-unknown-linux-gnu/lib"
export LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:$HOME/.rustup/toolchains/nightly-2023-06-02-x86_64-unknown-linux-gnu/lib"
```

## install with `install.sh`
```
// $0 is bin
cargo install --path "$(dirname $0)" --force
```
You should be able to see the message that `cargo-rumorph` and `rumorph` are intalled.

## Troubleshoot
```
error: failed to run `rustc` to learn about target-specific information
```


## Build up my clippy lints
```bash
cd rust-clippy
cargo build --release --bin cargo-clippy --bin clippy-driver -Zunstable-options --out-dir "$(rustc --print=sysroot)/bin"
cd /path/to/crate
cargo +nightly-2023-06-02 clippy -- -Wclippy::transmute_statistics
```