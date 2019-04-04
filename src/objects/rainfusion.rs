//! The RoR1 Mod object for the Rainfusion website.
#[cfg(feature = "json_backend")]
use crate::backends::json;
#[cfg(feature = "msgpack_backend")]
use crate::backends::msgpack;
#[cfg(feature = "redis_backend")]
use crate::backends::redis;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// The RoR1 Mod Object
#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
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

/// ModType implementation, contains general functions for matching values.
impl ModType {
    /// Safely match ModType types into a usable string literal.
    /// Returns the name as a string literal if item exists in the object.
    /// Returns "mod" if item does not contain a ModType in the object.
    pub fn type_to_string(&self) -> &str {
        match self {
            ModType::Mod => "mod",
            ModType::Library => "lib",
        }
    }
}

/// Safely match a usable string literal into ModType.
pub fn string_to_type(object: &str) -> ModType {
    match object {
        "mod" => ModType::Mod,
        "lib" => ModType::Library,
        _ => ModType::Mod,
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

/// Display implementation for the Mod struct.
/// This function orders the values as they are in the struct.
/// Values that are "N/A" or "0" get displayed in their correct place.
impl std::fmt::Display for Mod {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Name: {}, 
             Author: {}, 
             Image URL: {}, 
             Summary: {}, 
             Description: {}, 
             Version: {}",
            self.match_name(),
            self.match_author(),
            self.match_url(),
            self.match_summary(),
            self.match_description(),
            self.match_version(),
        )
    }
}

/// Mod implementation, contains general functions for matching values.
impl Mod {
    /// Creates an empty Mod object.
    pub fn new() -> Self {
        Self {
            name: None,
            author: None,
            img_url: None,
            summary: None,
            description: None,
            version: None,
            item_type: ModType::Mod,
            dependencies: None,
        }
    }

    /// Converts a field map into an item object.
    #[cfg(feature = "redis_backend")]
    #[cfg(feature = "json_backend")]
    pub fn from_field_map(field_map: Vec<(String, String)>) -> Self {
        let values: Vec<String> = field_map.iter().map(|x| x.clone().1).collect();

        Self {
            name: match values.get(0) {
                None => None,
                Some(x) => Some(x.to_owned()),
            },
            author: match values.get(1) {
                None => None,
                Some(x) => Some(x.to_owned()),
            },
            img_url: match values.get(2) {
                None => None,
                Some(x) => Some(x.to_owned()),
            },
            summary: match values.get(3) {
                None => None,
                Some(x) => Some(x.to_owned()),
            },
            description: match values.get(4) {
                None => None,
                Some(x) => Some(x.to_owned()),
            },
            version: match values.get(5) {
                None => None,
                Some(x) => Some(x.to_owned()),
            },
            item_type: string_to_type(&values.get(6).unwrap()),
            dependencies: match json::string_to_objects(&values.get(7).unwrap()) {
                Ok(x) => Some(x),
                Err(_) => None,
            },
        }
    }

    /// Convert an object into a field map.
    /// Can be used when interfacing with a database.
    #[cfg(feature = "redis_backend")]
    #[cfg(feature = "json_backend")]
    pub fn convert_to_field_map(&self) -> Vec<(String, String)> {
        let mut output: Vec<(String, String)> = Vec::new();
        output.push(("name".to_owned(), self.match_name().to_owned()));
        output.push(("author".to_owned(), self.match_author().to_owned()));
        output.push(("img_url".to_owned(), self.match_url().to_owned()));
        output.push(("summary".to_owned(), self.match_summary().to_owned()));
        output.push((
            "description".to_owned(),
            self.match_description().to_owned(),
        ));
        output.push(("version".to_owned(), self.match_version().to_owned()));
        output.push((
            "item_type".to_owned(),
            self.item_type.type_to_string().to_owned(),
        ));
        output.push((
            "dependencies".to_owned(),
            serde_json::to_string(&self.dependencies).unwrap(),
        ));

        output
    }

    /// Safely match Name into a usable value.
    /// Returns name in the object if value exists.
    /// Returns N/A if no name exists.
    pub fn match_name(&self) -> &str {
        match self.name {
            Some(ref v) => v,
            None => "N/A",
        }
    }

