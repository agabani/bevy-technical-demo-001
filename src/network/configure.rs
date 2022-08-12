use bevy::prelude::*;

use crate::{config, network};

pub(crate) fn configure(app: &mut App, config: &config::Config, runtime: &tokio::runtime::Runtime) {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

    app.insert_resource(receiver);

    #[cfg(feature = "server")]
    {
        app.insert_resource(network::plugin::ServerPublicId(uuid::Uuid::new_v4()));
        app.insert_resource(network::plugin::ServerEndpoint {
            ip_address: local_ip_address::local_ip().unwrap().to_string(),
            port: config.quic_server.port,
        });
    }

    app.add_plugin(network::plugin::Plugin);

    let quic_config = config.clone();
    runtime.spawn(async move {
        if let Err(error) = network::run(quic_config, sender).await {
            error!(error = error, "error");
        }
    });
}
