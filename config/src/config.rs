use std::net::SocketAddr;

#[derive(Debug)]
pub struct AppConfig {
    pub address: SocketAddr,
    pub service_name: String,
    pub log_level: String,
}
