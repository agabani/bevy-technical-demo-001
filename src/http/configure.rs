use bevy::prelude::*;

use crate::config;

pub(crate) fn configure(config: &config::Config, runtime: &tokio::runtime::Runtime) {
    #[allow(clippy::redundant_clone)]
    let config = config.clone();

    runtime.spawn(async move {
        if let Err(error) = crate::http::backend::run(config).await {
            error!(error = error, "error");
        }
    });
}
