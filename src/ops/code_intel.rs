//! # Code Intelligence Tools
//!
//! AST-like analysis, import extraction, language detection,
//! complexity scoring, and dead code hints.

use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Maximum bytes to read for code intelligence analysis.
const MAX_CODE_READ_BYTES: u64 = 512 * 1024;

/// Maximum characters for dead-code grep output.
const _MAX_GREP_CHARS: usize = 8_000;

// ---------------------------------------------------------------------------
// Language detection helpers
// ---------------------------------------------------------------------------

/// Language name and confidence level.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Lang {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    C,
    Cpp,
    Ruby,
    Php,
    Swift,
    Kotlin,
    CSharp,
    Shell,
    Sql,
    Unknown,
}

impl std::fmt::Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lang::Rust => write!(f, "Rust"),
            Lang::Python => write!(f, "Python"),
            Lang::JavaScript => write!(f, "JavaScript"),
            Lang::TypeScript => write!(f, "TypeScript"),
            Lang::Go => write!(f, "Go"),
            Lang::Java => write!(f, "Java"),
            Lang::C => write!(f, "C"),
            Lang::Cpp => write!(f, "C++"),
            Lang::Ruby => write!(f, "Ruby"),
            Lang::Php => write!(f, "PHP"),
            Lang::Swift => write!(f, "Swift"),
            Lang::Kotlin => write!(f, "Kotlin"),
            Lang::CSharp => write!(f, "C#"),
            Lang::Shell => write!(f, "Shell"),
            Lang::Sql => write!(f, "SQL"),
            Lang::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Detect language primarily from file extension.
fn lang_from_extension(path: &Path) -> Lang {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("rs") => Lang::Rust,
        Some("py") | Some("pyw") => Lang::Python,
        Some("js") | Some("mjs") | Some("cjs") => Lang::JavaScript,
        Some("ts") | Some("tsx") => Lang::TypeScript,
        Some("go") => Lang::Go,
        Some("java") => Lang::Java,
        Some("c") | Some("h") => Lang::C,
        Some("cpp") | Some("cc") | Some("cxx") | Some("hpp") | Some("hxx") => Lang::Cpp,
        Some("rb") => Lang::Ruby,
        Some("php") => Lang::Php,
        Some("swift") => Lang::Swift,
        Some("kt") | Some("kts") => Lang::Kotlin,
        Some("cs") => Lang::CSharp,
        Some("sh") | Some("bash") | Some("zsh") => Lang::Shell,
        Some("sql") => Lang::Sql,
        _ => Lang::Unknown,
    }
}

/// Detect language from content heuristics (secondary check).
fn lang_from_content(content: &str) -> Option<Lang> {
    let mut scores: HashMap<Lang, usize> = HashMap::new();

    // Rust
    if content.contains("fn ") && content.contains("let ") {
        *scores.entry(Lang::Rust).or_insert(0) += 2;
    }
    if content.contains("impl ") || content.contains("pub fn ") || content.contains("mut ") {
        *scores.entry(Lang::Rust).or_insert(0) += 2;
    }
    if content.contains("::") && content.contains("&self") {
        *scores.entry(Lang::Rust).or_insert(0) += 1;
    }

    // Python
    if content.contains("def ") && content.contains("import ") {
        *scores.entry(Lang::Python).or_insert(0) += 2;
    }
    if content.contains("from ") && content.contains(" import") {
        *scores.entry(Lang::Python).or_insert(0) += 2;
    }
    if content.contains("class ") && content.contains("self.") {
        *scores.entry(Lang::Python).or_insert(0) += 1;
    }
    if content.contains("if __name__") {
        *scores.entry(Lang::Python).or_insert(0) += 3;
    }

    // JavaScript
    if content.contains("function ") && content.contains("const ") {
        *scores.entry(Lang::JavaScript).or_insert(0) += 1;
    }
    if content.contains("require(") || content.contains("module.exports") {
        *scores.entry(Lang::JavaScript).or_insert(0) += 2;
    }
    if content.contains("=>") && content.contains("const ") {
        *scores.entry(Lang::JavaScript).or_insert(0) += 1;
    }

    // TypeScript
    if content.contains(": string") || content.contains(": number") || content.contains(": boolean")
    {
        *scores.entry(Lang::TypeScript).or_insert(0) += 2;
    }
    if content.contains("interface ") && content.contains("{") {
        *scores.entry(Lang::TypeScript).or_insert(0) += 2;
    }

    // Go
    if content.contains("func ") && content.contains("package ") {
        *scores.entry(Lang::Go).or_insert(0) += 3;
    }
    if content.contains(":= ") || content.contains("go func") {
        *scores.entry(Lang::Go).or_insert(0) += 2;
    }
    if content.contains("chan ") || content.contains("goroutine") {
        *scores.entry(Lang::Go).or_insert(0) += 2;
    }

    // Java
    if content.contains("public class ") || content.contains("private void ") {
        *scores.entry(Lang::Java).or_insert(0) += 2;
    }
    if content.contains("package ") && content.contains("import java.") {
        *scores.entry(Lang::Java).or_insert(0) += 3;
    }

    // C
    if content.contains("#include <") && content.contains("int main(") {
        *scores.entry(Lang::C).or_insert(0) += 3;
    }
    if content.contains("printf(") && content.contains("void ") {
        *scores.entry(Lang::C).or_insert(0) += 1;
    }

    // C++
    if content.contains("#include <iostream>") || content.contains("std::") {
        *scores.entry(Lang::Cpp).or_insert(0) += 2;
    }
    if content.contains("class ") && content.contains("::") && !content.contains("fn ") {
        *scores.entry(Lang::Cpp).or_insert(0) += 1;
    }

    // Ruby
    if content.contains("def ") && content.contains("end") && content.contains("require '") {
        *scores.entry(Lang::Ruby).or_insert(0) += 2;
    }
    if content.contains("class ") && content.contains(" < ") && content.contains("end") {
        *scores.entry(Lang::Ruby).or_insert(0) += 1;
    }

    // Return the highest scoring language
    scores
        .into_iter()
        .filter(|(_, s)| *s >= 2)
        .max_by_key(|(_, s)| *s)
        .map(|(l, _)| l)
}

/// Read file content with size limits.
fn read_code(path: &Path) -> Result<String, String> {
    let meta = fs::metadata(path).map_err(|e| format!("Error: {e}"))?;
    if meta.len() > MAX_CODE_READ_BYTES {
        return Err(format!(
            "File too large ({} bytes; max {})",
            meta.len(),
            MAX_CODE_READ_BYTES
        ));
    }
    fs::read_to_string(path).map_err(|e| format!("Error reading {}: {e}", path.display()))
}

// ---------------------------------------------------------------------------
// 1. code_ast_dump
// ---------------------------------------------------------------------------

