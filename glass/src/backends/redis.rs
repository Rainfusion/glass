//! Redis Functions and Config
//! These functions can be used to allow an object to perform Redis database actions.
//! The configuration can be used to generate a connection to the database.
use std::{error::Error, path::PathBuf, str::FromStr};

use log::debug;
use redis::{Client, Commands, Connection, ConnectionAddr, ConnectionInfo};
use serde::Deserialize;
use std::fmt::Debug;
use uuid::Uuid;

/// Custom Type Definitions
type FieldMap<T> = Vec<(String, T)>;

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
pub fn parse_redis_config<T: AsRef<std::path::Path>>(path: T) -> Result<RedisConfig, Box<Error>> {
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
    pub fn form_connection(self) -> Result<Connection, Box<Error>> {
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
                passwd: self.database_password.clone().and_then(|x| Some(x)),
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
                passwd: self.database_password.clone().and_then(|x| Some(x)),
            })?
            .get_connection()?)
        }
    }
}

/// Function to insert an object into a local Redis database.
/// Returns the UUID of where the object is on the database.
pub fn insert_object_into_database<T>(
    connection: &Connection,
    index: &str,
    field_map: FieldMap<T>,
    uuid: Option<Uuid>,
) -> Result<Uuid, Box<Error>>
where
    T: std::fmt::Debug + redis::ToRedisArgs,
{
    // Generate UUID or use provided one.
    let gen_key = match uuid {
        Some(k) => k,
        None => Uuid::new_v4(),
    };

    // Find next index in table.
    let count: i32 = connection.zcard(format!("{}-index", index))?;

    // Generate a command pipeline.
    let mut pipeline = redis::Pipeline::new();
    // Add UUID to index for object.
    pipeline.add_command(
        redis::cmd("ZADD")
            .arg(&format!("{}-index", index))
            .arg(count + 1)
            .arg(&gen_key.to_simple().to_string()),
    );

    // Iterate through map to find fields that need to be populated and generate a command for them.
    for item in field_map {
        pipeline.add_command(
            redis::cmd("HSET")
                .arg(&format!("{}:{}", index, &gen_key.to_simple().to_string()))
                .arg(item.0)
                .arg(item.1),
        );
    }

    // Finally send commands to database.
    pipeline.query(connection)?;

    Ok(gen_key)
}

