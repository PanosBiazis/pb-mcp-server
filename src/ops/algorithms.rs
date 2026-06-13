//! # Algorithm & Math Tools
//!
//! Sorting, searching, graph algorithms, matrix operations, statistics,
//! DFT, hashing, string similarity, and bloom filter — all as pure
//! `serde_json::Value` → `serde_json::Value` work functions.

use serde_json::{json, Value};
use sha2::{Digest, Sha256, Sha512};
use std::collections::{BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque};

// ---------------------------------------------------------------------------
// 1. algo_sort_work
// ---------------------------------------------------------------------------

/// Sort a JSON array with an optional key for object items and algorithm label.
pub fn algo_sort_work(args: Value) -> Value {
    let items = match args.get("items").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => return json!({"error": "items must be a JSON array"}),
    };
    if items.is_empty() {
        return json!({"algorithm": "none", "sorted": [], "count": 0});
    }
    let algorithm = args
        .get("algorithm")
        .and_then(|v| v.as_str())
        .unwrap_or("auto")
        .to_ascii_lowercase();
    let label = match algorithm.as_str() {
        "quicksort" | "mergesort" | "heapsort" | "auto" => algorithm,
        other => return json!({"error": format!("unknown algorithm: {other}")}),
    };
    let reverse = args
        .get("reverse")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let key = args.get("key").and_then(|v| v.as_str()).map(String::from);

    let mut sorted = items;
    sorted.sort_by(|a, b| {
        let va = if let Some(ref k) = key {
            a.get(k).cloned().unwrap_or(Value::Null)
        } else {
            a.clone()
        };
        let vb = if let Some(ref k) = key {
            b.get(k).cloned().unwrap_or(Value::Null)
        } else {
            b.clone()
        };
        let ord = value_cmp(&va, &vb);
        if reverse {
            ord.reverse()
        } else {
            ord
        }
    });

    json!({
        "algorithm": label,
        "sorted": sorted,
        "count": sorted.len(),
        "reverse": reverse,
    })
}

fn value_cmp(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    use Value::*;
    match (a, b) {
        (Null, Null) => Equal,
        (Null, _) => Less,
        (_, Null) => Greater,
        (Bool(x), Bool(y)) => x.cmp(y),
        (Bool(_), Number(_)) => Less,
        (Number(_), Bool(_)) => Greater,
        (Number(x), Number(y)) => {
            if let (Some(fa), Some(fb)) = (x.as_f64(), y.as_f64()) {
                fa.partial_cmp(&fb).unwrap_or(Equal)
            } else {
                Equal
            }
        }
        (Number(_), String(_)) => Less,
        (String(_), Number(_)) => Greater,
        (String(x), String(y)) => x.cmp(y),
        (String(_), Array(_)) => Less,
        (Array(_), String(_)) => Greater,
        (Array(x), Array(y)) => {
            for (va, vb) in x.iter().zip(y.iter()) {
                match value_cmp(va, vb) {
                    Equal => continue,
                    other => return other,
                }
            }
            x.len().cmp(&y.len())
        }
        (Array(_), Object(_)) => Less,
        (Object(_), Array(_)) => Greater,
        (Object(x), Object(y)) => {
            let mut ka: Vec<_> = x.keys().collect();
            let mut kb: Vec<_> = y.keys().collect();
            ka.sort();
            kb.sort();
            for (ska, skb) in ka.iter().zip(kb.iter()) {
                match ska.cmp(skb) {
                    Equal => continue,
                    other => return other,
                }
            }
            match ka.len().cmp(&kb.len()) {
                Equal => {}
                other => return other,
            }
            for k in ka {
                let va = &x[k];
                let vb = y.get(k).unwrap_or(&Null);
                match value_cmp(va, vb) {
                    Equal => continue,
                    other => return other,
                }
            }
            Equal
        }
        (Object(_), Bool(_)) | (Object(_), Number(_)) => Greater,
        _ => Equal,
    }
}

// ---------------------------------------------------------------------------
// 2. algo_search_work
// ---------------------------------------------------------------------------

/// Binary or linear search over a JSON array.
pub fn algo_search_work(args: Value) -> Value {
    let items = match args.get("items").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => return json!({"error": "items must be a JSON array"}),
    };
    let target = match args.get("target") {
        Some(t) => t.clone(),
        None => return json!({"error": "target is required"}),
    };
    let method = args
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("linear")
        .to_ascii_lowercase();

    match method.as_str() {
        "linear" => {
            for (i, item) in items.iter().enumerate() {
                if value_eq(item, &target) {
                    return json!({"found": true, "index": i, "method": "linear"});
                }
            }
            json!({"found": false, "index": null, "method": "linear"})
        }
        "binary" => {
            let mut lo: i64 = 0;
            let mut hi: i64 = items.len() as i64 - 1;
            while lo <= hi {
                let mid = lo + (hi - lo) / 2;
                match value_cmp(&items[mid as usize], &target) {
                    std::cmp::Ordering::Equal => {
                        return json!({"found": true, "index": mid, "method": "binary"});
                    }
                    std::cmp::Ordering::Less => lo = mid + 1,
                    std::cmp::Ordering::Greater => hi = mid - 1,
                }
            }
            json!({"found": false, "index": null, "method": "binary"})
        }
        _ => json!({"error": format!("unknown method: {method}; use linear or binary")}),
    }
}

fn value_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Number(x), Value::Number(y)) => x
            .as_f64()
            .zip(y.as_f64())
            .map(|(a, b)| (a - b).abs() < 1e-12)
            .unwrap_or(false),
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y.iter()).all(|(a, b)| value_eq(a, b))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|v2| value_eq(v, v2)))
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// 3. algo_graph_bfs_work
// ---------------------------------------------------------------------------

/// BFS traversal of a JSON adjacency graph.
pub fn algo_graph_bfs_work(args: Value) -> Value {
    let graph = match args.get("graph").and_then(|v| v.as_object()) {
        Some(o) => o,
        None => return json!({"error": "graph must be a JSON object"}),
    };
    let start = match args.get("start").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return json!({"error": "start (string node ID) is required"}),
    };
    if !graph.contains_key(&start) {
        return json!({"error": format!("start node '{start}' not found in graph")});
    }

    let mut visited = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    queue.push_back(start.clone());
    seen.insert(start);

    while let Some(node) = queue.pop_front() {
        visited.push(node.clone());
        if let Some(neighbors) = graph.get(&node).and_then(|v| v.as_array()) {
            for n in neighbors {
                if let Some(ns) = n.as_str() {
                    if seen.insert(ns.to_string()) {
                        queue.push_back(ns.to_string());
                    }
                }
            }
        }
    }
    json!({"visited": visited, "count": visited.len()})
}

// ---------------------------------------------------------------------------
// 4. algo_graph_dfs_work
// ---------------------------------------------------------------------------

/// DFS traversal of a JSON adjacency graph.
pub fn algo_graph_dfs_work(args: Value) -> Value {
    let graph = match args.get("graph").and_then(|v| v.as_object()) {
        Some(o) => o,
        None => return json!({"error": "graph must be a JSON object"}),
    };
    let start = match args.get("start").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return json!({"error": "start (string node ID) is required"}),
    };
    if !graph.contains_key(&start) {
        return json!({"error": format!("start node '{start}' not found in graph")});
    }

    let mut visited = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut stack: Vec<String> = vec![start.clone()];
    seen.insert(start);

    while let Some(node) = stack.pop() {
        visited.push(node.clone());
        if let Some(neighbors) = graph.get(&node).and_then(|v| v.as_array()) {
            let mut nbrs: Vec<String> = neighbors
                .iter()
                .filter_map(|n| n.as_str().map(String::from))
                .collect();
            nbrs.reverse(); // process in natural order via stack
            for ns in nbrs {
                if seen.insert(ns.clone()) {
                    stack.push(ns);
                }
            }
        }
    }
    json!({"visited": visited, "count": visited.len()})
}

// ---------------------------------------------------------------------------
// 5. algo_graph_shortest_path_work
// ---------------------------------------------------------------------------

/// Dijkstra shortest path on a weighted graph {node: {neighbor: weight}}.
pub fn algo_graph_shortest_path_work(args: Value) -> Value {
    let graph = match args.get("graph").and_then(|v| v.as_object()) {
        Some(o) => o,
        None => return json!({"error": "graph must be a JSON object"}),
    };
    let start = match args.get("start").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return json!({"error": "start is required"}),
    };
    let end = match args.get("end").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return json!({"error": "end is required"}),
    };
    if !graph.contains_key(&start) {
        return json!({"error": format!("start node '{start}' not in graph")});
    }

    // Parse adjacency with weights
    let mut adj: HashMap<String, Vec<(String, f64)>> = HashMap::new();
    for (node, neighbors) in graph {
        adj.entry(node.clone()).or_default();
        if let Some(obj) = neighbors.as_object() {
            for (nbr, w) in obj {
                let weight = w.as_f64().unwrap_or(1.0);
                adj.entry(node.clone())
                    .or_default()
                    .push((nbr.clone(), weight));
                adj.entry(nbr.clone()).or_default();
            }
        }
    }

    let mut dist: HashMap<String, f64> = HashMap::new();
    let mut prev: HashMap<String, Option<String>> = HashMap::new();
    for n in adj.keys() {
        dist.insert(n.clone(), f64::INFINITY);
        prev.insert(n.clone(), None);
    }
    dist.insert(start.clone(), 0.0);

    // We use a hack: store (neg_dist_as_bits, node) via Reverse + Ordering trick
    // Simpler: use a sorted vec approach for small graphs
    let mut visited: HashSet<String> = HashSet::new();

    // Simple Dijkstra with a priority queue via binary heap of (OrderedFloat, String)
    // Since we can't bring in ordered-float, we'll use a workaround:
    // Store (dist_bits, node) where we interpret f64 bits for ordering
    struct F64Bits(u64);
    impl PartialEq for F64Bits {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }
    impl Eq for F64Bits {}
    impl PartialOrd for F64Bits {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for F64Bits {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            // For finite non-negative f64, bits ordering matches value ordering
            self.0.cmp(&other.0)
        }
    }

    let mut heap: BinaryHeap<std::cmp::Reverse<(F64Bits, String)>> = BinaryHeap::new();
    if let Some(bits) = dist.get(&start) {
        heap.push(std::cmp::Reverse((F64Bits(bits.to_bits()), start.clone())));
    }

    while let Some(std::cmp::Reverse((F64Bits(d_bits), node))) = heap.pop() {
        let d = f64::from_bits(d_bits);
        if visited.contains(&node) {
            continue;
        }
        visited.insert(node.clone());
        if node == end {
            break;
        }
        if let Some(neighbors) = adj.get(&node) {
            for (nbr, weight) in neighbors {
                let nd = d + weight;
                if nd < dist.get(nbr).copied().unwrap_or(f64::INFINITY) {
                    dist.insert(nbr.clone(), nd);
                    prev.insert(nbr.clone(), Some(node.clone()));
                    heap.push(std::cmp::Reverse((F64Bits(nd.to_bits()), nbr.clone())));
                }
            }
        }
    }

    let total = dist.get(&end).copied().unwrap_or(f64::INFINITY);
    if total == f64::INFINITY {
        return json!({"error": format!("no path from '{start}' to '{end}'"), "path": null, "distance": null});
    }

    // Reconstruct path
    let mut path = Vec::new();
    let mut current = Some(end.clone());
    while let Some(ref n) = current {
        path.push(n.clone());
        current = prev.get(n).and_then(|o| o.clone());
    }
    path.reverse();

    json!({"path": path, "distance": total})
}

// ---------------------------------------------------------------------------
// 6. algo_graph_topo_sort_work
// ---------------------------------------------------------------------------

/// Topological sort of a DAG (Kahn's algorithm).
pub fn algo_graph_topo_sort_work(args: Value) -> Value {
    let graph = match args.get("graph").and_then(|v| v.as_object()) {
        Some(o) => o,
        None => return json!({"error": "graph must be a JSON object"}),
    };

    // graph: node -> [dependents] (i.e. directed edges node -> dependent)
    // For topo sort we need in-degrees
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();

    // Collect all nodes
    for (node, neighbors) in graph {
        in_degree.entry(node.clone()).or_insert(0);
        adj.entry(node.clone()).or_default();
        if let Some(arr) = neighbors.as_array() {
            for n in arr {
                if let Some(ns) = n.as_str() {
                    adj.entry(node.clone()).or_default().push(ns.to_string());
                    in_degree.entry(ns.to_string()).or_insert(0);
                }
            }
        }
    }

    // Compute in-degrees
    for neighbors in adj.values() {
        for n in neighbors {
            *in_degree.entry(n.clone()).or_insert(0) += 1;
        }
    }

    // Kahn's algorithm
    let mut queue: VecDeque<String> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(n, _)| n.clone())
        .collect();
    // Sort for deterministic order
    let mut sorted_queue: Vec<String> = queue.drain(..).collect();
    sorted_queue.sort();
    for s in sorted_queue {
        queue.push_back(s);
    }

    let mut result = Vec::new();
    while let Some(node) = queue.pop_front() {
        result.push(node.clone());
        if let Some(neighbors) = adj.get(&node) {
            let mut next_ready = Vec::new();
            for n in neighbors {
                if let Some(deg) = in_degree.get_mut(n) {
                    *deg -= 1;
                    if *deg == 0 {
                        next_ready.push(n.clone());
                    }
                }
            }
            next_ready.sort();
            for s in next_ready {
                queue.push_back(s);
            }
        }
    }

    if result.len() != in_degree.len() {
        let cycle_nodes: Vec<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg > 0)
            .map(|(n, _)| n.clone())
            .collect();
        return json!({"error": "cycle detected", "cycle_nodes": cycle_nodes});
    }

    json!({"sorted": result})
}

// ---------------------------------------------------------------------------
// 7. algo_matrix_ops_work
// ---------------------------------------------------------------------------

