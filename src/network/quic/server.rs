use bevy::prelude::*;
use futures::StreamExt as _;
use tokio::io::AsyncReadExt as _;

use crate::{config, network::quic::shared};

pub(crate) async fn run(
    config: config::Config,
    sender: tokio::sync::mpsc::UnboundedSender<crate::network::protocol::Event>,
) -> crate::Result<()> {
    let (endpoint, mut incoming) = create_endpoint(&config).await?;

    info!(local_addr = ?endpoint.local_addr()?, "listening");

    while let Some(connection) = incoming.next().await {
        info!("connection incoming");

        let sender = sender.clone();

        tokio::spawn(async move {
            if let Err(error) = handle_connection(connection, sender).await {
                error!(error = error, "connection failed");
            }
        });
    }

    Ok(())
}

async fn create_endpoint(c: &config::Config) -> crate::Result<(quinn::Endpoint, quinn::Incoming)> {
    // load server certificate
    let mut certificate_file = tokio::fs::File::open(&c.quic_server.certificate).await?;
    let mut private_key_file = tokio::fs::File::open(&c.quic_server.private_key).await?;

    let mut certificate_contents = vec![];
    certificate_file
        .read_to_end(&mut certificate_contents)
        .await?;

    let mut private_key_contents = vec![];
    private_key_file
        .read_to_end(&mut private_key_contents)
        .await?;

    let certificate = match rustls_pemfile::read_one(&mut &*certificate_contents)?.unwrap() {
        rustls_pemfile::Item::X509Certificate(e) => rustls::Certificate(e),
        _ => todo!(),
    };
    let private_key = match rustls_pemfile::read_one(&mut &*private_key_contents)?.unwrap() {
        rustls_pemfile::Item::RSAKey(e)
        | rustls_pemfile::Item::PKCS8Key(e)
        | rustls_pemfile::Item::ECKey(e) => rustls::PrivateKey(e),
        _ => todo!(),
    };

    // create config
    let crypto = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![certificate], private_key)?;

    let mut config = quinn::ServerConfig::with_crypto(std::sync::Arc::new(crypto));
    config.use_retry(true);

    // create endpoint
    let addr = format!("{}:{}", c.quic_server.host, c.quic_server.port).parse()?;
    let (endpoint, incoming) = quinn::Endpoint::server(config, addr)?;

    Ok((endpoint, incoming))
}

async fn handle_connection(
    connection: quinn::Connecting,
    sender: tokio::sync::mpsc::UnboundedSender<crate::network::protocol::Event>,
) -> crate::Result<()> {
    shared::handle_connection(connection.await?, sender).await
}
