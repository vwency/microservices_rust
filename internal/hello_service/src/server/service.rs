use crate::handlers::hello::MyHelloService;
use crate::hello::hello_service_server::HelloServiceServer;
use config::config::AppConfig;
use tonic::transport::Server;

use std::error::Error;

pub async fn run_server(config: AppConfig) -> Result<(), Box<dyn Error>> {
    let hello_service = MyHelloService::default();

    println!("{} running on {}", config.service_name, config.address);

    Server::builder()
        .add_service(HelloServiceServer::new(hello_service))
        .serve(config.address)
        .await?;

    Ok(())
}
