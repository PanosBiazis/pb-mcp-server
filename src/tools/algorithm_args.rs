// Algorithm tool argument structs.
//
// Included at module scope via include!() in the parent file.
// NOTE: `JsonSchema`, `Deserialize`, and `schemars` are already in scope from
// the parent — do NOT add `use` statements here.

// ---------------------------------------------------------------------------
// 1. algo_sort
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct AlgoSortArgs {
    /// JSON input for sort. Required: "items" (array of values to sort).
    /// Optional: "key" (string field name to sort by when items are objects),
    /// "algorithm" ("auto" | "quicksort" | "mergesort" | "heapsort", default "auto"),
    /// "reverse" (bool, default false — set true for descending order).
    /// Example: {"items": [3, 1, 2], "algorithm": "auto", "reverse": false}
    input: serde_json::Value,
}

// ---------------------------------------------------------------------------
// 2. algo_search
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct AlgoSearchArgs {
    /// JSON input for search. Required: "items" (array), "target" (value to find).
    /// Optional: "method" ("linear" | "binary", default "linear").
    /// Binary search requires the array to be pre-sorted.
    /// Example: {"items": [1, 2, 3, 4, 5], "target": 3, "method": "binary"}
    input: serde_json::Value,
}

// ---------------------------------------------------------------------------
// 3. algo_graph_bfs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct AlgoGraphBfsArgs {
    /// JSON input for BFS traversal. Required: "graph" (adjacency-list object
    /// where each key is a node ID and the value is an array of neighbour node IDs),
    /// "start" (string node ID to begin traversal from).
    /// Example: {"graph": {"A": ["B","C"], "B": ["D"], "C": [], "D": []}, "start": "A"}
    input: serde_json::Value,
}

// ---------------------------------------------------------------------------
// 4. algo_graph_dfs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct AlgoGraphDfsArgs {
    /// JSON input for DFS traversal. Required: "graph" (adjacency-list object
    /// where each key is a node ID and the value is an array of neighbour node IDs),
    /// "start" (string node ID to begin traversal from).
    /// Example: {"graph": {"A": ["B","C"], "B": ["D"], "C": [], "D": []}, "start": "A"}
    input: serde_json::Value,
}

// ---------------------------------------------------------------------------
// 5. algo_graph_shortest_path
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct AlgoGraphShortestPathArgs {
    /// JSON input for Dijkstra shortest path. Required: "graph" (weighted adjacency
    /// object: {"node": {"neighbour": weight, ...}, ...}), "start" (source node ID),
    /// "end" (destination node ID). Weights must be non-negative numbers.
    /// Example: {"graph": {"A": {"B": 1, "C": 4}, "B": {"C": 2}, "C": {}}, "start": "A", "end": "C"}
    input: serde_json::Value,
}

// ---------------------------------------------------------------------------
// 6. algo_graph_topo_sort
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct AlgoGraphTopoSortArgs {
    /// JSON input for topological sort (Kahn's algorithm). Required: "graph"
    /// (DAG adjacency-list: {"node": ["dependent1", "dependent2", ...], ...}).
    /// Returns an error if a cycle is detected.
    /// Example: {"graph": {"build": ["test"], "test": ["deploy"], "deploy": []}}
    input: serde_json::Value,
}

// ---------------------------------------------------------------------------
// 7. algo_matrix_ops
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct AlgoMatrixOpsArgs {
    /// JSON input for matrix operations. Required: "operation" ("multiply" | "transpose"
    /// | "determinant" | "inverse"), "a" (2-D array of numbers representing matrix A).
    /// "b" is required for "multiply" (2-D array for matrix B).
    /// Matrices are row-major: [[row0col0, row0col1], [row1col0, row1col1]].
    /// Example: {"operation": "multiply", "a": [[1,2],[3,4]], "b": [[5,6],[7,8]]}
    input: serde_json::Value,
}

