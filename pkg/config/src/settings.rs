use crate::config::{AppConfig, RawConfig};
use std::error::Error;

pub fn load_config(service_name: &str) -> Result<AppConfig, Box<dyn Error>> {
    let config_path = format!("configs/{}/config", service_name);

    let settings = ::config::Config::builder()
        .add_source(::config::File::with_name(&config_path).required(true))
        .build()?;

    let raw_config: RawConfig = settings.try_deserialize()?;

    Ok(AppConfig {
        address: raw_config.server.address,
        service_name: raw_config.server.name,
        log_level: raw_config.server.log_level,
        auth_service_address: raw_config.auth_service.map(|a| a.address),
        tls_cert_path: raw_config.tls.as_ref().and_then(|t| t.cert_path.clone()),
        tls_key_path: raw_config.tls.as_ref().and_then(|t| t.key_path.clone()),
    })
}
