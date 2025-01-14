use crate::models::{
    edge::Edge,
    event::Event,
    gate::{Condition, Gate},
    node::{Node, NodeBehavior, NodeId, NodeStatus},
    workflow::Workflow,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserActivityCondition;

#[typetag::serde]
impl Condition for UserActivityCondition {
    fn evaluate(&self, event: &Event) -> bool {
        matches!(event, Event::UserActivity)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimerCondition {
    timer_id: String,
}

impl TimerCondition {
    pub fn new(timer_id: String) -> Self {
        Self { timer_id }
    }
}

#[typetag::serde]
impl Condition for TimerCondition {
    fn evaluate(&self, event: &Event) -> bool {
        matches!(event, Event::Timer { timer_id } if *timer_id == self.timer_id)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmptyBehavior;

#[typetag::serde]
impl NodeBehavior for EmptyBehavior {
    fn on_activated(&self) -> Option<String> {
        None
    }

    fn on_completed(&self) {}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimerNodeBehavior;

#[typetag::serde]
impl NodeBehavior for TimerNodeBehavior {
    fn on_activated(&self) -> Option<String> {
        Some("1".to_string())
    }

    fn on_completed(&self) {}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinishNodeBehavior;

#[typetag::serde]
impl NodeBehavior for FinishNodeBehavior {
    fn on_activated(&self) -> Option<String> {
        None
    }

    fn on_completed(&self) {
        println!("FINISHED");
    }
}

pub fn create_demo_workflow_with_ids(
    user_activity_node_id: NodeId,
    timer_node_id: NodeId,
    finish_node_id: NodeId,
) -> Workflow {
    let nodes = vec![
        Node {
            id: user_activity_node_id,
            name: "User Activity".to_string(),
            status: NodeStatus::Active,
            edges: vec![Edge {
                target: timer_node_id,
                gate: Gate::Single(Box::new(UserActivityCondition)),
            }],
            behavior: Box::new(EmptyBehavior),
        },
        Node {
            id: timer_node_id,
            name: "Timer".to_string(),
            status: NodeStatus::NotStarted,
            edges: vec![Edge {
                target: finish_node_id,
                gate: Gate::Single(Box::new(TimerCondition::new("1".to_string()))),
            }],
            behavior: Box::new(TimerNodeBehavior),
        },
        Node {
            id: finish_node_id,
            name: "Finish".to_string(),
            status: NodeStatus::NotStarted,
            edges: vec![],
            behavior: Box::new(FinishNodeBehavior),
        },
    ];

    Workflow::new(nodes)
}

pub fn create_demo_workflow() -> Workflow {
    create_demo_workflow_with_ids(NodeId(0), NodeId(1), NodeId(2))
}
