use futures::StreamExt;
use tokio::io::AsyncReadExt;

fn main() -> bevy_technical_demo::Result<()> {
    let config = bevy_technical_demo::config::load(&[])?;

    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(run(config))?;

    Ok(())
}

async fn run(config: bevy_technical_demo::config::Config) -> bevy_technical_demo::Result<()> {
    let (endpoint, mut incoming) = create_endpoint(&config).await?;

    println!("listening on {}", endpoint.local_addr()?);

    while let Some(connection) = incoming.next().await {
        println!("connection incoming");
        let future = handle_connection(connection);
        tokio::spawn(async move {
            if let Err(error) = future.await {
                eprintln!("connection failed: {reason}", reason = error)
            }
        });
    }

    Ok(())
}

async fn handle_connection(connection: quinn::Connecting) -> bevy_technical_demo::Result<()> {
    let mut connection = connection.await?;

    println!(
        "established, remote address: {remote_address}, protocol: {protocol}",
        remote_address = connection.connection.remote_address(),
        protocol = connection
            .connection
            .handshake_data()
            .unwrap()
            .downcast::<quinn::crypto::rustls::HandshakeData>()
            .unwrap()
            .protocol
            .map_or_else(
                || "<none>".into(),
                |x| String::from_utf8_lossy(&x).into_owned()
            )
    );

    while let Some(stream) = connection.bi_streams.next().await {
        let stream = match stream {
            Ok(s) => s,
            Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                println!("connection closed");
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };
        let future = handle_request(stream);
        tokio::spawn(async move {
            if let Err(error) = future.await {
                eprintln!("failed: {reason}", reason = error)
            }
        });
    }

    Ok(())
}

async fn handle_request(
    (mut send, recv): (quinn::SendStream, quinn::RecvStream),
) -> bevy_technical_demo::Result<()> {
    let request = recv.read_to_end(64 * 1024).await?;
    let request = bevy_technical_demo::protocol::Payload::deserialize(&request)?;

    println!("request: {:?}", request);

    let response =
        bevy_technical_demo::protocol::Payload::V1(bevy_technical_demo::protocol::Version1::Pong);
    let response = response.serialize()?;

    send.write_all(&response).await?;
    send.finish().await?;

    Ok(())
}

async fn create_endpoint(
    c: &bevy_technical_demo::config::Config,
) -> bevy_technical_demo::Result<(quinn::Endpoint, quinn::Incoming)> {
    // load server certificate
    let mut certificate_file = tokio::fs::File::open(&c.quic_client.certificate).await?;
    let mut private_key_file = tokio::fs::File::open(&c.quic_client.private_key).await?;

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
