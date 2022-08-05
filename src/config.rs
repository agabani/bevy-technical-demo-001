#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub http_server: HttpServer,
    pub quic_client: QuicClient,
    pub quic_server: QuicServer,
}

#[derive(Clone, serde::Deserialize)]
pub struct HttpServer {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, serde::Deserialize)]
pub struct QuicClient {
    pub host: String,
    pub port: u16,
    pub certificate: String,
    pub private_key: String,
}

#[derive(Clone, serde::Deserialize)]
pub struct QuicServer {
    pub host: String,
    pub port: u16,
    pub certificate: String,
    pub private_key: String,
    pub name: String,
}

/// Loads the configuration from the environment variables and the config file.
///
/// # Errors
///
/// If the configuration file cannot be loaded, an error is returned.
pub fn load(overrides: &[(&str, &str)]) -> crate::Result<Config> {
    let mut config_builder = config::Config::builder()
        .set_default("http_server.host", "127.0.0.1")?
        .set_default("http_server.port", "80")?
        .set_default("quic_client.host", "127.0.0.1")?
        .set_default("quic_client.port", "0")?
        .set_default("quic_client.certificate", "tls.crt")?
        .set_default("quic_client.private_key", "tls.key")?
        .set_default("quic_server.host", "127.0.0.1")?
        .set_default("quic_server.port", "4433")?
        .set_default("quic_server.certificate", "tls.crt")?
        .set_default("quic_server.private_key", "tls.key")?
        .set_default("quic_server.name", "localhost")?
        .add_source(config::File::with_name("config").required(false))
        .add_source(config::Environment::with_prefix("BEVY_TECHNICAL_DEMO").separator("__"));

    for &(key, value) in overrides {
        config_builder = config_builder.set_override(key, value)?;
    }

    config_builder
        .build()?
        .try_deserialize()
        .map_err(Into::into)
}