    /// Safely match Author into a usable value.
    /// Returns the author in the object if value exists.
    /// Returns N/A if no author exists.
    pub fn match_author(&self) -> &str {
        match self.author {
            Some(ref v) => v,
            None => "N/A",
        }
    }

    /// Safely match Image URL into a usable URL.
    /// Returns the url in the object if value exists.
    /// Returns N/A as a URL if no url exists.
    pub fn match_url(&self) -> &str {
        match self.img_url {
            Some(ref v) => v,
            None => "N/A",
        }
    }

    /// Safely match Summary into a usable value.
    /// Returns summary string literal in object if value exists.
    /// Returns "N/A" as a string literal if no summary exists.
    pub fn match_summary(&self) -> &str {
        match self.summary {
            Some(ref v) => v,
            None => "N/A",
        }
    }

    /// Safely match Description into a usable value.
    /// Returns description string literal in object if value exists.
    /// Returns "N/A" as a string literal if no description exists.
    pub fn match_description(&self) -> &str {
        match self.description {
            Some(ref v) => v,
            None => "N/A",
        }
    }

    /// Safely match Version into a usable value.
    /// Returns version string literal in object if version exists.
    /// Returns "N/A" as a string literal if no version exists.
    pub fn match_version(&self) -> &str {
        match self.version {
            Some(ref v) => v.as_str(),
            None => "N/A",
        }
    }
}

// Bunch of tests to make sure JSON parses correctly for this object.
#[cfg(test)]
#[cfg(feature = "json_backend")]
mod json_tests {
    use super::{json, Mod, ModDependency, ModType};
    use std::str::FromStr;
    use uuid::Uuid;

    #[test]
    fn test_json_empty() {
        let data = Mod::default();
        let uuid: Uuid = Uuid::from_str("426497c2-1f94-4a75-889f-ecc04629da1d").unwrap();
        let serialized = json::object_to_string((uuid, data)).unwrap();
        assert_eq!(
            r#"["426497c2-1f94-4a75-889f-ecc04629da1d",{"name":null,"author":null,"img_url":null,"summary":null,"description":null,"version":null,"item_type":"Mod","dependencies":null}]"#,
            serialized
        );
    }

    #[test]
    fn test_json_object() {
        let data = Mod {
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
        };
        let uuid: Uuid = Uuid::from_str("426497c2-1f94-4a75-889f-ecc04629da1d").unwrap();
        let serialized = json::object_to_string((uuid, data)).unwrap();
        assert_eq!(
            r#"["426497c2-1f94-4a75-889f-ecc04629da1d",{"name":"Example Mod","author":"Example Author","img_url":"localhost","summary":"Example Summary","description":"Example Description","version":"0.0.1","item_type":"Mod","dependencies":[["2b770fa6-749f-4aee-b49d-7bc4a0fe5dbe",{"name":"Example Dependency","summary":"Example Summary","version":"0.0.1"}]]}]"#,
            serialized
        );
    }

    #[test]
    fn test_json_empty_vec() {
        let mut data_vec: Vec<(Uuid, Mod)> = Vec::new();
        let uuid: Uuid = Uuid::from_str("426497c2-1f94-4a75-889f-ecc04629da1d").unwrap();
        for _ in 0..2 {
            data_vec.push((uuid, Mod::default()));
        }
        let serialized = json::objects_to_string(data_vec).unwrap();
        assert_eq!(
            r#"[["426497c2-1f94-4a75-889f-ecc04629da1d",{"name":null,"author":null,"img_url":null,"summary":null,"description":null,"version":null,"item_type":"Mod","dependencies":null}],["426497c2-1f94-4a75-889f-ecc04629da1d",{"name":null,"author":null,"img_url":null,"summary":null,"description":null,"version":null,"item_type":"Mod","dependencies":null}]]"#,
            serialized
        );
    }

