use super::{Workflow, WorkflowStorage};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::error::Error;
use std::fmt;
use uuid::Uuid;

#[derive(Debug)]
pub enum PostgresStorageError {
    Database(sqlx::Error),
    Serialization(serde_json::Error),
    UuidParse(uuid::Error),
}

impl fmt::Display for PostgresStorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(e) => write!(f, "Database error: {}", e),
            Self::Serialization(e) => write!(f, "Serialization error: {}", e),
            Self::UuidParse(e) => write!(f, "UUID parsing error: {}", e),
        }
    }
}

impl Error for PostgresStorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(e) => Some(e),
            Self::Serialization(e) => Some(e),
            Self::UuidParse(e) => Some(e),
        }
    }
}

impl From<sqlx::Error> for PostgresStorageError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err)
    }
}

impl From<serde_json::Error> for PostgresStorageError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err)
    }
}

impl From<uuid::Error> for PostgresStorageError {
    fn from(err: uuid::Error) -> Self {
        Self::UuidParse(err)
    }
}

pub struct PostgresStorage {
    pool: PgPool,
}

impl PostgresStorage {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn setup_database(&self) -> Result<(), sqlx::Error> {
        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE TABLE IF NOT EXISTS workflows (
                id UUID PRIMARY KEY,
                user_id UUID NOT NULL REFERENCES users(id),
                name TEXT NOT NULL,
                state JSONB NOT NULL,
                status TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                CONSTRAINT valid_status CHECK (status IN ('ACTIVE', 'COMPLETED', 'FAILED'))
            );

            CREATE TABLE IF NOT EXISTS events (
                id UUID PRIMARY KEY,
                user_id UUID NOT NULL REFERENCES users(id),
                name TEXT NOT NULL,
                payload JSONB NOT NULL,
                processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_workflows_user_id ON workflows(user_id);
            CREATE INDEX IF NOT EXISTS idx_workflows_status ON workflows(status);
            CREATE INDEX IF NOT EXISTS idx_events_user_id ON events(user_id);
            CREATE INDEX IF NOT EXISTS idx_events_name ON events(name);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn save_workflow_for_user(
        &self,
        user_id: Uuid,
        workflow_id: Uuid,
        name: &str,
        workflow: &Workflow,
    ) -> Result<(), PostgresStorageError> {
        let state = serde_json::to_value(workflow)?;

        sqlx::query(
            r#"
            INSERT INTO workflows (id, user_id, name, state, status)
            VALUES ($1, $2, $3, $4, 'ACTIVE')
            ON CONFLICT (id) 
            DO UPDATE SET 
                state = EXCLUDED.state,
                status = EXCLUDED.status,
                updated_at = NOW()
            "#,
        )
        .bind(workflow_id)
        .bind(user_id)
        .bind(name)
        .bind(state)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_workflow_for_user(
        &self,
        user_id: Uuid,
        workflow_id: Uuid,
    ) -> Result<Option<Workflow>, PostgresStorageError> {
        let row = sqlx::query(
            r#"
            SELECT state
            FROM workflows
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(workflow_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let state: serde_json::Value = row.try_get("state")?;
            let workflow: Workflow = serde_json::from_value(state)?;
            Ok(Some(workflow))
        } else {
            Ok(None)
        }
    }

    pub async fn record_event(
        &self,
        user_id: Uuid,
        event: &super::Event,
    ) -> Result<(), PostgresStorageError> {
        let payload = serde_json::to_value(&event.payload)?;

        sqlx::query(
            r#"
            INSERT INTO events (id, user_id, name, payload)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(&event.name)
        .bind(payload)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_active_workflows_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<(Uuid, Workflow)>, PostgresStorageError> {
        let rows = sqlx::query(
            r#"
            SELECT id, state
            FROM workflows
            WHERE user_id = $1 AND status = 'ACTIVE'
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut workflows = Vec::new();
        for row in rows {
            let id: Uuid = row.try_get("id")?;
            let state: serde_json::Value = row.try_get("state")?;
            let workflow: Workflow = serde_json::from_value(state)?;
            workflows.push((id, workflow));
        }

        Ok(workflows)
    }
}

#[async_trait::async_trait]
impl WorkflowStorage for PostgresStorage {
    type Error = PostgresStorageError;

    async fn save_workflow(
        &self,
        workflow_id: &str,
        workflow: &Workflow,
    ) -> Result<(), Self::Error> {
        // This is just a compatibility layer for the old interface
        // In a real application, you'd want to refactor this to use proper user IDs
        let user_id = Uuid::new_v4(); // Temporary user ID
        self.save_workflow_for_user(user_id, Uuid::parse_str(workflow_id)?, "default", workflow)
            .await
    }

    async fn load_workflow(&self, workflow_id: &str) -> Result<Option<Workflow>, Self::Error> {
        // This is just a compatibility layer for the old interface
        let user_id = Uuid::new_v4(); // Temporary user ID
        self.load_workflow_for_user(user_id, Uuid::parse_str(workflow_id)?)
            .await
    }
}
