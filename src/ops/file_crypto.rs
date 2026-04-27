//! # File Cryptography & Integrity Tools
//!
//! Encryption / decryption (AES-256-GCM simulated via XOR + base64 for
//! environments where the `aes-gcm` crate is not available), file signing
//! and verification (HMAC-SHA256 using a passphrase), duplicate-file
//! detection by SHA-256 content hash, and file-watch snapshot helpers.
//!
//! Every public work function takes a `serde_json::Value` and returns a
//! JSON string of the form `{"ok": true, "data": {...}}` or
//! `{"ok": false, "error": "..."}`.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// HMAC-SHA256 type alias.
type HmacSha256 = Hmac<Sha256>;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract a required string parameter from a `Value`, or return an
/// error string.
fn require_str(params: &Value, key: &str) -> Result<String, String> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or(format!("Missing required arg: {key}"))
}

/// Extract an optional string parameter from a `Value`.
#[allow(dead_code)]
fn opt_str(params: &Value, key: &str) -> Option<String> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Extract an optional u64 parameter from a `Value`.
#[allow(dead_code)]
fn opt_u64(params: &Value, key: &str) -> Option<u64> {
    params.get(key).and_then(|v| v.as_u64())
}

/// Build an error JSON response: `{"ok": false, "error": "..."}`.
fn err_json(msg: impl std::fmt::Display) -> String {
    serde_json::json!({"ok": false, "error": format!("{msg}")}).to_string()
}

/// Build a success JSON response: `{"ok": true, "data": {...}}`.
fn ok_json(data: serde_json::Value) -> String {
    serde_json::json!({"ok": true, "data": data}).to_string()
}

/// Derive a 32-byte key from a passphrase using repeated SHA-256 hashing.
///
/// This is **not** a proper KDF like HKDF or Argon2 — it is a deterministic
/// derivation so that encryption and decryption produce the same key from the
/// same passphrase. For production use, replace with a real KDF.
fn derive_key(passphrase: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(passphrase.as_bytes());
    hasher.update(b"pb-mcp-server-salt-v1");
    let h1 = hasher.finalize();

    let mut hasher2 = Sha256::new();
    hasher2.update(h1);
    hasher2.update(passphrase.as_bytes());
    let h2 = hasher2.finalize();

    let mut key = [0u8; 32];
    key.copy_from_slice(&h2);
    key
}

/// XOR bytes with a cycling key. Used as a stand-in for AES-256-GCM when
/// the `aes-gcm` crate is not in `Cargo.toml`.
fn xor_cycle(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % 32])
        .collect()
}

/// Compute the SHA-256 hex digest of an entire file.
fn sha256_file(path: &Path) -> Result<String, String> {
    let mut f = fs::File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = f
            .read(&mut buf)
            .map_err(|e| format!("read {}: {e}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex_encode(hasher.finalize().as_slice()))
}

/// Encode bytes as lowercase hex.
fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect()
}

/// Resolve a path string to a `PathBuf`, checking that it stays within the
/// project root (i.e. does not escape via `..` traversal).
fn safe_path(path_str: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from(path_str);
    let mut depth: i64 = 0;
    for comp in p.components() {
        match comp {
            std::path::Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return Err(format!(
                        "Path traversal detected: {path_str} escapes project root"
                    ));
                }
            }
            std::path::Component::Normal(_) => {
                depth += 1;
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                // Absolute paths are allowed but will be treated as-is.
            }
            _ => {}
        }
    }
    Ok(p)
}

/// Ensure a path points to an existing regular file.
fn ensure_file(path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Err(format!("Not a file: {}", path.display()));
    }
    Ok(())
}

/// Ensure a directory exists, creating it if necessary.
fn ensure_dir(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(path).map_err(|e| format!("Cannot create directory {}: {e}", path.display()))
}

// ---------------------------------------------------------------------------
// 1. file_encrypt_work
// ---------------------------------------------------------------------------

