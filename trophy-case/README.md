# Trophy cases 🏆


1. In the column of Bugs, there are two elements `Method-Cause`:  
`Method`:  
**M**: Manual  
**D**: Detector  
`Root Cause`:  
**UM**: Uninitialized MaybeUninit  
**UT**: Unsound transmute (Others)  
**RTR**: Relaxing Type Requirement  
**ML**: Mis-Layout  
**STR**: Stricter Type Requirement

2. In the column of Conv, it shows the method of type conversion:  
**ptr-as**: raw pointer casting  
**transmute**: `transmute`  
**from**: with `From` trait


## Table
| Crate | Bugs | Conv | Issue Report |
| ----- | ---- | -------- | ------------ |
| [web-synth](https://github.com/Ameobea/web-synth) | M-UM | | [#41](https://github.com/Ameobea/web-synth/issues/41) |
| [rCore-arm64](https://github.com/rcore-os/rCore-Tutorial-v3-arm64) | M-UM | | [#1](https://github.com/rcore-os/rCore-Tutorial-v3-arm64/issues/1) |
| [mmtk](https://crates.io/crates/mmtk) | M-UT | | [#825](https://github.com/mmtk/mmtk-core/issues/825) |
| [vrp/heuristic-research](https://crates.io/crates/vrp-cli) | M-ML | from-transmute | [#110](https://github.com/reinterpretcat/vrp/issues/110) |
| [OLD-twizzler/twz-rust](https://github.com/twizzler-operating-system/OLD-twizzler) | M- | transmute | [#9](https://github.com/twizzler-operating-system/OLD-twizzler/issues/9) |
| [rust-8080](https://github.com/irevoire/rust-8080) | M-ML | transmute | [#16](https://github.com/irevoire/rust-8080/issues/16) |
| [cyfs-base](https://crates.io/crates/cyfs-base) | D-ML | transmute | [![RUSTSEC-2023-0046](https://img.shields.io/badge/RUSTSEC-2023--0046-blue?style=flat-square)](https://rustsec.org/advisories/RUSTSEC-2023-0046.html) |
| [cyfs-base](https://crates.io/crates/cyfs-base) | D-ML | transmute | [#274](https://github.com/buckyos/CYFS/issues/274) |
| [d4](https://crates.io/crates/d4) | D-ML | transmute | [#71](https://github.com/38/d4-format/issues/71) |
| [hash-rs](https://crates.io/crates/hash-rs) | D-ML | ptr-as | [#2](https://github.com/asukharev/hash-rs/issues/2) |
| [lmdb-rs](https://crates.io/crates/lmdb-rs) | M-STR | transmute | [#67](https://github.com/vhbit/lmdb-rs/issues/67) |
