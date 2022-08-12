mod server_register;
mod server_update_last_seen;

use std::path;

use bevy::prelude::*;
use sqlx::{
    migrate::{MigrateDatabase, Migrator},
    postgres::{PgConnectOptions, PgPool, PgPoolOptions, PgSslMode},
    Postgres,
};

use crate::config;

pub(crate) async fn run(
    config: config::Config,
    mut receiver: tokio::sync::mpsc::UnboundedReceiver<Request>,
    sender: tokio::sync::mpsc::UnboundedSender<Response>,
) -> crate::Result<()> {
    Database::migrate(&config).await?;

    let database = Database::new(&config, sender).await?;

    while let Some(message) = receiver.recv().await {
        tokio::spawn(handle_request(database.clone(), message));
    }

    info!("this shouldn't have happened");

    Ok(())
}

async fn handle_request(database: Database, request: Request) -> crate::Result<()> {
    match request {
        Request::ServerRegister {
            public_id,
            ip_address,
            port,
        } => {
            database
                .server_register(public_id, ip_address, port.into())
                .await?;
        }
        Request::ServerUpdateServerLastSeen { public_id } => {
            database.update_last_seen(public_id).await?
        }
    };

    Ok(())
}

#[derive(Debug)]
pub enum Request {
    ServerRegister {
        public_id: uuid::Uuid,
        ip_address: String,
        port: u16,
    },
    ServerUpdateServerLastSeen {
        public_id: uuid::Uuid,
    },
}

#[derive(Debug)]
pub enum Response {
    ServerRegistered,
    ServerRegisterConflicted,
    ServerDeregistered,
}

#[derive(Clone)]
pub(crate) struct Database {
    pool: PgPool,
    sender: tokio::sync::mpsc::UnboundedSender<Response>,
}

impl Database {
    pub(crate) async fn new(
        config: &config::Config,
        sender: tokio::sync::mpsc::UnboundedSender<Response>,
    ) -> crate::Result<Database> {
        let pool = PgPoolOptions::new()
            .connect_with(database_connect_options(config))
            .await?;

        Ok(Database { pool, sender })
    }

    pub(crate) async fn migrate(config: &config::Config) -> crate::Result<()> {
        if config.database.migration.migrate {
            let migrator = Migrator::new(path::Path::new(&config.database.migration.path)).await?;

            if config.database.migration.create_database {
                let url = format!(
                    "postgres://{}:{}@{}:{}/{}",
                    config.database.username,
                    config.database.password,
                    config.database.host,
                    config.database.port,
                    config.database.database_name
                );

                if !Postgres::database_exists(&url).await? {
                    Postgres::create_database(&url).await?;
                }
            }

            let pool = PgPoolOptions::new()
                .connect_with(database_connect_options(config))
                .await?;

            migrator.run(&pool).await?;
        }

        Ok(())
    }
}

fn database_connect_options(config: &config::Config) -> PgConnectOptions {
    server_connect_options(config).database(&config.database.database_name)
}

fn server_connect_options(config: &config::Config) -> PgConnectOptions {
    PgConnectOptions::new()
        .host(&config.database.host)
        .port(config.database.port)
        .username(&config.database.username)
        .password(&config.database.password)
        .ssl_mode(if config.database.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        })
}
