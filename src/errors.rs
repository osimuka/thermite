use thiserror::Error;
use redis::RedisError;
use serde_json::Error as SerdeError;

#[derive(Error, Debug)]
pub enum TaskQueueError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),

    #[error("Serialization/Deserialization error: {0}")]
    Serde(#[from] SerdeError),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Redis error: {0}")]
    RedisError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid cron expression: {0}")]
    InvalidCronExpression(String),

    #[error("Application state error: {0}")]
    StateError(String),
}
