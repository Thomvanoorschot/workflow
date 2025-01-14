use super::{Event, Node, NodeStatus};
use bincode::{DefaultOptions, Options};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub nodes: Vec<Node>,
    pub status: WorkflowStatus,
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
        // Start with all active nodes
        let active_nodes: Vec<_> = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.status == NodeStatus::Active)
            .map(|(i, _)| i)
            .collect();

        // Process each active node recursively
        for node_idx in active_nodes {
            self.process_node(node_idx, event);
        }

        // Check if workflow is completed
        if self
            .nodes
            .iter()
            .all(|node| node.status == NodeStatus::Completed)
        {
            println!("Workflow completed!");
            self.status = WorkflowStatus::Completed;
        }
    }

    fn process_node(&mut self, node_idx: usize, event: &Event) {
        // Complete nodes with no edges
        if self.nodes[node_idx].edges.is_empty() {
            self.nodes[node_idx].status = NodeStatus::Completed;
            self.nodes[node_idx].behavior.on_completed();
            return;
        }

        let mut all_edges_activated = true;
        let mut any_edge_activated = false;

        // Collect target indices first to avoid borrow issues
        let mut targets = Vec::new();
        for edge in &self.nodes[node_idx].edges {
            if edge.gate.evaluate(&self.nodes, event) {
                if let Some(target_idx) = self.nodes.iter().position(|n| n.id == edge.target) {
                    if self.nodes[target_idx].status == NodeStatus::NotStarted {
                        targets.push((target_idx, edge.target.0));
                    }
                }
            } else {
                all_edges_activated = false;
            }
        }

        // Process collected targets
        for (target_idx, target_id) in targets {
            self.nodes[target_idx].status = NodeStatus::Active;
            if let Some(timer_id) = self.nodes[target_idx].behavior.on_activated() {
                println!("Timer node {} activated with ID {}", target_id, timer_id);
            }
            self.process_node(target_idx, event);
            any_edge_activated = true;
        }

        // Complete node if all edges activated and at least one was activated
        if all_edges_activated && any_edge_activated {
            self.nodes[node_idx].status = NodeStatus::Completed;
            self.nodes[node_idx].behavior.on_completed();
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        let config = DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes()
            .with_native_endian()
            .with_no_limit();
        config.serialize(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        let config = DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes()
            .with_native_endian()
            .with_no_limit();
        config.deserialize(bytes)
    }
}