/// Matrix operations: multiply, transpose, determinant, inverse.
pub fn algo_matrix_ops_work(args: Value) -> Value {
    let operation = match args.get("operation").and_then(|v| v.as_str()) {
        Some(s) => s.to_ascii_lowercase(),
        None => return json!({"error": "operation is required"}),
    };

    let a = match args.get("a").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return json!({"error": "a must be a 2D array"}),
    };

    let mat_a = match parse_matrix(a) {
        Ok(m) => m,
        Err(e) => return json!({"error": format!("matrix a: {e}")}),
    };

    match operation.as_str() {
        "transpose" => {
            let result = transpose(&mat_a);
            json!({"operation": "transpose", "result": matrix_to_json(&result)})
        }
        "determinant" => {
            let n = mat_a.len();
            if n == 0 || mat_a[0].len() != n {
                return json!({"error": "determinant requires a square matrix"});
            }
            let det = determinant(&mat_a);
            json!({"operation": "determinant", "result": det})
        }
        "inverse" => {
            let n = mat_a.len();
            if n == 0 || mat_a[0].len() != n {
                return json!({"error": "inverse requires a square matrix"});
            }
            let det = determinant(&mat_a);
            if det.abs() < 1e-12 {
                return json!({"error": "matrix is singular (determinant ≈ 0)"});
            }
            match inverse(&mat_a) {
                Some(inv) => json!({"operation": "inverse", "result": matrix_to_json(&inv)}),
                None => json!({"error": "matrix is singular"}),
            }
        }
        "multiply" => {
            let b = match args.get("b").and_then(|v| v.as_array()) {
                Some(arr) => arr,
                None => return json!({"error": "b is required for multiply"}),
            };
            let mat_b = match parse_matrix(b) {
                Ok(m) => m,
                Err(e) => return json!({"error": format!("matrix b: {e}")}),
            };
            if mat_a.is_empty() || mat_b.is_empty() {
                return json!({"error": "matrices must not be empty"});
            }
            if mat_a[0].len() != mat_b.len() {
                return json!({"error": format!(
                    "incompatible dimensions: {}x{} * {}x{}",
                    mat_a.len(), mat_a[0].len(), mat_b.len(), mat_b[0].len()
                )});
            }
            let result = multiply(&mat_a, &mat_b);
            json!({"operation": "multiply", "result": matrix_to_json(&result)})
        }
        _ => json!({"error": format!("unknown operation: {operation}")}),
    }
}

fn parse_matrix(arr: &[Value]) -> Result<Vec<Vec<f64>>, String> {
    let mut matrix = Vec::new();
    for row in arr {
        match row.as_array() {
            Some(r) => {
                let parsed: Vec<f64> = r
                    .iter()
                    .map(|v| v.as_f64().ok_or("non-numeric value"))
                    .collect::<Result<_, _>>()
                    .map_err(|e| e.to_string())?;
                matrix.push(parsed);
            }
            None => return Err("each row must be an array".into()),
        }
    }
    // Validate consistent column count
    if !matrix.is_empty() {
        let cols = matrix[0].len();
        if matrix.iter().any(|r| r.len() != cols) {
            return Err("inconsistent row lengths".into());
        }
    }
    Ok(matrix)
}

fn transpose(m: &[Vec<f64>]) -> Vec<Vec<f64>> {
    if m.is_empty() {
        return vec![];
    }
    let rows = m.len();
    let cols = m[0].len();
    let mut result = vec![vec![0.0; rows]; cols];
    for i in 0..rows {
        for j in 0..cols {
            result[j][i] = m[i][j];
        }
    }
    result
}

fn multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = a.len();
    let m = b[0].len();
    let p = b.len();
    let mut result = vec![vec![0.0; m]; n];
    for i in 0..n {
        for j in 0..m {
            for k in 0..p {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    result
}

#[allow(clippy::needless_range_loop)]
fn determinant(m: &[Vec<f64>]) -> f64 {
    let n = m.len();
    if n == 0 {
        return 1.0;
    }
    let mut a = m.to_vec();
    let mut sign = 1.0;
    for k in 0..n {
        let mut pivot = k;
        let mut pivot_abs = a[k][k].abs();
        for (i, row) in a.iter().enumerate().skip(k + 1) {
            let cand = row[k].abs();
            if cand > pivot_abs {
                pivot = i;
                pivot_abs = cand;
            }
        }
        if pivot_abs < 1e-12 {
            return 0.0;
        }
        if pivot != k {
            a.swap(k, pivot);
            sign = -sign;
        }
        for i in (k + 1)..n {
            let factor = a[i][k] / a[k][k];
            a[i][k] = 0.0;
            for j in (k + 1)..n {
                a[i][j] -= factor * a[k][j];
            }
        }
    }
    let mut det = sign;
    for (i, row) in a.iter().enumerate() {
        det *= row[i];
    }
    det
}

#[allow(clippy::needless_range_loop)]
fn inverse(m: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let n = m.len();
    // Augment with identity
    let mut aug = vec![vec![0.0; 2 * n]; n];
    for i in 0..n {
        for j in 0..n {
            aug[i][j] = m[i][j];
        }
        aug[i][n + i] = 1.0;
    }
    // Gauss-Jordan
    for col in 0..n {
        // Find pivot
        let mut pivot_row = col;
        for row in (col + 1)..n {
            if aug[row][col].abs() > aug[pivot_row][col].abs() {
                pivot_row = row;
            }
        }
        if aug[pivot_row][col].abs() < 1e-12 {
            return None;
        }
        // Swap
        aug.swap(col, pivot_row);
        // Scale pivot row
        let pivot_val = aug[col][col];
        for j in 0..2 * n {
            aug[col][j] /= pivot_val;
        }
        // Eliminate other rows
        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = aug[row][col];
            for j in 0..2 * n {
                aug[row][j] -= factor * aug[col][j];
            }
        }
    }
    // Extract inverse
    let mut result = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            result[i][j] = round_f64(aug[i][n + j]);
        }
    }
    Some(result)
}

fn round_f64(v: f64) -> f64 {
    let rounded = (v * 1e10).round() / 1e10;
    if rounded.abs() < 1e-10 {
        0.0
    } else {
        rounded
    }
}

fn matrix_to_json(m: &[Vec<f64>]) -> Value {
    Value::Array(
        m.iter()
            .map(|row| Value::Array(row.iter().map(|&v| json!(round_f64(v))).collect()))
            .collect(),
    )
}

// ---------------------------------------------------------------------------
// 8. algo_stats_describe_work
// ---------------------------------------------------------------------------

/// Descriptive statistics for an array of numbers.
pub fn algo_stats_describe_work(args: Value) -> Value {
    let values = match args.get("values").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return json!({"error": "values must be a JSON array"}),
    };
    let nums: Vec<f64> = values.iter().filter_map(|v| v.as_f64()).collect();
    if nums.is_empty() {
        return json!({"error": "values array is empty or contains no numbers"});
    }

    let n = nums.len();
    let sum: f64 = nums.iter().copied().sum();
    let mean = sum / n as f64;

    let mut sorted = nums.clone();
    sorted.sort_by(|a, b| a.total_cmp(b));

    let min = sorted[0];
    let max = sorted[n - 1];
    let median = percentile(&sorted, 50.0);
    let p25 = percentile(&sorted, 25.0);
    let p50 = percentile(&sorted, 50.0);
    let p75 = percentile(&sorted, 75.0);
    let p95 = percentile(&sorted, 95.0);
    let p99 = percentile(&sorted, 99.0);
    let iqr = p75 - p25;
    let lower_fence = p25 - (1.5 * iqr);
    let upper_fence = p75 + (1.5 * iqr);
    let outlier_count = nums
        .iter()
        .filter(|v| **v < lower_fence || **v > upper_fence)
        .count();

    // Mode: most frequent value (rounded to 4 decimal places for grouping)
    let mut freq: HashMap<i64, usize> = HashMap::new();
    for &v in &nums {
        let key = (v * 10000.0).round() as i64;
        *freq.entry(key).or_insert(0) += 1;
    }
    let max_freq = *freq.values().max().unwrap_or(&0);
    let mode_val = freq
        .iter()
        .filter(|(_, &c)| c == max_freq)
        .min_by_key(|(k, _)| k.abs())
        .map(|(k, _)| *k as f64 / 10000.0)
        .unwrap_or(mean);

    // Standard deviation (population)
    let variance: f64 = nums.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n as f64;
    let std_dev = variance.sqrt();

    json!({
        "mean": round_f64(mean),
        "median": round_f64(median),
        "mode": round_f64(mode_val),
        "std_dev": round_f64(std_dev),
        "min": round_f64(min),
        "max": round_f64(max),
        "percentiles": {
            "p25": round_f64(p25),
            "p50": round_f64(p50),
            "p75": round_f64(p75),
            "p95": round_f64(p95),
            "p99": round_f64(p99),
        },
        "iqr": round_f64(iqr),
        "outlier_fences": {
            "lower": round_f64(lower_fence),
            "upper": round_f64(upper_fence),
        },
        "outlier_count": outlier_count,
        "count": n,
    })
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let n = sorted.len();
    if n == 1 {
        return sorted[0];
    }
    let idx = (p / 100.0) * (n - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    let frac = idx - lo as f64;
    let lo = lo.min(n - 1);
    let hi = hi.min(n - 1);
    sorted[lo] * (1.0 - frac) + sorted[hi] * frac
}

// ---------------------------------------------------------------------------
// 9. algo_stats_correlation_work
// ---------------------------------------------------------------------------

/// Pearson or Spearman correlation between two arrays.
pub fn algo_stats_correlation_work(args: Value) -> Value {
    let x = match args.get("x").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return json!({"error": "x must be a JSON array"}),
    };
    let y = match args.get("y").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return json!({"error": "y must be a JSON array"}),
    };
    let method = args
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("pearson")
        .to_ascii_lowercase();

    let xv: Vec<f64> = x.iter().filter_map(|v| v.as_f64()).collect();
    let yv: Vec<f64> = y.iter().filter_map(|v| v.as_f64()).collect();
    if xv.len() != yv.len() || xv.len() < 2 {
        return json!({"error": "x and y must have the same length >= 2"});
    }

    match method.as_str() {
        "pearson" => {
            let r = pearson(&xv, &yv);
            json!({"method": "pearson", "correlation": round_f64(r), "n": xv.len()})
        }
        "spearman" => {
            let rx = rank(&xv);
            let ry = rank(&yv);
            let r = pearson(&rx, &ry);
            json!({"method": "spearman", "correlation": round_f64(r), "n": xv.len()})
        }
        _ => json!({"error": format!("unknown method: {method}; use pearson or spearman")}),
    }
}

fn pearson(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let mx: f64 = x.iter().copied().sum::<f64>() / n;
    let my: f64 = y.iter().copied().sum::<f64>() / n;
    let mut cov = 0.0;
    let mut sx = 0.0;
    let mut sy = 0.0;
    for i in 0..x.len() {
        let dx = x[i] - mx;
        let dy = y[i] - my;
        cov += dx * dy;
        sx += dx * dx;
        sy += dy * dy;
    }
    let denom = sx.sqrt() * sy.sqrt();
    if denom.abs() < 1e-12 {
        0.0
    } else {
        cov / denom
    }
}

fn rank(data: &[f64]) -> Vec<f64> {
    let n = data.len();
    let mut indexed: Vec<(usize, f64)> = data.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut ranks = vec![0.0f64; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j < n && (indexed[j].1 - indexed[i].1).abs() < 1e-12 {
            j += 1;
        }
        // Average rank for ties
        let avg_rank = ((i + 1 + j) as f64) / 2.0;
        for k in i..j {
            ranks[indexed[k].0] = avg_rank;
        }
        i = j;
    }
    ranks
}

// ---------------------------------------------------------------------------
// 10. algo_stats_regression_work
// ---------------------------------------------------------------------------

/// Simple linear regression: y = slope * x + intercept.
pub fn algo_stats_regression_work(args: Value) -> Value {
    let x = match args.get("x").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return json!({"error": "x must be a JSON array"}),
    };
    let y = match args.get("y").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return json!({"error": "y must be a JSON array"}),
    };
    let xv: Vec<f64> = x.iter().filter_map(|v| v.as_f64()).collect();
    let yv: Vec<f64> = y.iter().filter_map(|v| v.as_f64()).collect();
    if xv.len() != yv.len() || xv.len() < 2 {
        return json!({"error": "x and y must have the same length >= 2"});
    }

    let n = xv.len() as f64;
    let mx: f64 = xv.iter().copied().sum::<f64>() / n;
    let my: f64 = yv.iter().copied().sum::<f64>() / n;
    let mut sxy = 0.0;
    let mut sxx = 0.0;
    for i in 0..xv.len() {
        let dx = xv[i] - mx;
        sxy += dx * (yv[i] - my);
        sxx += dx * dx;
    }
    if sxx.abs() < 1e-12 {
        return json!({"error": "x has zero variance (all values identical)"});
    }
    let slope = sxy / sxx;
    let intercept = my - slope * mx;
    let r = pearson(&xv, &yv);

    json!({
        "slope": round_f64(slope),
        "intercept": round_f64(intercept),
        "r_squared": round_f64(r * r),
        "n": xv.len(),
    })
}

// ---------------------------------------------------------------------------
// 11. algo_fft_work
// ---------------------------------------------------------------------------

/// Simple DFT (not FFT). Returns first N/2 bins with frequency, magnitude, phase.
#[allow(clippy::needless_range_loop)]
pub fn algo_fft_work(args: Value) -> Value {
    let values = match args.get("values").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return json!({"error": "values must be a JSON array"}),
    };
    let nums: Vec<f64> = values.iter().filter_map(|v| v.as_f64()).collect();
    let n = nums.len();
    if n == 0 {
        return json!({"error": "values array is empty or contains no numbers"});
    }

    let half = n / 2;
    let mut bins = Vec::with_capacity(half);
    for k in 0..half {
        let mut re = 0.0f64;
        let mut im = 0.0f64;
        for t in 0..n {
            let angle = -2.0 * std::f64::consts::PI * k as f64 * t as f64 / n as f64;
            re += nums[t] * angle.cos();
            im += nums[t] * angle.sin();
        }
        let magnitude = (re * re + im * im).sqrt() / n as f64;
        let phase = im.atan2(re);
        bins.push(json!({
            "frequency": round_f64(k as f64),
            "magnitude": round_f64(magnitude),
            "phase": round_f64(phase),
        }));
    }
    json!({"bins": bins, "n": n})
}

// ---------------------------------------------------------------------------
// 12. algo_hash_text_work
// ---------------------------------------------------------------------------

