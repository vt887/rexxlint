use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rexx_formatter::format_rexx;

fn bench_formatter(c: &mut Criterion) {
    let source = "/* header */\nmain: do i=1 to 100; say i; end;".repeat(50);
    c.bench_function("formatter_throughput", |b| {
        b.iter(|| {
            let _formatted = format_rexx(black_box(&source));
        })
    });
}

criterion_group!(benches, bench_formatter);
criterion_main!(benches);
