use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::io::Write;
use tempfile::TempDir;

fn create_test_project(file_count: usize) -> TempDir {
    let tmp = TempDir::new().unwrap();

    // Create a realistic project structure
    let dirs = ["src", "src/ops", "src/tools", "tests", "benches"];
    for d in &dirs {
        std::fs::create_dir_all(tmp.path().join(d)).unwrap();
    }

    for i in 0..file_count {
        let dir = dirs[i % dirs.len()];
        let ext = if i % 3 == 0 {
            "rs"
        } else if i % 3 == 1 {
            "ts"
        } else {
            "py"
        };
        let path = tmp.path().join(dir).join(format!("file_{}.{}", i, ext));
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "// File {} - benchmark test file", i).unwrap();
        writeln!(f, "pub fn function_{}() -> i32 {{", i).unwrap();
        writeln!(f, "    let value_{} = {} * 2;", i, i).unwrap();
        writeln!(f, "    value_{}", i).unwrap();
        writeln!(f, "}}").unwrap();
        writeln!(f, "").unwrap();
        writeln!(f, "pub struct Struct{} {{", i).unwrap();
        writeln!(f, "    pub field: String,").unwrap();
        writeln!(f, "}}").unwrap();
    }

    tmp
}

fn bench_index_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("code_index_build");

    for file_count in [10, 50, 100] {
        let project = create_test_project(file_count);

        group.bench_with_input(
            BenchmarkId::new("files", file_count),
            &file_count,
            |b, _| {
                b.iter(|| {
                    // Walk files and build a basic index
                    let mut entries = Vec::new();
                    for entry in walkdir::WalkDir::new(project.path())
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter(|e| e.file_type().is_file())
                    {
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            entries.push((
                                entry.path().to_string_lossy().to_string(),
                                content.lines().count(),
                            ));
                        }
                    }
                    entries
                });
            },
        );
    }
    group.finish();
}

fn bench_grep_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("code_grep_search");
    let project = create_test_project(100);

    for pattern in ["function_", "Struct\\d+", "pub fn"] {
        group.bench_with_input(
            BenchmarkId::new("pattern", pattern),
            &pattern,
            |b, &pattern| {
                b.iter(|| {
                    let re = regex::Regex::new(pattern).unwrap();
                    let mut matches = 0usize;
                    for entry in walkdir::WalkDir::new(project.path())
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter(|e| e.file_type().is_file())
                    {
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            matches += content.lines().filter(|line| re.is_match(line)).count();
                        }
                    }
                    matches
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_index_build, bench_grep_search);
criterion_main!(benches);
