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

    original_a.serialize(&mut serialized);
    original_b.serialize(&mut serialized);
    original_c.serialize(&mut serialized);

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