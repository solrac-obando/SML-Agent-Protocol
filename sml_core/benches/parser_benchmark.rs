use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sml_core::parser::parse_sml_token;

fn benchmark_basic_parsing(c: &mut Criterion) {
    let commands = [
        "@[read:src/main.rs]",
        "@[write:app.py|print('hello')]",
        "@[term:cargo build]",
        "@[read:config.json]",
        "@[list:src]",
        "@[exist:Cargo.toml]",
        "@[info:src/lib.rs]",
    ];

    c.bench_function("parse_sml_token", |b| {
        b.iter(|| {
            for cmd in &commands {
                black_box(parse_sml_token(cmd));
            }
        });
    });
}

fn benchmark_large_args(c: &mut Criterion) {
    let long_path = "@[read:";
    let path = "src/very/long/path/to/some/deeply/nested/file/that/has/a/long/name.rs".repeat(3);
    let input = format!("{}]", path);

    c.bench_function("parse_long_path", |b| {
        b.iter(|| black_box(parse_sml_token(&input)));
    });
}

fn benchmark_invalid_inputs(c: &mut Criterion) {
    let invalid_inputs = [
        "plain text",
        "no brackets",
        "@[incomplete",
        "@no closing",
        "@[::]",
        "@[tool:]",
        "random noise here @[read:file.rs]",
    ];

    c.bench_function("parse_invalid", |b| {
        b.iter(|| {
            for input in &invalid_inputs {
                black_box(parse_sml_token(input));
            }
        });
    });
}

fn benchmark_many_args(c: &mut Criterion) {
    let many_args = "@[write:file.txt|arg1|arg2|arg3|arg4|arg5|arg6|arg7|arg8]";
    
    c.bench_function("parse_many_args", |b| {
        b.iter(|| black_box(parse_sml_token(many_args)));
    });
}

criterion_group!(
    benches,
    benchmark_basic_parsing,
    benchmark_large_args,
    benchmark_invalid_inputs,
    benchmark_many_args
);
criterion_main!(benches);