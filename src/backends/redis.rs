//! Redis Functions and Config
//! These functions can be used to allow an object to perform Redis database actions.
//! The configuration can be used to generate a connection to the database.

use crate::objects::Sortable;
use redis::{Client, Commands, Connection, ConnectionAddr, ConnectionInfo};
use serde::Deserialize;
use std::{collections::HashMap, error::Error, path::PathBuf, str::FromStr};
use uuid::Uuid;

/// Custom Type Definitions
type FieldMap<T> = HashMap<String, T>;
type RedisResult<T> = Result<Vec<(Uuid, FieldMap<T>)>, Box<dyn Error>>;

/// Redis Connection Config
/// Supports both TCP and Socket connections.
#[derive(Deserialize, Debug, Clone)]
pub struct RedisConfig {
    // TCP
    pub database_ip: Option<String>,
    pub database_port: Option<u16>,

    // Socket
    pub database_socket: Option<String>,

    // General database information
    pub database_id: i64,
    pub database_password: Option<String>,
}

/// Function to parse a RedisConfig from a JSON file in a folder.
pub fn parse_redis_config<T: AsRef<std::path::Path>>(
    path: T,
) -> Result<RedisConfig, Box<dyn Error>> {
    let file = std::fs::File::open(path)?;
    Ok(serde_json::from_reader(file)?)
}

impl Default for RedisConfig {
    fn default() -> Self {
        RedisConfig {
            database_ip: Some(String::from("localhost")),
            database_port: Some(6379),
            database_socket: None,
            database_id: 0,
            database_password: None,
        }
    }
}

impl RedisConfig {
    /// Creates a connection to the Redis database using the RedisConfig
    pub fn form_connection(self) -> Result<Connection, Box<dyn Error>> {
        if self.database_socket.is_none() {
            // Handle TCP Connection

            Ok(Client::open(ConnectionInfo {
                addr: Box::new(ConnectionAddr::Tcp(
                    self.database_ip
                        .clone()
                        .map_or("localhost".to_owned(), |x| x),
                    self.database_port.map_or(6379, |x| x),
                )),
                db: self.database_id,
                passwd: self.database_password.clone().and_then(Some),
            })?
            .get_connection()?)
        } else {
            // Handle Socket Connection

            Ok(Client::open(ConnectionInfo {
                addr: Box::new(ConnectionAddr::Unix(PathBuf::from_str(
                    self.database_socket
                        .clone()
                        .map_or("/tmp/redis.sock".to_owned(), |x| x)
                        .as_str(),
                )?)),
                db: self.database_id,
                passwd: self.database_password.clone().and_then(Some),
            })?
            .get_connection()?)
        }
    }
}

/// Function to insert an object into a local Redis database.
/// Returns the UUID of where the object is on the database.
pub fn insert_object_into_database<O>(
    connection: &mut Connection,
    object: O,
    uuid: Option<Uuid>,
) -> Result<Uuid, Box<dyn Error>>
where
    O: Sortable,
{
    // Generate UUID or use provided one.
    let gen_key = match uuid {
        Some(k) => k,
        None => Uuid::new_v4(),
    };

    // Get Object Variables
    let field_map: Vec<(String, O::DataType)> = object.object_to_map();
    let index = O::object_to_index();

    // Find next index in table.
    let count: i32 = connection.zcard(&format!("{}-index", index))?;

    // Generate a command pipeline.
    let mut pipeline = redis::Pipeline::new();

    // Add UUID to index for object.
    pipeline.add_command(
        redis::cmd("ZADD")
            .arg(&format!("{}-index", index))
            .arg(count + 1)
            .arg(&gen_key.to_simple().to_string())
            .to_owned(),
    );

    // Iterate through map to find fields that need to be populated and generate a command for them.
    field_map.into_iter().for_each(|item| {
        pipeline.add_command(
            redis::cmd("HSET")
                .arg(&format!("{}:{}", index, &gen_key.to_simple().to_string()))
                .arg(item.0)
                .arg(item.1)
                .to_owned(),
        );
    });

    // Finally send commands to database.
    pipeline.query(connection)?;

    Ok(gen_key)
}

/// Function to remove an object from a local Redis database.
pub fn remove_object_from_database<O>(
    connection: &mut Connection,
    uuid: Uuid,
) -> Result<(), Box<dyn Error>>
where
    O: Sortable,
{
    // Get Object Index
    let index = O::object_to_index();
    let index_id = format!("{}:{}", index, &uuid.to_simple().to_string());

    // Generate a field map for the object.
    let map: Vec<String> = connection.hkeys(&index_id)?;

    // Remove uuid in table.
    let _res: i32 = connection.zrem(format!("{}-index", index), uuid.to_simple().to_string())?;

    // Generate a command pipeline.
    let mut pipeline = redis::Pipeline::new();

    // Iterate through map to find fields that need to be removed and generate a command for them.
    map.into_iter().for_each(|item| {
        pipeline.add_command(redis::cmd("HDEL").arg(&index_id).arg(item).to_owned());
    });

    // Finally send commands to database.
    pipeline.query(connection)?;

    Ok(())
}

