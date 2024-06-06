fn overflow(amount: u64) {
    if let Some(_) = (amount - 2).checked_add(1) {
        println!("safe!");   
    }
    println!("end of basic block!");
}

fn main() {
    overflow(u64::MAX);
}