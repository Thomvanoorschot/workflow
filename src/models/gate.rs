use super::{
    node::{Node, NodeId, NodeStatus},
    Event,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[typetag::serde(tag = "type")]
pub trait Condition: Send + Sync + Debug {
    fn evaluate(&self, event: &Event) -> bool;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Gate {
    Single(Box<dyn Condition>),
    And(Vec<Gate>),
    Or(Vec<Gate>),
    Not(Box<Gate>),
    WaitForNodes(Vec<NodeId>),
}

impl Gate {
    pub fn evaluate(&self, nodes: &[Node], event: &Event) -> bool {
        match self {
            Gate::Single(condition) => condition.evaluate(event),
            Gate::And(gates) => gates.iter().all(|g| g.evaluate(nodes, event)),
            Gate::Or(gates) => gates.iter().any(|g| g.evaluate(nodes, event)),
            Gate::Not(gate) => !gate.evaluate(nodes, event),
            Gate::WaitForNodes(required_node_ids) => required_node_ids
                .iter()
                .all(|node_id| nodes[node_id.0].status == NodeStatus::Completed),
        }
    }
}
