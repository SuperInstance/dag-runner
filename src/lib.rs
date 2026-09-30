use std::collections::{HashMap, VecDeque};

/// A node in a directed acyclic graph.
#[derive(Debug, Clone)]
pub struct Node<T: Clone> {
    pub id: String,
    pub data: T,
    pub deps: Vec<String>,
}

/// Topological DAG runner that executes nodes in dependency order.
pub struct DagRunner<T: Clone> {
    nodes: HashMap<String, Node<T>>,
}

impl<T: Clone> DagRunner<T> {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node<T>) -> &mut Self {
        self.nodes.insert(node.id.clone(), node);
        self
    }

    /// Returns node ids in topological order. Panics on cycles.
    pub fn topological_order(&self) -> Result<Vec<String>, String> {
        let mut in_degree: HashMap<&String, usize> = HashMap::new();
        let mut adj: HashMap<&String, Vec<&String>> = HashMap::new();

        for id in self.nodes.keys() {
            in_degree.insert(id, 0);
            adj.insert(id, vec![]);
        }

        for node in self.nodes.values() {
            for dep in &node.deps {
                if !self.nodes.contains_key(dep) {
                    return Err(format!("missing dependency: {}", dep));
                }
                adj.get_mut(dep).unwrap().push(&node.id);
                *in_degree.get_mut(&node.id).unwrap() += 1;
            }
        }

        let mut queue: VecDeque<&String> = in_degree
            .iter()
            .filter(|&(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut order = Vec::new();
        while let Some(id) = queue.pop_front() {
            order.push(id.clone());
            for &next in &adj[id] {
                *in_degree.get_mut(next).unwrap() -= 1;
                if in_degree[next] == 0 {
                    queue.push_back(next);
                }
            }
        }

        if order.len() != self.nodes.len() {
            return Err("cycle detected in DAG".into());
        }
        Ok(order)
    }

    pub fn run<F>(&self, mut executor: F) -> Result<Vec<(String, bool)>, String>
    where
        F: FnMut(&Node<T>) -> bool,
    {
        let order = self.topological_order()?;
        let mut results = Vec::new();
        for id in &order {
            let node = self.nodes.get(id).unwrap();
            let ok = executor(node);
            results.push((id.clone(), ok));
        }
        Ok(results)
    }
}

impl<T: Clone> Default for DagRunner<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// FNV-1a 64 — the digest every substrate in the SuperInstance fleet agrees on.
pub const FNV_OFFSET: u64 = 0xcbf29ce484222325;
pub const FNV_PRIME: u64 = 0x100000001b3;

#[inline]
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    for &b in bytes {
        h = (h ^ b as u64).wrapping_mul(FNV_PRIME);
    }
    h
}

/// True if this crate's FNV-1a still agrees with the rest of the fleet.
pub fn canary_holds() -> bool {
    fnv1a64("café Δ 日本語".as_bytes()) == 0x024a555471370b18d
}
