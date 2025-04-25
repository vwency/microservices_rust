use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct AppConfig {
    pub address: SocketAddr,
    pub service_name: String,
    pub log_level: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub log_level: String,
    pub grpc_address: String,
    pub http3_address: String,
}

#[derive(Debug, Deserialize)]
pub struct TlsConfig {
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub tls: Option<TlsConfig>,
}
