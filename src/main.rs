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

#[cfg(feature = "wamr")]
#[path = "wamr.rs"]
mod run_wasm;

#[cfg(feature = "coremark")]
use run_wasm::run_coremark as benchmark;

#[cfg(feature = "embench-1")]
use run_wasm::embench1::run_bench as run_embench1;

#[ariel_os::task(autostart)]
async fn main() {
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
        #[allow(unused_variables)]
        let (score_mean, score_std, times_means, times_std) = benchmark();

        #[cfg(not(feature = "monitor-heap"))]
        ariel_os::debug::log::info!("{}, {}, {}, {}, {}", crate::benchmark_name!(), score_mean, score_std, times_means, times_std);
    }

    #[cfg(feature = "monitor-heap")]
    {
        let max = critical_section::with(|cs| {
            instrumented_allocator::MAX.counters.borrow(cs)
                .get().1
        });
        ariel_os::debug::log::info!("{}, {}", crate::benchmark_name!(), max);
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

#[cfg(all(not(feature = "wasm-interpreter"),feature = "embench-1", not(feature = "monitor-heap")))]
static BENCHMARK_LOOPS: usize = 100;

#[cfg(all(any(feature = "wasm-interpreter", feature = "wasefire-interpreter"),feature = "embench-1", not(feature = "monitor-heap")))]
static BENCHMARK_LOOPS: usize = 10;

#[cfg(all(feature = "monitor-heap", feature = "embench-1"))]
static BENCHMARK_LOOPS: usize = 2;

#[cfg(feature = "monitor-heap")]
pub mod instrumented_allocator {
    use core::{alloc::GlobalAlloc, cell::Cell};
    use critical_section::Mutex;

    #[cfg(context = "cortex-m")]
    use ariel_os_alloc::alloc::HEAP;

    #[cfg(context = "esp")]
    use esp_alloc::HEAP;

    pub struct HeapThatKnows {
        pub counters: Mutex<Cell<(usize, usize)>>
    }

    #[global_allocator]
    pub static MAX: HeapThatKnows = HeapThatKnows { counters: Mutex::new(Cell::new((0, 0)))};

    #[allow(unsafe_code)]
    unsafe impl GlobalAlloc for HeapThatKnows{
        unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
            critical_section::with(|cs| {
                self.counters.borrow(cs).update(|(current, max)| {
                    let new_cur = current + layout.size();
                    if new_cur >= max {
                        (new_cur, new_cur)
                    } else {
                        (new_cur, max)
                    }
                });
            });
            unsafe { HEAP.alloc(layout) }
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
            critical_section::with(|cs| {
                self.counters.borrow(cs).update(|(current, max)| {
                    let new_cur = current - layout.size();
                    (new_cur, max)
                });
            });
            unsafe { HEAP.dealloc(ptr, layout) }
        }
    }
}