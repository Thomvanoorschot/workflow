use super::{gate::Gate, node::NodeId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge {
    pub source_node: NodeId,
    pub target_node: NodeId,
    pub gate: Gate,
}
