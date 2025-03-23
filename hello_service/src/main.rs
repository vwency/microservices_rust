mod handlers;
mod config;
mod server;

use config::settings::load_config;
use server::service::run_server;
use std::error::Error;

pub mod hello {
    tonic::include_proto!("hello");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = load_config()?;
    run_server(config).await
}
