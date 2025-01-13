use super::{
    event::Event,
    node::{Node, NodeId},
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConditionType {
    UserActivity,
    Timer(String), // Store timer ID
    Custom(String),
}

pub type Condition = Arc<dyn Fn(&Event) -> bool + Send + Sync>;

#[derive(Clone, Serialize, Deserialize)]
pub struct Gate {
    #[serde(skip)]
    condition: Option<Condition>,
    condition_type: ConditionType,
    gate_type: GateType,
}

impl fmt::Debug for Gate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Gate")
            .field("condition_type", &self.condition_type)
            .field("gate_type", &self.gate_type)
            .finish()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GateType {
    Single,
    And(Vec<Gate>),
    Or(Vec<Gate>),
    Not(Box<Gate>),
    WaitForNodes(Vec<NodeId>),
}

impl Gate {
    pub fn evaluate(&self, nodes: &[Node], event: &Event) -> bool {
        match &self.gate_type {
            GateType::Single => {
                if let Some(condition) = &self.condition {
                    condition(event)
                } else {
                    // If condition is not set, reconstruct it
                    let condition = self.create_condition();
                    condition(event)
                }
            }
            GateType::And(gates) => gates.iter().all(|g| g.evaluate(nodes, event)),
            GateType::Or(gates) => gates.iter().any(|g| g.evaluate(nodes, event)),
            GateType::Not(sub_gate) => !sub_gate.evaluate(nodes, event),
            GateType::WaitForNodes(required_node_ids) => required_node_ids
                .iter()
                .all(|node_id| nodes[node_id.0].status == super::node::NodeStatus::Completed),
        }
    }

    fn create_condition(&self) -> Condition {
        match &self.condition_type {
            ConditionType::UserActivity => {
                Arc::new(|event: &Event| matches!(event, Event::UserActivity))
            }
            ConditionType::Timer(timer_id) => {
                let timer_id = timer_id.clone();
                Arc::new(move |event: &Event| {
                    if let Event::Timer {
                        timer_id: event_timer_id,
                    } = event
                    {
                        &timer_id == event_timer_id
                    } else {
                        false
                    }
                })
            }
            ConditionType::Custom(_) => Arc::new(|_| false), // Default for custom conditions
        }
    }

    pub fn new_user_activity() -> Self {
        Self {
            condition: None,
            condition_type: ConditionType::UserActivity,
            gate_type: GateType::Single,
        }
    }

    pub fn new_timer(timer_id: String) -> Self {
        Self {
            condition: None,
            condition_type: ConditionType::Timer(timer_id),
            gate_type: GateType::Single,
        }
    }

    pub fn new_and(gates: Vec<Gate>) -> Self {
        Self {
            condition: None,
            condition_type: ConditionType::Custom("and".to_string()),
            gate_type: GateType::And(gates),
        }
    }

    pub fn new_or(gates: Vec<Gate>) -> Self {
        Self {
            condition: None,
            condition_type: ConditionType::Custom("or".to_string()),
            gate_type: GateType::Or(gates),
        }
    }

    pub fn new_not(gate: Gate) -> Self {
        Self {
            condition: None,
            condition_type: ConditionType::Custom("not".to_string()),
            gate_type: GateType::Not(Box::new(gate)),
        }
    }

    pub fn new_wait_for_nodes(nodes: Vec<NodeId>) -> Self {
        Self {
            condition: None,
            condition_type: ConditionType::Custom("wait_for_nodes".to_string()),
            gate_type: GateType::WaitForNodes(nodes),
        }
    }
}
