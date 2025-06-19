use crate::*;

impl<T: Sirius> Sirius for Vec<T> {
    fn serialize(&self, output: &mut Vec<u8>) -> usize {
        if self.len() >= LengthPrefix::MAX as usize {
            panic!("length is greater than LengthPrefix::MAX");
        }

        output.extend_from_slice(&(self.len() as LengthPrefix).to_be_bytes());

        // FIXME: replace with a size_hint function in the future as the allocation
        // size is not correct
        output.reserve(self.len() * std::mem::size_of::<T>());

        self.iter().map(|i| i.serialize(output)).sum::<usize>() + LENGTH_BYTES
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SiriusError> {
        let mut offset = 0;
        let (data_len, bytes_read) = LengthPrefix::deserialize(data)?;
        offset += bytes_read;

        let mut deserialized: Vec<T> = Vec::with_capacity(data_len as _);
        dbg!(data_len);
        for _ in 0..data_len {
            let (elem, bytes_read) =
                T::deserialize(data.get(offset..).ok_or(SiriusError::NotEnoughData)?)?;

            offset += bytes_read;
            deserialized.push(elem);
        }

        Ok((deserialized, offset))
    }
}

impl<T: Sirius, const N: usize> Sirius for [T; N] {
    fn serialize(&self, output: &mut Vec<u8>) -> usize {
        if N >= LengthPrefix::MAX as usize {
            panic!("length is greater than LengthPrefix::MAX");
        }

        output.extend_from_slice(&(self.len() as LengthPrefix).to_be_bytes());

        // FIXME: replace with a size_hint function in the future as the allocation
        // size is not correct
        output.reserve(self.len() * std::mem::size_of::<T>());

        self.iter().map(|i| i.serialize(output)).sum::<usize>() + LENGTH_BYTES
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SiriusError> {
        let mut offset = 0;
        let (data_len, bytes_read) = LengthPrefix::deserialize(data)?;
        offset += bytes_read;

        assert_eq!(data_len, N as _);

        let mut deserialized: [T; N] = unsafe { std::mem::zeroed() };
        for i in deserialized.iter_mut() {
            let (elem, bytes_read) =
                T::deserialize(data.get(offset..).ok_or(SiriusError::NotEnoughData)?)?;

            offset += bytes_read;
            *i = elem;
        }

        Ok((deserialized, offset))
    }
}

impl Sirius for String {
    fn serialize(&self, output: &mut Vec<u8>) -> usize {
        serialize_with_length_prefix(self.as_bytes(), output)
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SiriusError> {
        let deserialized =
            deserialize_with_length_prefix(data, |i, _| String::from_utf8(i.to_vec()));

        if let Ok((ref d, _)) = deserialized {
            if let Err(e) = d {
                return Err(SiriusError::ParsingError {
                    ty_name: "String",
                    error: e.to_string(),
                });
            }
        }

        deserialized.map(|(d, size)| (d.unwrap(), size))
    }
}

impl<T: Sirius> Sirius for Box<T> {
    fn serialize(&self, output: &mut Vec<u8>) -> usize {
        T::serialize(self, output)
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SiriusError> {
        T::deserialize(data).map(|(t, l)| (Box::new(t), l))
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

impl_sirius_for_numbers! {
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
    f32, f64
}

#[test]
fn test_char_sirius() {
    let original = '💯';
    let serialized = original.serialize_buffered();
    let (deserialized, bytes_read) = char::deserialize(&serialized).unwrap();

    assert_eq!(deserialized, original);
    assert_eq!(bytes_read, serialized.len());
}

#[test]
fn test_array_sirius() {
    let mut data: [u32; 100] = [69 as _; 100];
    data.iter_mut().enumerate().for_each(|(idx, itm)| {
        *itm = idx as _;
    });

    let v = data.serialize_buffered();
    let (n, bytes_read) = <[u32; 100] as Sirius>::deserialize(&v).unwrap();

    eprintln!("orig: {data:?}\ndeserialized: {n:?}");

    assert!(data.iter().zip(n.iter()).all(|(&a, &b)| a == b));
    assert_eq!(bytes_read, v.len());
}

#[test]
fn test_char_sirius_check() {
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
fn test_vec_sirius() {
    let original = "The quick brown fox jumps over the lazy dog.".chars().collect::<Vec<_>>();
    let serialized = original.serialize_buffered();
    dbg!(serialized.len());

    let (deserialized, bytes_read) = Vec::<char>::deserialize(&serialized).unwrap();

    assert_eq!(deserialized, original);
    assert_eq!(bytes_read, serialized.len());
}
