use auth_service::auth_service_server::AuthServiceServer;
use config::Config;
use http3_server::run_http3_server;
use logger::Logger;
use server::GatewayServer;
use std::net::SocketAddr;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{Any, CorsLayer};

mod http3_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Инициализация логгера и конфига
    Logger::init("gateway_service")?;
    let config = Config::load("gateway_service")?;
    
    // Создаем экземпляр нашего сервера
    let gateway = GatewayServer::default();

    // Запускаем gRPC сервер (HTTP/2)
    let grpc_addr: SocketAddr = config.server.grpc_address.parse()?;
    let grpc_future = {
        let gateway = gateway.clone();
        async move {
            let service = AuthServiceServer::new(gateway)
                .accept_gzip()
                .send_gzip();

            Server::builder()
                .accept_http1(true)
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                )
                .layer(GrpcWebLayer::new())
                .add_service(service)
                .serve(grpc_addr)
                .await
        }
    };

    // Запускаем HTTP/3 сервер
    let http3_addr: SocketAddr = config.server.http3_address.parse()?;
    let http3_future = run_http3_server(http3_addr, gateway);

    // Ожидаем завершения любого из серверов
    tokio::select! {
        res = grpc_future => {
            res.map_err(|e| {
                log::error!("gRPC server error: {}", e);
                e
            })?
        },
        res = http3_future => {
            res.map_err(|e| {
                log::error!("HTTP/3 server error: {}", e);
                e
            })?
        }
    }

    Ok(())
}