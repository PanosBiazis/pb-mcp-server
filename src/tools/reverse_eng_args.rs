// Reverse-engineering tool argument structs.
//
// Included at module scope via include!() in the parent file.
// NOTE: `JsonSchema`, `Deserialize`, and `schemars` are already in scope from
// the parent — do NOT add `use` statements here.

// ---------------------------------------------------------------------------
// 1. re_binary_info
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct ReBinaryInfoArgs {
    /// Path to the binary file to inspect, relative to the project root.
    path: String,
}

// ---------------------------------------------------------------------------
// 2. re_strings
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct ReStringsArgs {
    /// Path to the binary file to extract strings from, relative to the project root.
    path: String,
    /// Minimum printable-character run length to include (default is typically 4).
    #[serde(default)]
    min_length: Option<usize>,
    /// Character encoding to scan for: "ascii", "unicode", or "all". Omit for default (ascii).
    #[serde(default)]
    encoding: Option<String>,
}

// ---------------------------------------------------------------------------
// 3. re_disassemble
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct ReDisassembleArgs {
    /// Path to the binary or object file to disassemble, relative to the project root.
    path: String,
    /// ELF/PE section to disassemble (e.g. ".text"). Omit for the default code section.
    #[serde(default)]
    section: Option<String>,
    /// Target architecture hint (e.g. "x86", "x86_64", "arm", "aarch64"). Omit for auto-detect.
    #[serde(default)]
    architecture: Option<String>,
    /// Start address (hex or decimal) at which to begin disassembly. Omit for section start.
    #[serde(default)]
    start_addr: Option<String>,
    /// Maximum number of instructions to disassemble. Omit for all instructions in the range.
    #[serde(default)]
    count: Option<usize>,
}

// ---------------------------------------------------------------------------
// 4. re_symbols
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct ReSymbolsArgs {
    /// Path to the binary or object file to extract symbols from, relative to the project root.
    path: String,
    /// Symbol type filter: "function", "object", "undefined", or "all". Omit for all types.
    #[serde(default)]
    symbol_type: Option<String>,
}

// ---------------------------------------------------------------------------
// 5. re_sections
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct ReSectionsArgs {
    /// Path to the binary or object file whose sections are to be listed, relative to the project root.
    path: String,
}

// ---------------------------------------------------------------------------
// 6. re_dependencies
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct ReDependenciesArgs {
    /// Path to the binary file whose dynamic library dependencies are to be listed, relative to the project root.
    path: String,
}

// ---------------------------------------------------------------------------
// 7. re_hexdump
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct ReHexdumpArgs {
    /// Path to the binary file to hex-dump, relative to the project root.
    path: String,
    /// Byte offset at which to start the dump. Omit to start from the beginning of the file.
    #[serde(default)]
    offset: Option<u64>,
    /// Number of bytes to dump. Omit to dump the entire file (capped internally).
    #[serde(default)]
    length: Option<usize>,
    /// Number of bytes per row in the hex dump (e.g. 16, 32). Omit for default (16).
    #[serde(default)]
    width: Option<usize>,
}

// ---------------------------------------------------------------------------
// 8. re_entropy
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct ReEntropyArgs {
    /// Path to the binary file to compute Shannon entropy over, relative to the project root.
    path: String,
    /// Block size in bytes for per-block entropy computation. Omit for whole-file entropy.
    #[serde(default)]
    block_size: Option<usize>,
}

// ---------------------------------------------------------------------------
// 9. re_carve
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct ReCarveArgs {
    /// Path to the source binary file to carve bytes from, relative to the project root.
    path: String,
    /// Byte offset at which carving starts.
    offset: u64,
    /// Number of bytes to carve from the offset.
    length: usize,
    /// Output file path for the carved bytes, relative to the project root. Omit to auto-generate.
    #[serde(default)]
    output: Option<String>,
}

// ---------------------------------------------------------------------------
// 10. re_pattern_search
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct RePatternSearchArgs {
    /// Path to the binary file to search, relative to the project root.
    path: String,
    /// Byte pattern to search for (hex string such as "DEADBEEF", or a regex string).
    pattern: String,
    /// Maximum number of matches to return. Omit for all matches.
    #[serde(default)]
    max_results: Option<usize>,
}

// ---------------------------------------------------------------------------
// 11. re_struct_parse
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct ReStructParseArgs {
    /// Path to the binary file to parse a structure from, relative to the project root.
    path: String,
    /// Byte offset at which the structure starts.
    offset: u64,
    /// Structure format descriptor (e.g. a C-struct-like string or pack format spec).
    format: String,
}

// ---------------------------------------------------------------------------
// 12. re_diff_binary
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct ReDiffBinaryArgs {
    /// Path to the first binary file, relative to the project root.
    path_a: String,
    /// Path to the second binary file to compare against, relative to the project root.
    path_b: String,
}

// ---------------------------------------------------------------------------
// 13. re_strace
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct ReStraceArgs {
    /// Path to the binary to execute under strace, relative to the project root.
    path: String,
    /// Extra command-line arguments to pass to the traced binary.
    #[serde(default)]
    extra_args: Vec<String>,
    /// Maximum execution time in seconds before killing the traced process. Omit for default.
    #[serde(default)]
    timeout_secs: Option<u64>,
}

// ---------------------------------------------------------------------------
// 14. re_binwalk_scan
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct ReBinwalkScanArgs {
    /// Path to the firmware or binary blob to scan with binwalk, relative to the project root.
    path: String,
}
