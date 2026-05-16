use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rexx_lexer::Lexer;

fn bench_lexer(c: &mut Criterion) {
    let source = "/* header */\nmain: do i=1 to 1000; say i; end; exit 0;".repeat(100);
    c.bench_function("lexer_throughput", |b| {
        b.iter(|| {
            let lexer = Lexer::new(black_box(&source));
            let _tokens: Vec<_> = lexer.collect();
        })
    });
}

criterion_group!(benches, bench_lexer);
criterion_main!(benches);
