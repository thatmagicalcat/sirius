use crate::*;

use std::io::Write;

impl<T: Sirius> Sirius for Vec<T> {
    fn serialize(&self, output: &mut impl Write) -> Result<usize, SiriusError> {
        if self.len() >= LengthPrefix::MAX as usize {
            panic!("length is greater than LengthPrefix::MAX");
        }

        output.write_all(&(self.len() as LengthPrefix).to_be_bytes())?;
        Ok(LENGTH_BYTES
            + self
                .iter()
                .map(|item| item.serialize(output))
                .sum::<Result<usize, SiriusError>>()?)
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SiriusError> {
        let mut offset = 0;
        let (data_len, bytes_read) = LengthPrefix::deserialize(data)?;
        let mut deserialized: Vec<T> = Vec::with_capacity(data_len as _);
        let ptr = deserialized.as_mut_ptr();

        offset += bytes_read;
        for i in 0..data_len {
            let (elem, bytes_read) =
                T::deserialize(data.get(offset..).ok_or(SiriusError::NotEnoughData)?)?;

            offset += bytes_read;

            // SAFETY: Vector is pre-allocated, so this is safe
            unsafe { ptr.add(i as _).write(elem) };
        }

        unsafe { deserialized.set_len(data_len as _) };

        Ok((deserialized, offset))
    }
}

impl<T: Sirius, const N: usize> Sirius for [T; N] {
    fn serialize(&self, output: &mut impl Write) -> Result<usize, SiriusError> {
        self.iter()
            .map(|i| i.serialize(output))
            .sum::<Result<usize, SiriusError>>()
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SiriusError> {
        let mut offset = 0;
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
    fn serialize(&self, output: &mut impl Write) -> Result<usize, SiriusError> {
        serialize_with_length_prefix(self.as_bytes(), output)
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SiriusError> {
        deserialize_with_length_prefix(data, |i, _| unsafe {
            let mut s = String::with_capacity(i.len());
            let ptr = s.as_bytes_mut().as_mut_ptr();

            std::ptr::copy_nonoverlapping(i.as_ptr(), ptr, i.len());
            s.as_mut_vec().set_len(i.len());

            s
        })
    }
}

impl<T: Sirius> Sirius for Box<T> {
    fn serialize(&self, output: &mut impl Write) -> Result<usize, SiriusError> {
        T::serialize(self, output)
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SiriusError> {
        T::deserialize(data).map(|(t, l)| (Box::new(t), l))
    }
}

fn serialize_with_length_prefix(
    slice: &[u8],
    output: &mut impl Write,
) -> Result<usize, SiriusError> {
    if slice.len() >= LengthPrefix::MAX as usize {
        panic!("size exceeded length prefix");
    }

    output.write_all(&(slice.len() as LengthPrefix).to_be_bytes())?;
    output.write_all(slice)?;

    Ok(slice.len() + LENGTH_BYTES)
}

fn deserialize_with_length_prefix<T, F: FnOnce(&[u8], usize) -> T>(
    data: &[u8],
    f: F,
) -> Result<(T, usize), SiriusError> {
    let len = u32::from_be_bytes(
        data.get(0..LENGTH_BYTES)
            .ok_or(SiriusError::NotEnoughData)?
            .try_into()
            .expect("slice length is always 4 bytes because of LENGHT_BYTES constant"),
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
    fn serialize(&self, output: &mut impl Write) -> Result<usize, SiriusError> {
        output.write_all(&(*self as u32).to_be_bytes())?;
        Ok(std::mem::size_of::<Self>())
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SiriusError> {
        let raw = u32::from_be_bytes(
            data.get(..std::mem::size_of::<Self>())
                .ok_or(SiriusError::NotEnoughData)?
                .try_into()
                .expect("slice length is always 4 bytes because of std::mem::size_of::<Self>() constant"),
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
    f32, f64, usize, isize
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
fn test_string_sirius() {
    let original = "The quick brown fox jumps over the lazy dog.".to_string();
    let serialized = original.serialize_buffered();
    let (deserialized, bytes_read) = String::deserialize(&serialized).unwrap();

    assert_eq!(deserialized, original);
    assert_eq!(bytes_read, serialized.len());
}

#[test]
fn test_vec_sirius() {
    let original = "The quick brown fox jumps over the lazy dog."
        .chars()
        .collect::<Vec<_>>();
    let serialized = original.serialize_buffered();

    let (deserialized, bytes_read) = Vec::<char>::deserialize(&serialized).unwrap();

    assert_eq!(deserialized, original);
    assert_eq!(bytes_read, serialized.len());
}
