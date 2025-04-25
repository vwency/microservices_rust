use thiserror::Error;

#[derive(Error, Debug)]
pub enum GatewayError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("gRPC transport error: {0}")]
    TransportError(#[from] tonic::transport::Error),

    #[error("gRPC status error: {0}")]
    StatusError(#[from] tonic::Status),
}

impl From<config::ConfigError> for GatewayError {
    fn from(err: config::ConfigError) -> Self {
        GatewayError::ConfigError(err.to_string())
    }
}

impl From<logger::LoggerError> for GatewayError {
    fn from(err: logger::LoggerError) -> Self {
        GatewayError::ConfigError(err.to_string())
    }
}
