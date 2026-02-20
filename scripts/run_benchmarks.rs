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

    /// Provide the arch string required for wamr
    #[arg(long)]
    arch: Option<Arch>,

    /// Monitor the Dynamic Memory usage
    #[arg(long = "monitor-heap")]
    monitor: bool,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Runtime {
    Wasmtime,
    WasmtimeNoSIMD,
    Wasmi,
    WasmInterpreter,
    WasefireNative,
    WasefirePulley,
    WamrFast,
    WamrAOT,
    Wamr,
}

impl ValueEnum for Runtime {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::Wasmtime,
            Self::WasmtimeNoSIMD,
            Self::Wasmi,
            Self::WasmInterpreter,
            Self::WasefireNative,
            Self::WasefirePulley,
            Self::WamrFast,
            Self::WamrAOT,
            Self::Wamr,
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            Self::Wasmtime => Some(PossibleValue::new("wasmtime")),
            Self::WasmtimeNoSIMD => Some(PossibleValue::new("wasmtime-no-simd")),
            Self::Wasmi => Some(PossibleValue::new("wasmi")),
            Self::WasmInterpreter => Some(PossibleValue::new("wasm-interpreter")),
            Self::WasefireNative => Some(PossibleValue::new("wasefire")),
            Self::WasefirePulley => Some(PossibleValue::new("wasefire-pulley")),
            Self::WamrFast => Some(PossibleValue::new("wamr-fast")),
            Self::WamrAOT => Some(PossibleValue::new("wamr-aot")),
            Self::Wamr => Some(PossibleValue::new("wamr")),
        }
    }
}

impl Runtime {
    fn payload_extension(&self) -> &str {
        match self {
            Self::Wasmtime | Self::WasmtimeNoSIMD | Self::WasefirePulley => {
                "cwasm"
            },
            Self::WamrAOT => {
                "aot"
            },
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
            Self::WamrFast => "wamr-fast",
            Self::WamrAOT => unimplemented!("This runtime isn't yet supported"),
            Self::Wamr => "wamr",
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Arch {
    ThumbV7,
    ThumbV8,
    Xtensa,
    RiscV32,
}

impl ValueEnum for Arch {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::ThumbV7, Self::ThumbV8, Self::Xtensa, Self::RiscV32]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            Self::ThumbV7 => Some(PossibleValue::new("thumbv7")),
            Self::ThumbV8 => Some(PossibleValue::new("thumbv8")),
            Self::Xtensa => Some(PossibleValue::new("xtensa")),
            Self::RiscV32 => Some(PossibleValue::new("riscv32")),
        }
    }
}

impl Arch {
    fn to_wamr_build_target(&self) -> &str {
        match self {
            Self::ThumbV7 => "THUMBV7",
            Self::ThumbV8 => "THUMBV8.MAIN",
            Self::Xtensa => "XTENSA",
            Self::RiscV32 => "RISCV32",
        }
    }

    fn from_board_name(board: &str) -> Self {
        match board {
            "nrf52840dk" => Self::ThumbV7,
            "rpi-pico2-w" => Self::ThumbV8,
            "espressif-esp32-devkitc" => Self::Xtensa,
            "espressif-esp32c6-devkit" => Self::RiscV32,
            "dfrobot-firebeetle2-esp32-c6" => Self::RiscV32,
            _ => panic!("This board isn't recognized, update this script or explicitly specify the architecture"),
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
    let monitor_heap = args.monitor;
    let arch = if let Some(arch) = args.arch {
        arch
    } else {
        Arch::from_board_name(&board)
    };

    let mut entries = fs::read_dir(dir_path).map_err(Error::from)?.filter_map(|e| e.ok()).collect::<Vec<_>>();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        match entry.path().extension().map(|ext| { ext.to_str() }).flatten() {
            Some(extension) if extension == runtime.payload_extension() => {
                let bench_path = entry.path().to_owned();
                let bench_name = bench_path.file_prefix().unwrap().to_owned();

                let mut laze_args = Vec::from_iter(
                    [
                        "build",
                        "-s", runtime.to_laze_module(),
                        "-s", benchmark.to_laze_module(),
                        "-b", &board,
                    ]
                );
                if monitor_heap {
                    laze_args.extend([
                        "-s", "dynamic-memory-measure"
                    ]);
                }

                laze_args.push("run");
                // FIXME: do better to know that this is indeed an esp32
                if !board.contains("esp") {
                    laze_args.extend([
                        "--",
                        "--log-format", "{s}",
                        "--target-output-file", output_file.to_str().unwrap()
                    ]);
                }


                match probe.as_str() {
                    s if s == String::default() => {}
                    _ => laze_args.extend(["--probe", &probe]),
                }
                match runtime {
                    Runtime::Wamr | Runtime::WamrAOT | Runtime::WamrFast => {
                        let cflag = match arch {
                            Arch::ThumbV7 | Arch::ThumbV8 => {
                                "TARGET_C_FLAG=--specs=nosys.specs "
                            },
                            _ => {
                                ""
                            }
                        };
                        println!(
                            "{}={:?} {}=../{:?} {}={} {}={} {}laze {}",
                            "BENCHMARK", bench_name,
                            "BENCHMARK_PATH", bench_path,
                            "WAMR_BUILD_PLATFORM", "ariel-os",
                            "WAMR_BUILD_TARGET", arch.to_wamr_build_target(),
                            cflag,
                            laze_args.join(" ")
                        );
                    }
                    _ => {
                        println!("BENCHMARK={:?} BENCHMARK_PATH=../{:?} laze {}", bench_name, bench_path, laze_args.join(" "));
                    }
                }

                let mut command = process::Command::new("laze");
                command
                    .env("BENCHMARK", bench_name.to_str().unwrap())
                    .env("BENCHMARK_PATH", format!("../{}", bench_path.to_str().unwrap()));

                let output = match runtime {
                    Runtime::Wamr | Runtime::WamrAOT | Runtime::WamrFast => {
                        command
                            .env("WAMR_BUILD_PLATFORM", "ariel-os")
                            .env("WAMR_BUILD_TARGET", arch.to_wamr_build_target());
                        match arch {
                            Arch::ThumbV7 | Arch::ThumbV8 => {
                                command.env("TARGET_C_FLAG", "--specs=nosys.specs");
                            }
                            _ => { }
                        }
                        command
                    },
                    _ => {
                        command
                    }
                }
                    .args(&laze_args)
                    .output()
                    .map_err(Error::from)?;

                let process::Output { status, stdout: _, stderr} = output;

                if !status.success() {
                    std::println!(
                        "{}", String::from_utf8_lossy(&stderr)
                    );
                }
            }
            _ => {
            }
        }
    }

    Ok(())
}