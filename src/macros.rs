#[macro_export]
macro_rules! impl_sirius_for_numbers {
    [ $($t:ty),+ $(,)? ] => {
        $(
            impl Sirius for $t {
                fn serialize(&self, output: &mut impl std::io::Write) -> Result<usize, SiriusError> {
                    output.write_all(&self.to_be_bytes())?;
                    Ok(std::mem::size_of::<Self>())
                }

                fn deserialize(data: &[u8]) -> Result<(Self, usize), SiriusError> {
                    Ok((
                        Self::from_be_bytes(
                            data.get(..std::mem::size_of::<Self>())
                                .ok_or(SiriusError::NotEnoughData)?
                                .try_into()
                                .expect("slice length is std::mem::size_of::<Self>() bytes"),
                        ),
                        std::mem::size_of::<Self>(),
                    ))
                }
            }
        )+

        #[test]
        fn test_numeric_sirius() {
            $(
                let n: $t = 69 as _;

                let v = n.serialize_buffered();
                let (m, bytes_read) = <$t as Sirius>::deserialize(&v).unwrap();

                assert_eq!(n, m);
                assert_eq!(bytes_read, v.len());
            )+
        }
    };
}
