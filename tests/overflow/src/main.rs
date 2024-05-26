fn withdraw(amount: u64) {
    let mut balance = 1u64;
    let mut remained = balance - amount;
    println!("{:?}", remained);
}

fn main() { // run with cargo run --release
    withdraw(100);
}
