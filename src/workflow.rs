use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// Represents a transition (edge) from one node to another, guarded by a Gate.
#[derive(Clone)]
pub struct Edge {
    pub gate: Gate,
    pub target_node: NodeId,
}

/// A single node in the workflow.
pub struct Node {
    pub name: String,
    pub edges: Vec<Edge>,
    pub memory: NodeMemory,
}

/// Memory that each node keeps about inputs it has seen, etc.
#[derive(Default)]
pub struct NodeMemory {
    /// Example: store all keywords seen so far
    pub seen_keywords: Vec<String>,
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
    pub statuses: Vec<NodeStatus>,
}

impl Workflow {
    /// Create a new workflow with all nodes in `NotStarted` status initially.
    pub fn new(nodes: Vec<Node>) -> Self {
        let statuses = vec![NodeStatus::NotStarted; nodes.len()];
        Workflow { nodes, statuses }
    }

    /// Retrieve a node by ID (if it exists).
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.0)
    }

    /// Mark a node as active.
    /// (e.g., for the "start" node(s), or re-activation if desired.)
    pub fn activate_node(&mut self, id: NodeId) {
        if id.0 < self.statuses.len() {
            self.statuses[id.0] = NodeStatus::Active;
        }
    }

    /// "Process" the workflow with the given input:
    /// 1) Find all nodes that are currently 'Active'.
    /// 2) For each active node, update its memory with the new input.
    /// 3) Evaluate edges. Any edge whose gate passes leads to a 'target_node' which we can activate.
    /// 4) Mark the current node as `Completed`.
    pub fn process(&mut self, input: &MyInput) -> Vec<NodeId> {
        let mut newly_activated = Vec::new();

        let active_nodes: Vec<usize> = self
            .statuses
            .iter()
            .enumerate()
            .filter_map(|(i, status)| {
                if *status == NodeStatus::Active {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        for i in active_nodes {
            // 2) Update memory with the new input
            let node = &mut self.nodes[i];
            node.memory.seen_keywords.push(input.keyword.clone());

            // 3) Evaluate edges
            //    We pass both the current input *and* the node's memory.
            for edge in &node.edges {
                let gate_ok = edge.gate.evaluate(input, &node.memory);
                if gate_ok {
                    let target_i = edge.target_node.0;
                    if self.statuses[target_i] == NodeStatus::NotStarted {
                        self.statuses[target_i] = NodeStatus::Active;
                        newly_activated.push(edge.target_node);
                        self.statuses[i] = NodeStatus::Completed;
                    }
                }
            }
            // 4) Mark this node completed
        }

        newly_activated
    }
}

#[derive(Debug)]
pub struct MyInput {
    pub value: i32,
    pub keyword: String,
}

/// Use Arc so the closure is reference-counted.
pub type Condition = Arc<dyn Fn(&MyInput) -> bool + Send + Sync>;

#[derive(Clone)]
pub enum Gate {
    Single(Condition),
    And(Vec<Gate>),
    Or(Vec<Gate>),
    Not(Box<Gate>),
    CumulativeAnd(Vec<String>),
}

impl Gate {
    /// Evaluate the gate against the input.
    pub fn evaluate(&self, input: &MyInput, node_memory: &NodeMemory) -> bool {
        match self {
            Gate::Single(condition) => (condition)(input),
            Gate::And(gates) => gates.iter().all(|g| g.evaluate(input, node_memory)),
            Gate::Or(gates) => gates.iter().any(|g| g.evaluate(input, node_memory)),
            Gate::Not(gate) => !gate.evaluate(input, node_memory),

            // Our new variant
            Gate::CumulativeAnd(required_keywords) => {
                // If node_memory.seen_keywords contains *all* required_keywords
                required_keywords
                    .iter()
                    .all(|req| node_memory.seen_keywords.contains(req))
            }
        }
    }
}
