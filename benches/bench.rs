use ::bulloak;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn big_tree(c: &mut Criterion) {
    let tree = std::fs::read_to_string("benches/bench_data/cancel.tree").unwrap();

    let scaffolder = bulloak::Scaffolder::new(true, 2, "some_version");
    let mut group = c.benchmark_group("sample-size-10");
    // group.sample_size(10);
    group.bench_function("big-tree", |b| {
        b.iter(|| scaffolder.scaffold(black_box(&tree)))
    });
    group.finish();
}

criterion_group!(benches, big_tree);
criterion_main!(benches);
