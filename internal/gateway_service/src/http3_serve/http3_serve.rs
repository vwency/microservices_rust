use crate::server::service::GatewayServer;
use quinn::{Endpoint, ServerConfig};
use rustls::{Certificate, PrivateKey, ServerConfig as RustlsConfig};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn run_http3_server(
    addr: SocketAddr,
    gateway: Arc<tokio::sync::Mutex<GatewayServer>>, // <-- Updated to use Mutex<GatewayServer>
) -> Result<(), Box<dyn std::error::Error>> {
    // Generate a self-signed certificate (for development)
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
    let cert_chain = vec![Certificate(cert.serialize_der()?)];
    let priv_key = PrivateKey(cert.serialize_private_key_der());

    // Configure TLS
    let mut tls_config = RustlsConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, priv_key)?;
    tls_config.alpn_protocols = vec![b"h3".to_vec()];

    // Configure Quinn
    let mut server_config = ServerConfig::with_crypto(Arc::new(tls_config));
    let mut transport_config = quinn::TransportConfig::default();
    transport_config.max_concurrent_uni_streams(0_u8.into());
    server_config.transport = Arc::new(transport_config);

    // Start the server
    let endpoint = Endpoint::server(server_config, addr)?;

    log::info!("HTTP/3 server listening on {}", endpoint.local_addr()?);

    loop {
        if let Some(connecting) = endpoint.accept().await {
            let conn = connecting.await?;
            let gateway = Arc::clone(&gateway); // Safely clone Arc

            tokio::spawn(async move {
                if let Err(e) = handle_http3_connection(conn, gateway).await {
                    log::error!("HTTP/3 connection error: {}", e);
                }
            });
        }
    }
}

async fn handle_http3_connection(
    conn: quinn::Connection,
    gateway: Arc<tokio::sync::Mutex<GatewayServer>>, // <-- Using Mutex for GatewayServer
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let (mut send_stream, mut recv_stream) = conn.accept_bi().await?;

        let max_size = 64 * 1024;
        let request_data = recv_stream.read_to_end(max_size).await?;

        // You can process the request_data via the gateway
        let response_data = {
            // Simulate parsing the request and getting a response from the gateway
            let gateway = gateway.lock().await; // Lock the gateway for thread-safe access

            // Example response
            // This could be a real call like `gateway.handle_request(request_data).await`
            let response = "HTTP/3 response";
            response.as_bytes().to_vec() // Return the response as a byte vector
        };

        send_stream.write_all(&response_data).await?;
        send_stream.finish().await?;
    }
}
