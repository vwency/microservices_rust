use crate::server::service::GatewayServer;
use quinn::{Connection, Endpoint, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::Result;

pub async fn run_http3_server(
    addr: SocketAddr,
    gateway: Arc<tokio::sync::Mutex<GatewayServer>>,
) -> Result<()> {
    // Generate a self-signed certificate (for development)
    let cert_key = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
    let cert_chain = vec![CertificateDer::from(cert_key.cert.der().to_vec())];
    let priv_key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(cert_key.key_pair.serialized_der().to_vec()));

    // Configure TLS with rustls 0.23.27 for QUIC
    let mut crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, priv_key)?;

    // Configure ALPN for HTTP/3
    crypto.alpn_protocols = vec![b"h3".to_vec()];

    // Create QUIC-compatible server config
    let crypto = quinn::crypto::rustls::QuicServerConfig::try_from(crypto)
        .map_err(|e| anyhow::anyhow!("Failed to create QUIC server config: {}", e))?;

    // Configure Quinn
    let mut server_config = ServerConfig::with_crypto(Arc::new(crypto));
    let mut transport_config = quinn::TransportConfig::default();
    transport_config.max_concurrent_uni_streams(0_u8.into());
    server_config.transport = Arc::new(transport_config);

    // Start the server
    let endpoint = Endpoint::server(server_config, addr)?;

    log::info!("HTTP/3 server listening on {}", endpoint.local_addr()?);

    loop {
        if let Some(connecting) = endpoint.accept().await {
            let conn = connecting.await?;
            let gateway = Arc::clone(&gateway);

            tokio::spawn(async move {
                if let Err(e) = handle_http3_connection(conn, gateway).await {
                    log::error!("HTTP/3 connection error: {}", e);
                }
            });
        }
    }
}

async fn handle_http3_connection(
    conn: Connection,
    gateway: Arc<tokio::sync::Mutex<GatewayServer>>,
) -> Result<()> {
    loop {
        let (mut send_stream, mut recv_stream) = conn.accept_bi().await?;

        // Read request data with a reasonable size limit
        let mut request_data = Vec::new();
        let max_size = 1024 * 1024; // 1MB limit
        let bytes_read = recv_stream.read_buf(&mut request_data).await?;

        // Process the request via the gateway
        let response_data = {
            let gateway = gateway.lock().await;
            // Example response - replace with actual gateway processing
            let response = "HTTP/3 response";
            response.as_bytes().to_vec()
        };

        send_stream.write_all(&response_data).await?;

        // Finish the stream properly without awaiting a result
        send_stream.finish()?;
    }
}