/// Parse a source file and dump a simplified AST-like structure.
///
/// Uses regex-based pattern matching (not a real parser) to identify
/// function/method definitions, struct/class/interface definitions,
/// enum/union definitions, import/use/require statements, and
/// const/let/var declarations at module level.
pub fn code_ast_dump_work(path: PathBuf, language: Option<String>) -> String {
    let content = match read_code(&path) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let lang = if let Some(ref l) = language {
        match l.to_lowercase().as_str() {
            "rust" => Lang::Rust,
            "python" | "py" => Lang::Python,
            "javascript" | "js" => Lang::JavaScript,
            "typescript" | "ts" => Lang::TypeScript,
            "go" => Lang::Go,
            "java" => Lang::Java,
            "c" => Lang::C,
            "cpp" | "c++" => Lang::Cpp,
            "ruby" | "rb" => Lang::Ruby,
            "php" => Lang::Php,
            "swift" => Lang::Swift,
            "kotlin" | "kt" => Lang::Kotlin,
            "csharp" | "c#" => Lang::CSharp,
            _ => lang_from_extension(&path),
        }
    } else {
        lang_from_extension(&path)
    };

    let mut out = String::from("CODE: AST DUMP\n\n");
    out.push_str(&format!("File: {}\n", path.display()));
    out.push_str(&format!("Language: {lang}\n"));
    out.push_str(&format!("Lines: {}\n\n", content.lines().count()));

    let items = extract_ast_items(&content, &lang);

    if items.is_empty() {
        out.push_str("No structural items detected (may need a different language hint).\n");
    } else {
        for item in &items {
            out.push_str(&format!(
                "{}: {} (line {})\n",
                item.kind, item.name, item.line
            ));
            if !item.signature.is_empty() {
                out.push_str(&format!("  signature: {}\n", item.signature));
            }
        }
    }

    out.push_str(&format!("\nTotal items: {}\n", items.len()));

    let (out, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
    out
}

/// An AST-like item extracted from source code.
#[derive(Debug, Clone)]
struct AstItem {
    kind: String,
    name: String,
    line: usize,
    signature: String,
}

/// Extract structural items from source code based on language.
fn extract_ast_items(content: &str, lang: &Lang) -> Vec<AstItem> {
    let mut items = Vec::new();

    match lang {
        Lang::Rust => extract_rust_items(content, &mut items),
        Lang::Python => extract_python_items(content, &mut items),
        Lang::JavaScript | Lang::TypeScript => extract_js_items(content, &mut items),
        Lang::Go => extract_go_items(content, &mut items),
        Lang::Java => extract_java_items(content, &mut items),
        Lang::C | Lang::Cpp => extract_c_items(content, &mut items),
        Lang::Ruby => extract_ruby_items(content, &mut items),
        Lang::Php => extract_php_items(content, &mut items),
        Lang::Swift => extract_swift_items(content, &mut items),
        Lang::Kotlin => extract_kotlin_items(content, &mut items),
        _ => {
            // Generic: try Rust and Python patterns
            extract_rust_items(content, &mut items);
            extract_python_items(content, &mut items);
        }
    }

    items.sort_by_key(|i| i.line);
    items
}

fn extract_rust_items(content: &str, items: &mut Vec<AstItem>) {
    // use statements
    if let Ok(re) = Regex::new(r"^\s*(pub\s+)?use\s+(.+?)\s*;") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "use".to_string(),
                    name: caps[2].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // fn definitions
    if let Ok(re) = Regex::new(r"^\s*(pub\s+)?(async\s+)?fn\s+(\w+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "fn".to_string(),
                    name: caps[3].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // struct definitions
    if let Ok(re) = Regex::new(r"^\s*(pub\s+)?struct\s+(\w+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "struct".to_string(),
                    name: caps[2].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // enum definitions
    if let Ok(re) = Regex::new(r"^\s*(pub\s+)?enum\s+(\w+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "enum".to_string(),
                    name: caps[2].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // impl blocks
    if let Ok(re) = Regex::new(r"^\s*impl\s+(.+?)\s*\{") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "impl".to_string(),
                    name: caps[1].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // trait definitions
    if let Ok(re) = Regex::new(r"^\s*(pub\s+)?trait\s+(\w+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "trait".to_string(),
                    name: caps[2].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // const / static
    if let Ok(re) = Regex::new(r"^\s*(pub\s+)?(const|static)\s+(\w+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: caps[2].to_string(),
                    name: caps[3].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // type aliases
    if let Ok(re) = Regex::new(r"^\s*(pub\s+)?type\s+(\w+)\s*=") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "type".to_string(),
                    name: caps[2].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }
}

fn extract_python_items(content: &str, items: &mut Vec<AstItem>) {
    // import / from ... import
    if let Ok(re) = Regex::new(r"^\s*(import|from)\s+(\S+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "import".to_string(),
                    name: caps[2].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // def / async def
    if let Ok(re) = Regex::new(r"^\s*(async\s+)?def\s+(\w+)\s*\(") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "def".to_string(),
                    name: caps[2].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // class
    if let Ok(re) = Regex::new(r"^\s*class\s+(\w+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "class".to_string(),
                    name: caps[1].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // module-level variables (ALL_CAPS convention)
    if let Ok(re) = Regex::new(r"^([A-Z][A-Z0-9_]*)\s*=") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "const".to_string(),
                    name: caps[1].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }
}

fn extract_js_items(content: &str, items: &mut Vec<AstItem>) {
    // import / require
    if let Ok(re) = Regex::new(r#"^\s*(import\s+.+?from\s+['\"]|const\s+\w+\s*=\s*require\s*\()"#) {
        for (i, line) in content.lines().enumerate() {
            if re.is_match(line) {
                items.push(AstItem {
                    kind: "import".to_string(),
                    name: line.trim().to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // function / async function
    if let Ok(re) = Regex::new(r"^\s*(export\s+)?(async\s+)?function\s+(\w+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "function".to_string(),
                    name: caps[3].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // class
    if let Ok(re) = Regex::new(r"^\s*(export\s+)?(default\s+)?class\s+(\w+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "class".to_string(),
                    name: caps[3].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // const / let / var (module-level)
    if let Ok(re) = Regex::new(r"^\s*(export\s+)?(const|let|var)\s+(\w+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                let indent = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
                if indent <= 2 {
                    items.push(AstItem {
                        kind: caps[2].to_string(),
                        name: caps[3].to_string(),
                        line: i + 1,
                        signature: line.trim().to_string(),
                    });
                }
            }
        }
    }

    // interface / type (TypeScript)
    if let Ok(re) = Regex::new(r"^\s*(export\s+)?(interface|type)\s+(\w+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: caps[2].to_string(),
                    name: caps[3].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // export
    if let Ok(re) = Regex::new(r"^\s*export\s+default\s+(\w+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "export_default".to_string(),
                    name: caps[1].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }
}

fn extract_go_items(content: &str, items: &mut Vec<AstItem>) {
    // package
    if let Ok(re) = Regex::new(r"^\s*package\s+(\w+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "package".to_string(),
                    name: caps[1].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // import
    if let Ok(re) = Regex::new(r#"^\s*import\s+[\"`](.+?)[\"`]"#) {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "import".to_string(),
                    name: caps[1].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // func
    if let Ok(re) = Regex::new(r"^\s*func\s+(?:\(\w+\s+\*?\w+\)\s+)?(\w+)\s*\(") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "func".to_string(),
                    name: caps[1].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // type ... struct / interface
    if let Ok(re) = Regex::new(r"^\s*type\s+(\w+)\s+(struct|interface)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: caps[2].to_string(),
                    name: caps[1].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // var / const at package level
    if let Ok(re) = Regex::new(r"^\s*(var|const)\s+(\w+)\s+") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                let indent = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
                if indent <= 1 {
                    items.push(AstItem {
                        kind: caps[1].to_string(),
                        name: caps[2].to_string(),
                        line: i + 1,
                        signature: line.trim().to_string(),
                    });
                }
            }
        }
    }
}

fn extract_java_items(content: &str, items: &mut Vec<AstItem>) {
    // package
    if let Ok(re) = Regex::new(r"^\s*package\s+(.+?)\s*;") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "package".to_string(),
                    name: caps[1].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // import
    if let Ok(re) = Regex::new(r"^\s*import\s+(?:static\s+)?(.+?)\s*;") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "import".to_string(),
                    name: caps[1].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // class / interface / enum
    if let Ok(re) = Regex::new(
        r"^\s*(public|private|protected)?\s*(static)?\s*(abstract|final)?\s*(class|interface|enum)\s+(\w+)",
    ) {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: caps[4].to_string(),
                    name: caps[5].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // method
    if let Ok(re) =
        Regex::new(r"^\s*(public|private|protected)?\s*(static)?\s*(\w+(?:<[^>]+>)?)\s+(\w+)\s*\(")
    {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                let return_type = caps[3].to_string();
                let name = caps[4].to_string();
                // Filter out common non-method patterns
                if !["if", "for", "while", "switch", "catch", "return", "new"]
                    .contains(&name.as_str())
                {
                    items.push(AstItem {
                        kind: "method".to_string(),
                        name: format!("{return_type} {name}"),
                        line: i + 1,
                        signature: line.trim().to_string(),
                    });
                }
            }
        }
    }
}

fn extract_c_items(content: &str, items: &mut Vec<AstItem>) {
    // #include
    if let Ok(re) = Regex::new(r#"^\s*#include\s+(<.+?>|\".+?\")"#) {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "include".to_string(),
                    name: caps[1].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // function definitions (return_type name(...))
    if let Ok(re) = Regex::new(r"^\s*(?:inline\s+)?(\w[\w\s\*]+?)\s+(\w+)\s*\(") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                let return_type = caps[1].trim().to_string();
                let name = caps[2].to_string();
                // Filter out keywords and preprocessor directives
                if ![
                    "if", "for", "while", "switch", "return", "typedef", "struct", "enum", "class",
                ]
                .contains(&name.as_str())
                    && !return_type.starts_with('#')
                {
                    items.push(AstItem {
                        kind: "function".to_string(),
                        name: format!("{return_type} {name}"),
                        line: i + 1,
                        signature: line.trim().to_string(),
                    });
                }
            }
        }
    }

    // struct / class / enum
    if let Ok(re) = Regex::new(r"^\s*(typedef\s+)?(struct|class|enum|union)\s+(\w+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: caps[2].to_string(),
                    name: caps[3].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // #define
    if let Ok(re) = Regex::new(r"^\s*#define\s+(\w+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "define".to_string(),
                    name: caps[1].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }
}

fn extract_ruby_items(content: &str, items: &mut Vec<AstItem>) {
    // require / require_relative
    if let Ok(re) = Regex::new(r#"^\s*(require|require_relative)\s+['\"](.+?)['\"]"#) {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "require".to_string(),
                    name: caps[2].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // def / define_method
    if let Ok(re) = Regex::new(r"^\s*(def|define_method)\s+:?(\w+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "def".to_string(),
                    name: caps[2].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // class / module
    if let Ok(re) = Regex::new(r"^\s*(class|module)\s+(\w+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: caps[1].to_string(),
                    name: caps[2].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }
}

fn extract_php_items(content: &str, items: &mut Vec<AstItem>) {
    // use / require / include
    if let Ok(re) = Regex::new(r#"^\s*(use|require|include(?:_once)?)\s+['\"](.+?)['\"]"#) {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "import".to_string(),
                    name: caps[2].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // function
    if let Ok(re) = Regex::new(r"^\s*function\s+(\w+)\s*\(") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "function".to_string(),
                    name: caps[1].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // class / interface / trait
    if let Ok(re) = Regex::new(r"^\s*(class|interface|trait)\s+(\w+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: caps[1].to_string(),
                    name: caps[2].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }
}

fn extract_swift_items(content: &str, items: &mut Vec<AstItem>) {
    // import
    if let Ok(re) = Regex::new(r"^\s*import\s+(\w+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "import".to_string(),
                    name: caps[1].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // func
    if let Ok(re) = Regex::new(r"^\s*(public\s+|private\s+|internal\s+|static\s+)?(func)\s+(\w+)") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "func".to_string(),
                    name: caps[3].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // struct / class / enum / protocol
    if let Ok(re) =
        Regex::new(r"^\s*(public\s+|private\s+|internal\s+)?(struct|class|enum|protocol)\s+(\w+)")
    {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: caps[2].to_string(),
                    name: caps[3].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // var / let at top level
    if let Ok(re) = Regex::new(r"^\s*(public\s+|private\s+)?(let|var)\s+(\w+)\s*:") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                let indent = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
                if indent <= 2 {
                    items.push(AstItem {
                        kind: caps[2].to_string(),
                        name: caps[3].to_string(),
                        line: i + 1,
                        signature: line.trim().to_string(),
                    });
                }
            }
        }
    }
}

fn extract_kotlin_items(content: &str, items: &mut Vec<AstItem>) {
    // import
    if let Ok(re) = Regex::new(r"^\s*import\s+(.+?)\s*$") {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "import".to_string(),
                    name: caps[1].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // fun
    if let Ok(re) =
        Regex::new(r"^\s*(public\s+|private\s+|internal\s+|override\s+)*(suspend\s+)?fun\s+(\w+)")
    {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: "fun".to_string(),
                    name: caps[3].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }

    // class / interface / object / enum class
    if let Ok(re) = Regex::new(
        r"^\s*(public\s+|private\s+|internal\s+|open\s+|abstract\s+)*(class|interface|object|enum\s+class)\s+(\w+)",
    ) {
        for (i, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                items.push(AstItem {
                    kind: caps[2].replace("class", "enum_class").replace(" ", "_"),
                    name: caps[3].to_string(),
                    line: i + 1,
                    signature: line.trim().to_string(),
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 2. code_extract_imports
// ---------------------------------------------------------------------------

/// Extract all import/use/require/include statements from a source file.
///
/// Returns each import on its own line, grouped by type (stdlib, external, local/project).
pub fn code_extract_imports_work(path: PathBuf) -> String {
    let content = match read_code(&path) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let lang = lang_from_extension(&path);
    let mut imports: Vec<(String, usize, String)> = Vec::new(); // (module, line, raw)

    match lang {
        Lang::Rust => {
            if let Ok(re) = Regex::new(r"^\s*(pub\s+)?use\s+(.+?)\s*;") {
                for (i, line) in content.lines().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        imports.push((caps[2].to_string(), i + 1, line.trim().to_string()));
                    }
                }
            }
        }
        Lang::Python => {
            if let Ok(re) = Regex::new(r"^\s*(import|from)\s+(\S+)(?:\s+import\s+(.+))?") {
                for (i, line) in content.lines().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        let module = if &caps[1] == "from" {
                            format!(
                                "{}.{}",
                                &caps[2],
                                caps.get(3).map(|m| m.as_str()).unwrap_or("*")
                            )
                        } else {
                            caps[2].to_string()
                        };
                        imports.push((module, i + 1, line.trim().to_string()));
                    }
                }
            }
        }
        Lang::Go => {
            if let Ok(re) = Regex::new(r#"^\s*import\s+[\"`](.+?)[\"`]"#) {
                for (i, line) in content.lines().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        imports.push((caps[1].to_string(), i + 1, line.trim().to_string()));
                    }
                }
            }
            // Multi-line import blocks
            if let Ok(re) = Regex::new(r#"[\"`](.+?)[\"`]"#) {
                let _in_import_block = false;
                let mut block_start = 0usize;
                for (i, line) in content.lines().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("import (") {
                        block_start = i + 1;
                        continue;
                    }
                    if block_start > 0 && trimmed == ")" {
                        block_start = 0;
                        continue;
                    }
                    if block_start > 0 {
                        if let Some(caps) = re.captures(trimmed) {
                            imports.push((caps[1].to_string(), i + 1, line.trim().to_string()));
                        }
                    }
                }
            }
        }
        Lang::Java => {
            if let Ok(re) = Regex::new(r"^\s*import\s+(?:static\s+)?(.+?)\s*;") {
                for (i, line) in content.lines().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        imports.push((caps[1].to_string(), i + 1, line.trim().to_string()));
                    }
                }
            }
        }
        Lang::C | Lang::Cpp => {
            if let Ok(re) = Regex::new(r#"^\s*#include\s+(<.+?>|\".+?\")"#) {
                for (i, line) in content.lines().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        imports.push((caps[1].to_string(), i + 1, line.trim().to_string()));
                    }
                }
            }
        }
        Lang::Ruby => {
            if let Ok(re) = Regex::new(r#"^\s*(require|require_relative)\s+['\"](.+?)['\"]"#) {
                for (i, line) in content.lines().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        imports.push((caps[2].to_string(), i + 1, line.trim().to_string()));
                    }
                }
            }
        }
        Lang::JavaScript | Lang::TypeScript => {
            if let Ok(re) = Regex::new(r#"^\s*import\s+.+?\s+from\s+['\"](.+?)['\"]"#) {
                for (i, line) in content.lines().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        imports.push((caps[1].to_string(), i + 1, line.trim().to_string()));
                    }
                }
            }
            if let Ok(re) = Regex::new(r#"^\s*const\s+\w+\s*=\s*require\s*\(\s*['\"](.+?)['\"]"#) {
                for (i, line) in content.lines().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        imports.push((caps[1].to_string(), i + 1, line.trim().to_string()));
                    }
                }
            }
        }
        _ => {
            // Generic: try #include and import patterns
            if let Ok(re) = Regex::new(r"^\s*(#include|import|require|use)\s+") {
                for (i, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        imports.push((line.trim().to_string(), i + 1, line.trim().to_string()));
                    }
                }
            }
        }
    }

    if imports.is_empty() {
        return format!(
            "CODE: EXTRACT IMPORTS\n\nFile: {}\nNo imports found.",
            path.display()
        );
    }

    // Classify imports
    let stdlib_patterns = get_stdlib_patterns(&lang);
    let mut stdlib_imports = Vec::new();
    let mut external_imports = Vec::new();
    let mut local_imports = Vec::new();

    for (module, line, raw) in &imports {
        let is_stdlib = stdlib_patterns
            .iter()
            .any(|p| module.starts_with(p) || module.contains(p));
        let is_local = module.starts_with('.')
            || module.starts_with("./")
            || module.starts_with("../")
            || module.starts_with('"')
            || module.starts_with("self::")
            || module.starts_with("crate::")
            || module.starts_with("super::");

        if is_stdlib {
            stdlib_imports.push(format!("  L{line}: {raw}"));
        } else if is_local {
            local_imports.push(format!("  L{line}: {raw}"));
        } else {
            external_imports.push(format!("  L{line}: {raw}"));
        }
    }

    let mut out = String::from("CODE: EXTRACT IMPORTS\n\n");
    out.push_str(&format!("File: {}\n", path.display()));
    out.push_str(&format!("Language: {lang}\n"));
    out.push_str(&format!("Total imports: {}\n\n", imports.len()));

    if !stdlib_imports.is_empty() {
        out.push_str(&format!("Standard library ({}):\n", stdlib_imports.len()));
        for i in &stdlib_imports {
            out.push_str(i);
            out.push('\n');
        }
        out.push('\n');
    }

    if !external_imports.is_empty() {
        out.push_str(&format!(
            "External/third-party ({}):\n",
            external_imports.len()
        ));
        for i in &external_imports {
            out.push_str(i);
            out.push('\n');
        }
        out.push('\n');
    }

    if !local_imports.is_empty() {
        out.push_str(&format!("Local/project ({}):\n", local_imports.len()));
        for i in &local_imports {
            out.push_str(i);
            out.push('\n');
        }
        out.push('\n');
    }

    let (out, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
    out
}

/// Get standard library prefix patterns for a language.
fn get_stdlib_patterns(lang: &Lang) -> Vec<&'static str> {
    match lang {
        Lang::Rust => vec!["std::", "core::", "alloc::", "proc_macro::"],
        Lang::Python => vec![
            "os",
            "sys",
            "io",
            "re",
            "json",
            "math",
            "time",
            "datetime",
            "collections",
            "functools",
            "itertools",
            "pathlib",
            "logging",
            "unittest",
            "hashlib",
            "abc",
            "typing",
            "dataclasses",
            "subprocess",
            "threading",
            "multiprocessing",
        ],
        Lang::Go => vec![
            "fmt", "os", "io", "net", "http", "strings", "strconv", "sync", "context", "time",
            "errors", "bytes", "bufio", "encoding", "crypto",
        ],
        Lang::Java => vec!["java.", "javax.", "sun.", "com.sun."],
        Lang::C => vec![
            "<stdio", "<stdlib", "<string", "<math", "<ctype", "<assert", "<signal", "<time",
            "<errno", "<stddef", "<stdint", "<stdbool",
        ],
        Lang::Cpp => vec![
            "<iostream",
            "<string",
            "<vector",
            "<map",
            "<set",
            "<algorithm",
            "<memory",
            "<thread",
            "<chrono",
            "<fstream",
            "<sstream",
            "<cmath",
            "<cstdio",
            "<cstring",
        ],
        Lang::Ruby => vec![
            "json",
            "optparse",
            "time",
            "date",
            "fileutils",
            "pathname",
            "socket",
            "thread",
            "logger",
            "set",
            "digest",
            "io",
            "stringio",
        ],
        Lang::JavaScript => vec![
            "fs",
            "path",
            "http",
            "https",
            "url",
            "util",
            "stream",
            "crypto",
            "os",
            "net",
            "child_process",
            "events",
            "buffer",
            "assert",
            "querystring",
        ],
        Lang::TypeScript => vec![
            "fs",
            "path",
            "http",
            "https",
            "url",
            "util",
            "stream",
            "crypto",
            "os",
            "net",
            "child_process",
        ],
        _ => vec![],
    }
}

