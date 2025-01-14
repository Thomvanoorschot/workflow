use ariadne::models::{
    edge::Edge,
    event::Event,
    gate::{Condition, Gate},
    node::{Node, NodeBehavior, NodeId, NodeStatus},
    workflow::Workflow,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct BenchCondition(bool);

#[typetag::serde]
impl Condition for BenchCondition {
    fn evaluate(&self, _event: &Event) -> bool {
        self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchBehavior;

#[typetag::serde]
impl NodeBehavior for BenchBehavior {
    fn on_activated(&self) -> Option<String> {
        None
    }

    fn on_completed(&self) {}
}

fn create_linear_workflow(size: usize) -> Workflow {
    let mut nodes = Vec::with_capacity(size);

    for i in 0..size {
        let edges = if i < size - 1 {
            vec![Edge {
                target: NodeId(i + 1),
                gate: Gate::Single(Box::new(BenchCondition(true))),
            }]
        } else {
            vec![]
        };

        nodes.push(Node {
            id: NodeId(i),
            name: format!("Node {}", i),
            status: if i == 0 {
                NodeStatus::Active
            } else {
                NodeStatus::NotStarted
            },
            edges,
            behavior: Box::new(BenchBehavior),
        });
    }

    Workflow::new(nodes)
}

fn bench_workflow_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflow_processing");

    for size in [10, 100, 1000].iter() {
        group.bench_function(format!("linear_workflow_{}", size), |b| {
            b.iter(|| {
                let mut workflow = create_linear_workflow(*size);
                workflow.process_event(black_box(&Event::UserActivity));
            });
        });
    }

    group.finish();
}

fn bench_workflow_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflow_serialization");

    for size in [10, 100, 1000].iter() {
        let workflow = create_linear_workflow(*size);

        group.bench_function(format!("serialize_{}", size), |b| {
            b.iter(|| {
                black_box(workflow.to_bytes().unwrap());
            });
        });

        let bytes = workflow.to_bytes().unwrap();
        group.bench_function(format!("deserialize_{}", size), |b| {
            b.iter(|| {
                black_box(Workflow::from_bytes(black_box(&bytes)).unwrap());
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_workflow_processing,
    bench_workflow_serialization
);
criterion_main!(benches);
