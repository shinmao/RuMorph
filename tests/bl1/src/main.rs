fn risky_function(val: *const i32) {
    let t: *const i64 = unsafe {
        std::mem::transmute(val)
    };
    println!("{}", unsafe { &*t });
}

fn main() {
    let a: i32 = 3;
    risky_function(&a as *const i32);
}
