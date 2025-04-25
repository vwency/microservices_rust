use crate::server::GatewayServer;
use quinn::{Endpoint, ServerConfig};
use rustls::{Certificate, PrivateKey, ServerConfig as RustlsConfig};
use std::{net::SocketAddr, sync::Arc};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn run_http3_server(
    addr: SocketAddr,
    gateway: GatewayServer,
) -> Result<(), Box<dyn std::error::Error>> {
    // Генерация самоподписанного сертификата (для разработки)
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
    let cert_chain = vec![Certificate(cert.serialize_der()?)];
    let priv_key = PrivateKey(cert.serialize_private_key_der());

    // Настройка TLS
    let mut tls_config = RustlsConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, priv_key)?;
    tls_config.alpn_protocols = vec![b"h3".to_vec()];

    // Конфигурация Quinn
    let mut server_config = ServerConfig::with_crypto(Arc::new(tls_config));
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config
        .max_concurrent_uni_streams(0)
        .unwrap();

    // Запуск сервера
    let (endpoint, mut incoming) = Endpoint::server(server_config, addr)?;
    log::info!("HTTP/3 server listening on {}", endpoint.local_addr()?);

    while let Some(conn) = incoming.next().await {
        let conn = conn.await?;
        let gateway = gateway.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_http3_connection(conn, gateway).await {
                log::error!("HTTP/3 connection error: {}", e);
            }
        });
    }

    Ok(())
}

async fn handle_http3_connection(
    conn: quinn::Connection,
    gateway: GatewayServer,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let (mut send_stream, mut recv_stream) = conn.accept_bi().await?;
        
        let mut request_data = Vec::new();
        recv_stream.read_to_end(&mut request_data).await?;
        
        let response_data = gateway.handle_http3_request(&request_data).await?;
        
        send_stream.write_all(&response_data).await?;
        send_stream.finish().await?;
    }
}
