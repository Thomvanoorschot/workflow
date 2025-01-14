use super::edge::Edge;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[typetag::serde(tag = "type")]
pub trait NodeBehavior: Send + Sync + Debug {
    fn on_activated(&self) -> Option<String>;
    fn on_completed(&self);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub name: String,
    pub status: NodeStatus,
    pub edges: Vec<Edge>,
    pub behavior: Box<dyn NodeBehavior>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NodeStatus {
    NotStarted,
    Active,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NodeId(pub usize);
