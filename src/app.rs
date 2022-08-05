use bevy::prelude::*;

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

    // configure networking
    #[cfg(any(feature = "client", feature = "server"))]
    {
        use crate::{config, network, quic};

        let config = config::load(&[])?;
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<crate::protocol::Event>();

        app.insert_resource(receiver);
        app.add_plugin(network::Plugin);

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
