use super::Workflow;
use std::error::Error;

#[async_trait::async_trait]
pub trait WorkflowStorage {
    type Error: Error;

    async fn save_workflow(
        &self,
        workflow_id: &str,
        workflow: &Workflow,
    ) -> Result<(), Self::Error>;
    async fn load_workflow(&self, workflow_id: &str) -> Result<Option<Workflow>, Self::Error>;
}

// A simple in-memory implementation for demo purposes
use std::collections::HashMap;
use std::sync::Mutex;

pub struct InMemoryStorage {
    workflows: Mutex<HashMap<String, String>>, // workflow_id -> serialized workflow
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            workflows: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl WorkflowStorage for InMemoryStorage {
    type Error = serde_json::Error;

    async fn save_workflow(
        &self,
        workflow_id: &str,
        workflow: &Workflow,
    ) -> Result<(), Self::Error> {
        let serialized = serde_json::to_string(workflow)?;
        self.workflows
            .lock()
            .unwrap()
            .insert(workflow_id.to_string(), serialized);
        Ok(())
    }

    async fn load_workflow(&self, workflow_id: &str) -> Result<Option<Workflow>, Self::Error> {
        if let Some(serialized) = self.workflows.lock().unwrap().get(workflow_id) {
            Ok(Some(serde_json::from_str(serialized)?))
        } else {
            Ok(None)
        }
    }
}
