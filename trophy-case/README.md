# Trophy cases 🏆


1. In the column of Bugs, there are two elements `Method-Cause`:  
`Method`: **M**: Manual  
`Root Cause`:  
**UM**: Uninitialized MaybeUninit  
**UT**: Unsound transmute (Others)  
**RTI**: Relaxing Type Invariant  
**MA**: Misalignment

2. In the column of Conv, it shows the method of type conversion:  
**ptr-as**: raw pointer casting  
**transmute**: `transmute`  
**from**: with `From` trait


## Table
| Crate | Bugs | Conv | Issue Report | RustSec ID |
| ----- | ---- | -------- | ------------ | ---------- |
| [web-synth](https://github.com/Ameobea/web-synth) | M-UM | | [#41](https://github.com/Ameobea/web-synth/issues/41) | Not Lib |
| [rCore-arm64](https://github.com/rcore-os/rCore-Tutorial-v3-arm64) | M-UM | | [#1](https://github.com/rcore-os/rCore-Tutorial-v3-arm64/issues/1) | Not Lib |
| [mmtk](https://crates.io/crates/mmtk) | M-UT | | [#825](https://github.com/mmtk/mmtk-core/issues/825) | Not release |
| [vrp/heuristic-research](https://crates.io/crates/vrp-cli) | M-MA | from-transmute | [#110](https://github.com/reinterpretcat/vrp/issues/110) | Exp crate in side vrp |
