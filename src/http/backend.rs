use axum::{routing::get, Router, Server};
use tokio::net::TcpListener;

use crate::config;

pub(crate) async fn run(config: config::Config) -> crate::Result<()> {
    let addr = format!("{}:{}", config.http_server.host, config.http_server.port);

    let tcp_listener = TcpListener::bind(addr).await?.into_std()?;

    let app = Router::new()
        .route("/health/liveness", get(|| async { "Ok" }))
        .route("/health/readiness", get(|| async { "Ok" }));

    let server = Server::from_tcp(tcp_listener)?.serve(app.into_make_service());

    server.await?;

    Ok(())
}
