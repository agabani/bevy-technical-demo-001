use bevy::prelude::*;

use crate::config;

use super::{backend, plugin};

pub(crate) fn configure(app: &mut App, config: &config::Config, runtime: &tokio::runtime::Runtime) {
    let (request_sender, request_receiver) = tokio::sync::mpsc::unbounded_channel();
    let (response_sender, response_receiver) = tokio::sync::mpsc::unbounded_channel();

    app.insert_resource(request_sender);
    app.insert_resource(response_receiver);

    app.add_plugin(plugin::Plugin);

    let config = config.clone();

    runtime.spawn(async move {
        if let Err(error) = backend::run(config, request_receiver, response_sender).await {
            error!(error = error, "error");
        }
    });
}
