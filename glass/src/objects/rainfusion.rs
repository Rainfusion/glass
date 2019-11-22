//! The RoR1 Mod object for the Rainfusion website.
#[cfg(feature = "json_backend")]
use crate::backends::json;
#[cfg(feature = "redis_backend")]
use crate::backends::redis;

use super::Sortable;
use glass_derive::*;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use uuid::Uuid;

/// The RoR1 Mod Object
#[derive(Serialize, Deserialize, Debug, Indexable, PartialEq, Default, Clone)]
pub struct Mod {
    pub name: Option<String>,
    pub author: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,

    #[serde(deserialize_with = "deserialize_type_field")]
    pub item_type: ModType,

    pub dependencies: Option<Vec<(Uuid, ModDependency)>>,
    pub tags: Option<Vec<String>>,
}

/// Mod Dependency Struct
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct ModDependency {
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

/// Implementation of the Sortable trait.
impl Sortable for Mod {
    type DataType = String;

    fn object_to_index() -> &'static str {
        "mods"
    }

    #[cfg(feature = "redis_backend")]
    #[cfg(feature = "json_backend")]
    fn map_to_object(map: Vec<(String, Self::DataType)>) -> Self {
        let values: Vec<String> = map.into_iter().map(|x| x.1).collect();
        let convert_value = |n: usize| -> Option<String> { values.get(n).cloned() };
        let collapse_string =
            |x: Option<String>| -> String { x.map_or("".to_string(), |y| y.to_string()) };

        let dependencies = match json::string_to_objects(&collapse_string(convert_value(6))) {
            Ok(x) => Some(x),
            Err(_) => None,
        };

        let tags = match serde_json::from_str(&collapse_string(convert_value(7))) {
            Ok(x) => Some(x),
            Err(_) => None,
        };

        if values.is_empty() {
            Self {
                ..Default::default()
            }
        } else {
            Self {
                name: convert_value(0),
                author: convert_value(1),
                summary: convert_value(2),
                description: convert_value(3),
                version: convert_value(4),
                item_type: ModType::from(collapse_string(convert_value(5))),
                dependencies,
                tags,
            }
        }
    }

    #[cfg(feature = "redis_backend")]
    #[cfg(feature = "json_backend")]
    fn object_to_map(&self) -> Vec<(String, Self::DataType)> {
        let collapse_string = |x: &Option<String>| -> String {
            x.as_ref().map_or("N/A".to_string(), |y| y.to_string())
        };

        let dependencies = match self.dependencies {
            None => &[],
            Some(ref x) => x.as_slice(),
        };

        let dependencies_string = match json::objects_to_string(dependencies) {
            Ok(x) => x,
            Err(_) => "".into(),
        };

        let tags = match serde_json::to_string(&self.tags) {
            Ok(x) => x,
            Err(_) => "".into(),
        };

        vec![
            ("name".into(), collapse_string(&self.name)),
            ("author".into(), collapse_string(&self.author)),
            ("summary".into(), collapse_string(&self.summary)),
            ("description".into(), collapse_string(&self.description)),
            ("version".into(), collapse_string(&self.version)),
            ("item_type".into(), String::from(self.item_type.clone())),
            ("dependencies".into(), dependencies_string),
            ("tags".into(), tags),
        ]
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
            summary: Some("Example Summary".to_owned()),
            description: Some("Example Description".to_owned()),
            version: Some("0.1.0".to_owned()),
            item_type: ModType::Mod,
            dependencies: Some(vec![
                (
                    Uuid::from_str("2b770fa6-749f-4aee-b49d-7bc4a0fe5dbe").unwrap(),
                    ModDependency {
                        version: "0.1.0".to_string(),
                    },
                ),
                (
                    Uuid::from_str("929189e7-41e1-4f28-9419-e6376003ae32").unwrap(),
                    ModDependency {
                        version: "0.1.0".to_string(),
                    },
                ),
            ]),
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

    // Bunch of tests to make sure YAML parses correctly for this object.
    #[cfg(feature = "yaml_backend")]
    mod yaml_tests {
        use super::*;
        use crate::backends::yaml;

        #[test]
        fn test_yaml_empty() {
            let serialized = yaml::object_to_string((generic_uuid(), Mod::default())).unwrap();
            let deserialized: (Uuid, Mod) = yaml::string_to_object(&serialized).unwrap();
            assert_eq!((generic_uuid(), Mod::default()), deserialized);
        }

        #[test]
        fn test_yaml_object() {
            let serialized = yaml::object_to_string((generic_uuid(), generic_mod())).unwrap();
            let deserialized: (Uuid, Mod) = yaml::string_to_object(&serialized).unwrap();
            assert_eq!((generic_uuid(), generic_mod()), deserialized);
        }

        #[test]
        fn test_yaml_empty_vec() {
            let data_vec: Vec<(Uuid, Mod)> = vec![
                (generic_uuid(), Mod::default()),
                (generic_uuid(), Mod::default()),
            ];
            let serialized = yaml::objects_to_string(&data_vec).unwrap();
            let deserialized: Vec<(Uuid, Mod)> = yaml::string_to_objects(&serialized).unwrap();
            assert_eq!(data_vec, deserialized);
        }

        #[test]
        fn test_yaml_object_vec() {
            let data_vec: Vec<(Uuid, Mod)> = vec![
                (generic_uuid(), generic_mod()),
                (generic_uuid(), generic_mod()),
            ];
            let serialized = yaml::objects_to_string(&data_vec).unwrap();
            let deserialized: Vec<(Uuid, Mod)> = yaml::string_to_objects(&serialized).unwrap();
            assert_eq!(data_vec, deserialized);
        }
    }

    // Bunch of tests to make sure Redis performs actions correctly for this object.
    #[cfg(feature = "redis_backend")]
    mod redis_tests {
        use super::*;
        use crate::backends::redis;
        use crate::objects::Sortable;
        use std::fmt::Debug;

        #[test]
        fn test_redis_object() {
            let mut connection = redis::RedisConfig {
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
                &mut connection,
                generic_mod(),
                Some(generic_uuid()),
            )
            .unwrap();
            assert_eq!(result, generic_uuid());

            // Check if Object can be retrieved successfully.
            let second_result: Vec<(String, String)> = redis::retrieve_object_from_database(
                &mut connection,
                generic_mod(),
                generic_uuid(),
            )
            .unwrap();

            let object = Mod::map_to_object(second_result);
            assert_eq!(object, generic_mod());

            // Delete Object from database.
            redis::remove_object_from_database(&mut connection, object, generic_uuid()).unwrap();
        }
    }
}
