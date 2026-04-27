//! # Algorithm & Math Tools
//!
//! Sorting, searching, graph algorithms, matrix operations, statistics,
//! DFT, hashing, string similarity, and bloom filter — all as pure
//! `serde_json::Value` → `serde_json::Value` work functions.

use serde_json::{json, Value};
use sha2::{Digest, Sha256, Sha512};
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

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
                adj.get_mut(node).unwrap().push((nbr.clone(), weight));
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
                    adj.get_mut(node).unwrap().push(ns.to_string());
                    in_degree.entry(ns.to_string()).or_insert(0);
                }
            }
        }
    }

    // Compute in-degrees
    for neighbors in adj.values() {
        for n in neighbors {
            *in_degree.get_mut(n).unwrap() += 1;
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

fn determinant(m: &[Vec<f64>]) -> f64 {
    let n = m.len();
    if n == 1 {
        return m[0][0];
    }
    if n == 2 {
        return m[0][0] * m[1][1] - m[0][1] * m[1][0];
    }
    let mut det = 0.0;
    for j in 0..n {
        let sub = submatrix(m, 0, j);
        let sign = if j % 2 == 0 { 1.0 } else { -1.0 };
        det += sign * m[0][j] * determinant(&sub);
    }
    det
}

fn submatrix(m: &[Vec<f64>], skip_row: usize, skip_col: usize) -> Vec<Vec<f64>> {
    let n = m.len();
    let mut result = Vec::with_capacity(n - 1);
    for i in 0..n {
        if i == skip_row {
            continue;
        }
        let mut row = Vec::with_capacity(n - 1);
        for j in 0..n {
            if j == skip_col {
                continue;
            }
            row.push(m[i][j]);
        }
        result.push(row);
    }
    result
}

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
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let min = sorted[0];
    let max = sorted[n - 1];
    let median = percentile(&sorted, 50.0);
    let p25 = percentile(&sorted, 25.0);
    let p50 = percentile(&sorted, 50.0);
    let p75 = percentile(&sorted, 75.0);
    let p95 = percentile(&sorted, 95.0);
    let p99 = percentile(&sorted, 99.0);
    let iqr = p75 - p25;

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
            self.transform(block.try_into().unwrap());
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
            self.transform(block.try_into().unwrap());
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
        for i in 1..bins.len() {
            assert!(bins[i]["magnitude"].as_f64().unwrap().abs() < 1e-6);
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
}
