mod de;
mod error;
mod ser;

pub use crate::de::{/*from_bytes, */Deserializer};
pub use crate::error::{Error, Result};
pub use crate::ser::{to_bytes, Serializer};
