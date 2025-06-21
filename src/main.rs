use sirius::Sirius;

#[derive(Sirius)]
enum Foo {
    A { a: String },
    B(u16),
    C,
    D,
}

fn main() -> Result<(), sirius::SiriusError> {
    let mut v = Vec::<u8>::new();

    let a = Foo::A { a: "Hello".into() };
    a.serialize(&mut v)?;

    let b = Foo::B(42);
    b.serialize(&mut v)?;

    let c = Foo::C;
    c.serialize(&mut v)?;

    let d = Foo::D;
    d.serialize(&mut v)?;

    dbg!(v);

    Ok(())
}