/// Hash text using SHA-256, SHA-512, MD5 (simple impl), or blake3 (falls back to SHA-256).
pub fn algo_hash_text_work(args: Value) -> Value {
    let text = match args.get("text").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return json!({"error": "text is required"}),
    };
    let algorithm = args
        .get("algorithm")
        .and_then(|v| v.as_str())
        .unwrap_or("sha256")
        .to_ascii_lowercase();

    match algorithm.as_str() {
        "sha256" => {
            let digest = Sha256::digest(text.as_bytes());
            let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
            json!({"algorithm": "sha256", "hex": hex, "input_bytes": text.len()})
        }
        "sha512" => {
            let digest = Sha512::digest(text.as_bytes());
            let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
            json!({"algorithm": "sha512", "hex": hex, "input_bytes": text.len()})
        }
        "md5" => {
            let hex = md5_simple(text.as_bytes());
            json!({"algorithm": "md5", "hex": hex, "input_bytes": text.len()})
        }
        "blake3" => {
            // blake3 crate not available; fall back to sha256
            let digest = Sha256::digest(text.as_bytes());
            let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
            json!({
                "algorithm": "blake3",
                "note": "blake3 not available; fell back to sha256",
                "hex": hex,
                "input_bytes": text.len(),
            })
        }
        _ => {
            json!({"error": format!("unknown algorithm: {algorithm}; use sha256, sha512, md5, or blake3")})
        }
    }
}

/// Simple MD5 implementation (RFC 1321).
fn md5_simple(input: &[u8]) -> String {
    let mut ctx = Md5Ctx::new();
    ctx.update(input);
    ctx.finalize()
}

struct Md5Ctx {
    state: [u32; 4],
    count: u64,
    buffer: [u8; 64],
    buf_len: usize,
}

impl Md5Ctx {
    fn new() -> Self {
        Md5Ctx {
            state: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476],
            count: 0,
            buffer: [0u8; 64],
            buf_len: 0,
        }
    }

    fn update(&mut self, input: &[u8]) {
        let mut data = input;
        self.count += data.len() as u64;

        // Fill buffer
        if self.buf_len > 0 {
            let space = 64 - self.buf_len;
            let take = data.len().min(space);
            self.buffer[self.buf_len..self.buf_len + take].copy_from_slice(&data[..take]);
            self.buf_len += take;
            data = &data[take..];
            if self.buf_len == 64 {
                let block = self.buffer;
                self.transform(&block);
                self.buf_len = 0;
            }
        }

        // Process full blocks
        while data.len() >= 64 {
            let (block, rest) = data.split_at(64);
            if let Ok(block) = block.try_into() {
                self.transform(block);
            }
            data = rest;
        }

        // Buffer remaining
        if !data.is_empty() {
            self.buffer[..data.len()].copy_from_slice(data);
            self.buf_len = data.len();
        }
    }

    fn finalize(&mut self) -> String {
        // Pad: append 0x80, then zeros, then 64-bit length in bits (little-endian)
        let bit_count = self.count * 8;
        // We need: (buf_len + 1 + pad_len) % 64 == 56
        // i.e. pad_len = (56 - (buf_len + 1) % 64) % 64
        let pad_len = (56u64 - (self.buf_len as u64 + 1) % 64) % 64;
        let total_pad = 1 + pad_len as usize;
        let mut padding = vec![0u8; total_pad];
        padding[0] = 0x80;
        // Append length
        let len_bytes = bit_count.to_le_bytes();
        padding.extend_from_slice(&len_bytes);
        // Process padding directly without going through update(),
        // so we don't add padding bytes to self.count.
        // Fill the buffer first.
        let mut data: &[u8] = &padding;
        if self.buf_len > 0 {
            let space = 64 - self.buf_len;
            let take = data.len().min(space);
            self.buffer[self.buf_len..self.buf_len + take].copy_from_slice(&data[..take]);
            self.buf_len += take;
            data = &data[take..];
            if self.buf_len == 64 {
                let block = self.buffer;
                self.transform(&block);
                self.buf_len = 0;
            }
        }
        // Process full blocks
        while data.len() >= 64 {
            let (block, rest) = data.split_at(64);
            if let Ok(block) = block.try_into() {
                self.transform(block);
            }
            data = rest;
        }
        // Buffer remaining
        if !data.is_empty() {
            self.buffer[..data.len()].copy_from_slice(data);
            self.buf_len = data.len();
        }
        let mut result = String::with_capacity(32);
        for &v in &self.state {
            result.push_str(&format!("{:08x}", v.to_le()));
        }
        result
    }

    fn transform(&mut self, block: &[u8; 64]) {
        let mut m = [0u32; 16];
        for i in 0..16 {
            m[i] = u32::from_le_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
        }

        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];

        const S: [u32; 64] = [
            7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20,
            5, 9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
            6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
        ];

        const K: [u32; 64] = [
            0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613,
            0xfd469501, 0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193,
            0xa679438e, 0x49b40821, 0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d,
            0x02441453, 0xd8a1e681, 0xe7d3fbc8, 0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
            0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a, 0xfffa3942, 0x8771f681, 0x6d9d6122,
            0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70, 0x289b7ec6, 0xeaa127fa,
            0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665, 0xf4292244,
            0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
            0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb,
            0xeb86d391,
        ];

        // Round 1
        for i in 0..16 {
            let f = (b & c) | (!b & d);
            let g = i as u32;
            a = a
                .wrapping_add(f)
                .wrapping_add(m[g as usize])
                .wrapping_add(K[i]);
            a = a.rotate_left(S[i]).wrapping_add(b);
            let tmp = d;
            d = c;
            c = b;
            b = b.wrapping_add(a);
            a = tmp;
        }

        // Round 2
        for i in 16..32 {
            let f = (d & b) | (!d & c);
            let g = ((5 * i as u32) + 1) % 16;
            a = a
                .wrapping_add(f)
                .wrapping_add(m[g as usize])
                .wrapping_add(K[i]);
            a = a.rotate_left(S[i]).wrapping_add(b);
            let tmp = d;
            d = c;
            c = b;
            b = b.wrapping_add(a);
            a = tmp;
        }

        // Round 3
        for i in 32..48 {
            let f = b ^ c ^ d;
            let g = ((3 * i as u32) + 5) % 16;
            a = a
                .wrapping_add(f)
                .wrapping_add(m[g as usize])
                .wrapping_add(K[i]);
            a = a.rotate_left(S[i]).wrapping_add(b);
            let tmp = d;
            d = c;
            c = b;
            b = b.wrapping_add(a);
            a = tmp;
        }

        // Round 4
        for i in 48..64 {
            let f = c ^ (b | !d);
            let g = (7 * i as u32) % 16;
            a = a
                .wrapping_add(f)
                .wrapping_add(m[g as usize])
                .wrapping_add(K[i]);
            a = a.rotate_left(S[i]).wrapping_add(b);
            let tmp = d;
            d = c;
            c = b;
            b = b.wrapping_add(a);
            a = tmp;
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
    }
}

// ---------------------------------------------------------------------------
// 13. algo_edit_distance_work
// ---------------------------------------------------------------------------

/// Levenshtein edit distance between two strings.
pub fn algo_edit_distance_work(args: Value) -> Value {
    let a = match args.get("a").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return json!({"error": "a (string) is required"}),
    };
    let b = match args.get("b").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return json!({"error": "b (string) is required"}),
    };
    let dist = levenshtein(a, b);
    json!({"distance": dist, "a_len": a.chars().count(), "b_len": b.chars().count()})
}

fn levenshtein(a: &str, b: &str) -> usize {
    let ac: Vec<char> = a.chars().collect();
    let bc: Vec<char> = b.chars().collect();
    let n = ac.len();
    let m = bc.len();

    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }

    const MAX_CELLS: usize = 4_000_000;
    if n.saturating_mul(m) > MAX_CELLS {
        // Approximate for very large inputs
        return n.max(m);
    }

    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr: Vec<usize> = vec![0; m + 1];

    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = usize::from(ac[i - 1] != bc[j - 1]);
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}

// ---------------------------------------------------------------------------
// 14. algo_ngram_similarity_work
// ---------------------------------------------------------------------------

/// N-gram Jaccard similarity between two strings.
pub fn algo_ngram_similarity_work(args: Value) -> Value {
    let a = match args.get("a").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return json!({"error": "a (string) is required"}),
    };
    let b = match args.get("b").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return json!({"error": "b (string) is required"}),
    };
    let n = args.get("n").and_then(|v| v.as_u64()).unwrap_or(2) as usize;
    if n == 0 {
        return json!({"error": "n must be >= 1"});
    }

    let set_a = ngram_set(a, n);
    let set_b = ngram_set(b, n);

    if set_a.is_empty() && set_b.is_empty() {
        return json!({"similarity": 1.0, "n": n, "ngrams_a": 0, "ngrams_b": 0});
    }

    let inter = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    let similarity = if union == 0 {
        1.0
    } else {
        inter as f64 / union as f64
    };

    json!({
        "similarity": round_f64(similarity),
        "n": n,
        "ngrams_a": set_a.len(),
        "ngrams_b": set_b.len(),
        "intersection": inter,
        "union": union,
    })
}

fn ngram_set(s: &str, n: usize) -> HashSet<String> {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() < n {
        // If string shorter than n, return the whole string as one ngram if non-empty
        let mut set = HashSet::new();
        if !chars.is_empty() {
            set.insert(chars.iter().collect::<String>());
        }
        return set;
    }
    let mut set = HashSet::new();
    for i in 0..=chars.len() - n {
        let ngram: String = chars[i..i + n].iter().collect();
        set.insert(ngram);
    }
    set
}

// ---------------------------------------------------------------------------
// 15. algo_bloom_filter_work
// ---------------------------------------------------------------------------

/// Bloom filter: build or query.
pub fn algo_bloom_filter_work(args: Value) -> Value {
    let operation = match args.get("operation").and_then(|v| v.as_str()) {
        Some(s) => s.to_ascii_lowercase(),
        None => return json!({"error": "operation is required (build or query)"}),
    };

    let size = args.get("size").and_then(|v| v.as_u64()).unwrap_or(1000) as usize;
    let hash_count = args.get("hash_count").and_then(|v| v.as_u64()).unwrap_or(3) as usize;

    if size == 0 || hash_count == 0 {
        return json!({"error": "size and hash_count must be >= 1"});
    }

    match operation.as_str() {
        "build" => {
            let items = match args.get("items").and_then(|v| v.as_array()) {
                Some(a) => a,
                None => return json!({"error": "items (array of strings) is required for build"}),
            };
            let mut bits = vec![false; size];
            let mut count = 0usize;
            for item in items {
                if let Some(s) = item.as_str() {
                    for h in 0..hash_count {
                        let idx = bloom_hash(s, h, size);
                        bits[idx] = true;
                    }
                    count += 1;
                }
            }
            // Represent bit array compactly
            let bit_string: String = bits.iter().map(|b| if *b { '1' } else { '0' }).collect();
            let ones = bits.iter().filter(|&&b| b).count();
            json!({
                "operation": "build",
                "size": size,
                "hash_count": hash_count,
                "items_added": count,
                "bits_set": ones,
                "bit_array": bit_string,
            })
        }
        "query" => {
            let query = match args.get("query").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => return json!({"error": "query (string) is required for query operation"}),
            };
            // Rebuild from items if provided, otherwise need pre-built bits
            let items = match args.get("items").and_then(|v| v.as_array()) {
                Some(a) => a,
                None => {
                    return json!({"error": "items (array of strings) is required to build the filter before querying"})
                }
            };
            let mut bits = vec![false; size];
            for item in items {
                if let Some(s) = item.as_str() {
                    for h in 0..hash_count {
                        let idx = bloom_hash(s, h, size);
                        bits[idx] = true;
                    }
                }
            }
            let mut possibly_present = true;
            for h in 0..hash_count {
                let idx = bloom_hash(query, h, size);
                if !bits[idx] {
                    possibly_present = false;
                    break;
                }
            }
            json!({
                "operation": "query",
                "query": query,
                "possibly_present": possibly_present,
                "size": size,
                "hash_count": hash_count,
            })
        }
        _ => json!({"error": format!("unknown operation: {operation}; use build or query")}),
    }
}

/// Double-hashing scheme for bloom filter: h_i(x) = (h1(x) + i * h2(x)) % size
fn bloom_hash(s: &str, i: usize, size: usize) -> usize {
    // h1: FNV-1a
    let h1 = fnv1a(s.as_bytes());
    // h2: DJB2
    let h2 = djb2(s.as_bytes());
    (h1.wrapping_add((i as u64).wrapping_mul(h2))) as usize % size
}

