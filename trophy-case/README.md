# Trophy cases 🏆

In the column of Bugs, there are two elements `Method-Cause`:  
* Method: **M**anual or **D**etector    
* -`{Cause}`: Conversion without Inconsistent Memory Layout  
    * **SLR**: Conversion to type with **s**tricter **l**ayout **r**equirement
    * **APB**: Conversion from **a**rbitrary type without **p**adding **b**ytes exposure
    * **IAD**: **I**nconsistent layout betweeen **a**llocator and **d**eallocator
* -**O**: **O**thers (e.g., **UM**: Uninitialized Memory exposure, **UT**: Unsound transmute, **IT**: Invalid type creation)  

In the column of Conv, it shows the method of type conversion:  
* **ptr-as**: raw pointer casting  
* **transmute**: `transmute`  


## Table
| Crate | Bugs | Conv | trigger | Issue Report |
| ----- | ---- | -------- | ----------- | ------------ |
| [web-synth](https://github.com/Ameobea/web-synth) | M-O(UM) | |  | [#41](https://github.com/Ameobea/web-synth/issues/41) |
| [rCore-arm64](https://github.com/rcore-os/rCore-Tutorial-v3-arm64) | M-O(UM) | | | [#1](https://github.com/rcore-os/rCore-Tutorial-v3-arm64/issues/1) |
| [mmtk](https://crates.io/crates/mmtk) | M-O(UT) | | | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/mmtk/mmtk-core/825?logo=github)](https://github.com/mmtk/mmtk-core/issues/825) |
| [OLD-twizzler/twz-rust](https://github.com/twizzler-operating-system/OLD-twizzler) | M- | transmute |  | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/twizzler-operating-system/OLD-twizzler/9?logo=github)](https://github.com/twizzler-operating-system/OLD-twizzler/issues/9) |
| [rust-8080](https://github.com/irevoire/rust-8080) | M-IML 1 | transmute |  | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/irevoire/rust-8080/16?logo=github)](https://github.com/irevoire/rust-8080/issues/16) |
| [cyfs-base](https://crates.io/crates/cyfs-base) | D-SLR 1 |  transmute | dereference | [![RUSTSEC-2023-0046](https://img.shields.io/badge/RUSTSEC-2023--0046-blue?style=flat-square&logo=rust)](https://rustsec.org/advisories/RUSTSEC-2023-0046.html) |
| [cyfs-base](https://crates.io/crates/cyfs-base) | D-SLR 1 |  transmute | dereference | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/buckyos/CYFS/274?logo=github)](https://github.com/buckyos/CYFS/issues/274) |
| [d4](https://crates.io/crates/d4) | D-SLR 1 | transmute | dereference | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/38/d4-format/71?logo=github)](https://github.com/38/d4-format/issues/71) |
| [hash-rs](https://crates.io/crates/hash-rs) | D-SLR 1 | ptr-as | dereference | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/asukharev/hash-rs/2?logo=github)](https://github.com/asukharev/hash-rs/issues/2) |
| [lmdb-rs](https://crates.io/crates/lmdb-rs) | D-APB 1 / O(IT) 1 |  transmute | dereference | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/vhbit/lmdb-rs/67?logo=github)](https://github.com/vhbit/lmdb-rs/issues/67) |
| [rendy](https://crates.io/crates/rendy/) | D-APB 2 | ptr-as | `std::slice::from_raw_parts` | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/amethyst/rendy/328?logo=github)](https://github.com/amethyst/rendy/issues/328) |
| [data-buffer](https://crates.io/crates/data_buffer) | D-IAD 1 / APB 1 | ptr-as | `std::alloc::realloc` | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/elrnv/buffer/2?logo=github)](https://github.com/elrnv/buffer/issues/2) |
| [lonlat-bng](https://crates.io/crates/lonlat_bng) | D-APB 1 | ptr-as | `std::slice::from_raw_parts_mut` | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/urschrei/lonlat_bng/19?logo=github)](https://github.com/urschrei/lonlat_bng/issues/19#issuecomment-1618461663) |
| [preserves](https://crates.io/crates/preserves) | D-APB 1 / O(UM) 1 | ptr-as | dereference | [![GitLab all issues](https://img.shields.io/gitlab/issues/all/preserves%2Fpreserves?logo=gitlab&label=issue%2042)](https://gitlab.com/preserves/preserves/-/issues/42) |
| [byte-conv](https://crates.io/crates/byte_conv) | D-APB 1 | ptr-as | `std::slice::from_raw_parts` | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/Lolirofle/byte_conv/1?logo=github)](https://github.com/Lolirofle/byte_conv/issues/1) |


## Ground Truth
1. Positive cases for SLR
	- [] [Possible soundness bug: alignment not checked](https://github.com/softprops/atty/issues/50)
2. Positive cases for APB
	- [] [Can the pointer alignment situation be improved?](https://github.com/TimelyDataflow/abomonation/issues/23)
	- [] [ComponentBytes is unsound](https://github.com/kornelski/rust-rgb/issues/35)
3. Positive cases for IAD