/// Encrypt a file with AES-256-GCM (simulated via XOR + base64).
///
/// # Parameters
/// - `path` (string, required): Input file path.
/// - `passphrase` (string, required): Encryption passphrase.
///
/// # Output
/// Writes an encrypted file at `<path>.enc` and returns JSON with the
/// output path and file sizes.
#[allow(dead_code)]
pub fn file_encrypt_work(params: Value) -> String {
    let path_str = match require_str(&params, "path") {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };
    let passphrase = match require_str(&params, "passphrase") {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };

    let path = match safe_path(&path_str) {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };
    if let Err(e) = ensure_file(&path) {
        return err_json(e);
    }

    let data = match fs::read(&path) {
        Ok(d) => d,
        Err(e) => return err_json(format!("Read error: {e}")),
    };

    let key = derive_key(&passphrase);
    let xored = xor_cycle(&data, &key);
    let encoded = BASE64.encode(&xored);

    // Format: "ENCV1:<base64>" so we can distinguish encrypted files.
    let payload = format!("ENCV1:{encoded}");

    let out_path = {
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let new_name = if ext.is_empty() {
            format!("{stem}.enc")
        } else {
            format!("{stem}.{ext}.enc")
        };
        path.with_file_name(new_name)
    };

    if let Some(parent) = out_path.parent() {
        if let Err(e) = ensure_dir(parent) {
            return err_json(e);
        }
    }

    if let Err(e) = fs::write(&out_path, &payload) {
        return err_json(format!("Write error: {e}"));
    }

    let file_size = match fs::metadata(&out_path) {
        Ok(m) => m.len(),
        Err(_) => payload.len() as u64,
    };

    ok_json(serde_json::json!({
        "output_path": out_path.display().to_string(),
        "original_bytes": data.len(),
        "encrypted_bytes": file_size,
    }))
}

// ---------------------------------------------------------------------------
// 2. file_decrypt_work
// ---------------------------------------------------------------------------

/// Decrypt a file that was encrypted with `file_encrypt_work`.
///
/// # Parameters
/// - `path` (string, required): Encrypted file path (`.enc`).
/// - `passphrase` (string, required): Decryption passphrase.
///
/// # Output
/// Writes the decrypted file at `<path>` without the `.enc` extension.
#[allow(dead_code)]
pub fn file_decrypt_work(params: Value) -> String {
    let path_str = match require_str(&params, "path") {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };
    let passphrase = match require_str(&params, "passphrase") {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };

    let path = match safe_path(&path_str) {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };
    if let Err(e) = ensure_file(&path) {
        return err_json(e);
    }

    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return err_json(format!("Read error: {e}")),
    };

    let trimmed = raw.trim();
    if !trimmed.starts_with("ENCV1:") {
        return err_json(
            "File does not appear to be encrypted by this tool (missing ENCV1: header)",
        );
    }

    let b64_part = &trimmed["ENCV1:".len()..];
    let xored = match BASE64.decode(b64_part) {
        Ok(d) => d,
        Err(e) => return err_json(format!("Base64 decode error: {e}")),
    };

    let key = derive_key(&passphrase);
    let decrypted = xor_cycle(&xored, &key);

    // Derive the output path by stripping the `.enc` suffix.
    let out_path = {
        let ps = path.display().to_string();
        if ps.ends_with(".enc") {
            PathBuf::from(&ps[..ps.len() - 4])
        } else {
            path.with_extension("")
        }
    };

    if let Some(parent) = out_path.parent() {
        if let Err(e) = ensure_dir(parent) {
            return err_json(e);
        }
    }

    if let Err(e) = fs::write(&out_path, &decrypted) {
        return err_json(format!("Write error: {e}"));
    }

    ok_json(serde_json::json!({
        "output_path": out_path.display().to_string(),
        "decrypted_bytes": decrypted.len(),
    }))
}

// ---------------------------------------------------------------------------
// 3. file_sign_work
// ---------------------------------------------------------------------------

/// Sign a file with HMAC-SHA256 using a passphrase as the key.
///
/// # Parameters
/// - `path` (string, required): File to sign.
/// - `passphrase` (string, required): Signing passphrase (used to derive
///   the HMAC key via SHA-256).
///
/// # Output
/// Writes a `.sig` file next to the input containing the hex-encoded
/// HMAC-SHA256 digest.
#[allow(dead_code)]
pub fn file_sign_work(params: Value) -> String {
    let path_str = match require_str(&params, "path") {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };
    let passphrase = match require_str(&params, "passphrase") {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };

    let path = match safe_path(&path_str) {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };
    if let Err(e) = ensure_file(&path) {
        return err_json(e);
    }

    // Derive the HMAC key from the passphrase (same derivation as encrypt).
    let key = derive_key(&passphrase);

    // Read file and compute HMAC-SHA256.
    let data = match fs::read(&path) {
        Ok(d) => d,
        Err(e) => return err_json(format!("Read error: {e}")),
    };

    let mut mac = HmacSha256::new_from_slice(&key).expect("HMAC can accept any key size");
    mac.update(&data);
    let result = mac.finalize();
    let sig_hex = hex_encode(result.into_bytes().as_slice());

    // Write .sig file.
    let sig_path = {
        let ps = path.display().to_string();
        PathBuf::from(format!("{ps}.sig"))
    };

    if let Some(parent) = sig_path.parent() {
        if let Err(e) = ensure_dir(parent) {
            return err_json(e);
        }
    }

    // Include the original file's SHA-256 in the sig file for reference.
    let file_hash = match sha256_file(&path) {
        Ok(h) => h,
        Err(e) => return err_json(e),
    };

    let sig_content = format!("hmac-sha256:{sig_hex}\nsha256:{file_hash}\n");
    if let Err(e) = fs::write(&sig_path, &sig_content) {
        return err_json(format!("Sig write error: {e}"));
    }

    ok_json(serde_json::json!({
        "sig_path": sig_path.display().to_string(),
        "algorithm": "hmac-sha256",
        "signature_hex": sig_hex,
        "file_sha256": file_hash,
    }))
}

