//! YAML Functions
//! These functions can be used to allow an object to perform YAML actions.
use std::error::Error;
use uuid::Uuid;

/// Convert an object that implements Serialize to a String
pub fn object_to_string<T>(object: (Uuid, T)) -> Result<String, Box<Error>>
where
    T: serde::ser::Serialize,
{
    Ok(serde_yaml::to_string(&object)?)
}

/// Convert objects that implement Serialize to a String
pub fn objects_to_string<T>(objects: &[(Uuid, T)]) -> Result<String, Box<Error>>
where
    T: serde::ser::Serialize,
{
    Ok(serde_yaml::to_string(&objects)?)
}

/// Convert a YAML string into an object that implement Deserialize
pub fn string_to_object<'a, T>(string: &'a str) -> Result<(Uuid, T), Box<Error>>
where
    T: serde::de::DeserializeOwned,
{
    Ok(serde_yaml::from_str(string)?)
}

/// Convert a YAML string into objects that implement Deserialize
pub fn string_to_objects<'a, T>(string: &'a str) -> Result<Vec<(Uuid, T)>, Box<Error>>
where
    T: serde::de::DeserializeOwned,
{
    Ok(serde_yaml::from_str(string)?)
}
