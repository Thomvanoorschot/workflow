use super::{gate::Gate, node::NodeId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Edge {
    pub target: NodeId,
    pub gate: Gate,
}