// ---------------------------------------------------------------------------
// 4. file_verify_signature_work
// ---------------------------------------------------------------------------

/// Verify a file's HMAC-SHA256 signature.
///
/// # Parameters
/// - `path` (string, required): The original file.
/// - `sig_path` (string, required): Path to the `.sig` file.
/// - `passphrase` (string, required): Signing passphrase (must match the
///   one used during signing).
///
/// # Output
/// Returns `verified: true` if the signature matches, `verified: false`
/// otherwise.
#[allow(dead_code)]
pub fn file_verify_signature_work(params: Value) -> String {
    let path_str = match require_str(&params, "path") {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };
    let sig_path_str = match require_str(&params, "sig_path") {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };
    let passphrase = match require_str(&params, "passphrase") {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };

    let path = match safe_path(&path_str) {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };
    let sig_path = match safe_path(&sig_path_str) {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };

    if let Err(e) = ensure_file(&path) {
        return err_json(e);
    }
    if let Err(e) = ensure_file(&sig_path) {
        return err_json(e);
    }

    // Derive the HMAC key from the passphrase.
    let key = derive_key(&passphrase);

    // Read the file.
    let data = match fs::read(&path) {
        Ok(d) => d,
        Err(e) => return err_json(format!("File read error: {e}")),
    };

    // Compute HMAC-SHA256 of the file with the key.
    let mut mac = HmacSha256::new_from_slice(&key).expect("HMAC can accept any key size");
    mac.update(&data);
    let result = mac.finalize();
    let computed_hex = hex_encode(result.into_bytes().as_slice());

    // Read the .sig file and extract the stored HMAC.
    let sig_content = match fs::read_to_string(&sig_path) {
        Ok(s) => s,
        Err(e) => return err_json(format!("Sig read error: {e}")),
    };

    let stored_hex = sig_content
        .lines()
        .find_map(|line| {
            let stripped = line.trim();
            stripped
                .strip_prefix("hmac-sha256:")
                .map(|h| h.trim().to_string())
        })
        .unwrap_or_default();

    if stored_hex.is_empty() {
        return err_json("Signature file does not contain a valid hmac-sha256 line");
    }

    // Constant-time comparison to prevent timing attacks.
    let verified = hmac_is_equal(stored_hex.as_bytes(), computed_hex.as_bytes());

    // Also check SHA-256 if present in the sig file.
    let stored_sha256 = sig_content
        .lines()
        .find_map(|line| {
            let stripped = line.trim();
            stripped
                .strip_prefix("sha256:")
                .map(|h| h.trim().to_string())
        })
        .unwrap_or_default();

    let mut sha256_match: Option<bool> = None;
    if !stored_sha256.is_empty() {
        match sha256_file(&path) {
            Ok(current_hash) => {
                sha256_match = Some(current_hash == stored_sha256);
            }
            Err(_) => {
                sha256_match = None;
            }
        }
    }

    let mut data = serde_json::json!({
        "verified": verified,
        "algorithm": "hmac-sha256",
    });
    if let Some(sm) = sha256_match {
        data.as_object_mut()
            .unwrap()
            .insert("sha256_match".into(), serde_json::Value::Bool(sm));
    }

    ok_json(data)
}

/// Constant-time byte comparison wrapper (uses `subtle` via hmac crate).
fn hmac_is_equal(a: &[u8], b: &[u8]) -> bool {
    use subtle::ConstantTimeEq;
    a.ct_eq(b).into()
}

// ---------------------------------------------------------------------------
// 5. file_deduplicate_work
// ---------------------------------------------------------------------------

