use super::{Postgres, Response};

impl Postgres {
    pub(crate) async fn server_register(
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
            self.sender.send(Response::ServerRegistered)?;
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
            self.sender.send(Response::ServerRegisterConflicted)?;
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
            self.sender.send(Response::ServerRegisterConflicted)?;
            return Ok(());
        }

        self.sender.send(Response::ServerRegistered)?;
        Ok(())
    }
}
