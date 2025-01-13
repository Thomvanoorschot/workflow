use crate::models::Event;
use crate::workflow::storage::{EventRepository, StorageError};
use serde_json::Value as JsonValue;
use sqlx::{Executor, PgPool, Row};
use uuid::Uuid;

pub struct PostgresEventRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> PostgresEventRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl<'a> EventRepository for PostgresEventRepository<'a> {
    async fn save_event(&self, user_id: Uuid, event: &Event) -> Result<(), StorageError> {
        let (event_type, event_data) = match event {
            Event::UserActivity => ("user_activity".to_string(), serde_json::Value::Null),
            Event::Timer { timer_id } => (
                "timer".to_string(),
                serde_json::json!({ "timer_id": timer_id }),
            ),
        };

        let query = "INSERT INTO events (user_id, event_type, event_data) VALUES ($1, $2, $3)";
        self.pool
            .execute(
                sqlx::query(query)
                    .bind(user_id)
                    .bind(&event_type)
                    .bind(&event_data),
            )
            .await?;
        Ok(())
    }

    async fn get_all_events(
        &self,
    ) -> Result<Vec<(Uuid, Uuid, String, JsonValue, time::OffsetDateTime)>, StorageError> {
        let query = "SELECT id, user_id, event_type, event_data, created_at FROM events ORDER BY created_at";
        let rows = sqlx::query(query).fetch_all(self.pool).await?;

        let mut events = Vec::new();
        for row in rows {
            events.push((
                row.try_get("id")?,
                row.try_get("user_id")?,
                row.try_get("event_type")?,
                row.try_get("event_data")?,
                row.try_get("created_at")?,
            ));
        }

        Ok(events)
    }
}
