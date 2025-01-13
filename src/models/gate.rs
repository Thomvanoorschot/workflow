use super::{
    event::Event,
    node::{Node, NodeId},
};
use serde::{Deserialize, Serialize};
use std::fmt;

pub trait Condition: fmt::Debug + Send + Sync {
    fn evaluate(&self, event: &Event) -> bool;
    fn box_clone(&self) -> Box<dyn Condition>;
    fn condition_type(&self) -> &'static str;
    fn condition_data(&self) -> Option<String>;
}

impl Clone for Box<dyn Condition> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

#[derive(Clone, Serialize)]
#[serde(tag = "type")]
pub enum Gate {
    Single {
        #[serde(rename = "condition_type")]
        condition_type_str: String,
        #[serde(rename = "condition_data", skip_serializing_if = "Option::is_none")]
        condition_data_str: Option<String>,
        #[serde(skip)]
        condition: Box<dyn Condition>,
    },
    And {
        gates: Vec<Gate>,
    },
    Or {
        gates: Vec<Gate>,
    },
    Not {
        gate: Box<Gate>,
    },
    WaitForNodes {
        nodes: Vec<NodeId>,
    },
}

impl<'de> Deserialize<'de> for Gate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(tag = "type")]
        enum GateHelper {
            Single {
                #[serde(rename = "condition_type")]
                condition_type_str: String,
                #[serde(rename = "condition_data")]
                condition_data_str: Option<String>,
            },
            And {
                gates: Vec<Gate>,
            },
            Or {
                gates: Vec<Gate>,
            },
            Not {
                gate: Box<Gate>,
            },
            WaitForNodes {
                nodes: Vec<NodeId>,
            },
        }

        let helper = GateHelper::deserialize(deserializer)?;
        match helper {
            GateHelper::Single {
                condition_type_str,
                condition_data_str,
            } => {
                // During deserialization, we'll set a placeholder condition
                // The actual condition should be reconstructed by the workflow engine
                Ok(Gate::Single {
                    condition_type_str,
                    condition_data_str,
                    condition: Box::new(PlaceholderCondition),
                })
            }
            GateHelper::And { gates } => Ok(Gate::And { gates }),
            GateHelper::Or { gates } => Ok(Gate::Or { gates }),
            GateHelper::Not { gate } => Ok(Gate::Not { gate }),
            GateHelper::WaitForNodes { nodes } => Ok(Gate::WaitForNodes { nodes }),
        }
    }
}

#[derive(Debug, Clone)]
struct PlaceholderCondition;

impl Condition for PlaceholderCondition {
    fn evaluate(&self, _event: &Event) -> bool {
        false // Placeholder conditions always return false
    }

    fn box_clone(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }

    fn condition_type(&self) -> &'static str {
        "placeholder"
    }

    fn condition_data(&self) -> Option<String> {
        None
    }
}

impl fmt::Debug for Gate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Gate::Single { condition, .. } => write!(f, "Gate::Single({:?})", condition),
            Gate::And { gates } => f.debug_tuple("Gate::And").field(gates).finish(),
            Gate::Or { gates } => f.debug_tuple("Gate::Or").field(gates).finish(),
            Gate::Not { gate } => f.debug_tuple("Gate::Not").field(gate).finish(),
            Gate::WaitForNodes { nodes } => {
                f.debug_tuple("Gate::WaitForNodes").field(nodes).finish()
            }
        }
    }
}

impl Gate {
    pub fn single(condition: Box<dyn Condition>) -> Self {
        Gate::Single {
            condition_type_str: condition.condition_type().to_string(),
            condition_data_str: condition.condition_data(),
            condition,
        }
    }

    pub fn evaluate(&self, nodes: &[Node], event: &Event) -> bool {
        match self {
            Gate::Single { condition, .. } => condition.evaluate(event),
            Gate::And { gates } => gates.iter().all(|g| g.evaluate(nodes, event)),
            Gate::Or { gates } => gates.iter().any(|g| g.evaluate(nodes, event)),
            Gate::Not { gate } => !gate.evaluate(nodes, event),
            Gate::WaitForNodes {
                nodes: required_node_ids,
            } => required_node_ids
                .iter()
                .all(|node_id| nodes[node_id.0].status == super::node::NodeStatus::Completed),
        }
    }
}
