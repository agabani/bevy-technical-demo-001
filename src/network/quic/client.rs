use bevy::prelude::*;
use tokio::io::AsyncReadExt as _;

use crate::{config, network::quic::shared};

pub(crate) async fn run(
    config: config::Config,
    sender: tokio::sync::mpsc::UnboundedSender<crate::network::protocol::Event>,
) -> crate::Result<()> {
    let endpoint = create_endpoint(&config).await?;

    info!(local_addr = ?endpoint.local_addr()?, "listening");

    let addr = format!("{}:{}", config.quic_server.host, config.quic_server.port).parse()?;

    loop {
        info!("connecting");

        match endpoint.connect(addr, &config.quic_server.name)?.await {
            Ok(connection) => {
                if let Err(error) = shared::handle_connection(connection, sender.clone()).await {
                    error!(error = error, "connection failed");
                }
            }
            Err(error) => {
                error!(error = ?error, "connection failed");
            }
        }
    }
}

async fn create_endpoint(c: &config::Config) -> crate::Result<quinn::Endpoint> {
    // load server certificate
    let mut certificate_file = tokio::fs::File::open(&c.quic_client.certificate).await?;

    let mut certificate_contents = vec![];
    certificate_file
        .read_to_end(&mut certificate_contents)
        .await?;

    let certificate = match rustls_pemfile::read_one(&mut &*certificate_contents)?.unwrap() {
        rustls_pemfile::Item::X509Certificate(e) => rustls::Certificate(e),
        _ => todo!(),
    };

    // create root certificate store
    let mut roots = rustls::RootCertStore::empty();
    roots.add(&certificate)?;

    // create config
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();

    let config = quinn::ClientConfig::new(std::sync::Arc::new(crypto));

    // create endpoint
    let addr = format!("{}:{}", c.quic_client.host, c.quic_client.port).parse()?;
    let mut endpoint = quinn::Endpoint::client(addr)?;
    endpoint.set_default_client_config(config);

    Ok(endpoint)
}
