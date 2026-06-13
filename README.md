# DAG Runner

**A topological execution engine for directed acyclic graphs (DAGs)** that runs nodes in dependency order — guaranteeing that every node executes only after all its dependencies have completed. Built on Kahn's algorithm for topological sorting.

## Why It Matters

Many real-world workflows are naturally expressed as DAGs: CI/CD pipelines (build → test → deploy), data processing (extract → transform → load), machine learning pipelines (collect data → train → evaluate → deploy), and build systems (Make, Bazel, Ninja all use DAG resolution internally).

The critical requirement is **correct ordering**: a node must not execute until all its dependencies are satisfied. DAG Runner solves this by computing a topological order — a linear sequence of nodes where every dependency appears before its dependent.

**Cycle detection:** The runner also validates that the graph is acyclic. If a cycle exists (e.g., A depends on B depends on A), it returns an error rather than deadlocking. This is essential for user-defined workflows where accidental cycles are common.

**Algorithm choice:** Kahn's algorithm (BFS-based topological sort) is used because it's intuitive, detects cycles naturally (if the output has fewer nodes than the input, a cycle exists), and processes nodes in breadth-first order — which tends to maximize parallelism opportunities in practice.

## How It Works

The implementation uses **Kahn's algorithm** for topological sorting:

1. **Build in-degree map and adjacency list:** For each node, count how many dependencies it has (`in_degree`). Build an adjacency list where each node points to the nodes that depend on it.

2. **Initialize queue with roots:** Nodes with `in_degree = 0` (no dependencies) are added to a `VecDeque`. These are the starting points — they can execute immediately.

3. **Process in BFS order:** Pop a node from the queue, add it to the output order, then decrement the in-degree of each node that depends on it. If any node's in-degree reaches zero, it's enqueued.

4. **Cycle detection:** After processing, if the output order has fewer nodes than the graph, the remaining nodes form a cycle. The function returns `Err("cycle detected in DAG")`.

**Execution:** The `run` method takes the topological order and applies an executor closure to each node in sequence. The closure returns `bool` (success/failure), producing a results vector of `(node_id, success)` pairs.

**Time complexity:**
- Topological sort: O(V + E) where V = vertices, E = edges
- Full run: O(V + E) for ordering + O(V) for execution
- Space: O(V + E)

**Error handling:** Missing dependencies (referenced dep ID not in the graph) are detected before topological sorting and return a descriptive error message.

## Quick Start

```rust
use dag_runner::{DagRunner, Node};

let mut runner = DagRunner::new();

// Define a CI/CD pipeline as a DAG
runner.add_node(Node { id: "checkout".into(), data: "git clone", deps: vec![] });
runner.add_node(Node { id: "build".into(), data: "cargo build", deps: vec!["checkout"] });
runner.add_node(Node { id: "test".into(), data: "cargo test", deps: vec!["build"] });
runner.add_node(Node { id: "lint".into(), data: "cargo clippy", deps: vec!["build"] });
runner.add_node(Node { id: "deploy".into(), data: "deploy.sh", deps: vec!["test", "lint"] });

// Get the execution order
let order = runner.topological_order().unwrap();
// ["checkout", "build", "lint", "test", "deploy"] (or similar valid order)

// Execute each node
let results = runner.run(|node| {
    println!("Running {}: {}", node.id, node.data);
    true // return false to mark failure
}).unwrap();

for (id, success) in &results {
    println!("{}: {}", id, if *success { "✓" } else { "✗" });
}
```

## API

### `Node<T: Clone>`
- `id: String` — Unique identifier
- `data: T` — Payload (e.g., command to execute)
- `deps: Vec<String>` — IDs of nodes this depends on

### `DagRunner<T: Clone>`
- `new() -> Self` — Create empty DAG
- `add_node(&mut self, node: Node<T>) -> &mut Self` — Register a node. O(1)
- `topological_order(&self) -> Result<Vec<String>, String>` — Compute execution order via Kahn's algorithm. O(V + E). Returns error on cycle or missing dependency
- `run<F>(&self, executor: F) -> Result<Vec<(String, bool)>, String>` where `F: FnMut(&Node<T>) -> bool` — Execute nodes in topological order. O(V + E)

## Architecture Notes

DAG Runner is a core component of SuperInstance's workflow orchestration layer, used for pipeline execution (CI/CD, data transforms, ML training) and task dependency resolution. It integrates with the TaskFlow system for durable, resumable pipeline execution.

See the full architecture: [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md)

## License

MIT
