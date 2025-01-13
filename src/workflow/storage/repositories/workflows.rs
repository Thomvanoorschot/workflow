use crate::models::Workflow;
use crate::workflow::storage::{StorageError, WorkflowRepository};
use sqlx::{Executor, PgPool, Row};
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
        let query = r#"
            INSERT INTO workflows (id, user_id, name, nodes, status)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO UPDATE
            SET nodes = $4,
                status = $5,
                updated_at = CURRENT_TIMESTAMP
        "#;

        self.pool
            .execute(
                sqlx::query(query)
                    .bind(workflow.id)
                    .bind(workflow.user_id)
                    .bind(&workflow.name)
                    .bind(serde_json::to_value(&workflow.nodes)?)
                    .bind(workflow.status.to_string()),
            )
            .await?;

        Ok(())
    }

    async fn load_workflow(
        &self,
        user_id: Uuid,
        workflow_id: Uuid,
    ) -> Result<Option<Workflow>, StorageError> {
        let query =
            "SELECT id, user_id, name, nodes, status FROM workflows WHERE id = $1 AND user_id = $2";
        let row = sqlx::query(query)
            .bind(workflow_id)
            .bind(user_id)
            .fetch_optional(self.pool)
            .await?;

        if let Some(row) = row {
            let nodes = serde_json::from_value(row.try_get("nodes")?)?;
            Ok(Some(Workflow {
                id: row.try_get("id")?,
                user_id: row.try_get("user_id")?,
                name: row.try_get("name")?,
                nodes,
                status: row
                    .try_get::<String, _>("status")?
                    .parse()
                    .unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_active_workflows_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Workflow>, StorageError> {
        let query = "SELECT id, user_id, name, nodes, status FROM workflows WHERE user_id = $1 AND status = 'active'";
        let rows = sqlx::query(query)
            .bind(user_id)
            .fetch_all(self.pool)
            .await?;

        let workflows = rows
            .into_iter()
            .map(|row| {
                let nodes = serde_json::from_value(row.try_get("nodes")?)?;
                Ok(Workflow {
                    id: row.try_get("id")?,
                    user_id: row.try_get("user_id")?,
                    name: row.try_get("name")?,
                    nodes,
                    status: row
                        .try_get::<String, _>("status")?
                        .parse()
                        .unwrap_or_default(),
                })
            })
            .collect::<Result<Vec<_>, StorageError>>()?;

        Ok(workflows)
    }

    async fn get_all_workflows(&self) -> Result<Vec<(Uuid, Uuid, String, String)>, StorageError> {
        let query = "SELECT id, user_id, name, status FROM workflows ORDER BY created_at";
        let rows = sqlx::query(query).fetch_all(self.pool).await?;

        let mut workflows = Vec::new();
        for row in rows {
            workflows.push((
                row.try_get("id")?,
                row.try_get("user_id")?,
                row.try_get("name")?,
                row.try_get("status")?,
            ));
        }

        Ok(workflows)
    }
}