/// Find duplicate files in a directory tree by SHA-256 content hash.
///
/// # Parameters
/// - `path` (string, required): Directory to scan.
/// - `max_depth` (u64, optional): Maximum recursion depth (default 10).
///
/// # Output
/// Returns a list of duplicate groups, each containing the SHA-256 hash,
/// file size, and list of file paths that share the same content.
#[allow(dead_code)]
pub fn file_deduplicate_work(params: Value) -> String {
    let path_str = match require_str(&params, "path") {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };

    let max_depth = opt_u64(&params, "max_depth").unwrap_or(10) as usize;

    let dir = match safe_path(&path_str) {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };

    if !dir.is_dir() {
        return err_json(format!("Not a directory: {}", dir.display()));
    }

    // Map from SHA-256 hex → list of (path, size).
    let mut hash_map: HashMap<String, Vec<(String, u64)>> = HashMap::new();
    let mut errors: Vec<String> = Vec::new();
    let mut total_files: u64 = 0;

    // Walk the directory tree.
    if let Err(e) = walk_and_hash(
        &dir,
        0,
        max_depth,
        &mut hash_map,
        &mut errors,
        &mut total_files,
    ) {
        return err_json(e);
    }

    // Collect only groups with >1 entry (duplicates).
    let duplicate_groups: Vec<serde_json::Value> = hash_map
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|(hash, paths)| {
            let size = paths[0].1;
            let file_paths: Vec<String> = paths.into_iter().map(|(p, _)| p).collect();
            serde_json::json!({
                "hash": hash,
                "size": size,
                "count": file_paths.len(),
                "paths": file_paths,
            })
        })
        .collect();

    let duplicate_count: usize = duplicate_groups
        .iter()
        .map(|g| g["count"].as_u64().unwrap_or(0) as usize)
        .sum();

    ok_json(serde_json::json!({
        "total_files_scanned": total_files,
        "duplicate_groups": duplicate_groups.len(),
        "duplicate_files": duplicate_count,
        "groups": duplicate_groups,
        "errors": errors,
    }))
}

/// Recursively walk a directory, hash each file with SHA-256, and record
/// it in the hash map.
fn walk_and_hash(
    dir: &Path,
    depth: usize,
    max_depth: usize,
    hash_map: &mut HashMap<String, Vec<(String, u64)>>,
    errors: &mut Vec<String>,
    total_files: &mut u64,
) -> Result<(), String> {
    if depth > max_depth {
        return Ok(());
    }

    let entries = fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                errors.push(format!("dir entry error: {e}"));
                continue;
            }
        };

        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                errors.push(format!("file_type error for {}: {e}", path.display()));
                continue;
            }
        };

        if file_type.is_dir() {
            // Recurse into subdirectory.
            walk_and_hash(&path, depth + 1, max_depth, hash_map, errors, total_files)?;
        } else if file_type.is_file() {
            *total_files += 1;
            let size = match fs::metadata(&path) {
                Ok(m) => m.len(),
                Err(e) => {
                    errors.push(format!("metadata error for {}: {e}", path.display()));
                    continue;
                }
            };

            match sha256_file(&path) {
                Ok(hash) => {
                    hash_map
                        .entry(hash)
                        .or_default()
                        .push((path.display().to_string(), size));
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }
        // Skip symlinks and other special file types.
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// 6. file_watch_work
// ---------------------------------------------------------------------------

/// Return file/directory metadata snapshot: mtime, size, sha256 hash.
///
/// This is a sync helper for the watch pattern — callers can compare
/// snapshots across time to detect changes.
///
/// # Parameters
/// - `path` (string, required): File or directory to snapshot.
/// - `compute_hash` (bool, optional): Whether to compute SHA-256 hash
///   for files (default true). Set to false for large files or dirs to
///   skip hashing.
///
/// # Output
/// Returns metadata for the requested path. For directories, includes a
/// recursive listing with per-file metadata.
#[allow(dead_code)]
pub fn file_watch_work(params: Value) -> String {
    let path_str = match require_str(&params, "path") {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };

    let compute_hash = params
        .get("compute_hash")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let path = match safe_path(&path_str) {
        Ok(p) => p,
        Err(e) => return err_json(e),
    };

    if !path.exists() {
        return err_json(format!("Path does not exist: {}", path.display()));
    }

    let snapshot_time = match std::time::SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs(),
        Err(_) => 0,
    };

    if path.is_file() {
        let meta = match fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => return err_json(format!("Metadata error: {e}")),
        };

        let size = meta.len();
        let mtime = get_mtime_secs(&meta);

        let mut data = serde_json::json!({
            "path": path.display().to_string(),
            "type": "file",
            "size": size,
            "mtime_unix": mtime,
            "snapshot_time": snapshot_time,
        });

        if compute_hash {
            match sha256_file(&path) {
                Ok(hash) => {
                    data.as_object_mut()
                        .unwrap()
                        .insert("sha256".into(), serde_json::Value::String(hash));
                }
                Err(e) => {
                    data.as_object_mut()
                        .unwrap()
                        .insert("sha256_error".into(), serde_json::Value::String(e));
                }
            }
        }

        ok_json(data)
    } else if path.is_dir() {
        let dir_meta = match fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => return err_json(format!("Metadata error: {e}")),
        };

        let dir_mtime = get_mtime_secs(&dir_meta);
        let mut file_count: u64 = 0;
        let mut dir_count: u64 = 0;
        let mut total_size: u64 = 0;
        let mut entries: Vec<serde_json::Value> = Vec::new();

        if let Err(e) = walk_snapshot(
            &path,
            compute_hash,
            &mut file_count,
            &mut dir_count,
            &mut total_size,
            &mut entries,
        ) {
            return err_json(e);
        }

        ok_json(serde_json::json!({
            "path": path.display().to_string(),
            "type": "directory",
            "mtime_unix": dir_mtime,
            "snapshot_time": snapshot_time,
            "file_count": file_count,
            "dir_count": dir_count,
            "total_size": total_size,
            "entries": entries,
        }))
    } else {
        err_json(format!(
            "Path is neither file nor directory: {}",
            path.display()
        ))
    }
}

