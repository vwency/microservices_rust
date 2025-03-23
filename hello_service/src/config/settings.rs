use config::Config;
use std::net::SocketAddr;
use std::error::Error;

#[derive(Debug)]
pub struct AppConfig {
    pub address: SocketAddr,
    pub service_name: String,
    pub log_level: String,
}

pub fn load_config() -> Result<AppConfig, Box<dyn Error>> {
    let settings = Config::builder()
        .add_source(config::File::with_name("configs/hello_service/config"))
        .build()?;

    let address = settings.get::<String>("server.address")?.parse()?;
    let service_name = settings.get::<String>("server.name")?;
    let log_level = settings.get::<String>("server.log_level")?;

    Ok(AppConfig { address, service_name, log_level })
}
