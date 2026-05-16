use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rexx_analyzer::lint;

fn bench_lint(c: &mut Criterion) {
    let source = "/* header */\nmain: do i=1 to 100; say i; end;".repeat(50);
    c.bench_function("lint_throughput", |b| {
        b.iter(|| {
            let _diags = lint(black_box(&source));
        })
    });
}

criterion_group!(benches, bench_lint);
criterion_main!(benches);
