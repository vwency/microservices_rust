use crate::server::service::GatewayServer;
use hyper::{Body, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tonic::body::BoxBody;
use tower::Service;

// Убедитесь, что эти структуры имеют derive
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshResponse {
    pub token: String,
}

pub async fn run_http2_server(
    addr: SocketAddr,
    gateway: Arc<tokio::sync::Mutex<GatewayServer>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(addr).await?;
    log::info!("HTTP/2 server listening on {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let gateway = Arc::clone(&gateway);

        tokio::spawn(async move {
            let service = GatewayHttpService { gateway };
            let conn = hyper::server::conn::Http::new()
                .serve_connection(stream, service);

            if let Err(e) = conn.await {
                log::error!("HTTP/2 connection error: {}", e);
            }
        });
    }
}

#[derive(Clone)]
struct GatewayHttpService {
    gateway: Arc<tokio::sync::Mutex<GatewayServer>>,
}

impl Service<Request<Body>> for GatewayHttpService {
    type Response = Response<BoxBody>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let gateway = Arc::clone(&self.gateway);

        Box::pin(async move {
            let (parts, body) = req.into_parts();
            let body_bytes = hyper::body::to_bytes(body).await?;

            let response = match (parts.method, parts.uri.path()) {
                (hyper::Method::POST, "/login") => {
                    let request: LoginRequest = serde_json::from_slice(&body_bytes)?;
                    let mut gateway = gateway.lock().await;
                    let response = gateway.login(request).await?;
                    Response::builder()
                        .status(StatusCode::OK)
                        .body(tonic::body::boxed(tonic::body::Full::from(serde_json::to_vec(&response)?)))
                }
                (hyper::Method::POST, "/refresh") => {
                    let request: RefreshRequest = serde_json::from_slice(&body_bytes)?;
                    let mut gateway = gateway.lock().await;
                    let response = gateway.refresh(request).await?;
                    Response::builder()
                        .status(StatusCode::OK)
                        .body(tonic::body::boxed(tonic::body::Full::from(serde_json::to_vec(&response)?)))
                }
                _ => Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(tonic::body::boxed(tonic::body::Full::from("Not Found"))),
            }?;

            Ok(response)
        })
    }
}