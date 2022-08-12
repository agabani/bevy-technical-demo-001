use super::{Database, Response};

impl Database {
    pub(super) async fn update_last_seen(&self, public_id: uuid::Uuid) -> crate::Result<()> {
        let id = sqlx::query!(
            r#"
UPDATE server
SET last_seen = NOW()
WHERE public_id = $1
RETURNING id;
        "#,
            public_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if id.is_none() {
            self.sender.send(Response::ServerDeregistered)?;
        }

        Ok(())
    }
}
