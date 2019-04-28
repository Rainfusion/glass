//! The RoR1 Mod object for the Rainfusion website.
#[cfg(feature = "json_backend")]
use crate::backends::json;
#[cfg(feature = "msgpack_backend")]
use crate::backends::msgpack;
#[cfg(feature = "redis_backend")]
use crate::backends::redis;

use glass_derive::*;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// The RoR1 Mod Object
#[derive(Serialize, Deserialize, Debug, PartialEq, Default, Indexable, Clone)]
pub struct Mod {
    pub name: Option<String>,
    pub author: Option<String>,
    pub img_url: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,

    #[serde(deserialize_with = "deserialize_type_field")]
    pub item_type: ModType,

    pub dependencies: Option<Vec<(Uuid, ModDependency)>>,
    pub tags: Option<Vec<String>>,
}

/// ModDependency enum, all values in this enum are mod types.
/// All types match into string literals.
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct ModDependency {
    pub name: String,
    pub summary: String,
    pub version: String,
}

/// ModType enum, all values in this enum are item types.
/// All types match into string literals.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub enum ModType {
    Mod,
    Library,
}

/// Quick implementation of Default for ModType
impl Default for ModType {
    fn default() -> Self {
        ModType::Mod
    }
}

/// Match a ModType into a string literal.
impl From<ModType> for String {
    fn from(item: ModType) -> Self {
        match item {
            ModType::Mod => "mod".to_owned(),
            ModType::Library => "lib".to_owned(),
        }
    }
}

/// Match a usable string literal into ModType.
impl From<String> for ModType {
    fn from(string: String) -> Self {
        match string.as_str() {
            "mod" => ModType::Mod,
            "lib" => ModType::Library,
            _ => ModType::Mod,
        }
    }
}

/// Implementation of From to convert a field map into a Mod object.
#[cfg(feature = "redis_backend")]
#[cfg(feature = "json_backend")]
impl From<Vec<(String, String)>> for Mod {
    fn from(field_map: Vec<(String, String)>) -> Self {
        let values: Vec<String> = field_map.iter().map(|x| x.clone().1).collect();

        Self {
            name: values.get(0).and_then(|x| Some(x.to_owned())),
            author: values.get(1).and_then(|x| Some(x.to_owned())),
            img_url: values.get(2).and_then(|x| Some(x.to_owned())),
            summary: values.get(3).and_then(|x| Some(x.to_owned())),
            description: values.get(4).and_then(|x| Some(x.to_owned())),
            version: values.get(5).and_then(|x| Some(x.to_owned())),
            item_type: ModType::from(values[6].clone()),
            dependencies: match json::string_to_objects(&values[7]) {
                Ok(x) => Some(x),
                Err(_) => None,
            },
            tags: match serde_json::from_str(&values[8]) {
                Ok(x) => Some(x),
                Err(_) => None,
            },
        }
    }
}

/// Implementation of From to convert a Mod into a field map.
#[cfg(feature = "redis_backend")]
#[cfg(feature = "json_backend")]
impl From<Mod> for Vec<(String, String)> {
    fn from(object: Mod) -> Vec<(String, String)> {
        let mut output: Vec<(String, String)> = Vec::new();
        let collapse_string =
            |x: Option<String>| -> String { x.map_or("N/A".to_string(), |y| y.to_string()) };

        output.push(("name".to_owned(), collapse_string(object.name.clone())));
        output.push(("author".to_owned(), collapse_string(object.author.clone())));
        output.push((
            "img_url".to_owned(),
            collapse_string(object.img_url.clone()),
        ));
        output.push((
            "summary".to_owned(),
            collapse_string(object.summary.clone()),
        ));
        output.push((
            "description".to_owned(),
            collapse_string(object.description.clone()),
        ));
        output.push((
            "version".to_owned(),
            collapse_string(object.version.clone()),
        ));
        output.push(("item_type".to_owned(), String::from(object.item_type)));
        output.push((
            "dependencies".to_owned(),
            json::objects_to_string(match object.dependencies {
                Some(ref x) => x,
                None => &[],
            })
            .unwrap(),
        ));
        output.push((
            "tags".to_owned(),
            serde_json::to_string(&object.tags).unwrap(),
        ));

        output
    }
}

/// Custom parsing function for "item_type" string value into ModType Enum using Serde.
/// If value does not exist on a object it will return "mod" for the variable.
fn deserialize_type_field<'de, D>(de: D) -> Result<ModType, D::Error>
where
    D: Deserializer<'de>,
{
    let result: Value = Deserialize::deserialize(de)?;
    match result {
        Value::String(ref s) if &*s == "Mod" => Ok(ModType::Mod),
        Value::String(ref s) if &*s == "Lib" => Ok(ModType::Library),
        _ => Ok(ModType::Mod),
    }
}

#[cfg(test)]
mod tests {
    use super::{Mod, ModDependency, ModType};
    use std::str::FromStr;
    use uuid::Uuid;

    fn generic_uuid() -> Uuid {
        Uuid::from_str("426497c2-1f94-4a75-889f-ecc04629da1d").unwrap()
    }

