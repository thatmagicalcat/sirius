use crate::*;

macro_rules! impl_bytemagic_for_array {
    [ $($t:ty),+ ] => {
        $(
            impl<const N: usize> ByteMagic for [$t; N] {
                fn serialize(&self, output: &mut impl std::io::Write) -> usize {
                    let prefix_len = match N {
                        n if n < u8::MAX as usize => {
                            output.write_all(&(N as u8).to_be_bytes()).unwrap();
                            std::mem::size_of::<u8>()
                        }
                        n if n < u16::MAX as usize => {
                            output.write_all(&(N as u16).to_be_bytes()).unwrap();
                            std::mem::size_of::<u16>()
                        }
                        n if n < u32::MAX as usize => {
                            output.write_all(&(N as u32).to_be_bytes()).unwrap();
                            std::mem::size_of::<u32>()
                        }
                        n if n < u64::MAX as usize => {
                            output.write_all(&(N as u64).to_be_bytes()).unwrap();
                            std::mem::size_of::<u64>()
                        }

                        _ => panic!("bigger values are not supported"),
                    };

                    for i in self {
                        output.write_all(&i.to_be_bytes()).unwrap();
                    }

                    self.len() + prefix_len
                }

                fn deserialize(data: &[u8]) -> Result<(Self, usize), ByteMagicError>
                where
                    Self: Sized,
                {
                    let (prefix_len, data_len) = match N {
                        n if n < u8::MAX as usize => {
                            let prefix_len = std::mem::size_of::<u8>();
                            (
                                prefix_len,
                                u8::from_be_bytes(
                                    data.get(0..prefix_len)
                                        .ok_or(ByteMagicError::NotEnoughData)?
                                        .try_into()
                                        .unwrap(),
                                ) as usize,
                            )
                        }
                        n if n < u16::MAX as usize => {
                            let prefix_len = std::mem::size_of::<u16>();
                            (
                                prefix_len,
                                u16::from_be_bytes(
                                    data.get(0..prefix_len)
                                        .ok_or(ByteMagicError::NotEnoughData)?
                                        .try_into()
                                        .unwrap(),
                                ) as usize,
                            )
                        }
                        n if n < u32::MAX as usize => {
                            let prefix_len = std::mem::size_of::<u32>();
                            (
                                prefix_len,
                                u32::from_be_bytes(
                                    data.get(0..prefix_len)
                                        .ok_or(ByteMagicError::NotEnoughData)?
                                        .try_into()
                                        .unwrap(),
                                ) as usize,
                            )
                        }
                        n if n < u64::MAX as usize => {
                            let prefix_len = std::mem::size_of::<u64>();
                            (
                                prefix_len,
                                u64::from_be_bytes(
                                    data.get(0..prefix_len)
                                        .ok_or(ByteMagicError::NotEnoughData)?
                                        .try_into()
                                        .unwrap(),
                                ) as usize,
                            )
                        }

                        _ => panic!("bigger values are not supported"),
                    };

                    let data_size = data_len * std::mem::size_of::<$t>();
                    let data = data
                        .get(prefix_len..data_size + prefix_len)
                        .ok_or(ByteMagicError::NotEnoughData)?;
                    let chunk_size = std::mem::size_of::<$t>();

                    let mut output: [$t; N] = [0 as _; N];
                    for (idx, chunk) in data.chunks_exact(chunk_size).enumerate() {
                        output[idx] = <$t>::from_be_bytes(chunk.try_into().unwrap());
                    }

                    Ok((output, prefix_len + data_size))
                }
            }
        )+

        #[test]
        fn test_array_bytemagic() {
            $(
                let mut data: [$t; 100] = [69 as _; 100];
                data.iter_mut().enumerate().for_each(|(idx, itm)| {
                    *itm = idx as _;
                });

                let v = data.serialize_buffered();
                let (n, bytes_read) = <[$t; 100] as ByteMagic>::deserialize(&v).unwrap();

                assert!(data.iter().zip(n.iter()).all(|(&a, &b)| a == b));
                assert_eq!(bytes_read, v.len());
            )+
        }
    }
}

