use serde::{Deserialize, Serialize};

use super::{Codec, DeserializationError, SerializationError};

/// CBOR codec using [ciborium].
///
/// ## Compatibility
/// This codec is able to decode values encoded with `codec-cbor`
/// but the opposite is not true. Make sure you are using `codec-ciborium`
/// in both client and servers to avoid deserialization errors.
/// More information in the [`ciborium` README].
///
/// [`ciborium` README]: https://github.com/enarx/ciborium#compatibility-with-other-implementations
#[cfg_attr(docsrs, doc(cfg(feature = "codec-ciborium")))]
#[derive(Clone, Serialize, Deserialize)]
pub struct Ciborium;

impl Codec for Ciborium {
    #[inline]
    fn serialize<Writer, Item>(writer: Writer, item: &Item) -> Result<(), super::SerializationError>
    where
        Writer: std::io::Write,
        Item: serde::Serialize,
    {
        ciborium::ser::into_writer(item, writer).map_err(SerializationError::new)
    }

    #[inline]
    fn deserialize<Reader, Item>(reader: Reader) -> Result<Item, super::DeserializationError>
    where
        Reader: std::io::Read,
        Item: serde::de::DeserializeOwned,
    {
        ciborium::de::from_reader(reader).map_err(DeserializationError::new)
    }
}