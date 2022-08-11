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

    let database = Database::new(&config).await?;

    while let Some(message) = receiver.recv().await {
        tokio::spawn(handle_request(database.clone(), sender.clone(), message));
    }

    info!("this shouldn't have happened");

    Ok(())
}

async fn handle_request(
    database: Database,
    sender: tokio::sync::mpsc::UnboundedSender<Response>,
    request: Request,
) -> crate::Result<()> {
    match request {
        Request::RegisterServer {
            public_id,
            ip_address,
            port,
        } => {
            database
                .register_server(public_id, ip_address, port.into())
                .await?;

            sender.send(Response::RegisteredServer)?;
        }
        Request::UpdateServerLastSeen { public_id } => database.update_last_seen(public_id).await?,
    };

    Ok(())
}

#[derive(Debug)]
pub enum Request {
    RegisterServer {
        public_id: uuid::Uuid,
        ip_address: String,
        port: u16,
    },
    UpdateServerLastSeen {
        public_id: uuid::Uuid,
    },
}

#[derive(Debug)]
pub enum Response {
    RegisteredServer,
}

#[derive(Clone)]
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

    async fn register_server(
        &self,
        public_id: uuid::Uuid,
        ip_address: String,
        port: i32,
    ) -> crate::Result<()> {
        let id = sqlx::query!(
            r#"
INSERT INTO server (public_id, last_seen, ip_address, port)
VALUES ($1, NOW(), $2, $3)
ON CONFLICT (ip_address, port) DO NOTHING
RETURNING id;
"#,
            public_id,
            ip_address,
            port
        )
        .fetch_optional(&self.pool)
        .await?;

        if id.is_some() {
            // todo: send registered
            return Ok(());
        }

        let id = sqlx::query!(
            r#"
DELETE
FROM server
WHERE ip_address = $1
    AND port = $2
    AND last_seen < NOW() - INTERVAL '5 seconds'
RETURNING id;
            "#,
            ip_address,
            port
        )
        .fetch_optional(&self.pool)
        .await?;

        if id.is_none() {
            // todo: send conflict
            return Ok(());
        }

        let id = sqlx::query!(
            r#"
INSERT INTO server (public_id, last_seen, ip_address, port)
VALUES ($1, NOW(), $2, $3)
ON CONFLICT (ip_address, port) DO NOTHING
RETURNING id;
"#,
            public_id,
            ip_address,
            port
        )
        .fetch_optional(&self.pool)
        .await?;

        if id.is_none() {
            // todo: send registration error
            return Ok(());
        }

        Ok(())
    }

    pub(crate) async fn update_last_seen(&self, public_id: uuid::Uuid) -> crate::Result<()> {
        sqlx::query!(
            r#"
UPDATE server
SET last_seen = NOW()
WHERE public_id = $1
RETURNING id;
        "#,
            public_id
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
