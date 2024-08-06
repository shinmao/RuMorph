fn overflow(amount: u64) {
    if let Some(_) = (amount - 2).checked_add(1) {
        println!("safe!");   
    } else {
        println!("not safe");
    }
    
    println!("end of overflow");

    if let Some(_) = (amount - 1).checked_add(1) {
        println!("safe");
    } 

    println!("end of function");
}

fn main() {
    overflow(u64::MAX);
}