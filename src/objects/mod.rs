//! A collection of objects for the Rainfusion website.
pub mod rainfusion;

use redis::{FromRedisValue, ToRedisArgs};
use std::collections::HashMap;

/// A generic trait to allow objects to be used easily with
/// database backends in glass.
pub trait Sortable {
    type DataType: FromRedisValue + ToRedisArgs + Clone;

    fn object_to_index() -> &'static str;

    #[cfg(feature = "redis_backend")]
    #[cfg(feature = "json_backend")]
    fn map_to_object(map: HashMap<String, Self::DataType>) -> Self;

    #[cfg(feature = "redis_backend")]
    #[cfg(feature = "json_backend")]
    fn object_to_map(&self) -> Vec<(String, Self::DataType)>;
}