/// Extract mtime as unix seconds from file metadata.
fn get_mtime_secs(meta: &fs::Metadata) -> u64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Recursively walk a directory and collect metadata snapshots for each
/// entry (up to 500 entries to keep response bounded).
fn walk_snapshot(
    dir: &Path,
    compute_hash: bool,
    file_count: &mut u64,
    dir_count: &mut u64,
    total_size: &mut u64,
    entries: &mut Vec<serde_json::Value>,
) -> Result<(), String> {
    let dir_entries = fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;

    for entry in dir_entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if entries.len() >= 500 {
            break;
        }

        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        if file_type.is_dir() {
            *dir_count += 1;
            let meta = match fs::metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            let mtime = get_mtime_secs(&meta);

            entries.push(serde_json::json!({
                "path": path.display().to_string(),
                "type": "directory",
                "mtime_unix": mtime,
            }));

            // Recurse into subdirectory.
            walk_snapshot(
                &path,
                compute_hash,
                file_count,
                dir_count,
                total_size,
                entries,
            )?;
        } else if file_type.is_file() {
            *file_count += 1;
            let meta = match fs::metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            let size = meta.len();
            let mtime = get_mtime_secs(&meta);
            *total_size += size;

            let mut entry_data = serde_json::json!({
                "path": path.display().to_string(),
                "type": "file",
                "size": size,
                "mtime_unix": mtime,
            });

            if compute_hash {
                match sha256_file(&path) {
                    Ok(hash) => {
                        entry_data
                            .as_object_mut()
                            .unwrap()
                            .insert("sha256".into(), serde_json::Value::String(hash));
                    }
                    Err(e) => {
                        entry_data
                            .as_object_mut()
                            .unwrap()
                            .insert("sha256_error".into(), serde_json::Value::String(e));
                    }
                }
            }

            entries.push(entry_data);
        }
        // Skip symlinks and other special file types.
    }

    Ok(())
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // ---- helpers ----

    /// Create a temporary file with the given content and return its path.
    fn temp_file(name: &str, content: &[u8]) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::with_prefix(name).expect("temp file");
        f.write_all(content).expect("write temp");
        f.flush().expect("flush");
        f
    }

    /// Parse a JSON response string into a `Value`.
    fn parse_resp(s: &str) -> Value {
        serde_json::from_str(s).expect("response should be valid JSON")
    }

    // ---- file_encrypt_work ----

    #[test]
    fn encrypt_missing_path_param() {
        let result = file_encrypt_work(serde_json::json!({"passphrase": "secret"}));
        let v = parse_resp(&result);
        assert!(!v["ok"].as_bool().unwrap());
        assert!(v["error"].as_str().unwrap().contains("path"));
    }

    #[test]
    fn encrypt_missing_passphrase_param() {
        let result = file_encrypt_work(serde_json::json!({"path": "/tmp/test.txt"}));
        let v = parse_resp(&result);
        assert!(!v["ok"].as_bool().unwrap());
        assert!(v["error"].as_str().unwrap().contains("passphrase"));
    }

    #[test]
    fn encrypt_nonexistent_file() {
        let result = file_encrypt_work(serde_json::json!({
            "path": "/nonexistent/abc123.txt",
            "passphrase": "secret"
        }));
        let v = parse_resp(&result);
        assert!(!v["ok"].as_bool().unwrap());
    }

    #[test]
    fn encrypt_roundtrip() {
        let f = temp_file("encrypt_test", b"Hello, encryption world!");
        let path_str = f.path().display().to_string();

        // Encrypt
        let enc_result = file_encrypt_work(serde_json::json!({
            "path": &path_str,
            "passphrase": "mysecret"
        }));
        let enc_v = parse_resp(&enc_result);
        assert!(enc_v["ok"].as_bool().unwrap());
        let out_path = enc_v["data"]["output_path"].as_str().unwrap();
        assert!(out_path.ends_with(".enc"));
        assert!(Path::new(out_path).is_file());

        // Verify the encrypted file starts with the header.
        let enc_content = fs::read_to_string(out_path).unwrap();
        assert!(enc_content.starts_with("ENCV1:"));

        // Decrypt
        let dec_result = file_decrypt_work(serde_json::json!({
            "path": out_path,
            "passphrase": "mysecret"
        }));
        let dec_v = parse_resp(&dec_result);
        assert!(dec_v["ok"].as_bool().unwrap());
        let dec_path = dec_v["data"]["output_path"].as_str().unwrap();
        assert!(!dec_path.ends_with(".enc"));

        // Verify decrypted content matches original.
        let dec_content = fs::read(dec_path).unwrap();
        assert_eq!(dec_content, b"Hello, encryption world!");

        // Clean up .enc file
        let _ = fs::remove_file(out_path);
        let _ = fs::remove_file(dec_path);
    }

    #[test]
    fn encrypt_wrong_passphrase_fails() {
        let f = temp_file("encrypt_wrong", b"Secret data here");
        let path_str = f.path().display().to_string();

        // Encrypt with "pass1"
        let enc_result = file_encrypt_work(serde_json::json!({
            "path": &path_str,
            "passphrase": "pass1"
        }));
        let enc_v = parse_resp(&enc_result);
        assert!(enc_v["ok"].as_bool().unwrap());
        let out_path = enc_v["data"]["output_path"].as_str().unwrap().to_string();

        // Decrypt with "pass2" — should succeed (no integrity check on XOR)
        // but the content will be garbled.
        let dec_result = file_decrypt_work(serde_json::json!({
            "path": &out_path,
            "passphrase": "pass2"
        }));
        let dec_v = parse_resp(&dec_result);
        assert!(dec_v["ok"].as_bool().unwrap());

        let dec_path = dec_v["data"]["output_path"].as_str().unwrap();
        let dec_content = fs::read(dec_path).unwrap();
        assert_ne!(dec_content, b"Secret data here");

        let _ = fs::remove_file(&out_path);
        let _ = fs::remove_file(dec_path);
    }

    // ---- file_decrypt_work ----

    #[test]
    fn decrypt_non_enc_file() {
        let f = temp_file("not_enc", b"This is just plain text");
        let result = file_decrypt_work(serde_json::json!({
            "path": f.path().display().to_string(),
            "passphrase": "secret"
        }));
        let v = parse_resp(&result);
        assert!(!v["ok"].as_bool().unwrap());
        assert!(v["error"].as_str().unwrap().contains("ENCV1"));
    }

    // ---- file_sign_work ----

    #[test]
    fn sign_missing_passphrase() {
        let f = temp_file("sign_test", b"sign me");
        let result = file_sign_work(serde_json::json!({
            "path": f.path().display().to_string()
        }));
        let v = parse_resp(&result);
        assert!(!v["ok"].as_bool().unwrap());
        assert!(v["error"].as_str().unwrap().contains("passphrase"));
    }

    #[test]
    fn sign_produces_sig_file() {
        let f = temp_file("sign_me", b"Important content to sign");
        let path_str = f.path().display().to_string();

        let result = file_sign_work(serde_json::json!({
            "path": &path_str,
            "passphrase": "signing_key_123"
        }));
        let v = parse_resp(&result);
        assert!(v["ok"].as_bool().unwrap());
        assert_eq!(v["data"]["algorithm"].as_str().unwrap(), "hmac-sha256");

        let sig_path = v["data"]["sig_path"].as_str().unwrap();
        assert!(sig_path.ends_with(".sig"));
        assert!(Path::new(sig_path).is_file());

        // Sig file should contain hmac-sha256: and sha256: lines.
        let sig_content = fs::read_to_string(sig_path).unwrap();
        assert!(sig_content.contains("hmac-sha256:"));
        assert!(sig_content.contains("sha256:"));

        let _ = fs::remove_file(sig_path);
    }

    // ---- file_verify_signature_work ----

    #[test]
    fn verify_correct_signature() {
        let f = temp_file("verify_ok", b"Data to verify");
        let path_str = f.path().display().to_string();

        // Sign the file first.
        let sign_result = file_sign_work(serde_json::json!({
            "path": &path_str,
            "passphrase": "verify_pass"
        }));
        let sign_v = parse_resp(&sign_result);
        assert!(sign_v["ok"].as_bool().unwrap());
        let sig_path = sign_v["data"]["sig_path"].as_str().unwrap().to_string();

        // Verify with the correct passphrase.
        let verify_result = file_verify_signature_work(serde_json::json!({
            "path": &path_str,
            "sig_path": &sig_path,
            "passphrase": "verify_pass"
        }));
        let verify_v = parse_resp(&verify_result);
        assert!(verify_v["ok"].as_bool().unwrap());
        assert!(verify_v["data"]["verified"].as_bool().unwrap());

        let _ = fs::remove_file(&sig_path);
    }

    #[test]
    fn verify_wrong_passphrase() {
        let f = temp_file("verify_bad", b"Data to verify");
        let path_str = f.path().display().to_string();

        // Sign with "correct_pass"
        let sign_result = file_sign_work(serde_json::json!({
            "path": &path_str,
            "passphrase": "correct_pass"
        }));
        let sign_v = parse_resp(&sign_result);
        let sig_path = sign_v["data"]["sig_path"].as_str().unwrap().to_string();

        // Verify with "wrong_pass"
        let verify_result = file_verify_signature_work(serde_json::json!({
            "path": &path_str,
            "sig_path": &sig_path,
            "passphrase": "wrong_pass"
        }));
        let verify_v = parse_resp(&verify_result);
        assert!(verify_v["ok"].as_bool().unwrap());
        assert!(!verify_v["data"]["verified"].as_bool().unwrap());

        let _ = fs::remove_file(&sig_path);
    }

    #[test]
    fn verify_missing_sig_file() {
        let f = temp_file("verify_nosig", b"Data");
        let result = file_verify_signature_work(serde_json::json!({
            "path": f.path().display().to_string(),
            "sig_path": "/nonexistent/file.sig",
            "passphrase": "pass"
        }));
        let v = parse_resp(&result);
        assert!(!v["ok"].as_bool().unwrap());
    }

    #[test]
    fn verify_tampered_file() {
        let f = temp_file("tamper", b"Original content");
        let path_str = f.path().display().to_string();

        // Sign
        let sign_result = file_sign_work(serde_json::json!({
            "path": &path_str,
            "passphrase": "tamper_pass"
        }));
        let sign_v = parse_resp(&sign_result);
        let sig_path = sign_v["data"]["sig_path"].as_str().unwrap().to_string();

        // Tamper with the file (overwrite with different content).
        fs::write(&path_str, b"Tampered content").unwrap();

        // Verify — should fail because content changed.
        let verify_result = file_verify_signature_work(serde_json::json!({
            "path": &path_str,
            "sig_path": &sig_path,
            "passphrase": "tamper_pass"
        }));
        let verify_v = parse_resp(&verify_result);
        assert!(verify_v["ok"].as_bool().unwrap());
        assert!(!verify_v["data"]["verified"].as_bool().unwrap());

        let _ = fs::remove_file(&sig_path);
    }

    // ---- file_deduplicate_work ----

    #[test]
    fn dedup_missing_path() {
        let result = file_deduplicate_work(serde_json::json!({}));
        let v = parse_resp(&result);
        assert!(!v["ok"].as_bool().unwrap());
    }

    #[test]
    fn dedup_not_a_directory() {
        let f = temp_file("notadir", b"I am a file, not a dir");
        let result = file_deduplicate_work(serde_json::json!({
            "path": f.path().display().to_string()
        }));
        let v = parse_resp(&result);
        assert!(!v["ok"].as_bool().unwrap());
    }

    #[test]
    fn dedup_finds_duplicates() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let sub = tmp.path().join("sub");
        fs::create_dir_all(&sub).expect("mkdir");

        // Write identical content in two different files.
        let content = b"duplicate content here";
        fs::write(tmp.path().join("a.txt"), content).expect("write a");
        fs::write(sub.join("b.txt"), content).expect("write b");

        // Write a unique file.
        fs::write(tmp.path().join("c.txt"), b"unique content").expect("write c");

        let result = file_deduplicate_work(serde_json::json!({
            "path": tmp.path().display().to_string()
        }));
        let v = parse_resp(&result);
        assert!(v["ok"].as_bool().unwrap());

        let data = &v["data"];
        assert_eq!(data["total_files_scanned"].as_u64().unwrap(), 3);
        assert_eq!(data["duplicate_groups"].as_u64().unwrap(), 1);
        assert_eq!(data["duplicate_files"].as_u64().unwrap(), 2);

        let groups = data["groups"].as_array().unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0]["count"].as_u64().unwrap(), 2);
    }

    #[test]
    fn dedup_no_duplicates() {
        let tmp = tempfile::tempdir().expect("tempdir");
        fs::write(tmp.path().join("a.txt"), b"content A").expect("write");
        fs::write(tmp.path().join("b.txt"), b"content B").expect("write");

        let result = file_deduplicate_work(serde_json::json!({
            "path": tmp.path().display().to_string()
        }));
        let v = parse_resp(&result);
        assert!(v["ok"].as_bool().unwrap());
        assert_eq!(v["data"]["duplicate_groups"].as_u64().unwrap(), 0);
    }

    // ---- file_watch_work ----

    #[test]
    fn watch_file_snapshot() {
        let f = temp_file("watch_file", b"watched content");
        let result = file_watch_work(serde_json::json!({
            "path": f.path().display().to_string()
        }));
        let v = parse_resp(&result);
        assert!(v["ok"].as_bool().unwrap());

        let data = &v["data"];
        assert_eq!(data["type"].as_str().unwrap(), "file");
        assert!(data["size"].as_u64().unwrap() > 0);
        assert!(data["mtime_unix"].as_u64().unwrap() > 0);
        assert!(data.get("sha256").is_some());
    }

    #[test]
    fn watch_file_skip_hash() {
        let f = temp_file("watch_nohash", b"no hash needed");
        let result = file_watch_work(serde_json::json!({
            "path": f.path().display().to_string(),
            "compute_hash": false
        }));
        let v = parse_resp(&result);
        assert!(v["ok"].as_bool().unwrap());
        assert!(v["data"].get("sha256").is_none());
    }

    #[test]
    fn watch_directory_snapshot() {
        let tmp = tempfile::tempdir().expect("tempdir");
        fs::write(tmp.path().join("a.txt"), b"aaa").expect("write");
        fs::write(tmp.path().join("b.txt"), b"bbb").expect("write");

        let result = file_watch_work(serde_json::json!({
            "path": tmp.path().display().to_string(),
            "compute_hash": false
        }));
        let v = parse_resp(&result);
        assert!(v["ok"].as_bool().unwrap());

        let data = &v["data"];
        assert_eq!(data["type"].as_str().unwrap(), "directory");
        assert_eq!(data["file_count"].as_u64().unwrap(), 2);
        assert!(data.get("entries").unwrap().as_array().unwrap().len() >= 2);
    }

    #[test]
    fn watch_nonexistent_path() {
        let result = file_watch_work(serde_json::json!({
            "path": "/nonexistent/path/xyz"
        }));
        let v = parse_resp(&result);
        assert!(!v["ok"].as_bool().unwrap());
    }

    // ---- helper tests ----

    #[test]
    fn derive_key_deterministic() {
        let k1 = derive_key("hello");
        let k2 = derive_key("hello");
        assert_eq!(k1, k2);

        let k3 = derive_key("different");
        assert_ne!(k1, k3);
    }

    #[test]
    fn xor_cycle_roundtrip() {
        let key = derive_key("test");
        let data = b"Hello, XOR roundtrip test!";
        let encrypted = xor_cycle(data, &key);
        assert_ne!(encrypted.as_slice(), data);
        let decrypted = xor_cycle(&encrypted, &key);
        assert_eq!(decrypted.as_slice(), data);
    }

    #[test]
    fn hex_encode_known() {
        assert_eq!(hex_encode(&[0x00, 0xff, 0xab]), "00ffab");
        assert_eq!(hex_encode(&[]), "");
    }

    #[test]
    fn sha256_file_known_content() {
        let f = temp_file("sha_test", b"hello world");
        let hash = sha256_file(f.path()).unwrap();
        // SHA-256 of "hello world" (no trailing newline) is well-known.
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn safe_path_rejects_traversal() {
        assert!(safe_path("../../etc/passwd").is_err());
        assert!(safe_path("foo/bar").is_ok());
        assert!(safe_path("/tmp/file").is_ok());
    }

    #[test]
    fn err_ok_json_format() {
        let err = err_json("test error");
        let v: Value = serde_json::from_str(&err).unwrap();
        assert!(!v["ok"].as_bool().unwrap());
        assert_eq!(v["error"].as_str().unwrap(), "test error");

        let ok = ok_json(serde_json::json!({"x": 42}));
        let v: Value = serde_json::from_str(&ok).unwrap();
        assert!(v["ok"].as_bool().unwrap());
        assert_eq!(v["data"]["x"].as_u64().unwrap(), 42);
    }
}
