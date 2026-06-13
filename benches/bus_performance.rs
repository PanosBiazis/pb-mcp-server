#![allow(
    clippy::expect_used,
    clippy::field_reassign_with_default,
    clippy::io_other_error,
    clippy::needless_borrows_for_generic_args,
    clippy::panic,
    clippy::unwrap_used
)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use pb_mcp_server::bus::AgentMessage;
use tempfile::TempDir;

fn create_test_message(i: usize) -> AgentMessage {
    AgentMessage {
        from_agent: format!("bench-agent-{}", i % 5),
        body: format!(
            "Benchmark message number {} with some payload content for realistic sizing",
            i
        ),
        to_agent: if i.is_multiple_of(3) {
            Some("target-agent".into())
        } else {
            None
        },
        topic: if i.is_multiple_of(2) {
            Some("bench-topic".into())
        } else {
            None
        },
        kind: "note".into(),
        ts: 1700000000000 + i as u64,
        id: format!("{}-{}", 1700000000000 + i as u64, i),
    }
}

fn bench_bus_append(c: &mut Criterion) {
    let mut group = c.benchmark_group("bus_append");

    for count in [10, 100, 500] {
        group.bench_with_input(BenchmarkId::new("messages", count), &count, |b, &count| {
            b.iter(|| {
                let tmp = TempDir::new().unwrap();
                let bus_dir = tmp.path().join(".mint-mcp");
                std::fs::create_dir_all(&bus_dir).unwrap();
                let bus_file = bus_dir.join("agent-bus.jsonl");

                for i in 0..count {
                    let msg = create_test_message(i);
                    let line = serde_json::to_string(&msg).unwrap();
                    use std::io::Write;
                    let mut f = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&bus_file)
                        .unwrap();
                    writeln!(f, "{}", line).unwrap();
                }
            });
        });
    }
    group.finish();
}

fn bench_bus_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("bus_read");

    for count in [100, 500, 1000] {
        // Pre-populate bus file
        let tmp = TempDir::new().unwrap();
        let bus_dir = tmp.path().join(".mint-mcp");
        std::fs::create_dir_all(&bus_dir).unwrap();
        let bus_file = bus_dir.join("agent-bus.jsonl");

        {
            use std::io::Write;
            let mut f = std::fs::File::create(&bus_file).unwrap();
            for i in 0..count {
                let msg = create_test_message(i);
                let line = serde_json::to_string(&msg).unwrap();
                writeln!(f, "{}", line).unwrap();
            }
        }

        group.bench_with_input(BenchmarkId::new("read_all", count), &count, |b, _| {
            b.iter(|| {
                let contents = std::fs::read_to_string(&bus_file).unwrap();
                let _messages: Vec<AgentMessage> = contents
                    .lines()
                    .filter_map(|line| serde_json::from_str(line).ok())
                    .collect();
            });
        });

        group.bench_with_input(
            BenchmarkId::new("filter_by_agent", count),
            &count,
            |b, _| {
                b.iter(|| {
                    let contents = std::fs::read_to_string(&bus_file).unwrap();
                    let _messages: Vec<AgentMessage> = contents
                        .lines()
                        .filter_map(|line| serde_json::from_str::<AgentMessage>(line).ok())
                        .filter(|m| m.from_agent == "bench-agent-0")
                        .collect();
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_bus_append, bench_bus_read);
criterion_main!(benches);
