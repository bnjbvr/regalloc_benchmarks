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

fn run_file(
    c: &mut Criterion,
    sample_size: Option<usize>,
    lsra_opts: &Options,
    bt_opts: &Options,
    snapshot_path: &str,
) {
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

fn run_dir(
    c: &mut Criterion,
    sample_size: Option<usize>,
    lsra_opts: &Options,
    bt_opts: &Options,
    snapshot_path: &str,
) {
    let mut group = c.benchmark_group(snapshot_path);
    if let Some(sample_size) = sample_size {
        group.sample_size(sample_size);
    }

    let path = format!("./snapshots/{}", snapshot_path);
    let mut snapshots = Vec::new();
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        assert!(!path.is_dir());
        snapshots.push(read(path.into_os_string().to_str().unwrap()));
    }

    group.bench_function("lsra", |b| {
        b.iter_batched(
            || (snapshots.clone(), lsra_opts.clone()),
            |(snapshots, opts)| {
                for mut snapshot in snapshots {
                    snapshot.allocate(opts.clone()).unwrap();
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("bt", |b| {
        b.iter_batched(
            || (snapshots.clone(), bt_opts.clone()),
            |(snapshots, opts)| {
                for mut snapshot in snapshots {
                    snapshot.allocate(opts.clone()).unwrap();
                }
            },
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

    run_file(c, None, &lsra_opts, &bt_opts, "arm-clif/small");
    run_file(c, None, &lsra_opts, &bt_opts, "arm-clif/medium");
    run_file(c, Some(10), &lsra_opts, &bt_opts, "arm-clif/big");
    run_dir(c, Some(20), &lsra_opts, &bt_opts, "bz2-wasm");
    run_dir(c, Some(10), &lsra_opts, &bt_opts, "regex-rs-wasm");
}

criterion_group!(benches, run_bench);
criterion_main!(benches);
