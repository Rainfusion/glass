//! Glass is a library required when working on backend work for the Rainfusion website.
//! It provides the objects and backends required to cache and store data throughout many formats.
//! While Redis is the main database backend this library can be modified to contain any other backend.
//! A user can choose to disable certain backends by using features available in this library.
#![allow(unused_imports)]
#![allow(clippy::redundant_closure)]
pub mod backends;
pub mod objects;