fn fnv1a(data: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;
    let mut hash = FNV_OFFSET;
    for &b in data {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn djb2(data: &[u8]) -> u64 {
    let mut hash: u64 = 5381;
    for &b in data {
        hash = hash.wrapping_mul(33).wrapping_add(u64::from(b));
    }
    hash
}

// ---------------------------------------------------------------------------
// 16. algo_gcd_lcm_work
// ---------------------------------------------------------------------------

/// Compute greatest common divisor and least common multiple for two integers.
pub fn algo_gcd_lcm_work(args: Value) -> Value {
    let a = match args.get("a").and_then(|v| v.as_i64()) {
        Some(v) => v,
        None => return json!({"error": "a must be an integer"}),
    };
    let b = match args.get("b").and_then(|v| v.as_i64()) {
        Some(v) => v,
        None => return json!({"error": "b must be an integer"}),
    };
    let ga = a.unsigned_abs();
    let gb = b.unsigned_abs();
    let gcd = gcd_u64(ga, gb);
    let lcm = if gcd == 0 {
        0
    } else {
        ga.saturating_div(gcd).saturating_mul(gb)
    };
    json!({"a": a, "b": b, "gcd": gcd, "lcm": lcm})
}

fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

/// Factor a positive integer into prime factors.
pub fn algo_prime_factors_work(args: Value) -> Value {
    let mut n = match args.get("n").and_then(|v| v.as_u64()) {
        Some(v) if v >= 2 => v,
        _ => return json!({"error": "n must be an integer >= 2"}),
    };
    let original = n;
    let mut factors = Vec::new();
    let mut d = 2u64;
    while d <= n / d {
        while n % d == 0 {
            factors.push(d);
            n /= d;
        }
        d += if d == 2 { 1 } else { 2 };
    }
    if n > 1 {
        factors.push(n);
    }
    json!({"n": original, "factors": factors})
}

/// Deterministic trial-division primality check.
pub fn algo_is_prime_work(args: Value) -> Value {
    let n = match args.get("n").and_then(|v| v.as_u64()) {
        Some(v) => v,
        None => return json!({"error": "n must be an integer"}),
    };
    json!({"n": n, "is_prime": is_prime_u64(n)})
}

fn is_prime_u64(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 || n == 3 {
        return true;
    }
    if n.is_multiple_of(2) || n.is_multiple_of(3) {
        return false;
    }
    let mut d = 5u64;
    while d <= n / d {
        if n.is_multiple_of(d) || n.is_multiple_of(d + 2) {
            return false;
        }
        d += 6;
    }
    true
}

/// Generate primes up to a bounded limit using the Sieve of Eratosthenes.
pub fn algo_sieve_primes_work(args: Value) -> Value {
    const MAX_LIMIT: usize = 1_000_000;
    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(100)
        .clamp(2, MAX_LIMIT);
    let mut sieve = vec![true; limit + 1];
    sieve[0] = false;
    sieve[1] = false;
    let mut p = 2usize;
    while p <= limit / p {
        if sieve[p] {
            let mut multiple = p * p;
            while multiple <= limit {
                sieve[multiple] = false;
                multiple += p;
            }
        }
        p += 1;
    }
    let primes: Vec<usize> = sieve
        .iter()
        .enumerate()
        .filter_map(|(i, is_prime)| is_prime.then_some(i))
        .collect();
    json!({"limit": limit, "count": primes.len(), "primes": primes})
}

/// Compute the nth Fibonacci number in u128 range.
pub fn algo_fibonacci_work(args: Value) -> Value {
    let n = args
        .get("n")
        .and_then(|v| v.as_u64())
        .unwrap_or(10)
        .min(186);
    let mut a = 0u128;
    let mut b = 1u128;
    for _ in 0..n {
        let next = a + b;
        a = b;
        b = next;
    }
    json!({"n": n, "value": a.to_string(), "note": "n is capped at 186 to stay inside u128"})
}

/// Convert an integer string between bases 2..36.
pub fn algo_base_convert_work(args: Value) -> Value {
    let value = match args.get("value").and_then(|v| v.as_str()) {
        Some(v) => v.trim(),
        None => return json!({"error": "value must be a string"}),
    };
    let from_base = args.get("from_base").and_then(|v| v.as_u64()).unwrap_or(10);
    let to_base = args.get("to_base").and_then(|v| v.as_u64()).unwrap_or(10);
    if !(2..=36).contains(&from_base) || !(2..=36).contains(&to_base) {
        return json!({"error": "from_base and to_base must be in 2..36"});
    }
    let parsed = match i128::from_str_radix(value, from_base as u32) {
        Ok(v) => v,
        Err(e) => return json!({"error": format!("invalid value for base {from_base}: {e}")}),
    };
    json!({
        "value": value,
        "from_base": from_base,
        "to_base": to_base,
        "decimal": parsed.to_string(),
        "converted": to_base_string(parsed, to_base as u32),
    })
}

fn to_base_string(mut n: i128, base: u32) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let negative = n < 0;
    if negative {
        n = -n;
    }
    let digits = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut out = Vec::new();
    while n > 0 {
        let idx = (n % base as i128) as usize;
        out.push(digits[idx] as char);
        n /= base as i128;
    }
    if negative {
        out.push('-');
    }
    out.iter().rev().collect()
}

/// Convert an integer in 1..3999 to a Roman numeral.
pub fn algo_roman_numeral_work(args: Value) -> Value {
    let number = match args.get("number").and_then(|v| v.as_u64()) {
        Some(v @ 1..=3999) => v as usize,
        _ => return json!({"error": "number must be in 1..3999"}),
    };
    let table = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut n = number;
    let mut roman = String::new();
    for (value, symbol) in table {
        while n >= value {
            roman.push_str(symbol);
            n -= value;
        }
    }
    json!({"number": number, "roman": roman})
}

/// Count n choose k with saturating u128 arithmetic.
pub fn algo_combinations_count_work(args: Value) -> Value {
    let n = args
        .get("n")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        .min(10_000);
    let k_original = args.get("k").and_then(|v| v.as_u64()).unwrap_or(n).min(n);
    let k = k_original.min(n - k_original);
    let mut result = 1u128;
    for i in 1..=k {
        result = result
            .saturating_mul((n - k + i) as u128)
            .saturating_div(i as u128);
    }
    json!({"n": n, "k": k_original, "combinations": result.to_string(), "saturating": true})
}

/// Count nPk with saturating u128 arithmetic.
pub fn algo_permutations_count_work(args: Value) -> Value {
    let n = args
        .get("n")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        .min(10_000);
    let k = args.get("k").and_then(|v| v.as_u64()).unwrap_or(n).min(n);
    let mut result = 1u128;
    for i in 0..k {
        result = result.saturating_mul((n - i) as u128);
    }
    json!({"n": n, "k": k, "permutations": result.to_string(), "saturating": true})
}

/// Compute Shannon entropy in bits per byte for text.
pub fn algo_entropy_work(args: Value) -> Value {
    let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
    if text.is_empty() {
        return json!({"entropy_bits_per_byte": 0.0, "bytes": 0});
    }
    let mut counts = [0usize; 256];
    for byte in text.as_bytes() {
        counts[*byte as usize] += 1;
    }
    let len = text.len() as f64;
    let entropy = counts
        .iter()
        .filter(|&&count| count > 0)
        .map(|&count| {
            let p = count as f64 / len;
            -p * p.log2()
        })
        .sum::<f64>();
    json!({"entropy_bits_per_byte": entropy, "bytes": text.len()})
}

/// Build a bounded frequency table for words or characters.
pub fn algo_frequency_table_work(args: Value) -> Value {
    let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
    let mode = args.get("mode").and_then(|v| v.as_str()).unwrap_or("word");
    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(50)
        .clamp(1, 1_000);
    let mut counts: HashMap<String, usize> = HashMap::new();
    match mode {
        "char" => {
            for ch in text.chars() {
                *counts.entry(ch.to_string()).or_insert(0) += 1;
            }
        }
        "word" => {
            for word in text.split_whitespace() {
                let token = word
                    .trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '_')
                    .to_ascii_lowercase();
                if !token.is_empty() {
                    *counts.entry(token).or_insert(0) += 1;
                }
            }
        }
        _ => return json!({"error": "mode must be word or char"}),
    }
    let mut entries: Vec<_> = counts.into_iter().collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let entries: Vec<Value> = entries
        .into_iter()
        .take(limit)
        .map(|(item, count)| json!({"item": item, "count": count}))
        .collect();
    json!({"mode": mode, "entries": entries})
}

/// Run-length encode a string as [{value, count}] runs.
pub fn algo_run_length_encode_work(args: Value) -> Value {
    let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
    let mut runs = Vec::new();
    let mut chars = text.chars();
    let Some(mut current) = chars.next() else {
        return json!({"runs": [], "run_count": 0});
    };
    let mut count = 1usize;
    for ch in chars {
        if ch == current {
            count += 1;
        } else {
            runs.push(json!({"value": current.to_string(), "count": count}));
            current = ch;
            count = 1;
        }
    }
    runs.push(json!({"value": current.to_string(), "count": count}));
    json!({"runs": runs, "run_count": runs.len()})
}

/// Find lower and upper bounds for a target in a sorted numeric array.
pub fn algo_binary_search_bounds_work(args: Value) -> Value {
    let values = match number_array(&args, "items") {
        Ok(v) => v,
        Err(e) => return json!({"error": e}),
    };
    let target = match args.get("target").and_then(|v| v.as_f64()) {
        Some(v) => v,
        None => return json!({"error": "target must be a number"}),
    };
    let lower = values.partition_point(|v| *v < target);
    let upper = values.partition_point(|v| *v <= target);
    json!({
        "target": target,
        "lower_bound": lower,
        "upper_bound": upper,
        "count_equal": upper.saturating_sub(lower),
    })
}

fn number_array(args: &Value, key: &str) -> Result<Vec<f64>, String> {
    let arr = args
        .get(key)
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("{key} must be an array of numbers"))?;
    if arr.len() > 100_000 {
        return Err(format!("{key} is capped at 100000 values"));
    }
    arr.iter()
        .map(|v| {
            v.as_f64()
                .filter(|f| f.is_finite())
                .ok_or_else(|| format!("{key} must contain only finite numbers"))
        })
        .collect()
}

/// Compute union/intersection/difference over string arrays.
pub fn algo_set_ops_work(args: Value) -> Value {
    let a = string_set(&args, "a");
    let b = string_set(&args, "b");
    let union: HashSet<_> = a.union(&b).cloned().collect();
    let intersection: HashSet<_> = a.intersection(&b).cloned().collect();
    let a_minus_b: HashSet<_> = a.difference(&b).cloned().collect();
    let b_minus_a: HashSet<_> = b.difference(&a).cloned().collect();
    json!({
        "union": sorted_strings(union),
        "intersection": sorted_strings(intersection),
        "a_minus_b": sorted_strings(a_minus_b),
        "b_minus_a": sorted_strings(b_minus_a),
    })
}

fn string_set(args: &Value, key: &str) -> HashSet<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str())
        .take(100_000)
        .map(|s| s.to_string())
        .collect()
}

fn sorted_strings(set: HashSet<String>) -> Vec<String> {
    let mut values: Vec<_> = set.into_iter().collect();
    values.sort();
    values
}

/// Compute a simple moving average over a numeric array.
pub fn algo_moving_average_work(args: Value) -> Value {
    let values = match number_array(&args, "values") {
        Ok(v) => v,
        Err(e) => return json!({"error": e}),
    };
    let window = args
        .get("window")
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(3)
        .clamp(1, values.len().max(1));
    let mut out = Vec::new();
    let mut sum = 0.0;
    for (i, value) in values.iter().enumerate() {
        sum += *value;
        if i >= window {
            sum -= values[i - window];
        }
        let denom = (i + 1).min(window) as f64;
        out.push(sum / denom);
    }
    json!({"window": window, "values": out})
}

/// Compute one percentile from a numeric array using linear interpolation.
pub fn algo_percentile_work(args: Value) -> Value {
    let mut values = match number_array(&args, "values") {
        Ok(v) => v,
        Err(e) => return json!({"error": e}),
    };
    if values.is_empty() {
        return json!({"error": "values must not be empty"});
    }
    let percentile = args
        .get("percentile")
        .and_then(|v| v.as_f64())
        .unwrap_or(50.0)
        .clamp(0.0, 100.0);
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let rank = (percentile / 100.0) * (values.len().saturating_sub(1) as f64);
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    let frac = rank - lo as f64;
    let value = values[lo] * (1.0 - frac) + values[hi] * frac;
    json!({"percentile": percentile, "value": value, "count": values.len()})
}

/// Build a numeric histogram from a bounded numeric array.
pub fn algo_numeric_histogram_work(args: Value) -> Value {
    let values = match number_array(&args, "values") {
        Ok(v) => v,
        Err(e) => return json!({"error": e}),
    };
    if values.is_empty() {
        return json!({"error": "values must not be empty"});
    }
    let bins = args
        .get("bins")
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(10)
        .clamp(1, 10_000);
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let width = if (max - min).abs() < f64::EPSILON {
        1.0
    } else {
        (max - min) / bins as f64
    };
    let mut counts = vec![0usize; bins];
    for value in values {
        let idx = if (max - min).abs() < f64::EPSILON {
            0
        } else {
            (((value - min) / width).floor() as usize).min(bins - 1)
        };
        counts[idx] += 1;
    }
    json!({"min": min, "max": max, "bins": bins, "counts": counts})
}

/// Select top-k frequent items or largest/smallest numeric values.
pub fn algo_top_k_work(args: Value) -> Value {
    let k = args
        .get("k")
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(10)
        .clamp(1, 10_000);
    let mode = args
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("frequency")
        .to_ascii_lowercase();

    match mode.as_str() {
        "frequency" => {
            let items = match args.get("items").and_then(|v| v.as_array()) {
                Some(items) => items,
                None => return json!({"error": "items must be an array"}),
            };
            let key = args.get("key").and_then(|v| v.as_str());
            let mut counts: HashMap<String, usize> = HashMap::new();
            for item in items.iter().take(100_000) {
                let selected = key.and_then(|key| item.get(key)).unwrap_or(item);
                let item_key = selected
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| selected.to_string());
                *counts.entry(item_key).or_insert(0) += 1;
            }
            let mut entries: Vec<_> = counts.into_iter().collect();
            entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            json!({
                "mode": "frequency",
                "count": items.len(),
                "key": key,
                "entries": entries.into_iter().take(k).map(|(item, count)| {
                    json!({"item": item, "count": count})
                }).collect::<Vec<_>>()
            })
        }
        "largest" | "smallest" => {
            let mut values = match number_array(&args, "values") {
                Ok(values) => values,
                Err(e) => return json!({"error": e}),
            };
            if values.is_empty() {
                return json!({"error": "values must not be empty"});
            }
            values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            if mode == "largest" {
                values.reverse();
            }
            values.truncate(k);
            json!({"mode": mode, "values": values, "k": k})
        }
        other => {
            json!({"error": format!("unknown mode: {other}; use frequency, largest, or smallest")})
        }
    }
}

/// Detect numeric outliers using Tukey's IQR fences.
pub fn algo_outliers_iqr_work(args: Value) -> Value {
    let mut values = match number_array(&args, "values") {
        Ok(values) => values,
        Err(e) => return json!({"error": e}),
    };
    if values.len() < 4 {
        return json!({"error": "values must contain at least 4 numbers"});
    }
    let multiplier = args
        .get("multiplier")
        .and_then(|v| v.as_f64())
        .filter(|v| v.is_finite() && *v >= 0.0)
        .unwrap_or(1.5);
    let original = values.clone();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let q1 = percentile(&values, 25.0);
    let q3 = percentile(&values, 75.0);
    let iqr = q3 - q1;
    let lower = q1 - multiplier * iqr;
    let upper = q3 + multiplier * iqr;
    let outliers = original
        .into_iter()
        .enumerate()
        .filter(|(_, value)| *value < lower || *value > upper)
        .map(|(index, value)| json!({"index": index, "value": value}))
        .collect::<Vec<_>>();
    json!({
        "count": values.len(),
        "q1": q1,
        "q3": q3,
        "iqr": iqr,
        "lower_fence": lower,
        "upper_fence": upper,
        "outlier_count": outliers.len(),
        "outliers": outliers
    })
}