// ---------------------------------------------------------------------------
// 3. code_detect_language
// ---------------------------------------------------------------------------

/// Detect the programming language of a file.
///
/// Uses file extension as primary signal and content heuristics as
/// secondary confirmation. Returns language name, confidence level
/// (high/medium/low), and evidence.
pub fn code_detect_language_work(path: PathBuf, content: Option<String>) -> String {
    let file_content = match &content {
        Some(c) => c.clone(),
        None => match read_code(&path) {
            Ok(c) => c,
            Err(e) => return e,
        },
    };

    let ext_lang = lang_from_extension(&path);
    let content_lang = lang_from_content(&file_content);

    let (detected, confidence, evidence) = match (&ext_lang, &content_lang) {
        (ext, Some(cont)) if ext == cont => (
            ext.clone(),
            "high",
            "Extension and content heuristics agree".to_string(),
        ),
        (ext, Some(cont)) if *ext != Lang::Unknown => (
            ext.clone(),
            "medium",
            format!(
                "Extension suggests {} but content patterns suggest {}",
                ext, cont
            ),
        ),
        (ext, None) if *ext != Lang::Unknown => (
            ext.clone(),
            "high",
            "Detected from file extension only; content heuristics inconclusive".to_string(),
        ),
        (Lang::Unknown, Some(cont)) => (
            cont.clone(),
            "medium",
            "Detected from content heuristics only; no known extension".to_string(),
        ),
        (Lang::Unknown, None) => (
            Lang::Unknown,
            "low",
            "Could not determine language from extension or content".to_string(),
        ),
        _ => (
            ext_lang.clone(),
            "low",
            "Could not determine language with confidence".to_string(),
        ),
    };

    let mut out = String::from("CODE: DETECT LANGUAGE\n\n");
    out.push_str(&format!("File: {}\n", path.display()));
    out.push_str(&format!("Language: {detected}\n"));
    out.push_str(&format!("Confidence: {confidence}\n"));
    out.push_str(&format!("Evidence: {evidence}\n"));

    // Show extension detail
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        out.push_str(&format!("Extension: .{ext}\n"));
    }

    // Show content signals
    if content_lang.is_some() {
        out.push_str("Content signals detected\n");
    } else if !file_content.trim().is_empty() {
        out.push_str("No strong content language signals detected\n");
    }

    out.push_str(&format!("\nFile size: {} bytes\n", file_content.len()));
    out.push_str(&format!("Lines: {}\n", file_content.lines().count()));

    let (out, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
    out
}

