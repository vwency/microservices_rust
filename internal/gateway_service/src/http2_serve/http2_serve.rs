use bytes::Bytes;
use futures::future::BoxFuture;
use hyper::{
    body::Incoming as Body,
    server::conn::http2,
    service::Service,
    Method, Request, Response, StatusCode,
};
use crate::server::service::{GatewayServer, LoginRequest, LoginResponse, RefreshRequest, RefreshResponse};
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Seek};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use tracing::{info, error};
use hyper_util::rt::{TokioExecutor, TokioIo};

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
        .map_err(|never| match never {} )
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

fn load_certs(path: &str) -> Result<Vec<CertificateDer<'static>>, Box<dyn Error + Send + Sync>> {
    let certfile = File::open(path)?;
    let mut reader = BufReader::new(certfile);
    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(CertificateDer::from)
        .collect();
    Ok(certs)
}

fn load_key(path: &str) -> Result<PrivateKeyDer<'static>, Box<dyn Error + Send + Sync>> {
    let keyfile = File::open(path)?;
    let mut reader = BufReader::new(keyfile);

    // Try PKCS8 first
    let keys = rustls_pemfile::pkcs8_private_keys(&mut reader).collect::<Result<Vec<_>, _>>()?;
    if let Some(key) = keys.into_iter().next() {
        return Ok(PrivateKeyDer::from(key));
    }

    // If no PKCS8 key found, try RSA
    reader.rewind()?;
    let keys = rustls_pemfile::rsa_private_keys(&mut reader).collect::<Result<Vec<_>, _>>()?;
    if let Some(key) = keys.into_iter().next() {
        return Ok(PrivateKeyDer::from(key));
    }

    Err("No private keys found.".into())
}

pub async fn run_http2_server(
    addr: SocketAddr,
    gateway: Arc<Mutex<GatewayServer>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let certs = load_certs("certs/server.crt")?;
    let key = load_key("certs/server.key")?;

    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    config.alpn_protocols = vec![b"h2".to_vec()];

    let tls_acceptor = TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind(addr).await?;
    info!("HTTPS/2 server with TLS listening on {}", addr);

    loop {
        let (tcp_stream, _) = listener.accept().await?;
        let acceptor = tls_acceptor.clone();
        let service = GatewayHttpService {
            gateway: gateway.clone(),
        };

        tokio::spawn(async move {
            match acceptor.accept(tcp_stream).await {
                Ok(tls_stream) => {
                    info!("Accepted new HTTPS/2 connection");
                    let io = TokioIo::new(tls_stream);

                    let conn = http2::Builder::new(TokioExecutor::new())
                        .serve_connection(io, service);

                    if let Err(e) = conn.await {
                        error!("HTTP/2 connection error: {}", e);
                    }
                }
                Err(e) => {
                    error!("TLS handshake failed: {}", e);
                }
            }
        });
    }
}
