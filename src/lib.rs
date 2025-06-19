use thiserror::Error;

mod impls;
mod macros;

pub use sirius_macros::Sirius;

/// The type that will be used to store the length of the slice.
pub type LengthPrefix = u32;

/// Number of bytes used to store the length of the slice.
const LENGTH_BYTES: usize = std::mem::size_of::<LengthPrefix>();

/// A trait for **data structures** that can be serialized or deserialized into binary.
///
/// To make the process faster, it tries to avoid allocations as much as possible,
/// this is why [serialize] function takes a `&mut impl Write`. Alternatively, you can use
/// the [serialize_buffered] function.
pub trait Sirius {
    /// Write the serialized data to output and return the bytes written
    fn serialize(&self, output: &mut Vec<u8>) -> usize;
    fn deserialize(data: &[u8]) -> Result<(Self, usize), SiriusError>
    where
        Self: Sized;

    fn serialize_buffered(&self) -> Vec<u8> {
        let mut data = vec![];
        _ = Sirius::serialize(self, &mut data);
        data
    }    
}

#[derive(Debug, Error)]
pub enum SiriusError {
    #[error("ran out of data bytes while parsing, cannot deserialize the remaining fields")]
    NotEnoughData,

    #[error("failed to parse data as `{ty_name}`: {error}")]
    ParsingError {
        ty_name: &'static str,
        error: String,
    },
}
