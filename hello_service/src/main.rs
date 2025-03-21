mod handlers;

use handlers::hello::MyHelloService;
use hello::hello_service_server::HelloServiceServer; 
use tonic::transport::Server;

pub mod hello {
    tonic::include_proto!("hello"); 
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let hello_service = MyHelloService::default();

    println!("HelloService running on {}", addr);

    Server::builder()
        .add_service(HelloServiceServer::new(hello_service))
        .serve(addr)
        .await?;

    Ok(())
}