// ---------------------------------------------------------------------------
// 4. code_complexity_score
// ---------------------------------------------------------------------------

/// Estimate cyclomatic complexity of a source file.
///
/// Counts decision points (if, elif, else, match, case, &&, ||, for,
/// while, loop, catch, ?, try, guard) and provides a per-function
/// breakdown with risk ratings.
pub fn code_complexity_score_work(path: PathBuf, content: Option<String>) -> String {
    let file_content = match &content {
        Some(c) => c.clone(),
        None => match read_code(&path) {
            Ok(c) => c,
            Err(e) => return e,
        },
    };

    let lang = lang_from_extension(&path);
    let lines = file_content.lines().collect::<Vec<&str>>();
    let total_lines = lines.len();

    // Count decision points
    let decision_patterns: Vec<&str> = match lang {
        Lang::Rust => vec![
            "if ",
            "else if",
            "else {",
            "match ",
            "=>",
            "&&",
            "||",
            "for ",
            "while ",
            "loop {",
            "?;",
            "catch",
            "guard ",
            ".unwrap_or_else",
        ],
        Lang::Python => vec![
            "if ", "elif ", "else:", "match ", "case ", "and ", "or ", "for ", "while ", "try:",
            "except ", "finally:", "assert ", "raise ",
        ],
        Lang::JavaScript | Lang::TypeScript => vec![
            "if ", "else if", "else {", "switch ", "case ", "&&", "||", "for ", "while ", "do {",
            "catch ", "?.", "??", "??=",
        ],
        Lang::Go => vec![
            "if ", "else if", "else {", "switch ", "case ", "&&", "||", "for ", "range ",
            "select ", "go ", "defer ",
        ],
        Lang::Java => vec![
            "if ", "else if", "else {", "switch ", "case ", "&&", "||", "for ", "while ", "do {",
            "catch ", "try ",
        ],
        Lang::C | Lang::Cpp => vec![
            "if ", "else if", "else {", "switch ", "case ", "&&", "||", "for ", "while ", "do {",
            "catch ", "try ", "goto ",
        ],
        _ => vec![
            "if ", "elif ", "else", "switch ", "case ", "&&", "||", "for ", "while ", "catch ",
            "try ",
        ],
    };

    // Identify function boundaries and compute per-function complexity
    let func_starts = find_function_starts(&lines, &lang);
    let mut total_complexity: usize = 1; // Base complexity
    let mut func_breakdown: Vec<(String, usize, usize)> = Vec::new(); // (name, line, complexity)

    let mut comment_lines = 0usize;
    let mut blank_lines = 0usize;
    let mut code_lines = 0usize;

    for line in lines.iter() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            blank_lines += 1;
            continue;
        }
        if trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
            || trimmed.starts_with("--")
            || trimmed.starts_with("///")
            || trimmed.starts_with("//!")
        {
            comment_lines += 1;
            continue;
        }
        code_lines += 1;

        // Count decision points in this line
        let lower = trimmed.to_lowercase();
        for pat in &decision_patterns {
            if lower.contains(pat) {
                total_complexity += 1;
                break; // One decision per line max
            }
        }
    }

    // Per-function complexity
    if func_starts.is_empty() {
        func_breakdown.push(("(entire file)".to_string(), 1, total_complexity));
    } else {
        for (i, (name, start_line)) in func_starts.iter().enumerate() {
            let end_line = if i + 1 < func_starts.len() {
                func_starts[i + 1].1
            } else {
                total_lines
            };

            let mut func_complexity: usize = 1;
            for line_idx in *start_line..end_line.min(total_lines) {
                if line_idx == 0 {
                    continue;
                }
                let line = lines.get(line_idx - 1).unwrap_or(&"");
                let lower = line.trim().to_lowercase();
                for pat in &decision_patterns {
                    if lower.contains(pat) {
                        func_complexity += 1;
                        break;
                    }
                }
            }
            func_breakdown.push((name.clone(), *start_line, func_complexity));
        }
    }

    // Risk rating
    fn risk_rating(complexity: usize) -> &'static str {
        match complexity {
            1..=5 => "LOW (1-5)",
            6..=10 => "MODERATE (6-10)",
            11..=20 => "HIGH (11-20)",
            _ => "VERY HIGH (20+)",
        }
    }

    let mut out = String::from("CODE: COMPLEXITY SCORE\n\n");
    out.push_str(&format!("File: {}\n", path.display()));
    out.push_str(&format!("Language: {lang}\n\n"));

    out.push_str("=== OVERALL ===\n");
    out.push_str(&format!("Total complexity: {total_complexity}\n"));
    out.push_str(&format!(
        "Risk rating: {}\n\n",
        risk_rating(total_complexity)
    ));

    out.push_str("=== LINE STATISTICS ===\n");
    out.push_str(&format!("Total lines: {total_lines}\n"));
    out.push_str(&format!("Code lines: {code_lines}\n"));
    out.push_str(&format!("Comment lines: {comment_lines}\n"));
    out.push_str(&format!("Blank lines: {blank_lines}\n"));
    let comment_ratio = if total_lines > 0 {
        (comment_lines as f64 / total_lines as f64) * 100.0
    } else {
        0.0
    };
    out.push_str(&format!("Comment ratio: {comment_ratio:.1}%\n\n"));

    out.push_str("=== PER-FUNCTION BREAKDOWN ===\n");
    for (name, line, complexity) in &func_breakdown {
        let rating = risk_rating(*complexity);
        out.push_str(&format!(
            " L{line:>4} {name:<40} complexity={complexity:>3} [{rating}]\n"
        ));
    }

    // Highlight high-complexity functions
    let high_complexity: Vec<_> = func_breakdown.iter().filter(|(_, _, c)| *c > 10).collect();
    if !high_complexity.is_empty() {
        out.push_str("\n⚠ HIGH COMPLEXITY FUNCTIONS:\n");
        for (name, line, complexity) in &high_complexity {
            out.push_str(&format!(
                "  {name} (line {line}) — complexity {complexity}\n"
            ));
        }
    }

    let (out, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
    out
}