/// Function to remove an object from a local Redis database.
pub fn remove_object_from_database(
    connection: &Connection,
    index: &str,
    field_map: &[&'static str],
    uuid: Uuid,
) -> Result<(), Box<Error>> {
    // Remove uuid in table.
    let _res: i32 = connection.zrem(format!("{}-index", index), uuid.to_simple().to_string())?;

    // Generate a command pipeline.
    let mut pipeline = redis::Pipeline::new();

    // Iterate through map to find fields that need to be removed and generate a command for them.
    for item in field_map {
        debug!(
            "DELETE: Got: {} -> for {}. Generating command.",
            item, index
        );
        pipeline.add_command(
            redis::cmd("HDEL")
                .arg(&format!("{}:{}", index, &uuid.to_simple().to_string()))
                .arg(*item),
        );
    }

    // Finally send commands to database.
    pipeline.query(connection)?;

    Ok(())
}

/// Function to edit a field in an object in a local Redis database.
pub fn edit_object_from_database<T>(
    connection: &Connection,
    index: &str,
    field_map: FieldMap<T>,
    uuid: Uuid,
) -> Result<(), Box<Error>>
where
    T: std::fmt::Debug + redis::ToRedisArgs,
{
    // Generate a command pipeline.
    let mut pipeline = redis::Pipeline::new();

    // Iterate through map to find fields that need to be edited and generate a command for them.
    for item in field_map {
        debug!(
            "EDIT: Got: {} - Value: {:?} -> for {}. Generating command.",
            item.0, item.1, index
        );
        pipeline.add_command(
            redis::cmd("HSET")
                .arg(&format!("{}:{}", index, &uuid.to_simple().to_string()))
                .arg(item.0)
                .arg(item.1),
        );
    }

    // Finally send commands to database.
    pipeline.query(connection)?;

    Ok(())
}

/// Function to retrieve a object in a local Redis database.
pub fn retrieve_object_from_database<T>(
    connection: &Connection,
    index: &str,
    field_map: &[&'static str],
    uuid: Uuid,
) -> Result<FieldMap<T>, Box<Error>>
where
    T: redis::FromRedisValue + std::fmt::Display + Clone + Debug,
{
    // Generate a command pipeline.
    let mut pipeline = redis::Pipeline::new();

    // Iterate through map to find fields that need to be retrieved and generate a command for them.
    for item in field_map.iter() {
        debug!("GET: Got: {} -> for {}. Generating command.", item, index);
        pipeline.add_command(
            redis::cmd("HGET")
                .arg(&format!("{}:{}", index, &uuid.to_simple().to_string()))
                .arg(*item),
        );
    }

    // Finally send commands to database.
    let output: Vec<T> = pipeline.query(connection)?;

    // Merge the two vectors.
    let mut merged: FieldMap<T> = Vec::new();

    // Merge Field with Value
    for i in 0..field_map.len() {
        debug!("GET COMBINE: Index: {}", i);
        let map_value = field_map.get(i).map_or("", |x| x);
        let value = output.get(i);

        debug!(
            "GET COMBINE: Map Value: {} - Output Value: {:?}",
            map_value, value
        );

        if !value.is_none() {
            merged.push((map_value.to_string(), value.unwrap().to_owned()));
        }
    }

    Ok(merged)
}

/// Function to request a range of objects from a local Redis database.
/// Returns the objects from the database with the key and object in a Vec.
pub fn request_group_of_objects<T>(
    connection: &Connection,
    field_map: &[&'static str],
    index: &str,
    amount: isize,
) -> Result<Vec<(Uuid, FieldMap<T>)>, Box<Error>>
where
    T: redis::FromRedisValue + std::fmt::Display + Clone + Debug,
{
    let mut vector: Vec<(Uuid, FieldMap<T>)> = Vec::new();

    let output: Vec<String> = connection.zrange(
        &format!("{}-index", index),
        8 * (amount - 1),
        (8 * amount) - 1,
    )?;

    for value in output {
        let uuid = Uuid::parse_str(&value)?;

        let object = retrieve_object_from_database(connection, index, field_map, uuid)?;
        vector.push((uuid, object));

        if uuid == grab_last_object(connection, index)? {
            vector.push((Uuid::nil(), vec![]));
        }
    }

    Ok(vector)
}

/// Function to return the current object count in a index the local Redis database.
pub fn current_object_count(connection: &Connection, index: &str) -> Result<i32, Box<Error>> {
    Ok(connection.zcard(format!("{}-index", index))?)
}

/// Function to return the score of the object in the Redis database index.
pub fn object_score(connection: &Connection, index: &str, uuid: Uuid) -> Result<i32, Box<Error>> {
    Ok(connection.zscore(format!("{}-index", index), uuid.to_simple().to_string())?)
}

/// Function to increase or decrease score of an object in the Redis database index.
pub fn change_object_score(connection: &Connection, index: &str, increment: i32, uuid: Uuid) -> Result<(), Box<Error>> {
    Ok(connection.zincr(format!("{}-index", index), increment, uuid.to_simple().to_string())?)
}

/// Function to return the first object in the Redis database index.
pub fn grab_first_object(connection: &Connection, index: &str) -> Result<Uuid, Box<Error>> {
    let output: Vec<String> = connection.zrange(format!("{}-index", index), 0, 0)?;
    let final_output = match output.first() {
        Some(s) => Uuid::parse_str(s)?,
        None => Uuid::nil(),
    };

    Ok(final_output)
}

/// Function to return the last object in the Redis database index.
pub fn grab_last_object(connection: &Connection, index: &str) -> Result<Uuid, Box<Error>> {
    let output: Vec<String> = connection.zrange(format!("{}-index", index), -1, -1)?;
    let final_output = match output.first() {
        Some(s) => Uuid::parse_str(s)?,
        None => Uuid::nil(),
    };

    Ok(final_output)
}
