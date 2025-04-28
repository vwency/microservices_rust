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
use std::{error::Error, net::SocketAddr, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let exe_name = std::env::current_exe()?
        .file_stem()
        .ok_or("Failed to get executable name")?
        .to_string_lossy()
        .to_string();

    let config = load_config(&exe_name)?;

    init_logger(&config.log_level);

    log::info!(
        "Сервис {} запущен на {}",
        config.service_name,
        config.address
    );

    let addr: SocketAddr = config.address;

    let gateway = GatewayServer::new("http://127.0.0.1:50056".to_string()).await?;
    let gateway = Arc::new(tokio::sync::Mutex::new(gateway));

    if let Err(e) = run_http3_server(addr, gateway.clone()).await {
        log::error!("Ошибка HTTP/3 сервера: {}", e);
        return Err(e);
    }

    Ok(())
}
