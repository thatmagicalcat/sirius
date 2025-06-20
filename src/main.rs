use sirius::Sirius;

#[derive(Sirius)]
enum Foo {
    A,
    B,
    C,
}

fn main() {
    use std::io::Write;
    let mut v = Vec::<u8>::new();
    let writer: &mut dyn Write = &mut v;

    _ = writer.write_all(b"hello");
    dbg!(v);
}