    fn generic_mod() -> Mod {
        Mod {
            name: Some("Example Mod".to_owned()),
            author: Some("Example Author".to_owned()),
            img_url: Some("localhost".to_owned()),
            summary: Some("Example Summary".to_owned()),
            description: Some("Example Description".to_owned()),
            version: Some("0.0.1".to_owned()),
            item_type: ModType::Mod,
            dependencies: Some(vec![(
                Uuid::from_str("2b770fa6-749f-4aee-b49d-7bc4a0fe5dbe").unwrap(),
                ModDependency {
                    name: "Example Dependency".to_string(),
                    summary: "Example Summary".to_string(),
                    version: "0.0.1".to_string(),
                },
            )]),
            tags: Some(vec!["test".to_owned(), "test2".to_owned()]),
        }
    }

    // Bunch of tests to make sure JSON parses correctly for this object.
    #[cfg(feature = "json_backend")]
    mod json_tests {
        use super::*;
        use crate::backends::json;

        #[test]
        fn test_json_empty() {
            let serialized = json::object_to_string((generic_uuid(), Mod::default())).unwrap();
            let deserialized: (Uuid, Mod) = json::string_to_object(&serialized).unwrap();
            assert_eq!((generic_uuid(), Mod::default()), deserialized);
        }

        #[test]
        fn test_json_object() {
            let serialized = json::object_to_string((generic_uuid(), generic_mod())).unwrap();
            let deserialized: (Uuid, Mod) = json::string_to_object(&serialized).unwrap();
            assert_eq!((generic_uuid(), generic_mod()), deserialized);
        }

        #[test]
        fn test_json_empty_vec() {
            let data_vec: Vec<(Uuid, Mod)> = vec![
                (generic_uuid(), Mod::default()),
                (generic_uuid(), Mod::default()),
            ];
            let serialized = json::objects_to_string(&data_vec).unwrap();
            let deserialized: Vec<(Uuid, Mod)> = json::string_to_objects(&serialized).unwrap();
            assert_eq!(data_vec, deserialized);
        }

        #[test]
        fn test_json_object_vec() {
            let data_vec: Vec<(Uuid, Mod)> = vec![
                (generic_uuid(), generic_mod()),
                (generic_uuid(), generic_mod()),
            ];
            let serialized = json::objects_to_string(&data_vec).unwrap();
            let deserialized: Vec<(Uuid, Mod)> = json::string_to_objects(&serialized).unwrap();
            assert_eq!(data_vec, deserialized);
        }
    }

    // Bunch of tests to make sure MessagePack converts properly for this object.
    #[cfg(feature = "msgpack_backend")]
    mod msgpack_tests {
        use super::*;
        use crate::backends::msgpack;

        #[test]
        fn test_msgpack_empty() {
            let serialized = msgpack::object_to_bytes((generic_uuid(), Mod::default())).unwrap();
            let deserialized: (Uuid, Mod) = msgpack::bytes_to_object(&serialized).unwrap();
            assert_eq!((generic_uuid(), Mod::default()), deserialized);
        }

        #[test]
        fn test_msgpack_object() {
            let serialized = msgpack::object_to_bytes((generic_uuid(), generic_mod())).unwrap();
            let deserialized: (Uuid, Mod) = msgpack::bytes_to_object(&serialized).unwrap();
            assert_eq!((generic_uuid(), generic_mod()), deserialized);
        }

        #[test]
        fn test_msgpack_empty_vec() {
            let data_vec: Vec<(Uuid, Mod)> = vec![
                (generic_uuid(), Mod::default()),
                (generic_uuid(), Mod::default()),
            ];
            let serialized = msgpack::objects_to_bytes(&data_vec).unwrap();
            let deserialized: Vec<(Uuid, Mod)> = msgpack::bytes_to_objects(&serialized).unwrap();
            assert_eq!(data_vec, deserialized);
        }

        #[test]
        fn test_msgpack_object_vec() {
            let data_vec: Vec<(Uuid, Mod)> = vec![
                (generic_uuid(), generic_mod()),
                (generic_uuid(), generic_mod()),
            ];
            let serialized = msgpack::objects_to_bytes(&data_vec).unwrap();
            let deserialized: Vec<(Uuid, Mod)> = msgpack::bytes_to_objects(&serialized).unwrap();
            assert_eq!(data_vec, deserialized);
        }
    }

    // Bunch of tests to make sure Redis performs actions correctly for this object.
    #[cfg(feature = "redis_backend")]
    mod redis_tests {
        use super::*;
        use crate::backends::redis;

        #[test]
        fn test_redis_object() {
            let connection = redis::RedisConfig {
                database_ip: Some("127.0.0.1".to_owned()),
                database_port: Some(6379),
                database_socket: None,
                database_id: 0,
                database_password: None,
            }
            .form_connection()
            .unwrap();

            // First Insert Object into database.
            let result: Uuid = redis::insert_object_into_database(
                &connection,
                "mods",
                From::from(generic_mod()),
                Some(generic_uuid()),
            )
            .unwrap();
            assert_eq!(result, generic_uuid());

            // Check if Object can be retrieved successfully.
            let second_result: Vec<(String, String)> = redis::retrieve_object_from_database(
                &connection,
                "mods",
                Mod::fields(),
                generic_uuid(),
            )
            .unwrap();

            let object = Mod::from(second_result);
            assert_eq!(object, generic_mod());

            // Delete Object from database.
            redis::remove_object_from_database(&connection, "mods", Mod::fields(), generic_uuid())
                .unwrap();
        }
    }
}
