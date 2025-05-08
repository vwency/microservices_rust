use bytes::Bytes;
use futures::future::BoxFuture;
use hyper::{
    body::Incoming as Body,
    server::conn::http2,
    service::Service,
    Method, Request, Response, StatusCode,
};
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::error;
use hyper_util::rt::{TokioExecutor, TokioIo};

use crate::server::service::GatewayServer;
use crate::auth::{LoginRequest, LoginResponse, RefreshRequest, RefreshResponse};

// Struct definitions
#[derive(Debug, Serialize, Deserialize)]
pub struct HttpLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpRefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpLoginResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpRefreshResponse {
    pub access_token: String,
}

// Utility functions
fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

fn json_response<T: Serialize>(
    value: &T,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Box<dyn Error + Send + Sync>> {
    let json = serde_json::to_vec(value)?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(full(json))?)
}

fn error_response(
    status: StatusCode,
    message: impl Into<String>,
) -> Response<BoxBody<Bytes, hyper::Error>> {
    Response::builder()
        .status(status)
        .header("content-type", "text/plain")
        .body(full(message.into()))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(full("Internal Server Error"))
                .unwrap()
        })
}

// GatewayHttpService implementation
#[derive(Clone)]
struct GatewayHttpService {
    gateway: Arc<Mutex<GatewayServer>>,
}

impl GatewayHttpService {
    async fn handle_login(
        &self,
        req: HttpLoginRequest,
    ) -> Result<HttpLoginResponse, Box<dyn Error + Send + Sync>> {
        let mut gateway = self.gateway.lock().await;
        let grpc_req = LoginRequest {
            username: req.username,
            password: req.password,
        };
        // Ensure the login result is compatible with Box<dyn Error + Send + Sync>
        let grpc_res = gateway.login(grpc_req).await?;
        Ok(HttpLoginResponse {
            access_token: grpc_res.access_token,
            refresh_token: grpc_res.refresh_token,
        })
    }

    async fn handle_refresh(
        &self,
        req: HttpRefreshRequest,
    ) -> Result<HttpRefreshResponse, Box<dyn Error + Send + Sync>> {
        let mut gateway = self.gateway.lock().await;
        let grpc_req = RefreshRequest {
            refresh_token: req.refresh_token,
        };
        let grpc_res = gateway.refresh(grpc_req).await?;
        Ok(HttpRefreshResponse {
            access_token: grpc_res.access_token,
        })
    }
}

impl Service<Request<Body>> for GatewayHttpService {
    type Response = Response<BoxBody<Bytes, hyper::Error>>;
    type Error = hyper::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&self, req: Request<Body>) -> Self::Future {
        let service = self.clone();

        Box::pin(async move {
            let (parts, body) = req.into_parts();
            let body_bytes = match body.collect().await {
                Ok(collected) => collected.to_bytes(),
                Err(e) => return Ok(error_response(StatusCode::BAD_REQUEST, e.to_string())),
            };

            let response = match (parts.method, parts.uri.path()) {
                (Method::POST, "/login") => {
                    match serde_json::from_slice::<HttpLoginRequest>(&body_bytes) {
                        Ok(parsed_req) => match service.handle_login(parsed_req).await {
                            Ok(res) => json_response(&res)
                                .unwrap_or_else(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
                            Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
                        },
                        Err(e) => error_response(StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)),
                    }
                }
                (Method::POST, "/refresh") => {
                    match serde_json::from_slice::<HttpRefreshRequest>(&body_bytes) {
                        Ok(parsed_req) => match service.handle_refresh(parsed_req).await {
                            Ok(res) => json_response(&res)
                                .unwrap_or_else(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
                            Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
                        },
                        Err(e) => error_response(StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)),
                    }
                }
                _ => error_response(StatusCode::NOT_FOUND, "Not Found"),
            };

            Ok(response)
        })
    }
}

// HTTP/2 server setup
pub async fn run_http2_server(
    addr: SocketAddr,
    gateway: Arc<Mutex<GatewayServer>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("HTTP/2 server listening on {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let service = GatewayHttpService {
            gateway: gateway.clone(),
        };

        tokio::spawn(async move {
            let conn = http2::Builder::new(TokioExecutor::new())
                .serve_connection(io, service);

            if let Err(e) = conn.await {
                error!("HTTP/2 connection error: {}", e);
            }
        });
    }
}