/// Function to edit a field in an object in a local Redis database.
pub fn edit_object_from_database<O>(
    connection: &mut Connection,
    changes: Vec<(String, O::DataType)>,
    uuid: Uuid,
) -> Result<(), Box<dyn Error>>
where
    O: Sortable,
{
    // Generate a command pipeline.
    let mut pipeline = redis::Pipeline::new();

    // Get Object Variables
    let index = O::object_to_index();

    // Iterate through map to find fields that need to be edited and generate a command for them.
    changes.into_iter().for_each(|item| {
        pipeline.add_command(
            redis::cmd("HSET")
                .arg(&format!("{}:{}", index, &uuid.to_simple().to_string()))
                .arg(item.0)
                .arg(item.1)
                .to_owned(),
        );
    });

    // Finally send commands to database.
    pipeline.query(connection)?;

    Ok(())
}

/// Function to retrieve a object in a local Redis database.
pub fn retrieve_object_from_database<O>(
    connection: &mut Connection,
    uuid: Uuid,
) -> Result<FieldMap<O::DataType>, Box<dyn Error>>
where
    O: Sortable,
{
    // Get Object Index
    let index = format!("{}:{}", O::object_to_index(), &uuid.to_simple().to_string());

    // Generate a field map for the object.
    let map: Vec<String> = connection.hkeys(&index)?;

    // Iterate through map and grab the value corrosponding to the key from the database and store it.
    let object: HashMap<String, O::DataType> = map
        .into_iter()
        .map(|key| {
            let value: O::DataType = connection
                .hget(&index, &key)
                .expect("Could not get value from key for object.");

            (key, value)
        })
        .collect();

    Ok(object)
}

/// Function to request a range of objects from a local Redis database.
/// Returns the objects from the database with the key and object in a Vec.
pub fn request_group_of_objects<O>(
    connection: &mut Connection,
    amount: isize,
) -> RedisResult<O::DataType>
where
    O: Sortable + Clone,
{
    let output: Vec<String> = connection.zrange(
        &format!("{}-index", O::object_to_index()),
        8 * (amount - 1),
        (8 * amount) - 1,
    )?;

    Ok(output
        .into_iter()
        .map(|x| {
            let uuid = Uuid::parse_str(&x).expect("Failed to parse UUID from object.");
            let object: FieldMap<O::DataType> =
                retrieve_object_from_database::<O>(connection, uuid)
                    .expect("Failed to retrieve object from the database.");

            (uuid, object)
        })
        .collect())
}

/// Function to request all the objects from a local Redis database.
/// Returns the objects from the database with the key and object in a Vec.
pub fn request_all_objects<O>(connection: &mut Connection) -> RedisResult<O::DataType>
where
    O: Sortable + Clone,
{
    let output: Vec<String> =
        connection.zrange(&format!("{}-index", O::object_to_index()), 0, -1)?;

    Ok(output
        .into_iter()
        .map(|x| {
            let uuid = Uuid::parse_str(&x).expect("Failed to parse UUID from object.");
            let object: FieldMap<O::DataType> =
                retrieve_object_from_database::<O>(connection, uuid)
                    .expect("Failed to retrieve object from the database.");

            (uuid, object)
        })
        .collect())
}

/// Function to return the current object count in a index the local Redis database.
pub fn current_object_count(
    connection: &mut Connection,
    index: &str,
) -> Result<i32, Box<dyn Error>> {
    Ok(connection.zcard(format!("{}-index", index))?)
}

/// Function to return the first object in the Redis database index.
pub fn grab_first_object(connection: &mut Connection, index: &str) -> Result<Uuid, Box<dyn Error>> {
    let output: Vec<String> = connection.zrange(format!("{}-index", index), 0, 0)?;
    let final_output = match output.first() {
        Some(s) => Uuid::parse_str(s)?,
        None => Uuid::nil(),
    };

    Ok(final_output)
}

/// Function to return the last object in the Redis database index.
pub fn grab_last_object(connection: &mut Connection, index: &str) -> Result<Uuid, Box<dyn Error>> {
    let output: Vec<String> = connection.zrange(format!("{}-index", index), -1, -1)?;
    let final_output = match output.first() {
        Some(s) => Uuid::parse_str(s)?,
        None => Uuid::nil(),
    };

    Ok(final_output)
}
