# Trophy cases 🏆


1. In the column of Bugs, there are two elements `Method-Cause`:  
Method:  
**M**: Manual  
**D**: Detector  
Root Cause:  
**UM**: Uninitialized MaybeUninit  
**UT**: Unsound transmute (Others)  
**RT**: Conversion to **R**elaxed **T**ype  
**ML**: Conversion without **M**emory **L**ayout consideration  
**ST**: Conversion to **S**tricter **T**ype  
**ABP**: Conversion with **A**rbitrary **B**it **P**atterns

2. In the column of Conv, it shows the method of type conversion:  
**ptr-as**: raw pointer casting  
**transmute**: `transmute`  
**from**: with `From` trait


## Table
| Crate | Bugs | Conv | Issue Report |
| ----- | ---- | -------- | ------------ |
| [web-synth](https://github.com/Ameobea/web-synth) | M-UM | | [#41](https://github.com/Ameobea/web-synth/issues/41) |
| [rCore-arm64](https://github.com/rcore-os/rCore-Tutorial-v3-arm64) | M-UM | | [#1](https://github.com/rcore-os/rCore-Tutorial-v3-arm64/issues/1) |
| [mmtk](https://crates.io/crates/mmtk) | M-UT 1 | | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/mmtk/mmtk-core/825)](https://github.com/mmtk/mmtk-core/issues/825) |
| [vrp/heuristic-research](https://crates.io/crates/vrp-cli) | M-ML 1 | from-transmute | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/reinterpretcat/vrp/110)](https://github.com/reinterpretcat/vrp/issues/110) |
| [OLD-twizzler/twz-rust](https://github.com/twizzler-operating-system/OLD-twizzler) | M- | transmute | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/twizzler-operating-system/OLD-twizzler/9)](https://github.com/twizzler-operating-system/OLD-twizzler/issues/9) |
| [rust-8080](https://github.com/irevoire/rust-8080) | M-ML 1 | transmute | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/irevoire/rust-8080/16)](https://github.com/irevoire/rust-8080/issues/16) |
| [cyfs-base](https://crates.io/crates/cyfs-base) | D-ML 1 | transmute | [![RUSTSEC-2023-0046](https://img.shields.io/badge/RUSTSEC-2023--0046-blue?style=flat-square)](https://rustsec.org/advisories/RUSTSEC-2023-0046.html) |
| [cyfs-base](https://crates.io/crates/cyfs-base) | D-ML 1 | transmute | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/buckyos/CYFS/274)](https://github.com/buckyos/CYFS/issues/274) |
| [d4](https://crates.io/crates/d4) | D-ML 1 | transmute | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/38/d4-format/71)](https://github.com/38/d4-format/issues/71) |
| [hash-rs](https://crates.io/crates/hash-rs) | D-ML 1 | ptr-as | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/asukharev/hash-rs/2)](https://github.com/asukharev/hash-rs/issues/2) |
| [lmdb-rs](https://crates.io/crates/lmdb-rs) | D-ABP 1 | transmute | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/vhbit/lmdb-rs/67)](https://github.com/vhbit/lmdb-rs/issues/67) |
| [rendy](https://crates.io/crates/rendy/) | D-ABP 2 | ptr-as | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/amethyst/rendy/328)](https://github.com/amethyst/rendy/issues/328) |
| [data-buffer](https://crates.io/crates/data_buffer) | D-ABP 2 | ptr-as | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/elrnv/buffer/2)](https://github.com/elrnv/buffer/issues/2) |
| [lonlat-bng](https://crates.io/crates/lonlat_bng) | D-ABP 1 | ptr-as | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/urschrei/lonlat_bng/19)](https://github.com/urschrei/lonlat_bng/issues/19#issuecomment-1618461663) |
