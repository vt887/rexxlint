use std::fs;
use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rexx_walker::{WalkOptions, discover};
use tempfile::tempdir;

const REXX_STUB: &str = "/* header */\nsay 'hello'\nEXIT 0\n";

fn create_flat(root: &std::path::Path, count: usize) {
    for i in 0..count {
        fs::write(root.join(format!("file_{i:04}.rexx")), REXX_STUB).unwrap();
    }
}

fn create_nested(root: &std::path::Path, files_per_dir: usize, depth: usize) {
    fn rec(dir: &std::path::Path, fpd: usize, depth: usize) {
        for i in 0..fpd {
            fs::write(dir.join(format!("f_{i:03}.rexx")), REXX_STUB).unwrap();
        }
        if depth > 1 {
            let sub = dir.join("sub");
            fs::create_dir_all(&sub).unwrap();
            rec(&sub, fpd, depth - 1);
        }
    }
    rec(root, files_per_dir, depth);
}

fn bench_walker(c: &mut Criterion) {
    let mut group = c.benchmark_group("walker_traversal");
    let opts = WalkOptions::default();

    for count in [100usize, 1000] {
        let dir = tempdir().unwrap();
        create_flat(dir.path(), count);
        let root = vec![dir.path().to_path_buf()];
        group.bench_with_input(
            BenchmarkId::new("flat", count),
            &(&root, &opts),
            |b, (paths, opts)| b.iter(|| discover(black_box(paths), black_box(opts)).unwrap()),
        );
    }

    // 1000 files spread over a 5-level tree (200 dirs × 5 files each, roughly)
    {
        let dir = tempdir().unwrap();
        create_nested(dir.path(), 5, 5); // 5^1 + 5^2 + ... = sum over depth
        let root = vec![dir.path().to_path_buf()];
        group.bench_with_input(
            BenchmarkId::new("nested_5deep", "5×5"),
            &(&root, &opts),
            |b, (paths, opts)| b.iter(|| discover(black_box(paths), black_box(opts)).unwrap()),
        );
    }

    group.finish();
}

criterion_group!(benches, bench_walker);
criterion_main!(benches);
