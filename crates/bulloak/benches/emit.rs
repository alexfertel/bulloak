#![allow(missing_docs)]
use bulloak_foundry::scaffold;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn emit_big_tree(c: &mut Criterion) {
    let tree =
        std::fs::read_to_string("benches/bench_data/cancel.tree").unwrap();

    let cfg = Default::default();
    let mut group = c.benchmark_group("sample-size-10");
    group.bench_function("emit-big-tree", |b| {
        b.iter(|| scaffold::scaffold(black_box(&tree), &cfg))
    });
    group.finish();
}

criterion_group!(benches, emit_big_tree);
criterion_main!(benches);
