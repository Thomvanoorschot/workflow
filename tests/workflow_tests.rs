use ariadne::{models::{
    edge::Edge,
    event::Event,
    gate::{Condition, Gate},
    node::{Node, NodeBehavior, NodeId, NodeStatus},
    workflow::{Workflow, WorkflowStatus},
}, workflow::user_activity_workflow::UserActivityCondition};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct TestCondition(bool);

#[typetag::serde]
impl Condition for TestCondition {
    fn evaluate(&self, _event: &Event) -> bool {
        self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TestBehavior {
    activated_count: std::sync::atomic::AtomicUsize,
    completed_count: std::sync::atomic::AtomicUsize,
}

impl Clone for TestBehavior {
    fn clone(&self) -> Self {
        Self {
            activated_count: std::sync::atomic::AtomicUsize::new(
                self.activated_count
                    .load(std::sync::atomic::Ordering::SeqCst),
            ),
            completed_count: std::sync::atomic::AtomicUsize::new(
                self.completed_count
                    .load(std::sync::atomic::Ordering::SeqCst),
            ),
        }
    }
}

#[typetag::serde]
impl NodeBehavior for TestBehavior {
    fn on_activated(&self) -> Option<String> {
        self.activated_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        None
    }

    fn on_completed(&self) {
        self.completed_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}
#[test]
fn test_workflow_activation() {
    let behavior1 = TestBehavior {
        activated_count: std::sync::atomic::AtomicUsize::new(0),
        completed_count: std::sync::atomic::AtomicUsize::new(0),
    };
    let behavior2 = behavior1.clone();

    let nodes = vec![
        Node {
            id: NodeId(0),
            name: "Start".to_string(),
            status: NodeStatus::Active,
            edges: vec![Edge {
                target: NodeId(1),
                gate: Gate::Single(Box::new(UserActivityCondition)),
            }],
            behavior: Box::new(behavior1),
        },
        Node {
            id: NodeId(1),
            name: "End".to_string(),
            status: NodeStatus::NotStarted,
            edges: vec![],
            behavior: Box::new(behavior2),
        },
    ];

    let mut workflow = Workflow::new(nodes);
    workflow.process_event(&Event::UserActivity);

    assert_eq!(workflow.nodes[0].status, NodeStatus::Completed);
    assert_eq!(workflow.nodes[1].status, NodeStatus::Completed);
    assert_eq!(workflow.status, WorkflowStatus::Completed);
}

#[test]
fn test_workflow_blocked_activation() {
    let behavior1 = TestBehavior {
        activated_count: std::sync::atomic::AtomicUsize::new(0),
        completed_count: std::sync::atomic::AtomicUsize::new(0),
    };
    let behavior2 = behavior1.clone();

    let nodes = vec![
        Node {
            id: NodeId(0),
            name: "Start".to_string(),
            status: NodeStatus::Active,
            edges: vec![Edge {
                target: NodeId(1),
                gate: Gate::Single(Box::new(TestCondition(false))),
            }],
            behavior: Box::new(behavior1),
        },
        Node {
            id: NodeId(1),
            name: "End".to_string(),
            status: NodeStatus::NotStarted,
            edges: vec![],
            behavior: Box::new(behavior2),
        },
    ];

    let mut workflow = Workflow::new(nodes);
    workflow.process_event(&Event::UserActivity);

    assert_eq!(workflow.nodes[0].status, NodeStatus::Active);
    assert_eq!(workflow.nodes[1].status, NodeStatus::NotStarted);
    assert_eq!(workflow.status, WorkflowStatus::Active);
}

#[test]
fn test_workflow_serialization() {
    let behavior1 = TestBehavior {
        activated_count: std::sync::atomic::AtomicUsize::new(0),
        completed_count: std::sync::atomic::AtomicUsize::new(0),
    };
    let behavior2 = behavior1.clone();

    let nodes = vec![
        Node {
            id: NodeId(0),
            name: "Start".to_string(),
            status: NodeStatus::Active,
            edges: vec![Edge {
                target: NodeId(1),
                gate: Gate::Single(Box::new(TestCondition(true))),
            }],
            behavior: Box::new(behavior1),
        },
        Node {
            id: NodeId(1),
            name: "End".to_string(),
            status: NodeStatus::NotStarted,
            edges: vec![],
            behavior: Box::new(behavior2),
        },
    ];

    let workflow = Workflow::new(nodes);
    let bytes = workflow.to_bytes().unwrap();
    let deserialized = Workflow::from_bytes(&bytes).unwrap();

    assert_eq!(workflow.id, deserialized.id);
    assert_eq!(workflow.user_id, deserialized.user_id);
    assert_eq!(workflow.status, deserialized.status);
    assert_eq!(workflow.nodes.len(), deserialized.nodes.len());
}
