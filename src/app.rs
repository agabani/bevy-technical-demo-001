use bevy::prelude::*;

use crate::{character, config};

pub fn run() -> crate::Result<()> {
    #[cfg(any(feature = "client", feature = "server"))]
    let runtime = tokio::runtime::Runtime::new()?;

    let mut app = App::new();

    let config = config::load(&[])?;

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
        use crate::database;
        database::configure(&mut app, &config, &runtime);
    }

    // configure networking
    #[cfg(any(feature = "client", feature = "server"))]
    {
        use crate::network;
        network::configure(&mut app, &config, &runtime);
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
    }

    app.run();

    Ok(())
}
