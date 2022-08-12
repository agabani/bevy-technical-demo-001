use bevy::prelude::*;

use crate::{
    config,
    network::{protocol, quic},
};

pub(crate) async fn run(
    config: config::Config,
    sender: protocol::InternalSender,
) -> crate::Result<()> {
    #[cfg(feature = "client")]
    {
        #[allow(clippy::redundant_clone)]
        quic::client::run(config.clone(), sender.clone()).await?;
    }

    #[cfg(feature = "server")]
    {
        #[allow(clippy::redundant_clone)]
        quic::server::run(config.clone(), sender.clone()).await?;
    }

    todo!();
}
