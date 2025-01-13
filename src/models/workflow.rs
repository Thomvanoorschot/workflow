use super::{Event, Node, NodeId, NodeStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub nodes: Vec<Node>,
    pub status: WorkflowStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Active,
    Completed,
    Failed,
}

impl ToString for WorkflowStatus {
    fn to_string(&self) -> String {
        match self {
            WorkflowStatus::Active => "active".to_string(),
            WorkflowStatus::Completed => "completed".to_string(),
            WorkflowStatus::Failed => "failed".to_string(),
        }
    }
}

impl Default for WorkflowStatus {
    fn default() -> Self {
        WorkflowStatus::Active
    }
}

impl std::str::FromStr for WorkflowStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(WorkflowStatus::Active),
            "completed" => Ok(WorkflowStatus::Completed),
            "failed" => Ok(WorkflowStatus::Failed),
            _ => Err(format!("Invalid workflow status: {}", s)),
        }
    }
}

impl Workflow {
    pub fn new(nodes: Vec<Node>) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: String::new(),
            nodes,
            status: WorkflowStatus::Active,
        }
    }

    pub fn process_event(&mut self, event: &Event) {
        let mut to_complete = Vec::new();
        let mut to_activate = Vec::new();

        // First pass: collect nodes to complete and activate
        for (node_id, node) in self.nodes.iter().enumerate() {
            if node.status == NodeStatus::Active {
                if node.edges.is_empty() {
                    to_complete.push(node_id);
                    continue;
                }

                for edge in &node.edges {
                    if edge.gate.evaluate(&self.nodes, event) {
                        to_complete.push(node_id);
                        to_activate.push(edge.target_node);
                    }
                }
            }
        }

        // Second pass: update node statuses
        for node_id in to_complete {
            self.nodes[node_id].status = NodeStatus::Completed;
        }

        for node_id in to_activate {
            self.nodes[node_id.0].status = NodeStatus::Active;
            // If the activated node has no edges, complete it immediately
            if self.nodes[node_id.0].edges.is_empty() {
                self.nodes[node_id.0].status = NodeStatus::Completed;
            }
        }

        // Check if all nodes are completed
        if self.nodes.iter().all(|n| n.status == NodeStatus::Completed) {
            self.status = WorkflowStatus::Completed;
        }
    }
}
