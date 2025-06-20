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

    let foo = Foo::A { a: "Hello".into() };
    foo.serialize(&mut v)?;

    let foo = Foo::B(42);
    foo.serialize(&mut v)?;

    let foo = Foo::C;
    foo.serialize(&mut v)?;

    let foo = Foo::D;
    foo.serialize(&mut v)?;

    dbg!(v);

    Ok(())
}
