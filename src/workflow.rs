use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// Represents a transition (edge) from one node to another, guarded by a Gate.
#[derive(Clone)]
pub struct Edge {
    pub gate: Gate,
    pub target_node: NodeId,
}

/// In addition to edges guarded by a Gate,
/// each node can have a set of prerequisite NodeIds:
#[derive(Clone)]
pub struct Node {
    pub name: String,

    /// For dynamic event-based transitions
    pub edges: Vec<Edge>,

    pub status: NodeStatus,
}

/// Status of each node: has it been started, is it active, or completed?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    NotStarted,
    Active,
    Completed,
}

/// The entire workflow, storing:
/// - All nodes
/// - A status vector that parallels the nodes
pub struct Workflow {
    pub nodes: Vec<Node>,
}

impl Workflow {
    /// Create a new workflow with all nodes in `NotStarted` status initially.
    pub fn new(nodes: Vec<Node>) -> Self {
        Workflow { nodes }
    }

    /// Retrieve a node by ID (if it exists).
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.0)
    }

    /// Mark a node as active.
    /// (e.g., for the "start" node(s), or re-activation if desired.)
    pub fn activate_node(&mut self, id: NodeId) {
        self.nodes[id.0].status = NodeStatus::Active;
    }

    pub fn process_event(&mut self, event: &Event) {
        // 1) Find all active nodes
        let active_node_ids: Vec<usize> = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.status == NodeStatus::Active)
            .map(|(i, _)| i)
            .collect();

        // We'll collect nodes that should become "Completed" here
        let mut to_complete = Vec::new();

        // 2) For each active node, handle event
        for node_id in active_node_ids {
            // Instead of holding a &mut to self.nodes[node_id],
            // we just grab data we need and store it in a local variable.
            let edges = self.nodes[node_id].edges.clone();

            // 3) Evaluate edges using an *immutable* borrow of `self.nodes`
            for edge in edges {
                if edge.gate.evaluate(&self.nodes, event) {
                    // We'll mark this node Completed, but only after the loop.
                    to_complete.push(node_id);
                    self.activate_node(edge.target_node);
                }
            }
        }

        // 4) Now apply the changes after we're done iterating
        for node_id in to_complete {
            self.nodes[node_id].status = NodeStatus::Completed;
        }
    }
}

/// A general event with a name and some payload data.
/// The payload here is a very rough example; you could store JSON,
/// typed structs, or anything you like.
#[derive(Debug, Clone)]
pub struct Event {
    pub name: String,
    pub payload: HashMap<String, String>,
}

/// A trait describing *some* operation the node can perform upon receiving an event.
/// Returns a `bool` indicating whether the node “succeeded” (or “unlocked”).
/// In reality, you might return a `Result<bool, Error>`, or run async/await for HTTP calls, etc.
pub trait EventHandler: Send + Sync {
    fn handle(&self, event: &Event) -> bool;
}

// pub type Condition = Arc<dyn Fn(&Event) -> bool + Send + Sync>;
pub type Condition = Arc<dyn Fn(&Event) -> bool + Send + Sync>;

#[derive(Clone)]
pub enum Gate {
    Single(Condition),
    And(Vec<Gate>),
    Or(Vec<Gate>),
    Not(Box<Gate>),
    WaitForNodes(Vec<NodeId>),
}

impl Gate {
    /// Evaluate the gate against the given event and current workflow statuses.
    pub fn evaluate(&self, nodes: &Vec<Node>, event: &Event) -> bool {
        match self {
            Gate::Single(condition) => condition(event),
            Gate::And(gates) => gates.iter().all(|g| g.evaluate(nodes, event)),
            Gate::Or(gates) => gates.iter().any(|g| g.evaluate(nodes, event)),
            Gate::Not(sub_gate) => !sub_gate.evaluate(nodes, event),
            Gate::WaitForNodes(required_node_ids) => {
                // Instead of checking node_mem, we check if
                // all listed node IDs are completed in `statuses`.
                required_node_ids
                    .iter()
                    .all(|node_id| nodes[node_id.0].status == NodeStatus::Completed)
            }
        }
    }
}
