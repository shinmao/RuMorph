# Trophy cases 🏆


1. In the column of Bugs, there are two elements `Method-Cause`:  
Method:  
**M**: Manual  
**D**: Detector  
Root Cause:  
**UM**: Uninitialized MaybeUninit  
**UT**: Unsound transmute (Others)  
**RT**: Conversion to **R**elaxed **T**ype  
**IML**: Conversion without **I**nconsistent **M**emory **L**ayout  
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
| [mmtk](https://crates.io/crates/mmtk) | M-UT 1 | | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/mmtk/mmtk-core/825?logo=github)](https://github.com/mmtk/mmtk-core/issues/825) |
| [OLD-twizzler/twz-rust](https://github.com/twizzler-operating-system/OLD-twizzler) | M- | transmute | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/twizzler-operating-system/OLD-twizzler/9?logo=github)](https://github.com/twizzler-operating-system/OLD-twizzler/issues/9) |
| [rust-8080](https://github.com/irevoire/rust-8080) | M-IML 1 | Concrete / transmute | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/irevoire/rust-8080/16?logo=github)](https://github.com/irevoire/rust-8080/issues/16) |
| [cyfs-base](https://crates.io/crates/cyfs-base) | D-IML 1 | Concrete / transmute | [![RUSTSEC-2023-0046](https://img.shields.io/badge/RUSTSEC-2023--0046-blue?style=flat-square&logo=rust)](https://rustsec.org/advisories/RUSTSEC-2023-0046.html) |
| [cyfs-base](https://crates.io/crates/cyfs-base) | D-IML 1 | Concrete / transmute | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/buckyos/CYFS/274?logo=github)](https://github.com/buckyos/CYFS/issues/274) |
| [d4](https://crates.io/crates/d4) | D-IML 1 | Concrete / transmute | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/38/d4-format/71?logo=github)](https://github.com/38/d4-format/issues/71) |
| [hash-rs](https://crates.io/crates/hash-rs) | D-IML 1 | Concrete / ptr-as | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/asukharev/hash-rs/2?logo=github)](https://github.com/asukharev/hash-rs/issues/2) |
| [lmdb-rs](https://crates.io/crates/lmdb-rs) | D-ABP 1 | Generic / transmute | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/vhbit/lmdb-rs/67?logo=github)](https://github.com/vhbit/lmdb-rs/issues/67) |
| [rendy](https://crates.io/crates/rendy/) | D-ABP 2 | Generic / ptr-as | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/amethyst/rendy/328?logo=github)](https://github.com/amethyst/rendy/issues/328) |
| [data-buffer](https://crates.io/crates/data_buffer) | D-IML(alloc) 1 / ABP 1 | Generic / ptr-as | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/elrnv/buffer/2?logo=github)](https://github.com/elrnv/buffer/issues/2) |
| [lonlat-bng](https://crates.io/crates/lonlat_bng) | D-IML 1 | Generic / ptr-as | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/urschrei/lonlat_bng/19?logo=github)](https://github.com/urschrei/lonlat_bng/issues/19#issuecomment-1618461663) |
| [preserves](https://crates.io/crates/preserves) | D-IML 1 / ABP 1 | Generic / ptr-as | [![GitLab all issues](https://img.shields.io/gitlab/issues/all/preserves%2Fpreserves?logo=gitlab&label=issue%2042)](https://gitlab.com/preserves/preserves/-/issues/42) |