/// Find function/method definition lines.
fn find_function_starts(lines: &[&str], lang: &Lang) -> Vec<(String, usize)> {
    let mut starts = Vec::new();

    match lang {
        Lang::Rust => {
            if let Ok(re) = Regex::new(r"^\s*(pub\s+)?(async\s+)?fn\s+(\w+)") {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        starts.push((caps[3].to_string(), i + 1));
                    }
                }
            }
        }
        Lang::Python => {
            if let Ok(re) = Regex::new(r"^\s*(async\s+)?def\s+(\w+)") {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        starts.push((caps[2].to_string(), i + 1));
                    }
                }
            }
        }
        Lang::JavaScript | Lang::TypeScript => {
            if let Ok(re) = Regex::new(r"^\s*(export\s+)?(async\s+)?function\s+(\w+)") {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        starts.push((caps[3].to_string(), i + 1));
                    }
                }
            }
            // Also const foo = (args) => ...
            if let Ok(re) = Regex::new(r"^\s*(export\s+)?const\s+(\w+)\s*=\s*(?:async\s+)?\(") {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        starts.push((format!("{} (arrow)", &caps[2]), i + 1));
                    }
                }
            }
        }
        Lang::Go => {
            if let Ok(re) = Regex::new(r"^\s*func\s+(?:\(\w+\s+\*?\w+\)\s+)?(\w+)\s*\(") {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        starts.push((caps[1].to_string(), i + 1));
                    }
                }
            }
        }
        Lang::Java => {
            if let Ok(re) = Regex::new(
                r"^\s*(public|private|protected)\s+(?:static\s+)?(?:\w+(?:<[^>]+>)?)\s+(\w+)\s*\(",
            ) {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        let name = caps[2].to_string();
                        if !["if", "for", "while", "switch", "catch"].contains(&name.as_str()) {
                            starts.push((name, i + 1));
                        }
                    }
                }
            }
        }
        Lang::C | Lang::Cpp => {
            if let Ok(re) = Regex::new(r"^\s*(?:inline\s+)?(\w[\w\s\*]+?)\s+(\w+)\s*\(") {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        let name = caps[2].to_string();
                        if !["if", "for", "while", "switch", "return", "typedef"]
                            .contains(&name.as_str())
                        {
                            starts.push((name, i + 1));
                        }
                    }
                }
            }
        }
        _ => {
            if let Ok(re) = Regex::new(r"^\s*(?:function|def|func|fn)\s+(\w+)") {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        starts.push((caps[1].to_string(), i + 1));
                    }
                }
            }
        }
    }

    starts
}