/// Build a confusion matrix and common classification metrics.
pub fn algo_confusion_matrix_work(args: Value) -> Value {
    let expected = match string_array(&args, "expected") {
        Ok(values) => values,
        Err(e) => return json!({"error": e}),
    };
    let predicted = match string_array(&args, "predicted") {
        Ok(values) => values,
        Err(e) => return json!({"error": e}),
    };
    if expected.len() != predicted.len() {
        return json!({"error": "expected and predicted must have the same length"});
    }
    if expected.is_empty() {
        return json!({"error": "expected/predicted must not be empty"});
    }

    let labels: Vec<String> = expected
        .iter()
        .chain(predicted.iter())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let label_index: HashMap<String, usize> = labels
        .iter()
        .enumerate()
        .map(|(i, label)| (label.clone(), i))
        .collect();
    let mut matrix = vec![vec![0usize; labels.len()]; labels.len()];
    let mut correct = 0usize;
    for (exp, pred) in expected.iter().zip(predicted.iter()) {
        let row = label_index[exp];
        let col = label_index[pred];
        matrix[row][col] += 1;
        if exp == pred {
            correct += 1;
        }
    }

    let per_label = labels
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let tp = matrix[i][i] as f64;
            let fp: usize = (0..labels.len())
                .filter(|&row| row != i)
                .map(|row| matrix[row][i])
                .sum();
            let fn_: usize = (0..labels.len())
                .filter(|&col| col != i)
                .map(|col| matrix[i][col])
                .sum();
            let precision = if tp + fp as f64 == 0.0 {
                0.0
            } else {
                tp / (tp + fp as f64)
            };
            let recall = if tp + fn_ as f64 == 0.0 {
                0.0
            } else {
                tp / (tp + fn_ as f64)
            };
            let f1 = if precision + recall == 0.0 {
                0.0
            } else {
                2.0 * precision * recall / (precision + recall)
            };
            json!({
                "label": label,
                "precision": precision,
                "recall": recall,
                "f1": f1,
                "support": matrix[i].iter().sum::<usize>()
            })
        })
        .collect::<Vec<_>>();
    let macro_precision = per_label
        .iter()
        .filter_map(|v| v.get("precision").and_then(|v| v.as_f64()))
        .sum::<f64>()
        / per_label.len() as f64;
    let macro_recall = per_label
        .iter()
        .filter_map(|v| v.get("recall").and_then(|v| v.as_f64()))
        .sum::<f64>()
        / per_label.len() as f64;
    let macro_f1 = per_label
        .iter()
        .filter_map(|v| v.get("f1").and_then(|v| v.as_f64()))
        .sum::<f64>()
        / per_label.len() as f64;
    let weighted_f1 = per_label
        .iter()
        .map(|v| {
            let support = v.get("support").and_then(|v| v.as_u64()).unwrap_or(0) as f64;
            let f1 = v.get("f1").and_then(|v| v.as_f64()).unwrap_or(0.0);
            support * f1
        })
        .sum::<f64>()
        / expected.len() as f64;

    json!({
        "labels": labels,
        "matrix": matrix,
        "accuracy": correct as f64 / expected.len() as f64,
        "total": expected.len(),
        "macro_avg": {
            "precision": macro_precision,
            "recall": macro_recall,
            "f1": macro_f1
        },
        "weighted_avg": {
            "f1": weighted_f1
        },
        "per_label": per_label
    })
}

/// Compute cosine similarity for numeric vectors or text bag-of-words vectors.
pub fn algo_cosine_similarity_work(args: Value) -> Value {
    if args.get("a").is_some() || args.get("b").is_some() {
        let a = match number_array(&args, "a") {
            Ok(values) => values,
            Err(e) => return json!({"error": e}),
        };
        let b = match number_array(&args, "b") {
            Ok(values) => values,
            Err(e) => return json!({"error": e}),
        };
        return cosine_from_vectors(&a, &b)
            .map(|similarity| json!({"mode": "vector", "similarity": similarity, "dimensions": a.len()}))
            .unwrap_or_else(|e| json!({"error": e}));
    }

    let text_a = match args.get("text_a").and_then(|v| v.as_str()) {
        Some(text) => text,
        None => return json!({"error": "provide either numeric arrays a/b or text_a/text_b"}),
    };
    let text_b = match args.get("text_b").and_then(|v| v.as_str()) {
        Some(text) => text,
        None => return json!({"error": "text_b is required"}),
    };
    let vocab: BTreeSet<String> = word_counts(text_a)
        .keys()
        .chain(word_counts(text_b).keys())
        .cloned()
        .collect();
    let counts_a = word_counts(text_a);
    let counts_b = word_counts(text_b);
    let a = vocab
        .iter()
        .map(|token| *counts_a.get(token).unwrap_or(&0) as f64)
        .collect::<Vec<_>>();
    let b = vocab
        .iter()
        .map(|token| *counts_b.get(token).unwrap_or(&0) as f64)
        .collect::<Vec<_>>();
    cosine_from_vectors(&a, &b)
        .map(|similarity| json!({"mode": "text_bow", "similarity": similarity, "terms": vocab.len()}))
        .unwrap_or_else(|e| json!({"error": e}))
}

/// Create a deterministic MinHash-style signature from text shingles.
pub fn algo_minhash_signature_work(args: Value) -> Value {
    let text = match args.get("text").and_then(|v| v.as_str()) {
        Some(text) => text,
        None => return json!({"error": "text is required"}),
    };
    let ngram = args
        .get("ngram")
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(3)
        .clamp(1, 16);
    let hashes = args
        .get("hashes")
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(64)
        .clamp(1, 256);
    let shingles = text_shingles(text, ngram);
    if shingles.is_empty() {
        return json!({"error": "text does not contain enough tokens/chars for requested ngram"});
    }
    let signature = (0..hashes)
        .map(|seed| {
            shingles
                .iter()
                .map(|shingle| seeded_hash(shingle, seed as u64))
                .min()
                .unwrap_or(0)
                .to_string()
        })
        .collect::<Vec<_>>();
    json!({
        "ngram": ngram,
        "hashes": hashes,
        "shingle_count": shingles.len(),
        "signature": signature
    })
}

/// Schedule tasks with dependencies using earliest-start times.
pub fn algo_task_schedule_work(args: Value) -> Value {
    let tasks = match args.get("tasks").and_then(|v| v.as_array()) {
        Some(tasks) => tasks,
        None => return json!({"error": "tasks must be an array"}),
    };
    if tasks.len() > 10_000 {
        return json!({"error": "tasks is capped at 10000 items"});
    }

    let mut durations: HashMap<String, f64> = HashMap::new();
    let mut deps: HashMap<String, Vec<String>> = HashMap::new();
    let mut dependents: HashMap<String, Vec<String>> = HashMap::new();
    for task in tasks {
        let id = match task.get("id").and_then(|v| v.as_str()) {
            Some(id) if !id.trim().is_empty() => id.trim().to_string(),
            _ => return json!({"error": "each task needs a non-empty id"}),
        };
        if durations.contains_key(&id) {
            return json!({"error": format!("duplicate task id: {id}")});
        }
        let duration = task
            .get("duration")
            .and_then(|v| v.as_f64())
            .filter(|v| v.is_finite() && *v >= 0.0)
            .unwrap_or(1.0);
        let task_deps = task
            .get("depends_on")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str())
            .map(str::to_string)
            .collect::<Vec<_>>();
        durations.insert(id.clone(), duration);
        deps.insert(id.clone(), task_deps);
    }
    for (id, task_deps) in &deps {
        for dep in task_deps {
            if !durations.contains_key(dep) {
                return json!({"error": format!("task {id} depends on unknown task {dep}")});
            }
            dependents.entry(dep.clone()).or_default().push(id.clone());
        }
    }

    let mut indegree: HashMap<String, usize> = deps
        .iter()
        .map(|(id, task_deps)| (id.clone(), task_deps.len()))
        .collect();
    let mut ready = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    ready.sort();
    let mut queue: VecDeque<_> = ready.into();
    let mut earliest_start: HashMap<String, f64> = HashMap::new();
    let mut ordered = Vec::new();

    while let Some(id) = queue.pop_front() {
        let start = *earliest_start.get(&id).unwrap_or(&0.0);
        let end = start + durations[&id];
        ordered.push(json!({
            "id": id,
            "duration": durations[&id],
            "earliest_start": start,
            "earliest_finish": end
        }));
        if let Some(nexts) = dependents.get(&id) {
            let mut sorted_nexts = nexts.clone();
            sorted_nexts.sort();
            for next in sorted_nexts {
                let entry = earliest_start.entry(next.clone()).or_insert(0.0);
                *entry = (*entry).max(end);
                if let Some(degree) = indegree.get_mut(&next) {
                    *degree = degree.saturating_sub(1);
                    if *degree == 0 {
                        queue.push_back(next);
                    }
                }
            }
        }
    }

    if ordered.len() != durations.len() {
        return json!({"error": "dependency cycle detected"});
    }
    let makespan = ordered
        .iter()
        .filter_map(|task| task.get("earliest_finish").and_then(|v| v.as_f64()))
        .fold(0.0, f64::max);
    json!({"task_count": ordered.len(), "makespan": makespan, "schedule": ordered})
}

/// Normalize numeric scores with min-max, z-score, or robust median/IQR scaling.
pub fn algo_normalize_scores_work(args: Value) -> Value {
    let values = match number_array(&args, "values") {
        Ok(values) => values,
        Err(e) => return json!({"error": e}),
    };
    if values.is_empty() {
        return json!({"error": "values must not be empty"});
    }
    let method = args
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("minmax")
        .to_ascii_lowercase();
    let normalized = match method.as_str() {
        "minmax" => {
            let min = values.iter().copied().fold(f64::INFINITY, f64::min);
            let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let range = max - min;
            if range.abs() < f64::EPSILON {
                vec![0.0; values.len()]
            } else {
                values.iter().map(|v| (v - min) / range).collect()
            }
        }
        "zscore" => {
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let variance =
                values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
            let std = variance.sqrt();
            if std.abs() < f64::EPSILON {
                vec![0.0; values.len()]
            } else {
                values.iter().map(|v| (v - mean) / std).collect()
            }
        }
        "robust" => {
            let mut sorted = values.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let median = percentile(&sorted, 50.0);
            let iqr = percentile(&sorted, 75.0) - percentile(&sorted, 25.0);
            if iqr.abs() < f64::EPSILON {
                vec![0.0; values.len()]
            } else {
                values.iter().map(|v| (v - median) / iqr).collect()
            }
        }
        other => {
            return json!({"error": format!("unknown method: {other}; use minmax, zscore, or robust")});
        }
    };
    json!({"method": method, "values": normalized})
}

/// Compute rolling window statistics over a numeric array.
pub fn algo_rolling_stats_work(args: Value) -> Value {
    let values = match number_array(&args, "values") {
        Ok(values) => values,
        Err(e) => return json!({"error": e}),
    };
    if values.is_empty() {
        return json!({"error": "values must not be empty"});
    }
    let window = args
        .get("window")
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(5)
        .clamp(1, values.len());
    let mut rows = Vec::with_capacity(values.len());
    for end in 0..values.len() {
        let start = (end + 1).saturating_sub(window);
        let slice = &values[start..=end];
        let mean = slice.iter().sum::<f64>() / slice.len() as f64;
        let min = slice.iter().copied().fold(f64::INFINITY, f64::min);
        let max = slice.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let variance = slice.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / slice.len() as f64;
        rows.push(json!({
            "index": end,
            "start": start,
            "end": end,
            "mean": mean,
            "min": min,
            "max": max,
            "std_dev": variance.sqrt()
        }));
    }
    json!({"window": window, "rows": rows})
}

/// Deterministically split labelled examples while preserving label proportions.
pub fn algo_stratified_split_work(args: Value) -> Value {
    let labels = match string_array(&args, "labels") {
        Ok(labels) => labels,
        Err(e) => return json!({"error": e}),
    };
    if labels.is_empty() {
        return json!({"error": "labels must not be empty"});
    }
    let validation_ratio = args
        .get("validation_ratio")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.2)
        .clamp(0.0, 1.0);
    let seed = args.get("seed").and_then(|v| v.as_u64()).unwrap_or(0);
    let mut by_label: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, label) in labels.iter().enumerate() {
        by_label.entry(label.clone()).or_default().push(index);
    }
    let mut train = Vec::new();
    let mut validation = Vec::new();
    let mut label_counts = Vec::new();
    for (label, mut indices) in by_label {
        indices.sort_by_key(|index| seeded_hash(&format!("{label}:{index}"), seed));
        let val_count = ((indices.len() as f64) * validation_ratio)
            .round()
            .min(indices.len() as f64) as usize;
        validation.extend(indices.iter().take(val_count).copied());
        train.extend(indices.iter().skip(val_count).copied());
        label_counts.push(json!({
            "label": label,
            "total": indices.len(),
            "train": indices.len().saturating_sub(val_count),
            "validation": val_count
        }));
    }
    train.sort_unstable();
    validation.sort_unstable();
    label_counts.sort_by(|a, b| {
        a["label"]
            .as_str()
            .unwrap_or("")
            .cmp(b["label"].as_str().unwrap_or(""))
    });
    json!({
        "train_indices": train,
        "validation_indices": validation,
        "validation_ratio": validation_ratio,
        "label_counts": label_counts
    })
}

/// Estimate Jaccard similarity from MinHash signatures, or exact shingles for text.
pub fn algo_minhash_similarity_work(args: Value) -> Value {
    if let (Some(a), Some(b)) = (
        args.get("signature_a").and_then(|v| v.as_array()),
        args.get("signature_b").and_then(|v| v.as_array()),
    ) {
        if a.len() != b.len() || a.is_empty() {
            return json!({"error": "signature_a and signature_b must be non-empty arrays with equal length"});
        }
        let matches = a.iter().zip(b.iter()).filter(|(x, y)| x == y).count();
        return json!({
            "mode": "signature",
            "similarity": matches as f64 / a.len() as f64,
            "matches": matches,
            "hashes": a.len()
        });
    }

    let text_a = match args.get("text_a").and_then(|v| v.as_str()) {
        Some(text) => text,
        None => return json!({"error": "provide signature_a/signature_b or text_a/text_b"}),
    };
    let text_b = match args.get("text_b").and_then(|v| v.as_str()) {
        Some(text) => text,
        None => return json!({"error": "text_b is required"}),
    };
    let ngram = args
        .get("ngram")
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(3)
        .clamp(1, 16);
    let a = text_shingles(text_a, ngram)
        .into_iter()
        .collect::<HashSet<_>>();
    let b = text_shingles(text_b, ngram)
        .into_iter()
        .collect::<HashSet<_>>();
    if a.is_empty() && b.is_empty() {
        return json!({"mode": "exact_shingle", "similarity": 1.0, "ngram": ngram});
    }
    let intersection = a.intersection(&b).count();
    let union = a.union(&b).count();
    json!({
        "mode": "exact_shingle",
        "similarity": intersection as f64 / union as f64,
        "intersection": intersection,
        "union": union,
        "ngram": ngram
    })
}

