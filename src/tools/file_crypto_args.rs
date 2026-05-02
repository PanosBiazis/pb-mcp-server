// File-crypto tool argument structs.
//
// Included at module scope via include!() in the parent file.
// NOTE: `JsonSchema`, `Deserialize`, and `schemars` are already in scope from
// the parent — do NOT add `use` statements here.

// ---------------------------------------------------------------------------
// 1. file_encrypt
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct FileEncryptArgs {
    /// File path to encrypt, relative to project root.
    path: String,
    /// Passphrase used to derive the encryption key.
    passphrase: String,
}

// ---------------------------------------------------------------------------
// 2. file_decrypt
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct FileDecryptArgs {
    /// Path to the encrypted .enc file, relative to project root.
    path: String,
    /// Passphrase used to derive the decryption key (must match the one used to encrypt).
    passphrase: String,
}

// ---------------------------------------------------------------------------
// 3. file_sign
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct FileSignArgs {
    /// Path to the file to sign, relative to project root.
    path: String,
    /// Passphrase used to derive the HMAC-SHA256 signing key.
    passphrase: String,
}

// ---------------------------------------------------------------------------
// 4. file_verify_signature
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct FileVerifySignatureArgs {
    /// Path to the original file, relative to project root.
    path: String,
    /// Path to the .sig signature file, relative to project root.
    sig_path: String,
    /// Passphrase that was used when the file was signed (must match exactly).
    passphrase: String,
}

// ---------------------------------------------------------------------------
// 5. file_deduplicate
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct FileDeduplicateArgs {
    /// Directory to scan for duplicate files, relative to project root.
    path: String,
    /// Maximum recursion depth when scanning subdirectories (default 10).
    #[serde(default)]
    max_depth: Option<u64>,
}

// ---------------------------------------------------------------------------
// 6. file_watch
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct FileWatchArgs {
    /// File or directory path to snapshot, relative to project root.
    path: String,
    /// Whether to compute a SHA-256 hash for each file (default true).
    /// Set to false for large files or directories to skip hashing.
    #[serde(default)]
    compute_hash: Option<bool>,
}
