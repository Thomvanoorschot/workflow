use crate::models::{Event, Workflow};
use crate::workflow::storage::error::StorageError;
use crate::workflow::storage::repositories::{
    PostgresEventRepository, PostgresUserRepository, PostgresWorkflowRepository,
};
use crate::workflow::storage::{EventRepository, Storage, UserRepository, WorkflowRepository};
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostgresStorage {
    pool: PgPool,
}

impl PostgresStorage {
    pub async fn new(database_url: &str) -> Result<Self, StorageError> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
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
        Ok(())
    }
}
