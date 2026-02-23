#[cfg(feature = "coremark")]
pub mod coremark {
    extern crate alloc;
    use alloc::vec;
    use alloc::alloc::{Layout, alloc_zeroed};

    use wasefire_interpreter::{FuncType, Module, RunResult, Store, Val, ValType, prepare};

    pub fn run_coremark() -> f32 {
        let wasm_bytes = include_bytes!(crate::benchmark_file!());
        let mut store = Store::default();

        // Link the clock_ms function that takes in 0 arguments and results a u64
        let ftype = FuncType {
            params: ().into(),
            results: ValType::I64.into()
        };
        store.link_func_custom("env", "clock_ms", ftype).unwrap();

        let wasm = prepare(wasm_bytes).unwrap();
        let module = Module::new(&wasm).unwrap();

        // Allocate Memory for the module, I don't know how much memory it needs so i'm going to give it 64KiB
        // For some reason the memory needs to be 16-aligned so enforce this
        let layout = Layout::from_size_align(64 * 1024, 16).unwrap();
        let mut memory = unsafe { core::slice::from_raw_parts_mut(alloc_zeroed(layout), 64 * 1024) };

        let inst = store.instantiate(module, &mut memory).unwrap();

        // Call the "run" function exported by the instance
        let mut result = store.invoke(inst, "run", vec![]).unwrap();

        // Process call from the module to the host until "run" terminates
        loop {
            let call = match result {
                // The function called into the host
                RunResult::Host(call) => call,
                RunResult::Done(results) => {
                    assert_eq!(results.len(), 1);
                    match results[0] {
                        Val::F32(score) => return  f32::from_bits(score),
                        _ => unreachable!()
                    }
                }
            };

            // We only have a single linked function so the index should be 0.
            assert_eq!(call.index(), 0);
            result = call.resume(&[Val::I64(ariel_os::time::Instant::now().as_millis())]).unwrap();
        }
    }
}

#[cfg(feature = "embench-1")]
pub mod embench1 {
    extern crate alloc;
    use alloc::{vec, vec::Vec};
    use alloc::alloc::{Layout, alloc_zeroed};

    use wasefire_interpreter::{Module, RunResult, Store, Val, prepare};

    use ariel_os::time::Instant;
    use libm::{pow, exp, log, sqrt};
    use ariel_os::debug::log::{debug, error};

    const BENCHMARK_LOOPS: usize = 2;
    use crate::{BENCH_SCORE, benchmark_name, benchmark_file};

    pub fn run_bench() -> (f64, f64, f64, f64) {
        let bench_name = benchmark_name!();
        let wasm = include_bytes!(benchmark_file!());

        let mut store = Store::default();


        store.link_func("env",  "initialise_board", 0, 0).unwrap();
        store.link_func("env",  "start_trigger", 0, 0).unwrap();
        store.link_func("env",  "stop_trigger", 0, 0).unwrap();

        let wasm = prepare(wasm).unwrap();
        let module = Module::new(&wasm).unwrap();

        // Allocate Memory for the module, I don't know how much memory it needs so i'm going to give it 64KiB
        // For some reason the memory needs to be 16-aligned so enforce this
        let layout = Layout::from_size_align(2 * 64 * 1024, 16).unwrap();
        let mut memory = unsafe { core::slice::from_raw_parts_mut(alloc_zeroed(layout), 2 * 64 * 1024) };

        let inst = store.instantiate(module, &mut memory).unwrap();

        let mut times_to_run = Vec::new();
        let mut start = Instant::now();
        let mut stop = Instant::now();
        for i in 1..=BENCHMARK_LOOPS {
            debug!("Run {}", i);
            let mut result = store.invoke(inst, "__original_main", vec![]).unwrap();
            loop {
                let call = match result {
                    RunResult::Done(correct) => {
                        match correct.first() {
                            Some(Val::I32(0)) => {
                                times_to_run.push((stop - start).as_millis());
                            },
                            _ => {
                                error!("Benchmarking went wrong for some reason, aborting");
                                return (0_f64, 0_f64, 0_f64, 0_f64);
                            }
                        }
                        break;
                    },
                    RunResult::Host(call) => call,
                };
                match call.index() {
                    // Initialise_board, does nothing
                    0 => {},
                    // start_trigger
                    1 => {
                        start = Instant::now();
                    }
                    // stop_triger
                    2 => {
                        stop = Instant::now();
                    }
                    _ => unreachable!(),
                }
                result = call.resume(&[]).unwrap();
            }
        }

        let mut geo_mean = 1_f64;
        let mut times_geo_mean = 1_f64;
        let score_to_div = BENCH_SCORE.iter().find(|(b_name, _)| *b_name == bench_name).unwrap().1;

        for dur in times_to_run.iter() {
            let normalized_speed = score_to_div as f64 / *dur as f64;
            geo_mean *= pow(normalized_speed as f64, 1_f64/BENCHMARK_LOOPS as f64);
            times_geo_mean *= pow(*dur as f64, 1_f64/BENCHMARK_LOOPS as f64);
        }

        // sigma = exp( sqrt( 1/N sum( ln ( A_i / mean )^2 ) ) ) https://en.wikipedia.org/wiki/Geometric_standard_deviation
        let mut times_geo_std = 0_f64;
        let mut geo_std = 0_f64;
        for dur in times_to_run.iter() {
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
}