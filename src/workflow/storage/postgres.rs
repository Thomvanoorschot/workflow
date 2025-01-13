use crate::models::{Event, Workflow};
use crate::workflow::storage::error::StorageError;
use crate::workflow::storage::repositories::{
    PostgresEventRepository, PostgresUserRepository, PostgresWorkflowRepository,
};
use crate::workflow::storage::{EventRepository, Storage, UserRepository, WorkflowRepository};
use sqlx::{Executor, PgPool};
use uuid::Uuid;

pub struct PostgresStorage {
    pool: PgPool,
}

impl PostgresStorage {
    pub async fn new(database_url: &str) -> Result<Self, StorageError> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn setup_database(&self) -> Result<(), StorageError> {
        let drop_query = r#"
            DROP TABLE IF EXISTS workflows CASCADE;
            DROP TABLE IF EXISTS events CASCADE;
            DROP TABLE IF EXISTS users CASCADE;
        "#;
        self.pool.execute(drop_query).await?;

        let query = r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS workflows (
                id UUID PRIMARY KEY,
                user_id UUID NOT NULL REFERENCES users(id),
                name TEXT NOT NULL,
                nodes JSONB NOT NULL,
                status TEXT NOT NULL CHECK (status IN ('active', 'completed', 'failed')),
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS events (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL REFERENCES users(id),
                event_type TEXT NOT NULL,
                event_data JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_workflows_user_id ON workflows(user_id);
            CREATE INDEX IF NOT EXISTS idx_workflows_status ON workflows(status);
            CREATE INDEX IF NOT EXISTS idx_events_user_id ON events(user_id);
        "#;

        self.pool.execute(query).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl UserRepository for PostgresStorage {
    async fn create_user(&self, user_id: Uuid, name: &str) -> Result<(), StorageError> {
        PostgresUserRepository::new(&self.pool)
            .create_user(user_id, name)
            .await
    }
}

#[async_trait::async_trait]
impl WorkflowRepository for PostgresStorage {
    async fn save_workflow(&self, workflow: &Workflow) -> Result<(), StorageError> {
        PostgresWorkflowRepository::new(&self.pool)
            .save_workflow(workflow)
            .await
    }

    async fn load_workflow(
        &self,
        user_id: Uuid,
        workflow_id: Uuid,
    ) -> Result<Option<Workflow>, StorageError> {
        PostgresWorkflowRepository::new(&self.pool)
            .load_workflow(user_id, workflow_id)
            .await
    }

    async fn get_active_workflows_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Workflow>, StorageError> {
        PostgresWorkflowRepository::new(&self.pool)
            .get_active_workflows_for_user(user_id)
            .await
    }

    async fn get_all_workflows(&self) -> Result<Vec<(Uuid, Uuid, String, String)>, StorageError> {
        PostgresWorkflowRepository::new(&self.pool)
            .get_all_workflows()
            .await
    }
}

#[async_trait::async_trait]
impl EventRepository for PostgresStorage {
    async fn save_event(&self, user_id: Uuid, event: &Event) -> Result<(), StorageError> {
        PostgresEventRepository::new(&self.pool)
            .save_event(user_id, event)
            .await
    }

    async fn get_all_events(
        &self,
    ) -> Result<Vec<(Uuid, Uuid, String, serde_json::Value, time::OffsetDateTime)>, StorageError>
    {
        PostgresEventRepository::new(&self.pool)
            .get_all_events()
            .await
    }
}

impl Storage for PostgresStorage {
    async fn setup_database(&self) -> Result<(), StorageError> {
        let drop_query = r#"
            DROP TABLE IF EXISTS workflows CASCADE;
            DROP TABLE IF EXISTS events CASCADE;
            DROP TABLE IF EXISTS users CASCADE;
        "#;
        self.pool.execute(drop_query).await?;

        let query = r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS workflows (
                id UUID PRIMARY KEY,
                user_id UUID NOT NULL REFERENCES users(id),
                name TEXT NOT NULL,
                nodes JSONB NOT NULL,
                status TEXT NOT NULL CHECK (status IN ('active', 'completed', 'failed')),
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS events (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL REFERENCES users(id),
                event_type TEXT NOT NULL,
                event_data JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_workflows_user_id ON workflows(user_id);
            CREATE INDEX IF NOT EXISTS idx_workflows_status ON workflows(status);
            CREATE INDEX IF NOT EXISTS idx_events_user_id ON events(user_id);
        "#;

        self.pool.execute(query).await?;
        Ok(())
    }
}
