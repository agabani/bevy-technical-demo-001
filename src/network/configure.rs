use bevy::prelude::*;

use crate::config;

use super::{backend, plugin};

pub(crate) fn configure(app: &mut App, config: &config::Config, runtime: &tokio::runtime::Runtime) {
    let config = config.clone();

    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

    app.insert_resource(receiver);

    #[cfg(feature = "server")]
    {
        app.insert_resource(plugin::ServerPublicId(uuid::Uuid::new_v4()));
        app.insert_resource(plugin::ServerEndpoint {
            ip_address: local_ip_address::local_ip().unwrap().to_string(),
            port: config.quic_server.port,
        });
    }

    app.add_plugin(plugin::Plugin);

    runtime.spawn(async move {
        if let Err(error) = backend::run(config, sender).await {
            error!(error = error, "error");
        }
    });
}