// ---------------------------------------------------------------------------
// 5. code_dead_code_hints
// ---------------------------------------------------------------------------

/// Find potentially dead code by checking if definitions are referenced elsewhere.
///
/// Reads a source file, identifies all function/method/struct/enum/class
/// definitions, then checks if each name appears in other files in the
/// project using `grep -r`. Definitions that appear only once (where
/// they're defined) are flagged as potentially dead code.
pub fn code_dead_code_hints_work(root: PathBuf, path: PathBuf) -> String {
    let content = match read_code(&path) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let lang = lang_from_extension(&path);
    let lines: Vec<&str> = content.lines().collect();

    // Extract definition names
    let definitions = extract_definition_names(&lines, &lang);

    if definitions.is_empty() {
        return format!(
            "CODE: DEAD CODE HINTS\n\nFile: {}\nNo definitions found to analyze.",
            path.display()
        );
    }

    let mut out = String::from("CODE: DEAD CODE HINTS\n\n");
    out.push_str(&format!("File: {}\n", path.display()));
    out.push_str(&format!("Definitions found: {}\n", definitions.len()));
    out.push_str(&format!("Language: {lang}\n\n"));

    let mut potentially_dead = Vec::new();
    let mut likely_used = Vec::new();
    let mut check_errors = Vec::new();

    // For each definition, grep the codebase for references
    for (name, line_num, kind) in &definitions {
        // Skip very short names (likely false positives)
        if name.len() < 3 {
            continue;
        }

        // Skip main/test/setup functions (common entry points)
        if ["main", "setup", "teardown", "init", "new"].contains(&name.as_str()) {
            likely_used.push((
                name.clone(),
                *line_num,
                kind.clone(),
                "entry point".to_string(),
            ));
            continue;
        }

        // Use grep -r to count references
        let grep_output = Command::new("grep")
            .args([
                "-r",
                "--count",
                "--include=*.rs",
                "--include=*.py",
                "--include=*.js",
                "--include=*.ts",
                "--include=*.go",
                "--include=*.java",
                "--include=*.c",
                "--include=*.cpp",
                "--include=*.rb",
                "-w",
                name,
            ])
            .current_dir(&root)
            .output();

        match grep_output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let file_count = stdout.lines().filter(|l| !l.is_empty()).count();

                if file_count <= 1 {
                    potentially_dead.push((name.clone(), *line_num, kind.clone()));
                } else {
                    likely_used.push((
                        name.clone(),
                        *line_num,
                        kind.clone(),
                        format!("referenced in {file_count} file(s)"),
                    ));
                }
            }
            Err(_) => {
                check_errors.push(name.clone());
            }
        }
    }

    if !potentially_dead.is_empty() {
        out.push_str(&format!(
            "⚠ Potentially dead code ({}):\n",
            potentially_dead.len()
        ));
        for (name, line, kind) in &potentially_dead {
            out.push_str(&format!(
                "  L{line} {kind} {name} — not referenced elsewhere\n"
            ));
        }
        out.push('\n');
    }

    if !likely_used.is_empty() {
        out.push_str(&format!("✓ Likely used ({}):\n", likely_used.len()));
        for (name, line, kind, reason) in &likely_used {
            out.push_str(&format!("  L{line} {kind} {name} — {reason}\n"));
        }
        out.push('\n');
    }

    if !check_errors.is_empty() {
        out.push_str(&format!(
            "Could not check: {} (grep unavailable)\n",
            check_errors.join(", ")
        ));
    }

    out.push_str("\nNote: This is heuristic analysis. Public API items, trait implementations,\n");
    out.push_str("and dynamically-called functions may be false positives.\n");

    let (out, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
    out
}

