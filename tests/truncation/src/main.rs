fn buggy_function(data: &[u8; 32]) {
    let mut buffer = [0u8; 16];
    buffer.copy_from_slice(&data[16..]);
    println!("buffer: {:?}", buffer);
}

fn main() {
    let data = [1u8; 32];
    buggy_function(&data);
    println!("bug found!");
}
