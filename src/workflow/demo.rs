use crate::models::workflow::{Workflow, WorkflowStatus};
use crate::models::{Edge, Gate, Node, NodeId, NodeStatus};
use uuid::Uuid;

pub fn create_demo_workflow() -> Workflow {
    let id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    create_demo_workflow_with_ids(id, user_id)
}

pub fn create_demo_workflow_with_ids(id: Uuid, user_id: Uuid) -> Workflow {
    // Create nodes with their edges
    let nodes = vec![
        Node {
            name: "Start".to_string(),
            status: NodeStatus::Active,
            edges: vec![Edge {
                source_node: NodeId(0),
                target_node: NodeId(1),
                gate: Gate::new_user_activity(),
            }],
        },
        Node {
            name: "User Activity".to_string(),
            status: NodeStatus::NotStarted,
            edges: vec![Edge {
                source_node: NodeId(1),
                target_node: NodeId(2),
                gate: Gate::new_timer("1".to_string()),
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