/// Extract definition names from source code.
fn extract_definition_names(lines: &[&str], lang: &Lang) -> Vec<(String, usize, String)> {
    let mut defs = Vec::new();

    match lang {
        Lang::Rust => {
            if let Ok(re) = Regex::new(r"^\s*(pub\s+)?(async\s+)?fn\s+(\w+)") {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        defs.push((caps[3].to_string(), i + 1, "fn".to_string()));
                    }
                }
            }
            if let Ok(re) = Regex::new(r"^\s*(pub\s+)?struct\s+(\w+)") {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        defs.push((caps[2].to_string(), i + 1, "struct".to_string()));
                    }
                }
            }
            if let Ok(re) = Regex::new(r"^\s*(pub\s+)?enum\s+(\w+)") {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        defs.push((caps[2].to_string(), i + 1, "enum".to_string()));
                    }
                }
            }
            if let Ok(re) = Regex::new(r"^\s*(pub\s+)?trait\s+(\w+)") {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        defs.push((caps[2].to_string(), i + 1, "trait".to_string()));
                    }
                }
            }
        }
        Lang::Python => {
            if let Ok(re) = Regex::new(r"^\s*(async\s+)?def\s+(\w+)") {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        defs.push((caps[2].to_string(), i + 1, "def".to_string()));
                    }
                }
            }
            if let Ok(re) = Regex::new(r"^\s*class\s+(\w+)") {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        defs.push((caps[1].to_string(), i + 1, "class".to_string()));
                    }
                }
            }
        }
        Lang::Go => {
            if let Ok(re) = Regex::new(r"^\s*func\s+(?:\(\w+\s+\*?\w+\)\s+)?(\w+)\s*\(") {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        defs.push((caps[1].to_string(), i + 1, "func".to_string()));
                    }
                }
            }
            if let Ok(re) = Regex::new(r"^\s*type\s+(\w+)\s+(struct|interface)") {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        defs.push((caps[1].to_string(), i + 1, caps[2].to_string()));
                    }
                }
            }
        }
        Lang::JavaScript | Lang::TypeScript => {
            if let Ok(re) = Regex::new(r"^\s*(export\s+)?(async\s+)?function\s+(\w+)") {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        defs.push((caps[3].to_string(), i + 1, "function".to_string()));
                    }
                }
            }
            if let Ok(re) = Regex::new(r"^\s*(export\s+)?class\s+(\w+)") {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        defs.push((caps[2].to_string(), i + 1, "class".to_string()));
                    }
                }
            }
            if let Ok(re) = Regex::new(r"^\s*(export\s+)?const\s+(\w+)\s*=\s*(?:async\s+)?\(") {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        defs.push((caps[2].to_string(), i + 1, "arrow_fn".to_string()));
                    }
                }
            }
        }
        _ => {
            // Generic: look for function definitions
            if let Ok(re) = Regex::new(r"^\s*(?:function|def|func|fn)\s+(\w+)") {
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        defs.push((caps[1].to_string(), i + 1, "function".to_string()));
                    }
                }
            }
        }
    }

    defs
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_temp_file(content: &str, ext: &str) -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join(format!("test.{ext}"));
        fs::write(&path, content).unwrap();
        (temp, path)
    }

    #[test]
    fn test_lang_from_extension() {
        assert_eq!(lang_from_extension(Path::new("foo.rs")), Lang::Rust);
        assert_eq!(lang_from_extension(Path::new("foo.py")), Lang::Python);
        assert_eq!(lang_from_extension(Path::new("foo.js")), Lang::JavaScript);
        assert_eq!(lang_from_extension(Path::new("foo.ts")), Lang::TypeScript);
        assert_eq!(lang_from_extension(Path::new("foo.go")), Lang::Go);
        assert_eq!(lang_from_extension(Path::new("foo.java")), Lang::Java);
        assert_eq!(lang_from_extension(Path::new("foo.c")), Lang::C);
        assert_eq!(lang_from_extension(Path::new("foo.cpp")), Lang::Cpp);
        assert_eq!(lang_from_extension(Path::new("foo.rb")), Lang::Ruby);
        assert_eq!(lang_from_extension(Path::new("foo.unknown")), Lang::Unknown);
        assert_eq!(lang_from_extension(Path::new("Makefile")), Lang::Unknown);
    }

    #[test]
    fn test_lang_from_content_rust() {
        let content = "fn main() {\n    let x = 42;\n    println!(\"{}\", x);\n}\n";
        assert_eq!(lang_from_content(content), Some(Lang::Rust));
    }

    #[test]
    fn test_lang_from_content_python() {
        let content = "def main():\n    import os\n    from sys import argv\n    pass\n";
        assert_eq!(lang_from_content(content), Some(Lang::Python));
    }

    #[test]
    fn test_lang_from_content_go() {
        let content = "package main\n\nfunc main() {\n    fmt.Println(\"hello\")\n}\n";
        assert_eq!(lang_from_content(content), Some(Lang::Go));
    }

    #[test]
    fn test_lang_from_content_java() {
        let content = "package com.example;\n\nimport java.util.List;\n\npublic class Main {\n}\n";
        assert_eq!(lang_from_content(content), Some(Lang::Java));
    }

    #[test]
    fn test_lang_from_content_empty() {
        assert_eq!(lang_from_content(""), None);
    }

    #[test]
    fn test_ast_dump_rust() {
        let content = "use std::io;\n\nfn main() {\n    println!(\"hello\");\n}\n\nstruct Foo {\n    x: i32,\n}\n\nenum Bar {\n    A,\n    B,\n}\n";
        let (_temp, path) = write_temp_file(content, "rs");
        let result = code_ast_dump_work(path, None);
        assert!(result.contains("CODE: AST DUMP"));
        assert!(result.contains("fn"));
        assert!(result.contains("main"));
        assert!(result.contains("struct"));
        assert!(result.contains("Foo"));
        assert!(result.contains("enum"));
        assert!(result.contains("Bar"));
        assert!(result.contains("use"));
    }

    #[test]
    fn test_ast_dump_python() {
        let content = "import os\nfrom sys import argv\n\ndef hello():\n    pass\n\nclass Foo:\n    def bar(self):\n        pass\n";
        let (_temp, path) = write_temp_file(content, "py");
        let result = code_ast_dump_work(path, None);
        assert!(result.contains("CODE: AST DUMP"));
        assert!(result.contains("def"));
        assert!(result.contains("hello"));
        assert!(result.contains("class"));
        assert!(result.contains("Foo"));
    }

    #[test]
    fn test_extract_imports_rust() {
        let content = "use std::io;\nuse std::fs;\nuse serde::Deserialize;\nuse crate::bus::AgentMessage;\n\nfn main() {}\n";
        let (_temp, path) = write_temp_file(content, "rs");
        let result = code_extract_imports_work(path);
        assert!(result.contains("CODE: EXTRACT IMPORTS"));
        assert!(result.contains("std::io"));
        assert!(result.contains("serde::Deserialize"));
        assert!(result.contains("crate::bus"));
    }

    #[test]
    fn test_extract_imports_python() {
        let content = "import os\nimport sys\nfrom collections import defaultdict\nimport requests\nfrom .utils import helper\n";
        let (_temp, path) = write_temp_file(content, "py");
        let result = code_extract_imports_work(path);
        assert!(result.contains("CODE: EXTRACT IMPORTS"));
        assert!(result.contains("Standard library"));
    }

    #[test]
    fn test_extract_imports_go() {
        let content = "package main\n\nimport (\n\t\"fmt\"\n\t\"os\"\n\t\"github.com/gin-gonic/gin\"\n)\n\nfunc main() {}\n";
        let (_temp, path) = write_temp_file(content, "go");
        let result = code_extract_imports_work(path);
        assert!(result.contains("CODE: EXTRACT IMPORTS"));
        assert!(result.contains("fmt"));
    }

    #[test]
    fn test_detect_language_by_extension() {
        let content = "fn main() {}";
        let (_temp, path) = write_temp_file(content, "rs");
        let result = code_detect_language_work(path, None);
        assert!(result.contains("Rust"));
        assert!(result.contains("high"));
    }

    #[test]
    fn test_detect_language_by_content() {
        let content = "def main():\n    import os\n    from sys import argv\n    if __name__ == '__main__':\n        main()\n";
        let (_temp, path) = write_temp_file(content, "txt");
        let result = code_detect_language_work(path, None);
        assert!(result.contains("Python"));
    }

    #[test]
    fn test_complexity_score_simple() {
        let content = "fn main() {\n    println!(\"hello\");\n}\n";
        let (_temp, path) = write_temp_file(content, "rs");
        let result = code_complexity_score_work(path, None);
        assert!(result.contains("CODE: COMPLEXITY SCORE"));
        assert!(result.contains("complexity"));
    }

    #[test]
    fn test_complexity_score_branching() {
        let content = "fn classify(x: i32) -> &'static str {\n    if x > 0 {\n        if x > 10 {\n            \"big\"\n        } else {\n            \"small\"\n        }\n    } else if x < 0 {\n        \"negative\"\n    } else {\n        \"zero\"\n    }\n}\n";
        let (_temp, path) = write_temp_file(content, "rs");
        let result = code_complexity_score_work(path, None);
        assert!(result.contains("CODE: COMPLEXITY SCORE"));
        assert!(result.contains("classify"));
    }

    #[test]
    fn test_dead_code_hints() {
        let content = "fn used_function() {}\n\nfn unused_function() {}\n\nfn main() {\n    used_function();\n}\n";
        let (_temp, path) = write_temp_file(content, "rs");
        // This test may not find grep, so we just check it doesn't crash
        let result = code_dead_code_hints_work(PathBuf::from("/tmp"), path);
        assert!(result.contains("CODE: DEAD CODE HINTS"));
    }

    #[test]
    fn test_stdlib_patterns() {
        let rust_patterns = get_stdlib_patterns(&Lang::Rust);
        assert!(rust_patterns.contains(&"std::"));

        let python_patterns = get_stdlib_patterns(&Lang::Python);
        assert!(python_patterns.contains(&"os"));

        let go_patterns = get_stdlib_patterns(&Lang::Go);
        assert!(go_patterns.contains(&"fmt"));
    }

    #[test]
    fn test_extract_definition_names_rust() {
        let content = "fn foo() {}\nstruct Bar {}\nenum Baz {}\ntrait Qux {}";
        let lines: Vec<&str> = content.lines().collect();
        let defs = extract_definition_names(&lines, &Lang::Rust);
        assert!(defs.iter().any(|(n, _, k)| n == "foo" && k == "fn"));
        assert!(defs.iter().any(|(n, _, k)| n == "Bar" && k == "struct"));
        assert!(defs.iter().any(|(n, _, k)| n == "Baz" && k == "enum"));
        assert!(defs.iter().any(|(n, _, k)| n == "Qux" && k == "trait"));
    }

    #[test]
    fn test_find_function_starts() {
        let content = "fn foo() {}\n\nfn bar() {}\n\nfn baz() {}";
        let lines: Vec<&str> = content.lines().collect();
        let starts = find_function_starts(&lines, &Lang::Rust);
        assert_eq!(starts.len(), 3);
        assert_eq!(starts[0], ("foo".to_string(), 1));
        assert_eq!(starts[1], ("bar".to_string(), 3));
    }

    #[test]
    fn test_read_code_nonexistent() {
        let result = read_code(Path::new("/nonexistent/file.rs"));
        assert!(result.is_err());
    }
}
