# Trophy cases 🏆


1. In the column of Bugs, there are two elements `Method-Cause`:  
`Method`: **M**: Manual  
`Root Cause`: **UM**: Uninitialized MaybeUninit `|` **UT**: Unsound Transmute

2. In the column of Affect:  
**BVI**: Broken Validity Invariant


## Table
| Crate | Bugs | Affect | Issue Report | RustSec ID |
| ----- | ---- | -------- | ------------ | ---------- |
| [web-synth](https://github.com/Ameobea/web-synth) | M-UM | BVI | [#41](https://github.com/Ameobea/web-synth/issues/41) | Not Lib |
| [rCore-arm64](https://github.com/rcore-os/rCore-Tutorial-v3-arm64) | M-UM | BVI | [#1](https://github.com/rcore-os/rCore-Tutorial-v3-arm64/issues/1) | Not Lib |
| [mmtk](https://crates.io/crates/mmtk) | M-UT | BVI | [#825](https://github.com/mmtk/mmtk-core/issues/825) | Not release |
