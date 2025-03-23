mod handlers;

use handlers::hello::MyHelloService;
use hello::hello_service_server::HelloServiceServer; 
use tonic::transport::Server;
use config::Config;
use std::error::Error;

pub mod hello {
    tonic::include_proto!("hello"); 
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let settings = Config::builder()
        .add_source(config::File::with_name("configs/hello_service/config"))
        .build()?;

    let addr: std::net::SocketAddr = settings.get::<String>("server.address")?.parse()?;
    let service_name = settings.get::<String>("server.name")?;
    
    let hello_service = MyHelloService::default();

    println!("{} running on {}", service_name, addr);

    Server::builder()
        .add_service(HelloServiceServer::new(hello_service))
        .serve(addr)
        .await?;

    Ok(())
}
