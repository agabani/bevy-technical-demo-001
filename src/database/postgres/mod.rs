mod server_register;
mod server_update_last_seen;

use sqlx::{
    migrate::{MigrateDatabase, Migrator},
    postgres::{PgConnectOptions, PgPool, PgPoolOptions, PgSslMode},
};

use crate::config;

use super::protocol::Response;

#[derive(Clone)]
pub(crate) struct Postgres {
    pool: PgPool,
    sender: tokio::sync::mpsc::UnboundedSender<Response>,
}

impl Postgres {
    pub(crate) async fn new(
        config: &config::Config,
        sender: tokio::sync::mpsc::UnboundedSender<Response>,
    ) -> crate::Result<Postgres> {
        let pool = PgPoolOptions::new()
            .connect_with(database_connect_options(config))
            .await?;

        Ok(Postgres { pool, sender })
    }

    pub(crate) async fn migrate(config: &config::Config) -> crate::Result<()> {
        if config.database.migration.migrate {
            let migrator =
                Migrator::new(std::path::Path::new(&config.database.migration.path)).await?;

            if config.database.migration.create_database {
                let url = format!(
                    "postgres://{}:{}@{}:{}/{}",
                    config.database.username,
                    config.database.password,
                    config.database.host,
                    config.database.port,
                    config.database.database_name
                );

                if !sqlx::Postgres::database_exists(&url).await? {
                    sqlx::Postgres::create_database(&url).await?;
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
