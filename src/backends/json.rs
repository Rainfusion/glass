//! JSON Functions
//! These functions can be used to allow an object to perform JSON actions.
use std::error::Error;
use uuid::Uuid;

/// Convert an object that implements Serialize to a String
pub fn object_to_string<T>(object: (Uuid, T)) -> Result<String, Box<Error>>
where
    T: serde::ser::Serialize,
{
    Ok(serde_json::to_string(&object)?)
}

/// Convert objects that implement Serialize to a String
pub fn objects_to_string<T>(objects: Vec<(Uuid, T)>) -> Result<String, Box<Error>>
where
    T: serde::ser::Serialize,
{
    Ok(serde_json::to_string(&objects)?)
}

/// Convert a JSON string into an object that implement Deserialize
pub fn string_to_object<'de, T>(string: &'de str) -> Result<(Uuid, T), Box<Error>>
where
    T: serde::de::Deserialize<'de>,
{
    Ok(serde_json::from_str(string)?)
}

/// Convert a JSON string into objects that implement Deserialize
pub fn string_to_objects<'de, T>(string: &'de str) -> Result<Vec<(Uuid, T)>, Box<Error>>
where
    T: serde::de::Deserialize<'de>,
{
    Ok(serde_json::from_str(string)?)
}
