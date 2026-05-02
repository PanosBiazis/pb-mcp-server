// Algorithm tool handler methods.
//
// Included inside `#[tool_router] impl MintMcp { ... }` via include!().
// NOTE: No `impl` block, no `use` statements — raw method bodies only.

// ---------------------------------------------------------------------------
// 1. algo_sort
// ---------------------------------------------------------------------------

#[tool(
    description = "Sort a JSON array with optional key (for object items), algorithm label (auto/quicksort/mergesort/heapsort), and reverse flag. Input: {\"items\": [...], \"key\": \"field\", \"algorithm\": \"auto\", \"reverse\": false}. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: sort", read_only_hint = true)
)]
async fn algo_sort(
    &self,
    Parameters(args): Parameters<AlgoSortArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_sort_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 2. algo_search
// ---------------------------------------------------------------------------

#[tool(
    description = "Linear or binary search over a JSON array for a target value. Input: {\"items\": [...], \"target\": <value>, \"method\": \"linear\"|\"binary\"}. Binary search requires a pre-sorted array. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: search", read_only_hint = true)
)]
async fn algo_search(
    &self,
    Parameters(args): Parameters<AlgoSearchArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_search_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 3. algo_graph_bfs
// ---------------------------------------------------------------------------

#[tool(
    description = "Breadth-first search traversal of a JSON adjacency-list graph. Input: {\"graph\": {\"node\": [\"neighbour\", ...]}, \"start\": \"nodeID\"}. Returns visited node order and count. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: graph BFS", read_only_hint = true)
)]
async fn algo_graph_bfs(
    &self,
    Parameters(args): Parameters<AlgoGraphBfsArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_graph_bfs_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 4. algo_graph_dfs
// ---------------------------------------------------------------------------

#[tool(
    description = "Depth-first search traversal of a JSON adjacency-list graph. Input: {\"graph\": {\"node\": [\"neighbour\", ...]}, \"start\": \"nodeID\"}. Returns visited node order and count. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: graph DFS", read_only_hint = true)
)]
async fn algo_graph_dfs(
    &self,
    Parameters(args): Parameters<AlgoGraphDfsArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_graph_dfs_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 5. algo_graph_shortest_path
// ---------------------------------------------------------------------------

#[tool(
    description = "Dijkstra shortest-path on a weighted directed graph. Input: {\"graph\": {\"node\": {\"neighbour\": weight}}, \"start\": \"A\", \"end\": \"Z\"}. Returns path array and total distance. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: graph shortest path", read_only_hint = true)
)]
async fn algo_graph_shortest_path(
    &self,
    Parameters(args): Parameters<AlgoGraphShortestPathArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_graph_shortest_path_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 6. algo_graph_topo_sort
// ---------------------------------------------------------------------------

#[tool(
    description = "Topological sort of a DAG using Kahn's algorithm. Input: {\"graph\": {\"node\": [\"dependent\", ...]}}. Returns sorted order or an error with cycle nodes. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: graph topological sort", read_only_hint = true)
)]
async fn algo_graph_topo_sort(
    &self,
    Parameters(args): Parameters<AlgoGraphTopoSortArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_graph_topo_sort_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 7. algo_matrix_ops
// ---------------------------------------------------------------------------

#[tool(
    description = "Matrix operations: multiply, transpose, determinant, or inverse. Input: {\"operation\": \"multiply\"|\"transpose\"|\"determinant\"|\"inverse\", \"a\": [[...]], \"b\": [[...]]}. \"b\" is required for multiply. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: matrix operations", read_only_hint = true)
)]
async fn algo_matrix_ops(
    &self,
    Parameters(args): Parameters<AlgoMatrixOpsArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_matrix_ops_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 8. algo_stats_describe
// ---------------------------------------------------------------------------

#[tool(
    description = "Compute descriptive statistics for a numeric array: mean, median, mode, std_dev, min, max, percentiles (p25/p50/p75/p95/p99), IQR, and count. Input: {\"values\": [1, 2, 3, ...]}. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: stats describe", read_only_hint = true)
)]
async fn algo_stats_describe(
    &self,
    Parameters(args): Parameters<AlgoStatsDescribeArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_stats_describe_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 9. algo_stats_correlation
// ---------------------------------------------------------------------------

#[tool(
    description = "Pearson or Spearman correlation between two numeric arrays. Input: {\"x\": [...], \"y\": [...], \"method\": \"pearson\"|\"spearman\"}. Both arrays must have the same length (>= 2). Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: stats correlation", read_only_hint = true)
)]
async fn algo_stats_correlation(
    &self,
    Parameters(args): Parameters<AlgoStatsCorrelationArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_stats_correlation_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 10. algo_stats_regression
// ---------------------------------------------------------------------------

#[tool(
    description = "Simple linear regression (y = slope * x + intercept) over two numeric arrays. Input: {\"x\": [...], \"y\": [...]}. Returns slope, intercept, r_squared, and n. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: stats linear regression", read_only_hint = true)
)]
async fn algo_stats_regression(
    &self,
    Parameters(args): Parameters<AlgoStatsRegressionArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_stats_regression_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 11. algo_fft
// ---------------------------------------------------------------------------

#[tool(
    description = "Discrete Fourier Transform (DFT) over a real-valued signal. Input: {\"values\": [0.0, 1.0, ...]}. Returns the first N/2 frequency bins with frequency index, magnitude, and phase. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: FFT / DFT", read_only_hint = true)
)]
async fn algo_fft(
    &self,
    Parameters(args): Parameters<AlgoFftArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_fft_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 12. algo_hash_text
// ---------------------------------------------------------------------------

#[tool(
    description = "Hash a text string using SHA-256 (default), SHA-512, MD5, or blake3 (falls back to SHA-256). Input: {\"text\": \"hello\", \"algorithm\": \"sha256\"}. Returns hex-encoded digest. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: hash text", read_only_hint = true)
)]
async fn algo_hash_text(
    &self,
    Parameters(args): Parameters<AlgoHashTextArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_hash_text_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 13. algo_edit_distance
// ---------------------------------------------------------------------------

#[tool(
    description = "Compute Levenshtein edit distance between two strings. Input: {\"a\": \"kitten\", \"b\": \"sitting\"}. Returns the minimum number of single-character edits (insert/delete/substitute). Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: edit distance", read_only_hint = true)
)]
async fn algo_edit_distance(
    &self,
    Parameters(args): Parameters<AlgoEditDistanceArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_edit_distance_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 14. algo_ngram_similarity
// ---------------------------------------------------------------------------

#[tool(
    description = "N-gram Jaccard similarity between two strings. Input: {\"a\": \"hello world\", \"b\": \"hello there\", \"n\": 2}. Returns similarity in [0, 1] and intersection/union counts. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: n-gram similarity", read_only_hint = true)
)]
async fn algo_ngram_similarity(
    &self,
    Parameters(args): Parameters<AlgoNgramSimilarityArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_ngram_similarity_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 15. algo_bloom_filter
// ---------------------------------------------------------------------------

#[tool(
    description = "Bloom filter: build a filter from a string set, or query for probable membership. Input: {\"operation\": \"build\"|\"query\", \"items\": [...], \"query\": \"item\", \"size\": 1000, \"hash_count\": 3}. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: bloom filter", read_only_hint = true)
)]
async fn algo_bloom_filter(
    &self,
    Parameters(args): Parameters<AlgoBloomFilterArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_bloom_filter_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}
