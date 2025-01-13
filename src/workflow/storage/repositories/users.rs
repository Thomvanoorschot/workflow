use crate::workflow::storage::{StorageError, UserRepository};
use sqlx::{Executor, PgPool};
use uuid::Uuid;

pub struct PostgresUserRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> PostgresUserRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl<'a> UserRepository for PostgresUserRepository<'a> {
    async fn create_user(&self, user_id: Uuid, name: &str) -> Result<(), StorageError> {
        let query = "INSERT INTO users (id, name) VALUES ($1, $2) ON CONFLICT (id) DO NOTHING";
        self.pool
            .execute(sqlx::query(query).bind(user_id).bind(name))
            .await?;
        Ok(())
    }
}
