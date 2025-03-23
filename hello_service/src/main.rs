mod handlers;
mod server;

use config::{load_config, AppConfig};
use logger::init_logger;
use server::service::run_server;
use std::error::Error;

pub mod hello {
    tonic::include_proto!("hello");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = load_config()?;

    init_logger(&config.log_level);  

    log::info!("Сервис {} запущен на {}", config.service_name, config.address);

    run_server(config).await
}
