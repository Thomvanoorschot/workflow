use crate::models::Workflow;
use crate::workflow::storage::error::StorageError;
use crate::workflow::storage::WorkflowRepository;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct PostgresWorkflowRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> PostgresWorkflowRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl<'a> WorkflowRepository for PostgresWorkflowRepository<'a> {
    async fn save_workflow(&self, workflow: &Workflow) -> Result<(), StorageError> {
        let bytes = workflow.to_bytes()?;

        sqlx::query(
            "INSERT INTO workflows (id, user_id, name, data, status)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (id) DO UPDATE 
             SET data = EXCLUDED.data,
                 status = EXCLUDED.status,
                 updated_at = CURRENT_TIMESTAMP",
        )
        .bind(workflow.id)
        .bind(workflow.user_id)
        .bind(&workflow.name)
        .bind(&bytes)
        .bind(workflow.status.to_string())
        .execute(self.pool)
        .await?;

        Ok(())
    }

    async fn load_workflow(
        &self,
        user_id: Uuid,
        workflow_id: Uuid,
    ) -> Result<Option<Workflow>, StorageError> {
        let row = sqlx::query("SELECT data FROM workflows WHERE id = $1 AND user_id = $2")
            .bind(workflow_id)
            .bind(user_id)
            .fetch_optional(self.pool)
            .await?;

        match row {
            Some(row) => {
                let bytes: Vec<u8> = row.get("data");
                let workflow = Workflow::from_bytes(&bytes)?;
                Ok(Some(workflow))
            }
            None => Ok(None),
        }
    }

    async fn get_active_workflows_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Workflow>, StorageError> {
        let rows =
            sqlx::query("SELECT data FROM workflows WHERE user_id = $1 AND status = 'active'")
                .bind(user_id)
                .fetch_all(self.pool)
                .await?;

        let mut workflows = Vec::new();
        for row in rows {
            let bytes: Vec<u8> = row.get("data");
            let workflow = Workflow::from_bytes(&bytes)?;
            workflows.push(workflow);
        }

        Ok(workflows)
    }

    async fn get_all_workflows(&self) -> Result<Vec<(Uuid, Uuid, String, String)>, StorageError> {
        let rows = sqlx::query("SELECT id, user_id, name, status FROM workflows")
            .fetch_all(self.pool)
            .await?;

        let mut workflows = Vec::new();
        for row in rows {
            workflows.push((
                row.get("id"),
                row.get("user_id"),
                row.get("name"),
                row.get("status"),
            ));
        }

        Ok(workflows)
    }
}
