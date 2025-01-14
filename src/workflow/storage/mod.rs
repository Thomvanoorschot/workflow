pub mod error;
pub mod postgres;
pub mod repositories;

use crate::models::{Event, Workflow};
use error::StorageError;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait UserRepository {
    async fn create_user(&self, user_id: Uuid, name: &str) -> Result<(), StorageError>;
}

#[async_trait::async_trait]
pub trait WorkflowRepository {
    async fn save_workflow(&self, workflow: &Workflow) -> Result<(), StorageError>;
    async fn load_workflow(
        &self,
        user_id: Uuid,
        workflow_id: Uuid,
    ) -> Result<Option<Workflow>, StorageError>;
    async fn get_active_workflows_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Workflow>, StorageError>;
    async fn get_all_workflows(&self) -> Result<Vec<(Uuid, Uuid, String, String)>, StorageError>;
}

#[async_trait::async_trait]
pub trait EventRepository {
    async fn save_event(&self, user_id: Uuid, event: &Event) -> Result<(), StorageError>;
    async fn get_all_events(
        &self,
    ) -> Result<Vec<(Uuid, Uuid, String, serde_json::Value, time::OffsetDateTime)>, StorageError>;
}

pub trait Storage: UserRepository + WorkflowRepository + EventRepository {
    async fn setup_database(&self) -> Result<(), StorageError>;
}
