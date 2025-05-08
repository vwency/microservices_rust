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
use tokio_rustls::TlsAcceptor;
use crate::server::service::GatewayServer;
use crate::auth::{LoginRequest, LoginResponse, RefreshRequest, RefreshResponse};
use rustls::ServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::fs::File;
use std::io::{BufReader, BufRead};
use rustls_pemfile::{certs, pkcs8_private_keys};


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



fn load_tls_config() -> Result<ServerConfig, Box<dyn Error + Send + Sync>> {
    let cert_file = File::open("cert.pem")
        .map_err(|e| format!("Failed to open cert.pem: {}", e))?;
    let key_file = File::open("key.pem")
        .map_err(|e| format!("Failed to open key.pem: {}", e))?;

    let mut cert_reader = BufReader::new(cert_file);
    let mut key_reader = BufReader::new(key_file);

    let certs = certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Failed to parse certificate".to_string())?;

    let mut keys = pkcs8_private_keys(&mut key_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Failed to parse private key".to_string())?;

    if keys.is_empty() {
        return Err("No private keys found".into());
    }

    let key = PrivateKeyDer::Pkcs8(keys.remove(0));

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| format!("Failed to create TLS config: {}", e))?;

    Ok(config)
}


pub async fn run_http2_server(
    addr: SocketAddr,
    gateway: Arc<Mutex<GatewayServer>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let listener = TcpListener::bind(addr).await?;
    let tls_config = tokio::task::spawn_blocking(load_tls_config).await??;
    let acceptor = TlsAcceptor::from(Arc::new(tls_config));

    tracing::info!("HTTP/2 server with TLS listening on {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let service = GatewayHttpService {
            gateway: gateway.clone(),
        };

        tokio::spawn(async move {
            let stream = match acceptor.accept(stream).await {
                Ok(stream) => stream,
                Err(e) => {
                    error!("TLS handshake error: {}", e);
                    return;
                }
            };

            let io = TokioIo::new(stream);
            let conn = http2::Builder::new(TokioExecutor::new())
                .serve_connection(io, service);

            if let Err(e) = conn.await {
                error!("HTTP/2 connection error: {}", e);
            }
        });
    }
}