    #[test]
    fn test_json_object_vec() {
        let mut data_vec: Vec<(Uuid, Mod)> = Vec::new();
        let uuid: Uuid = Uuid::from_str("426497c2-1f94-4a75-889f-ecc04629da1d").unwrap();
        for _ in 0..2 {
            data_vec.push((
                uuid,
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
                },
            ))
        }
        let serialized = json::objects_to_string(data_vec).unwrap();
        assert_eq!(r#"[["426497c2-1f94-4a75-889f-ecc04629da1d",{"name":"Example Mod","author":"Example Author","img_url":"localhost","summary":"Example Summary","description":"Example Description","version":"0.0.1","item_type":"Mod","dependencies":[["2b770fa6-749f-4aee-b49d-7bc4a0fe5dbe",{"name":"Example Dependency","summary":"Example Summary","version":"0.0.1"}]]}],["426497c2-1f94-4a75-889f-ecc04629da1d",{"name":"Example Mod","author":"Example Author","img_url":"localhost","summary":"Example Summary","description":"Example Description","version":"0.0.1","item_type":"Mod","dependencies":[["2b770fa6-749f-4aee-b49d-7bc4a0fe5dbe",{"name":"Example Dependency","summary":"Example Summary","version":"0.0.1"}]]}]]"#, serialized);
    }

    #[test]
    fn test_string_to_empty() {
        let string = r#"["426497c2-1f94-4a75-889f-ecc04629da1d",{"name":null,"author":null,"img_url":null,"summary":null,"description":null,"version":null,"item_type":"Mod","dependencies":null}]"#;
        let uuid: Uuid = Uuid::from_str("426497c2-1f94-4a75-889f-ecc04629da1d").unwrap();
        let deserialized: (Uuid, Mod) = json::string_to_object(string).unwrap();
        assert_eq!((uuid, Mod::default()), deserialized);
    }

    #[test]
    fn test_string_to_object() {
        let string = r#"["426497c2-1f94-4a75-889f-ecc04629da1d",{"name":"Example Mod","author":"Example Author","img_url":"localhost","summary":"Example Summary","description":"Example Description","version":"0.0.1","item_type":"Mod","dependencies":[["2b770fa6-749f-4aee-b49d-7bc4a0fe5dbe",{"name":"Example Dependency","summary":"Example Summary","version":"0.0.1"}]]}]"#;
        let uuid: Uuid = Uuid::from_str("426497c2-1f94-4a75-889f-ecc04629da1d").unwrap();
        let deserialized: (Uuid, Mod) = json::string_to_object(string).unwrap();
        assert_eq!(
            (
                uuid,
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
                        }
                    )]),
                }
            ),
            deserialized
        );
    }

    #[test]
    fn test_string_to_empty_vec() {
        let string = r#"[["426497c2-1f94-4a75-889f-ecc04629da1d",{"name":null,"author":null,"img_url":null,"summary":null,"description":null,"version":null,"item_type":"mod"}],["426497c2-1f94-4a75-889f-ecc04629da1d",{"name":null,"author":null,"img_url":null,"summary":null,"description":null,"version":null,"item_type":"mod"}]]"#;
        let uuid: Uuid = Uuid::from_str("426497c2-1f94-4a75-889f-ecc04629da1d").unwrap();
        let deserialized: Vec<(Uuid, Mod)> = json::string_to_objects(string).unwrap();
        assert_eq!(
            vec![(uuid, Mod::default()), (uuid, Mod::default())],
            deserialized
        );
    }

    #[test]
    fn test_string_to_object_vec() {
        let string = r#"[["426497c2-1f94-4a75-889f-ecc04629da1d",{"name":"Example Mod","author":"Example Author","img_url":"localhost","summary":"Example Summary","description":"Example Description","version":"0.0.1","item_type":"Mod","dependencies":[["2b770fa6-749f-4aee-b49d-7bc4a0fe5dbe",{"name":"Example Dependency","summary":"Example Summary","version":"0.0.1"}]]}],["426497c2-1f94-4a75-889f-ecc04629da1d",{"name":"Example Mod","author":"Example Author","img_url":"localhost","summary":"Example Summary","description":"Example Description","version":"0.0.1","item_type":"Mod","dependencies":[["2b770fa6-749f-4aee-b49d-7bc4a0fe5dbe",{"name":"Example Dependency","summary":"Example Summary","version":"0.0.1"}]]}]]"#;
        let uuid: Uuid = Uuid::from_str("426497c2-1f94-4a75-889f-ecc04629da1d").unwrap();
        let deserialized: Vec<(Uuid, Mod)> = json::string_to_objects(string).unwrap();
        assert_eq!(
            vec![
                (
                    uuid,
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
                            }
                        )]),
                    }
                ),
                (
                    uuid,
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
                            }
                        )]),
                    }
                )
            ],
            deserialized
        );
    }
}

