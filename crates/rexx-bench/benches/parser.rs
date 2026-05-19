use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rexx_lexer::Lexer;
use rexx_parser::Parser;

fn bench_parser(c: &mut Criterion) {
    let source = "main: do i=1 to 100; say i; end;".repeat(50);
    c.bench_function("parser_throughput", |b| {
        b.iter(|| {
            let lexer = Lexer::new(black_box(&source));
            let tokens: Vec<_> = lexer.collect();
            let mut parser = Parser::new(tokens);
            let (_prog, _) = parser.parse_program();
        })
    });
}

criterion_group!(benches, bench_parser);
criterion_main!(benches);
