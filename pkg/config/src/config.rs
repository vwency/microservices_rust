use serde::{Deserialize, Deserializer};
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct RawServerConfig {
    pub name: String,
    pub log_level: String,

    #[serde(deserialize_with = "deserialize_socket_addr")]
    pub address: SocketAddr, // Это теперь будет десериализоваться из строки
}

#[derive(Debug, Deserialize)]
pub struct RawAuthServiceConfig {
    pub address: String, // Просто строка (gRPC клиент требует String)
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
    pub auth_service: Option<RawAuthServiceConfig>, // Добавить это поле
}

#[derive(Debug)]
pub struct AppConfig {
    pub address: SocketAddr,
    pub service_name: String,
    pub log_level: String,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
    pub auth_service_address: Option<String>,  // Добавьте это поле
}



// Кастомный десериализатор для преобразования строки в SocketAddr
fn deserialize_socket_addr<'de, D>(deserializer: D) -> Result<SocketAddr, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}
