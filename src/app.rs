use bevy::prelude::*;

use crate::character;

pub fn run() -> crate::Result<()> {
    #[cfg(any(feature = "client", feature = "server"))]
    let runtime = tokio::runtime::Runtime::new()?;

    let mut app = App::new();

    // configure default plugins
    #[cfg(feature = "client")]
    {
        app.add_plugins(DefaultPlugins);
    }
    #[cfg(feature = "server")]
    {
        app.add_plugin(bevy::log::LogPlugin::default())
            .add_plugins(MinimalPlugins);
    }

    app.add_plugin(character::Plugin);

    // configure database
    #[cfg(feature = "server")]
    {
        use crate::{config, database};

        let config = config::load(&[])?;

        let (request_sender, request_receiver) = tokio::sync::mpsc::unbounded_channel();
        let (response_sender, response_receiver) = tokio::sync::mpsc::unbounded_channel();

        app.insert_resource(request_sender);
        app.insert_resource(response_receiver);

        app.add_plugin(database::plugin::Plugin);

        runtime.spawn(async move {
            if let Err(error) = database::run(config, request_receiver, response_sender).await {
                error!(error = error, "error");
            }
        });
    }

    // configure networking
    #[cfg(any(feature = "client", feature = "server"))]
    {
        use crate::{config, network_1, quic};

        let config = config::load(&[])?;
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

        app.insert_resource(receiver);

        #[cfg(feature = "server")]
        {
            app.insert_resource(network_1::ServerPublicId(uuid::Uuid::new_v4()));
            app.insert_resource(network_1::ServerEndpoint {
                ip_address: local_ip_address::local_ip().unwrap().to_string(),
                port: config.quic_server.port,
            });
        }

        app.add_plugin(network_1::Plugin);

        #[cfg(feature = "client")]
        {
            #[allow(clippy::redundant_clone)]
            let quic_config = config.clone();

            #[allow(clippy::redundant_clone)]
            let sender = sender.clone();

            runtime.spawn(async move {
                if let Err(error) = quic::client::run(quic_config, sender).await {
                    error!(error = error, "error");
                }
            });
        }
        #[cfg(feature = "server")]
        {
            use crate::http_server;

            #[allow(clippy::redundant_clone)]
            let http_config = config.clone();

            runtime.spawn(async move {
                if let Err(error) = http_server::run(http_config).await {
                    error!(error = error, "error");
                }
            });

            #[allow(clippy::redundant_clone)]
            let quic_config = config.clone();

            #[allow(clippy::redundant_clone)]
            let sender = sender.clone();

            runtime.spawn(async move {
                if let Err(error) = quic::server::run(quic_config, sender).await {
                    error!(error = error, "error");
                }
            });
        }
    }

    app.run();

    Ok(())
}
