use std::path;

use sqlx::{
    migrate::{MigrateDatabase, Migrator},
    postgres::{PgConnectOptions, PgPool, PgPoolOptions, PgSslMode},
    Postgres,
};

use crate::config;

pub(crate) struct Database {
    pool: PgPool,
}

impl Database {
    pub(crate) async fn new(config: &config::Config) -> crate::Result<Database> {
        let pool = PgPoolOptions::new()
            .connect_with(database_connect_options(config))
            .await?;

        Ok(Database { pool })
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

            let database = Database::new(config).await?;
            migrator.run(&database.pool).await?;
        }

        Ok(())
    }

    pub(crate) async fn add_server(&self, ip_address: String) -> crate::Result<()> {
        sqlx::query!(
            r#"
INSERT INTO server (ip_address, last_seen)
VALUES ($1, NOW())
ON CONFLICT (ip_address) DO UPDATE SET last_seen = NOW()
RETURNING id;
"#,
            ip_address
        )
        .fetch_one(&self.pool)
        .await?;

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
