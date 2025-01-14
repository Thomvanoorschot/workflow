use crate::models::{
    edge::Edge,
    event::Event,
    gate::{Condition, Gate},
    node::{Node, NodeId, NodeStatus},
    workflow::{Workflow, WorkflowStatus},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
struct UserActivityCondition;

impl Condition for UserActivityCondition {
    fn evaluate(&self, event: &Event) -> bool {
        matches!(event, Event::UserActivity)
    }

    fn box_clone(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
struct TimerCondition(String);

impl Condition for TimerCondition {
    fn evaluate(&self, event: &Event) -> bool {
        matches!(event, Event::Timer { timer_id } if timer_id == &self.0)
    }

    fn box_clone(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }
}

pub fn create_demo_workflow() -> Workflow {
    let id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    create_demo_workflow_with_ids(id, user_id)
}

pub fn create_demo_workflow_with_ids(id: Uuid, user_id: Uuid) -> Workflow {
    let nodes = vec![
        Node {
            name: "Start".to_string(),
            status: NodeStatus::Active,
            edges: vec![Edge {
                target: NodeId(1),
                gate: Gate::single(Box::new(UserActivityCondition)),
            }],
        },
        Node {
            name: "User Activity".to_string(),
            status: NodeStatus::NotStarted,
            edges: vec![Edge {
                target: NodeId(2),
                gate: Gate::single(Box::new(TimerCondition("1".to_string()))),
            }],
        },
        Node {
            name: "Timer".to_string(),
            status: NodeStatus::NotStarted,
            edges: vec![],
        },
    ];

    Workflow {
        id,
        user_id,
        name: "Demo Workflow".to_string(),
        nodes,
        status: WorkflowStatus::Active,
    }
}
