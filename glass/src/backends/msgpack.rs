//! MessagePack Functions
//! These functions can be used to allow an object to perform MessagePack actions.
use std::error::Error;
use uuid::Uuid;

/// Converts an object that implements Serialize into a vector of bytes.
pub fn object_to_bytes<T>(object: (Uuid, T)) -> Result<Vec<u8>, Box<dyn Error>>
where
    T: serde::ser::Serialize,
{
    Ok(rmp_serde::to_vec(&object)?)
}

/// Converts objects that implement Serialize into a vector of bytes.
pub fn objects_to_bytes<T>(objects: &[(Uuid, T)]) -> Result<Vec<u8>, Box<dyn Error>>
where
    T: serde::ser::Serialize,
{
    Ok(rmp_serde::to_vec(objects)?)
}

/// Converts a vector of bytes into an object that implement Deserialize.
pub fn bytes_to_object<'de, T>(bytes: &'de [u8]) -> Result<(Uuid, T), Box<dyn Error>>
where
    T: serde::de::Deserialize<'de>,
{
    Ok(rmp_serde::from_slice(bytes)?)
}

/// Converts a vector of bytes into objects that implement Deserialize.
pub fn bytes_to_objects<'de, T>(bytes: &'de [u8]) -> Result<Vec<(Uuid, T)>, Box<dyn Error>>
where
    T: serde::de::Deserialize<'de>,
{
    Ok(rmp_serde::from_slice(bytes)?)
}
