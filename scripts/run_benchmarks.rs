#!/usr/bin/env -S cargo +nightly -Zscript

---cargo
[package]
edition = "2024"

[dependencies]
clap = { version = "4.5.40", features = ["derive"] }
miette = { version = "7.2", features = ["fancy"] }
thiserror = { version = "2.0" }
---

use std::{fs, io, path::PathBuf};
use std::process;
use clap::{Parser, ValueEnum, builder::PossibleValue};
use miette::Diagnostic;

/// Helper script to run benchmarks and report the results
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Type of benchmark to use
    #[arg(short, long)]
    benchmark: Benchmark,

    /// Output file of the benchmark results. If it exists, results will be appended to it
    #[arg(short = 'o', long = "output-file")]
    output_file: PathBuf,

    /// Board to run the benchmarks on
    #[arg(long = "board")]
    board: String,

    /// Runtime to evaluate defaults to wasmtime
    #[arg(short, long)]
    runtime: Runtime,

    /// Probe ID used by probe-rs to disambiguate in presence of several devices
    #[arg(short, long)]
    probe: Option<String>,
}

#[derive(Clone, Copy, Debug)]
enum Benchmark {
    Embench1,
    Embench2,
    CoreMark,
}

impl ValueEnum for Benchmark {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Embench1, Self::Embench2, Self::CoreMark]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            Self::Embench1 => Some(PossibleValue::new("embench-1")),
            Self::Embench2 => Some(PossibleValue::new("embench-2")),
            Self::CoreMark => Some(PossibleValue::new("coremark")),
        }
    }
}

impl Benchmark {
    fn to_dirname(&self) -> &str {
        match self {
            Self::Embench1 => "embench-1.0",
            Self::Embench2 => unimplemented!("This benchmark suite isn't yet supported"),
            Self::CoreMark => "coremark",
        }
    }

    // It's the same but separating
    fn to_laze_module(&self) -> &str {
        self.to_dirname()
    }
}

#[derive(Clone, Copy, Debug)]
enum Runtime {
    Wasmtime,
    WasmtimeNoSIMD,
    Wasmi,
    WasmInterpreter,
    WasefireNative,
    WasefirePulley,
}

impl ValueEnum for Runtime {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Wasmtime, Self::WasmtimeNoSIMD, Self::Wasmi, Self::WasmInterpreter, Self::WasefireNative, Self::WasefirePulley]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            Self::Wasmtime => Some(PossibleValue::new("wasmtime")),
            Self::WasmtimeNoSIMD => Some(PossibleValue::new("wasmtime-no-simd")),
            Self::Wasmi => Some(PossibleValue::new("wasmi")),
            Self::WasmInterpreter => Some(PossibleValue::new("wasm-interpreter")),
            Self::WasefireNative => Some(PossibleValue::new("wasefire")),
            Self::WasefirePulley => Some(PossibleValue::new("wasefire-pulley")),
        }
    }
}

impl Runtime {
    fn payload_extension(&self) -> &str {
        match self {
            Self::Wasmtime | Self::WasmtimeNoSIMD | Self::WasefirePulley => {
                "cwasm"
            }
            _ => {
                "wasm"
            }
        }
    }

    fn to_laze_module(&self) -> &str {
        match self {
            Self::Wasmtime => "wasmtime",
            Self::WasmtimeNoSIMD => "wasmtime-no-simd",
            Self::Wasmi => "wasmi",
            Self::WasmInterpreter => "wasm-interpreter",
            Self::WasefireNative => unimplemented!("This runtime isn't yet supported"),
            Self::WasefirePulley => unimplemented!("This runtime isn't yet supported"),
        }
    }
}

#[derive(Debug, thiserror::Error, Diagnostic)]
enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

fn main() -> miette::Result<()> {
    let args = Args::parse();
    let runtime = args.runtime;
    let benchmark = args.benchmark;
    let dir_path = format!("benchmarks/{}", benchmark.to_dirname());
    let board = args.board;
    let output_file = args.output_file;
    let probe = args.probe.unwrap_or_default();


    for entry in fs::read_dir(dir_path).map_err(Error::from)? {
        if let Ok(entry) = entry {
            match entry.path().extension().map(|ext| { ext.to_str() }).flatten() {
                Some(extension) if extension == runtime.payload_extension() => {
                    let bench_path = entry.path().to_owned();
                    let bench_name = bench_path.file_prefix().unwrap().to_owned();

                    println!("BENCHMARK={:?} BENCHMARK_PATH=../{:?} laze build -s {:?} -s {:?} -b {:?} run -- --log-format \"{{s}}\" --target-output-file {:?} --probe {:?}",
                        bench_name, bench_path, runtime.to_laze_module(), benchmark.to_laze_module(),
                        board, output_file, probe);

                    let mut laze_args = Vec::from_iter(
                        [
                            "build",
                            "-s", runtime.to_laze_module(),
                            "-s", benchmark.to_laze_module(),
                            "-b", &board,
                            "run",
                            "--",
                            "--log-format", "{s}",
                            "--target-output-file", output_file.to_str().unwrap(),
                        ]
                    );
                    match probe.as_str() {
                        s if s == String::default() => {}
                        _ => laze_args.extend(["--probe", &probe]),
                    }

                    let output = process::Command::new("laze")
                        .env("BENCHMARK", bench_name.to_str().unwrap())
                        .env("BENCHMARK_PATH", format!("../{}", bench_path.to_str().unwrap()))
                        .args(&laze_args)
                        .output()
                        .map_err(Error::from)?;

                    let process::Output { status, stdout: _, stderr} = output;

                    if !status.success() {
                        std::println!(
                            "{}", String::from_utf8_lossy(&stderr)
                        );
                        break;
                    }
                }
                _ => {
                }
            }
        }
    }

    Ok(())
}