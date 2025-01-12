use std::collections::HashMap;
use std::sync::Arc;

mod workflow;

use workflow::{Edge, Event, Gate, Node, EventHandler, NodeId, NodeStatus, Workflow};

struct MatchesEvent {
    pub event_name: String,
}

impl EventHandler for MatchesEvent {
    fn handle(&self, event: &Event) -> bool {
       event.name == self.event_name
    }
}

fn main() {
    // Node0's gate is a Single gate with the HTTP action
    // let single_gate = Gate::Single(http_action);

    // 2) Node1’s gate is WaitForNodes, referencing Node0 (which is ID=0 in our array)
    let wait_gate = Gate::WaitForNodes(vec![NodeId(0)]);

    // 3) Build nodes:
    // Node0: does an HTTP check. If success, we attempt to transition to Node1.
    let node0 = Node {
        name: "UserActivity".to_string(),
        status: NodeStatus::NotStarted,
        edges: vec![
            // If value>0 && keyword=="foo", go to Node #1
            Edge {
                gate: Gate::Single(Arc::new(|event: &Event| {event.name == "UserActivity"})),
                target_node: NodeId(1),
            },
        ],
    };
    let node1 = Node {
        name: "Timer".to_string(),
        status: NodeStatus::NotStarted,
        edges: vec![
            // If value>0 && keyword=="foo", go to Node #1
            Edge {
                gate: Gate::Single(Arc::new(|inp: &Event| true)),
                target_node: NodeId(2),
            },
        ],
    };
    let node2 = Node {
        name: "API Call".to_string(),
        edges: vec![], // no outgoing edges
        status: NodeStatus::NotStarted,
    };


    // Node1: no edges for this example, or you could add more if you like
    let node10 = Node {
        name: "WaitNode".to_string(),
        edges: vec![], // no outgoing edges
        status: NodeStatus::NotStarted,
    };

    // 4) Build the workflow
    let mut workflow = Workflow::new(vec![node0, node1, node2]);

    // Mark Node0 as active to start
    workflow.activate_node(NodeId(0));

    // 5) Create an event that might pass the HTTP check
    let mut payload = HashMap::new();
    payload.insert("id".to_string(), "12345".to_string());

    let event = Event {
        name: "UserActivity".to_string(),
        payload,
    };

    println!("== Processing first event ==");
    workflow.process_event(&event);
    println!(
        "Node statuses after first event: {:?}",
        workflow.nodes.iter().map(|n|  n.status).collect::<Vec<_>>()
    );

    // At this point, Node0 is Completed. Node1 will become Active ONLY if
    // its WaitForNodes(0) is satisfied. Because we tried to evaluate wait_gate
    // from Node0 -> Node1, it checks if Node0 is Completed.
    // If it was completed during the same pass, Node1 becomes Active.

    // If you want Node1 to get processed further, you can send a second event:
    let second_event = Event {
        name: "SecondEvent".to_string(),
        payload: HashMap::new(),
    };

    println!("\n== Processing second event ==");
    workflow.process_event(&second_event);
    println!(
        "Node statuses after second event: {:?}",
        workflow.nodes.iter().map(|n| n.status).collect::<Vec<_>>()
    );
}