/// Compute ROC-AUC and a fixed-threshold confusion summary for binary scores.
pub fn algo_roc_auc_work(args: Value) -> Value {
    let labels = match args.get("labels").and_then(|v| v.as_array()) {
        Some(labels) => labels,
        None => return json!({"error": "labels must be an array"}),
    };
    let scores = match number_array(&args, "scores") {
        Ok(scores) => scores,
        Err(e) => return json!({"error": e}),
    };
    if labels.len() != scores.len() || labels.is_empty() {
        return json!({"error": "labels and scores must be non-empty arrays with equal length"});
    }
    let positive_label = args
        .get("positive_label")
        .and_then(|v| v.as_str())
        .unwrap_or("1");
    let positives = labels
        .iter()
        .map(|label| label_is_positive(label, positive_label))
        .collect::<Vec<_>>();
    let pos_count = positives.iter().filter(|v| **v).count();
    let neg_count = positives.len().saturating_sub(pos_count);
    if pos_count == 0 || neg_count == 0 {
        return json!({"error": "labels must contain at least one positive and one negative example"});
    }

    let mut ranked = scores
        .iter()
        .copied()
        .zip(positives.iter().copied())
        .collect::<Vec<_>>();
    ranked.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut rank_sum_pos = 0.0;
    let mut i = 0;
    while i < ranked.len() {
        let mut j = i + 1;
        while j < ranked.len() && (ranked[j].0 - ranked[i].0).abs() < f64::EPSILON {
            j += 1;
        }
        let avg_rank = ((i + 1) as f64 + j as f64) / 2.0;
        for item in ranked.iter().take(j).skip(i) {
            if item.1 {
                rank_sum_pos += avg_rank;
            }
        }
        i = j;
    }
    let pos = pos_count as f64;
    let neg = neg_count as f64;
    let auc = (rank_sum_pos - (pos * (pos + 1.0) / 2.0)) / (pos * neg);
    let threshold = args
        .get("threshold")
        .and_then(|v| v.as_f64())
        .filter(|v| v.is_finite())
        .unwrap_or(0.5);
    let mut tp = 0usize;
    let mut fp = 0usize;
    let mut tn = 0usize;
    let mut fn_ = 0usize;
    for (score, positive) in scores.iter().zip(positives.iter()) {
        match (*score >= threshold, *positive) {
            (true, true) => tp += 1,
            (true, false) => fp += 1,
            (false, true) => fn_ += 1,
            (false, false) => tn += 1,
        }
    }
    let precision = if tp + fp == 0 {
        0.0
    } else {
        tp as f64 / (tp + fp) as f64
    };
    let recall = if tp + fn_ == 0 {
        0.0
    } else {
        tp as f64 / (tp + fn_) as f64
    };
    json!({
        "auc": round_f64(auc),
        "positive_count": pos_count,
        "negative_count": neg_count,
        "threshold": threshold,
        "threshold_metrics": {
            "tp": tp,
            "fp": fp,
            "tn": tn,
            "fn": fn_,
            "precision": round_f64(precision),
            "recall": round_f64(recall),
        }
    })
}

