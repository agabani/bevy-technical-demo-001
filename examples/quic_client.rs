use tokio::io::AsyncReadExt;

fn main() -> bevy_technical_demo::Result<()> {
    let config = bevy_technical_demo::config::load(&[])?;

    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(run(config))?;

    Ok(())
}

async fn run(config: bevy_technical_demo::config::Config) -> bevy_technical_demo::Result<()> {
    let endpoint = create_endpoint(&config).await?;

    let addr = format!("{}:{}", config.quic_server.host, config.quic_server.port).parse()?;
    let connection = endpoint.connect(addr, &config.quic_server.name)?.await?;

    loop {
        let (mut send, recv) = connection.connection.open_bi().await?;

        let request = bevy_technical_demo::protocol::Payload::V1(
            bevy_technical_demo::protocol::Version1::Ping,
        );
        let request = request.serialize()?;

        send.write_all(&request).await?;
        send.finish().await?;

        let response = recv.read_to_end(64 * 1024).await?;

        let response = bevy_technical_demo::protocol::Payload::deserialize(&response)?;

        println!("response: {:?}", response);
    }

    // connection.connection.close(0u32.into(), b"done");

    // Ok(())
}

async fn create_endpoint(
    c: &bevy_technical_demo::config::Config,
) -> bevy_technical_demo::Result<quinn::Endpoint> {
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
