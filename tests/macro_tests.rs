use sirius::Sirius;

#[test]
fn test_struct_sirius() {
    #[derive(Sirius, Debug, PartialEq)]
    struct TestStruct {
        a: u32,
        b: String,
        c: Vec<char>,
    }

    let original = TestStruct {
        a: 42,
        b: "Hello, world!".to_string(),
        c: vec!['H', 'e', 'l', 'l', 'o'],
    };

    let serialized = original.serialize_buffered();
    assert_eq!(
        serialized,
        vec![
            0, 0, 0, 42, // a
            0, 0, 0, 13, // length of b
            72, 101, 108, 108, 111, 44, 32, 119, 111, 114, 108, 100, 33, // "Hello, world!"
            0, 0, 0, 5, // length of c
            0, 0, 0, 72, // 'H'
            0, 0, 0, 101, // 'e'
            0, 0, 0, 108, // 'l'
            0, 0, 0, 108, // 'l'
            0, 0, 0, 111 // 'o'
        ]
    );

    let (deserialized, bytes_read) = TestStruct::deserialize(&serialized).unwrap();

    assert_eq!(deserialized, original);
    assert_eq!(bytes_read, serialized.len());
}

#[test]
fn test_enum_sirius() {
    #[derive(Sirius, Debug, PartialEq)]
    enum TestEnum {
        VariantA { x: u32, y: String },
        VariantB(u16),
        VariantC,
    }

    let original_a = TestEnum::VariantA {
        x: 10,
        y: "Hello".to_string(),
    };
    let original_b = TestEnum::VariantB(42);
    let original_c = TestEnum::VariantC;

    let mut serialized = Vec::new();

    original_a.serialize(&mut serialized).unwrap();
    original_b.serialize(&mut serialized).unwrap();
    original_c.serialize(&mut serialized).unwrap();

    #[rustfmt::skip]
    assert_eq!(
        serialized,
        vec![
            0, // VariantA
            0, 0, 0, 10, // x
            0, 0, 0, 5, // length of y
            72, 101, 108, 108, 111, // "Hello"

            1,   // VariantB
            0, 42, // value of VariantB

            2,  // VariantC (no data)
        ]
    );

    let mut offset = 0;
    let (deserialized_a, bytes_read_a) = TestEnum::deserialize(&serialized[offset..]).unwrap();
    offset += bytes_read_a;
    let (deserialized_b, bytes_read_b) = TestEnum::deserialize(&serialized[offset..]).unwrap();
    offset += bytes_read_b;
    let (deserialized_c, bytes_read_c) = TestEnum::deserialize(&serialized[offset..]).unwrap();

    assert_eq!(deserialized_a, original_a);
    assert_eq!(deserialized_b, original_b);
    assert_eq!(deserialized_c, original_c);
    assert_eq!(bytes_read_a + bytes_read_b + bytes_read_c, serialized.len());
}
