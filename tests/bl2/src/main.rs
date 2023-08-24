#[derive(Copy, Clone)]
struct A {
    a: i8,
    b: i32,
    c: i8,
}

#[derive(Copy, Clone)]
struct B {
    a: i8,
    b: i32,
    c: i8,
}

fn main() {
    let a = A { a: 10, b: 11, c: 12};
    let b: &B = unsafe { std::mem::transmute(&a) };
    println!("{}", b.a);
}
