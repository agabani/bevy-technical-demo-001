use bevy::prelude::*;

use crate::{
    config,
    database::{postgres, protocol},
};

pub(crate) async fn run(
    config: config::Config,
    mut receiver: protocol::InternalReceiver,
    sender: protocol::InternalSender,
) -> crate::Result<()> {
    postgres::Postgres::migrate(&config).await?;

    let database = postgres::Postgres::new(&config, sender).await?;

    while let Some(message) = receiver.recv().await {
        tokio::spawn(handle_request(database.clone(), message));
    }

    info!("this shouldn't have happened");

    Ok(())
}

async fn handle_request(
    database: postgres::Postgres,
    request: protocol::Request,
) -> crate::Result<()> {
    match request {
        protocol::Request::ServerRegister {
            public_id,
            ip_address,
            port,
        } => {
            database
                .server_register(public_id, ip_address, port.into())
                .await?;
        }
        protocol::Request::ServerUpdateServerLastSeen { public_id } => {
            database.update_last_seen(public_id).await?
        }
    };

    Ok(())
}
