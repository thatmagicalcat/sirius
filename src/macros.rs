#[macro_export]
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

#[macro_export]
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
