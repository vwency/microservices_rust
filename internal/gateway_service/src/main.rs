mod errors;
mod handler;
mod http3_serve;
mod server;
mod http2_serve;

pub mod auth {
    tonic::include_proto!("auth_service");
}

use config::{load_config, AppConfig};
use http2_serve::http2_serve::run_http2_server;
use http3_serve::http3_serve::run_http3_server;
use logger::init_logger;
use server::service::GatewayServer;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Debug)]
enum AppError {
    Io(std::io::Error),
    Anyhow(String),
    Config(String),
    Gateway(String),
    Other(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO error: {}", e),
            AppError::Anyhow(e) => write!(f, "Error: {}", e),
            AppError::Config(e) => write!(f, "Config error: {}", e),
            AppError::Gateway(e) => write!(f, "Gateway error: {}", e),
            AppError::Other(e) => write!(f, "Error: {}", e),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Anyhow(err.to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize Rustls crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|e| AppError::Other(format!("Failed to install crypto provider: {:?}", e)))?;

    // Get executable name
    let exe_name = std::env::current_exe()?
        .file_stem()
        .ok_or_else(|| AppError::Anyhow("Failed to get executable name".to_string()))?
        .to_string_lossy()
        .to_string();

    // Load config
    let config = load_config(&exe_name)
        .map_err(|e| AppError::Config(e.to_string()))?;

    // Initialize logger
    init_logger(&config.log_level);

    log::info!(
        "Service {} started",
        config.service_name
    );

    // Define separate addresses for HTTP/2 and HTTP/3
    let http2_addr: SocketAddr = "127.0.0.1:50053"
        .parse()
        .map_err(|e| AppError::Other(format!("Failed to parse HTTP/2 address: {}", e)))?;
    let http3_addr: SocketAddr = "127.0.0.1:50054"
        .parse()
        .map_err(|e| AppError::Other(format!("Failed to parse HTTP/3 address: {}", e)))?;

    log::info!("HTTP/2 server will listen on {}", http2_addr);
    log::info!("HTTP/3 server will listen on {}", http3_addr);

    // Create Gateway server
    let gateway = GatewayServer::new(
        config
            .auth_service_address
            .unwrap_or_else(|| "http://127.0.0.1:50056".to_string()),
    )
        .await
        .map_err(|e| AppError::Gateway(e.to_string()))?;

    let gateway = Arc::new(tokio::sync::Mutex::new(gateway));

    // Start both HTTP/2 and HTTP/3 servers
    let http3_future = run_http3_server(http3_addr, gateway.clone());
    let http2_future = run_http2_server(http2_addr, gateway.clone());

    // Run servers concurrently
    tokio::select! {
        res = http3_future => {
            res.map_err(|e| AppError::Other(e.to_string()))?;
            log::info!("HTTP/3 server stopped");
        }
        res = http2_future => {
            res.map_err(|e| AppError::Other(e.to_string()))?;
            log::info!("HTTP/2 server stopped");
        }
    }

    Ok(())
}