// Bunch of tests to make sure MessagePack converts properly for this object.
#[cfg(test)]
#[cfg(feature = "msgpack_backend")]
mod msgpack_tests {
    use super::{msgpack, Mod, ModDependency, ModType};
    use std::str::FromStr;
    use uuid::Uuid;

    #[test]
    fn test_msgpack_empty() {
        let data = Mod::default();
        let uuid: Uuid = Uuid::from_str("426497c2-1f94-4a75-889f-ecc04629da1d").unwrap();
        let serialized = msgpack::object_to_bytes(&(uuid, data)).unwrap();
        let deserialized: (Uuid, Mod) = msgpack::bytes_to_object(&serialized).unwrap();
        assert_eq!((uuid, Mod::default()), deserialized);
    }

    #[test]
    fn test_msgpack_object() {
        let data = Mod {
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
        };
        let uuid: Uuid = Uuid::from_str("426497c2-1f94-4a75-889f-ecc04629da1d").unwrap();
        let serialized = msgpack::object_to_bytes(&(uuid, data)).unwrap();
        let deserialized: (Uuid, Mod) = msgpack::bytes_to_object(&serialized).unwrap();
        assert_eq!(
            (
                uuid,
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
                        }
                    )]),
                }
            ),
            deserialized
        );
    }

    #[test]
    fn test_msgpack_empty_vec() {
        let mut data_vec: Vec<(Uuid, Mod)> = Vec::new();
        let uuid: Uuid = Uuid::from_str("426497c2-1f94-4a75-889f-ecc04629da1d").unwrap();
        for _ in 0..2 {
            data_vec.push((uuid, Mod::default()));
        }
        let serialized = msgpack::objects_to_bytes(&data_vec).unwrap();
        let deserialized: Vec<(Uuid, Mod)> = msgpack::bytes_to_objects(&serialized).unwrap();
        assert_eq!(data_vec, deserialized);
    }

    #[test]
    fn test_msgpack_object_vec() {
        let mut data_vec: Vec<(Uuid, Mod)> = Vec::new();
        let uuid: Uuid = Uuid::from_str("426497c2-1f94-4a75-889f-ecc04629da1d").unwrap();
        for _ in 0..2 {
            data_vec.push((
                uuid,
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
                },
            ))
        }
        let serialized = msgpack::objects_to_bytes(&data_vec).unwrap();
        let deserialized: Vec<(Uuid, Mod)> = msgpack::bytes_to_objects(&serialized).unwrap();
        assert_eq!(data_vec, deserialized);
    }
}

// Bunch of tests to make sure Redis performs actions correctly for this object.
#[cfg(test)]
#[cfg(feature = "redis_backend")]
mod redis_tests {
    use super::{json, redis, string_to_type, Mod, ModDependency, ModType};
    use std::str::FromStr;
    use uuid::Uuid;

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

        let uuid: Uuid = Uuid::from_str("426497c2-1f94-4a75-889f-ecc04629da1d").unwrap();

        let data = Mod {
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
        };

        // First Insert Object into database.
        let result: Uuid = redis::insert_object_into_database(
            &connection,
            "mods",
            data.convert_to_field_map(),
            Some(uuid),
        )
        .unwrap();
        assert_eq!(result, uuid);

        // Check if Object can be retrieved successfully.
        let field_map: Vec<String> = vec![
            "name".to_string(),
            "author".to_string(),
            "img_url".to_string(),
            "summary".to_string(),
            "description".to_string(),
            "version".to_string(),
            "item_type".to_string(),
            "dependencies".to_string(),
        ];

        let second_result: Vec<(String, String)> =
            redis::retrieve_object_from_database(&connection, "mods", &field_map, uuid).unwrap();

        let mut values = Vec::new();
        for i in second_result {
            values.push(i.1);
        }

        let object = Mod {
            name: Some(values[0].to_owned()),
            author: Some(values[1].to_owned()),
            img_url: Some(values[2].to_owned()),
            summary: Some(values[3].to_owned()),
            description: Some(values[4].to_owned()),
            version: Some(values[5].to_owned()),
            item_type: string_to_type(&values[6].to_owned()),
            dependencies: Some(json::string_to_objects(&values[7].to_owned()).unwrap()),
        };
        assert_eq!(object, data);

        // Delete Object from database.
        redis::remove_object_from_database(&connection, "mods", &field_map, uuid).unwrap();
    }
}
