mod handlers;
mod server;

use config::load_config;
use logger::init_logger;
use server::service::run_server;
use std::error::Error;

pub mod hello {
    tonic::include_proto!("hello");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Просто указываем путь к конфигу
    let config = load_config("hello_service")?; // <-- здесь путь к папке конфига: configs/hello_service/config.toml

    init_logger(&config.log_level);

    log::info!(
        "Сервис {} запущен на {}",
        config.service_name,
        config.address
    );

    run_server(config).await
}
