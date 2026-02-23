use wamr_rust_sdk::{runtime::Runtime, module::Module, instance::Instance, function::Function};

extern crate alloc;

use core::ffi::c_void;

use alloc::{vec, vec::Vec};

// Required to pull the tinyrlibc code that implements extern "C" functions needed by wamr
extern crate tinyrlibc;

#[allow(unused_imports, reason = "The extern 'C' functions are actually used")]
use tinyrlibc as _;

pub mod coremark {
    use super::*;
    extern "C" fn clock_ms() -> u64 {
        ariel_os::time::Instant::now().as_millis()
    }

    pub fn run_coremark() -> f32 {
        let wasm_bytes = Vec::from(include_bytes!(crate::benchmark_file!()));

        let runtime = Runtime::builder_with_module_name("env")
            .use_system_allocator()
            .run_as_interpreter()
            .register_host_function("clock_ms",  clock_ms as *mut c_void)
            .build().unwrap();

        let module = Module::from_vec(&runtime, wasm_bytes, "test-module").unwrap();

        // 2KiB stack size
        let instance = Instance::new(&runtime, &module, 1024 * 2).unwrap();

        let function = Function::find_export_func(&instance, "run").unwrap();

        let res = function.call(&instance, &vec![])
            .unwrap().into_iter().next().unwrap()
            .into_f32()
            .unwrap();

        return res;
    }
}



#[unsafe(no_mangle)]
extern "C" fn ariel_time_get_boot_us() -> u64 {
    ariel_os::time::Instant::now().as_micros()
}

#[cfg(feature = "embench-1")]
pub mod embench1 {
    use ariel_os::time::Instant;
    use libm::{pow, exp, log, sqrt};
    use ariel_os::debug::log::{debug, error};

    use super::*;
    use crate::{BENCH_SCORE, BENCHMARK_LOOPS, benchmark_name, benchmark_file};
    use crate::utils::SendCell;

    static TIMINGS: SendCell<Vec<Instant>> = SendCell::new(Vec::new());

    pub fn run_bench() -> (f64, f64, f64, f64) {
        let bench_name = benchmark_name!();
        let wasm_bytes = Vec::from(include_bytes!(benchmark_file!()));

        let runtime = Runtime::builder_with_module_name("env")
            .use_system_allocator()
            .run_as_interpreter()
            .register_host_function("initialise_board", initialise_board as *mut c_void)
            .register_host_function("start_trigger", start_trigger as *mut c_void)
            .register_host_function("stop_trigger", stop_trigger as *mut c_void)
            .build().unwrap();

        let module = Module::from_vec(&runtime, wasm_bytes, "test-module").unwrap();

        // 4KiB stack size
        let instance = Instance::new(&runtime, &module, 1024 * 4).unwrap();

        let function = Function::find_export_func(&instance, "__original_main").unwrap();

        debug!("Starting wasm app");
        for i in 1..=BENCHMARK_LOOPS {
            debug!("Run {}", i);
            let correct = function.call(&instance, &vec![]).unwrap().into_iter().next().unwrap().into_i32();
            match correct {
                Ok(0) =>  { }
                _ => {
                    error!("Benchmarking went wrong from some reason, aborting");
                    return (0_f64, 0_f64, 0_f64, 0_f64);
                }
            }
        }

        let mut geo_mean = 1_f64;
        let mut times_geo_mean = 1_f64;
        let score_to_div = BENCH_SCORE.iter().find(|(b_name, _)| *b_name == bench_name).unwrap().1;

        assert!(TIMINGS.borrow_mut().len() % 2 == 0);

        let durations = TIMINGS.borrow_mut().chunks(2).map(|instants| { (instants[1] - instants[0]).as_millis() }).collect::<Vec<_>>();

        for dur in durations.iter() {
            let normalized_speed = score_to_div as f64 / *dur as f64;
            geo_mean *= pow(normalized_speed as f64, 1_f64/BENCHMARK_LOOPS as f64);
            times_geo_mean *= pow(*dur as f64, 1_f64/BENCHMARK_LOOPS as f64);
        }

        // sigma = exp( sqrt( 1/N sum( ln ( A_i / mean )^2 ) ) ) https://en.wikipedia.org/wiki/Geometric_standard_deviation
        let mut times_geo_std = 0_f64;
        let mut geo_std = 0_f64;

        for dur in durations.iter() {
            let normalized_speed = score_to_div as f64 / *dur as f64;
            let logged = log(normalized_speed / geo_mean);
            geo_std += logged * logged;

            let logged_times = log(*dur as f64 / times_geo_mean);
            times_geo_std += logged_times * logged_times;
        }
        geo_std = exp(sqrt(1_f64 / BENCHMARK_LOOPS as f64 * geo_std));
        times_geo_std = exp(sqrt(1_f64 / BENCHMARK_LOOPS as f64 * times_geo_std));

        debug!("Benchmark results for {}:", bench_name);
        debug!("(Geometric) Mean score: {}", geo_mean);
        debug!("Geometric Standard Deviation Score: {}", geo_std);
        debug!("Range: [{}, {}]", geo_mean / geo_std, geo_mean * geo_std);

        debug!("Timing results:");
        debug!("(Geometric) Mean time to completion: {}ms", times_geo_mean);
        debug!("Geometric Standard Deviation Time: {}", times_geo_std);
        debug!("Range(ms): [{}, {}]", times_geo_mean / times_geo_std, times_geo_mean * times_geo_std);

        (geo_mean, geo_std, times_geo_mean, times_geo_std)
    }

    extern "C" fn initialise_board() { }

    extern "C" fn start_trigger() {
        TIMINGS.borrow_mut().push(Instant::now());
    }
    extern "C" fn stop_trigger() {
        TIMINGS.borrow_mut().push(Instant::now());
    }
}
