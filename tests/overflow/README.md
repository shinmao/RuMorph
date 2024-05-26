```
2024-05-24 20:09:07.079032 |INFO | [rumorph-progress] RuMorph started
2024-05-24 20:09:07.079254 |INFO | [rumorph-progress] Overflow analysis started
2024-05-24 20:09:07.079277 |INFO | [rumorph-progress] OverflowChecker::analyze(withdraw)
2024-05-24 20:09:07.080100 |INFO | [rumorph-progress] find the bug with behavior_flag: EXTERNAL
2024-05-24 20:09:07.080129 |INFO | [rumorph-progress] OverflowChecker::analyze(main)
2024-05-24 20:09:07.080197 |INFO | [rumorph-progress] bug not found
2024-05-24 20:09:07.080205 |INFO | [rumorph-progress] Overflow analysis finished
2024-05-24 20:09:07.080209 |INFO | [rumorph-progress] RuMorph finished
Error (Overflow:): Potential overflow issue in `withdraw`
-> src/main.rs:1:1: 5:2
fn withdraw(amount: u64) {
    let mut balance = 1u64;
    let mut remained = balance - amount;
    println!("{:?}", remained);
}

2024-05-24 20:09:07.090216 |INFO | [rumorph-progress] cargo rumorph finished
```