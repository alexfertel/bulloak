#![allow(missing_docs)]
use std::fs;

use bulloak_syntax::{parse, parser::Parser, semantics, tokenizer};
use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
    Throughput,
};

fn load(name: &str) -> String {
    let path = format!("benches/bench_data/{}", name);
    fs::read_to_string(&path).unwrap()
}

fn bench_tokenizer(c: &mut Criterion) {
    let small = load("small.tree");
    let medium = load("medium.tree");
    let large = load("large.tree");
    let cases = [("small", &small), ("medium", &medium), ("large", &large)];

    let mut group = c.benchmark_group("tokenizer");
    for (label, text) in &cases {
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("tokenize", label),
            text,
            |b, t| {
                b.iter(|| {
                    tokenizer::Tokenizer::new().tokenize(black_box(t)).unwrap()
                });
            },
        );
    }
    group.finish();
}

fn bench_parser(c: &mut Criterion) {
    let medium = load("medium.tree");
    let large = load("large.tree");
    let cases = [("medium", &medium), ("large", &large)];

    let mut group = c.benchmark_group("parser");
    for (label, text) in &cases {
        // pre‐tokenize once
        let tokens = tokenizer::Tokenizer::new().tokenize(text).unwrap();
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("parse_only", label),
            &(text.as_str(), &tokens[..]),
            |b, &(txt, toks)| {
                let mut p = Parser::new();
                b.iter(|| {
                    p.parse(black_box(txt), black_box(toks)).unwrap();
                });
            },
        );
    }
    group.finish();
}

fn bench_semantics(c: &mut Criterion) {
    let large = load("large.tree");
    // Build AST once.
    let ast = {
        let toks = tokenizer::Tokenizer::new().tokenize(&large).unwrap();
        Parser::new().parse(&large, &toks).unwrap()
    };
    let mut group = c.benchmark_group("semantics");
    group.throughput(Throughput::Bytes(large.len() as u64));
    group.bench_function("analyze", |b| {
        b.iter(|| {
            let mut analyzer =
                semantics::SemanticAnalyzer::new(black_box(&large));
            analyzer.analyze(black_box(&ast)).unwrap();
        })
    });
    group.finish();
}

fn bench_e2e(c: &mut Criterion) {
    let example = load("large.tree");
    let mut group = c.benchmark_group("parse+analyze");
    group.throughput(Throughput::Bytes(example.len() as u64));
    group.bench_function("e2e_parse", |b| {
        b.iter(|| {
            let _ = parse(black_box(&example)).unwrap();
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_tokenizer,
    bench_parser,
    bench_semantics,
    bench_e2e,
);
criterion_main!(benches);
