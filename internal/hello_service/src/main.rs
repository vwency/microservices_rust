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
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <service_name>", args[0]);
        std::process::exit(1);
    }
    let service_name = &args[1];

    let config = load_config(service_name)?;

    init_logger(&config.log_level);

    log::info!(
        "Сервис {} запущен на {}",
        config.service_name,
        config.address
    );

    run_server(config).await
}
