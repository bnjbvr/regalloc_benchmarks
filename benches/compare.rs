use criterion::{self, criterion_group, criterion_main, Criterion};
use regalloc::*;

use std::fs;
use std::io::Read;

fn read(path: &str) -> IRSnapshot {
    let mut contents = Vec::new();
    let mut file = fs::File::open(path).expect("couldn't read snapshot path");
    file.read_to_end(&mut contents)
        .expect("couldn't read snapshot file");
    bincode::deserialize(&contents).expect("couldn't deserialize")
}

fn run_group(c: &mut Criterion, sample_size: Option<usize>, lsra_opts: &Options, bt_opts: &Options, snapshot_path: &str,) {
    let mut group = c.benchmark_group(snapshot_path);
    if let Some(sample_size) = sample_size {
        group.sample_size(sample_size);
    }

    let binary = read(&format!("./snapshots/{}.bin", snapshot_path));

    group.bench_function("lsra", |b| {
        b.iter_batched(
            || (binary.clone(), lsra_opts.clone()),
            |(mut snapshot, opts)| snapshot.allocate(opts).unwrap(),
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("bt", |b| {
        b.iter_batched(
            || (binary.clone(), bt_opts.clone()),
            |(mut snapshot, opts)| snapshot.allocate(opts).unwrap(),
            criterion::BatchSize::SmallInput,
        )
    });
}

pub fn run_bench(c: &mut Criterion) {
    let lsra_opts = Options {
        run_checker: false,
        algorithm: Algorithm::LinearScan(LinearScanOptions::default()),
    };

    let bt_opts = Options {
        run_checker: false,
        algorithm: Algorithm::Backtracking(BacktrackingOptions::default()),
    };

    run_group(c, None, &lsra_opts, &bt_opts, "arm-clif/small");
    run_group(c, None, &lsra_opts, &bt_opts, "arm-clif/medium");
    run_group(c, Some(10), &lsra_opts, &bt_opts, "arm-clif/big");
}

criterion_group!(benches, run_bench);
criterion_main!(benches);
