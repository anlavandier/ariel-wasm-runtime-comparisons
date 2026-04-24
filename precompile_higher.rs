#!/usr/bin/env -S cargo +nightly -Zscript

---cargo
[package]
edition = "2024"

[dependencies]
clap = { version = "4.5.40", features = ["derive"] }
miette = { version = "7.2", features = ["fancy"] }
thiserror = { version = "2.0" }
---

use std::{fs, io};
use clap::Parser;
use miette::Diagnostic;


/// Helper script to precompile the benchmarks using the selected wasmtime version
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Wasmtime version to use
    #[arg(short = 'w', long = "w-version")]
    wasmtime_version: String
}

#[derive(Debug, thiserror::Error, Diagnostic)]
enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

fn main() -> miette::Result<()> {
    let args = Args::parse();
    let version = args.wasmtime_version;
    // Write the template in a file
    fs::write("temp.rs", PRECOMPILING_TEMPLATE.as_bytes()).map_err(Error::from)?;
    // Prepare the expression to subsitute TOCHANGE with the asked version number
    let substitute_regexp = format!("s/TOCHANGE/{}/g", version);
    std::process::Command::new("sed").args(
        ["-i", "-e", substitute_regexp.as_str(), "temp.rs"]
    ).output().map_err(Error::from)?;

    let std::process::Output {stdout, stderr: _, status: _} = std::process::Command::new("cargo").args(
        ["+nightly", "-Z", "script", "temp.rs"]
    ).output().map_err(Error::from)?;
    std::println!("{}", String::from_utf8_lossy(&stdout));
    fs::remove_file("temp.rs").map_err(Error::from)?;
    Ok(())
}

const PRECOMPILING_TEMPLATE: &str = r####"#!/usr/bin/env -S cargo +nightly -Zscript

---cargo
[package]
edition = "2024"

[dependencies]
wasmtime = {version = "=TOCHANGE", default-features = false, features = ["cranelift", "pulley"]}
---

use std::fs;
use wasmtime::{Config, Engine, OptLevel};

const WAMSTIME_VERSION: &str ="TOCHANGE";

fn main() {

    std::println!("Precompiling using wasmtime version {}", WAMSTIME_VERSION);
    let mut config = Config::new();

    // Options found to reduce the output code size the most at least for components
    config.memory_init_cow(false);
    config.generate_address_map(false);
    config.table_lazy_init(false);
    config.cranelift_opt_level(OptLevel::Speed);

    config.wasm_custom_page_sizes(true);
    config.target("pulley32").unwrap();

    // 0 means limiting ourselves to what the module asked
    // This needs to be set at pre-compile time and at runtime in the engine
    config.memory_reservation(0);

    // Disabling this allows runtime optimizations but means that the maximum memory
    // that the module can have is
    // S = min(initial_memory, memory_reservation) + memory_reserver_for_growth
    // since it can grow by reallocating.
    config.memory_may_move(false);


    // Create an `Engine` with that configuration.
    let engine = Engine::new(&config).unwrap();

    // read the benchmarks dir

    for dir in fs::read_dir("benchmarks").unwrap() {
        for file in fs::read_dir(dir.unwrap().path()).unwrap().filter_map(
            |entry| {
                if entry.is_ok() {
                    let f = entry.unwrap();
                    match f.path().extension().map(|ext| { ext.to_str() }).flatten() {
                        Some("wasm" ) => { Some(f) },
                        _ => None
                    }
                } else {
                    None
                }
            }
        ) {
            std::println!("Precompiling {:?}", file.path());
            let mut path_copy = file.path().clone();
            let wasm = fs::read(&path_copy).unwrap();
            let precompiled = engine.precompile_module(&wasm).unwrap();
            path_copy.set_extension("cwasm");
            // std::println!("Writing the precompiled file at {:?}", path_copy);
            fs::write(path_copy, &precompiled).unwrap();
        }
    }
}"####;