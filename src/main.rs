// src/main.rs

mod workflow;

use std::sync::Arc;
use workflow::{Edge, Node, NodeId, Workflow, Gate, MyInput};
use crate::workflow::NodeMemory;

fn main() {
    cumulativeMain();
    // let node0 = Node {
    //     name: "Start".to_string(),
    //     edges: vec![],
    // };
    // let workflow = Workflow { nodes: vec![node0] };
    //
    // workflow.next_nodes()
}

fn cumulativeMain(){
    // 1) We’ll create just two nodes:
    //    Node0 is a "collector node" that waits until it has seen keywords "A", "B", and "C".
    //    Node1 is a "final node" with no edges.

    // Node0 edges: If we have cumulatively seen A, B, and C in Node0’s memory, go to Node1.
    // Example gate: Gate::CumulativeAnd(vec!["A".to_string(), "B".to_string(), "C".to_string()])
    let node0 = Node {
        name: "CollectorNode".to_string(),
        edges: vec![
            Edge {
                gate: Gate::CumulativeAnd(vec![
                    "A".to_string(),
                    "B".to_string(),
                    "C".to_string(),
                ]),
                target_node: NodeId(1), // Node1
            },
        ],
        memory: NodeMemory::default(),
    };

    // Node1: Final node with no edges
    let node1 = Node {
        name: "FinalNode".to_string(),
        edges: vec![],
        memory: NodeMemory::default(),
    };

    // 2) Create the workflow:
    let mut workflow = Workflow::new(vec![node0, node1]);

    // For demo, we consider Node0 to be our starting node:
    workflow.activate_node(NodeId(0));

    println!("Initial node statuses: {:?}", workflow.statuses);

    // 3) We’ll feed three separate inputs, each with a different keyword.
    //    The gate requires all three keywords ("A", "B", and "C") to have been seen
    //    across multiple calls to process() before transitioning to Node1.

    let inputs = vec![
        MyInput { value: 0, keyword: "A".to_string() },
        MyInput { value: 0, keyword: "B".to_string() },
        MyInput { value: 0, keyword: "C".to_string() },
    ];

    for (idx, input) in inputs.iter().enumerate() {
        println!(
            "\n-- Processing input #{} => {:?} --",
            idx + 1,
            input
        );
        let newly_activated = workflow.process(input);
        println!("  Newly activated nodes: {:?}", newly_activated);
        println!("  Current node statuses: {:?}", workflow.statuses);

        // Optional: Print out the memory we’ve collected so far in Node0
        let node0_mem = &workflow.nodes[0].memory;
        println!("  Node0 seen_keywords: {:?}", node0_mem.seen_keywords);
    }

    // After all three inputs (A, B, C), Node0’s gate should be satisfied,
    // so Node1 becomes active (and then completed on the same cycle).

    // Let's see final statuses:
    println!("\n== Final statuses ==\n{:?}", workflow.statuses);

    // You’d see something like:
    // Node0 => Completed
    // Node1 => Active (and possibly Completed if you process again)
    // That indicates the workflow reached the FinalNode.
}

fn exampleMain() {
    // -- Build conditions (closures) --
    let condition_is_positive = Arc::new(|inp: &MyInput| inp.value > 0);
    let condition_has_keyword_foo = Arc::new(|inp: &MyInput| inp.keyword == "foo");
    let condition_has_keyword_bar = Arc::new(|inp: &MyInput| inp.keyword == "bar");

    // -- Build gates --
    let gate_positive_and_foo = Gate::And(vec![
        Gate::Single(condition_is_positive.clone()),
        Gate::Single(condition_has_keyword_foo.clone()),
    ]);

    let gate_positive_or_bar = Gate::Or(vec![
        Gate::Single(condition_is_positive.clone()),
        Gate::Single(condition_has_keyword_bar.clone()),
    ]);

    // -- Define nodes with edges --
    let node0 = Node {
        name: "Start".to_string(),
        edges: vec![
            // If value>0 && keyword=="foo", go to Node #1
            Edge {
                gate: gate_positive_and_foo.clone(),
                target_node: NodeId(1),
            },
            // If value>0 || keyword=="bar", go to Node #2
            Edge {
                gate: gate_positive_or_bar.clone(),
                target_node: NodeId(2),
            },
        ],
        memory: NodeMemory::default(),
    };

    let node1 = Node {
        name: "NodeFoo".to_string(),
        edges: vec![
            // If NOT(keyword=="foo"), go to Node #2
            Edge {
                gate: Gate::Not(Box::new(Gate::Single(condition_has_keyword_foo.clone()))),
                target_node: NodeId(2),
            },
        ],
        memory: NodeMemory::default(),
    };

    let node2 = Node {
        name: "NodeBar".to_string(),
        edges: vec![],
        memory: NodeMemory::default(),
    };

    // -- Create the workflow --
    let mut workflow = Workflow::new(vec![node0, node1, node2]);

    // -- At the start, let's say we don't know (or care) which node is start.
    //    We'll "activate_node(0)" to begin from node0, but we *could* also
    //    activate multiple. If you want them all active, just loop over them.
    workflow.activate_node(NodeId(0));

    // Display initial status
    println!("Initial statuses: {:?}", workflow.statuses);

    // -- Provide an input --
    let input = MyInput {
        value: 10,
        keyword: "foo".to_string(),
    };

    // 1) Process step
    println!("Processing with input: {:?}", input);
    let newly_activated = workflow.process(&input);
    println!("  Newly activated nodes: {:?}", newly_activated);
    println!("  Current statuses: {:?}", workflow.statuses);

    // Possibly process again with the *same* or *new* input
    // to see if more nodes get activated.  For a single-step example, let's
    // do it again with a changed input:
    let second_input = MyInput {
        value: 10,
        keyword: "not_foo".to_string(),
    };

    println!("\nProcessing again with input: {:?}", second_input);
    let newly_activated = workflow.process(&second_input);
    println!("  Newly activated nodes: {:?}", newly_activated);
    println!("  Current statuses: {:?}", workflow.statuses);
}