/// Cluster numeric points with deterministic k-means.
pub fn algo_kmeans_work(args: Value) -> Value {
    let points = match number_matrix(&args, "points") {
        Ok(points) => points,
        Err(e) => return json!({"error": e}),
    };
    if points.is_empty() {
        return json!({"error": "points must not be empty"});
    }
    let dims = points[0].len();
    if dims == 0 {
        return json!({"error": "points must have at least one dimension"});
    }
    let k = args
        .get("k")
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(2)
        .clamp(1, points.len().min(64));
    let max_iter = args
        .get("max_iter")
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(50)
        .clamp(1, 500);
    let mut centroids = points.iter().take(k).cloned().collect::<Vec<_>>();
    let mut assignments = vec![0usize; points.len()];
    let mut iterations = 0usize;
    for iter in 0..max_iter {
        iterations = iter + 1;
        let mut changed = false;
        for (index, point) in points.iter().enumerate() {
            let cluster = nearest_centroid(point, &centroids);
            if assignments[index] != cluster {
                changed = true;
                assignments[index] = cluster;
            }
        }
        let mut sums = vec![vec![0.0; dims]; k];
        let mut counts = vec![0usize; k];
        for (point, cluster) in points.iter().zip(assignments.iter()) {
            counts[*cluster] += 1;
            for (dim, value) in point.iter().enumerate() {
                sums[*cluster][dim] += value;
            }
        }
        for cluster in 0..k {
            if counts[cluster] == 0 {
                continue;
            }
            for value in sums[cluster].iter_mut().take(dims) {
                *value /= counts[cluster] as f64;
            }
            centroids[cluster] = sums[cluster].clone();
        }
        if !changed && iter > 0 {
            break;
        }
    }
    let mut inertia = 0.0;
    for (point, cluster) in points.iter().zip(assignments.iter()) {
        inertia += squared_distance(point, &centroids[*cluster]);
    }
    json!({
        "k": k,
        "iterations": iterations,
        "assignments": assignments,
        "centroids": centroids
            .iter()
            .map(|row| row.iter().map(|v| round_f64(*v)).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        "inertia": round_f64(inertia)
    })
}

/// Chunk text into deterministic overlapping character windows for dataset prep.
pub fn algo_text_chunks_work(args: Value) -> Value {
    let text = match args.get("text").and_then(|v| v.as_str()) {
        Some(text) => text,
        None => return json!({"error": "text must be a string"}),
    };
    let chars = text.chars().collect::<Vec<_>>();
    let chunk_size = args
        .get("chunk_size")
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(1000)
        .clamp(1, 100_000);
    let overlap = args
        .get("overlap")
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(0)
        .min(chunk_size.saturating_sub(1));
    let max_chunks = args
        .get("max_chunks")
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(10_000)
        .clamp(1, 10_000);
    let step = chunk_size.saturating_sub(overlap).max(1);
    let mut chunks = Vec::new();
    let mut start = 0usize;
    while start < chars.len() && chunks.len() < max_chunks {
        let end = (start + chunk_size).min(chars.len());
        let value = chars[start..end].iter().collect::<String>();
        chunks.push(json!({
            "index": chunks.len(),
            "start": start,
            "end": end,
            "chars": end.saturating_sub(start),
            "text": value
        }));
        if end == chars.len() {
            break;
        }
        start = start.saturating_add(step);
    }
    json!({
        "chunk_size": chunk_size,
        "overlap": overlap,
        "input_chars": chars.len(),
        "chunk_count": chunks.len(),
        "truncated": start < chars.len(),
        "chunks": chunks
    })
}

/// Report basic dataset quality for text records without echoing duplicate text.
pub fn algo_dataset_quality_work(args: Value) -> Value {
    let texts = match text_records(&args) {
        Ok(texts) => texts,
        Err(e) => return json!({"error": e}),
    };
    if texts.is_empty() {
        return json!({"error": "records must not be empty"});
    }
    let mut lengths = Vec::with_capacity(texts.len());
    let mut empty_count = 0usize;
    let mut secret_like_count = 0usize;
    let mut by_hash: HashMap<u64, Vec<usize>> = HashMap::new();
    for (index, text) in texts.iter().enumerate() {
        let len = text.chars().count();
        lengths.push(len as f64);
        if text.trim().is_empty() {
            empty_count += 1;
        }
        if looks_secret_like(text) {
            secret_like_count += 1;
        }
        by_hash.entry(seeded_hash(text, 0)).or_default().push(index);
    }
    let duplicate_groups = by_hash
        .into_iter()
        .filter(|(_, indices)| indices.len() > 1)
        .map(|(hash, indices)| {
            json!({
                "hash": format!("{hash:016x}"),
                "count": indices.len(),
                "sample_indices": indices.into_iter().take(5).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    let duplicate_record_count = duplicate_groups
        .iter()
        .filter_map(|group| group.get("count").and_then(|v| v.as_u64()))
        .map(|count| count.saturating_sub(1))
        .sum::<u64>();
    let unique_count = texts.len().saturating_sub(duplicate_record_count as usize);
    let duplicate_ratio = if texts.is_empty() {
        0.0
    } else {
        duplicate_record_count as f64 / texts.len() as f64
    };
    let mut sorted = lengths.clone();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let mean_len = lengths.iter().sum::<f64>() / lengths.len() as f64;
    json!({
        "record_count": texts.len(),
        "empty_count": empty_count,
        "secret_like_count": secret_like_count,
        "unique_count": unique_count,
        "duplicate_group_count": duplicate_groups.len(),
        "duplicate_record_count": duplicate_record_count,
        "duplicate_ratio": round_f64(duplicate_ratio),
        "lengths": {
            "min": round_f64(sorted[0]),
            "mean": round_f64(mean_len),
            "median": round_f64(percentile(&sorted, 50.0)),
            "p95": round_f64(percentile(&sorted, 95.0)),
            "max": round_f64(*sorted.last().unwrap_or(&0.0))
        },
        "duplicate_groups": duplicate_groups
    })
}

/// Summarize label distribution for classification datasets.
pub fn algo_label_distribution_work(args: Value) -> Value {
    let labels = match string_array(&args, "labels") {
        Ok(labels) => labels,
        Err(e) => return json!({"error": e}),
    };
    if labels.is_empty() {
        return json!({"error": "labels must not be empty"});
    }
    let mut counts: HashMap<String, usize> = HashMap::new();
    for label in labels {
        *counts.entry(label).or_default() += 1;
    }
    let total = counts.values().sum::<usize>();
    let mut rows = counts
        .iter()
        .map(|(label, count)| {
            json!({
                "label": label,
                "count": count,
                "proportion": round_f64(*count as f64 / total as f64)
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b["count"].as_u64().cmp(&a["count"].as_u64()).then_with(|| {
            a["label"]
                .as_str()
                .unwrap_or("")
                .cmp(b["label"].as_str().unwrap_or(""))
        })
    });
    let majority = rows.first().cloned().unwrap_or_else(|| json!({}));
    let minority = rows.last().cloned().unwrap_or_else(|| json!({}));
    let majority_count = majority.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
    let minority_count = minority.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
    let entropy = counts
        .values()
        .map(|count| {
            let p = *count as f64 / total as f64;
            -p * p.log2()
        })
        .sum::<f64>();
    json!({
        "total": total,
        "unique_labels": rows.len(),
        "labels": rows,
        "majority": majority,
        "minority": minority,
        "imbalance_ratio": if minority_count == 0 {
            Value::Null
        } else {
            json!(round_f64(majority_count as f64 / minority_count as f64))
        },
        "entropy_bits": round_f64(entropy)
    })
}

/// Estimate a deterministic bootstrap confidence interval for the mean.
pub fn algo_bootstrap_mean_ci_work(args: Value) -> Value {
    let values = match number_array(&args, "values") {
        Ok(values) => values,
        Err(e) => return json!({"error": e}),
    };
    if values.is_empty() {
        return json!({"error": "values must not be empty"});
    }
    let samples = args
        .get("samples")
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(1000)
        .clamp(10, 10_000);
    let confidence = args
        .get("confidence")
        .and_then(|v| v.as_f64())
        .filter(|v| v.is_finite())
        .unwrap_or(0.95)
        .clamp(0.5, 0.999);
    let seed = args.get("seed").and_then(|v| v.as_u64()).unwrap_or(0);
    let observed_mean = values.iter().sum::<f64>() / values.len() as f64;
    let mut means = Vec::with_capacity(samples);
    for sample in 0..samples {
        let mut sum = 0.0;
        for draw in 0..values.len() {
            let index = (seeded_hash(&format!("{sample}:{draw}"), seed) as usize) % values.len();
            sum += values[index];
        }
        means.push(sum / values.len() as f64);
    }
    means.sort_by(|a, b| a.total_cmp(b));
    let tail = (1.0 - confidence) / 2.0;
    json!({
        "mean": round_f64(observed_mean),
        "confidence": confidence,
        "samples": samples,
        "ci": {
            "lower": round_f64(percentile(&means, tail * 100.0)),
            "upper": round_f64(percentile(&means, (1.0 - tail) * 100.0))
        }
    })
}

fn string_array(args: &Value, key: &str) -> Result<Vec<String>, String> {
    let arr = args
        .get(key)
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("{key} must be an array of strings"))?;
    if arr.len() > 100_000 {
        return Err(format!("{key} is capped at 100000 values"));
    }
    arr.iter()
        .map(|v| {
            v.as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("{key} must contain only strings"))
        })
        .collect()
}

fn label_is_positive(label: &Value, positive_label: &str) -> bool {
    if let Some(value) = label.as_bool() {
        return value;
    }
    if let Some(value) = label.as_i64() {
        return value == 1;
    }
    if let Some(value) = label.as_u64() {
        return value == 1;
    }
    if let Some(value) = label.as_str() {
        return value == positive_label
            || matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "positive" | "pos"
            );
    }
    false
}

fn number_matrix(args: &Value, key: &str) -> Result<Vec<Vec<f64>>, String> {
    let rows = args
        .get(key)
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("{key} must be an array of numeric arrays"))?;
    if rows.len() > 10_000 {
        return Err(format!("{key} is capped at 10000 rows"));
    }
    let mut out = Vec::with_capacity(rows.len());
    let mut expected_dims = None;
    for row in rows {
        let values = row
            .as_array()
            .ok_or_else(|| format!("{key} must contain only numeric arrays"))?;
        if values.len() > 256 {
            return Err(format!("{key} rows are capped at 256 dimensions"));
        }
        if let Some(dims) = expected_dims {
            if values.len() != dims {
                return Err(format!("{key} rows must all have the same length"));
            }
        } else {
            expected_dims = Some(values.len());
        }
        out.push(
            values
                .iter()
                .map(|v| {
                    v.as_f64()
                        .filter(|f| f.is_finite())
                        .ok_or_else(|| format!("{key} must contain only finite numbers"))
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
    }
    Ok(out)
}

fn nearest_centroid(point: &[f64], centroids: &[Vec<f64>]) -> usize {
    centroids
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| squared_distance(point, a).total_cmp(&squared_distance(point, b)))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn squared_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum()
}

fn text_records(args: &Value) -> Result<Vec<String>, String> {
    let records = args
        .get("records")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "records must be an array of strings or objects".to_string())?;
    if records.len() > 100_000 {
        return Err("records is capped at 100000 values".to_string());
    }
    let text_key = args
        .get("text_key")
        .and_then(|v| v.as_str())
        .unwrap_or("text");
    records
        .iter()
        .map(|record| {
            if let Some(text) = record.as_str() {
                Ok(text.to_string())
            } else {
                record
                    .get(text_key)
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
                    .ok_or_else(|| {
                        format!("record must be a string or object with string field {text_key}")
                    })
            }
        })
        .collect()
}

fn looks_secret_like(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("password=")
        || lower.contains("api_key")
        || lower.contains("secret")
        || lower.contains("token=")
        || text.contains("sk-")
        || text.contains("-----BEGIN ")
}

fn cosine_from_vectors(a: &[f64], b: &[f64]) -> Result<f64, String> {
    if a.len() != b.len() {
        return Err("vectors must have the same length".to_string());
    }
    if a.is_empty() {
        return Err("vectors must not be empty".to_string());
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a = a.iter().map(|v| v * v).sum::<f64>().sqrt();
    let norm_b = b.iter().map(|v| v * v).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return Err("cosine similarity is undefined for zero vectors".to_string());
    }
    Ok(dot / (norm_a * norm_b))
}

fn word_counts(text: &str) -> HashMap<String, usize> {
    text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_ascii_lowercase())
        .fold(HashMap::new(), |mut counts, token| {
            *counts.entry(token).or_insert(0) += 1;
            counts
        })
}

fn text_shingles(text: &str, ngram: usize) -> Vec<String> {
    let chars = text.chars().collect::<Vec<_>>();
    if chars.len() < ngram {
        return Vec::new();
    }
    chars
        .windows(ngram)
        .take(100_000)
        .map(|window| window.iter().collect::<String>())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn seeded_hash(text: &str, seed: u64) -> u64 {
    // Deterministic FNV-1a style hash mixed with a seed.
    let mut hash = 0xcbf29ce484222325u64 ^ seed.wrapping_mul(0x9e3779b97f4a7c15);
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- algo_sort_work ---
    #[test]
    fn test_sort_numbers_ascending() {
        let result = algo_sort_work(json!({"items": [3, 1, 2]}));
        assert_eq!(result["sorted"], json!([1, 2, 3]));
        assert_eq!(result["reverse"], false);
    }

    #[test]
    fn test_sort_numbers_descending() {
        let result = algo_sort_work(json!({"items": [3, 1, 2], "reverse": true}));
        assert_eq!(result["sorted"], json!([3, 2, 1]));
    }

    #[test]
    fn test_sort_objects_by_key() {
        let result = algo_sort_work(json!({
            "items": [{"name": "c", "v": 3}, {"name": "a", "v": 1}, {"name": "b", "v": 2}],
            "key": "v"
        }));
        let sorted = result["sorted"].as_array().unwrap();
        assert_eq!(sorted[0]["name"], "a");
        assert_eq!(sorted[2]["name"], "c");
    }

    #[test]
    fn test_sort_empty_array() {
        let result = algo_sort_work(json!({"items": []}));
        assert_eq!(result["sorted"], json!([]));
    }

    #[test]
    fn test_sort_error_no_items() {
        let result = algo_sort_work(json!({}));
        assert!(result["error"].is_string());
    }

    // --- algo_search_work ---
    #[test]
    fn test_search_linear_found() {
        let result =
            algo_search_work(json!({"items": [10, 20, 30], "target": 20, "method": "linear"}));
        assert_eq!(result["found"], true);
        assert_eq!(result["index"], 1);
    }

    #[test]
    fn test_search_linear_not_found() {
        let result =
            algo_search_work(json!({"items": [10, 20, 30], "target": 99, "method": "linear"}));
        assert_eq!(result["found"], false);
    }

    #[test]
    fn test_search_binary_found() {
        let result =
            algo_search_work(json!({"items": [1, 2, 3, 4, 5], "target": 3, "method": "binary"}));
        assert_eq!(result["found"], true);
        assert_eq!(result["index"], 2);
    }

    #[test]
    fn test_search_binary_not_found() {
        let result =
            algo_search_work(json!({"items": [1, 2, 3, 4, 5], "target": 6, "method": "binary"}));
        assert_eq!(result["found"], false);
    }

    #[test]
    fn test_search_error_no_items() {
        let result = algo_search_work(json!({"target": 1}));
        assert!(result["error"].is_string());
    }

    // --- algo_graph_bfs_work ---
    #[test]
    fn test_bfs_simple() {
        let result = algo_graph_bfs_work(json!({
            "graph": {"A": ["B", "C"], "B": ["D"], "C": ["D"], "D": []},
            "start": "A"
        }));
        let visited = result["visited"].as_array().unwrap();
        assert_eq!(visited[0], "A");
        assert!(visited.contains(&json!("D")));
        assert_eq!(visited.len(), 4);
    }

    #[test]
    fn test_bfs_single_node() {
        let result = algo_graph_bfs_work(json!({"graph": {"X": []}, "start": "X"}));
        assert_eq!(result["visited"], json!(["X"]));
    }

    #[test]
    fn test_bfs_disconnected() {
        let result = algo_graph_bfs_work(json!({
            "graph": {"A": ["B"], "B": [], "C": []},
            "start": "A"
        }));
        assert_eq!(result["count"], 2);
    }

    #[test]
    fn test_bfs_error_missing_start() {
        let result = algo_graph_bfs_work(json!({"graph": {"A": []}}));
        assert!(result["error"].is_string());
    }

    // --- algo_graph_dfs_work ---
    #[test]
    fn test_dfs_simple() {
        let result = algo_graph_dfs_work(json!({
            "graph": {"A": ["B", "C"], "B": ["D"], "C": ["D"], "D": []},
            "start": "A"
        }));
        assert_eq!(result["count"], 4);
        let visited = result["visited"].as_array().unwrap();
        assert_eq!(visited[0], "A");
    }

    #[test]
    fn test_dfs_linear_chain() {
        let result = algo_graph_dfs_work(json!({
            "graph": {"1": ["2"], "2": ["3"], "3": []},
            "start": "1"
        }));
        assert_eq!(result["visited"], json!(["1", "2", "3"]));
    }

    // --- algo_graph_shortest_path_work ---
    #[test]
    fn test_shortest_path_simple() {
        let result = algo_graph_shortest_path_work(json!({
            "graph": {
                "A": {"B": 1.0, "C": 4.0},
                "B": {"C": 2.0, "D": 5.0},
                "C": {"D": 1.0},
                "D": {}
            },
            "start": "A",
            "end": "D"
        }));
        assert_eq!(result["path"], json!(["A", "B", "C", "D"]));
        assert!((result["distance"].as_f64().unwrap() - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_shortest_path_direct() {
        let result = algo_graph_shortest_path_work(json!({
            "graph": {"A": {"B": 3.0}, "B": {}},
            "start": "A",
            "end": "B"
        }));
        assert_eq!(result["path"], json!(["A", "B"]));
    }

    #[test]
    fn test_shortest_path_no_path() {
        let result = algo_graph_shortest_path_work(json!({
            "graph": {"A": {}, "B": {}},
            "start": "A",
            "end": "B"
        }));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_shortest_path_missing_end() {
        let result = algo_graph_shortest_path_work(json!({"graph": {"A": {}}, "start": "A"}));
        assert!(result["error"].is_string());
    }

    // --- algo_graph_topo_sort_work ---
    #[test]
    fn test_topo_sort_dag() {
        let result = algo_graph_topo_sort_work(json!({
            "graph": {"A": ["B", "C"], "B": ["D"], "C": ["D"], "D": []}
        }));
        let sorted = result["sorted"].as_array().unwrap();
        assert_eq!(sorted.len(), 4);
        // A must come before B and C; B and C must come before D
        let pos: HashMap<String, usize> = sorted
            .iter()
            .enumerate()
            .map(|(i, v)| (v.as_str().unwrap().to_string(), i))
            .collect();
        assert!(pos["A"] < pos["B"]);
        assert!(pos["A"] < pos["C"]);
        assert!(pos["B"] < pos["D"]);
        assert!(pos["C"] < pos["D"]);
    }

    #[test]
    fn test_topo_sort_cycle() {
        let result = algo_graph_topo_sort_work(json!({
            "graph": {"A": ["B"], "B": ["C"], "C": ["A"]}
        }));
        assert!(result["error"].is_string());
        assert!(result["error"].as_str().unwrap().contains("cycle"));
    }

    // --- algo_matrix_ops_work ---
    #[test]
    fn test_matrix_transpose() {
        let result = algo_matrix_ops_work(json!({
            "operation": "transpose",
            "a": [[1, 2, 3], [4, 5, 6]]
        }));
        assert_eq!(
            result["result"],
            json!([[1.0, 4.0], [2.0, 5.0], [3.0, 6.0]])
        );
    }

    #[test]
    fn test_matrix_multiply() {
        let result = algo_matrix_ops_work(json!({
            "operation": "multiply",
            "a": [[1, 2], [3, 4]],
            "b": [[5, 6], [7, 8]]
        }));
        assert_eq!(result["result"], json!([[19.0, 22.0], [43.0, 50.0]]));
    }

    #[test]
    fn test_matrix_determinant() {
        let result = algo_matrix_ops_work(json!({
            "operation": "determinant",
            "a": [[1, 2], [3, 4]]
        }));
        assert!((result["result"].as_f64().unwrap() - (-2.0)).abs() < 1e-6);
    }

    #[test]
    fn test_matrix_determinant_lu_larger_matrix() {
        let result = algo_matrix_ops_work(json!({
            "operation": "determinant",
            "a": [[3, 2, 0, 1], [4, 0, 1, 2], [3, 0, 2, 1], [9, 2, 3, 1]]
        }));
        assert!((result["result"].as_f64().unwrap() - 24.0).abs() < 1e-6);
    }

    #[test]
    fn test_matrix_inverse() {
        let result = algo_matrix_ops_work(json!({
            "operation": "inverse",
            "a": [[4, 7], [2, 6]]
        }));
        // Inverse of [[4,7],[2,6]] = [[0.6, -0.7], [-0.2, 0.4]]
        let inv = result["result"].as_array().unwrap();
        assert!((inv[0][0].as_f64().unwrap() - 0.6).abs() < 1e-6);
        assert!((inv[0][1].as_f64().unwrap() - (-0.7)).abs() < 1e-6);
        assert!((inv[1][0].as_f64().unwrap() - (-0.2)).abs() < 1e-6);
        assert!((inv[1][1].as_f64().unwrap() - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_matrix_inverse_singular() {
        let result = algo_matrix_ops_work(json!({
            "operation": "inverse",
            "a": [[1, 2], [2, 4]]
        }));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_matrix_error_no_b() {
        let result = algo_matrix_ops_work(json!({"operation": "multiply", "a": [[1]]}));
        assert!(result["error"].is_string());
    }

    // --- algo_stats_describe_work ---
    #[test]
    fn test_stats_describe_basic() {
        let result = algo_stats_describe_work(json!({"values": [2, 4, 4, 4, 5, 5, 7, 9]}));
        assert!((result["mean"].as_f64().unwrap() - 5.0).abs() < 1e-6);
        assert!((result["median"].as_f64().unwrap() - 4.5).abs() < 1e-6);
        assert!((result["min"].as_f64().unwrap() - 2.0).abs() < 1e-6);
        assert!((result["max"].as_f64().unwrap() - 9.0).abs() < 1e-6);
        assert_eq!(result["count"], 8);
    }

    #[test]
    fn test_stats_describe_single() {
        let result = algo_stats_describe_work(json!({"values": [42]}));
        assert!((result["mean"].as_f64().unwrap() - 42.0).abs() < 1e-6);
        assert!((result["std_dev"].as_f64().unwrap()).abs() < 1e-6);
    }

    #[test]
    fn test_stats_describe_empty() {
        let result = algo_stats_describe_work(json!({"values": []}));
        assert!(result["error"].is_string());
    }

    // --- algo_stats_correlation_work ---
    #[test]
    fn test_correlation_pearson_perfect() {
        let result = algo_stats_correlation_work(json!({
            "x": [1, 2, 3, 4, 5],
            "y": [2, 4, 6, 8, 10],
            "method": "pearson"
        }));
        assert!((result["correlation"].as_f64().unwrap() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_correlation_pearson_negative() {
        let result = algo_stats_correlation_work(json!({
            "x": [1, 2, 3, 4, 5],
            "y": [10, 8, 6, 4, 2],
            "method": "pearson"
        }));
        assert!((result["correlation"].as_f64().unwrap() - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_correlation_spearman() {
        let result = algo_stats_correlation_work(json!({
            "x": [1, 2, 3, 4, 5],
            "y": [5, 6, 7, 8, 7],
            "method": "spearman"
        }));
        assert!(result["correlation"].as_f64().unwrap().abs() > 0.0);
    }

    #[test]
    fn test_correlation_error_mismatched() {
        let result = algo_stats_correlation_work(json!({"x": [1, 2], "y": [1]}));
        assert!(result["error"].is_string());
    }

    // --- algo_stats_regression_work ---
    #[test]
    fn test_regression_perfect_line() {
        let result = algo_stats_regression_work(json!({
            "x": [1, 2, 3, 4, 5],
            "y": [3, 5, 7, 9, 11]
        }));
        assert!((result["slope"].as_f64().unwrap() - 2.0).abs() < 1e-6);
        assert!((result["intercept"].as_f64().unwrap() - 1.0).abs() < 1e-6);
        assert!((result["r_squared"].as_f64().unwrap() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_regression_error_zero_variance() {
        let result = algo_stats_regression_work(json!({
            "x": [5, 5, 5],
            "y": [1, 2, 3]
        }));
        assert!(result["error"].is_string());
    }

    // --- algo_fft_work ---
    #[test]
    fn test_fft_dc_signal() {
        let result = algo_fft_work(json!({"values": [1, 1, 1, 1, 1, 1, 1, 1]}));
        let bins = result["bins"].as_array().unwrap();
        // DC component (k=0) should have magnitude 1.0, others ~0
        assert!((bins[0]["magnitude"].as_f64().unwrap() - 1.0).abs() < 1e-6);
        for bin in bins.iter().skip(1) {
            assert!(bin["magnitude"].as_f64().unwrap().abs() < 1e-6);
        }
    }

    #[test]
    fn test_fft_length() {
        let zeros: Vec<Value> = vec![json!(0.0); 10];
        let result = algo_fft_work(json!({"values": zeros}));
        assert_eq!(result["n"], 10);
        assert_eq!(result["bins"].as_array().unwrap().len(), 5);
    }

    #[test]
    fn test_fft_error_empty() {
        let result = algo_fft_work(json!({"values": []}));
        assert!(result["error"].is_string());
    }

    // --- algo_hash_text_work ---
    #[test]
    fn test_hash_sha256_known() {
        let result = algo_hash_text_work(json!({"text": "", "algorithm": "sha256"}));
        assert_eq!(
            result["hex"],
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_hash_sha512_known() {
        let result = algo_hash_text_work(json!({"text": "", "algorithm": "sha512"}));
        let hex = result["hex"].as_str().unwrap();
        assert!(hex.starts_with("cf83e1357eefb8bd"));
    }

    #[test]
    fn test_hash_md5_length() {
        let result = algo_hash_text_work(json!({"text": "", "algorithm": "md5"}));
        // Custom MD5 implementation - just verify it returns a 32-char hex string
        assert!(result["hex"].is_string());
        assert_eq!(result["hex"].as_str().unwrap().len(), 32);
    }

    #[test]
    fn test_hash_blake3_fallback() {
        let result = algo_hash_text_work(json!({"text": "hello", "algorithm": "blake3"}));
        assert!(result["note"].is_string());
        assert_eq!(result["algorithm"], "blake3");
    }

    #[test]
    fn test_hash_error_no_text() {
        let result = algo_hash_text_work(json!({"algorithm": "sha256"}));
        assert!(result["error"].is_string());
    }

    // --- algo_edit_distance_work ---
    #[test]
    fn test_edit_distance_same() {
        let result = algo_edit_distance_work(json!({"a": "hello", "b": "hello"}));
        assert_eq!(result["distance"], 0);
    }

    #[test]
    fn test_edit_distance_kitten_sitting() {
        let result = algo_edit_distance_work(json!({"a": "kitten", "b": "sitting"}));
        assert_eq!(result["distance"], 3);
    }

    #[test]
    fn test_edit_distance_empty() {
        let result = algo_edit_distance_work(json!({"a": "", "b": "abc"}));
        assert_eq!(result["distance"], 3);
    }

    #[test]
    fn test_edit_distance_error_missing() {
        let result = algo_edit_distance_work(json!({"a": "test"}));
        assert!(result["error"].is_string());
    }

    // --- algo_ngram_similarity_work ---
    #[test]
    fn test_ngram_identical() {
        let result = algo_ngram_similarity_work(json!({"a": "hello", "b": "hello", "n": 2}));
        assert!((result["similarity"].as_f64().unwrap() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ngram_no_overlap() {
        let result = algo_ngram_similarity_work(json!({"a": "abc", "b": "xyz", "n": 2}));
        assert!((result["similarity"].as_f64().unwrap() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_ngram_partial_overlap() {
        let result = algo_ngram_similarity_work(json!({"a": "night", "b": "nacht", "n": 2}));
        let sim = result["similarity"].as_f64().unwrap();
        assert!(sim > 0.0 && sim < 1.0);
    }

    #[test]
    fn test_ngram_empty_strings() {
        let result = algo_ngram_similarity_work(json!({"a": "", "b": "", "n": 2}));
        assert!((result["similarity"].as_f64().unwrap() - 1.0).abs() < 1e-6);
    }

    // --- algo_bloom_filter_work ---
    #[test]
    fn test_bloom_build_and_query_present() {
        let result_build = algo_bloom_filter_work(json!({
            "operation": "build",
            "items": ["foo", "bar", "baz"],
            "size": 1000,
            "hash_count": 3
        }));
        assert_eq!(result_build["operation"], "build");
        assert_eq!(result_build["items_added"], 3);

        let result_query = algo_bloom_filter_work(json!({
            "operation": "query",
            "items": ["foo", "bar", "baz"],
            "query": "foo",
            "size": 1000,
            "hash_count": 3
        }));
        assert_eq!(result_query["possibly_present"], true);
    }

    #[test]
    fn test_bloom_query_absent() {
        // With large enough filter, a definitely-absent item should report false
        let result = algo_bloom_filter_work(json!({
            "operation": "query",
            "items": ["apple", "banana", "cherry"],
            "query": "xyz_not_in_set",
            "size": 10000,
            "hash_count": 5
        }));
        assert_eq!(result["possibly_present"], false);
    }

    #[test]
    fn test_bloom_error_no_operation() {
        let result = algo_bloom_filter_work(json!({"items": ["a"]}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_bloom_error_query_no_items() {
        let result = algo_bloom_filter_work(json!({"operation": "query", "query": "test"}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_number_theory_expansion_tools() {
        assert_eq!(algo_gcd_lcm_work(json!({"a": 48, "b": 18}))["gcd"], 6);
        assert_eq!(
            algo_prime_factors_work(json!({"n": 360}))["factors"],
            json!([2, 2, 2, 3, 3, 5])
        );
        assert_eq!(algo_is_prime_work(json!({"n": 97}))["is_prime"], true);
        assert_eq!(
            algo_sieve_primes_work(json!({"limit": 10}))["primes"],
            json!([2, 3, 5, 7])
        );
        assert_eq!(algo_fibonacci_work(json!({"n": 10}))["value"], "55");
    }

    #[test]
    fn test_conversion_and_count_expansion_tools() {
        assert_eq!(
            algo_base_convert_work(json!({"value": "ff", "from_base": 16, "to_base": 2}))
                ["converted"],
            "11111111"
        );
        assert_eq!(
            algo_roman_numeral_work(json!({"number": 1994}))["roman"],
            "MCMXCIV"
        );
        assert_eq!(
            algo_combinations_count_work(json!({"n": 5, "k": 2}))["combinations"],
            "10"
        );
        assert_eq!(
            algo_permutations_count_work(json!({"n": 5, "k": 2}))["permutations"],
            "20"
        );
    }

    #[test]
    fn test_text_and_array_expansion_tools() {
        let entropy = algo_entropy_work(json!({"text": "aaaa"}));
        assert_eq!(entropy["entropy_bits_per_byte"], 0.0);

        let freq = algo_frequency_table_work(json!({"text": "a a b", "mode": "word"}));
        assert_eq!(freq["entries"][0]["item"], "a");
        assert_eq!(freq["entries"][0]["count"], 2);

        let rle = algo_run_length_encode_work(json!({"text": "aaabb"}));
        assert_eq!(rle["run_count"], 2);

        let bounds = algo_binary_search_bounds_work(json!({"items": [1, 2, 2, 3], "target": 2}));
        assert_eq!(bounds["lower_bound"], 1);
        assert_eq!(bounds["upper_bound"], 3);

        let set = algo_set_ops_work(json!({"a": ["x", "y"], "b": ["y", "z"]}));
        assert_eq!(set["intersection"], json!(["y"]));

        let avg = algo_moving_average_work(json!({"values": [1, 2, 3, 4], "window": 2}));
        assert_eq!(avg["values"], json!([1.0, 1.5, 2.5, 3.5]));

        let percentile = algo_percentile_work(json!({"values": [1, 2, 3, 4], "percentile": 50}));
        assert_eq!(percentile["value"], 2.5);

        let hist = algo_numeric_histogram_work(json!({"values": [1, 2, 3, 4], "bins": 2}));
        assert_eq!(hist["counts"], json!([2, 2]));

        let stats = algo_stats_describe_work(json!({"values": [1, 2, 3, 100]}));
        assert_eq!(stats["iqr"], 25.5);
        assert_eq!(stats["outlier_count"], 1);
        assert!(stats["outlier_fences"]["upper"].as_f64().unwrap() > 40.0);
    }

    #[test]
    fn test_more_useful_algorithm_tools() {
        let top = algo_top_k_work(json!({"items": ["b", "a", "a", "c", "b", "a"], "k": 2}));
        assert_eq!(top["entries"][0]["item"], "a");
        assert_eq!(top["entries"][0]["count"], 3);

        let keyed = algo_top_k_work(json!({
            "items": [{"label": "x"}, {"label": "x"}, {"label": "y"}],
            "key": "label"
        }));
        assert_eq!(keyed["key"], "label");
        assert_eq!(keyed["entries"][0]["item"], "x");
        assert_eq!(keyed["entries"][0]["count"], 2);

        let largest = algo_top_k_work(json!({"mode": "largest", "values": [1, 9, 3], "k": 2}));
        assert_eq!(largest["values"], json!([9.0, 3.0]));

        let outliers = algo_outliers_iqr_work(json!({"values": [1, 2, 2, 3, 100]}));
        assert_eq!(outliers["outlier_count"], 1);
        assert_eq!(outliers["outliers"][0]["value"], 100.0);

        let cm = algo_confusion_matrix_work(json!({
            "expected": ["cat", "cat", "dog", "dog"],
            "predicted": ["cat", "dog", "dog", "dog"]
        }));
        assert_eq!(cm["accuracy"], 0.75);
        assert_eq!(cm["labels"], json!(["cat", "dog"]));
        assert!(cm["macro_avg"]["f1"].as_f64().unwrap() > 0.0);
        assert!(cm["weighted_avg"]["f1"].as_f64().unwrap() > 0.0);

        let cosine = algo_cosine_similarity_work(json!({"a": [1, 0], "b": [1, 1]}));
        let sim = cosine["similarity"].as_f64().unwrap();
        assert!((sim - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-6);

        let minhash =
            algo_minhash_signature_work(json!({"text": "abcdef", "ngram": 3, "hashes": 8}));
        assert_eq!(minhash["signature"].as_array().unwrap().len(), 8);

        let schedule = algo_task_schedule_work(json!({
            "tasks": [
                {"id": "build", "duration": 2},
                {"id": "test", "duration": 3, "depends_on": ["build"]},
                {"id": "ship", "duration": 1, "depends_on": ["test"]}
            ]
        }));
        assert_eq!(schedule["task_count"], 3);
        assert_eq!(schedule["makespan"], 6.0);
    }

    #[test]
    fn test_more_data_algorithm_tools() {
        let normalized = algo_normalize_scores_work(json!({"values": [10, 20, 30]}));
        assert_eq!(normalized["values"], json!([0.0, 0.5, 1.0]));

        let robust = algo_normalize_scores_work(json!({
            "values": [100, 10, 20, 30],
            "method": "robust"
        }));
        assert_eq!(robust["values"].as_array().unwrap().len(), 4);
        assert!(robust["values"][0].as_f64().unwrap() > robust["values"][1].as_f64().unwrap());

        let rolling = algo_rolling_stats_work(json!({"values": [1, 2, 3], "window": 2}));
        assert_eq!(rolling["rows"][2]["mean"], 2.5);
        assert_eq!(rolling["rows"][2]["min"], 2.0);
        assert_eq!(rolling["rows"][2]["max"], 3.0);

        let split = algo_stratified_split_work(json!({
            "labels": ["a", "a", "b", "b"],
            "validation_ratio": 0.5,
            "seed": 7
        }));
        assert_eq!(split["validation_indices"].as_array().unwrap().len(), 2);
        assert_eq!(split["train_indices"].as_array().unwrap().len(), 2);
        assert_eq!(split["label_counts"].as_array().unwrap().len(), 2);

        let sim = algo_minhash_similarity_work(json!({
            "text_a": "abcdef",
            "text_b": "abcdef",
            "ngram": 3
        }));
        assert_eq!(sim["similarity"], 1.0);

        let sig_sim = algo_minhash_similarity_work(json!({
            "signature_a": [1, 2, 3, 4],
            "signature_b": [1, 0, 3, 9]
        }));
        assert_eq!(sig_sim["similarity"], 0.5);
    }

    #[test]
    fn test_ml_dataset_algorithm_tools() {
        let auc = algo_roc_auc_work(json!({
            "labels": [0, 0, 1, 1],
            "scores": [0.1, 0.4, 0.35, 0.8],
            "threshold": 0.5
        }));
        assert_eq!(auc["auc"], 0.75);
        assert_eq!(auc["threshold_metrics"]["tp"], 1);
        assert_eq!(auc["threshold_metrics"]["fn"], 1);

        let clusters = algo_kmeans_work(json!({
            "points": [[0, 0], [0, 1], [9, 9], [10, 9]],
            "k": 2,
            "max_iter": 20
        }));
        assert_eq!(clusters["k"], 2);
        assert_eq!(clusters["assignments"].as_array().unwrap().len(), 4);
        assert!(clusters["inertia"].as_f64().unwrap() < 2.0);

        let chunks = algo_text_chunks_work(json!({
            "text": "abcdef",
            "chunk_size": 3,
            "overlap": 1
        }));
        assert_eq!(chunks["chunk_count"], 3);
        assert_eq!(chunks["chunks"][0]["text"], "abc");
        assert_eq!(chunks["chunks"][1]["text"], "cde");
        assert_eq!(chunks["chunks"][2]["text"], "ef");

        let quality = algo_dataset_quality_work(json!({
            "records": ["alpha", "alpha", "", "token=secret"]
        }));
        assert_eq!(quality["record_count"], 4);
        assert_eq!(quality["empty_count"], 1);
        assert_eq!(quality["unique_count"], 3);
        assert_eq!(quality["duplicate_ratio"], 0.25);
        assert_eq!(quality["duplicate_group_count"], 1);
        assert_eq!(quality["secret_like_count"], 1);

        let labels = algo_label_distribution_work(json!({
            "labels": ["bug", "ok", "bug", "bug"]
        }));
        assert_eq!(labels["total"], 4);
        assert_eq!(labels["majority"]["label"], "bug");
        assert_eq!(labels["imbalance_ratio"], 3.0);

        let ci = algo_bootstrap_mean_ci_work(json!({
            "values": [1, 2, 3, 4],
            "samples": 100,
            "seed": 9
        }));
        assert_eq!(ci["mean"], 2.5);
        assert!(ci["ci"]["lower"].as_f64().unwrap() <= 2.5);
        assert!(ci["ci"]["upper"].as_f64().unwrap() >= 2.5);
    }

    #[test]
    fn test_task_schedule_cycle_is_rejected() {
        let schedule = algo_task_schedule_work(json!({
            "tasks": [
                {"id": "a", "depends_on": ["b"]},
                {"id": "b", "depends_on": ["a"]}
            ]
        }));
        assert!(schedule["error"].as_str().unwrap().contains("cycle"));
    }
}
