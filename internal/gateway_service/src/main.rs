mod errors;
mod handler;
mod http3_serve;
mod server;

pub mod auth {
    tonic::include_proto!("auth_service");
}

use config::{load_config, AppConfig};
use http3_serve::http3_serve::run_http3_server;
use logger::init_logger;
use server::service::GatewayServer;
use std::{error::Error, net::SocketAddr};

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

    let gateway = GatewayServer::default();

    let http3_addr: SocketAddr = config
        .http3_address
        .as_ref()
        .ok_or("HTTP/3 address not configured")?
        .parse()
        .map_err(|e| format!("Invalid HTTP/3 address format: {}", e))?;

    run_http3_server(http3_addr, gateway).await.map_err(|e| {
        log::error!("HTTP/3 server error: {}", e);
        e
    })?;

    Ok(())
}
