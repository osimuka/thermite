use thiserror::Error;
use redis::RedisError;
use serde_json::Error as SerdeError;

#[derive(Error, Debug)]
pub enum TaskQueueError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),

    #[error("Serialization/Deserialization error: {0}")]
    Serde(#[from] SerdeError),
}
