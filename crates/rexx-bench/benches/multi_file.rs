use std::fs;
use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rexx_analyzer::lint;
use tempfile::tempdir;

const REXX_SOURCE: &str = "/* header */\nDO i = 1 TO 100\n  SAY i\nEND\nEXIT 0\n";

fn create_files(root: &std::path::Path, count: usize) -> Vec<std::path::PathBuf> {
    (0..count)
        .map(|i| {
            let path = root.join(format!("file_{i:04}.rexx"));
            fs::write(&path, REXX_SOURCE).unwrap();
            path
        })
        .collect()
}

fn bench_lint_parallel(c: &mut Criterion) {
    use rayon::prelude::*;

    let mut group = c.benchmark_group("multi_file_lint");

    for count in [100usize, 1000] {
        let dir = tempdir().unwrap();
        let paths = create_files(dir.path(), count);
        let sources: Vec<String> = paths
            .iter()
            .map(|p| fs::read_to_string(p).unwrap())
            .collect();

        // Sequential (jobs=1)
        group.bench_with_input(
            BenchmarkId::new("sequential", count),
            &sources,
            |b, sources| {
                b.iter(|| {
                    sources
                        .iter()
                        .map(|s| lint(black_box(s)))
                        .collect::<Vec<_>>()
                })
            },
        );

        // Parallel (all CPUs)
        group.bench_with_input(
            BenchmarkId::new("parallel", count),
            &sources,
            |b, sources| {
                b.iter(|| {
                    sources
                        .par_iter()
                        .map(|s| lint(black_box(s)))
                        .collect::<Vec<_>>()
                })
            },
        );
    }

    group.finish();
}

fn bench_format_throughput(c: &mut Criterion) {
    use rayon::prelude::*;
    use rexx_formatter::format_rexx_with_profile_name;

    let mut group = c.benchmark_group("formatter_throughput");

    for count in [100usize, 1000] {
        let dir = tempdir().unwrap();
        let paths = create_files(dir.path(), count);
        let sources: Vec<String> = paths
            .iter()
            .map(|p| fs::read_to_string(p).unwrap())
            .collect();

        group.bench_with_input(
            BenchmarkId::new("parallel", count),
            &sources,
            |b, sources| {
                b.iter(|| {
                    sources
                        .par_iter()
                        .map(|s| {
                            format_rexx_with_profile_name(black_box(s), "mainframe-compatible")
                                .unwrap()
                        })
                        .collect::<Vec<_>>()
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_lint_parallel, bench_format_throughput);
criterion_main!(benches);
