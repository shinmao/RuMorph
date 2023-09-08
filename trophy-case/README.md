# Trophy cases 🏆

In the column of Bugs, there are two elements `Method-Bug`:  
* Method: **M**anual or **D**etector    
* -`{Bug}`: Categories of Bugs  
    * **BL**: Broken Layout Bug
    * **UM**: Uninitialized Memory Exposure Bug
    * **BB**: Broken Bit Patterns Bug
* -**O**: Others 

In the column of Conv, it shows the method of type conversion:  
* **ptr-as**: raw pointer casting  
* **transmute**: `transmute`  


## Table
| Crate | Bugs | Conv | trigger | Issue Report |
| ----- | ---- | -------- | ----------- | ------------ |
| [web-synth](https://github.com/Ameobea/web-synth) | M-O(UM) | |  | [#41](https://github.com/Ameobea/web-synth/issues/41) |
| [mmtk](https://crates.io/crates/mmtk) | M-O(UT) | | | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/mmtk/mmtk-core/825?logo=github)](https://github.com/mmtk/mmtk-core/issues/825) |
| [rust-8080](https://github.com/irevoire/rust-8080) | M-BL 1 | transmute |  | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/irevoire/rust-8080/16?logo=github)](https://github.com/irevoire/rust-8080/issues/16) |
| [cyfs-base](https://crates.io/crates/cyfs-base) | D-BL |  transmute | dereference | [![RUSTSEC-2023-0046](https://img.shields.io/badge/RUSTSEC-2023--0046-blue?style=flat-square&logo=rust)](https://rustsec.org/advisories/RUSTSEC-2023-0046.html) |
| [cyfs-base](https://crates.io/crates/cyfs-base) | D-BL |  transmute | dereference | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/buckyos/CYFS/274?logo=github)](https://github.com/buckyos/CYFS/issues/274) |
| [d4](https://crates.io/crates/d4) | D-BL | transmute | dereference | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/38/d4-format/71?logo=github)](https://github.com/38/d4-format/issues/71) |
| [hash-rs](https://crates.io/crates/hash-rs) | D-BL | ptr-as | dereference | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/asukharev/hash-rs/2?logo=github)](https://github.com/asukharev/hash-rs/issues/2) |
| [lmdb-rs](https://crates.io/crates/lmdb-rs) | D-BL / BB / UM |  transmute | dereference | [![RUSTSEC-2023-0047](https://img.shields.io/badge/RUSTSEC-2023--0047-blue?style=flat-square&logo=rust)](https://rustsec.org/advisories/RUSTSEC-2023-0047.html) |
| [rendy](https://crates.io/crates/rendy/) | D-UM 2 | ptr-as | `std::slice::from_raw_parts` | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/amethyst/rendy/328?logo=github)](https://github.com/amethyst/rendy/issues/328) |
| [data-buffer](https://crates.io/crates/data_buffer) | D-UM 2 / O | ptr-as | `std::alloc::realloc` | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/elrnv/buffer/2?logo=github)](https://github.com/elrnv/buffer/issues/2) |
| [lonlat-bng](https://crates.io/crates/lonlat_bng) | D-BL 1 / UM 1 | ptr-as | `std::slice::from_raw_parts_mut` | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/urschrei/lonlat_bng/19?logo=github)](https://github.com/urschrei/lonlat_bng/issues/19#issuecomment-1618461663) |
| [preserves](https://crates.io/crates/preserves) | D-BL 1 / UM 1 | ptr-as | dereference | [![GitLab all issues](https://img.shields.io/gitlab/issues/all/preserves%2Fpreserves?logo=gitlab&label=issue%2042)](https://gitlab.com/preserves/preserves/-/issues/42) |
| [byte-conv](https://crates.io/crates/byte_conv) | D-UM 1 | ptr-as | `std::slice::from_raw_parts` | [![GitHub issue/pull request detail](https://img.shields.io/github/issues/detail/state/Lolirofle/byte_conv/1?logo=github)](https://github.com/Lolirofle/byte_conv/issues/1) |
| [endian-type-rs](https://crates.io/crates/endian-type-rs) | D-BL 2 / BB 2 | ptr-as | dereference | [issue](https://gitlab.com/ertos/endian-type-rs/-/issues/1) |
| [burst](https://crates.io/crates/burst) | D-BL 2 | ptr-as | dereference | [issue](https://github.com/endoli/burst.rs/issues/8) |
| [sfmt](https://crates.io/crates/sfmt) | D-BL 1 | ptr-as | dereference | [issue](https://github.com/rust-math/sfmt/issues/37) |
| [dtb](https://crates.io/crates/dtb) | D-BL 5 / O | ptr-as | dereference | [issue](https://github.com/ababo/dtb/issues/11) |
| [odbc-rs](https://crates.io/crates/odbc) | D-BL 2 |  ptr-as | dereference | [issue](https://github.com/Koka/odbc-rs/issues/174) |
| [netstat2](https://crates.io/crates/netstat2) | D-BL 1 | ptr-as | dereference | [issue](https://github.com/ohadravid/netstat2-rs/issues/9) |
| [radixt](https://crates.io/crates/radixt) | D-BL 4 | ptr-as | dereference | [issue](https://github.com/marekgalovic/radixt/issues/1) |
| [aws\_auth](https://github.com/golddranks/aws_auth/tree/main) | D-BL | ptr-as | dereference | [issue](https://github.com/golddranks/aws_auth/issues/1) |
| [journal](https://crates.io/crates/journal) | D-BL 2 | ptr-as | dereference | [issue](https://github.com/polygonhell/rusttests/issues/1) |
| [unicode\_converter](https://crates.io/crates/unicode_converter) | D-BL | ptr-as | dereference | [issue](https://github.com/Arkaeriit/unicode_converter/issues/1) |
| [mpack](https://crates.io/crates/mpack) | D-UM | ptr-as | `std::slice::from_raw_parts` | [issue](https://github.com/dradtke/mpack/issues/2) |
| [rafx-base](https://crates.io/crates/rafx-base) | D-UM 2 | ptr-as | `std::slice::from_raw_parts` | [issue](https://github.com/aclysma/rafx/issues/255) |
| [runes](https://crates.io/crates/runes) | D-UM 2 | transmute | `std::slice::from_raw_parts_mut` | [issue](https://github.com/Determinant/runes/issues/2) |
| [crayon](https://crates.io/crates/crayon) | D-UM 3 | ptr-as | `std::slice::from_raw_parts` | [issue](https://github.com/shawnscode/crayon/issues/108) |
| [skyline](https://crates.io/crates/skyline) | D-UM | ptr-as | dereference | [issue](https://github.com/ultimate-research/skyline-rs/issues/32) |
| [dream-go](https://github.com/Chicoryn/dream-go) | D-UM 2 | ptr-as | | [issue](https://github.com/Chicoryn/dream-go/issues/65) |
| [pipe-channel](https://crates.io/crates/pipe-channel) | D-UM 2 | ptr-as | `std::slice::from_raw_parts_mut` | [issue](https://github.com/bugaevc/pipe-channel/issues/4) |
| [granne](https://crates.io/crates/granne) | D-UM | ptr-as | `std::slice::from_raw_parts` | [issue](https://github.com/granne/granne/issues/25) |



## Ground Truth
* Positive cases for APB
	- [ ] [Can the pointer alignment situation be improved?](https://github.com/TimelyDataflow/abomonation/issues/23)
	- [ ] [ComponentBytes is unsound](https://github.com/kornelski/rust-rgb/issues/35)

1. Cases for Uninitialized Memory Exposure bugs
    * [MaybeUninit](https://github.com/alekseysidorov/static-box/issues/8)

## Tips for PoC
1. [struct visibility](https://doc.rust-lang.org/rust-by-example/mod/struct_visibility.html)
