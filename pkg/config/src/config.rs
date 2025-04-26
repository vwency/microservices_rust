use serde::Deserialize;
use std::net::SocketAddr;

// Структуры для десериализации конфига из файла
#[derive(Debug, Deserialize)]
pub struct RawServerConfig {
    pub name: String,
    pub log_level: String,
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct RawTlsConfig {
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawConfig {
    pub server: RawServerConfig,
    pub tls: Option<RawTlsConfig>,
}

// Финализированная структура конфига для использования в приложении
#[derive(Debug)]
pub struct AppConfig {
    pub address: SocketAddr,
    pub service_name: String,
    pub log_level: String,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
}
