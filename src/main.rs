#![no_main]
#![no_std]

use ariel_os::debug::{exit, ExitCode};


#[cfg(feature = "wasmi")]
#[path = "wasmi.rs"]
mod run_wasm;

#[cfg(feature = "wasmtime")]
#[path = "wasmtime.rs"]
mod run_wasm;

#[cfg(feature = "wasm-interpreter")]
#[path = "wasm_interpreter.rs"]
mod run_wasm;

use run_wasm::run_wasm;

#[ariel_os::task(autostart)]
async fn main() {
    // 100 is hardcoded for the wasm-interpreter
    assert_eq!(run_wasm(26, 18, 100), 26 + 18 + 100);
    exit(ExitCode::SUCCESS);

}
