#![no_main]
#![no_std]

use ariel_os::{debug::{ExitCode, exit}, time};

mod utils;

#[cfg(feature = "wasmi")]
#[path = "wasmi.rs"]
mod run_wasm;

#[cfg(feature = "wasmtime")]
#[path = "wasmtime.rs"]
mod run_wasm;

#[cfg(feature = "wasm-interpreter")]
#[path = "wasm_interpreter.rs"]
mod run_wasm;

#[cfg(feature = "minimal")]
use run_wasm::run_wasm as minimal;

#[cfg(feature = "coremark")]
use run_wasm::run_coremark as benchmark;

#[cfg(feature = "embench-1")]
use run_wasm::embench1::run_bench as run_embench1;

#[ariel_os::task(autostart)]
async fn main() {

    #[cfg(feature = "minimal")]
    minimal();

    #[cfg(feature = "coremark")]
    {
        // Using coremark.minimal
        // https://github.com/wasm3/wasm-coremark/tree/main
        ariel_os::debug::log::debug!("Running CoreMark 1.0...");
        let score = benchmark();
        ariel_os::debug::log::info!("coremark, {:?}", score);
        ariel_os::debug::log::debug!("Score: {:?}", score);
    }

    #[cfg(feature = "embench-1")]
    {
        ariel_os::debug::log::debug!("Running Embench 1.0 benchmark");
        run_embench1();
    }
    time::Timer::after_millis(100).await;
    exit(ExitCode::SUCCESS);

}

#[cfg(feature = "embench-1")]
static BENCH_SCORE: [(&str, u64);  19] = [
    ("aha-mont64", 4_004),
    ("crc32", 4_010),
    ("cubic", 3_931),
    ("edn", 4_010),
    ("huffbench", 4_120),
    ("matmult-int", 3_985),
    ("minver", 3_998),
    ("nbody", 2_808),
    ("neetle-aes", 4_026),
    ("neetle-sha256", 3_997),
    ("nsichneu", 4_001),
    ("picojpeg", 4_030),
    ("qrduino", 4_253),
    ("sglib-combined", 3_981),
    ("slre", 4_010),
    ("st", 4_080),
    ("statemate", 4_001),
    ("ud", 3_999),
    ("wikisort", 2_779),
];

#[cfg(all(not(feature = "wasm-interpreter"),feature = "embench-1"))]
static BENCHMARK_LOOPS: usize = 100;

#[cfg(all(feature = "wasm-interpreter",feature = "embench-1"))]
static BENCHMARK_LOOPS: usize = 10;