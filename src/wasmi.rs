use wasmi::{Caller, Config, Engine, Linker, Module, Store};

#[cfg(feature = "coremark")]
pub fn run_coremark() -> f32 {

    let wasm = include_bytes!(crate::benchmark_file!());

    let config = Config::default();

    // config.floats(false);

    let engine = Engine::new(&config);
    let mut store = Store::new(&engine, ());

    let module = unsafe { Module::new_unchecked(&engine, wasm).unwrap() };

    let mut linker = Linker::new(&engine);

    linker.func_wrap("env", "clock_ms", |_: Caller<'_, _>| { ariel_os::time::Instant::now().as_millis() }).unwrap();

    let instance = linker.instantiate_and_start(&mut store, &module).unwrap();

    instance.get_typed_func::<(), f32>(&mut store, "run").unwrap()
        .call(&mut store, ()).unwrap()
}

#[cfg(feature = "embench-1")]
pub mod embench1 {
    use ariel_os::time::Instant;
    use libm::{pow, exp, log, sqrt};
    use ariel_os::debug::log::{debug, error};

    use super::*;
    use crate::{BENCH_SCORE, BENCHMARK_LOOPS, benchmark_name, benchmark_file};

    extern crate alloc;
    use alloc::vec::Vec;

    pub fn run_bench() -> (f64, f64, f64, f64) {
        let bench_name = benchmark_name!();
        let wasm = include_bytes!(benchmark_file!());

        let config = Config::default();

        let engine = Engine::new(&config);

        let mut store = Store::new(&engine, (Instant::now(), Instant::now()));

        let module = unsafe { Module::new_unchecked(&engine, wasm).unwrap() };

        let mut linker = Linker::new(&engine);

        linker.func_wrap("env", "initialise_board", || {}).unwrap();
        linker.func_wrap("env", "start_trigger", |mut c: Caller<'_, (Instant, Instant)>| {
            c.data_mut().0 = Instant::now();
        }).unwrap();
        linker.func_wrap("env", "stop_trigger", |mut c: Caller<'_, (Instant, Instant)>| {
            c.data_mut().1 = Instant::now();
        }).unwrap();

        let instance = linker.instantiate_and_start(&mut store, &module).unwrap();

        let mut times_to_run = Vec::new();

        debug!("Starting wasm app");
        for i in 1..=BENCHMARK_LOOPS {
            debug!("Run {}", i);
            let correct = instance.get_typed_func::<(), u32>(&mut store, "__original_main").unwrap()
                .call(&mut store, ()).unwrap();
            match correct {
                0 => {
                    let &(start, end) = store.data();
                    // The relative speed is in milli seconds
                    times_to_run.push((end - start).as_millis());
                },
                _ => {
                    error!("Benchmarking went wrong for some reason, aborting");
                    return (0_f64, 0_f64, 0_f64, 0_f64);
                }
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