use crate::*;

impl<T: Sirius> Sirius for Box<T> {
    fn serialize(&self, output: &mut Vec<u8>) -> usize {
        T::serialize(self, output)
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SiriusError> {
        T::deserialize(data).map(|(t, l)| (Box::new(t), l))
    }
}

impl Sirius for Box<[u8]> {
    fn serialize(&self, output: &mut Vec<u8>) -> usize {
        serialize_with_length_prefix(self, output)
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SiriusError> {
        deserialize_with_length_prefix(data, |i, _| i.into())
    }
}

impl Sirius for Vec<u8> {
    fn serialize(&self, output: &mut Vec<u8>) -> usize {
        serialize_with_length_prefix(self, output)
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SiriusError> {
        deserialize_with_length_prefix(data, |i, _| i.into())
    }
}

fn serialize_with_length_prefix(slice: &[u8], output: &mut Vec<u8>) -> usize {
    if slice.len() >= LengthPrefix::MAX as usize {
        panic!("size exceeded length prefix");
    }

    output.extend_from_slice(&(slice.len() as LengthPrefix).to_be_bytes());
    output.extend_from_slice(slice);

    slice.len() + LENGTH_BYTES
}

fn deserialize_with_length_prefix<T, F: FnOnce(&[u8], usize) -> T>(
    data: &[u8],
    f: F,
) -> Result<(T, usize), SiriusError> {
    let len = u32::from_be_bytes(
        data.get(0..LENGTH_BYTES)
            .ok_or(SiriusError::NotEnoughData)?
            .try_into()
            .unwrap(),
    ) as usize;

    Ok((
        f(
            data.get(LENGTH_BYTES..len + LENGTH_BYTES)
                .ok_or(SiriusError::NotEnoughData)?,
            len + LENGTH_BYTES,
        ),
        len + LENGTH_BYTES,
    ))
}

impl Sirius for char {
    fn serialize(&self, output: &mut Vec<u8>) -> usize {
        output.extend_from_slice(&(*self as u32).to_be_bytes());
        std::mem::size_of::<Self>()
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SiriusError> {
        let raw = u32::from_be_bytes(
            data.get(..std::mem::size_of::<Self>())
                .ok_or(SiriusError::NotEnoughData)?
                .try_into()
                .unwrap(),
        );

        Ok((
            char::from_u32(raw).ok_or(SiriusError::ParsingError {
                ty_name: "char",
                error: format!("invalid character: {raw:X}"),
            })?,
            std::mem::size_of::<Self>(),
        ))
    }
}

impl_bytemagic_for_numbers! {
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
    f32, f64
}

impl_bytemagic_for_array! {
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
    f32, f64
}

#[test]
fn test_char_bytemagic() {
    let original = '💯';
    let serialized = original.serialize_buffered();
    let (deserialized, bytes_read) = char::deserialize(&serialized).unwrap();

    assert_eq!(deserialized, original);
    assert_eq!(bytes_read, serialized.len());
}

#[test]
fn test_char_bytemagic_check() {
    let data = 0x110000_u32.to_be_bytes();
    assert!(matches!(
        char::deserialize(&data),
        Err(SiriusError::ParsingError {
            ty_name: "char",
            ..
        })
    ));
}

#[test]
fn test_vec_bytemagic() {
    let original = b"The quick brown fox jumps over the lazy dog.".to_vec();
    let serialized = original.serialize_buffered();

    let (deserialized, bytes_read) = Vec::<u8>::deserialize(&serialized).unwrap();

    assert_eq!(deserialized, original);
    assert_eq!(bytes_read, serialized.len());
}