// ---------------------------------------------------------------------------
// 8. algo_stats_describe
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct AlgoStatsDescribeArgs {
    /// JSON input for descriptive statistics. Required: "values" (array of numbers).
    /// Returns mean, median, mode, std_dev, min, max, percentiles (p25/p50/p75/p95/p99),
    /// IQR, and count.
    /// Example: {"values": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]}
    input: serde_json::Value,
}

// ---------------------------------------------------------------------------
// 9. algo_stats_correlation
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct AlgoStatsCorrelationArgs {
    /// JSON input for correlation analysis. Required: "x" (array of numbers),
    /// "y" (array of numbers, same length as x, minimum 2 elements).
    /// Optional: "method" ("pearson" | "spearman", default "pearson").
    /// Example: {"x": [1,2,3,4,5], "y": [2,4,5,4,5], "method": "pearson"}
    input: serde_json::Value,
}

// ---------------------------------------------------------------------------
// 10. algo_stats_regression
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct AlgoStatsRegressionArgs {
    /// JSON input for simple linear regression (y = slope * x + intercept).
    /// Required: "x" (array of numbers), "y" (array of numbers, same length, min 2).
    /// Returns slope, intercept, r_squared, and n.
    /// Example: {"x": [1,2,3,4,5], "y": [2,4,5,4,5]}
    input: serde_json::Value,
}

// ---------------------------------------------------------------------------
// 11. algo_fft
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct AlgoFftArgs {
    /// JSON input for DFT (Discrete Fourier Transform). Required: "values"
    /// (array of real numbers representing the time-domain signal). Returns
    /// the first N/2 frequency bins with frequency index, magnitude, and phase.
    /// Example: {"values": [0, 1, 0, -1, 0, 1, 0, -1]}
    input: serde_json::Value,
}

// ---------------------------------------------------------------------------
// 12. algo_hash_text
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct AlgoHashTextArgs {
    /// JSON input for text hashing. Required: "text" (string to hash).
    /// Optional: "algorithm" ("sha256" | "sha512" | "md5" | "blake3", default "sha256").
    /// Note: blake3 falls back to sha256 when the blake3 crate is unavailable.
    /// Example: {"text": "hello world", "algorithm": "sha256"}
    input: serde_json::Value,
}

// ---------------------------------------------------------------------------
// 13. algo_edit_distance
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct AlgoEditDistanceArgs {
    /// JSON input for Levenshtein edit distance. Required: "a" (first string),
    /// "b" (second string). Returns the minimum number of single-character edits
    /// (insertions, deletions, substitutions) to transform "a" into "b".
    /// Example: {"a": "kitten", "b": "sitting"}
    input: serde_json::Value,
}

// ---------------------------------------------------------------------------
// 14. algo_ngram_similarity
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct AlgoNgramSimilarityArgs {
    /// JSON input for N-gram Jaccard similarity. Required: "a" (first string),
    /// "b" (second string). Optional: "n" (ngram size, default 2).
    /// Returns a similarity score in [0, 1] where 1 = identical ngram sets.
    /// Example: {"a": "hello world", "b": "hello there", "n": 2}
    input: serde_json::Value,
}

// ---------------------------------------------------------------------------
// 15. algo_bloom_filter
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct AlgoBloomFilterArgs {
    /// JSON input for bloom filter operations. Required: "operation" ("build" | "query").
    /// For "build": "items" (array of strings to insert). Optional: "size" (bit-array
    /// size, default 1000), "hash_count" (number of hash functions, default 3).
    /// For "query": "query" (string to test), "items" (array of strings to rebuild
    /// the filter), same optional "size" and "hash_count".
    /// Example (build): {"operation": "build", "items": ["foo","bar","baz"], "size": 1000, "hash_count": 3}
    /// Example (query): {"operation": "query", "query": "foo", "items": ["foo","bar"]}
    input: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AlgoJsonArgs {
    /// JSON input for the selected algorithm tool. Each tool documents its
    /// required keys in the tool description.
    input: serde_json::Value,
}
