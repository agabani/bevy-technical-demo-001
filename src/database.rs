use std::path;

use sqlx::{
    migrate::{MigrateDatabase, Migrator},
    postgres::{PgConnectOptions, PgPool, PgPoolOptions, PgSslMode},
    Postgres,
};

use crate::config;

pub(crate) async fn new(config: &config::Config) -> crate::Result<PgPool> {
    PgPoolOptions::new()
        .connect_with(database_connect_options(config))
        .await
        .map_err(Into::into)
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

        let database_pool = new(config).await?;
        migrator.run(&database_pool).await?;
    }

    Ok(())
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
