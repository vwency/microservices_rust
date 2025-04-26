use quinn::ConfigError;
use thiserror::Error;
use tonic::{transport, Status};

#[derive(Error, Debug)]
pub enum GatewayError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("gRPC transport error: {0}")]
    TransportError(#[from] transport::Error),

    #[error("gRPC status error: {0}")]
    StatusError(#[from] Status),
}

impl From<ConfigError> for GatewayError {
    fn from(err: ConfigError) -> Self {
        GatewayError::ConfigError(err.to_string())
    }
}
