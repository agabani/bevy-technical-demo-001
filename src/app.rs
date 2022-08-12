use bevy::prelude::*;

use crate::character;

pub fn run() -> crate::Result<()> {
    let mut app = App::new();

    #[cfg(any(feature = "client", feature = "server"))]
    let config = crate::config::load(&[])?;

    #[cfg(any(feature = "client", feature = "server"))]
    let runtime = tokio::runtime::Runtime::new()?;

    #[cfg(feature = "client")]
    app.add_plugins(DefaultPlugins);

    #[cfg(feature = "server")]
    app.add_plugin(bevy::log::LogPlugin::default())
        .add_plugins(MinimalPlugins);

    #[cfg(feature = "server")]
    crate::database::configure(&mut app, &config, &runtime);

    #[cfg(any(feature = "client", feature = "server"))]
    crate::network::configure(&mut app, &config, &runtime);

    #[cfg(feature = "server")]
    crate::http::configure(&config, &runtime);

    app.add_plugin(character::Plugin);

    app.run();

    Ok(())
}
