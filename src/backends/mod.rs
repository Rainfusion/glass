//! The backends for the objects.
//! Each backend can be disabled / enabled depending on the users configuration.
#[cfg(feature = "json_backend")]
pub mod json;
#[cfg(feature = "redis_backend")]
pub mod redis;
#[cfg(feature = "yaml_backend")]
pub mod yaml;