impl<T: ByteMagic> ByteMagic for Box<T> {
    fn serialize(&self, output: &mut impl std::io::Write) -> usize {
        T::serialize(self, output)
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), ByteMagicError>
    where
        Self: Sized,
    {
        T::deserialize(data).map(|(t, l)| (Box::new(t), l))
    }
}

impl ByteMagic for Box<[u8]> {
    fn serialize(&self, output: &mut impl std::io::Write) -> usize {
        serialize_with_length_prefix(self, output)
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), ByteMagicError> {
        deserialize_with_length_prefix(data, |i, _| i.into())
    }
}

impl ByteMagic for Vec<u8> {
    fn serialize(&self, output: &mut impl std::io::Write) -> usize {
        serialize_with_length_prefix(self, output)
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), ByteMagicError> {
        deserialize_with_length_prefix(data, |i, _| i.into())
    }
}

fn serialize_with_length_prefix(slice: &[u8], output: &mut impl std::io::Write) -> usize {
    if slice.len() >= LengthPrefix::MAX as usize {
        panic!("size exceeded length prefix");
    }

    output
        .write_all(&(slice.len() as LengthPrefix).to_be_bytes())
        .unwrap();
    output.write_all(slice).unwrap();

    slice.len() + LENGTH_BYTES
}

fn deserialize_with_length_prefix<T, F: FnOnce(&[u8], usize) -> T>(
    data: &[u8],
    f: F,
) -> Result<(T, usize), ByteMagicError> {
    let len = u32::from_be_bytes(
        data.get(0..LENGTH_BYTES)
            .ok_or(ByteMagicError::NotEnoughData)?
            .try_into()
            .unwrap(),
    ) as usize;

    Ok((
        f(
            data.get(LENGTH_BYTES..len + LENGTH_BYTES)
                .ok_or(ByteMagicError::NotEnoughData)?,
            len + LENGTH_BYTES,
        ),
        len + LENGTH_BYTES,
    ))
}

impl ByteMagic for char {
    fn serialize(&self, output: &mut impl std::io::Write) -> usize {
        output.write_all(&(*self as u32).to_be_bytes()).unwrap();
        std::mem::size_of::<Self>()
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), ByteMagicError> {
        let raw = u32::from_be_bytes(
            data.get(..std::mem::size_of::<Self>())
                .ok_or(ByteMagicError::NotEnoughData)?
                .try_into()
                .unwrap(),
        );

        Ok((
            char::from_u32(raw).ok_or(ByteMagicError::ParsingError {
                ty_name: "char",
                error: format!("invalid character: {raw:X}"),
            })?,
            std::mem::size_of::<Self>(),
        ))
    }
}

macro_rules! impl_bytemagic_for_numbers {
    [ $($t:ty),+ ] => {
        $(
            impl ByteMagic for $t {
                fn serialize(&self, output: &mut impl std::io::Write) -> usize {
                    output.write_all(&self.to_be_bytes()).unwrap();
                    std::mem::size_of::<Self>()
                }

                fn deserialize(data: &[u8]) -> Result<(Self, usize), ByteMagicError> {
                    Ok((
                        Self::from_be_bytes(
                            data.get(..std::mem::size_of::<Self>())
                                .ok_or(ByteMagicError::NotEnoughData)?
                                .try_into()
                                .unwrap(),
                        ),
                        std::mem::size_of::<Self>(),
                    ))
                }
            }
        )+

        #[test]
        fn test_numeric_bytemagic() {
            $(
                let n: $t = 69 as _;

                let v = n.serialize_buffered();
                let (m, bytes_read) = <$t as ByteMagic>::deserialize(&v).unwrap();

                assert_eq!(n, m);
                assert_eq!(bytes_read, v.len());
            )+
        }
    };
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
        Err(ByteMagicError::ParsingError {
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
